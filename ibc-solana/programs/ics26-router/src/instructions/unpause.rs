// SPDX-License-Identifier: Apache-2.0

use crate::errors::RouterError;
use crate::events::RouterUnpausedEvent;
use crate::state::RouterState;
use anchor_lang::prelude::*;

/// Unpauses the ICS26 router, resuming all IBC packet processing.
/// Requires `UNPAUSER_ROLE` and rejects CPI calls.
#[derive(Accounts)]
pub struct Unpause<'info> {
    /// Mutable global router configuration PDA whose `paused` flag will be cleared.
    #[account(
        mut,
        seeds = [RouterState::SEED],
        bump,
        constraint = router_state.paused @ RouterError::RouterNotPaused,
    )]
    pub router_state: Account<'info, RouterState>,

    /// Global access control state used for unpauser role verification.
    /// CHECK: Validated via seeds constraint using the stored `access_manager` program ID
    #[account(
        seeds = [access_manager::state::AccessManager::SEED],
        bump,
        seeds::program = router_state.am_state.access_manager,
    )]
    pub access_manager: AccountInfo<'info>,

    /// Signer authorized to unpause the router.
    pub unpauser: Signer<'info>,

    /// Instructions sysvar used for CPI detection.
    /// CHECK: Address constraint verifies this is the instructions sysvar
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instructions_sysvar: AccountInfo<'info>,
}

pub fn unpause(ctx: Context<Unpause>) -> Result<()> {
    access_manager::require_role(
        &ctx.accounts.access_manager,
        solana_ibc_types::roles::UNPAUSER_ROLE,
        &ctx.accounts.unpauser,
        &ctx.accounts.instructions_sysvar,
        &crate::ID,
    )?;

    ctx.accounts.router_state.paused = false;

    emit!(RouterUnpausedEvent {});

    Ok(())
}
