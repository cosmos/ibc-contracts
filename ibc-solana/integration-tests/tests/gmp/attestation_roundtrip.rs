// SPDX-License-Identifier: Apache-2.0

use super::*;
use integration_tests::{
    attestation, attestor::Attestors, programs::AttestationLc, read_commitment,
    router::PROOF_HEIGHT,
};

/// Bidirectional GMP roundtrip with attestation light client.
///
/// Each chain has its own independent attestor set (2-of-2 quorum),
/// verifying that proofs are correctly tied to the verifying chain's
/// attestors rather than being shared. All recv and ack proofs are
/// real attestation proofs verified on-chain via ECDSA recovery.
#[tokio::test]
async fn test_gmp_attestation_roundtrip() {
    // ── Attestors (independent sets per chain) ──
    let attestors_a = Attestors::new(2);
    let attestors_b = Attestors::new(3);

    // ── Actors ──
    let deployer = Deployer::new();
    let admin = Admin::new();
    let relayer = Relayer::new();
    let user = User::new();

    // ── Test data ──
    let sequence = 1u64;
    let amount_a_to_b = 10u64;
    let amount_b_to_a = 20u64;

    // ── Chains (each with its own attestation LC) ──
    let attestation_lc_a = AttestationLc::new(&attestors_a);
    let programs_a: &[&dyn ChainProgram] = &[&Ics27Gmp, &TestGmpApp, &attestation_lc_a];

    let attestation_lc_b = AttestationLc::new(&attestors_b);
    let programs_b: &[&dyn ChainProgram] = &[&Ics27Gmp, &TestGmpApp, &attestation_lc_b];

    let (mut chain_a, mut chain_b) = Chain::pair(&deployer, programs_a, programs_b);
    chain_a.prefund(&[&admin, &relayer, &user]);
    chain_b.prefund(&[&admin, &relayer, &user]);

    let gmp_pda_on_a = gmp::derive_gmp_account_pda(chain_a.client_id(), &user.pubkey());
    chain_a.prefund_lamports(gmp_pda_on_a, GMP_ACCOUNT_PREFUND_LAMPORTS);
    let gmp_pda_on_b = gmp::derive_gmp_account_pda(chain_b.client_id(), &user.pubkey());
    chain_b.prefund_lamports(gmp_pda_on_b, GMP_ACCOUNT_PREFUND_LAMPORTS);

    // ── Init (includes update_client at PROOF_HEIGHT) ──
    chain_a
        .init_with_attestation(&deployer, &admin, &relayer, programs_a, &attestors_a)
        .await;
    chain_b
        .init_with_attestation(&deployer, &admin, &relayer, programs_b, &attestors_b)
        .await;

    // ── Build payloads ──
    let counter_on_a = gmp::derive_user_counter_pda(&gmp_pda_on_a);
    let counter_state_a = chain_a.counter_app_state_pda();
    let counter_on_b = gmp::derive_user_counter_pda(&gmp_pda_on_b);
    let counter_state_b = chain_b.counter_app_state_pda();

    let payload_a_to_b =
        gmp::encode_increment_payload(counter_state_b, counter_on_b, gmp_pda_on_b, amount_a_to_b);
    let packet_a_to_b = gmp::encode_gmp_packet(&user.pubkey(), &test_gmp_app::ID, &payload_a_to_b);

    let payload_b_to_a =
        gmp::encode_increment_payload(counter_state_a, counter_on_a, gmp_pda_on_a, amount_b_to_a);
    let packet_b_to_a = gmp::encode_gmp_packet(&user.pubkey(), &test_gmp_app::ID, &payload_b_to_a);

    // ── Send on both chains ──
    let commitment_a = user
        .send_call(
            &mut chain_a,
            GmpSendCallParams {
                sequence,
                timeout_timestamp: GMP_TIMEOUT,
                receiver: &test_gmp_app::ID.to_string(),
                payload: payload_a_to_b.encode_to_vec(),
            },
        )
        .await
        .expect("send_call on A failed");

    let commitment_b = user
        .send_call(
            &mut chain_b,
            GmpSendCallParams {
                sequence,
                timeout_timestamp: GMP_TIMEOUT,
                receiver: &test_gmp_app::ID.to_string(),
                payload: payload_b_to_a.encode_to_vec(),
            },
        )
        .await
        .expect("send_call on B failed");

    // ── Build attestation proof for A→B recv on Chain B ──
    // Chain B verifies → sign with attestors_b
    let packet_commitment_a = read_commitment(&chain_a, commitment_a).await;
    let recv_entry_a = attestation::packet_commitment_entry(
        chain_b.counterparty_client_id(),
        sequence,
        packet_commitment_a,
    );
    let recv_proof_a =
        attestation::build_packet_membership_proof(&attestors_b, PROOF_HEIGHT, &[recv_entry_a]);
    let recv_proof_a_bytes = attestation::serialize_proof(&recv_proof_a);

    // ── Deliver A→B recv on Chain B ──
    let (b_payload, b_proof) = relayer
        .upload_chunks(&mut chain_b, sequence, &packet_a_to_b, &recv_proof_a_bytes)
        .await
        .expect("upload A→B recv chunks failed");

    let remaining_b =
        gmp::build_increment_remaining_accounts(gmp_pda_on_b, counter_state_b, counter_on_b);
    let recv_on_b = relayer
        .gmp_recv_packet(
            &mut chain_b,
            GmpRecvPacketParams {
                sequence,
                payload_chunk_pda: b_payload,
                proof_chunk_pda: b_proof,
                remaining_accounts: remaining_b,
            },
        )
        .await
        .expect("A→B recv_packet failed");

    assert_receipt_created(&chain_b, recv_on_b.receipt_pda).await;

    // ── Build attestation proof for B→A recv on Chain A ──
    // Chain A verifies → sign with attestors_a
    let packet_commitment_b = read_commitment(&chain_b, commitment_b).await;
    let recv_entry_b = attestation::packet_commitment_entry(
        chain_a.counterparty_client_id(),
        sequence,
        packet_commitment_b,
    );
    let recv_proof_b =
        attestation::build_packet_membership_proof(&attestors_a, PROOF_HEIGHT, &[recv_entry_b]);
    let recv_proof_b_bytes = attestation::serialize_proof(&recv_proof_b);

    // ── Deliver B→A recv on Chain A ──
    let (a_payload, a_proof) = relayer
        .upload_chunks(&mut chain_a, sequence, &packet_b_to_a, &recv_proof_b_bytes)
        .await
        .expect("upload B→A recv chunks failed");

    let remaining_a =
        gmp::build_increment_remaining_accounts(gmp_pda_on_a, counter_state_a, counter_on_a);
    let recv_on_a = relayer
        .gmp_recv_packet(
            &mut chain_a,
            GmpRecvPacketParams {
                sequence,
                payload_chunk_pda: a_payload,
                proof_chunk_pda: a_proof,
                remaining_accounts: remaining_a,
            },
        )
        .await
        .expect("B→A recv_packet failed");

    assert_receipt_created(&chain_a, recv_on_a.receipt_pda).await;

    // ── Build attestation proof for A→B ack on Chain A ──
    // Chain A verifies → sign with attestors_a
    let raw_ack_b = ics27_gmp::encoding::encode_gmp_ack(
        &amount_a_to_b.to_le_bytes(),
        gmp::ICS27_ENCODING_PROTOBUF,
    )
    .expect("encode GMP ack B");

    let ack_commitment_b = extract_ack_data(&chain_b, recv_on_b.ack_pda).await;
    let ack_entry_b = attestation::ack_commitment_entry(
        chain_a.counterparty_client_id(),
        sequence,
        ack_commitment_b
            .as_slice()
            .try_into()
            .expect("ack should be 32 bytes"),
    );
    let ack_proof_b =
        attestation::build_packet_membership_proof(&attestors_a, PROOF_HEIGHT, &[ack_entry_b]);
    let ack_proof_b_bytes = attestation::serialize_proof(&ack_proof_b);

    // ── Deliver A→B ack back on Chain A ──
    relayer
        .cleanup_chunks(&mut chain_a, sequence, a_payload, a_proof)
        .await
        .expect("cleanup B→A recv chunks on A failed");
    let (a_ack_payload, a_ack_proof) = relayer
        .upload_chunks(&mut chain_a, sequence, &packet_a_to_b, &ack_proof_b_bytes)
        .await
        .expect("upload A→B ack chunks failed");
    relayer
        .gmp_ack_packet(
            &mut chain_a,
            GmpAckPacketParams {
                sequence,
                acknowledgement: raw_ack_b,
                payload_chunk_pda: a_ack_payload,
                proof_chunk_pda: a_ack_proof,
            },
        )
        .await
        .expect("A→B ack_packet failed");

    // ── Build attestation proof for B→A ack on Chain B ──
    // Chain B verifies → sign with attestors_b
    let raw_ack_a = ics27_gmp::encoding::encode_gmp_ack(
        &amount_b_to_a.to_le_bytes(),
        gmp::ICS27_ENCODING_PROTOBUF,
    )
    .expect("encode GMP ack A");

    let ack_commitment_a = extract_ack_data(&chain_a, recv_on_a.ack_pda).await;
    let ack_entry_a = attestation::ack_commitment_entry(
        chain_b.counterparty_client_id(),
        sequence,
        ack_commitment_a
            .as_slice()
            .try_into()
            .expect("ack should be 32 bytes"),
    );
    let ack_proof_a =
        attestation::build_packet_membership_proof(&attestors_b, PROOF_HEIGHT, &[ack_entry_a]);
    let ack_proof_a_bytes = attestation::serialize_proof(&ack_proof_a);

    // ── Deliver B→A ack back on Chain B ──
    relayer
        .cleanup_chunks(&mut chain_b, sequence, b_payload, b_proof)
        .await
        .expect("cleanup A→B recv chunks on B failed");
    let (b_ack_payload, b_ack_proof) = relayer
        .upload_chunks(&mut chain_b, sequence, &packet_b_to_a, &ack_proof_a_bytes)
        .await
        .expect("upload B→A ack chunks failed");
    relayer
        .gmp_ack_packet(
            &mut chain_b,
            GmpAckPacketParams {
                sequence,
                acknowledgement: raw_ack_a,
                payload_chunk_pda: b_ack_payload,
                proof_chunk_pda: b_ack_proof,
            },
        )
        .await
        .expect("B→A ack_packet failed");

    // ── Verify counters on each chain ──
    let counter_b = read_user_counter(&chain_b, counter_on_b).await;
    assert_eq!(counter_b.count, amount_a_to_b);

    let counter_a = read_user_counter(&chain_a, counter_on_a).await;
    assert_eq!(counter_a.count, amount_b_to_a);

    assert_gmp_result_exists(&chain_a, chain_a.client_id(), sequence).await;
    assert_gmp_result_exists(&chain_b, chain_b.client_id(), sequence).await;
}
