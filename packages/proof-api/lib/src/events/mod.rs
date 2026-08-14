// SPDX-License-Identifier: Apache-2.0

//! Contains the `EurekaEvent` type, which is used to parse Cosmos SDK and EVM IBC Eureka events.

pub mod cosmos_sdk;
mod eureka;
pub mod solana;

pub use eureka::{EurekaEvent, EurekaEventWithHeight};
pub use solana::{SolanaEurekaEvent, SolanaEurekaEventWithHeight};
