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

        let trusted_state = consensus_state(&self.fetch_source_header(trusted_height).await?)?;

        let calldata = match self.consensus_type {
            BesuConsensusType::Qbft => besu_qbft_light_client::BesuQBFTLightClient::deploy_builder(
                self.dst_provider.clone(),
                *self.src_ics26_router.address(),
                trusted_height,
                trusted_state.timestamp,
                trusted_state.stateRoot,
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
                    trusted_state.stateRoot,
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

        let trusted_state = consensus_state(&self.fetch_source_header(trusted_height).await?)?;
        let target_header = self.fetch_source_header(target_height).await?;

        Ok(Self::build_update_client_calldata(
            dst_client_id,
            trusted_height,
            trusted_state,
            &target_header,
        ))
    }

    pub async fn relay_events(&self, params: RelayEventsParams) -> Result<Vec<u8>> {
        // Prove against the latest source block rather than the event heights. A Bonsai-backed
        // Besu node only serves `eth_getProof` for recent state, so historical event heights may
        // already be pruned, whereas the latest block is always available. Packets that have
        // already been completed on the source chain since their event was emitted are detected
        // from the proven slot values and skipped instead of being sent as calls that would
        // revert, and timeouts benefit from the later timestamp.
        let min_proof_height = params
            .src_events
            .iter()
            .map(|event| event.height)
            .chain(params.timeout_relay_height)
            .max()
            .ok_or_else(|| anyhow!("no packets collected"))?;
        let proof_height = self
            .src_provider
            .get_block_number()
            .await
            .context("failed to fetch latest source block number")?;
        ensure!(
            proof_height >= min_proof_height,
            "latest source block {proof_height} is behind the required proof height {min_proof_height}"
        );
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
        let trusted_state = consensus_state(&self.fetch_source_header(trusted_height).await?)?;

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
        let storage_proofs = map_storage_proofs(&storage_keys, &proof.storage_proof)?;
        retain_provable_packet_calls(&mut packet_calls, &storage_proofs)?;
        if packet_calls.is_empty() {
            bail!("all packets have already been completed on the source chain at height {proof_height}")
        }
        attach_packet_proofs(
            &mut packet_calls,
            &proof.account_proof,
            &storage_proofs,
            &consensus_state(&header)?,
        )?;

        let update_call = Self::build_update_client_calldata(
            &params.dst_client_id,
            trusted_height,
            trusted_state,
            &header,
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
        target_header: &Header,
    ) -> Vec<u8> {
        let update_msg = IBesuLightClientMsgs::MsgUpdateClient {
            headerRlp: alloy_rlp::encode(target_header).into(),
            trustedHeight: MsgHeight {
                revisionNumber: 0,
                revisionHeight: trusted_height,
            },
            consensusStatePreimage: trusted_state,
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

/// Rebuilds the consensus state the light client derives from a source header.
///
/// The light client stores only `keccak256(abi.encode(ConsensusState))` per height, so every
/// message that references a height must carry the full consensus state. Every field comes from the
/// header itself, so no state that a Bonsai-backed Besu node may have pruned is needed.
fn consensus_state(header: &Header) -> Result<IBesuLightClientMsgs::ConsensusState> {
    Ok(IBesuLightClientMsgs::ConsensusState {
        timestamp: header.timestamp,
        stateRoot: header.state_root,
        validators: extract_validators_from_extra_data(&header.extra_data).with_context(|| {
            format!(
                "failed to extract validators from source block {}",
                header.number
            )
        })?,
    })
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

/// The `eth_getProof` result for a single storage slot of the source router.
#[derive(Debug, Clone, PartialEq, Eq)]
struct StorageSlotProof {
    /// The slot value at the proven height, zero when the slot is empty.
    value: U256,
    /// Ordered, RLP-encoded MPT nodes from the router account storage trie.
    nodes: Vec<Bytes>,
}

/// Drops packet calls whose proof can no longer be verified against the proven source state.
///
/// Proofs are fetched at the latest source block, so a packet completed on the source chain after
/// its event was emitted yields a proof of the opposite kind from what the router call needs.
/// `ICS26Router` verifies the proof before its no-op checks, so such a call would revert the whole
/// multicall instead of no-op'ing. Each dropped call is logged.
fn retain_provable_packet_calls(
    packet_calls: &mut Vec<routerCalls>,
    storage_proofs: &HashMap<B256, StorageSlotProof>,
) -> Result<()> {
    let mut error = None;
    packet_calls.retain(|call| {
        let storage_key = packet_storage_key(call);
        let Some(slot) = storage_proofs.get(&storage_key) else {
            error.get_or_insert_with(|| anyhow!("missing storage proof for key {storage_key}"));
            return false;
        };
        let slot_is_set = !slot.value.is_zero();
        let (kind, packet, reason) = match call {
            routerCalls::recvPacket(call) if !slot_is_set => (
                "recvPacket",
                &call.msg_.packet,
                "packet commitment is absent from the source router; the packet was already acknowledged or timed out",
            ),
            routerCalls::ackPacket(call) if !slot_is_set => (
                "ackPacket",
                &call.msg_.packet,
                "acknowledgement commitment is absent from the source router",
            ),
            routerCalls::timeoutPacket(call) if slot_is_set => (
                "timeoutPacket",
                &call.msg_.packet,
                "packet receipt exists on the source router; the packet was received before it could time out",
            ),
            routerCalls::recvPacket(_) | routerCalls::ackPacket(_) | routerCalls::timeoutPacket(_) => {
                return true;
            }
            _ => unreachable!("only recv, ack, and timeout calls are constructed"),
        };
        tracing::warn!(
            kind,
            source_client = %packet.sourceClient,
            dest_client = %packet.destClient,
            sequence = packet.sequence,
            "skipping packet call: {reason}"
        );
        false
    });
    error.map_or(Ok(()), Err)
}

/// Attaches an `IBesuLightClientMsgs::MembershipProof` to every packet call, pairing the storage
/// proof nodes for the packet's commitment slot with the consensus state they are verified against.
///
/// The router account proof is only attached to the first call. All calls share the same proof
/// height and execute in a single atomic multicall, so the light client verifies the account proof
/// once, caches the storage root in transient storage, and serves the remaining calls from that
/// cache when their `accountProofNodes` are empty.
fn attach_packet_proofs(
    packet_calls: &mut [routerCalls],
    account_proof: &[Bytes],
    storage_proofs: &HashMap<B256, StorageSlotProof>,
    proven_state: &IBesuLightClientMsgs::ConsensusState,
) -> Result<()> {
    let mut account_proof = Some(account_proof);
    packet_calls.iter_mut().try_for_each(|call| {
        let storage_key = packet_storage_key(call);
        let proof_nodes = &storage_proofs
            .get(&storage_key)
            .ok_or_else(|| anyhow!("missing storage proof for key {storage_key}"))?
            .nodes;
        let proof: Bytes = IBesuLightClientMsgs::MembershipProof {
            consensusStatePreimage: proven_state.clone(),
            accountProofNodes: account_proof
                .take()
                .map(<[Bytes]>::to_vec)
                .unwrap_or_default(),
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

/// Indexes storage slot proofs by slot key, requiring exactly one proof per expected key.
fn map_storage_proofs(
    expected_keys: &[B256],
    storage_proofs: &[EIP1186StorageProof],
) -> Result<HashMap<B256, StorageSlotProof>> {
    let expected_keys = expected_keys.iter().copied().collect::<HashSet<_>>();
    let proofs = storage_proofs.iter().try_fold(
        HashMap::with_capacity(expected_keys.len()),
        |mut proofs, storage_proof| {
            let key = storage_proof.key.as_b256();
            ensure!(
                expected_keys.contains(&key),
                "unexpected storage proof key {key}"
            );
            let slot = StorageSlotProof {
                value: storage_proof.value,
                nodes: storage_proof.proof.clone(),
            };
            ensure!(
                proofs.insert(key, slot).is_none(),
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

/// Extracts the list of validators from the `extraData` field in ascending order.
fn extract_validators_from_extra_data(extra_data: &[u8]) -> Result<Vec<Address>> {
    let out = Rlp::new(extra_data)
        .at(1)
        .context("failed to read validator list from extraData")?
        .into_iter()
        .map(|validator_rlp| {
            let validator = validator_rlp
                .data()
                .context("failed to decode validator address")?;
            Address::try_from(validator)
                .map_err(|_| anyhow!("invalid validator address length: {}", validator.len()))
        })
        .collect::<Result<Vec<Address>>>()?;
    // Besu returns the validators in ascending order, so we can check that the list is sorted.
    if !out.is_sorted() {
        bail!("validators are not sorted in ascending order");
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{
        attach_packet_proofs, map_storage_proofs, packet_storage_key, retain_provable_packet_calls,
        StorageSlotProof,
    };
    use alloy::{
        primitives::{Address, Bytes, B256, U256},
        rpc::types::EIP1186StorageProof,
        sol_types::SolValue,
    };
    use ibc_eureka_solidity_types::{
        ics26::{
            router::{ackPacketCall, recvPacketCall, routerCalls, timeoutPacketCall},
            IICS02ClientMsgs::Height,
            IICS26RouterMsgs::{MsgAckPacket, MsgRecvPacket, MsgTimeoutPacket, Packet},
        },
        msgs::IBesuLightClientMsgs,
    };

    fn packet(sequence: u64) -> Packet {
        Packet {
            sequence,
            sourceClient: "client-0".to_string(),
            destClient: "client-1".to_string(),
            timeoutTimestamp: 0,
            payloads: vec![],
        }
    }

    fn proof_height() -> Height {
        Height {
            revisionNumber: 0,
            revisionHeight: 1,
        }
    }

    fn recv_packet_call(sequence: u64) -> routerCalls {
        routerCalls::recvPacket(recvPacketCall {
            msg_: MsgRecvPacket {
                packet: packet(sequence),
                proofHeight: proof_height(),
                proofCommitment: Bytes::default(),
            },
        })
    }

    fn ack_packet_call(sequence: u64) -> routerCalls {
        routerCalls::ackPacket(ackPacketCall {
            msg_: MsgAckPacket {
                packet: packet(sequence),
                acknowledgement: Bytes::from(vec![0x01]),
                proofAcked: Bytes::default(),
                proofHeight: proof_height(),
            },
        })
    }

    fn timeout_packet_call(sequence: u64) -> routerCalls {
        routerCalls::timeoutPacket(timeoutPacketCall {
            msg_: MsgTimeoutPacket {
                packet: packet(sequence),
                proofTimeout: Bytes::default(),
                proofHeight: proof_height(),
            },
        })
    }

    fn sequence_of(call: &routerCalls) -> u64 {
        match call {
            routerCalls::recvPacket(call) => call.msg_.packet.sequence,
            routerCalls::ackPacket(call) => call.msg_.packet.sequence,
            routerCalls::timeoutPacket(call) => call.msg_.packet.sequence,
            _ => unreachable!("only recv, ack, and timeout calls are constructed"),
        }
    }

    fn decode_recv_proof(call: &routerCalls) -> IBesuLightClientMsgs::MembershipProof {
        let routerCalls::recvPacket(call) = call else {
            unreachable!("call kind is preserved");
        };
        IBesuLightClientMsgs::MembershipProof::abi_decode(&call.msg_.proofCommitment)
            .expect("proof decodes as MembershipProof")
    }

    fn proven_state() -> IBesuLightClientMsgs::ConsensusState {
        IBesuLightClientMsgs::ConsensusState {
            timestamp: 7,
            stateRoot: B256::repeat_byte(0xaa),
            validators: vec![Address::repeat_byte(0x11)],
        }
    }

    /// Builds one slot proof per call with a distinct single node, using `value` to pick the
    /// proven slot value from the call's index.
    fn slot_proofs(
        calls: &[routerCalls],
        value: impl Fn(usize) -> U256,
    ) -> HashMap<B256, StorageSlotProof> {
        calls
            .iter()
            .enumerate()
            .map(|(i, call)| {
                let slot = StorageSlotProof {
                    value: value(i),
                    nodes: vec![Bytes::from(vec![0xc1, u8::try_from(i).unwrap()])],
                };
                (packet_storage_key(call), slot)
            })
            .collect()
    }

    const SET: U256 = U256::from_limbs([1, 0, 0, 0]);

    #[test]
    fn maps_storage_proof_values_and_nodes_by_key() {
        let key = B256::from(U256::from(1));
        let nodes = vec![Bytes::from(vec![0xc2, 0x01, 0x02])];
        let mapped = map_storage_proofs(
            &[key],
            &[EIP1186StorageProof {
                key: U256::from(1).into(),
                value: U256::from(42),
                proof: nodes.clone(),
            }],
        )
        .unwrap();

        assert_eq!(
            mapped[&key],
            StorageSlotProof {
                value: U256::from(42),
                nodes,
            }
        );
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
    fn retains_recv_and_ack_calls_only_while_their_commitment_exists() {
        let mut calls = vec![
            recv_packet_call(1),
            recv_packet_call(2),
            ack_packet_call(3),
            ack_packet_call(4),
        ];
        // Sequences 2 and 4 have been cleared on the source chain.
        let storage_proofs = slot_proofs(&calls, |i| if i % 2 == 0 { SET } else { U256::ZERO });

        retain_provable_packet_calls(&mut calls, &storage_proofs).unwrap();

        assert_eq!(
            calls.iter().map(sequence_of).collect::<Vec<_>>(),
            vec![1, 3]
        );
        assert!(matches!(calls[0], routerCalls::recvPacket(_)));
        assert!(matches!(calls[1], routerCalls::ackPacket(_)));
    }

    #[test]
    fn retains_timeout_calls_only_while_no_receipt_exists() {
        let mut calls = vec![timeout_packet_call(1), timeout_packet_call(2)];
        // Sequence 1 was received on the source chain, so it can no longer time out.
        let storage_proofs = slot_proofs(&calls, |i| if i == 0 { SET } else { U256::ZERO });

        retain_provable_packet_calls(&mut calls, &storage_proofs).unwrap();

        assert_eq!(calls.iter().map(sequence_of).collect::<Vec<_>>(), vec![2]);
    }

    #[test]
    fn retain_rejects_missing_storage_proof() {
        let mut calls = vec![recv_packet_call(1)];
        assert!(retain_provable_packet_calls(&mut calls, &HashMap::default()).is_err());
    }

    #[test]
    fn attaches_membership_proofs_with_consensus_state_preimage() {
        let mut calls = vec![recv_packet_call(1)];
        let storage_key = packet_storage_key(&calls[0]);
        let nodes = vec![Bytes::from(vec![0xc2, 0x01, 0x02])];
        let account_nodes = vec![Bytes::from(vec![0xc3, 0x01, 0x02, 0x03])];
        let proven_state = proven_state();

        assert!(
            attach_packet_proofs(
                &mut calls,
                &account_nodes,
                &HashMap::default(),
                &proven_state
            )
            .is_err(),
            "missing storage proof must be rejected"
        );

        let storage_proofs = HashMap::from([(
            storage_key,
            StorageSlotProof {
                value: SET,
                nodes: nodes.clone(),
            },
        )]);
        attach_packet_proofs(&mut calls, &account_nodes, &storage_proofs, &proven_state).unwrap();

        let proof = decode_recv_proof(&calls[0]);
        assert_eq!(
            proof.consensusStatePreimage.abi_encode(),
            proven_state.abi_encode()
        );
        assert_eq!(proof.accountProofNodes, account_nodes);
        assert_eq!(proof.proofNodes, nodes);
    }

    #[test]
    fn attaches_account_proof_only_to_first_call_in_batch() {
        let mut calls = vec![
            recv_packet_call(1),
            recv_packet_call(2),
            recv_packet_call(3),
        ];
        let account_nodes = vec![Bytes::from(vec![0xc3, 0x01, 0x02, 0x03])];
        let proven_state = proven_state();
        let storage_proofs = slot_proofs(&calls, |_| SET);

        attach_packet_proofs(&mut calls, &account_nodes, &storage_proofs, &proven_state).unwrap();

        for (i, call) in calls.iter().enumerate() {
            let proof = decode_recv_proof(call);
            let expected_account_nodes = if i == 0 { &account_nodes[..] } else { &[][..] };
            assert_eq!(
                proof.accountProofNodes, expected_account_nodes,
                "only the first call should carry the account proof"
            );
            assert_eq!(
                proof.proofNodes,
                storage_proofs[&packet_storage_key(call)].nodes,
                "every call carries its own storage proof"
            );
            assert_eq!(
                proof.consensusStatePreimage.abi_encode(),
                proven_state.abi_encode()
            );
        }
    }

    #[test]
    fn account_proof_moves_to_first_surviving_call_after_filtering() {
        let mut calls = vec![recv_packet_call(1), recv_packet_call(2)];
        let account_nodes = vec![Bytes::from(vec![0xc3, 0x01, 0x02, 0x03])];
        // The first packet was already completed on the source chain.
        let storage_proofs = slot_proofs(&calls, |i| if i == 0 { U256::ZERO } else { SET });

        retain_provable_packet_calls(&mut calls, &storage_proofs).unwrap();
        attach_packet_proofs(&mut calls, &account_nodes, &storage_proofs, &proven_state()).unwrap();

        assert_eq!(calls.iter().map(sequence_of).collect::<Vec<_>>(), vec![2]);
        assert_eq!(
            decode_recv_proof(&calls[0]).accountProofNodes,
            account_nodes
        );
    }
}
