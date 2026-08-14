// SPDX-License-Identifier: Apache-2.0

use super::*;
use solana_ibc_proto::{RawGmpSolanaPayload, RawSolanaAccountMeta};
use solana_sdk::{instruction::AccountMeta, pubkey::Pubkey, system_program};

/// When the target CPI fails (wrong PDA), the entire recv transaction reverts
/// and no receipt or ack is created.
#[tokio::test]
async fn test_gmp_failed_execution_aborts() {
    // ── Attestors (independent per chain) ──
    let attestors_a = Attestors::new(2);
    let attestors_b = Attestors::new(3);

    // ── Actors ──
    let deployer = Deployer::new();
    let admin = Admin::new();
    let relayer = Relayer::new();
    let user = User::new();

    // ── Test data ──
    let sequence = 1u64;
    let increment_amount = 10u64;

    // ── Chains ──
    let attestation_lc_a = AttestationLc::new(&attestors_a);
    let programs_a: &[&dyn ChainProgram] = &[&Ics27Gmp, &TestGmpApp, &attestation_lc_a];

    let attestation_lc_b = AttestationLc::new(&attestors_b);
    let programs_b: &[&dyn ChainProgram] = &[&Ics27Gmp, &TestGmpApp, &attestation_lc_b];

    let (mut chain_a, mut chain_b) = Chain::pair(&deployer, programs_a, programs_b);
    chain_a.prefund(&[&admin, &relayer, &user]);
    chain_b.prefund(&[&admin, &relayer]);

    let gmp_account_pda = gmp::derive_gmp_account_pda(chain_b.client_id(), &user.pubkey());
    chain_b.prefund_lamports(gmp_account_pda, GMP_ACCOUNT_PREFUND_LAMPORTS);

    let fake_counter_app_state = Pubkey::new_unique();
    chain_b.prefund_lamports(fake_counter_app_state, 1_000_000);

    // ── Init ──
    chain_a
        .init_with_attestation(&deployer, &admin, &relayer, programs_a, &attestors_a)
        .await;
    chain_b
        .init_with_attestation(&deployer, &admin, &relayer, programs_b, &attestors_b)
        .await;

    // ── Build payload ──
    let user_counter_pda = gmp::derive_user_counter_pda(&gmp_account_pda);

    let mut ix_data = integration_tests::accounts::anchor_discriminator("increment").to_vec();
    ix_data.extend_from_slice(&increment_amount.to_le_bytes());

    let bad_payload = RawGmpSolanaPayload {
        data: ix_data,
        accounts: vec![
            RawSolanaAccountMeta {
                pubkey: fake_counter_app_state.to_bytes().to_vec(),
                is_signer: false,
                is_writable: true,
            },
            RawSolanaAccountMeta {
                pubkey: user_counter_pda.to_bytes().to_vec(),
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
        prefund_lamports: GMP_PAYLOAD_PREFUND_LAMPORTS,
    };

    let gmp_packet_bytes = gmp::encode_gmp_packet(&user.pubkey(), &test_gmp_app::ID, &bad_payload);

    // ── Send on Chain A ──
    let commitment_pda = user
        .send_call(
            &mut chain_a,
            GmpSendCallParams {
                sequence,
                timeout_timestamp: GMP_TIMEOUT,
                receiver: &test_gmp_app::ID.to_string(),
                payload: bad_payload.encode_to_vec(),
            },
        )
        .await
        .expect("send_call failed");

    // ── Build attestation proof for recv on Chain B ──
    let packet_commitment = read_commitment(&chain_a, commitment_pda).await;
    let recv_entry = attestation::packet_commitment_entry(
        chain_b.counterparty_client_id(),
        sequence,
        packet_commitment,
    );
    let recv_proof =
        attestation::build_packet_membership_proof(&attestors_b, PROOF_HEIGHT, &[recv_entry]);
    let recv_proof_bytes = attestation::serialize_proof(&recv_proof);

    // ── Upload chunks and attempt recv on Chain B ──
    let (b_payload, b_proof) = relayer
        .upload_chunks(&mut chain_b, sequence, &gmp_packet_bytes, &recv_proof_bytes)
        .await
        .expect("upload recv chunks failed");

    let remaining = vec![
        AccountMeta::new(gmp_account_pda, false),
        AccountMeta::new_readonly(test_gmp_app::ID, false),
        AccountMeta::new(fake_counter_app_state, false),
        AccountMeta::new(user_counter_pda, false),
        AccountMeta::new(gmp_account_pda, false),
        AccountMeta::new(gmp_account_pda, false),
        AccountMeta::new_readonly(system_program::ID, false),
    ];

    let err = relayer
        .gmp_recv_packet(
            &mut chain_b,
            GmpRecvPacketParams {
                sequence,
                payload_chunk_pda: b_payload,
                proof_chunk_pda: b_proof,
                remaining_accounts: remaining,
            },
        )
        .await
        .expect_err("recv_packet with bad PDA should fail");

    // The error is a CPI failure — check that it's a non-zero custom error
    let code = extract_custom_error(&err);
    assert_ne!(code, 0, "expected non-zero custom error from failed CPI");

    // No receipt or ack should exist on Chain B
    let receipt_pda = integration_tests::router::derive_receipt_pda(chain_b.client_id(), sequence);
    assert!(
        chain_b.get_account(receipt_pda).await.is_none(),
        "receipt should not exist after failed execution"
    );
}
