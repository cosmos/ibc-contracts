// SPDX-License-Identifier: Apache-2.0

//! ICS27 General Message Passing (GMP) instruction builders.
//!
//! Wraps router primitives with GMP-specific port, version and protobuf
//! encoding. Also provides payload encoding helpers for `test_gmp_app`,
//! raw `on_recv_packet` construction for security tests and
//! access-manager transfer operations.

use crate::accounts::anchor_discriminator;
use crate::chain::{Chain, LcAccounts};
use crate::router::RecvResult;
use prost::Message as ProstMessage;
use solana_ibc_proto::{RawGmpPacketData, RawGmpSolanaPayload, RawSolanaAccountMeta};
use solana_ibc_sdk::access_manager::instructions as am_sdk;
use solana_ibc_sdk::ics27_gmp::instructions as gmp_sdk;
use solana_ibc_sdk::ics27_gmp::types::{OnRecvPacketMsg, Payload, SendCallMsg};
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
};

/// IBC port identifier used by `ics27_gmp`.
pub const GMP_PORT_ID: &str = "gmpport";
/// ICS27 application version string.
pub const ICS27_VERSION: &str = "ics27-2";
/// Protobuf content-type used for GMP payload encoding.
pub const ICS27_ENCODING_PROTOBUF: &str = "application/x-protobuf";

// ── Send Call ───────────────────────────────────────────────────────────

/// Parameters for building a GMP `send_call` instruction.
pub struct GmpSendCallParams<'a> {
    /// Packet sequence number.
    pub sequence: u64,
    /// Absolute timeout timestamp (seconds).
    pub timeout_timestamp: u64,
    /// Receiver address on the destination chain.
    pub receiver: &'a str,
    /// Encoded GMP payload bytes.
    pub payload: Vec<u8>,
}

/// Build a GMP `send_call` instruction.
///
/// Returns `(instruction, commitment_pda)`.
pub fn build_gmp_send_call_ix(
    sender: Pubkey,
    payer: Pubkey,
    client_id: &str,
    lc: &LcAccounts,
    params: GmpSendCallParams<'_>,
) -> (Instruction, Pubkey) {
    let (router_state, _) = gmp_sdk::SendCall::router_state_pda(&ics26_router::ID);
    let (commitment_pda, _) =
        gmp_sdk::SendCall::packet_commitment_pda(client_id, params.sequence, &ics26_router::ID);
    let (ibc_app, _) = gmp_sdk::SendCall::ibc_app_pda(&ics26_router::ID);
    let (client, _) = gmp_sdk::SendCall::client_pda(client_id, &ics26_router::ID);

    let msg = SendCallMsg {
        source_client: client_id.to_string(),
        sequence: params.sequence,
        timeout_timestamp: params.timeout_timestamp,
        receiver: params.receiver.to_string(),
        salt: vec![],
        payload: params.payload,
        memo: String::new(),
        encoding: ICS27_ENCODING_PROTOBUF.to_string(),
    };

    let ix = gmp_sdk::SendCall::builder(&ics27_gmp::ID)
        .accounts(gmp_sdk::SendCallAccounts {
            sender,
            payer,
            router_state,
            packet_commitment: commitment_pda,
            ibc_app,
            client,
            light_client_program: lc.program_id,
            client_state: lc.client_state,
            consensus_state: lc.consensus_state,
        })
        .args(&msg)
        .build();

    (ix, commitment_pda)
}

// ── Recv Packet ─────────────────────────────────────────────────────────

/// Parameters for building a GMP `recv_packet` instruction.
pub struct GmpRecvPacketParams {
    /// Packet sequence number.
    pub sequence: u64,
    /// PDA of the uploaded payload chunk.
    pub payload_chunk_pda: Pubkey,
    /// PDA of the uploaded proof chunk.
    pub proof_chunk_pda: Pubkey,
    /// GMP-specific remaining accounts (GMP account PDA, target program, etc.).
    pub remaining_accounts: Vec<AccountMeta>,
}

/// Build a GMP `recv_packet` instruction (delegates to [`router::build_recv_packet_ix`]).
pub fn build_gmp_recv_packet_ix(
    relayer: Pubkey,
    dest_client: &str,
    source_client: &str,
    clock_time: i64,
    lc: &LcAccounts,
    params: GmpRecvPacketParams,
) -> RecvResult {
    crate::router::build_recv_packet_ix(
        relayer,
        dest_client,
        source_client,
        clock_time,
        lc,
        crate::router::RecvPacketParams {
            sequence: params.sequence,
            payload_chunk_pda: params.payload_chunk_pda,
            proof_chunk_pda: params.proof_chunk_pda,
            port_id: GMP_PORT_ID,
            version: ICS27_VERSION,
            encoding: ICS27_ENCODING_PROTOBUF,
            app_program: ics27_gmp::ID,
            app_state_pda: derive_gmp_app_state_pda(),
            extra_remaining_accounts: params.remaining_accounts,
        },
    )
}

// ── Ack Packet ──────────────────────────────────────────────────────────

/// Parameters for building a GMP `ack_packet` instruction.
pub struct GmpAckPacketParams {
    /// Packet sequence number.
    pub sequence: u64,
    /// Raw acknowledgement bytes from the destination chain.
    pub acknowledgement: Vec<u8>,
    /// PDA of the uploaded payload chunk.
    pub payload_chunk_pda: Pubkey,
    /// PDA of the uploaded proof chunk.
    pub proof_chunk_pda: Pubkey,
}

/// Build a GMP `ack_packet` instruction.
///
/// Returns `(instruction, commitment_pda)`.
pub fn build_gmp_ack_packet_ix(
    relayer: Pubkey,
    source_client: &str,
    dest_client: &str,
    clock_time: i64,
    lc: &LcAccounts,
    params: GmpAckPacketParams,
) -> (Instruction, Pubkey) {
    let (result_pda, _) = gmp_sdk::OnAcknowledgementPacket::result_account_pda(
        source_client,
        params.sequence,
        &ics27_gmp::ID,
    );

    crate::router::build_ack_packet_ix(
        relayer,
        source_client,
        dest_client,
        clock_time,
        lc,
        crate::router::AckPacketParams {
            sequence: params.sequence,
            acknowledgement: params.acknowledgement,
            payload_chunk_pda: params.payload_chunk_pda,
            proof_chunk_pda: params.proof_chunk_pda,
            port_id: GMP_PORT_ID,
            version: ICS27_VERSION,
            encoding: ICS27_ENCODING_PROTOBUF,
            app_program: ics27_gmp::ID,
            app_state_pda: derive_gmp_app_state_pda(),
            extra_remaining_accounts: vec![AccountMeta::new(result_pda, false)],
        },
    )
}

// ── Timeout Packet ─────────────────────────────────────────────────────

/// Parameters for building a GMP `timeout_packet` instruction.
pub struct GmpTimeoutPacketParams {
    /// Packet sequence number.
    pub sequence: u64,
    /// PDA of the uploaded payload chunk.
    pub payload_chunk_pda: Pubkey,
    /// PDA of the uploaded proof chunk.
    pub proof_chunk_pda: Pubkey,
}

/// Build a GMP `timeout_packet` instruction.
///
/// Returns `(instruction, commitment_pda)`.
pub fn build_gmp_timeout_packet_ix(
    relayer: Pubkey,
    source_client: &str,
    dest_client: &str,
    clock_time: i64,
    lc: &LcAccounts,
    params: GmpTimeoutPacketParams,
) -> (Instruction, Pubkey) {
    let (result_pda, _) = gmp_sdk::OnTimeoutPacket::result_account_pda(
        source_client,
        params.sequence,
        &ics27_gmp::ID,
    );

    crate::router::build_timeout_packet_ix(
        relayer,
        source_client,
        dest_client,
        clock_time,
        lc,
        crate::router::TimeoutPacketParams {
            sequence: params.sequence,
            payload_chunk_pda: params.payload_chunk_pda,
            proof_chunk_pda: params.proof_chunk_pda,
            port_id: GMP_PORT_ID,
            version: ICS27_VERSION,
            encoding: ICS27_ENCODING_PROTOBUF,
            app_program: ics27_gmp::ID,
            app_state_pda: derive_gmp_app_state_pda(),
            extra_remaining_accounts: vec![AccountMeta::new(result_pda, false)],
        },
    )
}

// ── Payload Encoding ────────────────────────────────────────────────────

/// Encode a `GmpSolanaPayload` for `test_gmp_app::increment`.
///
/// Target accounts layout:
/// - `[0]`: `CounterAppState` (mutable)
/// - `[1]`: `UserCounter` PDA (mutable, `init_if_needed`)
/// - `[2]`: GMP account PDA (`user_authority`, signer via `invoke_signed`)
/// - `[3]`: GMP account PDA (payer, signer + mutable)
/// - `[4]`: system program
pub fn encode_increment_payload(
    counter_app_state: Pubkey,
    user_counter: Pubkey,
    gmp_account_pda: Pubkey,
    amount: u64,
) -> RawGmpSolanaPayload {
    let mut ix_data = anchor_discriminator("increment").to_vec();
    ix_data.extend_from_slice(&amount.to_le_bytes());

    RawGmpSolanaPayload {
        data: ix_data,
        accounts: vec![
            RawSolanaAccountMeta {
                pubkey: counter_app_state.to_bytes().to_vec(),
                is_signer: false,
                is_writable: true,
            },
            RawSolanaAccountMeta {
                pubkey: user_counter.to_bytes().to_vec(),
                is_signer: false,
                is_writable: true,
            },
            RawSolanaAccountMeta {
                pubkey: gmp_account_pda.to_bytes().to_vec(),
                is_signer: true,
                is_writable: false,
            },
            RawSolanaAccountMeta {
                pubkey: gmp_account_pda.to_bytes().to_vec(),
                is_signer: true,
                is_writable: true,
            },
            RawSolanaAccountMeta {
                pubkey: system_program::ID.to_bytes().to_vec(),
                is_signer: false,
                is_writable: false,
            },
        ],
        prefund_lamports: 5_000_000,
    }
}

/// Encode a full GMP packet wrapping a `GmpSolanaPayload`.
pub fn encode_gmp_packet(
    sender: &Pubkey,
    receiver: &Pubkey,
    solana_payload: &RawGmpSolanaPayload,
) -> Vec<u8> {
    let raw_packet = RawGmpPacketData {
        sender: sender.to_string(),
        receiver: receiver.to_string(),
        salt: vec![],
        payload: solana_payload.encode_to_vec(),
        memo: String::new(),
    };

    ics27_gmp::encoding::encode_gmp_packet(
        solana_ibc_gmp_types::GmpPacketData::try_from(raw_packet).expect("valid GMP packet data"),
        ICS27_ENCODING_PROTOBUF,
    )
    .expect("GMP packet encoding should succeed")
}

/// Build `remaining_accounts` entries for GMP `recv_packet` targeting
/// `test_gmp_app::increment`.
pub fn build_increment_remaining_accounts(
    gmp_account_pda: Pubkey,
    counter_app_state: Pubkey,
    user_counter: Pubkey,
) -> Vec<AccountMeta> {
    vec![
        AccountMeta::new(gmp_account_pda, false),
        AccountMeta::new_readonly(test_gmp_app::ID, false),
        AccountMeta::new(counter_app_state, false),
        AccountMeta::new(user_counter, false),
        AccountMeta::new(gmp_account_pda, false),
        AccountMeta::new(gmp_account_pda, false),
        AccountMeta::new_readonly(system_program::ID, false),
    ]
}

/// Derive the GMP account PDA for a given sender and `client_id`.
pub fn derive_gmp_account_pda(client_id: &str, sender: &Pubkey) -> Pubkey {
    let gmp_account = solana_ibc_gmp_types::GMPAccount::new(
        client_id.to_string().try_into().expect("valid client_id"),
        sender.to_string().try_into().expect("valid sender"),
        vec![].try_into().expect("empty salt"),
        &ics27_gmp::ID,
    );
    gmp_account.pda
}

/// Derive the `UserCounter` PDA for `test_gmp_app`.
pub fn derive_user_counter_pda(user: &Pubkey) -> Pubkey {
    let (pda, _) = Pubkey::find_program_address(
        &[test_gmp_app::state::UserCounter::SEED, user.as_ref()],
        &test_gmp_app::ID,
    );
    pda
}

/// Build a raw `on_recv_packet` instruction targeting `ics27_gmp` directly.
///
/// Used by security tests that bypass the router: either as a top-level
/// instruction (direct-call test) or wrapped in `test_cpi_proxy::proxy_cpi`
/// (unauthorized-CPI test).
pub fn build_raw_gmp_on_recv_packet_ix(
    relayer: Pubkey,
    dest_client: &str,
    source_client: &str,
    sequence: u64,
    packet_bytes: &[u8],
    remaining_accounts: Vec<AccountMeta>,
) -> Instruction {
    let msg = OnRecvPacketMsg {
        source_client: source_client.to_string(),
        dest_client: dest_client.to_string(),
        sequence,
        payload: build_gmp_ibc_payload(packet_bytes),
        relayer,
    };

    gmp_sdk::OnRecvPacket::builder(&ics27_gmp::ID)
        .accounts(gmp_sdk::OnRecvPacketAccounts { payer: relayer })
        .args(&msg)
        .remaining_accounts(remaining_accounts)
        .build()
}

/// Build IBC payload for GMP (used in `OnRecvPacketMsg`).
pub fn build_gmp_ibc_payload(packet_bytes: &[u8]) -> Payload {
    Payload {
        source_port: GMP_PORT_ID.to_string(),
        dest_port: GMP_PORT_ID.to_string(),
        version: ICS27_VERSION.to_string(),
        encoding: ICS27_ENCODING_PROTOBUF.to_string(),
        value: packet_bytes.to_vec(),
    }
}

// ── AM transfer instruction builders ────────────────────────────────────

fn derive_gmp_app_state_pda() -> Pubkey {
    gmp_sdk::Initialize::app_state_pda(&ics27_gmp::ID).0
}

fn derive_am_pda(am_program_id: Pubkey) -> Pubkey {
    am_sdk::Initialize::access_manager_pda(&am_program_id).0
}

/// Build a GMP `propose_access_manager_transfer` instruction.
pub fn build_gmp_propose_am_transfer_ix(admin: Pubkey, new_access_manager: Pubkey) -> Instruction {
    let am_pda = derive_am_pda(access_manager::ID);
    gmp_sdk::ProposeAccessManagerTransfer::builder(&ics27_gmp::ID)
        .accounts(gmp_sdk::ProposeAccessManagerTransferAccounts {
            access_manager: am_pda,
            admin,
        })
        .args(&gmp_sdk::ProposeAccessManagerTransferArgs { new_access_manager })
        .build()
}

/// Build a GMP `accept_access_manager_transfer` instruction.
pub fn build_gmp_accept_am_transfer_ix(admin: Pubkey, new_am_program_id: Pubkey) -> Instruction {
    let new_am_state = derive_am_pda(new_am_program_id);
    gmp_sdk::AcceptAccessManagerTransfer::builder(&ics27_gmp::ID)
        .accounts(gmp_sdk::AcceptAccessManagerTransferAccounts {
            new_am_state,
            admin,
        })
        .build()
}

/// Build a GMP `cancel_access_manager_transfer` instruction.
pub fn build_gmp_cancel_am_transfer_ix(admin: Pubkey) -> Instruction {
    let am_state = derive_am_pda(access_manager::ID);
    gmp_sdk::CancelAccessManagerTransfer::builder(&ics27_gmp::ID)
        .accounts(gmp_sdk::CancelAccessManagerTransferAccounts { am_state, admin })
        .build()
}

// ── State readers ───────────────────────────────────────────────────────

/// Deserialize the on-chain `GMPAppState` from its PDA.
pub async fn read_gmp_app_state(chain: &Chain) -> ics27_gmp::state::GMPAppState {
    use anchor_lang::AccountDeserialize;

    let pda = derive_gmp_app_state_pda();
    let account = chain
        .get_account(pda)
        .await
        .expect("GMPAppState should exist");
    ics27_gmp::state::GMPAppState::try_deserialize(&mut &account.data[..])
        .expect("deserialize GMPAppState")
}
