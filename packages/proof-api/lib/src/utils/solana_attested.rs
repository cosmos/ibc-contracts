// SPDX-License-Identifier: Apache-2.0

//! Utilities for attested relays targeting Solana chains.

use std::time::Duration;

use crate::aggregator::rpc::{AggregatedAttestation, CommitmentType};
use crate::aggregator::Aggregator;
use crate::events::SolanaEurekaEventWithHeight;
use crate::utils::attestor::get_packet_attestation;
use crate::utils::solana::{
    ibc_to_solana_ack_packet, ibc_to_solana_recv_packet, max_timeout_timestamp,
    target_events_to_timeout_msgs, TimeoutPacketWithChunks,
};
use crate::utils::{cosmos, solana_v0_tx, wait_for_condition, RelayEventsParams};
use alloy::sol_types::SolValue;
use anchor_lang::AnchorDeserialize;
use anyhow::{Context, Result};
use borsh::BorshSerialize;
use ibc_eureka_solidity_types::ics26::IICS26RouterMsgs::{Packet, Payload};
use ibc_proto_eureka::ibc::core::{
    channel::v2::{MsgAcknowledgement, MsgRecvPacket},
    client::v1::Height,
};
use solana_client::rpc_client::RpcClient;
use solana_ibc_constants::CHUNK_DATA_SIZE;
use solana_ibc_sdk::attestation::{
    accounts::{ClientState as AttestationClientState, ConsensusStateStore},
    instructions as attestation_instructions,
};
use solana_ibc_sdk::ics26_router::{
    accounts::{IBCApp, RouterState},
    instructions as router_instructions,
    types::{
        ClientAccount, Delivery, MsgAckPacket as SolanaMsgAckPacket,
        MsgRecvPacket as SolanaMsgRecvPacket, MsgTimeoutPacket as SolanaMsgTimeoutPacket,
    },
};
use solana_sdk::{
    commitment_config::CommitmentConfig, compute_budget::ComputeBudgetInstruction,
    instruction::Instruction, pubkey::Pubkey,
};

/// Maximum compute units allowed per Solana transaction.
const MAX_COMPUTE_UNIT_LIMIT: u32 = 1_400_000;

/// Priority fee in micro-lamports per compute unit.
const DEFAULT_PRIORITY_FEE: u64 = 1000;

/// Size of the Anchor account discriminator that precedes the account data.
const ANCHOR_DISCRIMINATOR_SIZE: usize = 8;

/// A Solana transaction builder targeting the ICS26 router program.
///
/// Implementors provide the Solana RPC and program configuration and build the packet
/// transactions. The router, light client and transaction primitives that every relay to Solana
/// needs are provided on top of those, and [`SolanaAttestedTxBuilder`] builds attested relays
/// with them.
pub trait SolanaAttestationTxBuilder: Sync {
    /// The transactions that deliver one packet.
    type PacketTxs;

    /// The Solana RPC client.
    fn solana_client(&self) -> &RpcClient;
    /// The ICS26 router program ID.
    fn solana_ics26_program_id(&self) -> Pubkey;
    /// The fee payer of the built transactions.
    fn fee_payer(&self) -> Pubkey;
    /// The Address Lookup Table used to reduce transaction size, if any.
    fn alt_address(&self) -> Option<Pubkey>;

    /// Builds the transactions that deliver a recv packet, chunking payloads and proof.
    ///
    /// # Errors
    /// Returns an error if transaction construction fails.
    fn build_recv_packet_chunked(
        &self,
        msg: &SolanaMsgRecvPacket,
        payload_data: &[Vec<u8>],
        proof_data: &[u8],
    ) -> Result<Self::PacketTxs>;
    /// Builds the transactions that deliver an ack packet, chunking payloads and proof.
    ///
    /// # Errors
    /// Returns an error if transaction construction fails.
    fn build_ack_packet_chunked(
        &self,
        msg: &SolanaMsgAckPacket,
        payload_data: &[Vec<u8>],
        proof_data: &[u8],
    ) -> Result<Self::PacketTxs>;
    /// Builds the transactions that deliver a timeout packet, chunking payloads and proof.
    ///
    /// # Errors
    /// Returns an error if transaction construction fails.
    fn build_timeout_packet_chunked(
        &self,
        msg: &SolanaMsgTimeoutPacket,
        payload_data: &[Vec<u8>],
        proof_data: &[u8],
    ) -> Result<Self::PacketTxs>;

    /// Resolves the access manager program ID from the router state.
    ///
    /// # Errors
    /// Returns an error if the router state cannot be read.
    fn resolve_access_manager_program_id(&self) -> Result<Pubkey> {
        let (router_state_pda, _) =
            router_instructions::Initialize::router_state_pda(&self.solana_ics26_program_id());

        let account = self
            .solana_client()
            .get_account_with_commitment(&router_state_pda, CommitmentConfig::confirmed())
            .map_err(|e| anyhow::anyhow!("Failed to fetch RouterState account: {e}"))?
            .value
            .ok_or_else(|| anyhow::anyhow!("Router state account not found"))?;

        if account.data.len() < ANCHOR_DISCRIMINATOR_SIZE {
            return Err(anyhow::anyhow!(
                "Account data too short for RouterState account"
            ));
        }

        let mut data = &account.data[ANCHOR_DISCRIMINATOR_SIZE..];
        let router_state = RouterState::deserialize(&mut data)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize RouterState account: {e}"))?;

        Ok(router_state.am_state.access_manager)
    }

    /// Resolves the IBC app program ID registered for `port_id`.
    ///
    /// # Errors
    /// Returns an error if the IBC app account cannot be read.
    fn resolve_port_program_id(&self, port_id: &str) -> Result<Pubkey> {
        let (ibc_app_account, _) =
            router_instructions::AddIbcApp::ibc_app_pda(port_id, &self.solana_ics26_program_id());

        let account = self
            .solana_client()
            .get_account_with_commitment(&ibc_app_account, CommitmentConfig::confirmed())
            .map_err(|e| {
                anyhow::anyhow!("Failed to fetch IBCApp account for port '{port_id}': {e}")
            })?
            .value
            .ok_or_else(|| anyhow::anyhow!("IBCApp account not found for port '{port_id}'"))?;

        if account.data.len() < ANCHOR_DISCRIMINATOR_SIZE {
            return Err(anyhow::anyhow!("Account data too short for IBCApp account"));
        }

        let mut data = &account.data[ANCHOR_DISCRIMINATOR_SIZE..];
        let ibc_app = IBCApp::deserialize(&mut data)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize IBCApp account: {e}"))?;

        tracing::debug!("Resolved port '{}' → {}", port_id, ibc_app.app_program_id);

        Ok(ibc_app.app_program_id)
    }

    /// Resolves the light client program ID of `client_id`.
    ///
    /// # Errors
    /// Returns an error if the client account cannot be read.
    fn resolve_client_program_id(&self, client_id: &str) -> Result<Pubkey> {
        let (client_account, _) =
            router_instructions::SendPacket::client_pda(client_id, &self.solana_ics26_program_id());

        let account = self
            .solana_client()
            .get_account_with_commitment(&client_account, CommitmentConfig::confirmed())
            .map_err(|e| {
                anyhow::anyhow!("Failed to fetch Client account for client '{client_id}': {e}")
            })?
            .value
            .ok_or_else(|| anyhow::anyhow!("Client account not found for client '{client_id}'"))?;

        if account.data.len() < ANCHOR_DISCRIMINATOR_SIZE {
            return Err(anyhow::anyhow!("Account data too short for Client account"));
        }

        let mut data = &account.data[ANCHOR_DISCRIMINATOR_SIZE..];
        let client = ClientAccount::deserialize(&mut data)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize Client account: {e}"))?;

        tracing::debug!(
            "Resolved client '{}' → {}",
            client_id,
            client.client_program_id
        );

        Ok(client.client_program_id)
    }

    /// Fetches the attestation light client state.
    ///
    /// # Errors
    /// Returns an error if the account cannot be fetched or deserialized.
    fn attestation_client_state(
        &self,
        light_client_program_id: Pubkey,
    ) -> Result<AttestationClientState> {
        let (client_state_pda, _) =
            attestation_instructions::Initialize::client_state_pda(&light_client_program_id);

        let account = self
            .solana_client()
            .get_account_with_commitment(&client_state_pda, CommitmentConfig::confirmed())
            .context("Failed to fetch attestation client state account")?
            .value
            .ok_or_else(|| anyhow::anyhow!("Attestation client state account not found"))?;

        let mut data = &account.data[ANCHOR_DISCRIMINATOR_SIZE..];
        let client_state = AttestationClientState::deserialize(&mut data)
            .context("Failed to deserialize attestation client state")?;

        Ok(client_state)
    }

    /// Fetches the timestamp (in seconds) of the attestation consensus state at `height`.
    ///
    /// # Errors
    /// Returns an error if the account cannot be fetched or deserialized.
    fn attestation_consensus_state_timestamp_secs(
        &self,
        height: u64,
        light_client_program_id: Pubkey,
    ) -> Result<u64> {
        let (consensus_state_pda, _) =
            attestation_instructions::VerifyMembership::consensus_state_at_height_pda(
                height,
                &light_client_program_id,
            );

        let account = self
            .solana_client()
            .get_account_with_commitment(&consensus_state_pda, CommitmentConfig::confirmed())
            .context("Failed to fetch attestation consensus state account")?
            .value
            .ok_or_else(|| anyhow::anyhow!("Attestation consensus state account not found"))?;

        let mut data = &account.data[ANCHOR_DISCRIMINATOR_SIZE..];
        let consensus_state = ConsensusStateStore::deserialize(&mut data)
            .context("Failed to deserialize attestation consensus state")?;

        // Attestation consensus state stores timestamp in seconds (not nanoseconds)
        Ok(consensus_state.timestamp)
    }

    /// Serializes instructions into a v0 transaction, resolving the ALT if present.
    ///
    /// # Errors
    /// Returns an error if transaction construction fails.
    fn create_tx_bytes(&self, instructions: &[Instruction]) -> Result<Vec<u8>> {
        solana_v0_tx::create_tx_bytes(
            self.solana_client(),
            self.fee_payer(),
            self.alt_address(),
            instructions,
        )
    }

    /// Serializes instructions into a v0 transaction using an explicit ALT and its entries.
    ///
    /// # Errors
    /// Returns an error if transaction construction fails.
    fn create_tx_bytes_with_alt(
        &self,
        instructions: &[Instruction],
        alt_address: Pubkey,
        alt_addresses: Vec<Pubkey>,
    ) -> Result<Vec<u8>> {
        solana_v0_tx::create_tx_bytes_with_alt(
            self.solana_client(),
            self.fee_payer(),
            instructions,
            alt_address,
            alt_addresses,
        )
    }

    /// Builds a transaction that creates a new Address Lookup Table.
    ///
    /// # Errors
    /// Returns an error if transaction construction fails.
    fn build_create_alt_tx(&self, slot: u64) -> Result<Vec<u8>> {
        solana_v0_tx::build_create_alt_tx(
            self.solana_client(),
            self.fee_payer(),
            self.alt_address(),
            slot,
        )
    }

    /// Builds a transaction that extends an Address Lookup Table with `accounts`.
    ///
    /// # Errors
    /// Returns an error if transaction construction fails.
    fn build_extend_alt_tx(&self, slot: u64, accounts: Vec<Pubkey>) -> Result<Vec<u8>> {
        solana_v0_tx::build_extend_alt_tx(
            self.solana_client(),
            self.fee_payer(),
            self.alt_address(),
            slot,
            accounts,
        )
    }

    /// Builds the transactions of every recv, ack and timeout packet, in that order.
    ///
    /// # Errors
    /// Returns an error if a message cannot be converted or its transactions cannot be built.
    fn build_packet_transactions(
        &self,
        recv_msgs: Vec<MsgRecvPacket>,
        ack_msgs: Vec<MsgAcknowledgement>,
        timeout_msgs: Vec<TimeoutPacketWithChunks>,
    ) -> Result<Vec<Self::PacketTxs>> {
        let mut results = Vec::new();

        for msg in recv_msgs {
            let recv = ibc_to_solana_recv_packet(msg)?;
            results.push(self.build_recv_packet_chunked(
                &recv.msg,
                &recv.payload_chunks,
                &recv.proof_chunks,
            )?);
        }

        for msg in ack_msgs {
            let ack = ibc_to_solana_ack_packet(msg)?;
            results.push(self.build_ack_packet_chunked(
                &ack.msg,
                &ack.payload_chunks,
                &ack.proof_chunks,
            )?);
        }

        for timeout in timeout_msgs {
            tracing::info!(
                "Building timeout packet for sequence {}",
                timeout.msg.packet.sequence
            );
            results.push(self.build_timeout_packet_chunked(
                &timeout.msg,
                &timeout.payload_chunks,
                &timeout.proof_chunks,
            )?);
        }

        Ok(results)
    }
}

/// Result of building an attestation `update_client` transaction.
pub struct AttestationUpdateClientResult {
    /// Serialized assembly transaction bytes.
    pub assembly_tx: Vec<u8>,
    /// The target height for this update.
    pub target_height: u64,
}

/// Borsh-serialized membership proof for Solana attestation light client.
///
/// Uses Borsh for efficient binary serialization (~2.5x smaller than JSON).
/// Note: Must match the format expected by the attestation light client on Solana.
#[derive(BorshSerialize)]
pub struct SolanaMembershipProof {
    /// ABI-encoded attestation data from the aggregator.
    pub attestation_data: Vec<u8>,
    /// ECDSA signatures from attestors.
    pub signatures: Vec<Vec<u8>>,
}

/// Builds a Borsh-encoded membership proof for Solana attestation light client.
///
/// # Arguments
/// * `attested_data` - ABI-encoded attestation data from the aggregator
/// * `signatures` - ECDSA signatures from attestors
/// * `max_signatures` - Maximum number of signatures to include (Solana tx size limit)
///
/// # Panics
/// Panics if Borsh serialization of `MembershipProof` fails (should not happen).
#[must_use]
pub fn build_solana_membership_proof(
    attested_data: Vec<u8>,
    signatures: Vec<Vec<u8>>,
    max_signatures: usize,
) -> Vec<u8> {
    let limited_signatures: Vec<_> = signatures.into_iter().take(max_signatures).collect();

    let proof = SolanaMembershipProof {
        attestation_data: attested_data,
        signatures: limited_signatures,
    };

    borsh::to_vec(&proof).expect("MembershipProof serialization should not fail")
}

/// Fetches a packet attestation from the aggregator.
///
/// # Arguments
/// * `aggregator` - The aggregator client
/// * `packets` - ABI-encoded packets to get attestation for
/// * `height` - The height to fetch attestation at
/// * `commitment_type` - Type of commitment (Packet or Ack)
///
/// # Errors
/// Returns an error if fetching attestation fails.
pub async fn fetch_packet_attestation(
    aggregator: &Aggregator,
    packets: Vec<Vec<u8>>,
    height: u64,
    commitment_type: CommitmentType,
) -> Result<Option<AggregatedAttestation>> {
    if packets.is_empty() {
        return Ok(None);
    }

    let result = get_packet_attestation(aggregator, packets, height, commitment_type).await?;
    Ok(result.map(|r| r.packet))
}

/// Collects ABI-encoded packets from recv messages.
#[must_use]
pub fn collect_recv_packets(recv_msgs: &[MsgRecvPacket]) -> Vec<Vec<u8>> {
    recv_msgs
        .iter()
        .filter_map(|msg| msg.packet.as_ref())
        .map(|p| Packet::from(p.clone()).abi_encode())
        .collect()
}

/// Collects ABI-encoded packets from ack messages.
#[must_use]
pub fn collect_ack_packets(ack_msgs: &[MsgAcknowledgement]) -> Vec<Vec<u8>> {
    ack_msgs
        .iter()
        .filter_map(|msg| msg.packet.as_ref())
        .map(|p| Packet::from(p.clone()).abi_encode())
        .collect()
}

/// Collects ABI-encoded packets from Solana timeout messages.
#[must_use]
pub fn collect_timeout_packets_solana(timeout_msgs: &[TimeoutPacketWithChunks]) -> Vec<Vec<u8>> {
    timeout_msgs
        .iter()
        .map(|t| {
            Packet {
                sequence: t.msg.packet.sequence,
                sourceClient: t.msg.packet.source_client.clone(),
                destClient: t.msg.packet.dest_client.clone(),
                timeoutTimestamp: t.msg.packet.timeout_timestamp,
                payloads: t
                    .msg
                    .packet
                    .payloads
                    .iter()
                    .map(|p| {
                        let value = match &p.data {
                            Delivery::Inline { data } => data.clone(),
                            Delivery::Chunked { .. } => {
                                vec![]
                            }
                        };
                        Payload {
                            sourcePort: p.source_port.clone(),
                            destPort: p.dest_port.clone(),
                            version: p.version.clone(),
                            encoding: p.encoding.clone(),
                            value: value.into(),
                        }
                    })
                    .collect(),
            }
            .abi_encode()
        })
        .collect()
}

/// Injects attestation proofs into Solana-bound IBC messages.
///
/// This function:
/// 1. Collects packets from recv, ack and timeout messages
/// 2. Fetches attestations from the aggregator in parallel
/// 3. Builds Borsh-encoded proofs
/// 4. Injects proofs into the messages
///
/// # Arguments
/// * `aggregator` - The aggregator client
/// * `recv_msgs` - Mutable recv messages to inject proofs into
/// * `ack_msgs` - Mutable ack messages to inject proofs into
/// * `timeout_msgs` - Mutable timeout messages to inject proofs into
/// * `target_height` - Height to fetch attestations at
/// * `max_signatures` - Maximum signatures to include (Solana tx size limit)
///
/// # Errors
/// Returns an error if fetching attestations fails.
pub async fn inject_solana_attestor_proofs(
    aggregator: &Aggregator,
    recv_msgs: &mut [MsgRecvPacket],
    ack_msgs: &mut [MsgAcknowledgement],
    timeout_msgs: &mut [TimeoutPacketWithChunks],
    target_height: &Height,
    max_signatures: usize,
) -> Result<()> {
    let recv_packets = collect_recv_packets(recv_msgs);
    let ack_packets = collect_ack_packets(ack_msgs);
    let timeout_packets = collect_timeout_packets_solana(timeout_msgs);

    let (recv_attestation, ack_attestation, timeout_attestation) = tokio::join!(
        fetch_packet_attestation(
            aggregator,
            recv_packets,
            target_height.revision_height,
            CommitmentType::Packet
        ),
        fetch_packet_attestation(
            aggregator,
            ack_packets,
            target_height.revision_height,
            CommitmentType::Ack
        ),
        fetch_packet_attestation(
            aggregator,
            timeout_packets,
            target_height.revision_height,
            CommitmentType::Receipt
        ),
    );

    if let Some(attestation) = recv_attestation? {
        let proof_bytes = build_solana_membership_proof(
            attestation.attested_data,
            attestation.signatures,
            max_signatures,
        );
        for msg in recv_msgs.iter_mut() {
            msg.proof_commitment.clone_from(&proof_bytes);
            msg.proof_height = Some(*target_height);
        }
        tracing::info!(
            "Injected attestation proof into {} recv messages",
            recv_msgs.len()
        );
    }

    if let Some(attestation) = ack_attestation? {
        let proof_bytes = build_solana_membership_proof(
            attestation.attested_data,
            attestation.signatures,
            max_signatures,
        );
        for msg in ack_msgs.iter_mut() {
            msg.proof_acked.clone_from(&proof_bytes);
            msg.proof_height = Some(*target_height);
        }
        tracing::info!(
            "Injected attestation proof into {} ack messages",
            ack_msgs.len()
        );
    }

    if let Some(attestation) = timeout_attestation? {
        let proof_bytes = build_solana_membership_proof(
            attestation.attested_data,
            attestation.signatures,
            max_signatures,
        );
        for timeout in timeout_msgs.iter_mut() {
            timeout.proof_chunks.clone_from(&proof_bytes);
            timeout.msg.proof.height = target_height.revision_height;
            timeout.msg.proof.data = Delivery::Chunked {
                total_chunks: u8::try_from(proof_bytes.len().div_ceil(CHUNK_DATA_SIZE).max(1))
                    .unwrap_or(u8::MAX),
            };
        }
        tracing::info!(
            "Injected attestation proof into {} timeout messages",
            timeout_msgs.len()
        );
    }

    Ok(())
}

/// Builds attested relay transactions to Solana with the transaction builder `B`.
pub struct SolanaAttestedTxBuilder<B> {
    aggregator: Aggregator,
    tx_builder: B,
}

impl<B: SolanaAttestationTxBuilder> SolanaAttestedTxBuilder<B> {
    /// Creates an attested builder that builds its transactions with `tx_builder`.
    pub const fn new(aggregator: Aggregator, tx_builder: B) -> Self {
        Self {
            aggregator,
            tx_builder,
        }
    }

    /// The transaction builder.
    pub const fn tx_builder(&self) -> &B {
        &self.tx_builder
    }

    /// Builds the packet transactions for `params`, and the `update_client` transaction when the
    /// attestation client is behind the events or the timeouts.
    ///
    /// # Errors
    /// Returns an error if attestation retrieval or transaction building fails.
    #[allow(clippy::too_many_lines)]
    pub async fn relay_events(
        &self,
        params: RelayEventsParams<SolanaEurekaEventWithHeight>,
    ) -> Result<(Vec<B::PacketTxs>, Option<AttestationUpdateClientResult>)> {
        tracing::info!(
            "Building attested relay transaction for {} source events, {} target events",
            params.src_events.len(),
            params.target_events.len()
        );

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();

        let (mut recv_msgs, mut ack_msgs) = cosmos::src_events_to_recv_and_ack_msgs(
            params.src_events.clone(),
            &params.src_client_id,
            &params.dst_client_id,
            &params.src_packet_seqs,
            &params.dst_packet_seqs,
            "",
            now,
        );

        // Get Solana slot for timeout packet handling
        let slot = self
            .tx_builder
            .solana_client()
            .get_slot_with_commitment(CommitmentConfig::finalized())
            .map_err(|e| anyhow::anyhow!("Failed to get Solana slot: {e}"))?;

        // Extract max timeout timestamp before consuming target_events
        let max_timeout_ts = max_timeout_timestamp(&params.target_events);

        // Build timeout messages from destination events
        let mut timeout_msgs = target_events_to_timeout_msgs(
            params.target_events,
            &params.src_client_id,
            &params.dst_client_id,
            &params.dst_packet_seqs,
            slot,
            now,
        );

        if recv_msgs.is_empty() && ack_msgs.is_empty() && timeout_msgs.is_empty() {
            tracing::info!("No packets to relay");
            return Ok((vec![], None));
        }

        // Resolve light client program ID for the destination client
        let light_client_program_id = self
            .tx_builder
            .resolve_client_program_id(&params.dst_client_id)?;

        // Get current attestation client state to check if update is needed
        let client_state = self
            .tx_builder
            .attestation_client_state(light_client_program_id)?;
        let current_height = client_state.latest_height;

        let max_height = params
            .src_events
            .iter()
            .map(|e| e.height)
            .max()
            .unwrap_or(0);
        let required_height = max_height.saturating_add(1);

        // Check if height update is needed for proofs
        let needs_height_update = current_height < required_height;

        // Check if timestamp update is needed for timeouts
        let consensus_ts = max_timeout_ts.and_then(|_| {
            self.tx_builder
                .attestation_consensus_state_timestamp_secs(current_height, light_client_program_id)
                .ok()
        });

        let needs_timestamp_update = matches!(
            (consensus_ts, max_timeout_ts),
            (Some(cs_ts), Some(timeout_ts)) if cs_ts < timeout_ts
        );

        if needs_timestamp_update {
            tracing::info!(
                "Client update needed for timeout: consensus_ts={:?} < timeout_ts={:?}",
                consensus_ts,
                max_timeout_ts
            );
        }

        let needs_update = needs_height_update || needs_timestamp_update;

        // For timeouts, use the source chain height (where non-membership is proven).
        // This ensures the consensus state timestamp is past the packet's timeout.
        let proof_height = if needs_update {
            let min_height = required_height.max(current_height.saturating_add(1));
            let target = params
                .timeout_relay_height
                .map_or(min_height, |h| h.max(min_height));

            tracing::info!(
                "Attestation client at height {}, waiting for aggregator to finalize height {} \
                 (needs_height: {}, needs_timestamp: {})",
                current_height,
                target,
                needs_height_update,
                needs_timestamp_update
            );

            wait_for_condition(Duration::from_mins(15), Duration::from_secs(1), || async {
                let finalized_height = self.aggregator.get_latest_height().await?;
                Ok(finalized_height >= target)
            })
            .await
            .context("Timeout waiting for aggregator to finalize height")?;

            tracing::info!("Aggregator has finalized height {}", target);
            target
        } else {
            current_height
        };

        let min_sigs = client_state.min_required_sigs as usize;
        tracing::info!("Attestation client requires {} signatures", min_sigs);

        let target_height = Height {
            revision_number: 0,
            revision_height: proof_height,
        };

        inject_solana_attestor_proofs(
            &self.aggregator,
            &mut recv_msgs,
            &mut ack_msgs,
            &mut timeout_msgs,
            &target_height,
            min_sigs,
        )
        .await?;

        // Build update_client transaction only if needed
        let update_client = if needs_update {
            Some(
                self.update_client_at(&params.dst_client_id, proof_height)
                    .await?,
            )
        } else {
            tracing::info!(
                "Client already at sufficient height {}, skipping update",
                current_height
            );
            None
        };

        let packets =
            self.tx_builder
                .build_packet_transactions(recv_msgs, ack_msgs, timeout_msgs)?;

        Ok((packets, update_client))
    }

    /// Builds the `update_client` transaction to the aggregator's latest finalized height.
    ///
    /// # Errors
    /// Returns an error if fetching the latest height or building the transaction fails.
    pub async fn update_client(
        &self,
        dst_client_id: &str,
    ) -> Result<AttestationUpdateClientResult> {
        let latest_height = self.aggregator.get_latest_height().await?;
        tracing::info!(
            "Updating attestation client {} to latest height {}",
            dst_client_id,
            latest_height
        );
        self.update_client_at(dst_client_id, latest_height).await
    }

    /// Builds the `update_client` transaction to `target_height` from the aggregator's state
    /// attestation.
    async fn update_client_at(
        &self,
        dst_client_id: &str,
        target_height: u64,
    ) -> Result<AttestationUpdateClientResult> {
        use solana_ibc_sdk::access_manager::instructions as access_manager_instructions;
        use solana_ibc_sdk::attestation::{
            instructions::{UpdateClient, UpdateClientAccounts, UpdateClientArgs},
            types::UpdateClientParams,
        };

        tracing::info!(
            "Building attestation update_client for height {}",
            target_height
        );

        let light_client_program_id = self.tx_builder.resolve_client_program_id(dst_client_id)?;
        let client_state = self
            .tx_builder
            .attestation_client_state(light_client_program_id)?;
        let min_sigs = client_state.min_required_sigs as usize;

        let state_attestation = self.aggregator.get_state_attestation(target_height).await?;

        tracing::info!(
            "Aggregator returned {} signatures, attestation_data size: {} bytes",
            state_attestation.signatures.len(),
            state_attestation.attested_data.len()
        );

        let proof = build_solana_membership_proof(
            state_attestation.attested_data,
            state_attestation.signatures,
            min_sigs,
        );

        let access_manager_program_id = self.tx_builder.resolve_access_manager_program_id()?;
        let (access_manager, _) =
            access_manager_instructions::Initialize::access_manager_pda(&access_manager_program_id);

        let instruction = UpdateClient::builder(&light_client_program_id)
            .accounts(UpdateClientAccounts {
                access_manager,
                submitter: self.tx_builder.fee_payer(),
                new_height: target_height,
            })
            .args(&UpdateClientArgs {
                new_height: target_height,
                params: UpdateClientParams { proof },
            })
            .build();

        let assembly_tx = self.tx_builder.create_tx_bytes(&[
            ComputeBudgetInstruction::set_compute_unit_limit(MAX_COMPUTE_UNIT_LIMIT),
            ComputeBudgetInstruction::set_compute_unit_price(DEFAULT_PRIORITY_FEE),
            instruction,
        ])?;

        Ok(AttestationUpdateClientResult {
            assembly_tx,
            target_height,
        })
    }
}
