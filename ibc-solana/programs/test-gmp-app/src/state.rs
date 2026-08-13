// SPDX-License-Identifier: Apache-2.0

use anchor_lang::prelude::*;

/// Global state for the test GMP application.
///
/// Singleton PDA that tracks aggregate statistics (total user counters
/// created and total GMP calls processed) and stores the authority
/// allowed to manage the app. Serves as the program's signer PDA
/// when calling into the GMP program via CPI to send cross-chain calls.
#[account]
#[derive(InitSpace)]
pub struct CounterAppState {
    /// Authority that can manage the app
    pub authority: Pubkey,
    /// Total number of user counters created
    pub total_counters: u64,
    /// Total number of GMP calls processed
    pub total_gmp_calls: u64,
    /// Program bump seed
    pub bump: u8,
}

impl CounterAppState {
    pub const SEED: &'static [u8] = b"counter_app_state";
}

/// Per-user counter value and statistics.
///
/// Each user gets their own PDA derived from `["user_counter", user_pubkey]`.
/// Tracks the current counter value and the number of increments/decrements
/// applied, whether triggered locally or via incoming GMP cross-chain calls.
#[account]
#[derive(InitSpace)]
pub struct UserCounter {
    /// User's public key
    pub user: Pubkey,
    /// Current counter value
    pub count: u64,
    /// Number of increments
    pub increments: u64,
    /// Number of decrements
    pub decrements: u64,
    /// Last updated timestamp
    pub last_updated: i64,
    /// PDA bump seed
    pub bump: u8,
}

impl UserCounter {
    pub const SEED: &'static [u8] = b"user_counter";

    pub fn increment(&mut self, amount: u64, current_time: i64) -> Result<()> {
        self.count = self
            .count
            .checked_add(amount)
            .ok_or(crate::errors::CounterError::CounterOverflow)?;
        self.increments = self.increments.saturating_add(1);
        self.last_updated = current_time;
        Ok(())
    }

    pub fn decrement(&mut self, amount: u64, current_time: i64) -> Result<()> {
        self.count = self
            .count
            .checked_sub(amount)
            .ok_or(crate::errors::CounterError::CounterUnderflow)?;
        self.decrements = self.decrements.saturating_add(1);
        self.last_updated = current_time;
        Ok(())
    }
}

/// Record of an outgoing GMP cross-chain call initiated by a user.
///
/// Created when the counter app sends a cross-chain increment/decrement
/// via the GMP program. Stores the payload hash so the app can correlate
/// the callback (acknowledgement or timeout) with the original request
/// and update the success status accordingly.
#[account]
#[derive(InitSpace)]
pub struct GMPCallState {
    /// User who initiated the call
    pub user: Pubkey,
    /// Original payload hash
    pub payload_hash: [u8; 32],
    /// Call timestamp
    pub timestamp: i64,
    /// Success status
    pub success: bool,
    /// PDA bump seed
    pub bump: u8,
}

impl GMPCallState {
    pub const SEED: &'static [u8] = b"gmp_call_state";
}
