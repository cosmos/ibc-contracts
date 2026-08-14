// SPDX-License-Identifier: Apache-2.0

//! IBC event types for ICS26 Router

use crate::state::{ClientAccount, Packet};
use anchor_lang::prelude::*;

/// Event emitted when a packet is sent
#[event]
#[derive(Debug, Clone)]
pub struct SendPacketEvent {
    pub client_id: String,
    pub sequence: u64,
    pub packet: Packet,
    pub timeout_timestamp: u64,
}

/// Event emitted when a packet acknowledgement is written
#[event]
#[derive(Debug, Clone)]
pub struct WriteAcknowledgementEvent {
    pub client_id: String,
    pub sequence: u64,
    pub packet: Packet,
    pub acknowledgements: Vec<Vec<u8>>,
}

/// Event emitted when a packet is acknowledged
#[event]
#[derive(Debug, Clone)]
pub struct AckPacketEvent {
    pub client_id: String,
    pub sequence: u64,
    pub packet: Packet,
    pub acknowledgement: Vec<Vec<u8>>,
}

/// Event emitted when a packet times out
#[event]
#[derive(Debug, Clone)]
pub struct TimeoutPacketEvent {
    pub client_id: String,
    pub sequence: u64,
    pub packet: Packet,
}

/// Event emitted when a client is added
#[event]
#[derive(Debug, Clone)]
pub struct ClientAddedEvent {
    pub client: ClientAccount,
}

/// Event emitted when a client is updated
#[event]
#[derive(Debug, Clone)]
pub struct ClientUpdatedEvent {
    pub client: ClientAccount,
}

/// Event emitted when an IBC app is added
#[event]
#[derive(Debug, Clone)]
pub struct IBCAppAdded {
    pub port_id: String,
    pub app_program_id: Pubkey,
}

/// No-op event for testing
#[event]
#[derive(Debug, Clone)]
pub struct NoopEvent {}

/// Event emitted when the router is paused
#[event]
#[derive(Debug, Clone)]
pub struct RouterPausedEvent {}

/// Event emitted when the router is unpaused
#[event]
#[derive(Debug, Clone)]
pub struct RouterUnpausedEvent {}
