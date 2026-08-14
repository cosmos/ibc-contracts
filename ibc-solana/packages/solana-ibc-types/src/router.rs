// SPDX-License-Identifier: Apache-2.0

//! Router message types for IBC on Solana
//!
//! These types define the messages for packet handling in the ICS26 router.

use crate::Payload;
use anchor_lang::prelude::*;

// Import validation constant from solana-ibc-proto (single source of truth)
use solana_ibc_proto::MAX_CLIENT_ID_LENGTH;

/// Account schema version for upgradability
#[derive(Debug, AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
pub enum AccountVersion {
    V1,
}

/// Counterparty chain information
#[derive(Debug, AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct CounterpartyInfo {
    /// Client ID on the counterparty chain
    #[max_len(MAX_CLIENT_ID_LENGTH)]
    pub client_id: String,
    /// Merkle prefix for proof verification
    #[max_len(8, 128)]
    pub merkle_prefix: Vec<Vec<u8>>,
}

/// Client account structure mapping client IDs to light client program IDs
#[derive(Debug, AnchorSerialize, AnchorDeserialize, Clone, InitSpace)]
pub struct ClientAccount {
    /// Schema version for upgrades
    pub version: AccountVersion,
    /// The client identifier
    #[max_len(MAX_CLIENT_ID_LENGTH)]
    pub client_id: String,
    /// The program ID of the light client
    pub client_program_id: Pubkey,
    /// Counterparty chain information
    pub counterparty_info: CounterpartyInfo,
    /// Whether the client is active
    pub active: bool,
    /// Reserved space for future fields
    pub _reserved: [u8; 256],
}

/// Packet structure matching Ethereum's ICS26RouterMsgs.Packet
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct Packet {
    pub sequence: u64,
    pub source_client: String,
    pub dest_client: String,
    pub timeout_timestamp: u64,
    pub payloads: Vec<Payload>,
}

/// Payload metadata for chunked operations
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum Delivery {
    Inline { data: Vec<u8> },
    Chunked { total_chunks: u8 },
}

impl Delivery {
    pub const fn total_chunks(&self) -> u8 {
        match self {
            Self::Inline { .. } => 0,
            Self::Chunked { total_chunks } => *total_chunks,
        }
    }
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct MsgPayload {
    pub source_port: String,
    pub dest_port: String,
    pub version: String,
    pub encoding: String,
    pub data: Delivery,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct MsgProof {
    pub height: u64,
    pub data: Delivery,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct MsgPacket {
    pub sequence: u64,
    pub source_client: String,
    pub dest_client: String,
    pub timeout_timestamp: u64,
    pub payloads: Vec<MsgPayload>,
}

/// Message for sending a packet
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct MsgSendPacket {
    pub source_client: String,
    pub sequence: u64,
    pub timeout_timestamp: u64,
    pub payload: Payload,
}

/// Message for receiving a packet
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct MsgRecvPacket {
    pub packet: MsgPacket,
    pub proof: MsgProof,
}

/// Message for acknowledging a packet
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct MsgAckPacket {
    pub packet: MsgPacket,
    pub acknowledgement: Vec<u8>,
    pub proof: MsgProof,
}

/// Message for timing out a packet
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct MsgTimeoutPacket {
    pub packet: MsgPacket,
    pub proof: MsgProof,
}

/// Message for uploading chunks
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct MsgUploadChunk {
    pub client_id: String,
    pub sequence: u64,
    pub payload_index: u8,
    pub chunk_index: u8,
    pub chunk_data: Vec<u8>,
}

/// Message for cleanup
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct MsgCleanupChunks {
    pub client_id: String,
    pub sequence: u64,
    pub payload_chunks: Vec<u8>, // Number of chunks for each payload
    pub total_proof_chunks: u8,
}

pub struct IBCApp;

impl IBCApp {
    pub const SEED: &'static [u8] = b"ibc_app";

    /// Get IBC app PDA for a port
    pub fn pda(port_id: &str, program_id: Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[Self::SEED, port_id.as_bytes()], &program_id)
    }
}

pub struct RouterState;

impl RouterState {
    pub const SEED: &'static [u8] = b"router_state";

    /// Get router state PDA
    pub fn pda(program_id: Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[Self::SEED], &program_id)
    }
}

pub struct Client;

impl Client {
    pub const SEED: &'static [u8] = b"client";

    /// Get client PDA
    pub fn pda(client_id: &str, program_id: Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[Self::SEED, client_id.as_bytes()], &program_id)
    }
}

pub struct Commitment;

impl Commitment {
    pub const SEED: &'static [u8] = b"commitment";
    pub const PACKET_COMMITMENT_SEED: &'static [u8] = b"packet_commitment";
    pub const PACKET_RECEIPT_SEED: &'static [u8] = b"packet_receipt";
    pub const PACKET_ACK_SEED: &'static [u8] = b"packet_ack";

    /// Get packet commitment PDA
    pub fn packet_commitment_pda(
        client_id: &str,
        sequence: u64,
        program_id: Pubkey,
    ) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[
                Self::PACKET_COMMITMENT_SEED,
                client_id.as_bytes(),
                &sequence.to_le_bytes(),
            ],
            &program_id,
        )
    }

    /// Get packet receipt PDA
    pub fn packet_receipt_pda(client_id: &str, sequence: u64, program_id: Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[
                Self::PACKET_RECEIPT_SEED,
                client_id.as_bytes(),
                &sequence.to_le_bytes(),
            ],
            &program_id,
        )
    }

    /// Get packet acknowledgment PDA
    pub fn packet_ack_pda(client_id: &str, sequence: u64, program_id: Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[
                Self::PACKET_ACK_SEED,
                client_id.as_bytes(),
                &sequence.to_le_bytes(),
            ],
            &program_id,
        )
    }
}

pub struct IBCAppState;

impl IBCAppState {
    pub const SEED: &'static [u8] = b"app_state";

    /// Get app state PDA for IBC applications
    pub fn pda(program_id: Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[Self::SEED], &program_id)
    }
}

pub struct PayloadChunk;

impl PayloadChunk {
    pub const SEED: &'static [u8] = b"payload_chunk";

    /// Get payload chunk PDA
    #[allow(clippy::too_many_arguments)]
    pub fn pda(
        payer: Pubkey,
        client_id: &str,
        sequence: u64,
        payload_index: u8,
        chunk_index: u8,
        program_id: Pubkey,
    ) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[
                Self::SEED,
                payer.as_ref(),
                client_id.as_bytes(),
                &sequence.to_le_bytes(),
                &[payload_index],
                &[chunk_index],
            ],
            &program_id,
        )
    }
}

pub struct ProofChunk;

impl ProofChunk {
    pub const SEED: &'static [u8] = b"proof_chunk";

    /// Get proof chunk PDA
    pub fn pda(
        payer: Pubkey,
        client_id: &str,
        sequence: u64,
        chunk_index: u8,
        program_id: Pubkey,
    ) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[
                Self::SEED,
                payer.as_ref(),
                client_id.as_bytes(),
                &sequence.to_le_bytes(),
                &[chunk_index],
            ],
            &program_id,
        )
    }
}
