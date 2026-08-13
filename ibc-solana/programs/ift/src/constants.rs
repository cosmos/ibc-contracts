// SPDX-License-Identifier: Apache-2.0

//! Program constants for ICS27 IFT

/// Default timeout duration (15 minutes in seconds)
pub const DEFAULT_TIMEOUT_DURATION: u64 = 60 * 15;

/// Maximum timeout duration (24 hours in seconds)
pub const MAX_TIMEOUT_DURATION: u64 = 60 * 60 * 24;

/// Minimum timeout duration (10 seconds)
pub const MIN_TIMEOUT_DURATION: u64 = 10;

/// Maximum client ID length — capped at Solana's `MAX_SEED_LEN` (32 bytes per seed element).
pub const MAX_CLIENT_ID_LENGTH: usize = 32;

/// Maximum counterparty address length
pub const MAX_COUNTERPARTY_ADDRESS_LENGTH: usize = 128;

/// Maximum receiver address length
pub const MAX_RECEIVER_LENGTH: usize = 128;

/// PDA seed for IFT app state (global singleton)
pub const IFT_APP_STATE_SEED: &[u8] = b"ift_app_state";

/// PDA seed for IFT app mint state (per-mint)
pub const IFT_APP_MINT_STATE_SEED: &[u8] = b"ift_app_mint_state";

/// PDA seed for IFT bridge
pub const IFT_BRIDGE_SEED: &[u8] = b"ift_bridge";

/// PDA seed for pending transfer
pub const PENDING_TRANSFER_SEED: &[u8] = b"pending_transfer";

/// PDA seed for mint authority
pub const MINT_AUTHORITY_SEED: &[u8] = b"ift_mint_authority";

/// Seconds per day for rate limit day calculation
pub const SECONDS_PER_DAY: u64 = 60 * 60 * 24;

/// Lamports deposited into the destination GMP account PDA before
/// `ift_mint` is dispatched on a Solana↔Solana IFT transfer.
///
/// `5_000_000` lamports = 0.005 SOL (1 SOL = `1_000_000_000` lamports).
///
/// Covers rent for the receiver ATA creation (~2.04M lamports / ~0.00204 SOL)
/// plus a margin for any further init accounts the instruction might touch.
pub const SOLANA_MINT_PAYLOAD_PREFUND_LAMPORTS: u64 = 5_000_000;
