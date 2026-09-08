// SPDX-License-Identifier: Apache-2.0

use super::*;

/// Two GMP calls from the same sender: the `UserCounter` state accumulates
/// across both deliveries and `CounterAppState.total_counters` stays at 1.
#[tokio::test]
async fn test_multiple_gmp_calls() {
    // ── Attestors (independent per chain) ──
    let attestors_a = Attestors::new(2);
    let attestors_b = Attestors::new(3);

    // ── Actors ──
    let deployer = Deployer::new();
    let admin = Admin::new();
    let relayer = Relayer::new();
    let user = User::new();

    // ── Test data ──
    let first_amount = 42u64;
    let second_amount = 58u64;

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

    // ── Init ──
    chain_a
        .init_with_attestation(&deployer, &admin, &relayer, programs_a, &attestors_a)
        .await;
    chain_b
        .init_with_attestation(&deployer, &admin, &relayer, programs_b, &attestors_b)
        .await;

    // ── First call: increment by 42 ──
    let user_counter_pda = gmp::derive_user_counter_pda(&gmp_account_pda);
    let counter_app_state = chain_b.counter_app_state_pda();
    let first_payload = gmp::encode_increment_payload(
        counter_app_state,
        user_counter_pda,
        gmp_account_pda,
        first_amount,
    );
    let first_packet = gmp::encode_gmp_packet(&user.pubkey(), &test_gmp_app::ID, &first_payload);

    let first_commitment_pda = user
        .send_call(
            &mut chain_a,
            GmpSendCallParams {
                sequence: 1,
                timeout_timestamp: GMP_TIMEOUT,
                receiver: &test_gmp_app::ID.to_string(),
                payload: first_payload.encode_to_vec(),
            },
        )
        .await
        .expect("first send_call failed");

    // ── Build attestation proof for first recv on Chain B ──
    let first_packet_commitment = read_commitment(&chain_a, first_commitment_pda).await;
    let first_recv_entry = attestation::packet_commitment_entry(
        chain_b.counterparty_client_id(),
        1,
        first_packet_commitment,
    );
    let first_recv_proof =
        attestation::build_packet_membership_proof(&attestors_b, PROOF_HEIGHT, &[first_recv_entry]);
    let first_recv_proof_bytes = attestation::serialize_proof(&first_recv_proof);

    let (b_payload_1, b_proof_1) = relayer
        .upload_chunks(&mut chain_b, 1, &first_packet, &first_recv_proof_bytes)
        .await
        .expect("upload first recv chunks failed");

    let remaining = gmp::build_increment_remaining_accounts(
        gmp_account_pda,
        counter_app_state,
        user_counter_pda,
    );

    let recv_1 = relayer
        .gmp_recv_packet(
            &mut chain_b,
            GmpRecvPacketParams {
                sequence: 1,
                payload_chunk_pda: b_payload_1,
                proof_chunk_pda: b_proof_1,
                remaining_accounts: remaining.clone(),
            },
        )
        .await
        .expect("first recv_packet failed");

    // ── Build attestation proof for first ack on Chain A ──
    let ack_1_commitment = extract_ack_data(&chain_b, recv_1.ack_pda).await;
    let ack_1_entry = attestation::ack_commitment_entry(
        chain_a.counterparty_client_id(),
        1,
        ack_1_commitment
            .as_slice()
            .try_into()
            .expect("ack should be 32 bytes"),
    );
    let ack_1_proof =
        attestation::build_packet_membership_proof(&attestors_a, PROOF_HEIGHT, &[ack_1_entry]);
    let ack_1_proof_bytes = attestation::serialize_proof(&ack_1_proof);

    let (a_payload_1, a_proof_1) = relayer
        .upload_chunks(&mut chain_a, 1, &first_packet, &ack_1_proof_bytes)
        .await
        .expect("upload first ack chunks failed");

    let raw_ack_1 = ics27_gmp::encoding::encode_gmp_ack(
        &first_amount.to_le_bytes(),
        gmp::ICS27_ENCODING_PROTOBUF,
    )
    .expect("encode first GMP ack");

    relayer
        .gmp_ack_packet(
            &mut chain_a,
            GmpAckPacketParams {
                sequence: 1,
                acknowledgement: raw_ack_1,
                payload_chunk_pda: a_payload_1,
                proof_chunk_pda: a_proof_1,
            },
        )
        .await
        .expect("first ack_packet failed");

    // ── Second call: increment by 58 ──
    let second_payload = gmp::encode_increment_payload(
        counter_app_state,
        user_counter_pda,
        gmp_account_pda,
        second_amount,
    );
    let second_packet = gmp::encode_gmp_packet(&user.pubkey(), &test_gmp_app::ID, &second_payload);

    let second_commitment_pda = user
        .send_call(
            &mut chain_a,
            GmpSendCallParams {
                sequence: 2,
                timeout_timestamp: GMP_TIMEOUT,
                receiver: &test_gmp_app::ID.to_string(),
                payload: second_payload.encode_to_vec(),
            },
        )
        .await
        .expect("second send_call failed");

    // ── Build attestation proof for second recv on Chain B ──
    let second_packet_commitment = read_commitment(&chain_a, second_commitment_pda).await;
    let second_recv_entry = attestation::packet_commitment_entry(
        chain_b.counterparty_client_id(),
        2,
        second_packet_commitment,
    );
    let second_recv_proof = attestation::build_packet_membership_proof(
        &attestors_b,
        PROOF_HEIGHT,
        &[second_recv_entry],
    );
    let second_recv_proof_bytes = attestation::serialize_proof(&second_recv_proof);

    let (b_payload_2, b_proof_2) = relayer
        .upload_chunks(&mut chain_b, 2, &second_packet, &second_recv_proof_bytes)
        .await
        .expect("upload second recv chunks failed");

    let recv_2 = relayer
        .gmp_recv_packet(
            &mut chain_b,
            GmpRecvPacketParams {
                sequence: 2,
                payload_chunk_pda: b_payload_2,
                proof_chunk_pda: b_proof_2,
                remaining_accounts: remaining,
            },
        )
        .await
        .expect("second recv_packet failed");

    // ── Build attestation proof for second ack on Chain A ──
    let ack_2_commitment = extract_ack_data(&chain_b, recv_2.ack_pda).await;
    let ack_2_entry = attestation::ack_commitment_entry(
        chain_a.counterparty_client_id(),
        2,
        ack_2_commitment
            .as_slice()
            .try_into()
            .expect("ack should be 32 bytes"),
    );
    let ack_2_proof =
        attestation::build_packet_membership_proof(&attestors_a, PROOF_HEIGHT, &[ack_2_entry]);
    let ack_2_proof_bytes = attestation::serialize_proof(&ack_2_proof);

    let (a_payload_2, a_proof_2) = relayer
        .upload_chunks(&mut chain_a, 2, &second_packet, &ack_2_proof_bytes)
        .await
        .expect("upload second ack chunks failed");

    let raw_ack_2 = ics27_gmp::encoding::encode_gmp_ack(
        &first_amount.saturating_add(second_amount).to_le_bytes(),
        gmp::ICS27_ENCODING_PROTOBUF,
    )
    .expect("encode second GMP ack");

    relayer
        .gmp_ack_packet(
            &mut chain_a,
            GmpAckPacketParams {
                sequence: 2,
                acknowledgement: raw_ack_2,
                payload_chunk_pda: a_payload_2,
                proof_chunk_pda: a_proof_2,
            },
        )
        .await
        .expect("second ack_packet failed");

    // ── Verify accumulated state ──
    let user_counter = read_user_counter(&chain_b, user_counter_pda).await;
    assert_eq!(
        user_counter.count,
        first_amount.saturating_add(second_amount),
        "UserCounter should accumulate both increments"
    );

    let counter_state = read_counter_app_state(&chain_b, counter_app_state).await;
    assert_eq!(
        counter_state.total_counters, 1,
        "same user should still have only one counter"
    );

    for seq in [1u64, 2] {
        assert_gmp_result_exists(&chain_a, chain_a.client_id(), seq).await;
    }
}
