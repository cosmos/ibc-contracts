// SPDX-License-Identifier: Apache-2.0

use std::{
    collections::{HashMap, HashSet},
    str::FromStr,
    time::UNIX_EPOCH,
};

use alloy::{
    consensus::Header,
    network::Ethereum,
    primitives::{hex, Address, Bytes, B256, U256},
    providers::{Provider, RootProvider},
    rpc::types::{EIP1186AccountProofResponse, EIP1186StorageProof},
    sol_types::{SolCall, SolValue},
};
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
use proof_api_lib::utils::{
    eth_eureka::{src_events_to_recv_and_ack_msgs, target_events_to_timeout_msgs},
    RelayEventsParams,
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

const TRUSTING_PERIOD: &str = "trusting_period";
const MAX_CLOCK_DRIFT: &str = "max_clock_drift";
const TRUSTED_HEIGHT: &str = "trusted_height";
const ROLE_MANAGER: &str = "role_manager";

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

        let header = self
            .fetch_source_header(trusted_height)
            .await
            .with_context(|| format!("failed to fetch source block at height {trusted_height}"))?;
        let validators =
            extract_validators_from_extra_data(&header.extra_data).with_context(|| {
                format!("failed to extract validators from source block {trusted_height}")
            })?;
        let proof = self
            .fetch_source_proofs(trusted_height, &[])
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
                proof.storage_hash,
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
                    proof.storage_hash,
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
        let header = self
            .fetch_source_header(target_height)
            .await
            .with_context(|| format!("failed to fetch source block at height {target_height}"))?;
        let proof = self
            .fetch_source_proofs(target_height, &[])
            .await
            .with_context(|| {
                format!("failed to fetch account proof for source router at height {target_height}")
            })?;

        Ok(Self::build_update_client_calldata(
            dst_client_id,
            trusted_height,
            header,
            proof.account_proof.abi_encode(),
        ))
    }

    pub async fn relay_events(&self, params: RelayEventsParams) -> Result<Vec<u8>> {
        let proof_height = params
            .src_events
            .iter()
            .map(|event| event.height)
            .chain(params.timeout_relay_height)
            .max()
            .ok_or_else(|| anyhow!("no packets collected"))?;
        let header = self
            .fetch_source_header(proof_height)
            .await
            .with_context(|| format!("failed to fetch source block at height {proof_height}"))?;
        let proof_timestamp = header.timestamp;
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
        let proof = self
            .fetch_source_proofs(proof_height, &storage_keys)
            .await
            .with_context(|| {
                format!(
                    "failed to fetch account and storage proofs for source router at height {proof_height}"
                )
            })?;
        let account_proof = proof.account_proof.abi_encode();
        let storage_proofs = map_storage_proofs(&storage_keys, proof.storage_proof)?;
        let update_call = Self::build_update_client_calldata(
            &params.dst_client_id,
            client_state.latestHeight.revisionHeight,
            header,
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
        header: Header,
        account_proof: Vec<u8>,
    ) -> Vec<u8> {
        let update_msg = IBesuLightClientMsgs::MsgUpdateClient {
            headerRlp: alloy_rlp::encode(header).into(),
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

    async fn fetch_source_header(&self, block_height: u64) -> Result<Header> {
        let block = EthApiClient::new(self.src_provider.clone())
            .get_block(block_height)
            .await
            .with_context(|| format!("failed to fetch source block at height {block_height}"))?;
        Ok(block.into_consensus_header())
    }

    async fn fetch_source_proofs(
        &self,
        block_height: u64,
        storage_keys: &[B256],
    ) -> Result<EIP1186AccountProofResponse> {
        Ok(EthApiClient::new(self.src_provider.clone())
            .get_proof(
                &self.src_ics26_router.address().to_string(),
                storage_keys
                    .iter()
                    .map(|key| format!("0x{}", hex::encode(key)))
                    .collect(),
                format!("0x{block_height:x}"),
            )
            .await?)
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
            .insert(key, storage_proof.proof.abi_encode())
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
    parameters
        .keys()
        .find(|key| {
            ![TRUSTING_PERIOD, MAX_CLOCK_DRIFT, TRUSTED_HEIGHT, ROLE_MANAGER]
                .contains(&key.as_str())
        })
        .map_or(Ok(()), |key| {
            Err(anyhow!(
                "unexpected parameter `{key}`, only `{TRUSTING_PERIOD}`, `{MAX_CLOCK_DRIFT}`, `{TRUSTED_HEIGHT}`, and `{ROLE_MANAGER}` are allowed"
            ))
        })?;

    Ok(CreateClientParams {
        trusting_period: parameters
            .get(TRUSTING_PERIOD)
            .ok_or_else(|| anyhow!("missing `{TRUSTING_PERIOD}` parameter"))?
            .parse()
            .with_context(|| format!("failed to parse `{TRUSTING_PERIOD}` as decimal seconds"))?,
        max_clock_drift: parameters
            .get(MAX_CLOCK_DRIFT)
            .ok_or_else(|| anyhow!("missing `{MAX_CLOCK_DRIFT}` parameter"))?
            .parse()
            .with_context(|| format!("failed to parse `{MAX_CLOCK_DRIFT}` as decimal seconds"))?,
        trusted_height: parameters
            .get(TRUSTED_HEIGHT)
            .map(|value| {
                value.parse().with_context(|| {
                    format!("failed to parse `{TRUSTED_HEIGHT}` as decimal block height")
                })
            })
            .transpose()?,
        role_manager: parameters
            .get(ROLE_MANAGER)
            .map_or(Ok(Address::ZERO), |value| {
                Address::from_str(value)
                    .with_context(|| format!("failed to parse `{ROLE_MANAGER}` as hex address"))
            })?,
    })
}

fn extract_validators_from_extra_data(extra_data: &[u8]) -> Result<Vec<Address>> {
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

#[cfg(test)]
mod tests {
    use super::map_storage_proofs;
    use alloy::{
        primitives::{Bytes, B256, U256},
        rpc::types::EIP1186StorageProof,
        sol_types::SolValue,
    };

    #[test]
    fn maps_storage_proofs_using_solidity_abi_encoding() {
        let key = B256::from(U256::from(1));
        let nodes = vec![Bytes::from(vec![0xc2, 0x01, 0x02])];
        let mapped = map_storage_proofs(
            &[key],
            vec![EIP1186StorageProof {
                key: U256::from(1).into(),
                value: U256::ZERO,
                proof: nodes.clone(),
            }],
        )
        .unwrap();

        assert_eq!(mapped[&key], nodes.abi_encode());
    }
}
