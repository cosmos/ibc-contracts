// SPDX-License-Identifier: Apache-2.0

use anchor_lang::prelude::*;

declare_id!("GdEUjpVtKvHKStM3Hph6PnLSUMsJXvcVqugubhtQ5QUD");

pub mod errors;
pub mod instructions;
pub mod state;

use instructions::*;

/// Test GMP App Program
///
/// This program demonstrates a simple counter application that can be called
/// via the ICS27 GMP program through cross-chain IBC messages.
///
/// It provides:
/// - `initialize`: Initialize the counter app
/// - `increment`: Increment a user's counter (called by GMP)
/// - `decrement`: Decrement a user's counter (called by GMP)
/// - `get_counter`: Get a user's current counter value
///
#[program]
pub mod test_gmp_app {
    use super::*;

    /// Initialize the counter app
    pub fn initialize(ctx: Context<Initialize>, authority: Pubkey) -> Result<()> {
        instructions::initialize(ctx, authority)
    }

    /// Increment a user's counter (typically called by GMP program)
    /// The user is identified by the `user_authority` signer (ICS27 `gmp_account` PDA)
    pub fn increment(ctx: Context<IncrementCounter>, amount: u64) -> Result<()> {
        instructions::increment(ctx, amount)
    }

    /// Decrement a user's counter (typically called by GMP program)
    pub fn decrement(ctx: Context<DecrementCounter>, user: Pubkey, amount: u64) -> Result<()> {
        instructions::decrement(ctx, user, amount)
    }

    /// Get a user's counter value
    pub fn get_counter(ctx: Context<GetCounter>, user: Pubkey) -> Result<()> {
        instructions::get_counter(ctx, user)
    }
}
