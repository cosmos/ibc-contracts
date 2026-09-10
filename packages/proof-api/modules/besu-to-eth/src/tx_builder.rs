// SPDX-License-Identifier: Apache-2.0

use std::{
    collections::{BTreeSet, HashMap, HashSet},
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
use anyhow::{anyhow, bail, ensure, Context, Result};
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

/// Source chain data at a single height from which the light client derives its consensus state.
///
/// The light client stores only `keccak256(abi.encode(ConsensusState))` per height, so every
/// message that references a height must carry the full consensus state, rebuilt from the header
/// and the tracked router account at that height.
struct SourceSnapshot {
    header: Header,
    proof: EIP1186AccountProofResponse,
}

impl SourceSnapshot {
    fn consensus_state(&self) -> Result<IBesuLightClientMsgs::ConsensusState> {
        Ok(IBesuLightClientMsgs::ConsensusState {
            timestamp: self.header.timestamp,
            storageRoot: self.proof.storage_hash,
            validators: extract_validators_from_extra_data(&self.header.extra_data).with_context(
                || {
                    format!(
                        "failed to extract validators from source block {}",
                        self.header.number
                    )
                },
            )?,
        })
    }
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

        let trusted_state = self
            .fetch_source_snapshot(trusted_height)
            .await?
            .consensus_state()?;

        let calldata = match self.consensus_type {
            BesuConsensusType::Qbft => besu_qbft_light_client::BesuQBFTLightClient::deploy_builder(
                self.dst_provider.clone(),
                *self.src_ics26_router.address(),
                trusted_height,
                trusted_state.timestamp,
                trusted_state.storageRoot,
                trusted_state.validators,
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
                    trusted_state.timestamp,
                    trusted_state.storageRoot,
                    trusted_state.validators,
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
        let trusted_height = self.fetch_destination_trusted_height(dst_client_id).await?;
        let target_height = self
            .src_provider
            .get_block_number()
            .await
            .context("failed to fetch latest source block number")?;

        let trusted_state = self
            .fetch_source_snapshot(trusted_height)
            .await?
            .consensus_state()?;
        let target = self.fetch_source_snapshot(target_height).await?;

        Ok(Self::build_update_client_calldata(
            dst_client_id,
            trusted_height,
            trusted_state,
            target,
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

        let trusted_height = self
            .fetch_destination_trusted_height(&params.dst_client_id)
            .await?;
        let trusted_state = self
            .fetch_source_snapshot(trusted_height)
            .await?
            .consensus_state()?;

        let storage_keys = packet_calls
            .iter()
            .map(packet_storage_key)
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();
        let proof = self
            .fetch_source_proofs(proof_height, &storage_keys)
            .await
            .with_context(|| {
                format!("failed to fetch proofs for source router at height {proof_height}")
            })?;
        let target = SourceSnapshot { header, proof };
        let storage_proofs = map_storage_proofs(&storage_keys, &target.proof.storage_proof)?;
        attach_packet_proofs(
            &mut packet_calls,
            &storage_proofs,
            &target.consensus_state()?,
        )?;

        let update_call = Self::build_update_client_calldata(
            &params.dst_client_id,
            trusted_height,
            trusted_state,
            target,
        );

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
        trusted_state: IBesuLightClientMsgs::ConsensusState,
        target: SourceSnapshot,
    ) -> Vec<u8> {
        let update_msg = IBesuLightClientMsgs::MsgUpdateClient {
            headerRlp: alloy_rlp::encode(target.header).into(),
            trustedHeight: MsgHeight {
                revisionNumber: 0,
                revisionHeight: trusted_height,
            },
            consensusStatePreimage: trusted_state,
            accountProof: target.proof.account_proof.abi_encode().into(),
        };

        updateClientCall {
            clientId: dst_client_id.to_string(),
            updateMsg: update_msg.abi_encode().into(),
        }
        .abi_encode()
    }

    async fn fetch_source_snapshot(&self, block_height: u64) -> Result<SourceSnapshot> {
        let header = self
            .fetch_source_header(block_height)
            .await
            .with_context(|| format!("failed to fetch source block at height {block_height}"))?;
        let proof = self
            .fetch_source_proofs(block_height, &[])
            .await
            .with_context(|| {
                format!("failed to fetch proofs for source router at height {block_height}")
            })?;
        Ok(SourceSnapshot { header, proof })
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

    /// Returns the latest height trusted by the destination Besu light client.
    async fn fetch_destination_trusted_height(&self, dst_client_id: &str) -> Result<u64> {
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

        IBesuLightClientMsgs::ClientState::abi_decode(client_state_bz.as_ref())
            .map(|client_state| client_state.latestHeight.revisionHeight)
            .with_context(|| {
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

/// Attaches an `IBesuLightClientMsgs::MembershipProof` to every packet call, pairing the storage
/// proof nodes for the packet's commitment slot with the consensus state they are verified against.
fn attach_packet_proofs(
    packet_calls: &mut [routerCalls],
    storage_proofs: &HashMap<B256, Vec<Bytes>>,
    proven_state: &IBesuLightClientMsgs::ConsensusState,
) -> Result<()> {
    packet_calls.iter_mut().try_for_each(|call| {
        let storage_key = packet_storage_key(call);
        let proof_nodes = storage_proofs
            .get(&storage_key)
            .ok_or_else(|| anyhow!("missing storage proof for key {storage_key}"))?;
        let proof: Bytes = IBesuLightClientMsgs::MembershipProof {
            consensusStatePreimage: proven_state.clone(),
            proofNodes: proof_nodes.clone(),
        }
        .abi_encode()
        .into();

        match call {
            routerCalls::recvPacket(call) => call.msg_.proofCommitment = proof,
            routerCalls::ackPacket(call) => call.msg_.proofAcked = proof,
            routerCalls::timeoutPacket(call) => call.msg_.proofTimeout = proof,
            _ => unreachable!("only recv, ack, and timeout calls are constructed"),
        }
        Ok(())
    })
}

/// Indexes storage proof nodes by slot key, requiring exactly one proof per expected key.
fn map_storage_proofs(
    expected_keys: &[B256],
    storage_proofs: &[EIP1186StorageProof],
) -> Result<HashMap<B256, Vec<Bytes>>> {
    let expected_keys = expected_keys.iter().copied().collect::<HashSet<_>>();
    let proofs = storage_proofs.iter().try_fold(
        HashMap::with_capacity(expected_keys.len()),
        |mut proofs, storage_proof| {
            let key = storage_proof.key.as_b256();
            ensure!(
                expected_keys.contains(&key),
                "unexpected storage proof key {key}"
            );
            ensure!(
                proofs.insert(key, storage_proof.proof.clone()).is_none(),
                "duplicate storage proof key {key}"
            );
            Ok(proofs)
        },
    )?;

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

/// Reads the validator set committed in a Besu BFT header's `extraData`.
fn extract_validators_from_extra_data(extra_data: &[u8]) -> Result<Vec<Address>> {
    Rlp::new(extra_data)
        .at(1)
        .context("failed to read validator list from extraData")?
        .iter()
        .map(|validator| {
            let validator = validator
                .data()
                .context("failed to decode validator address")?;
            Address::try_from(validator)
                .map_err(|_| anyhow!("invalid validator address length: {}", validator.len()))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{attach_packet_proofs, map_storage_proofs, packet_storage_key};
    use alloy::{
        primitives::{Address, Bytes, B256, U256},
        rpc::types::EIP1186StorageProof,
        sol_types::SolValue,
    };
    use ibc_eureka_solidity_types::{
        ics26::{
            router::{recvPacketCall, routerCalls},
            IICS02ClientMsgs::Height,
            IICS26RouterMsgs::{MsgRecvPacket, Packet},
        },
        msgs::IBesuLightClientMsgs,
    };

    #[test]
    fn maps_storage_proof_nodes_by_key() {
        let key = B256::from(U256::from(1));
        let nodes = vec![Bytes::from(vec![0xc2, 0x01, 0x02])];
        let mapped = map_storage_proofs(
            &[key],
            &[EIP1186StorageProof {
                key: U256::from(1).into(),
                value: U256::ZERO,
                proof: nodes.clone(),
            }],
        )
        .unwrap();

        assert_eq!(mapped[&key], nodes);
    }

    #[test]
    fn rejects_missing_duplicate_and_unexpected_storage_proofs() {
        let key = B256::from(U256::from(1));
        let proof = |key: U256| EIP1186StorageProof {
            key: key.into(),
            value: U256::ZERO,
            proof: vec![],
        };

        assert!(map_storage_proofs(&[key], &[]).is_err());
        assert!(map_storage_proofs(&[key], &[proof(U256::from(1)), proof(U256::from(1))]).is_err());
        assert!(map_storage_proofs(&[key], &[proof(U256::from(2))]).is_err());
    }

    #[test]
    fn attaches_membership_proofs_with_consensus_state_preimage() {
        let packet = Packet {
            sequence: 1,
            sourceClient: "client-0".to_string(),
            destClient: "client-1".to_string(),
            timeoutTimestamp: 0,
            payloads: vec![],
        };
        let mut calls = vec![routerCalls::recvPacket(recvPacketCall {
            msg_: MsgRecvPacket {
                packet,
                proofHeight: Height {
                    revisionNumber: 0,
                    revisionHeight: 1,
                },
                proofCommitment: Bytes::default(),
            },
        })];
        let storage_key = packet_storage_key(&calls[0]);
        let nodes = vec![Bytes::from(vec![0xc2, 0x01, 0x02])];
        let proven_state = IBesuLightClientMsgs::ConsensusState {
            timestamp: 7,
            storageRoot: B256::repeat_byte(0xaa),
            validators: vec![Address::repeat_byte(0x11)],
        };

        assert!(
            attach_packet_proofs(&mut calls, &HashMap::default(), &proven_state).is_err(),
            "missing storage proof must be rejected"
        );

        let storage_proofs = HashMap::from([(storage_key, nodes.clone())]);
        attach_packet_proofs(&mut calls, &storage_proofs, &proven_state).unwrap();

        let routerCalls::recvPacket(call) = &calls[0] else {
            unreachable!("call kind is preserved");
        };
        let proof = IBesuLightClientMsgs::MembershipProof::abi_decode(&call.msg_.proofCommitment)
            .expect("proof decodes as MembershipProof");
        assert_eq!(
            proof.consensusStatePreimage.abi_encode(),
            proven_state.abi_encode()
        );
        assert_eq!(proof.proofNodes, nodes);
    }
}
