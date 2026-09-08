// SPDX-License-Identifier: Apache-2.0

//! Utilities for attested relays targeting Solana chains.

use crate::aggregator::rpc::{AggregatedAttestation, CommitmentType};
use crate::aggregator::Aggregator;
use crate::utils::attestor::get_packet_attestation;
use crate::utils::solana::TimeoutPacketWithChunks;
use alloy::sol_types::SolValue;
use anyhow::Result;
use borsh::BorshSerialize;
use ibc_eureka_solidity_types::ics26::IICS26RouterMsgs::{Packet, Payload};
use ibc_proto_eureka::ibc::core::{
    channel::v2::{MsgAcknowledgement, MsgRecvPacket},
    client::v1::Height,
};
use solana_ibc_constants::CHUNK_DATA_SIZE;
use solana_ibc_sdk::attestation::accounts::ClientState as AttestationClientState;
use solana_ibc_sdk::ics26_router::types::Delivery;
use solana_sdk::{
    compute_budget::ComputeBudgetInstruction, instruction::Instruction, pubkey::Pubkey,
};

/// Maximum compute units allowed per Solana transaction.
const MAX_COMPUTE_UNIT_LIMIT: u32 = 1_400_000;

/// Priority fee in micro-lamports per compute unit.
const DEFAULT_PRIORITY_FEE: u64 = 1000;

/// Trait abstracting the Solana tx builder methods needed for attestation
/// `update_client` transactions.
///
/// Implemented by both eth-to-solana `SolanaTxBuilder` and cosmos-to-solana
/// `TxBuilder` to allow shared attestation logic.
pub trait SolanaAttestationTxBuilder: Sync {
    /// Resolves the light client program ID for the given client ID.
    ///
    /// # Errors
    /// Returns an error if the client ID cannot be resolved.
    fn resolve_client_program_id(&self, client_id: &str) -> Result<Pubkey>;
    /// Fetches the attestation light client state from Solana.
    ///
    /// # Errors
    /// Returns an error if the account cannot be fetched or deserialized.
    fn attestation_client_state(&self, program_id: Pubkey) -> Result<AttestationClientState>;
    /// Resolves the access manager program ID from the router state.
    ///
    /// # Errors
    /// Returns an error if the router state cannot be read.
    fn resolve_access_manager_program_id(&self) -> Result<Pubkey>;
    /// Returns the fee payer public key.
    fn fee_payer(&self) -> Pubkey;
    /// Serializes instructions into a Solana versioned transaction (v0) with optional ALT.
    ///
    /// # Errors
    /// Returns an error if transaction construction fails.
    fn create_tx_bytes(&self, instructions: &[Instruction]) -> Result<Vec<u8>>;
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

/// Builds an attestation `update_client` transaction for the given height.
///
/// Fetches the state attestation from the aggregator, builds a membership
/// proof and assembles the Solana transaction.
///
/// # Errors
/// Returns an error if fetching the attestation or building the transaction fails.
pub async fn build_attestation_update_client_tx(
    aggregator: &Aggregator,
    tx_builder: &impl SolanaAttestationTxBuilder,
    dst_client_id: &str,
    target_height: u64,
) -> Result<AttestationUpdateClientResult> {
    tracing::info!(
        "Building attestation update_client for height {}",
        target_height
    );

    let light_client_program_id = tx_builder.resolve_client_program_id(dst_client_id)?;
    let client_state = tx_builder.attestation_client_state(light_client_program_id)?;
    let min_sigs = client_state.min_required_sigs as usize;

    let state_attestation = aggregator.get_state_attestation(target_height).await?;

    tracing::info!(
        "Aggregator returned {} signatures, attestation_data size: {} bytes",
        state_attestation.signatures.len(),
        state_attestation.attested_data.len()
    );

    let proof_bytes = build_solana_membership_proof(
        state_attestation.attested_data,
        state_attestation.signatures,
        min_sigs,
    );

    let assembly_tx = build_update_client_instruction_tx(
        tx_builder,
        target_height,
        proof_bytes,
        light_client_program_id,
    )?;

    Ok(AttestationUpdateClientResult {
        assembly_tx,
        target_height,
    })
}

/// Updates the attestation light client to the aggregator's latest finalized
/// height.
///
/// # Errors
/// Returns an error if fetching the latest height or building the transaction fails.
pub async fn update_attestation_client_tx(
    aggregator: &Aggregator,
    tx_builder: &impl SolanaAttestationTxBuilder,
    dst_client_id: &str,
) -> Result<AttestationUpdateClientResult> {
    let latest_height = aggregator.get_latest_height().await?;
    tracing::info!(
        "Updating attestation client {} to latest height {}",
        dst_client_id,
        latest_height
    );
    build_attestation_update_client_tx(aggregator, tx_builder, dst_client_id, latest_height).await
}

/// Builds the raw Solana instruction and transaction bytes for an attestation
/// `update_client` call.
fn build_update_client_instruction_tx(
    tx_builder: &impl SolanaAttestationTxBuilder,
    new_height: u64,
    proof: Vec<u8>,
    light_client_program_id: Pubkey,
) -> Result<Vec<u8>> {
    use solana_ibc_sdk::access_manager::instructions as access_manager_instructions;
    use solana_ibc_sdk::attestation::{
        instructions::{UpdateClient, UpdateClientAccounts, UpdateClientArgs},
        types::UpdateClientParams,
    };

    let access_manager_program_id = tx_builder.resolve_access_manager_program_id()?;
    let (access_manager, _) =
        access_manager_instructions::Initialize::access_manager_pda(&access_manager_program_id);

    let instruction = UpdateClient::builder(&light_client_program_id)
        .accounts(UpdateClientAccounts {
            access_manager,
            submitter: tx_builder.fee_payer(),
            new_height,
        })
        .args(&UpdateClientArgs {
            new_height,
            params: UpdateClientParams { proof },
        })
        .build();

    let instructions = vec![
        ComputeBudgetInstruction::set_compute_unit_limit(MAX_COMPUTE_UNIT_LIMIT),
        ComputeBudgetInstruction::set_compute_unit_price(DEFAULT_PRIORITY_FEE),
        instruction,
    ];

    tx_builder.create_tx_bytes(&instructions)
}
