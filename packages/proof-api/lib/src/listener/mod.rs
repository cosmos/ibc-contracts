// SPDX-License-Identifier: Apache-2.0

//! # This module defines the [`ChainListenerService`] trait and some of its implementations.

pub mod cosmos_sdk;
pub mod eth_eureka;
pub mod solana;
mod r#trait;

pub use r#trait::ChainListenerService;
