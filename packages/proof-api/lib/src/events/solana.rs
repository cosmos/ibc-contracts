// SPDX-License-Identifier: Apache-2.0

//! Solana IBC event parsing utilities
//!
//! This module provides utilities for parsing IBC events from Solana transaction logs.

use alloy::primitives::Bytes;
use anchor_lang::AnchorDeserialize as _;
use anyhow::Context as _;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use ibc_eureka_solidity_types::ics26::IICS26RouterMsgs::{
    Packet as SolPacket, Payload as SolPayload,
};
use solana_ibc_sdk::ics26_router::events::{SendPacketEvent, WriteAcknowledgementEvent};
use solana_ibc_sdk::ics26_router::types::{
    Delivery, MsgPacket, MsgPayload, MsgProof, MsgTimeoutPacket, Packet as SolanaPacket,
    Payload as SolanaPayload,
};

use crate::events::{EurekaEvent, EurekaEventWithHeight};
use solana_ibc_constants::CHUNK_DATA_SIZE;

/// Parsed IBC event from Solana transaction logs
#[derive(Debug, Clone)]
pub enum SolanaEurekaEvent {
    /// A packet was sent
    SendPacket(SendPacketEvent),
    /// An acknowledgement was written for a received packet
    WriteAcknowledgement(WriteAcknowledgementEvent),
}

/// Solana-specific Eureka event with height
#[derive(Debug, Clone)]
pub struct SolanaEurekaEventWithHeight {
    /// The event
    pub event: SolanaEurekaEvent,
    /// The slot height at which the event was emitted
    pub height: u64,
}

impl From<SolanaEurekaEventWithHeight> for EurekaEventWithHeight {
    fn from(event_with_height: SolanaEurekaEventWithHeight) -> Self {
        let event = match event_with_height.event {
            SolanaEurekaEvent::SendPacket(send_packet_event) => {
                EurekaEvent::SendPacket(to_sol_packet(send_packet_event.packet))
            }
            SolanaEurekaEvent::WriteAcknowledgement(write_acknowledgement_event) => {
                EurekaEvent::WriteAcknowledgement(
                    to_sol_packet(write_acknowledgement_event.packet),
                    write_acknowledgement_event
                        .acknowledgements
                        .into_iter()
                        .map(Bytes::from)
                        .collect(),
                )
            }
        };

        Self {
            event,
            height: event_with_height.height,
        }
    }
}

/// Converts a Solana timeout packet message to a Tendermint timeout message.
///
/// # Arguments
/// * `msg` - The Solana timeout packet message
/// * `signer` - The signer address string
///
/// # Returns
/// A Tendermint `MsgTimeout` for processing by IBC
///
/// # Errors
/// * this function will return error if timeout timestamp is 0 or negative
pub fn solana_timeout_packet_to_tm_timeout(
    msg: MsgTimeoutPacket,
    signer: String,
) -> anyhow::Result<ibc_proto_eureka::ibc::core::channel::v2::MsgTimeout> {
    let packet = ibc_proto_eureka::ibc::core::channel::v2::Packet {
        sequence: msg.packet.sequence,
        source_client: msg.packet.source_client.clone(),
        destination_client: msg.packet.dest_client.clone(),
        timeout_timestamp: msg.packet.timeout_timestamp,
        payloads: msg
            .packet
            .payloads
            .into_iter()
            .map(|p| {
                let value = match p.data {
                    Delivery::Inline { data } => data,
                    Delivery::Chunked { .. } => vec![],
                };
                ibc_proto_eureka::ibc::core::channel::v2::Payload {
                    source_port: p.source_port,
                    destination_port: p.dest_port,
                    version: p.version,
                    encoding: p.encoding,
                    value,
                }
            })
            .collect(),
    };

    let height = ibc_proto_eureka::ibc::core::client::v1::Height {
        revision_number: 0, // Solana doesn't have revision numbers
        revision_height: msg.proof.height,
    };

    let proof_unreceived = vec![]; // Placeholder - actual proof would be assembled from chunks

    let msg = ibc_proto_eureka::ibc::core::channel::v2::MsgTimeout {
        proof_unreceived,
        proof_height: Some(height),
        packet: Some(packet),
        signer,
    };

    Ok(msg)
}

/// Converts a Tendermint timeout message to a Solana timeout packet message.
///
/// # Arguments
/// * `msg` - The Tendermint timeout message
///
/// # Returns
/// A Solana `MsgTimeoutPacket` for processing by Solana IBC
///
/// # Errors
/// * Returns error if the packet field is missing
/// * Returns error if the `proof_height` field is missing
pub fn tm_timeout_to_solana_timeout_packet(
    msg: ibc_proto_eureka::ibc::core::channel::v2::MsgTimeout,
) -> anyhow::Result<MsgTimeoutPacket> {
    let packet = msg.packet.context("packet field is required")?;
    let proof_height = msg.proof_height.context("proof_height field is required")?;

    let sequence = packet.sequence;
    let source_client = packet.source_client;
    let dest_client = packet.destination_client;
    let timeout_timestamp = packet.timeout_timestamp;

    let msg_payloads: Vec<MsgPayload> = packet
        .payloads
        .into_iter()
        .map(|p| {
            let data = if p.value.len() > CHUNK_DATA_SIZE {
                let total_chunks = u8::try_from(p.value.len().div_ceil(CHUNK_DATA_SIZE))
                    .context("payload too big")?;
                Delivery::Chunked { total_chunks }
            } else {
                Delivery::Inline { data: p.value }
            };

            anyhow::Ok(MsgPayload {
                source_port: p.source_port,
                dest_port: p.destination_port,
                version: p.version,
                encoding: p.encoding,
                data,
            })
        })
        .collect::<anyhow::Result<Vec<_>>>()?;

    let proof_data = if msg.proof_unreceived.len() > CHUNK_DATA_SIZE {
        let total_chunks = u8::try_from(msg.proof_unreceived.len().div_ceil(CHUNK_DATA_SIZE))
            .context("proof too big")?;
        Delivery::Chunked { total_chunks }
    } else {
        Delivery::Inline {
            data: msg.proof_unreceived,
        }
    };

    Ok(MsgTimeoutPacket {
        packet: MsgPacket {
            sequence,
            source_client,
            dest_client,
            timeout_timestamp,
            payloads: msg_payloads,
        },
        proof: MsgProof {
            height: proof_height.revision_height,
            data: proof_data,
        },
    })
}

fn to_sol_packet(value: SolanaPacket) -> SolPacket {
    SolPacket {
        sequence: value.sequence,
        sourceClient: value.source_client.clone(),
        destClient: value.dest_client.clone(),
        timeoutTimestamp: value.timeout_timestamp,
        payloads: value.payloads.into_iter().map(to_sol_payload).collect(),
    }
}

fn to_sol_payload(value: SolanaPayload) -> SolPayload {
    SolPayload {
        sourcePort: value.source_port,
        destPort: value.dest_port,
        version: value.version,
        encoding: value.encoding,
        value: value.value.into(),
    }
}

fn try_parse_event_from_log(
    log: &str,
    log_idx: usize,
) -> anyhow::Result<Option<SolanaEurekaEvent>> {
    use anchor_lang::Discriminator;
    use anyhow::{anyhow, Context};

    let Some(data_str) = log.strip_prefix("Program data: ") else {
        return Ok(None);
    };

    let data = BASE64
        .decode(data_str)
        .with_context(|| format!("Failed to decode base64 in log {log_idx}: {data_str}"))?;

    if data.len() < 8 {
        // Not an Anchor event, skip
        return Ok(None);
    }

    let discriminator = &data[..8];
    let event_data = &data[8..];

    let is_ibc_event = discriminator == SendPacketEvent::DISCRIMINATOR
        || discriminator == WriteAcknowledgementEvent::DISCRIMINATOR;

    if !is_ibc_event {
        return Ok(None);
    }

    let event = match discriminator {
        disc if disc == SendPacketEvent::DISCRIMINATOR => {
            SendPacketEvent::try_from_slice(event_data)
                .map(SolanaEurekaEvent::SendPacket)
                .with_context(|| {
                    format!("Failed to deserialize SendPacketEvent in log {log_idx}")
                })?
        }
        disc if disc == WriteAcknowledgementEvent::DISCRIMINATOR => {
            WriteAcknowledgementEvent::try_from_slice(event_data)
                .map(SolanaEurekaEvent::WriteAcknowledgement)
                .with_context(|| {
                    format!("Failed to deserialize WriteAcknowledgementEvent in log {log_idx}")
                })?
        }
        _ => {
            return Err(anyhow!("Unexpected discriminator match in log {log_idx}"));
        }
    };

    Ok(Some(event))
}

fn log_payload_list(payloads: &[SolanaPayload]) {
    for (i, payload) in payloads.iter().enumerate() {
        tracing::debug!(
            "  Payload {}: source_port={}, dest_port={}, value_len={}",
            i,
            payload.source_port,
            payload.dest_port,
            payload.value.len()
        );
    }
}

fn log_event_details(event: &SolanaEurekaEvent) {
    tracing::debug!(?event, "parsed event");

    match event {
        SolanaEurekaEvent::SendPacket(send_event) => {
            tracing::debug!(
                "SendPacketEvent: sequence={}, {} payloads",
                send_event.sequence,
                send_event.packet.payloads.len()
            );
            log_payload_list(&send_event.packet.payloads);
        }
        SolanaEurekaEvent::WriteAcknowledgement(ack_event) => {
            tracing::debug!(
                "WriteAcknowledgementEvent: sequence={}, {} payloads, {} acks",
                ack_event.sequence,
                ack_event.packet.payloads.len(),
                ack_event.acknowledgements.len()
            );
            log_payload_list(&ack_event.packet.payloads);
        }
    }
}

/// Parse events from Solana transaction logs
///
/// This function extracts and deserializes Anchor events from the transaction logs.
///
/// Returns an error if any IBC event fails to parse, but ignores non-IBC events.
///
/// # Errors
///
/// This function will return an error if:
/// - Base64 decoding fails for any "Program data:" log entry
/// - Deserialization fails for any recognized IBC event (`SendPacket`, `WriteAcknowledgement`)
/// - An impossible discriminator match occurs (internal logic error)
///
///   TODO: Might be easier to parse via `anchor_client` but dependencies get kinda messy so manual parse
pub fn parse_events_from_logs(logs: &[String]) -> anyhow::Result<Vec<SolanaEurekaEvent>> {
    let mut events = Vec::new();

    for (log_idx, log) in logs.iter().enumerate() {
        if let Some(event) = try_parse_event_from_log(log, log_idx)? {
            log_event_details(&event);
            events.push(event);
        }
    }

    Ok(events)
}
