// SPDX-License-Identifier: Apache-2.0

use crate::errors::*;
use crate::state::*;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::set_return_data;

/// Accounts required to increment a user's counter.
///
/// The `user_authority` must be a signer so only the legitimate owner
/// (or the `gmp_account` PDA during cross-chain calls) can modify
/// the counter. If the [`UserCounter`] PDA does not yet exist it is
/// created automatically via `init_if_needed`.
#[derive(Accounts)]
#[instruction(amount: u64)]
pub struct IncrementCounter<'info> {
    /// Global app configuration PDA. Mutated to bump `total_counters`
    /// when a new [`UserCounter`] is initialized.
    #[account(
        mut,
        seeds = [CounterAppState::SEED],
        bump = app_state.bump
    )]
    pub app_state: Account<'info, CounterAppState>,

    /// Per-user counter PDA derived from `[UserCounter::SEED, user_authority]`.
    /// Created on first use and incremented by the requested `amount`.
    #[account(
        init_if_needed,
        payer = payer,
        space = 8 + UserCounter::INIT_SPACE,
        seeds = [UserCounter::SEED, user_authority.key().as_ref()],
        bump
    )]
    pub user_counter: Account<'info, UserCounter>,

    /// The user authority (`gmp_account` PDA for ICS27).
    /// Must be a signer to authorize operations on this user's counter.
    pub user_authority: Signer<'info>,

    /// Mutable signer that funds `user_counter` PDA creation when needed.
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Solana system program used to allocate the `user_counter` account.
    pub system_program: Program<'info, System>,
}

/// Accounts required to decrement a user's counter.
///
/// Unlike [`IncrementCounter`], no signer check is enforced on the
/// user because this instruction is typically invoked via CPI from
/// the GMP program during cross-chain callback processing.
#[derive(Accounts)]
#[instruction(user: Pubkey, amount: u64)]
pub struct DecrementCounter<'info> {
    /// Global app configuration PDA. Included for seed verification
    /// but not mutated during a decrement.
    #[account(
        mut,
        seeds = [CounterAppState::SEED],
        bump = app_state.bump
    )]
    pub app_state: Account<'info, CounterAppState>,

    /// Per-user counter PDA derived from `[UserCounter::SEED, user]`.
    /// Must already exist; its `count` is reduced by the requested `amount`.
    #[account(
        mut,
        seeds = [UserCounter::SEED, user.as_ref()],
        bump = user_counter.bump
    )]
    pub user_counter: Account<'info, UserCounter>,
}

/// Accounts required to read a user's current counter value.
///
/// This is a read-only instruction that writes the counter value
/// into the transaction return data via `set_return_data`.
#[derive(Accounts)]
#[instruction(user: Pubkey)]
pub struct GetCounter<'info> {
    /// Per-user counter PDA derived from `[UserCounter::SEED, user]`.
    /// Read-only; its `count` field is returned to the caller.
    #[account(
        seeds = [UserCounter::SEED, user.as_ref()],
        bump = user_counter.bump
    )]
    pub user_counter: Account<'info, UserCounter>,
}

pub fn increment(ctx: Context<IncrementCounter>, amount: u64) -> Result<()> {
    let app_state = &mut ctx.accounts.app_state;
    let user_counter = &mut ctx.accounts.user_counter;
    let user_authority = ctx.accounts.user_authority.key();
    let clock = Clock::get()?;

    // Initialize counter if it's new
    if user_counter.user == Pubkey::default() {
        user_counter.user = user_authority;
        user_counter.count = 0;
        user_counter.increments = 0;
        user_counter.decrements = 0;
        user_counter.bump = ctx.bumps.user_counter;
        app_state.total_counters = app_state.total_counters.saturating_add(1);
    }

    // Increment the counter
    user_counter.increment(amount, clock.unix_timestamp)?;

    msg!(
        "Incremented counter for user {} by {} to {}",
        user_authority,
        amount,
        user_counter.count
    );

    // Return the new counter value
    let result = user_counter.count.to_le_bytes();
    set_return_data(&result);

    Ok(())
}

pub fn decrement(ctx: Context<DecrementCounter>, user: Pubkey, amount: u64) -> Result<()> {
    let user_counter = &mut ctx.accounts.user_counter;
    let clock = Clock::get()?;

    require!(user_counter.user == user, CounterError::CounterNotFound);

    // Decrement the counter
    user_counter.decrement(amount, clock.unix_timestamp)?;

    msg!(
        "Decremented counter for user {} by {} to {}",
        user,
        amount,
        user_counter.count
    );

    // Return the new counter value
    let result = user_counter.count.to_le_bytes();
    set_return_data(&result);

    Ok(())
}

pub fn get_counter(ctx: Context<GetCounter>, user: Pubkey) -> Result<()> {
    let user_counter = &ctx.accounts.user_counter;

    require!(user_counter.user == user, CounterError::CounterNotFound);

    msg!("Counter for user {}: {}", user, user_counter.count);

    // Return the counter value
    let result = user_counter.count.to_le_bytes();
    set_return_data(&result);

    Ok(())
}
