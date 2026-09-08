// SPDX-License-Identifier: Apache-2.0

//! Protobuf types for GMP
//!
//! This module re-exports types from the shared solana-ibc-proto crate.
//! Proto generation is centralized to ensure type consistency across programs and relayer.

// Re-export from shared proto crate
pub use solana_ibc_proto::{
    GmpAcknowledgement, GmpPacketData, GmpSolanaPayload, GmpValidationError, RawGmpPacketData,
    RawGmpSolanaPayload, RawSolanaAccountMeta, SolanaAccountMeta,
};
