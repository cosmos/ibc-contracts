// SPDX-License-Identifier: Apache-2.0

use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
    time::UNIX_EPOCH,
};

use alloy::{
    network::Ethereum,
    primitives::{hex, Address, Bytes, B256, U256},
    providers::{Provider, RootProvider},
    rpc::types::EIP1186StorageProof,
    sol_types::{SolCall, SolValue},
};
use alloy_rlp::Header as RlpHeader;
use anyhow::{anyhow, bail, Context, Result};
use ethereum_apis::eth_api::client::EthApiClient;
use ethereum_light_client::membership::evm_ics26_commitment_path;
use ibc_eureka_solidity_types::{
    besu::{besu_ibft2_light_client, besu_qbft_light_client},
    ics26::{
        router::{multicallCall, routerCalls, routerInstance, updateClientCall},
        IICS02ClientMsgs::Height as RouterHeight,
        ICS26_IBC_STORAGE_SLOT,
    },
    msgs::{IBesuLightClientMsgs, IICS02ClientMsgs::Height as MsgHeight},
};
use proof_api_lib::{
    events::EurekaEventWithHeight,
    utils::{
        eth_eureka::{src_events_to_recv_and_ack_msgs, target_events_to_timeout_msgs},
        RelayEventsParams,
    },
};
use rlp::Rlp;

use crate::BesuConsensusType;

pub struct TxBuilder {
    src_provider: RootProvider,
    dst_provider: RootProvider,
    src_ics26_router: routerInstance<RootProvider, Ethereum>,
    dst_ics26_router: routerInstance<RootProvider, Ethereum>,
    consensus_type: BesuConsensusType,
}

struct CreateClientParams {
    trusting_period: u64,
    max_clock_drift: u64,
    trusted_height: Option<u64>,
    role_manager: Address,
}

impl TxBuilder {
    pub fn new(
        src_provider: RootProvider,
        dst_provider: RootProvider,
        src_ics26_address: Address,
        dst_ics26_address: Address,
        consensus_type: BesuConsensusType,
    ) -> Self {
        Self {
            src_ics26_router: routerInstance::new(src_ics26_address, src_provider.clone()),
            dst_ics26_router: routerInstance::new(dst_ics26_address, dst_provider.clone()),
            src_provider,
            dst_provider,
            consensus_type,
        }
    }

    pub const fn ics26_router_address(&self) -> &Address {
        self.dst_ics26_router.address()
    }

    pub async fn create_client(&self, parameters: &HashMap<String, String>) -> Result<Vec<u8>> {
        let params = parse_create_client_params(parameters)?;
        let trusted_height = match params.trusted_height {
            Some(height) => height,
            None => self
                .src_provider
                .get_block_number()
                .await
                .context("failed to fetch latest source block number")?,
        };

        let block = EthApiClient::new(self.src_provider.clone())
            .get_block(trusted_height)
            .await
            .with_context(|| format!("failed to fetch source block at height {trusted_height}"))?;
        let header = block.into_consensus_header();
        let header_rlp = alloy_rlp::encode(&header);
        let validators =
            extract_validators_from_header_extra_data(&header_rlp).with_context(|| {
                format!("failed to extract validators from source block {trusted_height}")
            })?;
        let (storage_root, _) = self
            .fetch_source_account_proof(trusted_height)
            .await
            .with_context(|| {
                format!(
                    "failed to fetch account proof for source router at height {trusted_height}"
                )
            })?;

        let calldata = match self.consensus_type {
            BesuConsensusType::Qbft => besu_qbft_light_client::BesuQBFTLightClient::deploy_builder(
                self.dst_provider.clone(),
                *self.src_ics26_router.address(),
                trusted_height,
                header.timestamp,
                storage_root,
                validators,
                params.trusting_period,
                params.max_clock_drift,
                params.role_manager,
            )
            .calldata()
            .to_vec(),
            BesuConsensusType::Ibft2 => {
                besu_ibft2_light_client::BesuIBFT2LightClient::deploy_builder(
                    self.dst_provider.clone(),
                    *self.src_ics26_router.address(),
                    trusted_height,
                    header.timestamp,
                    storage_root,
                    validators,
                    params.trusting_period,
                    params.max_clock_drift,
                    params.role_manager,
                )
                .calldata()
                .to_vec()
            }
        };

        Ok(calldata)
    }

    pub async fn update_client(&self, dst_client_id: &str) -> Result<Vec<u8>> {
        let client_state = self
            .fetch_destination_client_state(dst_client_id)
            .await
            .with_context(|| {
                format!("failed to decode destination Besu client state for client {dst_client_id}")
            })?;
        let trusted_height = client_state.latestHeight.revisionHeight;
        let target_height = self
            .src_provider
            .get_block_number()
            .await
            .context("failed to fetch latest source block number")?;
        let header_rlp = self
            .fetch_source_header_rlp(target_height)
            .await
            .with_context(|| format!("failed to fetch source block at height {target_height}"))?;
        let (_, account_proof) = self
            .fetch_source_account_proof(target_height)
            .await
            .with_context(|| {
                format!("failed to fetch account proof for source router at height {target_height}")
            })?;

        Ok(Self::build_update_client_calldata(
            dst_client_id,
            trusted_height,
            header_rlp,
            account_proof,
        ))
    }

    pub async fn relay_events(&self, params: RelayEventsParams) -> Result<Vec<u8>> {
        let proof_height = select_proof_height(&params.src_events, params.timeout_relay_height)?;
        let proof_block = EthApiClient::new(self.src_provider.clone())
            .get_block(proof_height)
            .await
            .with_context(|| format!("failed to fetch source block at height {proof_height}"))?;
        let proof_timestamp = proof_block.header.timestamp;
        let header_rlp = alloy_rlp::encode(proof_block.into_consensus_header());
        let now = std::time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .context("failed to read system time")?
            .as_secs();
        let proof_height_msg = RouterHeight {
            revisionNumber: 0,
            revisionHeight: proof_height,
        };

        let recv_and_ack_msgs = src_events_to_recv_and_ack_msgs(
            params.src_events,
            &params.src_client_id,
            &params.dst_client_id,
            &params.src_packet_seqs,
            &params.dst_packet_seqs,
            &proof_height_msg,
            now,
        )?;
        let timeout_msgs = target_events_to_timeout_msgs(
            params.target_events,
            &params.src_client_id,
            &params.dst_client_id,
            &params.dst_packet_seqs,
            &proof_height_msg,
            proof_timestamp,
        );

        let mut packet_calls: Vec<_> = recv_and_ack_msgs.into_iter().chain(timeout_msgs).collect();
        if packet_calls.is_empty() {
            bail!("no packets collected")
        }

        let client_state = self
            .fetch_destination_client_state(&params.dst_client_id)
            .await
            .with_context(|| {
                format!(
                    "failed to decode destination Besu client state for client {}",
                    params.dst_client_id
                )
            })?;
        let mut storage_keys = packet_calls
            .iter()
            .map(packet_storage_key)
            .collect::<Vec<_>>();
        storage_keys.sort_unstable();
        storage_keys.dedup();
        let (_, account_proof, storage_proofs) = self
            .fetch_source_proofs(proof_height, &storage_keys)
            .await
            .with_context(|| {
                format!(
                    "failed to fetch account and storage proofs for source router at height {proof_height}"
                )
            })?;
        let update_call = Self::build_update_client_calldata(
            &params.dst_client_id,
            client_state.latestHeight.revisionHeight,
            header_rlp,
            account_proof,
        );

        attach_packet_proofs(&mut packet_calls, &storage_proofs)?;

        let all_calls: Vec<Bytes> = std::iter::once(update_call.into())
            .chain(packet_calls.into_iter().map(|call| match call {
                routerCalls::ackPacket(call) => call.abi_encode().into(),
                routerCalls::recvPacket(call) => call.abi_encode().into(),
                routerCalls::timeoutPacket(call) => call.abi_encode().into(),
                _ => unreachable!("only recv, ack, and timeout calls are constructed"),
            }))
            .collect();

        Ok(multicallCall { data: all_calls }.abi_encode())
    }

    fn build_update_client_calldata(
        dst_client_id: &str,
        trusted_height: u64,
        header_rlp: Vec<u8>,
        account_proof: Vec<u8>,
    ) -> Vec<u8> {
        let update_msg = IBesuLightClientMsgs::MsgUpdateClient {
            headerRlp: header_rlp.into(),
            trustedHeight: MsgHeight {
                revisionNumber: 0,
                revisionHeight: trusted_height,
            },
            accountProof: account_proof.into(),
        };

        updateClientCall {
            clientId: dst_client_id.to_string(),
            updateMsg: update_msg.abi_encode().into(),
        }
        .abi_encode()
    }

    async fn fetch_source_header_rlp(&self, block_height: u64) -> Result<Vec<u8>> {
        let block = EthApiClient::new(self.src_provider.clone())
            .get_block(block_height)
            .await
            .with_context(|| format!("failed to fetch source block at height {block_height}"))?;
        Ok(alloy_rlp::encode(block.into_consensus_header()))
    }

    async fn fetch_source_account_proof(&self, block_height: u64) -> Result<(B256, Vec<u8>)> {
        let (storage_hash, account_proof, _) = self.fetch_source_proofs(block_height, &[]).await?;
        Ok((storage_hash, account_proof))
    }

    async fn fetch_source_proofs(
        &self,
        block_height: u64,
        storage_keys: &[B256],
    ) -> Result<(B256, Vec<u8>, HashMap<B256, Vec<u8>>)> {
        let proof = EthApiClient::new(self.src_provider.clone())
            .get_proof(
                &self.src_ics26_router.address().to_string(),
                storage_keys
                    .iter()
                    .map(|key| format!("0x{}", hex::encode(key)))
                    .collect(),
                format!("0x{block_height:x}"),
            )
            .await?;
        let storage_proofs = map_storage_proofs(storage_keys, proof.storage_proof)?;

        Ok((
            proof.storage_hash,
            encode_rlp_node_list(&proof.account_proof),
            storage_proofs,
        ))
    }

    async fn fetch_destination_client_state(
        &self,
        dst_client_id: &str,
    ) -> Result<IBesuLightClientMsgs::ClientState> {
        let client_address = self
            .dst_ics26_router
            .getClient(dst_client_id.to_string())
            .call()
            .await
            .with_context(|| {
                format!("failed to fetch destination client address for {dst_client_id}")
            })?;
        let client_state_bz: Bytes = besu_qbft_light_client::BesuQBFTLightClient::new(
            client_address,
            self.dst_provider.clone(),
        )
        .getClientState()
        .call()
        .await
        .with_context(|| format!("failed to fetch destination client state for {dst_client_id}"))?;

        IBesuLightClientMsgs::ClientState::abi_decode(client_state_bz.as_ref()).with_context(|| {
            format!("failed to decode destination client state for {dst_client_id}")
        })
    }
}

fn packet_storage_key(call: &routerCalls) -> B256 {
    let path = match call {
        routerCalls::recvPacket(call) => call.msg_.packet.commitment_path(),
        routerCalls::ackPacket(call) => call.msg_.packet.ack_commitment_path(),
        routerCalls::timeoutPacket(call) => call.msg_.packet.receipt_commitment_path(),
        _ => unreachable!("only recv, ack, and timeout calls are constructed"),
    };
    evm_ics26_commitment_path(&path, U256::from_be_slice(&ICS26_IBC_STORAGE_SLOT)).into()
}

fn attach_packet_proofs(
    packet_calls: &mut [routerCalls],
    storage_proofs: &HashMap<B256, Vec<u8>>,
) -> Result<()> {
    for call in packet_calls {
        let storage_key = packet_storage_key(call);
        let proof: Bytes = storage_proofs
            .get(&storage_key)
            .ok_or_else(|| anyhow!("missing storage proof for key {storage_key}"))?
            .clone()
            .into();
        match call {
            routerCalls::recvPacket(call) => call.msg_.proofCommitment = proof,
            routerCalls::ackPacket(call) => call.msg_.proofAcked = proof,
            routerCalls::timeoutPacket(call) => call.msg_.proofTimeout = proof,
            _ => unreachable!("only recv, ack, and timeout calls are constructed"),
        }
    }
    Ok(())
}

fn map_storage_proofs(
    expected_keys: &[B256],
    storage_proofs: Vec<EIP1186StorageProof>,
) -> Result<HashMap<B256, Vec<u8>>> {
    let expected_keys = expected_keys.iter().copied().collect::<HashSet<_>>();
    let mut proofs = HashMap::with_capacity(expected_keys.len());

    for storage_proof in storage_proofs {
        let key = storage_proof.key.as_b256();
        if !expected_keys.contains(&key) {
            bail!("unexpected storage proof key {key}");
        }
        if proofs
            .insert(key, encode_rlp_node_list(&storage_proof.proof))
            .is_some()
        {
            bail!("duplicate storage proof key {key}");
        }
    }

    if let Some(key) = expected_keys.iter().find(|key| !proofs.contains_key(*key)) {
        bail!("missing storage proof for key {key}");
    }
    Ok(proofs)
}

fn parse_create_client_params(parameters: &HashMap<String, String>) -> Result<CreateClientParams> {
    for key in parameters.keys() {
        if !matches!(
            key.as_str(),
            "trusting_period" | "max_clock_drift" | "trusted_height" | "role_manager"
        ) {
            bail!(
                "unexpected parameter `{key}`, only `trusting_period`, `max_clock_drift`, `trusted_height`, and `role_manager` are allowed"
            );
        }
    }

    Ok(CreateClientParams {
        trusting_period: parameters
            .get("trusting_period")
            .ok_or_else(|| anyhow!("missing `trusting_period` parameter"))?
            .parse()
            .context("failed to parse `trusting_period` as decimal seconds")?,
        max_clock_drift: parameters
            .get("max_clock_drift")
            .ok_or_else(|| anyhow!("missing `max_clock_drift` parameter"))?
            .parse()
            .context("failed to parse `max_clock_drift` as decimal seconds")?,
        trusted_height: parameters
            .get("trusted_height")
            .map(|value| {
                value
                    .parse()
                    .context("failed to parse `trusted_height` as decimal block height")
            })
            .transpose()?,
        role_manager: parameters
            .get("role_manager")
            .map_or(Ok(Address::ZERO), |value| {
                Address::from_str(value).context("failed to parse `role_manager` as hex address")
            })?,
    })
}

fn encode_rlp_node_list(nodes: &[Bytes]) -> Vec<u8> {
    let payload_length = nodes.iter().map(|node| node.len()).sum();
    let mut encoded = Vec::new();
    RlpHeader {
        list: true,
        payload_length,
    }
    .encode(&mut encoded);
    for node in nodes {
        encoded.extend_from_slice(node.as_ref());
    }
    encoded
}

fn extract_validators_from_header_extra_data(header_rlp: &[u8]) -> Result<Vec<Address>> {
    let header = Rlp::new(header_rlp);
    let extra_data = header
        .at(12)
        .context("failed to read header extraData field")?
        .data()
        .context("failed to decode header extraData bytes")?;
    let extra_data = Rlp::new(extra_data);
    let validators = extra_data
        .at(1)
        .context("failed to read validator list from extraData")?;

    let mut out = Vec::with_capacity(
        validators
            .item_count()
            .context("failed to read validator count")?,
    );
    for validator in &validators {
        let validator = validator
            .data()
            .context("failed to decode validator address")?;
        if validator.len() != 20 {
            bail!("invalid validator address length: {}", validator.len());
        }
        out.push(Address::from_slice(validator));
    }
    Ok(out)
}

fn select_proof_height(
    src_events: &[EurekaEventWithHeight],
    timeout_relay_height: Option<u64>,
) -> Result<u64> {
    match (
        src_events.iter().map(|event| event.height).max(),
        timeout_relay_height,
    ) {
        (Some(src_height), Some(timeout_height)) => Ok(src_height.max(timeout_height)),
        (Some(src_height), None) => Ok(src_height),
        (None, Some(timeout_height)) => Ok(timeout_height),
        (None, None) => bail!("no packets collected"),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        encode_rlp_node_list, extract_validators_from_header_extra_data, map_storage_proofs,
        select_proof_height,
    };
    use alloy::{
        primitives::{hex, Address, Bytes, B256, U256},
        rpc::types::EIP1186StorageProof,
    };
    use ibc_eureka_solidity_types::ics26::IICS26RouterMsgs::{Packet, Payload};
    use proof_api_lib::events::{EurekaEvent, EurekaEventWithHeight};
    use serde::Deserialize;
    use std::str::FromStr;

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Fixture {
        update_height12: UpdateFixture,
        membership: ProofFixture,
        non_membership: ProofFixture,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct UpdateFixture {
        header_rlp: String,
        account_proof: String,
        expected_validators: Vec<String>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct ProofFixture {
        proof: String,
    }

    fn load_fixture() -> Fixture {
        serde_json::from_str(include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../../ibc-solidity/test/besu-bft/fixtures/qbft.json"
        )))
        .unwrap()
    }

    fn decode_hex_bytes(hex_value: &str) -> Vec<u8> {
        hex::decode(hex_value.trim_start_matches("0x")).unwrap()
    }

    fn decode_proof_nodes(proof_rlp: &[u8]) -> Vec<Bytes> {
        let proof = rlp::Rlp::new(proof_rlp);
        proof
            .iter()
            .map(|node| Bytes::copy_from_slice(node.as_raw()))
            .collect()
    }

    fn packet(sequence: u64) -> Packet {
        Packet {
            sequence,
            sourceClient: "src-client".to_string(),
            destClient: "dst-client".to_string(),
            timeoutTimestamp: u64::MAX,
            payloads: vec![Payload {
                sourcePort: "transfer".to_string(),
                destPort: "transfer".to_string(),
                version: "ics20-1".to_string(),
                encoding: "abi".to_string(),
                value: Bytes::default(),
            }],
        }
    }

    #[test]
    fn extracts_validators_from_fixture_header() {
        let fixture = load_fixture();
        let header_rlp = decode_hex_bytes(&fixture.update_height12.header_rlp);
        let validators = extract_validators_from_header_extra_data(&header_rlp).unwrap();
        let expected = fixture
            .update_height12
            .expected_validators
            .iter()
            .map(|address| Address::from_str(address).unwrap())
            .collect::<Vec<_>>();

        assert_eq!(validators, expected);
    }

    #[test]
    fn packs_account_proof_nodes_without_reencoding_nodes() {
        let fixture = load_fixture();
        let account_proof_rlp = decode_hex_bytes(&fixture.update_height12.account_proof);
        let nodes = decode_proof_nodes(&account_proof_rlp);

        assert_eq!(encode_rlp_node_list(&nodes), account_proof_rlp);
    }

    #[test]
    fn packs_membership_and_non_membership_storage_proofs() {
        let fixture = load_fixture();
        let membership_proof = decode_hex_bytes(&fixture.membership.proof);
        let non_membership_proof = decode_hex_bytes(&fixture.non_membership.proof);

        let membership_nodes = decode_proof_nodes(&membership_proof);
        let non_membership_nodes = decode_proof_nodes(&non_membership_proof);

        assert_eq!(encode_rlp_node_list(&membership_nodes), membership_proof);
        assert_eq!(
            encode_rlp_node_list(&non_membership_nodes),
            non_membership_proof
        );
    }

    #[test]
    fn maps_storage_proofs_by_normalized_key_and_validates_the_response() {
        let first = B256::from(U256::from(1));
        let second = B256::from(U256::from(2));
        let make_proof = |key, marker| EIP1186StorageProof {
            key,
            value: U256::ZERO,
            proof: vec![Bytes::from(vec![marker])],
        };

        let mapped = map_storage_proofs(
            &[first, second, first],
            vec![
                make_proof(second.into(), 2),
                make_proof(U256::from(1).into(), 1),
            ],
        )
        .unwrap();
        assert_eq!(
            mapped[&first],
            encode_rlp_node_list(&[Bytes::from(vec![1])])
        );
        assert_eq!(
            mapped[&second],
            encode_rlp_node_list(&[Bytes::from(vec![2])])
        );

        assert!(map_storage_proofs(&[first], vec![])
            .unwrap_err()
            .to_string()
            .contains("missing storage proof"));
        assert!(map_storage_proofs(
            &[first],
            vec![make_proof(first.into(), 1), make_proof(first.into(), 1)]
        )
        .unwrap_err()
        .to_string()
        .contains("duplicate storage proof"));
        assert!(
            map_storage_proofs(&[first], vec![make_proof(second.into(), 2)])
                .unwrap_err()
                .to_string()
                .contains("unexpected storage proof")
        );
    }

    #[test]
    fn selects_max_proof_height() {
        let src_events = vec![
            EurekaEventWithHeight {
                event: EurekaEvent::SendPacket(packet(3)),
                height: 10,
            },
            EurekaEventWithHeight {
                event: EurekaEvent::WriteAcknowledgement(
                    packet(4),
                    vec![Bytes::from(vec![b'a', b'c', b'k'])],
                ),
                height: 12,
            },
        ];

        assert_eq!(select_proof_height(&src_events, None).unwrap(), 12);
        assert_eq!(select_proof_height(&[], Some(15)).unwrap(), 15);
        assert_eq!(select_proof_height(&src_events, Some(11)).unwrap(), 12);
        assert_eq!(select_proof_height(&src_events, Some(14)).unwrap(), 14);
    }
}
