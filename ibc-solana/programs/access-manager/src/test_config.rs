// SPDX-License-Identifier: Apache-2.0

//! Per-crate test constants for AM-to-AM migration tests.
//!
//! This file is NOT shared (hardlinked/symlinked) between `access-manager`
//! and `test-access-manager`. Each crate has its own copy with different
//! values so that shared test code (which IS hardlinked) can reference
//! `crate::test_config::*` and resolve to the correct binary names and
//! program IDs for each crate.

use solana_sdk::pubkey::Pubkey;

/// Binary name for loading this crate's `.so` in `ProgramTest`.
pub const PROGRAM_BINARY_NAME: &str = "access_manager";

/// Binary name for the other access-manager instance (AM-to-AM migration tests).
pub const OTHER_AM_BINARY_NAME: &str = "test_access_manager";

/// Program ID of the other access-manager instance (AM-to-AM migration tests).
pub const OTHER_AM_ID: Pubkey = solana_sdk::pubkey!("9dvkqiBj6G1fNZjNXEet88HSxy14dFBA3tCMaiSns9a3");
