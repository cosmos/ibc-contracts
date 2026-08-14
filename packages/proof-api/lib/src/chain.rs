// SPDX-License-Identifier: Apache-2.0

//! Defines the [`Chain`] interface and some of its implementations.

use alloy::primitives::TxHash;
use serde::{de::DeserializeOwned, Serialize};
use solana_sdk::signature::Signature;
use std::fmt::Debug;

use crate::events::{EurekaEventWithHeight, SolanaEurekaEventWithHeight};

/// The `Chain` trait defines the interface for a chain.
pub trait Chain {
    /// The event type that the listener will return.
    /// These should be the events that the proof API is interested in.
    type Event: Clone + Debug;
    /// The transaction identifier type that the listener will ask for.
    /// This is often a hash of the transaction.
    type TxId: Clone + Serialize + DeserializeOwned + Debug;
    /// The block height type that the listener will ask for.
    /// This is often a u64.
    type Height: Clone + Serialize + DeserializeOwned + Debug + std::cmp::PartialOrd;
}

/// The `CosmosSdk` is a concrete implementation of the `Chain` trait for the Cosmos SDK.
pub struct CosmosSdk;

impl Chain for CosmosSdk {
    type Event = EurekaEventWithHeight;
    type TxId = tendermint::Hash;
    type Height = u64;
}

/// The `EthEureka` is an implementation of the `Chain` trait for `solidity-ibc-eureka` contracts.
pub struct EthEureka;

impl Chain for EthEureka {
    type Event = EurekaEventWithHeight;
    type TxId = TxHash;
    type Height = u64;
}

/// The `SolanaEureka` is an implementation of the `Chain` trait for Solana IBC programs.
pub struct SolanaEureka;

impl Chain for SolanaEureka {
    type Event = SolanaEurekaEventWithHeight;
    type TxId = Signature;
    type Height = u64;
}
