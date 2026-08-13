// SPDX-License-Identifier: Apache-2.0

use super::*;

/// Acking the same packet twice fails — the commitment is already zeroed.
#[tokio::test]
async fn test_double_ack_fails() {
    // ── Attestors ──
    let attestors_a = Attestors::new(2);
    let attestors_b = Attestors::new(3);

    // ── Actors ──
    let deployer = Deployer::new();
    let admin = Admin::new();
    let relayer = Relayer::new();
    let user = User::new();

    // ── Test data ──
    let packet_data = b"double ack";
    let sequence = 1u64;
    let successful_ack = test_ibc_app::instructions::SUCCESSFUL_ACKNOWLEDGEMENT_JSON.to_vec();

    // ── Chains ──
    let attestation_lc_a = AttestationLc::new(&attestors_a);
    let attestation_lc_b = AttestationLc::new(&attestors_b);
    let programs_a: &[&dyn ChainProgram] = &[&TestIbcApp, &attestation_lc_a];
    let programs_b: &[&dyn ChainProgram] = &[&TestIbcApp, &attestation_lc_b];
    let (mut chain_a, mut chain_b) = Chain::pair(&deployer, programs_a, programs_b);
    chain_a.prefund(&[&admin, &relayer, &user]);
    chain_b.prefund(&[&admin, &relayer]);

    // ── Init ──
    chain_a
        .init_with_attestation(&deployer, &admin, &relayer, programs_a, &attestors_a)
        .await;
    chain_b
        .init_with_attestation(&deployer, &admin, &relayer, programs_b, &attestors_b)
        .await;

    // ── Send → recv → ack (full lifecycle) ──
    let send = user
        .send_packet(
            &mut chain_a,
            SendPacketParams {
                sequence,
                packet_data,
            },
        )
        .await
        .expect("send failed");

    // ── Build attestation proof for recv on Chain B ──
    let commitment = read_commitment(&chain_a, send.commitment_pda).await;
    let recv_entry = attestation::packet_commitment_entry(
        chain_b.counterparty_client_id(),
        sequence,
        commitment,
    );
    let recv_proof =
        attestation::build_packet_membership_proof(&attestors_b, PROOF_HEIGHT, &[recv_entry]);
    let recv_proof_bytes = attestation::serialize_proof(&recv_proof);

    let (b_payload, b_proof) = relayer
        .upload_chunks(&mut chain_b, sequence, packet_data, &recv_proof_bytes)
        .await
        .expect("upload recv chunks failed");
    let recv = relayer
        .recv_packet(
            &mut chain_b,
            RecvPacketParams {
                sequence,
                payload_chunk_pda: b_payload,
                proof_chunk_pda: b_proof,
                app_program: test_ibc_app::ID,
                ..Default::default()
            },
        )
        .await
        .expect("recv failed");

    // ── Build attestation proof for ack on Chain A ──
    let ack_commitment = extract_ack_data(&chain_b, recv.ack_pda).await;
    let ack_entry = attestation::ack_commitment_entry(
        chain_a.counterparty_client_id(),
        sequence,
        ack_commitment
            .as_slice()
            .try_into()
            .expect("ack should be 32 bytes"),
    );
    let ack_proof =
        attestation::build_packet_membership_proof(&attestors_a, PROOF_HEIGHT, &[ack_entry]);
    let ack_proof_bytes = attestation::serialize_proof(&ack_proof);

    let (a_payload, a_proof) = relayer
        .upload_chunks(&mut chain_a, sequence, packet_data, &ack_proof_bytes)
        .await
        .expect("upload ack chunks failed");

    let ack_params = AckPacketParams {
        sequence,
        acknowledgement: successful_ack.clone(),
        payload_chunk_pda: a_payload,
        proof_chunk_pda: a_proof,
        app_program: test_ibc_app::ID,
        ..Default::default()
    };

    relayer
        .ack_packet(&mut chain_a, ack_params)
        .await
        .expect("first ack failed");

    // Cleanup consumed chunks, then re-upload for second attempt
    relayer
        .cleanup_chunks(&mut chain_a, sequence, a_payload, a_proof)
        .await
        .expect("cleanup chunks failed");
    let (a_payload, a_proof) = relayer
        .upload_chunks(&mut chain_a, sequence, packet_data, &ack_proof_bytes)
        .await
        .expect("re-upload ack chunks failed");

    // Second ack — commitment is zeroed, should fail
    let err = relayer
        .ack_packet(
            &mut chain_a,
            AckPacketParams {
                sequence,
                acknowledgement: successful_ack,
                payload_chunk_pda: a_payload,
                proof_chunk_pda: a_proof,
                app_program: test_ibc_app::ID,
                ..Default::default()
            },
        )
        .await
        .expect_err("second ack should fail");

    assert_eq!(extract_custom_error(&err), PACKET_COMMITMENT_MISMATCH);
}
