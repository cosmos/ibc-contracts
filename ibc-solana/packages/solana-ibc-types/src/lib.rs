// SPDX-License-Identifier: Apache-2.0

//! IBC types and utilities for Solana programs
//!
//! This crate provides all the types, messages, and utilities needed for
//! implementing IBC on Solana, including router messages (ICS26),
//! light client types (ICS07), and Solana-specific PDA utilities.

pub mod access_manager;
pub mod app_msgs;
pub mod cpi;
pub mod ibc_app;
pub mod ics07;
pub mod ics24;
pub mod router;

pub use app_msgs::{
    IBCAppError, OnAcknowledgementPacketMsg, OnRecvPacketMsg, OnTimeoutPacketMsg, Payload,
};

pub use router::{
    AccountVersion, Client, ClientAccount, Commitment, CounterpartyInfo, Delivery, IBCApp,
    IBCAppState, MsgAckPacket, MsgCleanupChunks, MsgPacket, MsgPayload, MsgProof, MsgRecvPacket,
    MsgSendPacket, MsgTimeoutPacket, MsgUploadChunk, Packet, PayloadChunk, ProofChunk, RouterState,
};

pub use solana_ibc_proto::MAX_CLIENT_ID_LENGTH;

pub use ics07::{UpdateClientMsg, ASSEMBLE_UPDATE_CLIENT_STATIC_ACCOUNTS};

pub use solana_ibc_gmp_types::{
    CallResultStatus, ClientId, ConstrainedBytes, ConstrainedError, ConstrainedString,
    ConstrainedVec, GMPAccount, GMPAppState, GMPCallResult, GMPPacketError, GmpPacketData, Salt,
    SignerSeeds, MAX_MEMO_LENGTH, MAX_RECEIVER_LENGTH, MAX_SALT_LENGTH, MAX_SENDER_LENGTH,
};

pub use access_manager::{roles, AccessManager};
pub use cpi::{
    is_cpi, reject_cpi, reject_direct_calls, reject_nested_cpi,
    require_direct_call_or_whitelisted_caller, validate_cpi_caller, CpiValidationError,
};
pub use ics24::{
    packet_acknowledgement_commitment_bytes32, packet_acknowledgement_commitment_key,
    packet_acknowledgement_commitment_path, packet_commitment_bytes32, packet_commitment_key,
    packet_commitment_path, packet_receipt_commitment_bytes32, packet_receipt_commitment_key,
    packet_receipt_commitment_path, prefixed_path, Ics24Error, UNIVERSAL_ERROR_ACK,
};
