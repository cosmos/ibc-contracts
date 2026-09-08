// SPDX-License-Identifier: Apache-2.0

//! Second access-manager instance for AM-to-AM migration testing.
//!
//! Reuses all source files from `access-manager` via symlinks with a different
//! `declare_id!`. This means `crate::ID` resolves to this program's ID,
//! giving correct PDA derivation for a second AM instance.

use anchor_lang::prelude::*;

pub mod errors;
pub mod events;
pub mod helpers;
pub mod instructions;
pub mod state;
#[cfg(test)]
pub mod test_config;
#[cfg(test)]
pub mod test_utils;
pub mod types;

pub use errors::AccessManagerError;
pub use helpers::{require_admin, require_role, require_role_with_whitelist};
use instructions::*;
pub use state::AccessManagerState;
pub use types::RoleData;

declare_id!("9dvkqiBj6G1fNZjNXEet88HSxy14dFBA3tCMaiSns9a3");

#[program]
pub mod test_access_manager {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, admin: Pubkey) -> Result<()> {
        instructions::initialize(ctx, admin)
    }

    pub fn grant_role(ctx: Context<GrantRole>, role_id: u64, account: Pubkey) -> Result<()> {
        instructions::grant_role(ctx, role_id, account)
    }

    pub fn revoke_role(ctx: Context<RevokeRole>, role_id: u64, account: Pubkey) -> Result<()> {
        instructions::revoke_role(ctx, role_id, account)
    }

    pub fn renounce_role(ctx: Context<RenounceRole>, role_id: u64) -> Result<()> {
        instructions::renounce_role(ctx, role_id)
    }

    pub fn upgrade_program(ctx: Context<UpgradeProgram>, target_program: Pubkey) -> Result<()> {
        instructions::upgrade_program(ctx, target_program)
    }

    pub fn propose_upgrade_authority_transfer(
        ctx: Context<ProposeUpgradeAuthorityTransfer>,
        target_program: Pubkey,
        new_authority: Pubkey,
    ) -> Result<()> {
        instructions::propose_upgrade_authority_transfer(ctx, target_program, new_authority)
    }

    pub fn accept_upgrade_authority_transfer(
        ctx: Context<AcceptUpgradeAuthorityTransfer>,
        target_program: Pubkey,
    ) -> Result<()> {
        instructions::accept_upgrade_authority_transfer(ctx, target_program)
    }

    pub fn cancel_upgrade_authority_transfer(
        ctx: Context<CancelUpgradeAuthorityTransfer>,
        target_program: Pubkey,
    ) -> Result<()> {
        instructions::cancel_upgrade_authority_transfer(ctx, target_program)
    }

    pub fn set_whitelisted_programs(
        ctx: Context<SetWhitelistedPrograms>,
        whitelisted_programs: Vec<Pubkey>,
    ) -> Result<()> {
        instructions::set_whitelisted_programs(ctx, whitelisted_programs)
    }

    pub fn claim_upgrade_authority(
        ctx: Context<ClaimUpgradeAuthority>,
        target_program: Pubkey,
    ) -> Result<()> {
        instructions::claim_upgrade_authority(ctx, target_program)
    }
}

/// Returns the filesystem path to the compiled test-access-manager `.so` binary.
/// Used by Mollusk/ProgramTest in this crate and downstream crate tests.
#[cfg(any(test, feature = "test-utils"))]
pub const fn get_access_manager_program_path() -> &'static str {
    "../../target/deploy/test_access_manager"
}
