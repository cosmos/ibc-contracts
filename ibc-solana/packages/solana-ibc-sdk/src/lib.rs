// SPDX-License-Identifier: Apache-2.0

//! IDL-generated instruction builders and event types for Solana IBC programs.
//!
//! This crate is auto-generated from Anchor IDL files by `build.rs`.
//! It provides typed instruction account builders and event structs
//! for the ICS26 router, ICS07 Tendermint, attestation and IFT programs.
//!
//! This crate should only be consumed by off-chain code (relayer, tests).
//! On-chain programs should depend on `solana-ibc-types` directly.

pub mod generated;
pub use generated::*;
pub mod pda;
