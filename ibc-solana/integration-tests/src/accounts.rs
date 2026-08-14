// SPDX-License-Identifier: Apache-2.0

//! Low-level account construction utilities.
//!
//! Provides Anchor discriminator computation and rent-exempt account
//! scaffolding for pre-populating `ProgramTest` state.

use solana_sdk::{pubkey::Pubkey, rent::Rent};

/// Compute the 8-byte Anchor instruction discriminator (`sha256("global:<name>")[..8]`).
pub fn anchor_discriminator(instruction_name: &str) -> [u8; 8] {
    let hash = solana_sdk::hash::hash(format!("global:{instruction_name}").as_bytes());
    let mut disc = [0u8; 8];
    disc.copy_from_slice(&hash.to_bytes()[..8]);
    disc
}

/// Create a rent-exempt account with the given `data` and `owner`.
pub fn account_owned_by(data: Vec<u8>, owner: Pubkey) -> solana_sdk::account::Account {
    let rent = Rent::default();
    solana_sdk::account::Account {
        lamports: rent.minimum_balance(data.len()),
        data,
        owner,
        executable: false,
        rent_epoch: 0,
    }
}
