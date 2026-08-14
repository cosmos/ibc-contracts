// SPDX-License-Identifier: Apache-2.0

use anchor_lang::prelude::*;

#[event]
#[derive(Debug, Clone)]
pub struct RoleGrantedEvent {
    pub role_id: u64,
    pub account: Pubkey,
    pub granted_by: Pubkey,
}

#[event]
#[derive(Debug, Clone)]
pub struct RoleRevokedEvent {
    pub role_id: u64,
    pub account: Pubkey,
    pub revoked_by: Pubkey,
}

#[event]
#[derive(Debug, Clone)]
pub struct ProgramUpgradedEvent {
    pub program: Pubkey,
    pub authority: Pubkey,
    pub timestamp: i64,
}

#[event]
#[derive(Debug, Clone)]
pub struct WhitelistedProgramsUpdatedEvent {
    pub old_programs: Vec<Pubkey>,
    pub new_programs: Vec<Pubkey>,
    pub updated_by: Pubkey,
}

#[event]
#[derive(Debug, Clone)]
pub struct UpgradeAuthorityTransferProposedEvent {
    pub program: Pubkey,
    pub current_authority: Pubkey,
    pub proposed_authority: Pubkey,
    pub proposed_by: Pubkey,
}

#[event]
#[derive(Debug, Clone)]
pub struct UpgradeAuthorityTransferredEvent {
    pub program: Pubkey,
    pub old_authority: Pubkey,
    pub new_authority: Pubkey,
    pub accepted_by: Pubkey,
}

#[event]
#[derive(Debug, Clone)]
pub struct UpgradeAuthorityTransferCancelledEvent {
    pub program: Pubkey,
    pub cancelled_authority: Pubkey,
    pub cancelled_by: Pubkey,
}

#[event]
#[derive(Debug, Clone)]
pub struct UpgradeAuthorityClaimedEvent {
    pub program: Pubkey,
    pub source_access_manager: Pubkey,
    pub new_authority: Pubkey,
}

#[event]
#[derive(Debug, Clone)]
pub struct AccessManagerTransferProposed {
    pub current_access_manager: Pubkey,
    pub proposed_access_manager: Pubkey,
}

#[event]
#[derive(Debug, Clone)]
pub struct AccessManagerTransferAccepted {
    pub old_access_manager: Pubkey,
    pub new_access_manager: Pubkey,
}

#[event]
#[derive(Debug, Clone)]
pub struct AccessManagerTransferCancelled {
    pub access_manager: Pubkey,
    pub cancelled_access_manager: Pubkey,
}
