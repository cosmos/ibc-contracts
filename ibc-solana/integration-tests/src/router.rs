// SPDX-License-Identifier: Apache-2.0

//! ICS26 Router instruction builders for the full packet lifecycle.
//!
//! Covers `send_packet`, `recv_packet`, `ack_packet`, `timeout_packet`,
//! payload/proof chunk upload and cleanup, access-manager transfer
//! operations and on-chain state readers.

use crate::accounts::anchor_discriminator;
use crate::chain::{Chain, LcAccounts};
use anchor_lang::AnchorSerialize;
use ics26_router::state::{Packet, Payload, RouterState, CHUNK_DATA_SIZE};
use solana_ibc_sdk::access_manager::instructions as am_sdk;
use solana_ibc_sdk::ics26_router::instructions as router_sdk;
use solana_ibc_sdk::ics26_router::types::{
    Delivery, MsgAckPacket, MsgCleanupChunks, MsgPacket, MsgPayload, MsgProof, MsgRecvPacket,
    MsgTimeoutPacket, MsgUploadChunk,
};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
};

/// Default source port used by `test_ibc_app`.
pub const PORT_ID: &str = "transfer";
/// Default destination port used by `test_ibc_app`.
pub const DEST_PORT: &str = "transfer";
/// Default proof height used for packet verification.
pub const PROOF_HEIGHT: u64 = 100;

/// Compute a default timeout timestamp (~24 hours after `clock_time`).
pub const fn test_timeout(clock_time: i64) -> u64 {
    clock_time as u64 + 86_000
}

/// Derive the `test_ibc_app` state PDA.
pub fn test_ibc_app_state_pda() -> Pubkey {
    solana_ibc_sdk::pda::ibc_app::app_state_pda(&test_ibc_app::ID).0
}

/// Derive the packet receipt PDA for a given `dest_client` and `sequence`.
pub fn derive_receipt_pda(dest_client: &str, sequence: u64) -> Pubkey {
    router_sdk::RecvPacket::packet_receipt_pda(dest_client, sequence, &ics26_router::ID).0
}

// ── Send ────────────────────────────────────────────────────────────────

/// Parameters for building a `send_packet` instruction via `test_ibc_app`.
pub struct SendPacketParams<'a> {
    /// Packet sequence number (must match the router's next sequence).
    pub sequence: u64,
    /// Raw payload data to include in the packet.
    pub packet_data: &'a [u8],
}

/// Output of [`build_send_packet_ix`] containing the instruction,
/// commitment PDA and reconstructed packet for downstream recv/ack/timeout.
pub struct SendResult {
    /// The `send_packet` instruction to submit.
    pub ix: Instruction,
    /// PDA where the packet commitment is stored.
    pub commitment_pda: Pubkey,
    /// Reconstructed packet matching what the router committed.
    pub packet: Packet,
}

/// Build a `send_packet` instruction through `test_ibc_app`.
pub fn build_send_packet_ix(
    user: Pubkey,
    client_id: &str,
    counterparty_client_id: &str,
    clock_time: i64,
    lc: &LcAccounts,
    params: SendPacketParams<'_>,
) -> SendResult {
    let timeout = test_timeout(clock_time);

    let (app_state_pda, _) = solana_ibc_sdk::pda::ibc_app::app_state_pda(&test_ibc_app::ID);
    let (router_state_pda, _) = router_sdk::Initialize::router_state_pda(&ics26_router::ID);
    let (ibc_app_pda, _) = router_sdk::AddIbcApp::ibc_app_pda(PORT_ID, &ics26_router::ID);
    let (client_pda, _) = router_sdk::AddClient::client_pda(client_id, &ics26_router::ID);
    let (commitment_pda, _) = router_sdk::SendPacket::packet_commitment_pda(
        client_id,
        params.sequence,
        &ics26_router::ID,
    );

    let msg = test_ibc_app::instructions::SendPacketMsg {
        source_client: client_id.to_string(),
        source_port: PORT_ID.to_string(),
        dest_port: DEST_PORT.to_string(),
        version: "1".to_string(),
        encoding: "json".to_string(),
        packet_data: params.packet_data.to_vec(),
        timeout_timestamp: timeout,
        sequence: params.sequence,
    };
    let mut data = anchor_discriminator("send_packet").to_vec();
    msg.serialize(&mut data).unwrap();

    let ix = Instruction {
        program_id: test_ibc_app::ID,
        accounts: vec![
            AccountMeta::new(app_state_pda, false),
            AccountMeta::new(user, true),
            AccountMeta::new_readonly(router_state_pda, false),
            AccountMeta::new_readonly(ibc_app_pda, false),
            AccountMeta::new(commitment_pda, false),
            AccountMeta::new_readonly(client_pda, false),
            AccountMeta::new_readonly(lc.program_id, false),
            AccountMeta::new_readonly(lc.client_state, false),
            AccountMeta::new_readonly(lc.consensus_state, false),
            AccountMeta::new_readonly(ics26_router::ID, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data,
    };

    let packet = Packet {
        sequence: params.sequence,
        source_client: client_id.to_string(),
        dest_client: counterparty_client_id.to_string(),
        timeout_timestamp: timeout,
        payloads: vec![Payload {
            source_port: PORT_ID.to_string(),
            dest_port: DEST_PORT.to_string(),
            version: "1".to_string(),
            encoding: "json".to_string(),
            value: params.packet_data.to_vec(),
        }],
    };

    SendResult {
        ix,
        commitment_pda,
        packet,
    }
}

// ── Recv ────────────────────────────────────────────────────────────────

/// Parameters for building a `recv_packet` instruction.
pub struct RecvPacketParams<'a> {
    /// Packet sequence number.
    pub sequence: u64,
    /// PDA of the uploaded payload chunk.
    pub payload_chunk_pda: Pubkey,
    /// PDA of the uploaded proof chunk.
    pub proof_chunk_pda: Pubkey,
    /// IBC port identifier.
    pub port_id: &'a str,
    /// IBC application version string.
    pub version: &'a str,
    /// Payload encoding (e.g. `"json"`, `"application/x-protobuf"`).
    pub encoding: &'a str,
    /// Target IBC application program ID.
    pub app_program: Pubkey,
    /// Target IBC application state PDA.
    pub app_state_pda: Pubkey,
    /// Additional accounts appended after the standard layout.
    pub extra_remaining_accounts: Vec<AccountMeta>,
}

impl Default for RecvPacketParams<'_> {
    fn default() -> Self {
        Self {
            sequence: 0,
            payload_chunk_pda: Pubkey::default(),
            proof_chunk_pda: Pubkey::default(),
            port_id: PORT_ID,
            version: "1",
            encoding: "json",
            app_program: test_ibc_app::ID,
            app_state_pda: test_ibc_app_state_pda(),
            extra_remaining_accounts: vec![],
        }
    }
}

/// Output of a `recv_packet` instruction build.
#[derive(Debug)]
pub struct RecvResult {
    /// The `recv_packet` instruction to submit.
    pub ix: Instruction,
    /// PDA where the packet receipt is stored.
    pub receipt_pda: Pubkey,
    /// PDA where the acknowledgement is stored.
    pub ack_pda: Pubkey,
}

/// Build a `recv_packet` instruction with a single proof chunk.
pub fn build_recv_packet_ix(
    relayer: Pubkey,
    dest_client: &str,
    source_client: &str,
    clock_time: i64,
    lc: &LcAccounts,
    params: RecvPacketParams<'_>,
) -> RecvResult {
    let timeout = test_timeout(clock_time);
    let am_pda = derive_am_pda(access_manager::ID);

    let (receipt_pda, _) =
        router_sdk::RecvPacket::packet_receipt_pda(dest_client, params.sequence, &ics26_router::ID);
    let (ack_pda, _) =
        router_sdk::RecvPacket::packet_ack_pda(dest_client, params.sequence, &ics26_router::ID);

    let msg = MsgRecvPacket {
        packet: MsgPacket {
            sequence: params.sequence,
            source_client: source_client.to_string(),
            dest_client: dest_client.to_string(),
            timeout_timestamp: timeout,
            payloads: vec![MsgPayload {
                source_port: params.port_id.to_string(),
                dest_port: params.port_id.to_string(),
                version: params.version.to_string(),
                encoding: params.encoding.to_string(),
                data: Delivery::Chunked { total_chunks: 1 },
            }],
        },
        proof: MsgProof {
            height: PROOF_HEIGHT,
            data: Delivery::Chunked { total_chunks: 1 },
        },
    };

    let mut remaining = vec![
        AccountMeta::new(params.payload_chunk_pda, false),
        AccountMeta::new(params.proof_chunk_pda, false),
    ];
    remaining.extend(params.extra_remaining_accounts);

    let ix = router_sdk::RecvPacket::builder(&ics26_router::ID)
        .accounts(router_sdk::RecvPacketAccounts {
            access_manager: am_pda,
            ibc_app_program: params.app_program,
            ibc_app_state: params.app_state_pda,
            relayer,
            light_client_program: lc.program_id,
            client_state: lc.client_state,
            consensus_state: lc.consensus_state,
            dest_port: params.port_id.as_bytes(),
            dest_client,
            sequence: params.sequence,
        })
        .args(&msg)
        .remaining_accounts(remaining)
        .build();

    RecvResult {
        ix,
        receipt_pda,
        ack_pda,
    }
}

/// Build a `recv_packet` instruction with multiple proof chunks.
pub fn build_recv_packet_ix_multi_proof(
    relayer: Pubkey,
    dest_client: &str,
    source_client: &str,
    clock_time: i64,
    lc: &LcAccounts,
    params: RecvPacketParams<'_>,
    proof_chunk_pdas: &[Pubkey],
) -> RecvResult {
    let total_proof_chunks = proof_chunk_pdas.len() as u8;
    let timeout = test_timeout(clock_time);
    let am_pda = derive_am_pda(access_manager::ID);

    let (receipt_pda, _) =
        router_sdk::RecvPacket::packet_receipt_pda(dest_client, params.sequence, &ics26_router::ID);
    let (ack_pda, _) =
        router_sdk::RecvPacket::packet_ack_pda(dest_client, params.sequence, &ics26_router::ID);

    let msg = MsgRecvPacket {
        packet: MsgPacket {
            sequence: params.sequence,
            source_client: source_client.to_string(),
            dest_client: dest_client.to_string(),
            timeout_timestamp: timeout,
            payloads: vec![MsgPayload {
                source_port: params.port_id.to_string(),
                dest_port: params.port_id.to_string(),
                version: params.version.to_string(),
                encoding: params.encoding.to_string(),
                data: Delivery::Chunked { total_chunks: 1 },
            }],
        },
        proof: MsgProof {
            height: PROOF_HEIGHT,
            data: Delivery::Chunked {
                total_chunks: total_proof_chunks,
            },
        },
    };

    let mut remaining = vec![AccountMeta::new(params.payload_chunk_pda, false)];
    remaining.extend(
        proof_chunk_pdas
            .iter()
            .map(|pda| AccountMeta::new(*pda, false)),
    );
    remaining.extend(params.extra_remaining_accounts);

    let ix = router_sdk::RecvPacket::builder(&ics26_router::ID)
        .accounts(router_sdk::RecvPacketAccounts {
            access_manager: am_pda,
            ibc_app_program: params.app_program,
            ibc_app_state: params.app_state_pda,
            relayer,
            light_client_program: lc.program_id,
            client_state: lc.client_state,
            consensus_state: lc.consensus_state,
            dest_port: params.port_id.as_bytes(),
            dest_client,
            sequence: params.sequence,
        })
        .args(&msg)
        .remaining_accounts(remaining)
        .build();

    RecvResult {
        ix,
        receipt_pda,
        ack_pda,
    }
}

// ── Ack ─────────────────────────────────────────────────────────────────

/// Parameters for building an `ack_packet` instruction.
pub struct AckPacketParams<'a> {
    /// Packet sequence number.
    pub sequence: u64,
    /// Raw acknowledgement bytes from the destination chain.
    pub acknowledgement: Vec<u8>,
    /// PDA of the uploaded payload chunk.
    pub payload_chunk_pda: Pubkey,
    /// PDA of the uploaded proof chunk.
    pub proof_chunk_pda: Pubkey,
    /// IBC port identifier.
    pub port_id: &'a str,
    /// IBC application version string.
    pub version: &'a str,
    /// Payload encoding.
    pub encoding: &'a str,
    /// Target IBC application program ID.
    pub app_program: Pubkey,
    /// Target IBC application state PDA.
    pub app_state_pda: Pubkey,
    /// Additional accounts appended after the standard layout.
    pub extra_remaining_accounts: Vec<AccountMeta>,
}

impl Default for AckPacketParams<'_> {
    fn default() -> Self {
        Self {
            sequence: 0,
            acknowledgement: vec![],
            payload_chunk_pda: Pubkey::default(),
            proof_chunk_pda: Pubkey::default(),
            port_id: PORT_ID,
            version: "1",
            encoding: "json",
            app_program: test_ibc_app::ID,
            app_state_pda: test_ibc_app_state_pda(),
            extra_remaining_accounts: vec![],
        }
    }
}

/// Build an `ack_packet` instruction with a single proof chunk.
///
/// Returns `(instruction, commitment_pda)`.
pub fn build_ack_packet_ix(
    relayer: Pubkey,
    source_client: &str,
    dest_client: &str,
    clock_time: i64,
    lc: &LcAccounts,
    params: AckPacketParams<'_>,
) -> (Instruction, Pubkey) {
    let timeout = test_timeout(clock_time);
    let am_pda = derive_am_pda(access_manager::ID);

    let (commitment_pda, _) = router_sdk::AckPacket::packet_commitment_pda(
        source_client,
        params.sequence,
        &ics26_router::ID,
    );

    let msg = MsgAckPacket {
        packet: MsgPacket {
            sequence: params.sequence,
            source_client: source_client.to_string(),
            dest_client: dest_client.to_string(),
            timeout_timestamp: timeout,
            payloads: vec![MsgPayload {
                source_port: params.port_id.to_string(),
                dest_port: params.port_id.to_string(),
                version: params.version.to_string(),
                encoding: params.encoding.to_string(),
                data: Delivery::Chunked { total_chunks: 1 },
            }],
        },
        acknowledgement: params.acknowledgement,
        proof: MsgProof {
            height: PROOF_HEIGHT,
            data: Delivery::Chunked { total_chunks: 1 },
        },
    };

    let mut remaining = vec![
        AccountMeta::new(params.payload_chunk_pda, false),
        AccountMeta::new(params.proof_chunk_pda, false),
    ];
    remaining.extend(params.extra_remaining_accounts);

    let ix = router_sdk::AckPacket::builder(&ics26_router::ID)
        .accounts(router_sdk::AckPacketAccounts {
            access_manager: am_pda,
            ibc_app_program: params.app_program,
            ibc_app_state: params.app_state_pda,
            relayer,
            light_client_program: lc.program_id,
            client_state: lc.client_state,
            consensus_state: lc.consensus_state,
            source_port: params.port_id.as_bytes(),
            source_client,
            sequence: params.sequence,
        })
        .args(&msg)
        .remaining_accounts(remaining)
        .build();

    (ix, commitment_pda)
}

/// Build an `ack_packet` instruction with multiple proof chunks.
pub fn build_ack_packet_ix_multi_proof(
    relayer: Pubkey,
    source_client: &str,
    dest_client: &str,
    clock_time: i64,
    lc: &LcAccounts,
    params: AckPacketParams<'_>,
    proof_chunk_pdas: &[Pubkey],
) -> (Instruction, Pubkey) {
    let total_proof_chunks = proof_chunk_pdas.len() as u8;
    let timeout = test_timeout(clock_time);
    let am_pda = derive_am_pda(access_manager::ID);

    let (commitment_pda, _) = router_sdk::AckPacket::packet_commitment_pda(
        source_client,
        params.sequence,
        &ics26_router::ID,
    );

    let msg = MsgAckPacket {
        packet: MsgPacket {
            sequence: params.sequence,
            source_client: source_client.to_string(),
            dest_client: dest_client.to_string(),
            timeout_timestamp: timeout,
            payloads: vec![MsgPayload {
                source_port: params.port_id.to_string(),
                dest_port: params.port_id.to_string(),
                version: params.version.to_string(),
                encoding: params.encoding.to_string(),
                data: Delivery::Chunked { total_chunks: 1 },
            }],
        },
        acknowledgement: params.acknowledgement,
        proof: MsgProof {
            height: PROOF_HEIGHT,
            data: Delivery::Chunked {
                total_chunks: total_proof_chunks,
            },
        },
    };

    let mut remaining = vec![AccountMeta::new(params.payload_chunk_pda, false)];
    remaining.extend(
        proof_chunk_pdas
            .iter()
            .map(|pda| AccountMeta::new(*pda, false)),
    );
    remaining.extend(params.extra_remaining_accounts);

    let ix = router_sdk::AckPacket::builder(&ics26_router::ID)
        .accounts(router_sdk::AckPacketAccounts {
            access_manager: am_pda,
            ibc_app_program: params.app_program,
            ibc_app_state: params.app_state_pda,
            relayer,
            light_client_program: lc.program_id,
            client_state: lc.client_state,
            consensus_state: lc.consensus_state,
            source_port: params.port_id.as_bytes(),
            source_client,
            sequence: params.sequence,
        })
        .args(&msg)
        .remaining_accounts(remaining)
        .build();

    (ix, commitment_pda)
}

// ── Timeout ────────────────────────────────────────────────────────────

/// Parameters for building a `timeout_packet` instruction.
pub struct TimeoutPacketParams<'a> {
    /// Packet sequence number.
    pub sequence: u64,
    /// PDA of the uploaded payload chunk.
    pub payload_chunk_pda: Pubkey,
    /// PDA of the uploaded proof chunk.
    pub proof_chunk_pda: Pubkey,
    /// IBC port identifier.
    pub port_id: &'a str,
    /// IBC application version string.
    pub version: &'a str,
    /// Payload encoding.
    pub encoding: &'a str,
    /// Target IBC application program ID.
    pub app_program: Pubkey,
    /// Target IBC application state PDA.
    pub app_state_pda: Pubkey,
    /// Additional accounts appended after the standard layout.
    pub extra_remaining_accounts: Vec<AccountMeta>,
}

impl Default for TimeoutPacketParams<'_> {
    fn default() -> Self {
        Self {
            sequence: 0,
            payload_chunk_pda: Pubkey::default(),
            proof_chunk_pda: Pubkey::default(),
            port_id: PORT_ID,
            version: "1",
            encoding: "json",
            app_program: test_ibc_app::ID,
            app_state_pda: test_ibc_app_state_pda(),
            extra_remaining_accounts: vec![],
        }
    }
}

/// Build a `timeout_packet` instruction.
///
/// Returns `(instruction, commitment_pda)`.
pub fn build_timeout_packet_ix(
    relayer: Pubkey,
    source_client: &str,
    dest_client: &str,
    clock_time: i64,
    lc: &LcAccounts,
    params: TimeoutPacketParams<'_>,
) -> (Instruction, Pubkey) {
    let timeout = test_timeout(clock_time);
    let am_pda = derive_am_pda(access_manager::ID);

    let (commitment_pda, _) = router_sdk::TimeoutPacket::packet_commitment_pda(
        source_client,
        params.sequence,
        &ics26_router::ID,
    );

    let msg = MsgTimeoutPacket {
        packet: MsgPacket {
            sequence: params.sequence,
            source_client: source_client.to_string(),
            dest_client: dest_client.to_string(),
            timeout_timestamp: timeout,
            payloads: vec![MsgPayload {
                source_port: params.port_id.to_string(),
                dest_port: params.port_id.to_string(),
                version: params.version.to_string(),
                encoding: params.encoding.to_string(),
                data: Delivery::Chunked { total_chunks: 1 },
            }],
        },
        proof: MsgProof {
            height: PROOF_HEIGHT,
            data: Delivery::Chunked { total_chunks: 1 },
        },
    };

    let mut remaining = vec![
        AccountMeta::new(params.payload_chunk_pda, false),
        AccountMeta::new(params.proof_chunk_pda, false),
    ];
    remaining.extend(params.extra_remaining_accounts);

    let ix = router_sdk::TimeoutPacket::builder(&ics26_router::ID)
        .accounts(router_sdk::TimeoutPacketAccounts {
            access_manager: am_pda,
            ibc_app_program: params.app_program,
            ibc_app_state: params.app_state_pda,
            relayer,
            light_client_program: lc.program_id,
            client_state: lc.client_state,
            consensus_state: lc.consensus_state,
            source_port: params.port_id.as_bytes(),
            source_client,
            sequence: params.sequence,
        })
        .args(&msg)
        .remaining_accounts(remaining)
        .build();

    (ix, commitment_pda)
}

// ── Upload Chunks ─────────────────────────────────────────────────────

/// Build an `upload_payload_chunk` instruction.
///
/// Returns `(instruction, chunk_pda)`.
pub fn build_upload_payload_chunk_ix(
    relayer: Pubkey,
    client_id: &str,
    sequence: u64,
    payload_data: Vec<u8>,
) -> (Instruction, Pubkey) {
    let am_pda = derive_am_pda(access_manager::ID);
    let (chunk_pda, _) = solana_ibc_sdk::pda::ics26_router::payload_chunk_pda(
        &relayer,
        client_id,
        sequence,
        0,
        0,
        &ics26_router::ID,
    );

    let msg = MsgUploadChunk {
        client_id: client_id.to_string(),
        sequence,
        payload_index: 0,
        chunk_index: 0,
        chunk_data: payload_data,
    };

    let ix = router_sdk::UploadPayloadChunk::builder(&ics26_router::ID)
        .accounts(router_sdk::UploadPayloadChunkAccounts {
            access_manager: am_pda,
            chunk: chunk_pda,
            relayer,
        })
        .args(&msg)
        .build();

    (ix, chunk_pda)
}

/// Build an `upload_proof_chunk` instruction at chunk index 0.
///
/// Returns `(instruction, chunk_pda)`.
pub fn build_upload_proof_chunk_ix(
    relayer: Pubkey,
    client_id: &str,
    sequence: u64,
    proof_data: Vec<u8>,
) -> (Instruction, Pubkey) {
    build_upload_proof_chunk_ix_at(relayer, client_id, sequence, 0, proof_data)
}

/// Build an `upload_proof_chunk` instruction at a specific `chunk_index`.
///
/// Returns `(instruction, chunk_pda)`.
pub fn build_upload_proof_chunk_ix_at(
    relayer: Pubkey,
    client_id: &str,
    sequence: u64,
    chunk_index: u8,
    proof_data: Vec<u8>,
) -> (Instruction, Pubkey) {
    let am_pda = derive_am_pda(access_manager::ID);
    let (chunk_pda, _) = solana_ibc_sdk::pda::ics26_router::proof_chunk_pda(
        &relayer,
        client_id,
        sequence,
        chunk_index,
        &ics26_router::ID,
    );

    let msg = MsgUploadChunk {
        client_id: client_id.to_string(),
        sequence,
        payload_index: 0,
        chunk_index,
        chunk_data: proof_data,
    };

    let ix = router_sdk::UploadProofChunk::builder(&ics26_router::ID)
        .accounts(router_sdk::UploadProofChunkAccounts {
            access_manager: am_pda,
            chunk: chunk_pda,
            relayer,
        })
        .args(&msg)
        .build();

    (ix, chunk_pda)
}

// ── Cleanup Chunks ───────────────────────────────────────────────────

/// Build a `cleanup_chunks` instruction to reclaim rent from consumed chunk accounts.
pub fn build_cleanup_chunks_ix(
    relayer: Pubkey,
    client_id: &str,
    sequence: u64,
    payload_chunk_pda: Pubkey,
    proof_chunk_pda: Pubkey,
) -> Instruction {
    let am_pda = derive_am_pda(access_manager::ID);

    let msg = MsgCleanupChunks {
        client_id: client_id.to_string(),
        sequence,
        payload_chunks: vec![1],
        total_proof_chunks: 1,
    };

    router_sdk::CleanupChunks::builder(&ics26_router::ID)
        .accounts(router_sdk::CleanupChunksAccounts {
            access_manager: am_pda,
            relayer,
        })
        .args(&msg)
        .remaining_accounts([
            AccountMeta::new(payload_chunk_pda, false),
            AccountMeta::new(proof_chunk_pda, false),
        ])
        .build()
}

// ── AM transfer instruction builders ────────────────────────────────────

fn derive_router_state_pda() -> Pubkey {
    router_sdk::Initialize::router_state_pda(&ics26_router::ID).0
}

fn derive_am_pda(am_program_id: Pubkey) -> Pubkey {
    am_sdk::Initialize::access_manager_pda(&am_program_id).0
}

/// Build an ICS26 `propose_access_manager_transfer` instruction.
pub fn build_ics26_propose_am_transfer_ix(
    admin: Pubkey,
    new_access_manager: Pubkey,
) -> Instruction {
    let am_pda = derive_am_pda(access_manager::ID);

    router_sdk::ProposeAccessManagerTransfer::builder(&ics26_router::ID)
        .accounts(router_sdk::ProposeAccessManagerTransferAccounts {
            access_manager: am_pda,
            admin,
        })
        .args(&router_sdk::ProposeAccessManagerTransferArgs { new_access_manager })
        .build()
}

/// Build an ICS26 `accept_access_manager_transfer` instruction.
pub fn build_ics26_accept_am_transfer_ix(admin: Pubkey, new_am_program_id: Pubkey) -> Instruction {
    let new_am_pda = derive_am_pda(new_am_program_id);

    router_sdk::AcceptAccessManagerTransfer::builder(&ics26_router::ID)
        .accounts(router_sdk::AcceptAccessManagerTransferAccounts {
            new_am_state: new_am_pda,
            admin,
        })
        .build()
}

/// Build an ICS26 `cancel_access_manager_transfer` instruction.
pub fn build_ics26_cancel_am_transfer_ix(admin: Pubkey) -> Instruction {
    let am_pda = derive_am_pda(access_manager::ID);

    router_sdk::CancelAccessManagerTransfer::builder(&ics26_router::ID)
        .accounts(router_sdk::CancelAccessManagerTransferAccounts {
            am_state: am_pda,
            admin,
        })
        .build()
}

// ── Chunk helpers ────────────────────────────────────────────────────────

/// Split a byte slice into chunks that fit within the router's upload limit.
///
/// Used when a serialized proof exceeds `CHUNK_DATA_SIZE` bytes and must be
/// delivered via [`Relayer::upload_chunks_with_multi_proof`].
pub fn split_into_chunks(data: &[u8]) -> Vec<Vec<u8>> {
    data.chunks(CHUNK_DATA_SIZE).map(<[u8]>::to_vec).collect()
}

// ── State readers ───────────────────────────────────────────────────────

/// Deserialize the on-chain `RouterState` from its PDA.
pub async fn read_router_state(chain: &Chain) -> RouterState {
    use anchor_lang::AccountDeserialize;

    let pda = derive_router_state_pda();
    let account = chain
        .get_account(pda)
        .await
        .expect("RouterState should exist");
    RouterState::try_deserialize(&mut &account.data[..]).expect("deserialize RouterState")
}

/// Deserialize the `TestIbcAppState` from its PDA.
pub async fn read_test_ibc_app_state(chain: &Chain) -> test_ibc_app::state::TestIbcAppState {
    use anchor_lang::AccountDeserialize;

    let pda = test_ibc_app_state_pda();
    let account = chain
        .get_account(pda)
        .await
        .expect("TestIbcAppState should exist");
    test_ibc_app::state::TestIbcAppState::try_deserialize(&mut &account.data[..])
        .expect("deserialize TestIbcAppState")
}
