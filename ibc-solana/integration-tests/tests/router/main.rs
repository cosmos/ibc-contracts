// SPDX-License-Identifier: Apache-2.0

//! Solana-to-Solana IBC router integration tests.
//!
//! Two independent chains run as separate `ProgramTest` instances. The
//! `Relayer` actor delivers packets between them while the `User` actor
//! initiates sends. All proofs are verified on-chain by the attestation
//! light client.

use anchor_lang::AccountDeserialize;
use ics26_router::ics24;
use integration_tests::{
    admin::Admin,
    assert_commitment_set, assert_commitment_zeroed, assert_receipt_created, attestation,
    attestor::Attestors,
    chain::{
        attestation_lc_accounts, mock_ibc_app_state_pda, Chain, ChainConfig, ChainProgram,
        TEST_CLOCK_TIME,
    },
    deployer::Deployer,
    extract_ack_data, extract_custom_error,
    programs::{AttestationLc, MockIbcApp, TestIbcApp},
    read_commitment,
    relayer::Relayer,
    router::{
        self, AckPacketParams, RecvPacketParams, SendPacketParams, TimeoutPacketParams,
        PROOF_HEIGHT,
    },
    user::User,
    Actor, ASYNC_ACK_NOT_SUPPORTED, PACKET_COMMITMENT_MISMATCH,
};

mod ack_after_timeout;
mod bidirectional;
mod double_ack;
mod double_timeout;
mod empty_ack;
mod error_ack;
mod full_lifecycle;
mod multi_chunk_proof;
mod proof_rejection;
mod recv_after_timeout;
mod replay;
mod sequential;
mod three_chain;
mod timeout;
mod timeout_after_ack;
mod unauthorized_relayer;

async fn read_app_state(chain: &Chain) -> test_ibc_app::state::TestIbcAppState {
    let pda = router::test_ibc_app_state_pda();
    let account = chain
        .get_account(pda)
        .await
        .expect("app state should exist");
    test_ibc_app::state::TestIbcAppState::try_deserialize(&mut &account.data[..])
        .expect("failed to deserialize app state")
}
