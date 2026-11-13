// SPDX-License-Identifier: Apache-2.0

//! Generic utilities for attestor-based relays (chain-agnostic).

use std::time::Duration;

use crate::aggregator::rpc::{AggregatedAttestation, CommitmentType};
use crate::aggregator::{Aggregator, AttestationResult};
use crate::events::{EurekaEvent, EurekaEventWithHeight};
use crate::utils::{wait_for_condition, RelayEventsParams};
use alloy::sol_types::SolValue;
use anyhow::Result;
use ibc_eureka_solidity_types::ics26::IICS26RouterMsgs::Packet;

/// Fetches attestation for a set of packets if non-empty.
///
/// Returns `None` if packets is empty, otherwise fetches and returns the attestation.
///
/// # Errors
/// Returns an error if the aggregator request fails.
pub async fn get_packet_attestation(
    aggregator: &Aggregator,
    packets: Vec<Vec<u8>>,
    height: u64,
    commitment_type: CommitmentType,
) -> Result<Option<AttestationResult>> {
    if packets.is_empty() {
        return Ok(None);
    }

    tracing::info!(
        "Requesting attestation for {} packets with commitment_type={:?}",
        packets.len(),
        commitment_type
    );

    let result = aggregator
        .get_attestations(packets, height, commitment_type)
        .await?;

    tracing::info!(
        "Received attestation for commitment_type={:?}: {} signatures at height {}",
        commitment_type,
        result.packet.signatures.len(),
        result.packet.height
    );

    Ok(Some(result))
}

/// Attestation data needed for building relay transactions.
pub struct AttestationData {
    /// State attestation (always present).
    pub state: AggregatedAttestation,
    /// Optional send packet attestation.
    pub send_attestation: Option<AggregatedAttestation>,
    /// Optional ack packet attestation.
    pub ack_attestation: Option<AggregatedAttestation>,
    /// Optional timeout/receipt attestation.
    pub timeout_attestation: Option<AggregatedAttestation>,
}

/// Fetches all attestations needed for relay transactions.
///
/// Fetches packet attestations in parallel, then extracts or fetches the state attestation.
///
/// # Arguments
/// - `aggregator`: The aggregator for fetching attestations.
/// - `send_packets`: Encoded send packets.
/// - `ack_packets`: Encoded ack packets.
/// - `timeout_packets`: Encoded timeout packets.
/// - `event_height`: Height to use for send/ack packets (where commitments exist).
/// - `timeout_height`: Height to use for timeout packets (where timeout has elapsed).
///
/// # Errors
/// Returns an error if any attestation request fails.
pub async fn fetch_attestations(
    aggregator: &Aggregator,
    send_packets: Vec<Vec<u8>>,
    ack_packets: Vec<Vec<u8>>,
    timeout_packets: Vec<Vec<u8>>,
    height: u64,
) -> Result<AttestationData> {
    let (send_result, ack_result, timeout_result) = tokio::join!(
        get_packet_attestation(aggregator, send_packets, height, CommitmentType::Packet),
        get_packet_attestation(aggregator, ack_packets, height, CommitmentType::Ack),
        get_packet_attestation(aggregator, timeout_packets, height, CommitmentType::Receipt)
    );
    let (send_data, ack_data, timeout_data) = (send_result?, ack_result?, timeout_result?);

    // Extract state from first available attestation, or fetch directly
    let state = if let Some(r) = send_data
        .as_ref()
        .or(ack_data.as_ref())
        .or(timeout_data.as_ref())
    {
        r.state.clone()
    } else {
        aggregator.get_state_attestation(height).await?
    };

    Ok(AttestationData {
        state,
        send_attestation: send_data.map(|r| r.packet),
        ack_attestation: ack_data.map(|r| r.packet),
        timeout_attestation: timeout_data.map(|r| r.packet),
    })
}

/// Fetches the attestations needed to relay the packets in `params`.
///
/// Collects the send, ack and timeout packets to relay, waits up to 15 minutes for the aggregator
/// to finalize their relay height (the highest among them) and fetches their attestations at that
/// height.
///
/// # Errors
/// Returns an error if no packets are collected, timeout packets are collected without a
/// `timeout_relay_height`, the relay height is not finalized in time, or an attestation request
/// fails.
pub async fn fetch_relay_attestations(
    aggregator: &Aggregator,
    params: &RelayEventsParams,
) -> Result<AttestationData> {
    let (send_packets, ack_packets, mut relay_height) = collect_send_and_ack_packets_with_height(
        &params.src_events,
        &params.src_client_id,
        &params.dst_client_id,
        &params.src_packet_seqs,
        &params.dst_packet_seqs,
    );

    let timeout_packets = collect_timeout_packets(
        &params.target_events,
        &params.src_client_id,
        &params.dst_client_id,
        &params.dst_packet_seqs,
    );

    if timeout_packets.is_empty() {
        tracing::debug!("No timeout packets collected");
    } else {
        let timeout_height = params
            .timeout_relay_height
            .ok_or_else(|| anyhow::anyhow!("timeout_relay_height required for timeout packets"))?;
        // Use max of src_events height and timeout height
        relay_height = Some(relay_height.map_or(timeout_height, |h| h.max(timeout_height)));
    }

    let relay_height = relay_height.ok_or_else(|| anyhow::anyhow!("No packets collected"))?;
    wait_for_condition(Duration::from_mins(15), Duration::from_secs(1), || async {
        let finalized_height = aggregator.get_latest_height().await?;
        Ok(finalized_height >= relay_height)
    })
    .await?;

    fetch_attestations(
        aggregator,
        send_packets,
        ack_packets,
        timeout_packets,
        relay_height,
    )
    .await
}

/// Checks if a packet should be included based on client IDs and sequence filters.
#[must_use]
pub fn packet_matches(
    packet: &Packet,
    src_client_id: &str,
    dst_client_id: &str,
    seqs: &[u64],
) -> bool {
    packet.sourceClient == src_client_id
        && packet.destClient == dst_client_id
        && (seqs.is_empty() || seqs.contains(&packet.sequence))
}

/// Collects encoded send and ack packets from events.
///
/// Returns a tuple of (send packets, ack packets, max height). The max height is the highest
/// event height among the collected packets, or `None` if no packets were collected.
#[must_use]
pub fn collect_send_and_ack_packets_with_height(
    events: &[EurekaEventWithHeight],
    src_client_id: &str,
    dst_client_id: &str,
    src_packet_seqs: &[u64],
    dst_packet_seqs: &[u64],
) -> (Vec<Vec<u8>>, Vec<Vec<u8>>, Option<u64>) {
    let (send_packets, max_height) = events
        .iter()
        .filter_map(|event| match &event.event {
            EurekaEvent::SendPacket(packet)
                if packet_matches(packet, src_client_id, dst_client_id, src_packet_seqs) =>
            {
                Some((packet, event.height))
            }
            _ => None,
        })
        .fold(
            (Vec::new(), None),
            |(mut packets, mut max_h), (packet, height)| {
                packets.push(packet.abi_encode());
                max_h = max_h.max(Some(height));
                (packets, max_h)
            },
        );

    let (ack_packets, max_height) = events
        .iter()
        .filter_map(|event| match &event.event {
            EurekaEvent::WriteAcknowledgement(packet, _)
                if packet_matches(packet, dst_client_id, src_client_id, dst_packet_seqs) =>
            {
                Some((packet, event.height))
            }
            _ => None,
        })
        .fold(
            (Vec::new(), max_height),
            |(mut packets, mut max_h), (packet, height)| {
                packets.push(packet.abi_encode());
                max_h = max_h.max(Some(height));
                (packets, max_h)
            },
        );

    (send_packets, ack_packets, max_height)
}

/// Collects ABI-encoded timeout packets from events.
///
/// Timeout packets are send packet events where the packet direction is reversed
/// (dest client -> src client) indicating the original packet was sent from destination.
/// Additionally, the packet's timeout timestamp must have elapsed.
///
/// # Panics
/// Panics if the current system time cannot be determined.
#[must_use]
pub fn collect_timeout_packets(
    events: &[EurekaEventWithHeight],
    src_client_id: &str,
    dst_client_id: &str,
    dst_packet_seqs: &[u64],
) -> Vec<Vec<u8>> {
    let now_since_unix = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    events
        .iter()
        .filter_map(|event| match &event.event {
            EurekaEvent::SendPacket(packet)
                if packet_matches(packet, dst_client_id, src_client_id, dst_packet_seqs)
                    && packet.timeoutTimestamp <= now_since_unix =>
            {
                Some(packet.abi_encode())
            }
            _ => None,
        })
        .collect()
}
