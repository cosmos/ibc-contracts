// SPDX-License-Identifier: Apache-2.0

use super::*;

/// Empty ack rejection: `mock_ibc_app` returns `vec![]` when payload starts
/// with `RETURN_EMPTY_ACK`. The router rejects empty acks with
/// `AsyncAcknowledgementNotSupported`.
#[tokio::test]
async fn test_empty_ack_rejected() {
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
    // Payload prefix triggers empty ack in mock_ibc_app
    let packet_data = b"RETURN_EMPTY_ACKextra";

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

    // Relayer delivers to B — mock_ibc_app returns empty ack, router rejects
    let (b_payload, b_proof) = relayer
        .upload_chunks(&mut chain_b, sequence, packet_data, &recv_proof_bytes)
        .await
        .expect("upload recv chunks failed");
    let err = relayer
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
        .expect_err("recv_packet with empty ack should fail");

    assert_eq!(
        extract_custom_error(&err),
        ASYNC_ACK_NOT_SUPPORTED,
        "should fail with AsyncAcknowledgementNotSupported"
    );
}
