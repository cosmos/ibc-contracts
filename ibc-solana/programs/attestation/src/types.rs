// SPDX-License-Identifier: Apache-2.0

use access_manager::AccessManagerState;
use anchor_lang::prelude::*;

/// Account schema version for upgrades
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug, InitSpace)]
pub enum AccountVersion {
    V1,
}

mod sol_types {
    alloy_sol_types::sol!(
        "../../../ibc-solidity/contracts/light-clients/attestation/msgs/IAttestationMsgs.sol"
    );
}

pub use sol_types::IAttestationMsgs::{PacketAttestation, PacketCompact, StateAttestation};

use crate::ETH_ADDRESS_LEN;

/// Attestation light client state.
///
/// Holds the set of trusted attestor Ethereum addresses and the signature
/// threshold required to accept a new consensus state. Used by the ICS26
/// router to verify membership proofs that arrive as signed attestations
/// instead of ZK proofs.
#[account]
#[derive(InitSpace)]
pub struct ClientState {
    pub version: AccountVersion,
    /// 20-byte Ethereum addresses of trusted attestors.
    #[max_len(20)]
    pub attestor_addresses: Vec<[u8; ETH_ADDRESS_LEN]>,
    /// Minimum number of valid attestor signatures required for verification.
    pub min_required_sigs: u8,
    /// Highest block height for which a consensus state has been stored.
    pub latest_height: u64,
    /// Whether the client has been frozen due to misbehaviour detection.
    pub is_frozen: bool,
}

impl ClientState {
    pub const SEED: &'static [u8] = b"client";

    pub fn pda() -> Pubkey {
        Pubkey::find_program_address(&[Self::SEED], &crate::ID).0
    }
}

/// Global attestation program configuration.
///
/// Singleton PDA that links the attestation program to its access manager
/// for admin-gated operations such as updating the attestor set or freezing
/// the client.
#[account]
#[derive(InitSpace)]
pub struct AppState {
    pub version: AccountVersion,
    /// Access manager transfer state for two-step propose/accept
    pub am_state: AccessManagerState,
    /// Reserved for future upgrades without account migration.
    pub _reserved: [u8; 256],
}

impl AppState {
    pub const SEED: &'static [u8] = b"app_state";

    pub fn pda() -> Pubkey {
        Pubkey::find_program_address(&[Self::SEED], &crate::ID).0
    }
}

#[derive(
    AnchorSerialize, AnchorDeserialize, serde::Deserialize, serde::Serialize, Debug, Clone,
)]
pub struct MembershipProof {
    /// ABI-encoded attestation payload (`PacketAttestation` or `StateAttestation`).
    pub attestation_data: Vec<u8>,
    /// 65-byte ECDSA signatures (r || s || v) over `sha256(attestation_data)`.
    pub signatures: Vec<Vec<u8>>,
}
