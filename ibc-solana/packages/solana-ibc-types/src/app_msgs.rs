// SPDX-License-Identifier: Apache-2.0

//! Message types for IBC application callbacks on Solana
//!
//! These types define the standard interface for IBC applications to handle
//! packet lifecycle events (receive, acknowledgement, timeout).

use anchor_lang::prelude::*;

/// Payload structure shared between router and IBC apps
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct Payload {
    pub source_port: String,
    pub dest_port: String,
    pub version: String,
    pub encoding: String,
    pub value: Vec<u8>,
}

/// Message for onRecvPacket callback
/// Sent from router to IBC app when a packet is received
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct OnRecvPacketMsg {
    pub source_client: String,
    pub dest_client: String,
    pub sequence: u64,
    pub payload: Payload,
    pub relayer: Pubkey,
}

/// Message for onAcknowledgementPacket callback
/// Sent from router to IBC app when an acknowledgement is received
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct OnAcknowledgementPacketMsg {
    pub source_client: String,
    pub dest_client: String,
    pub sequence: u64,
    pub payload: Payload,
    pub acknowledgement: Vec<u8>,
    pub relayer: Pubkey,
}

/// Message for onTimeoutPacket callback
/// Sent from router to IBC app when a packet times out
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct OnTimeoutPacketMsg {
    pub source_client: String,
    pub dest_client: String,
    pub sequence: u64,
    pub payload: Payload,
    pub relayer: Pubkey,
}

/// Common error codes that IBC apps might return
#[error_code]
pub enum IBCAppError {
    #[msg("Unauthorized: Only the IBC router can call this instruction")]
    UnauthorizedCaller,
    #[msg("Invalid packet data")]
    InvalidPacketData,
    #[msg("App-specific processing error")]
    ProcessingError,
}
