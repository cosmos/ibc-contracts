// SPDX-License-Identifier: Apache-2.0

use crate::state::*;
use anchor_lang::prelude::*;

/// Accounts for initializing the test IBC app state PDA.
#[derive(Accounts)]
#[instruction(authority: Pubkey)]
pub struct Initialize<'info> {
    /// App state PDA that tracks packet counters and the authority.
    #[account(
        init,
        payer = payer,
        space = 8 + TestIbcAppState::INIT_SPACE,
        seeds = [IBCAppState::SEED],
        bump
    )]
    pub app_state: Account<'info, TestIbcAppState>,

    /// Fee payer for PDA creation.
    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn initialize(ctx: Context<Initialize>, authority: Pubkey) -> Result<()> {
    let app_state = &mut ctx.accounts.app_state;

    app_state.authority = authority;
    app_state.packets_received = 0;
    app_state.packets_acknowledged = 0;
    app_state.packets_timed_out = 0;
    app_state.packets_sent = 0;

    msg!("Test IBC App initialized with authority: {}", authority);

    Ok(())
}
