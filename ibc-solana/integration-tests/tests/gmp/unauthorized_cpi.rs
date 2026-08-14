// SPDX-License-Identifier: Apache-2.0

use super::*;
use anchor_lang::InstructionData;
use solana_sdk::{instruction::AccountMeta, transaction::Transaction};

/// Calling `ics27_gmp::on_recv_packet` via CPI from an unauthorized program
/// (`test_cpi_proxy`, not `ics26_router`) is rejected with `UnauthorizedRouter`.
#[tokio::test]
async fn test_gmp_unauthorized_cpi_rejected() {
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
    let programs: &[&dyn ChainProgram] = &[&Ics27Gmp, &TestGmpApp, &TestCpiProxy, &attestation_lc];

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
    let gmp_remaining = gmp::build_increment_remaining_accounts(
        gmp_account_pda,
        counter_app_state,
        user_counter_pda,
    );

    // ── Unauthorized CPI rejected ──
    let raw_ix = gmp::build_raw_gmp_on_recv_packet_ix(
        relayer.pubkey(),
        chain_a.client_id(),
        chain_a.counterparty_client_id(),
        sequence,
        &gmp_packet_bytes,
        gmp_remaining.clone(),
    );

    // Build CpiAccountMeta descriptors for each account in the raw instruction
    let cpi_account_metas: Vec<test_cpi_proxy::CpiAccountMeta> = raw_ix
        .accounts
        .iter()
        .map(|m| test_cpi_proxy::CpiAccountMeta {
            is_signer: m.is_signer,
            is_writable: m.is_writable,
        })
        .collect();

    let proxy_data = test_cpi_proxy::instruction::ProxyCpi {
        instruction_data: raw_ix.data,
        account_metas: cpi_account_metas,
    }
    .data();

    // proxy_cpi accounts: target_program, payer, then remaining_accounts
    let mut proxy_accounts = vec![
        AccountMeta::new_readonly(ics27_gmp::ID, false), // target_program
        AccountMeta::new(relayer.pubkey(), true),        // payer
    ];
    // All accounts from the raw instruction become remaining_accounts
    for m in &raw_ix.accounts {
        if m.is_writable {
            proxy_accounts.push(AccountMeta::new(m.pubkey, false));
        } else {
            proxy_accounts.push(AccountMeta::new_readonly(m.pubkey, false));
        }
    }

    let proxy_ix = solana_sdk::instruction::Instruction {
        program_id: test_cpi_proxy::ID,
        accounts: proxy_accounts,
        data: proxy_data,
    };

    let tx = Transaction::new_signed_with_payer(
        &[proxy_ix],
        Some(&relayer.pubkey()),
        &[relayer.keypair()],
        chain_a.blockhash(),
    );

    let err = chain_a
        .process_transaction(tx)
        .await
        .expect_err("CPI from unauthorized program should fail");

    assert_eq!(
        extract_custom_error(&err),
        anchor_error_code(GMPError::UnauthorizedRouter as u32),
        "expected UnauthorizedRouter error"
    );

    // No receipt PDA should have been created
    let receipt_pda = integration_tests::router::derive_receipt_pda(chain_a.client_id(), sequence);
    assert!(
        chain_a.get_account(receipt_pda).await.is_none(),
        "receipt should not exist after rejected unauthorized CPI"
    );
}
