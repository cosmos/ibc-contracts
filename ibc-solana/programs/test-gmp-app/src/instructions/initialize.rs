// SPDX-License-Identifier: Apache-2.0

use crate::state::*;
use anchor_lang::prelude::*;

/// Accounts required to initialize the test GMP application.
///
/// Creates the singleton [`CounterAppState`] PDA that stores
/// the designated authority and aggregate statistics.
#[derive(Accounts)]
#[instruction(authority: Pubkey)]
pub struct Initialize<'info> {
    /// Global app configuration PDA derived from a fixed seed.
    /// Created here and populated with the provided `authority`
    /// and zeroed counters.
    #[account(
        init,
        payer = payer,
        space = 8 + CounterAppState::INIT_SPACE,
        seeds = [CounterAppState::SEED],
        bump
    )]
    pub app_state: Account<'info, CounterAppState>,

    /// Mutable signer that funds the `app_state` PDA creation.
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Solana system program used to allocate the `app_state` account.
    pub system_program: Program<'info, System>,
}

pub fn initialize(ctx: Context<Initialize>, authority: Pubkey) -> Result<()> {
    let app_state = &mut ctx.accounts.app_state;

    app_state.authority = authority;
    app_state.total_counters = 0;
    app_state.total_gmp_calls = 0;
    app_state.bump = ctx.bumps.app_state;

    msg!("Counter App initialized with authority: {}", authority);
    Ok(())
}
