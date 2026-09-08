// SPDX-License-Identifier: Apache-2.0

use anchor_lang::prelude::*;

pub use solana_ibc_constants::roles;

/// Embedded access manager state for IBC programs.
///
/// Mirrors the on-chain struct in the access-manager program. Tracks which
/// access manager governs permissioned instructions and supports two-step
/// access manager migration (propose/accept).
///
/// Does not carry its own `_reserved` field — future fields can eat into the
/// `_reserved` space of the higher-level state that embeds this struct.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct AccessManagerState {
    /// Program ID of the access manager that governs this program's roles.
    pub access_manager: Pubkey,
    /// Proposed replacement access manager, set during a pending transfer.
    pub pending_access_manager: Option<Pubkey>,
}

/// Backwards-compatible helper struct for getting access manager PDA
/// All actual types have been moved to the access-manager program
pub struct AccessManager;

impl AccessManager {
    pub const SEED: &'static [u8] = b"access_manager";

    /// Get access manager PDA (backwards compatible helper)
    pub fn pda(program_id: Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[Self::SEED], &program_id)
    }
}
