// SPDX-License-Identifier: Apache-2.0

use crate::types::ConsensusState;
use anchor_lang::prelude::*;
pub use solana_ibc_constants::CHUNK_DATA_SIZE;

/// On-chain PDA storing a Tendermint consensus state for a specific block height.
///
/// Contains the block timestamp, Merkle root and next-validators hash.
/// The ICS26 router passes these accounts to `verify_membership` /
/// `verify_non_membership` to check IBC packet commitment proofs against
/// the confirmed Tendermint state. One account exists per
/// verified height, derived from `["consensus_state", height_le_bytes]`.
#[account]
#[derive(InitSpace)]
pub struct ConsensusStateStore {
    pub height: u64,
    pub consensus_state: ConsensusState,
}

impl ConsensusStateStore {
    pub const SEED: &'static [u8] = b"consensus_state";

    /// Returns `true` when `init_if_needed` just created the account
    /// (Tendermint heights start at 1, so zero means uninitialized).
    pub const fn is_uninitialized(&self) -> bool {
        self.height == 0
    }
}

/// Temporary storage for a single chunk of a Tendermint header during
/// multi-transaction upload.
///
/// Tendermint headers can exceed the Solana transaction size limit, so
/// they are uploaded in chunks across multiple transactions. Once all
/// chunks are present the `assemble_and_update_client` instruction
/// reassembles and verifies the full header. These accounts are not closed
/// automatically; `cleanup_incomplete_upload` reclaims their rent.
#[account]
#[derive(InitSpace)]
pub struct HeaderChunk {
    /// The submitter who created this chunk
    pub submitter: Pubkey,
    /// The chunk data
    #[max_len(CHUNK_DATA_SIZE)]
    pub chunk_data: Vec<u8>,
}

impl HeaderChunk {
    pub const SEED: &'static [u8] = b"header_chunk";
}

/// Temporary storage for a single chunk of misbehaviour evidence during
/// multi-transaction upload.
///
/// Similar to [`HeaderChunk`] but used for misbehaviour reports that prove
/// a validator set signed conflicting blocks. Once fully assembled, the
/// `assemble_and_submit_misbehaviour` instruction verifies the evidence,
/// freezes the client and closes these accounts.
#[account]
#[derive(InitSpace)]
pub struct MisbehaviourChunk {
    /// The chunk data
    #[max_len(CHUNK_DATA_SIZE)]
    pub chunk_data: Vec<u8>,
}

impl MisbehaviourChunk {
    pub const SEED: &'static [u8] = b"misbehaviour_chunk";
}

/// Stores the result of an Ed25519 signature verification.
///
/// Written by `pre_verify_signature`, which introspects an `Ed25519Program`
/// precompile instruction in the same transaction and records the outcome
/// here. `assemble_and_update_client` and `assemble_and_submit_misbehaviour`
/// accept these accounts to skip on-chain verification of those signatures.
///
/// IMPORTANT: Field order matters — the verifier reads `data[8]` for `is_valid`.
#[account]
#[derive(InitSpace)]
pub struct SignatureVerification {
    /// Whether the signature is valid
    pub is_valid: bool,
    /// The submitter who created this verification
    pub submitter: Pubkey,
}

impl SignatureVerification {
    pub const SEED: &'static [u8] = b"sig_verify";
}

// Compile-time verification that discriminator length matches the shared constant
const _: () = assert!(
    SignatureVerification::DISCRIMINATOR.len() == solana_ibc_constants::ANCHOR_DISCRIMINATOR_LEN
);
