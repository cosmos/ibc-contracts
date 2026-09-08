// SPDX-License-Identifier: Apache-2.0

use super::*;
use integration_tests::{
    attestation, attestor::Attestors, programs::AttestationLc, read_commitment,
    router::PROOF_HEIGHT,
};

/// Full GMP send → recv → ack lifecycle with attestation light client
/// (2-of-2 attestor quorum) instead of mock LC.
#[tokio::test]
async fn test_gmp_attestation_full_lifecycle() {
    // ── Attestors ──
    let attestors = Attestors::new(2);

    // ── Actors ──
    let deployer = Deployer::new();
    let admin = Admin::new();
    let relayer = Relayer::new();
    let user = User::new();

    // ── Test data ──
    let sequence = 1u64;
    let increment_amount = 42u64;

    // ── Chains ──
    let attestation_lc = AttestationLc::new(&attestors);
    let programs: &[&dyn ChainProgram] = &[&Ics27Gmp, &TestGmpApp, &attestation_lc];

    let (mut chain_a, mut chain_b) = Chain::pair(&deployer, programs, programs);
    chain_a.prefund(&[&admin, &relayer, &user]);
    chain_b.prefund(&[&admin, &relayer]);

    let gmp_account_pda = gmp::derive_gmp_account_pda(chain_b.client_id(), &user.pubkey());
    chain_b.prefund_lamports(gmp_account_pda, GMP_ACCOUNT_PREFUND_LAMPORTS);

    // ── Init (includes update_client at PROOF_HEIGHT) ──
    chain_a
        .init_with_attestation(&deployer, &admin, &relayer, programs, &attestors)
        .await;
    chain_b
        .init_with_attestation(&deployer, &admin, &relayer, programs, &attestors)
        .await;

    // ── Build payload ──
    let user_counter_pda = gmp::derive_user_counter_pda(&gmp_account_pda);
    let counter_app_state = chain_b.counter_app_state_pda();

    let solana_payload = gmp::encode_increment_payload(
        counter_app_state,
        user_counter_pda,
        gmp_account_pda,
        increment_amount,
    );
    let gmp_packet_bytes =
        gmp::encode_gmp_packet(&user.pubkey(), &test_gmp_app::ID, &solana_payload);

    // ── User sends GMP call on Chain A ──
    let commitment_pda = user
        .send_call(
            &mut chain_a,
            GmpSendCallParams {
                sequence,
                timeout_timestamp: GMP_TIMEOUT,
                receiver: &test_gmp_app::ID.to_string(),
                payload: solana_payload.encode_to_vec(),
            },
        )
        .await
        .expect("send_call on Chain A failed");

    assert_commitment_set(&chain_a, commitment_pda).await;

    // ── Build attestation proof for recv on Chain B ──
    let packet_commitment = read_commitment(&chain_a, commitment_pda).await;
    let recv_entry = attestation::packet_commitment_entry(
        chain_b.counterparty_client_id(),
        sequence,
        packet_commitment,
    );
    let recv_proof =
        attestation::build_packet_membership_proof(&attestors, PROOF_HEIGHT, &[recv_entry]);
    let recv_proof_bytes = attestation::serialize_proof(&recv_proof);

    // ── Relayer uploads chunks and delivers recv_packet to Chain B ──
    let (b_recv_payload, b_recv_proof) = relayer
        .upload_chunks(&mut chain_b, sequence, &gmp_packet_bytes, &recv_proof_bytes)
        .await
        .expect("upload recv chunks on Chain B failed");

    let remaining_accounts = gmp::build_increment_remaining_accounts(
        gmp_account_pda,
        counter_app_state,
        user_counter_pda,
    );

    let recv = relayer
        .gmp_recv_packet(
            &mut chain_b,
            GmpRecvPacketParams {
                sequence,
                payload_chunk_pda: b_recv_payload,
                proof_chunk_pda: b_recv_proof,
                remaining_accounts,
            },
        )
        .await
        .expect("recv_packet on Chain B failed");

    assert_receipt_created(&chain_b, recv.receipt_pda).await;

    let user_counter = read_user_counter(&chain_b, user_counter_pda).await;
    assert_eq!(user_counter.count, increment_amount);

    let counter_state = read_counter_app_state(&chain_b, counter_app_state).await;
    assert_eq!(counter_state.total_counters, 1);

    // ── Build attestation proof for ack on Chain A ──
    // The raw ack is the GMP-encoded success response with the counter value.
    // The router hashes this to get the commitment stored in the ack PDA.
    let raw_gmp_ack = ics27_gmp::encoding::encode_gmp_ack(
        &increment_amount.to_le_bytes(),
        gmp::ICS27_ENCODING_PROTOBUF,
    )
    .expect("encode GMP ack");

    // The ack PDA stores the commitment hash — verify consistency.
    let ack_commitment = extract_ack_data(&chain_b, recv.ack_pda).await;
    let expected_commitment = ics26_router::ics24::packet_acknowledgement_commitment_bytes32(
        std::slice::from_ref(&raw_gmp_ack),
    )
    .expect("compute ack commitment");
    assert_eq!(ack_commitment, expected_commitment);

    let ack_entry = attestation::ack_commitment_entry(
        chain_a.counterparty_client_id(),
        sequence,
        ack_commitment
            .as_slice()
            .try_into()
            .expect("ack should be 32 bytes"),
    );
    let ack_proof =
        attestation::build_packet_membership_proof(&attestors, PROOF_HEIGHT, &[ack_entry]);
    let ack_proof_bytes = attestation::serialize_proof(&ack_proof);

    // ── Relayer uploads chunks and delivers ack_packet back to Chain A ──
    let (a_ack_payload, a_ack_proof) = relayer
        .upload_chunks(&mut chain_a, sequence, &gmp_packet_bytes, &ack_proof_bytes)
        .await
        .expect("upload ack chunks on Chain A failed");

    let ack_commitment_pda = relayer
        .gmp_ack_packet(
            &mut chain_a,
            GmpAckPacketParams {
                sequence,
                acknowledgement: raw_gmp_ack,
                payload_chunk_pda: a_ack_payload,
                proof_chunk_pda: a_ack_proof,
            },
        )
        .await
        .expect("ack_packet on Chain A failed");

    assert_commitment_zeroed(&chain_a, ack_commitment_pda).await;
    assert_gmp_result_exists(&chain_a, chain_a.client_id(), sequence).await;
}
