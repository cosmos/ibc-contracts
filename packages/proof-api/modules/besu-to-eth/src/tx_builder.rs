// SPDX-License-Identifier: Apache-2.0

use std::{
    collections::{BTreeSet, HashMap, HashSet},
    str::FromStr,
    time::UNIX_EPOCH,
};

use alloy::{
    consensus::Header,
    network::Ethereum,
    primitives::{hex, keccak256, Address, Bytes, Signature, B256, U256},
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
use rlp::{Rlp, RlpStream};

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
    trust_level: IBesuLightClientMsgs::TrustThreshold,
    trusted_height: Option<u64>,
    role_manager: Address,
}

const TRUSTING_PERIOD: &str = "trusting_period";
const MAX_CLOCK_DRIFT: &str = "max_clock_drift";
const TRUST_LEVEL: &str = "trust_level";
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
                besu_qbft_light_client::IBesuLightClientMsgs::ClientState {
                    ibcRouter: *self.src_ics26_router.address(),
                    latestHeight: besu_qbft_light_client::IICS02ClientMsgs::Height {
                        revisionNumber: 0,
                        revisionHeight: trusted_height,
                    },
                    trustingPeriod: params.trusting_period,
                    maxClockDrift: params.max_clock_drift,
                    isFrozen: false,
                    trustLevel: besu_qbft_light_client::IBesuLightClientMsgs::TrustThreshold {
                        numerator: params.trust_level.numerator,
                        denominator: params.trust_level.denominator,
                    },
                },
                besu_qbft_light_client::IBesuLightClientMsgs::ConsensusState {
                    timestamp: trusted_state.timestamp,
                    stateRoot: trusted_state.stateRoot,
                    validators: trusted_state.validators,
                },
                params.role_manager,
            )
            .calldata()
            .to_vec(),
            BesuConsensusType::Ibft2 => {
                besu_ibft2_light_client::BesuIBFT2LightClient::deploy_builder(
                    self.dst_provider.clone(),
                    besu_ibft2_light_client::IBesuLightClientMsgs::ClientState {
                        ibcRouter: *self.src_ics26_router.address(),
                        latestHeight: besu_ibft2_light_client::IICS02ClientMsgs::Height {
                            revisionNumber: 0,
                            revisionHeight: trusted_height,
                        },
                        trustingPeriod: params.trusting_period,
                        maxClockDrift: params.max_clock_drift,
                        isFrozen: false,
                        trustLevel: besu_ibft2_light_client::IBesuLightClientMsgs::TrustThreshold {
                            numerator: params.trust_level.numerator,
                            denominator: params.trust_level.denominator,
                        },
                    },
                    besu_ibft2_light_client::IBesuLightClientMsgs::ConsensusState {
                        timestamp: trusted_state.timestamp,
                        stateRoot: trusted_state.stateRoot,
                        validators: trusted_state.validators,
                    },
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

        Self::build_update_client_calldata(
            dst_client_id,
            trusted_height,
            trusted_state,
            &target_header,
            self.consensus_type,
        )
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
            self.consensus_type,
        )?;

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
        consensus_type: BesuConsensusType,
    ) -> Result<Vec<u8>> {
        let target_header =
            sort_commit_seals(target_header, consensus_type).with_context(|| {
                format!(
                    "failed to sort commit seals of source block {}",
                    target_header.number
                )
            })?;
        let update_msg = IBesuLightClientMsgs::MsgUpdateClient {
            headerRlp: alloy_rlp::encode(&target_header).into(),
            trustedHeight: MsgHeight {
                revisionNumber: 0,
                revisionHeight: trusted_height,
            },
            consensusStatePreimage: trusted_state,
        };

        Ok(updateClientCall {
            clientId: dst_client_id.to_string(),
            updateMsg: update_msg.abi_encode().into(),
        }
        .abi_encode())
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
            ![
                TRUSTING_PERIOD,
                MAX_CLOCK_DRIFT,
                TRUST_LEVEL,
                TRUSTED_HEIGHT,
                ROLE_MANAGER,
            ]
            .contains(&key.as_str())
        })
        .map_or(Ok(()), |key| {
            Err(anyhow!(
                "unexpected parameter `{key}`, only `{TRUSTING_PERIOD}`, `{MAX_CLOCK_DRIFT}`, `{TRUST_LEVEL}`, `{TRUSTED_HEIGHT}`, and `{ROLE_MANAGER}` are allowed"
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
        trust_level: parameters
            .get(TRUST_LEVEL)
            .map_or(Ok(DEFAULT_TRUST_LEVEL), |value| parse_trust_level(value))?,
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

/// Default trust level, matching the `ceil(2n / 3)` commit-seal quorum.
const DEFAULT_TRUST_LEVEL: IBesuLightClientMsgs::TrustThreshold =
    IBesuLightClientMsgs::TrustThreshold {
        numerator: 2,
        denominator: 3,
    };

/// Parses a `<numerator>/<denominator>` trust level and checks that it is within `[1/3, 1]`.
fn parse_trust_level(value: &str) -> Result<IBesuLightClientMsgs::TrustThreshold> {
    let (numerator, denominator) = value
        .split_once('/')
        .and_then(|(numerator, denominator)| {
            Some((
                numerator.trim().parse::<u8>().ok()?,
                denominator.trim().parse::<u8>().ok()?,
            ))
        })
        .with_context(|| {
            format!("failed to parse `{TRUST_LEVEL}` as `<numerator>/<denominator>`: {value}")
        })?;
    ensure!(
        denominator != 0
            && numerator <= denominator
            && 3 * u16::from(numerator) >= u16::from(denominator),
        "`{TRUST_LEVEL}` must be within [1/3, 1], got {numerator}/{denominator}"
    );
    Ok(IBesuLightClientMsgs::TrustThreshold {
        numerator,
        denominator,
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

/// Returns a copy of `header` with its commit seals ordered by recovered signer address.
///
/// The light client requires commit seals in strictly ascending signer order, but Besu does not
/// order them that way. Reordering keeps the header valid because the commit-seal digest excludes
/// the seal list.
fn sort_commit_seals(header: &Header, consensus_type: BesuConsensusType) -> Result<Header> {
    let extra_data = Rlp::new(&header.extra_data);
    let item_count = extra_data
        .item_count()
        .context("failed to decode extraData")?;
    ensure!(
        item_count == 5,
        "expected 5 extraData items, got {item_count}"
    );
    let prefix = (0..4)
        .map(|i| extra_data.at(i).map(|item| item.as_raw().to_vec()))
        .collect::<Result<Vec<_>, _>>()
        .context("failed to decode extraData items")?;
    let seals: Vec<Vec<u8>> = extra_data
        .list_at(4)
        .context("failed to decode commit seals")?;

    let encode_extra_data = |seals: Option<&[Vec<u8>]>| {
        let mut stream = RlpStream::new_list(4 + usize::from(seals.is_some()));
        for item in &prefix {
            stream.append_raw(item, 1);
        }
        if let Some(seals) = seals {
            stream.begin_list(seals.len());
            for seal in seals {
                stream.append(seal);
            }
        }
        Bytes::from(stream.out().to_vec())
    };

    // QBFT signs the header with an empty seal list, IBFT2 with the seal list dropped.
    let mut signing_header = header.clone();
    signing_header.extra_data = match consensus_type {
        BesuConsensusType::Qbft => encode_extra_data(Some(&[])),
        BesuConsensusType::Ibft2 => encode_extra_data(None),
    };
    let digest = keccak256(alloy_rlp::encode(&signing_header));

    let mut signed_seals = seals
        .into_iter()
        .map(|seal| {
            let signer = Signature::from_raw(&seal)
                .and_then(|signature| signature.recover_address_from_prehash(&digest))
                .context("failed to recover commit seal signer")?;
            Ok((signer, seal))
        })
        .collect::<Result<Vec<_>>>()?;
    signed_seals.sort_by_key(|(signer, _)| *signer);
    if let Some(pair) = signed_seals.windows(2).find(|pair| pair[0].0 == pair[1].0) {
        bail!("duplicate commit seal signer {}", pair[0].0);
    }

    let sorted_seals: Vec<_> = signed_seals.into_iter().map(|(_, seal)| seal).collect();
    let mut sorted_header = header.clone();
    sorted_header.extra_data = encode_extra_data(Some(&sorted_seals));
    Ok(sorted_header)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{
        attach_packet_proofs, map_storage_proofs, packet_storage_key, parse_create_client_params,
        retain_provable_packet_calls, sort_commit_seals, StorageSlotProof, TRUST_LEVEL,
    };
    use crate::BesuConsensusType;
    use alloy::{
        consensus::Header,
        primitives::{keccak256, Address, Bytes, Signature, B256, U256},
        rpc::types::EIP1186StorageProof,
        signers::{local::PrivateKeySigner, SignerSync},
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
    use rlp::{Rlp, RlpStream};

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

    #[test]
    fn create_client_params_trust_level() {
        let params = |trust_level: Option<&str>| {
            let mut params = HashMap::from([
                ("trusting_period".to_string(), "1000".to_string()),
                ("max_clock_drift".to_string(), "10".to_string()),
            ]);
            if let Some(trust_level) = trust_level {
                params.insert(TRUST_LEVEL.to_string(), trust_level.to_string());
            }
            parse_create_client_params(&params)
                .map(|params| (params.trust_level.numerator, params.trust_level.denominator))
        };

        assert_eq!(params(None).unwrap(), (2, 3));
        assert_eq!(params(Some("1/3")).unwrap(), (1, 3));
        assert_eq!(params(Some("1/1")).unwrap(), (1, 1));
        for invalid in ["", "2", "2/", "a/3", "1/0", "0/0", "1/4", "4/3", "256/256"] {
            assert!(
                params(Some(invalid)).is_err(),
                "{invalid} should be rejected"
            );
        }
    }

    fn besu_extra_data(validators: &[Address], seals: Option<&[Vec<u8>]>) -> Bytes {
        let mut stream = RlpStream::new_list(4 + usize::from(seals.is_some()));
        stream.append(&vec![0u8; 32]);
        stream.begin_list(validators.len());
        for validator in validators {
            stream.append(&validator.as_slice());
        }
        stream.begin_list(0);
        stream.append(&vec![0u8; 4]);
        if let Some(seals) = seals {
            stream.begin_list(seals.len());
            for seal in seals {
                stream.append(seal);
            }
        }
        Bytes::from(stream.out().to_vec())
    }

    /// Returns a header sealed by `signers` in the given order, and the commit-seal digest.
    fn sealed_header(
        consensus_type: BesuConsensusType,
        signers: &[&PrivateKeySigner],
    ) -> (Header, B256) {
        let mut validators: Vec<_> = signers.iter().map(|signer| signer.address()).collect();
        validators.sort();
        let mut header = Header {
            number: 7,
            extra_data: besu_extra_data(
                &validators,
                matches!(consensus_type, BesuConsensusType::Qbft).then_some(&[]),
            ),
            ..Header::default()
        };
        let digest = keccak256(alloy_rlp::encode(&header));
        let seals: Vec<_> = signers
            .iter()
            .map(|signer| signer.sign_hash_sync(&digest).unwrap().as_bytes().to_vec())
            .collect();
        header.extra_data = besu_extra_data(&validators, Some(&seals));
        (header, digest)
    }

    #[test]
    fn sort_commit_seals_orders_seals_by_signer() {
        let signers: Vec<_> = (0..4).map(|_| PrivateKeySigner::random()).collect();
        let mut unsorted: Vec<_> = signers.iter().collect();
        unsorted.sort_by_key(|signer| std::cmp::Reverse(signer.address()));

        for consensus_type in [BesuConsensusType::Qbft, BesuConsensusType::Ibft2] {
            let (header, digest) = sealed_header(consensus_type, &unsorted);
            let sorted = sort_commit_seals(&header, consensus_type).unwrap();

            let extra_data = Rlp::new(&sorted.extra_data);
            let original = Rlp::new(&header.extra_data);
            for i in 0..4 {
                assert_eq!(
                    extra_data.at(i).unwrap().as_raw(),
                    original.at(i).unwrap().as_raw()
                );
            }
            let recovered: Vec<_> = extra_data
                .list_at::<Vec<u8>>(4)
                .unwrap()
                .iter()
                .map(|seal| {
                    Signature::from_raw(seal)
                        .unwrap()
                        .recover_address_from_prehash(&digest)
                        .unwrap()
                })
                .collect();
            let mut expected: Vec<_> = signers.iter().map(PrivateKeySigner::address).collect();
            expected.sort();
            assert_eq!(recovered, expected);
            assert_eq!(
                Header {
                    extra_data: header.extra_data.clone(),
                    ..sorted
                },
                header
            );
        }
    }

    #[test]
    fn sort_commit_seals_rejects_duplicate_signers() {
        let signer = PrivateKeySigner::random();
        let (header, _) = sealed_header(BesuConsensusType::Qbft, &[&signer, &signer]);
        let err = sort_commit_seals(&header, BesuConsensusType::Qbft).unwrap_err();
        assert!(err.to_string().contains("duplicate commit seal signer"));
    }
}
