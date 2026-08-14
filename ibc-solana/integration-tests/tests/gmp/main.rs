// SPDX-License-Identifier: Apache-2.0

//! Solana-to-Solana GMP integration tests.
//!
//! Two independent chains run as separate `ProgramTest` instances. The
//! `Relayer` actor bridges packets between them while the `User` actor
//! initiates GMP calls via `send_call`.
//!
//! All tests use the attestation light client with real ECDSA proofs
//! verified on-chain. Each chain gets its own independent attestor set.

use anchor_lang::AccountDeserialize;
use ics27_gmp::errors::GMPError;
use integration_tests::{
    admin::Admin,
    anchor_error_code, assert_commitment_set, assert_commitment_zeroed, assert_receipt_created,
    attestation,
    attestor::Attestors,
    chain::{Chain, ChainConfig, ChainProgram, TEST_CLOCK_TIME},
    deployer::Deployer,
    extract_ack_data, extract_custom_error, gmp,
    gmp::{GmpAckPacketParams, GmpRecvPacketParams, GmpSendCallParams},
    programs::{AttestationLc, Ics27Gmp, TestCpiProxy, TestGmpApp},
    read_commitment,
    relayer::Relayer,
    router::PROOF_HEIGHT,
    user::User,
    Actor,
};
use prost::Message as ProstMessage;
use solana_sdk::pubkey::Pubkey;

mod attestation_lifecycle;
mod attestation_roundtrip;
mod bidirectional;
mod direct_call_rejected;
mod failed_execution;
mod full_lifecycle;
mod multi_user;
mod multiple_calls;
mod prefunded_pda;
mod signer_exploit;
mod signer_exploit_sol_transfer;
mod three_chain;
mod timeout;
mod timeout_too_long;
mod unauthorized_cpi;

/// GMP timeout must match `router::test_timeout(TEST_CLOCK_TIME)` so that
/// the commitment computed by `send_call` agrees with the ack/recv packet builders.
const GMP_TIMEOUT: u64 = TEST_CLOCK_TIME as u64 + 86_000;
/// GMP `send_call` with a timeout exceeding `MAX_TIMEOUT_DURATION` is rejected.
const GMP_TIMEOUT_TOO_LONG: u64 = TEST_CLOCK_TIME as u64 + 86_400;

/// Lamports prefunded to the `GMPAccount` PDA so the counter program can pay
/// rent for the user-counter PDA it creates on receipt.
const GMP_ACCOUNT_PREFUND_LAMPORTS: u64 = 10_000_000;
/// Lamports requested via `RawGmpSolanaPayload.prefund_lamports` to fund the
/// payload PDA before executing the wrapped instruction.
const GMP_PAYLOAD_PREFUND_LAMPORTS: u64 = 5_000_000;

async fn read_user_counter(chain: &Chain, pda: Pubkey) -> test_gmp_app::state::UserCounter {
    let account = chain
        .get_account(pda)
        .await
        .expect("UserCounter should exist");
    test_gmp_app::state::UserCounter::try_deserialize(&mut &account.data[..])
        .expect("deserialize UserCounter")
}

async fn read_counter_app_state(
    chain: &Chain,
    pda: Pubkey,
) -> test_gmp_app::state::CounterAppState {
    let account = chain
        .get_account(pda)
        .await
        .expect("CounterAppState should exist");
    test_gmp_app::state::CounterAppState::try_deserialize(&mut &account.data[..])
        .expect("deserialize CounterAppState")
}

async fn assert_gmp_result_exists(chain: &Chain, client_id: &str, sequence: u64) {
    let (pda, _) = solana_ibc_gmp_types::GMPCallResult::pda(client_id, sequence, &ics27_gmp::ID);
    let account = chain
        .get_account(pda)
        .await
        .expect("GMPCallResult should exist");
    assert_eq!(account.owner, ics27_gmp::ID);
}
