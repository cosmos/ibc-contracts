// SPDX-License-Identifier: Apache-2.0

//! This module contains the utilities for proof API implementations.

use crate::events::EurekaEventWithHeight;
use futures_timer::Delay;
use std::future::Future;
use std::time::{Duration, Instant};

/// Parameters for relay events operations in attested mode.
///
/// This struct groups the common parameters needed for building relay transactions
/// across different chain combinations.
#[derive(Debug, Clone)]
pub struct RelayEventsParams {
    /// Events from the source chain containing send packets and acknowledgements.
    pub src_events: Vec<EurekaEventWithHeight>,
    /// Events from the target chain (used for timeout detection).
    pub target_events: Vec<EurekaEventWithHeight>,
    /// For timeout packets, the height from the source chain to use for attestation.
    /// Required when processing timeouts. The caller should provide the current height
    /// from the source chain (where non-membership needs to be proven).
    pub timeout_relay_height: Option<u64>,
    /// The client ID on the source chain.
    pub src_client_id: String,
    /// The client ID on the destination chain.
    pub dst_client_id: String,
    /// Packet sequences from the source chain to relay.
    pub src_packet_seqs: Vec<u64>,
    /// Packet sequences from the destination chain (for filtering).
    pub dst_packet_seqs: Vec<u64>,
}

/// Retries an operation until the condition is met or a timeout occurs.
///
/// The basic version just checks for a boolean condition.
///
/// # Errors
/// If the condition is not met within the timeout, an error is returned.
pub async fn wait_for_condition<F, Fut>(
    timeout: Duration,
    interval: Duration,
    mut condition: F,
) -> anyhow::Result<()>
where
    F: FnMut() -> Fut + Send,
    Fut: Future<Output = anyhow::Result<bool>> + Send,
{
    let start = Instant::now();
    while start.elapsed() < timeout {
        if condition().await? {
            return Ok(());
        }

        tracing::debug!(
            "Condition not met. Waiting for {} seconds before retrying",
            interval.as_secs()
        );
        Delay::new(interval).await;
    }
    anyhow::bail!("Timeout exceeded waiting for condition")
}

pub mod attestor;
pub mod tracing_layer;

/// Utils useful for type conversions for attestor clients
pub mod cosmos;
pub mod cosmos_attested;
pub mod eth_attested;
pub mod eth_eureka;
pub mod solana;
pub mod solana_attested;
pub mod solana_gmp;
pub mod solana_v0_tx;
