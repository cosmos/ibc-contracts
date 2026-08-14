// SPDX-License-Identifier: Apache-2.0

#![doc = include_str!("../README.md")]
#![deny(
    clippy::nursery,
    clippy::pedantic,
    warnings,
    missing_docs,
    unused_crate_dependencies
)]

pub mod aggregator;

use anchor_lang as _;
use ibc_core_commitment_types as _;
use solana_sdk as _;
use tonic as _;

pub mod chain;
pub mod events;
pub mod listener;
pub mod service_utils;
pub mod tendermint_client;
pub mod tx_builder;
pub mod utils;
