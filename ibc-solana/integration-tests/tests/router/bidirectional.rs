// SPDX-License-Identifier: Apache-2.0

use super::*;

/// Bidirectional: A->B and B->A with different sequences.
#[tokio::test]
async fn test_bidirectional_packets() {
    // ── Attestors ──
    let attestors_a = Attestors::new(2);
    let attestors_b = Attestors::new(3);

    // ── Actors ──
    let deployer = Deployer::new();
    let admin = Admin::new();
    let relayer = Relayer::new();
    let user_a = User::new();
    let user_b = User::new();

    // ── Test data ──
    let successful_ack = test_ibc_app::instructions::SUCCESSFUL_ACKNOWLEDGEMENT_JSON.to_vec();
    let data_a_to_b = b"A says hello to B";
    let data_b_to_a = b"B says hello to A";
    let seq_a_to_b = 1u64;
    let seq_b_to_a = 2u64;

    // ── Chains ──
    let attestation_lc_a = AttestationLc::new(&attestors_a);
    let attestation_lc_b = AttestationLc::new(&attestors_b);
    let programs_a: &[&dyn ChainProgram] = &[&TestIbcApp, &attestation_lc_a];
    let programs_b: &[&dyn ChainProgram] = &[&TestIbcApp, &attestation_lc_b];
    let (mut chain_a, mut chain_b) = Chain::pair(&deployer, programs_a, programs_b);
    chain_a.prefund(&[&admin, &relayer, &user_a]);
    chain_b.prefund(&[&admin, &relayer, &user_b]);

    // ── Init ──
    chain_a
        .init_with_attestation(&deployer, &admin, &relayer, programs_a, &attestors_a)
        .await;
    chain_b
        .init_with_attestation(&deployer, &admin, &relayer, programs_b, &attestors_b)
        .await;

    // ── User A sends A→B ──
    let send_ab = user_a
        .send_packet(
            &mut chain_a,
            SendPacketParams {
                sequence: seq_a_to_b,
                packet_data: data_a_to_b,
            },
        )
        .await
        .expect("A->B send failed");

    // ── User B sends B→A ──
    let send_ba = user_b
        .send_packet(
            &mut chain_b,
            SendPacketParams {
                sequence: seq_b_to_a,
                packet_data: data_b_to_a,
            },
        )
        .await
        .expect("B->A send failed");

    // ── Build attestation proof for A→B recv on Chain B ──
    let commitment_ab = read_commitment(&chain_a, send_ab.commitment_pda).await;
    let recv_ab_entry = attestation::packet_commitment_entry(
        chain_b.counterparty_client_id(),
        seq_a_to_b,
        commitment_ab,
    );
    let recv_ab_proof =
        attestation::build_packet_membership_proof(&attestors_b, PROOF_HEIGHT, &[recv_ab_entry]);
    let recv_ab_proof_bytes = attestation::serialize_proof(&recv_ab_proof);

    // ── Relayer uploads chunks and delivers A→B to Chain B ──
    let (b_recv_payload, b_recv_proof) = relayer
        .upload_chunks(&mut chain_b, seq_a_to_b, data_a_to_b, &recv_ab_proof_bytes)
        .await
        .expect("upload B recv chunks failed");
    let recv_ab = relayer
        .recv_packet(
            &mut chain_b,
            RecvPacketParams {
                sequence: seq_a_to_b,
                payload_chunk_pda: b_recv_payload,
                proof_chunk_pda: b_recv_proof,
                app_program: test_ibc_app::ID,
                ..Default::default()
            },
        )
        .await
        .expect("A->B recv on B failed");

    // ── Build attestation proof for B→A recv on Chain A ──
    let commitment_ba = read_commitment(&chain_b, send_ba.commitment_pda).await;
    let recv_ba_entry = attestation::packet_commitment_entry(
        chain_a.counterparty_client_id(),
        seq_b_to_a,
        commitment_ba,
    );
    let recv_ba_proof =
        attestation::build_packet_membership_proof(&attestors_a, PROOF_HEIGHT, &[recv_ba_entry]);
    let recv_ba_proof_bytes = attestation::serialize_proof(&recv_ba_proof);

    // ── Relayer uploads chunks and delivers B→A to Chain A ──
    let (a_recv_payload, a_recv_proof) = relayer
        .upload_chunks(&mut chain_a, seq_b_to_a, data_b_to_a, &recv_ba_proof_bytes)
        .await
        .expect("upload A recv chunks failed");
    let recv_ba = relayer
        .recv_packet(
            &mut chain_a,
            RecvPacketParams {
                sequence: seq_b_to_a,
                payload_chunk_pda: a_recv_payload,
                proof_chunk_pda: a_recv_proof,
                app_program: test_ibc_app::ID,
                ..Default::default()
            },
        )
        .await
        .expect("B->A recv on A failed");

    // ── Build attestation proof for A→B ack back to Chain A ──
    let ack_ab_commitment = extract_ack_data(&chain_b, recv_ab.ack_pda).await;
    let ack_ab_entry = attestation::ack_commitment_entry(
        chain_a.counterparty_client_id(),
        seq_a_to_b,
        ack_ab_commitment
            .as_slice()
            .try_into()
            .expect("ack should be 32 bytes"),
    );
    let ack_ab_proof =
        attestation::build_packet_membership_proof(&attestors_a, PROOF_HEIGHT, &[ack_ab_entry]);
    let ack_ab_proof_bytes = attestation::serialize_proof(&ack_ab_proof);

    // ── Relayer uploads chunks and delivers A→B ack back to Chain A ──
    let (a_ack_payload, a_ack_proof) = relayer
        .upload_chunks(&mut chain_a, seq_a_to_b, data_a_to_b, &ack_ab_proof_bytes)
        .await
        .expect("upload A ack chunks failed");
    relayer
        .ack_packet(
            &mut chain_a,
            AckPacketParams {
                sequence: seq_a_to_b,
                acknowledgement: successful_ack.clone(),
                payload_chunk_pda: a_ack_payload,
                proof_chunk_pda: a_ack_proof,
                app_program: test_ibc_app::ID,
                ..Default::default()
            },
        )
        .await
        .expect("A->B ack on A failed");

    // ── Build attestation proof for B→A ack back to Chain B ──
    let ack_ba_commitment = extract_ack_data(&chain_a, recv_ba.ack_pda).await;
    let ack_ba_entry = attestation::ack_commitment_entry(
        chain_b.counterparty_client_id(),
        seq_b_to_a,
        ack_ba_commitment
            .as_slice()
            .try_into()
            .expect("ack should be 32 bytes"),
    );
    let ack_ba_proof =
        attestation::build_packet_membership_proof(&attestors_b, PROOF_HEIGHT, &[ack_ba_entry]);
    let ack_ba_proof_bytes = attestation::serialize_proof(&ack_ba_proof);

    // ── Relayer uploads chunks and delivers B→A ack back to Chain B ──
    let (b_ack_payload, b_ack_proof) = relayer
        .upload_chunks(&mut chain_b, seq_b_to_a, data_b_to_a, &ack_ba_proof_bytes)
        .await
        .expect("upload B ack chunks failed");
    relayer
        .ack_packet(
            &mut chain_b,
            AckPacketParams {
                sequence: seq_b_to_a,
                acknowledgement: successful_ack,
                payload_chunk_pda: b_ack_payload,
                proof_chunk_pda: b_ack_proof,
                app_program: test_ibc_app::ID,
                ..Default::default()
            },
        )
        .await
        .expect("B->A ack on B failed");

    // ── Verify final counters ──
    let a_state = read_app_state(&chain_a).await;
    assert_eq!(a_state.packets_sent, 1);
    assert_eq!(a_state.packets_received, 1);
    assert_eq!(a_state.packets_acknowledged, 1);

    let b_state = read_app_state(&chain_b).await;
    assert_eq!(b_state.packets_sent, 1);
    assert_eq!(b_state.packets_received, 1);
    assert_eq!(b_state.packets_acknowledged, 1);
}
