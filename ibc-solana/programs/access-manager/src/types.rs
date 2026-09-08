// SPDX-License-Identifier: Apache-2.0

use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace, Debug)]
pub struct RoleData {
    pub role_id: u64,
    #[max_len(16)]
    pub members: Vec<Pubkey>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace, Debug, PartialEq, Eq)]
pub struct PendingAuthorityTransfer {
    pub target_program: Pubkey,
    pub new_authority: Pubkey,
}
