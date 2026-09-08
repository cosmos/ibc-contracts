// SPDX-License-Identifier: Apache-2.0

use super::*;
use solana_sdk::transaction::Transaction;

/// Calling `ics27_gmp::on_recv_packet` directly (not via router CPI) is rejected
/// with `DirectCallNotAllowed`.
#[tokio::test]
async fn test_gmp_direct_call_rejected() {
    // ── Attestors ──
    let attestors = Attestors::new(2);

    // ── Actors ──
    let deployer = Deployer::new();
    let admin = Admin::new();
    let relayer = Relayer::new();
    let user = User::new();

    // ── Test data ──
    let sequence = 1u64;
    let increment_amount = 10u64;

    // ── Chain ──
    let attestation_lc = AttestationLc::new(&attestors);
    let programs: &[&dyn ChainProgram] = &[&Ics27Gmp, &TestGmpApp, &attestation_lc];

    let mut chain_a = Chain::single(&deployer, programs);
    chain_a.prefund(&[&admin, &relayer]);
    let gmp_account_pda = gmp::derive_gmp_account_pda(chain_a.client_id(), &user.pubkey());
    chain_a.prefund_lamports(gmp_account_pda, GMP_ACCOUNT_PREFUND_LAMPORTS);

    // ── Init ──
    chain_a
        .init_with_attestation(&deployer, &admin, &relayer, programs, &attestors)
        .await;

    // ── Build payload ──
    let user_counter_pda = gmp::derive_user_counter_pda(&gmp_account_pda);
    let counter_app_state = chain_a.counter_app_state_pda();
    let solana_payload = gmp::encode_increment_payload(
        counter_app_state,
        user_counter_pda,
        gmp_account_pda,
        increment_amount,
    );
    let gmp_packet_bytes =
        gmp::encode_gmp_packet(&user.pubkey(), &test_gmp_app::ID, &solana_payload);
    let remaining = gmp::build_increment_remaining_accounts(
        gmp_account_pda,
        counter_app_state,
        user_counter_pda,
    );

    // ── Direct call rejected ──
    let ix = gmp::build_raw_gmp_on_recv_packet_ix(
        relayer.pubkey(),
        chain_a.client_id(),
        chain_a.counterparty_client_id(),
        sequence,
        &gmp_packet_bytes,
        remaining,
    );

    let tx = Transaction::new_signed_with_payer(
        &[ix],
        Some(&relayer.pubkey()),
        &[relayer.keypair()],
        chain_a.blockhash(),
    );

    let err = chain_a
        .process_transaction(tx)
        .await
        .expect_err("direct call to on_recv_packet should fail");

    assert_eq!(
        extract_custom_error(&err),
        anchor_error_code(GMPError::DirectCallNotAllowed as u32),
        "expected DirectCallNotAllowed error"
    );

    // No receipt PDA should have been created
    let receipt_pda = integration_tests::router::derive_receipt_pda(chain_a.client_id(), sequence);
    assert!(
        chain_a.get_account(receipt_pda).await.is_none(),
        "receipt should not exist after rejected direct call"
    );
}
