use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};

use besu_qbft::{parse_extra_data, validate_optional_tail};
use besu_zk_types::{Eip1186ProofWitness, QbftHeaderInput};
use reqwest::Url;
use serde_json::{json, Map, Value};

use crate::ToolError;

#[derive(Clone)]
pub struct RpcClient {
    endpoint: Url,
    http: reqwest::Client,
    settings: RpcSettings,
    next_id: Arc<AtomicU64>,
}

#[derive(Clone, Debug)]
pub struct RpcSettings {
    pub timeout: Duration,
    pub max_attempts: usize,
    pub retry_backoff: Duration,
    pub max_response_bytes: usize,
}

impl Default for RpcSettings {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(3),
            max_attempts: 2,
            retry_backoff: Duration::from_millis(100),
            max_response_bytes: 16 * 1024 * 1024,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConsensusState {
    pub state_root: [u8; 32],
    pub block_timestamp: u64,
    pub validator_set_hash: [u8; 32],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchEip1186Proof {
    pub account_proof: Vec<Vec<u8>>,
    pub storage_proofs: Vec<Vec<Vec<u8>>>,
}

impl RpcClient {
    pub fn new(endpoint: &str, timeout: Duration) -> Result<Self, ToolError> {
        Self::with_settings(
            endpoint,
            RpcSettings {
                timeout,
                ..RpcSettings::default()
            },
        )
    }

    pub fn with_settings(endpoint: &str, settings: RpcSettings) -> Result<Self, ToolError> {
        let endpoint = Url::parse(endpoint).map_err(|_| ToolError::InvalidRpcEndpoint)?;
        if !matches!(endpoint.scheme(), "http" | "https")
            || endpoint.host_str().is_none()
            || !endpoint.username().is_empty()
            || endpoint.password().is_some()
            || settings.max_attempts == 0
            || settings.max_attempts > 10
            || settings.max_response_bytes == 0
        {
            return Err(ToolError::InvalidRpcEndpoint);
        }
        let origin = endpoint.clone();
        let http = reqwest::Client::builder()
            .timeout(settings.timeout)
            .redirect(reqwest::redirect::Policy::custom(move |attempt| {
                if attempt.previous().len() >= 5 || attempt.url().origin() != origin.origin() {
                    attempt.error("RPC redirect rejected")
                } else {
                    attempt.follow()
                }
            }))
            .build()
            .map_err(|_| ToolError::InvalidRpcEndpoint)?;
        Ok(Self {
            endpoint,
            http,
            settings,
            next_id: Arc::new(AtomicU64::new(1)),
        })
    }

    pub async fn call(&self, method: &'static str, params: Value) -> Result<Value, ToolError> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        for attempt in 0..self.settings.max_attempts {
            let result = self.call_once(method, &params, id).await;
            let retry = matches!(
                &result,
                Err(ToolError::RpcTransport | ToolError::RpcHttpStatus(429 | 502 | 503 | 504))
            );
            if !retry || attempt + 1 == self.settings.max_attempts {
                return result;
            }
            tokio::time::sleep(self.settings.retry_backoff).await;
        }
        Err(ToolError::RpcTransport)
    }

    async fn call_once(
        &self,
        method: &'static str,
        params: &Value,
        id: u64,
    ) -> Result<Value, ToolError> {
        let mut response = self
            .http
            .post(self.endpoint.clone())
            .json(&json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}))
            .send()
            .await
            .map_err(|e| transport_error(e.is_timeout()))?;
        if !response.status().is_success() {
            return Err(ToolError::RpcHttpStatus(response.status().as_u16()));
        }
        let mut bytes = Vec::new();
        while let Some(chunk) = response
            .chunk()
            .await
            .map_err(|e| transport_error(e.is_timeout()))?
        {
            if chunk.len() > self.settings.max_response_bytes.saturating_sub(bytes.len()) {
                return Err(ToolError::RpcResponseTooLarge);
            }
            bytes.extend_from_slice(&chunk);
        }
        let body = serde_json::from_slice(&bytes).map_err(|_| ToolError::MalformedRpcResponse)?;
        parse_response_id(&body, id)
    }

    pub async fn chain_id(&self) -> Result<u64, ToolError> {
        let value = self.call("eth_chainId", json!([])).await?;
        parse_quantity_u64(&value, "eth_chainId")
    }

    pub async fn block(&self, height: u64) -> Result<QbftHeaderInput, ToolError> {
        let value = self
            .call("eth_getBlockByNumber", json!([quantity(height), false]))
            .await?;
        parse_block(&value, height)
    }

    pub async fn proof(
        &self,
        address: [u8; 20],
        slot: [u8; 32],
        height: u64,
    ) -> Result<Eip1186ProofWitness, ToolError> {
        let batch = self.proofs(address, &[slot], height).await?;
        let storage_proof = batch
            .storage_proofs
            .into_iter()
            .next()
            .ok_or(ToolError::WrongStorageProofCount(0))?;
        Ok(Eip1186ProofWitness {
            account_proof: batch.account_proof,
            storage_proof,
        })
    }

    pub async fn proofs(
        &self,
        address: [u8; 20],
        slots: &[[u8; 32]],
        height: u64,
    ) -> Result<BatchEip1186Proof, ToolError> {
        let slot_params: Vec<String> = slots
            .iter()
            .map(|slot| format!("0x{}", hex::encode(slot)))
            .collect();
        let value = self
            .call(
                "eth_getProof",
                json!([
                    format!("0x{}", hex::encode(address)),
                    slot_params,
                    quantity(height)
                ]),
            )
            .await?;
        parse_batch_proof(&value, address, slots)
    }

    pub async fn consensus_state(
        &self,
        light_client: [u8; 20],
        height: u64,
    ) -> Result<ConsensusState, ToolError> {
        let value = self
            .call(
                "eth_call",
                json!([{
                    "to": format!("0x{}", hex::encode(light_client)),
                    "data": format!("0x{}", hex::encode(consensus_state_calldata(height)?))
                }, "latest"]),
            )
            .await?;
        let encoded = parse_hex_value(&value, "eth_call result")?;
        parse_consensus_state(&encoded, height)
    }
}

fn transport_error(timed_out: bool) -> ToolError {
    if timed_out {
        ToolError::RpcTimeout
    } else {
        ToolError::RpcTransport
    }
}

#[must_use]
pub fn quantity(value: u64) -> String {
    format!("0x{value:x}")
}

pub fn parse_rpc_response(body: &Value) -> Result<Value, ToolError> {
    parse_response_id(body, 1)
}

fn parse_response_id(body: &Value, id: u64) -> Result<Value, ToolError> {
    let object = body.as_object().ok_or(ToolError::MalformedRpcResponse)?;
    if object.get("jsonrpc").and_then(Value::as_str) != Some("2.0")
        || object.get("id").and_then(Value::as_u64) != Some(id)
        || (object.get("error").is_some_and(|v| !v.is_null())
            && object.get("result").is_some_and(|v| !v.is_null()))
    {
        return Err(ToolError::MalformedRpcResponse);
    }
    if let Some(error) = object.get("error").filter(|v| !v.is_null()) {
        let error = error.as_object().ok_or(ToolError::MalformedRpcResponse)?;
        let code = error
            .get("code")
            .and_then(Value::as_i64)
            .ok_or(ToolError::MalformedRpcResponse)?;
        let message = error
            .get("message")
            .and_then(Value::as_str)
            .ok_or(ToolError::MalformedRpcResponse)?
            .to_ascii_lowercase();
        if [
            "historical state",
            "state not available",
            "missing trie node",
            "pruned",
            "archive",
        ]
        .iter()
        .any(|needle| message.contains(needle))
        {
            return Err(ToolError::HistoricalStateUnavailable);
        }
        return Err(ToolError::RpcRejected(code));
    }
    object
        .get("result")
        .cloned()
        .ok_or(ToolError::MalformedRpcResponse)
}

pub fn parse_block(value: &Value, expected_number: u64) -> Result<QbftHeaderInput, ToolError> {
    if value.is_null() {
        return Err(ToolError::NullResult("eth_getBlockByNumber"));
    }
    let block = value.as_object().ok_or(ToolError::WrongJsonType("block"))?;
    reject_unsupported_block_fields(block)?;
    let number = required_quantity_u64(block, "number")?;
    if number != expected_number {
        return Err(ToolError::WrongBlockNumber {
            expected: expected_number,
            actual: number,
        });
    }
    let header = QbftHeaderInput {
        parent_hash: required_fixed(block, "parentHash")?,
        ommers_hash: required_fixed(block, "sha3Uncles")?,
        beneficiary: required_fixed(block, "miner")?,
        state_root: required_fixed(block, "stateRoot")?,
        transactions_root: required_fixed(block, "transactionsRoot")?,
        receipts_root: required_fixed(block, "receiptsRoot")?,
        logs_bloom: required_fixed(block, "logsBloom")?,
        difficulty: required_quantity_word(block, "difficulty")?,
        number,
        gas_limit: required_quantity_u64(block, "gasLimit")?,
        gas_used: required_quantity_u64(block, "gasUsed")?,
        timestamp: required_quantity_u64(block, "timestamp")?,
        extra_data: required_hex(block, "extraData")?,
        mix_hash: required_fixed(block, "mixHash")?,
        nonce: required_fixed(block, "nonce")?,
        base_fee_per_gas: optional_quantity_word(block, "baseFeePerGas")?,
        withdrawals_root: optional_fixed(block, "withdrawalsRoot")?,
        blob_gas_used: optional_quantity_u64(block, "blobGasUsed")?,
        excess_blob_gas: optional_quantity_u64(block, "excessBlobGas")?,
        parent_beacon_block_root: optional_fixed(block, "parentBeaconBlockRoot")?,
        requests_hash: optional_fixed(block, "requestsHash")?,
        block_access_list_hash: optional_fixed(block, "blockAccessListHash")?,
    };
    validate_optional_tail(&header)
        .map_err(|error| ToolError::InvalidWitness(error.to_string()))?;
    parse_extra_data(&header.extra_data)
        .map_err(|error| ToolError::InvalidWitness(error.to_string()))?;
    Ok(header)
}

fn reject_unsupported_block_fields(block: &Map<String, Value>) -> Result<(), ToolError> {
    const PINNED_BESU_BLOCK_FIELDS: &[&str] = &[
        "baseFeePerGas",
        "blobGasUsed",
        "blockAccessListHash",
        "difficulty",
        "excessBlobGas",
        "extraData",
        "gasLimit",
        "gasUsed",
        "hash",
        "logsBloom",
        "miner",
        "mixHash",
        "nonce",
        "number",
        "parentBeaconBlockRoot",
        "parentHash",
        "receiptsRoot",
        "requestsHash",
        "sha3Uncles",
        "size",
        "stateRoot",
        "timestamp",
        "totalDifficulty",
        "transactions",
        "transactionsRoot",
        "uncles",
        "withdrawals",
        "withdrawalsRoot",
    ];
    if let Some(field) = block
        .keys()
        .find(|field| !PINNED_BESU_BLOCK_FIELDS.contains(&field.as_str()))
    {
        return Err(ToolError::UnsupportedBlockField(field.clone()));
    }
    Ok(())
}

pub fn parse_proof(
    value: &Value,
    expected_address: [u8; 20],
    expected_slot: [u8; 32],
) -> Result<Eip1186ProofWitness, ToolError> {
    let batch = parse_batch_proof(value, expected_address, &[expected_slot])?;
    let storage_proof = batch
        .storage_proofs
        .into_iter()
        .next()
        .ok_or(ToolError::WrongStorageProofCount(0))?;
    Ok(Eip1186ProofWitness {
        account_proof: batch.account_proof,
        storage_proof,
    })
}

pub fn parse_batch_proof(
    value: &Value,
    expected_address: [u8; 20],
    expected_slots: &[[u8; 32]],
) -> Result<BatchEip1186Proof, ToolError> {
    if value.is_null() {
        return Err(ToolError::NullResult("eth_getProof"));
    }
    let object = value
        .as_object()
        .ok_or(ToolError::WrongJsonType("eth_getProof result"))?;
    if required_fixed::<20>(object, "address")? != expected_address {
        return Err(ToolError::WrongProofAddress);
    }
    let account_proof = required_hex_array(object, "accountProof")?;
    if account_proof.is_empty() {
        return Err(ToolError::EmptyAccountProof);
    }
    validate_proof_nodes(&account_proof)?;
    let storage = required_array(object, "storageProof")?;
    if storage.len() != expected_slots.len() {
        return Err(ToolError::WrongStorageProofCount(storage.len()));
    }
    let mut by_key = std::collections::BTreeMap::new();
    for value in storage {
        let entry = value
            .as_object()
            .ok_or(ToolError::WrongJsonType("storageProof[]"))?;
        let key = parse_storage_key(required(entry, "key")?)?;
        if !expected_slots.contains(&key) || by_key.contains_key(&key) {
            return Err(ToolError::WrongStorageKey);
        }
        let proof = required_hex_array(entry, "proof")?;
        validate_proof_nodes(&proof)?;
        by_key.insert(key, proof);
    }
    let storage_proofs = expected_slots
        .iter()
        .map(|slot| by_key.remove(slot).ok_or(ToolError::WrongStorageKey))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(BatchEip1186Proof {
        account_proof,
        storage_proofs,
    })
}

fn validate_proof_nodes(nodes: &[Vec<u8>]) -> Result<(), ToolError> {
    for node in nodes {
        evm_mpt::rlp::decode_exact(node).map_err(|_| ToolError::MalformedProofNode)?;
    }
    Ok(())
}

pub fn parse_consensus_state(encoded: &[u8], height: u64) -> Result<ConsensusState, ToolError> {
    if encoded.len() != 128 {
        return Err(ToolError::MalformedConsensusState);
    }
    let words: [[u8; 32]; 4] = encoded
        .chunks_exact(32)
        .map(<[u8; 32]>::try_from)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| ToolError::MalformedConsensusState)?
        .try_into()
        .map_err(|_| ToolError::MalformedConsensusState)?;
    let [found_word, state_root, timestamp_word, validator_set_hash] = words;
    let found_prefix = found_word
        .get(..31)
        .ok_or(ToolError::MalformedConsensusState)?;
    let found = *found_word
        .last()
        .ok_or(ToolError::MalformedConsensusState)?;
    if found_prefix != [0; 31] || found > 1 {
        return Err(ToolError::MalformedConsensusState);
    }
    if found == 0 {
        return Err(ToolError::MissingConsensusState(height));
    }
    let timestamp_prefix = timestamp_word
        .get(..24)
        .ok_or(ToolError::MalformedConsensusState)?;
    if timestamp_prefix != [0; 24] {
        return Err(ToolError::MalformedConsensusState);
    }
    let timestamp_bytes: [u8; 8] = timestamp_word
        .get(24..)
        .ok_or(ToolError::MalformedConsensusState)?
        .try_into()
        .map_err(|_| ToolError::MalformedConsensusState)?;
    let block_timestamp = u64::from_be_bytes(timestamp_bytes);
    if block_timestamp == 0 || state_root == [0; 32] || validator_set_hash == [0; 32] {
        return Err(ToolError::MalformedConsensusState);
    }
    Ok(ConsensusState {
        state_root,
        block_timestamp,
        validator_set_hash,
    })
}

fn consensus_state_calldata(height: u64) -> Result<Vec<u8>, ToolError> {
    let selector = besu_zk_types::keccak256(b"getConsensusState(uint64)");
    let mut calldata = Vec::with_capacity(36);
    calldata.extend_from_slice(
        selector
            .get(..4)
            .ok_or(ToolError::MalformedConsensusState)?,
    );
    calldata.extend_from_slice(&[0; 24]);
    calldata.extend_from_slice(&height.to_be_bytes());
    Ok(calldata)
}

fn required<'a>(
    object: &'a Map<String, Value>,
    field: &'static str,
) -> Result<&'a Value, ToolError> {
    object.get(field).ok_or(ToolError::MissingField(field))
}

fn required_array<'a>(
    object: &'a Map<String, Value>,
    field: &'static str,
) -> Result<&'a Vec<Value>, ToolError> {
    required(object, field)?
        .as_array()
        .ok_or(ToolError::WrongJsonType(field))
}

fn required_hex(object: &Map<String, Value>, field: &'static str) -> Result<Vec<u8>, ToolError> {
    parse_hex_value(required(object, field)?, field)
}

fn required_hex_array(
    object: &Map<String, Value>,
    field: &'static str,
) -> Result<Vec<Vec<u8>>, ToolError> {
    required_array(object, field)?
        .iter()
        .map(|value| parse_hex_value(value, field))
        .collect()
}

fn parse_storage_key(value: &Value) -> Result<[u8; 32], ToolError> {
    let encoded = value.as_str().ok_or(ToolError::WrongJsonType("key"))?;
    let payload = encoded
        .strip_prefix("0x")
        .ok_or(ToolError::MalformedHex("key"))?;
    if payload.is_empty() || !payload.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(ToolError::MalformedHex("key"));
    }
    if payload.len() > 64 {
        return Err(ToolError::NumericOverflow("key"));
    }
    // Besu may omit leading zeroes from an EIP-1186 storage key.
    let padded = format!("{payload:0>64}");
    let mut key = [0_u8; 32];
    hex::decode_to_slice(padded, &mut key).map_err(|_| ToolError::MalformedHex("key"))?;
    Ok(key)
}

fn required_fixed<const N: usize>(
    object: &Map<String, Value>,
    field: &'static str,
) -> Result<[u8; N], ToolError> {
    let bytes = required_hex(object, field)?;
    let actual = bytes.len();
    bytes.try_into().map_err(|_| ToolError::WrongByteLength {
        field,
        expected: N,
        actual,
    })
}

fn optional_fixed<const N: usize>(
    object: &Map<String, Value>,
    field: &'static str,
) -> Result<Option<[u8; N]>, ToolError> {
    match object.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(_) => required_fixed(object, field).map(Some),
    }
}

fn required_quantity_u64(
    object: &Map<String, Value>,
    field: &'static str,
) -> Result<u64, ToolError> {
    parse_quantity_u64(required(object, field)?, field)
}

fn optional_quantity_u64(
    object: &Map<String, Value>,
    field: &'static str,
) -> Result<Option<u64>, ToolError> {
    match object.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => parse_quantity_u64(value, field).map(Some),
    }
}

fn required_quantity_word(
    object: &Map<String, Value>,
    field: &'static str,
) -> Result<[u8; 32], ToolError> {
    parse_quantity_word(required(object, field)?, field)
}

fn optional_quantity_word(
    object: &Map<String, Value>,
    field: &'static str,
) -> Result<Option<[u8; 32]>, ToolError> {
    match object.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => parse_quantity_word(value, field).map(Some),
    }
}

pub fn parse_hex_value(value: &Value, field: &'static str) -> Result<Vec<u8>, ToolError> {
    let encoded = value.as_str().ok_or(ToolError::WrongJsonType(field))?;
    variable_hex(encoded, field)
}

/// Parses a 0x-prefixed, byte-aligned hexadecimal string.
pub fn variable_hex(encoded: &str, field: &'static str) -> Result<Vec<u8>, ToolError> {
    let payload = encoded
        .strip_prefix("0x")
        .ok_or(ToolError::MalformedHex(field))?;
    if payload.len() % 2 != 0 || !payload.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(ToolError::MalformedHex(field));
    }
    hex::decode(payload).map_err(|_| ToolError::MalformedHex(field))
}

/// Parses a hexadecimal string containing exactly N bytes.
pub fn fixed_hex<const N: usize>(value: &str, field: &'static str) -> Result<[u8; N], ToolError> {
    let bytes = variable_hex(value, field)?;
    let actual = bytes.len();
    bytes.try_into().map_err(|_| ToolError::WrongByteLength {
        field,
        expected: N,
        actual,
    })
}

pub fn parse_quantity_u64(value: &Value, field: &'static str) -> Result<u64, ToolError> {
    let word = parse_quantity_bytes(value, field)?;
    if word.len() > 8 {
        return Err(ToolError::NumericOverflow(field));
    }
    let mut encoded = [0_u8; 8];
    let start = encoded
        .len()
        .checked_sub(word.len())
        .ok_or(ToolError::NumericOverflow(field))?;
    encoded
        .get_mut(start..)
        .ok_or(ToolError::NumericOverflow(field))?
        .copy_from_slice(&word);
    Ok(u64::from_be_bytes(encoded))
}

pub fn parse_quantity_word(value: &Value, field: &'static str) -> Result<[u8; 32], ToolError> {
    let bytes = parse_quantity_bytes(value, field)?;
    if bytes.len() > 32 {
        return Err(ToolError::NumericOverflow(field));
    }
    let mut word = [0_u8; 32];
    let start = word
        .len()
        .checked_sub(bytes.len())
        .ok_or(ToolError::NumericOverflow(field))?;
    word.get_mut(start..)
        .ok_or(ToolError::NumericOverflow(field))?
        .copy_from_slice(&bytes);
    Ok(word)
}

fn parse_quantity_bytes(value: &Value, field: &'static str) -> Result<Vec<u8>, ToolError> {
    let encoded = value.as_str().ok_or(ToolError::WrongJsonType(field))?;
    let payload = encoded
        .strip_prefix("0x")
        .ok_or(ToolError::MalformedHex(field))?;
    if !payload.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(ToolError::MalformedHex(field));
    }
    if payload.is_empty() || (payload.len() > 1 && payload.starts_with('0')) {
        return Err(ToolError::NonCanonicalQuantity(field));
    }
    let even = if payload.len() % 2 == 0 {
        payload.to_owned()
    } else {
        format!("0{payload}")
    };
    hex::decode(even).map_err(|_| ToolError::MalformedHex(field))
}

#[cfg(test)]
mod tests {
    use super::{consensus_state_calldata, transport_error};
    use crate::ToolError;

    #[test]
    fn transport_timeout_has_an_explicit_fail_closed_error() {
        assert!(matches!(transport_error(true), ToolError::RpcTimeout));
        assert!(matches!(transport_error(false), ToolError::RpcTransport));
    }

    #[test]
    fn consensus_state_call_is_exact_selector_plus_one_uint64_word() -> Result<(), ToolError> {
        let height = 0x0102_0304_0506_0708_u64;
        let calldata = consensus_state_calldata(height)?;
        assert_eq!(calldata.len(), 36);
        assert_eq!(
            &calldata[..4],
            &besu_zk_types::keccak256(b"getConsensusState(uint64)")[..4]
        );
        assert_eq!(&calldata[4..28], &[0; 24]);
        assert_eq!(&calldata[28..], &height.to_be_bytes());
        Ok(())
    }
}
