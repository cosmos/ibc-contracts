// SPDX-License-Identifier: Apache-2.0

use super::*;

/// Error ack lifecycle: `mock_ibc_app` returns `b"error"` when payload starts
/// with `RETURN_ERROR_ACK`. The router stores the error ack commitment and the
/// full send -> recv -> ack flow completes successfully.
#[tokio::test]
async fn test_error_ack_lifecycle() {
    // ── Attestors ──
    let attestors_a = Attestors::new(2);
    let attestors_b = Attestors::new(3);

    // ── Actors ──
    let deployer = Deployer::new();
    let admin = Admin::new();
    let relayer = Relayer::new();
    let user = User::new();

    // ── Test data ──
    let sequence = 1u64;
    // Payload prefix triggers error ack in mock_ibc_app (first 16 bytes checked)
    let packet_data = b"RETURN_ERROR_ACKextra";
    let error_ack = b"error".to_vec();

    // ── Chains ──
    let attestation_lc_a = AttestationLc::new(&attestors_a);
    let attestation_lc_b = AttestationLc::new(&attestors_b);
    let programs_a: &[&dyn ChainProgram] = &[&TestIbcApp, &attestation_lc_a];
    let programs_b: &[&dyn ChainProgram] = &[&MockIbcApp, &attestation_lc_b];
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

    // ── User sends on A ──
    let send = user
        .send_packet(
            &mut chain_a,
            SendPacketParams {
                sequence,
                packet_data,
            },
        )
        .await
        .expect("send_packet failed");

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

    // Relayer delivers to B — mock_ibc_app returns b"error"
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
                app_program: mock_ibc_app::ID,
                app_state_pda: mock_ibc_app_state_pda(),
                ..Default::default()
            },
        )
        .await
        .expect("recv_packet with error ack failed");

    // Verify ack was stored on B (non-zero commitment)
    let ack = chain_b
        .get_account(recv.ack_pda)
        .await
        .expect("ack should exist on B");
    assert_ne!(
        &ack.data[8..40],
        &[0u8; 32],
        "ack commitment should be non-zero"
    );

    // Verify the ack commitment matches hash of the error ack
    let expected_ack_commitment =
        ics24::packet_acknowledgement_commitment_bytes32(std::slice::from_ref(&error_ack))
            .expect("failed to compute ack commitment");
    assert_eq!(
        &ack.data[8..40],
        &expected_ack_commitment,
        "ack commitment should match hash of b\"error\""
    );

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

    // Relayer delivers ack back to A with the raw error ack bytes
    let (a_payload, a_proof) = relayer
        .upload_chunks(&mut chain_a, sequence, packet_data, &ack_proof_bytes)
        .await
        .expect("upload ack chunks failed");
    let commitment_pda = relayer
        .ack_packet(
            &mut chain_a,
            AckPacketParams {
                sequence,
                acknowledgement: error_ack,
                payload_chunk_pda: a_payload,
                proof_chunk_pda: a_proof,
                app_program: test_ibc_app::ID,
                ..Default::default()
            },
        )
        .await
        .expect("ack_packet with error ack failed");

    assert_commitment_zeroed(&chain_a, commitment_pda).await;
}
