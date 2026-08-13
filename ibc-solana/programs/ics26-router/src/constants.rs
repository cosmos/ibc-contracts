// SPDX-License-Identifier: Apache-2.0

//! Common constants used throughout the ICS26 router

/// Size of Anchor instruction discriminator in bytes
pub const ANCHOR_DISCRIMINATOR_SIZE: usize = 8;

/// Initial capacity for IBC CPI instruction data
/// Typical IBC message sizes:
/// - Discriminator: 8 bytes
/// - 2 client IDs: ~32 bytes each
/// - Sequence: 8 bytes
/// - Payload with ports/version/data: ~50-100 bytes
/// - Pubkey: 32 bytes
/// - Total: ~150-200 bytes typical
///   Using 200 to avoid reallocation in most cases
pub const IBC_CPI_INSTRUCTION_CAPACITY: usize = 200;
