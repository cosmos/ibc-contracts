// SPDX-License-Identifier: Apache-2.0

//! Attestation light client helpers for integration tests.
//!
//! Provides proof construction (state and packet membership) and an
//! `update_client` instruction builder for the attestation LC.

use crate::attestor::Attestors;
use alloy_sol_types::SolValue;
use attestation::crypto::AttestationType;
use attestation::types::{MembershipProof, PacketAttestation, PacketCompact, StateAttestation};
use borsh::BorshSerialize;
use ics26_router::ics24::{
    packet_acknowledgement_commitment_path, packet_commitment_path, packet_receipt_commitment_path,
};
use solana_ibc_sdk::access_manager::instructions as am_sdk;
use solana_ibc_sdk::attestation::instructions as attestation_sdk;
use solana_ibc_sdk::attestation::types::UpdateClientParams;
use solana_keccak_hasher::hash as keccak256;
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};

/// Build an `update_client` instruction for the default attestation LC instance.
pub fn build_update_client_ix(relayer: Pubkey, height: u64, proof: MembershipProof) -> Instruction {
    build_update_client_ix_for_program(attestation::ID, relayer, height, proof)
}

/// Build an `update_client` instruction targeting a specific attestation program.
pub fn build_update_client_ix_for_program(
    program_id: Pubkey,
    relayer: Pubkey,
    height: u64,
    proof: MembershipProof,
) -> Instruction {
    let am_pda = am_sdk::Initialize::access_manager_pda(&access_manager::ID).0;

    attestation_sdk::UpdateClient::builder(&program_id)
        .accounts(attestation_sdk::UpdateClientAccounts {
            access_manager: am_pda,
            submitter: relayer,
            new_height: height,
        })
        .args(&attestation_sdk::UpdateClientArgs {
            new_height: height,
            params: UpdateClientParams {
                proof: proof.try_to_vec().expect("MembershipProof serialization"),
            },
        })
        .build()
}

/// Build a signed `MembershipProof` for a state attestation (used by `update_client`).
pub fn build_state_membership_proof(
    attestors: &Attestors,
    height: u64,
    timestamp: u64,
) -> MembershipProof {
    let attestation_data = StateAttestation { height, timestamp }.abi_encode();
    let signatures = attestors
        .as_slice()
        .iter()
        .map(|a| a.sign(&attestation_data, AttestationType::State))
        .collect();
    MembershipProof {
        attestation_data,
        signatures,
    }
}

/// Default merkle prefix used by the deployer's `add_client`.
pub const DEFAULT_MERKLE_PREFIX: &[u8] = &[0x01, 0x02, 0x03];

/// A `(path, commitment)` pair for a packet attestation entry.
pub struct PacketEntry {
    pub path: [u8; 32],
    pub commitment: [u8; 32],
}

/// Hash a commitment path with the merkle prefix, matching how the router
/// constructs `ics24::prefixed_path` before calling `verify_membership`.
fn prefixed_path_hash(merkle_prefix: &[u8], raw_path: &[u8]) -> [u8; 32] {
    let mut prefixed = Vec::with_capacity(merkle_prefix.len().saturating_add(raw_path.len()));
    prefixed.extend_from_slice(merkle_prefix);
    prefixed.extend_from_slice(raw_path);
    keccak256(&prefixed).to_bytes()
}

/// Build a `PacketEntry` for a `send_packet` commitment (used in `recv_packet` proof).
pub fn packet_commitment_entry(
    counterparty_client_id: &str,
    sequence: u64,
    commitment: [u8; 32],
) -> PacketEntry {
    let raw_path = packet_commitment_path(counterparty_client_id, sequence);
    let path = prefixed_path_hash(DEFAULT_MERKLE_PREFIX, &raw_path);
    PacketEntry { path, commitment }
}

/// Build a `PacketEntry` for an ack commitment (used in `ack_packet` proof).
pub fn ack_commitment_entry(
    counterparty_client_id: &str,
    sequence: u64,
    commitment: [u8; 32],
) -> PacketEntry {
    let raw_path = packet_acknowledgement_commitment_path(counterparty_client_id, sequence);
    let path = prefixed_path_hash(DEFAULT_MERKLE_PREFIX, &raw_path);
    PacketEntry { path, commitment }
}

/// Build a `PacketEntry` for a receipt commitment (used in `timeout_packet` proof).
pub fn receipt_commitment_entry(
    counterparty_client_id: &str,
    sequence: u64,
    commitment: [u8; 32],
) -> PacketEntry {
    let raw_path = packet_receipt_commitment_path(counterparty_client_id, sequence);
    let path = prefixed_path_hash(DEFAULT_MERKLE_PREFIX, &raw_path);
    PacketEntry { path, commitment }
}

/// Build a signed `MembershipProof` for a packet attestation
/// (used by `recv_packet`, `ack_packet` and `timeout_packet`).
pub fn build_packet_membership_proof(
    attestors: &Attestors,
    height: u64,
    entries: &[PacketEntry],
) -> MembershipProof {
    let attestation_data = PacketAttestation {
        height,
        packets: entries
            .iter()
            .map(|e| PacketCompact {
                path: e.path.into(),
                commitment: e.commitment.into(),
            })
            .collect(),
    }
    .abi_encode();

    let signatures = attestors
        .as_slice()
        .iter()
        .map(|a| a.sign(&attestation_data, AttestationType::Packet))
        .collect();

    MembershipProof {
        attestation_data,
        signatures,
    }
}

/// Borsh-serialize a `MembershipProof` into the byte vector expected by
/// the router's `MsgProof.proof` field.
pub fn serialize_proof(proof: &MembershipProof) -> Vec<u8> {
    proof.try_to_vec().expect("MembershipProof serialization")
}

/// Build a serialized recv proof: read commitment from the source chain,
/// construct the packet entry and sign with the verifying attestors.
pub async fn build_recv_proof_bytes(
    source_chain: &crate::chain::Chain,
    commitment_pda: Pubkey,
    counterparty_client_id: &str,
    sequence: u64,
    attestors: &Attestors,
) -> Vec<u8> {
    let commitment = crate::read_commitment(source_chain, commitment_pda).await;
    let entry = packet_commitment_entry(counterparty_client_id, sequence, commitment);
    let proof = build_packet_membership_proof(attestors, crate::router::PROOF_HEIGHT, &[entry]);
    serialize_proof(&proof)
}

/// Build a serialized ack proof: read ack data from the dest chain,
/// construct the ack entry and sign with the verifying attestors.
pub async fn build_ack_proof_bytes(
    dest_chain: &crate::chain::Chain,
    ack_pda: Pubkey,
    counterparty_client_id: &str,
    sequence: u64,
    attestors: &Attestors,
) -> Vec<u8> {
    let ack_data = crate::extract_ack_data(dest_chain, ack_pda).await;
    let entry = ack_commitment_entry(
        counterparty_client_id,
        sequence,
        ack_data
            .as_slice()
            .try_into()
            .expect("ack should be 32 bytes"),
    );
    let proof = build_packet_membership_proof(attestors, crate::router::PROOF_HEIGHT, &[entry]);
    serialize_proof(&proof)
}
