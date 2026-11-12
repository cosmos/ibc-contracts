// SPDX-License-Identifier: Apache-2.0

use anchor_lang::prelude::*;

/// On-chain PDA storing the consensus state for a specific block height.
///
/// Created when enough attestor signatures confirm a new block and never
/// overwritten (a conflicting timestamp freezes the client instead). The
/// ICS26 router passes this account to `verify_membership` /
/// `verify_non_membership` to check packet proofs against the confirmed state. The block height is also encoded in the
/// PDA seeds so each height maps to exactly one account.
#[account]
#[derive(InitSpace)]
pub struct ConsensusStateStore {
    /// Block height this consensus state corresponds to.
    pub height: u64,
    /// Unix timestamp in seconds for this block height.
    pub timestamp: u64,
}

impl ConsensusStateStore {
    pub const SEED: &'static [u8] = b"consensus_state";

    pub fn pda(height: u64) -> Pubkey {
        Pubkey::find_program_address(&[Self::SEED, &height.to_le_bytes()], &crate::ID).0
    }
}
