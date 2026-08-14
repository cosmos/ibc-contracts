// SPDX-License-Identifier: Apache-2.0

use super::*;

/// Source chain times out a packet, but the destination chain independently
/// accepts `recv_packet` (chains don't share state). The subsequent `ack_packet`
/// back on the source fails because the commitment is already zeroed.
#[tokio::test]
async fn test_recv_after_source_timeout() {
    // ── Attestors ──
    let attestors_a = Attestors::new(2);
    let attestors_b = Attestors::new(3);

    // ── Actors ──
    let deployer = Deployer::new();
    let admin = Admin::new();
    let relayer = Relayer::new();
    let user = User::new();

    // ── Test data ──
    let packet_data = b"timeout then recv";
    let sequence = 1u64;
    let timeout_timestamp = router::test_timeout(TEST_CLOCK_TIME);

    // ── Chains ──
    let attestation_lc_a = AttestationLc::new(&attestors_a);
    let attestation_lc_b = AttestationLc::new(&attestors_b);
    let programs_a: &[&dyn ChainProgram] = &[&TestIbcApp, &attestation_lc_a];
    let programs_b: &[&dyn ChainProgram] = &[&TestIbcApp, &attestation_lc_b];
    let (mut chain_a, mut chain_b) = Chain::pair(&deployer, programs_a, programs_b);
    chain_a.prefund(&[&admin, &relayer, &user]);
    chain_b.prefund(&[&admin, &relayer]);

    // ── Init ──
    // Chain A needs timeout-compatible consensus timestamp.
    chain_a.init(&deployer, &admin, &relayer, programs_a).await;
    let timeout_consensus_proof =
        attestation::build_state_membership_proof(&attestors_a, PROOF_HEIGHT, timeout_timestamp);
    let update_ix = attestation::build_update_client_ix(
        relayer.pubkey(),
        PROOF_HEIGHT,
        timeout_consensus_proof,
    );
    relayer
        .send_tx(&mut chain_a, &[update_ix])
        .await
        .expect("update_client for timeout consensus failed");
    chain_b
        .init_with_attestation(&deployer, &admin, &relayer, programs_b, &attestors_b)
        .await;

    // ── Step 1: User sends on A ──
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

    // ── Step 2: Relayer times out on A ──
    let timeout_entry = attestation::receipt_commitment_entry(
        chain_a.counterparty_client_id(),
        sequence,
        [0u8; 32],
    );
    let timeout_proof =
        attestation::build_packet_membership_proof(&attestors_a, PROOF_HEIGHT, &[timeout_entry]);
    let timeout_proof_bytes = attestation::serialize_proof(&timeout_proof);

    let (a_to_payload, a_to_proof) = relayer
        .upload_chunks(&mut chain_a, sequence, packet_data, &timeout_proof_bytes)
        .await
        .expect("upload timeout chunks failed");

    relayer
        .timeout_packet(
            &mut chain_a,
            TimeoutPacketParams {
                sequence,
                payload_chunk_pda: a_to_payload,
                proof_chunk_pda: a_to_proof,
                app_program: test_ibc_app::ID,
                ..Default::default()
            },
        )
        .await
        .expect("timeout_packet on source failed");

    assert_commitment_zeroed(&chain_a, send.commitment_pda).await;

    // ── Step 3: Relayer delivers recv on B (succeeds — B is independent) ──
    // Commitment is zeroed on A after timeout, so compute from the packet.
    let recv_commitment = ics24::packet_commitment_bytes32(&send.packet);
    let recv_entry = attestation::packet_commitment_entry(
        chain_b.counterparty_client_id(),
        sequence,
        recv_commitment,
    );
    let recv_proof =
        attestation::build_packet_membership_proof(&attestors_b, PROOF_HEIGHT, &[recv_entry]);
    let recv_proof_bytes = attestation::serialize_proof(&recv_proof);

    let (b_recv_payload, b_recv_proof) = relayer
        .upload_chunks(&mut chain_b, sequence, packet_data, &recv_proof_bytes)
        .await
        .expect("upload recv chunks on B failed");

    let recv = relayer
        .recv_packet(
            &mut chain_b,
            RecvPacketParams {
                sequence,
                payload_chunk_pda: b_recv_payload,
                proof_chunk_pda: b_recv_proof,
                app_program: test_ibc_app::ID,
                ..Default::default()
            },
        )
        .await
        .expect("recv_packet on dest should succeed despite source timeout");

    assert_receipt_created(&chain_b, recv.receipt_pda).await;
    let ack_data = extract_ack_data(&chain_b, recv.ack_pda).await;

    // ── Step 4: Relayer attempts ack on A — fails (commitment already zeroed) ──
    // `test_ibc_app` returns a JSON success ack.
    let successful_ack = test_ibc_app::instructions::SUCCESSFUL_ACKNOWLEDGEMENT_JSON.to_vec();
    let ack_entry = attestation::ack_commitment_entry(
        chain_a.counterparty_client_id(),
        sequence,
        ack_data
            .as_slice()
            .try_into()
            .expect("ack should be 32 bytes"),
    );
    let ack_proof =
        attestation::build_packet_membership_proof(&attestors_a, PROOF_HEIGHT, &[ack_entry]);
    let ack_proof_bytes = attestation::serialize_proof(&ack_proof);

    // Cleanup timeout chunks first so the same PDAs can be re-created for ack
    relayer
        .cleanup_chunks(&mut chain_a, sequence, a_to_payload, a_to_proof)
        .await
        .expect("cleanup timeout chunks failed");

    let (a_ack_payload, a_ack_proof) = relayer
        .upload_chunks(&mut chain_a, sequence, packet_data, &ack_proof_bytes)
        .await
        .expect("upload ack chunks on A failed");

    let err = relayer
        .ack_packet(
            &mut chain_a,
            AckPacketParams {
                sequence,
                acknowledgement: successful_ack,
                payload_chunk_pda: a_ack_payload,
                proof_chunk_pda: a_ack_proof,
                app_program: test_ibc_app::ID,
                ..Default::default()
            },
        )
        .await
        .expect_err("ack_packet should fail — commitment already zeroed by timeout");

    assert_eq!(
        extract_custom_error(&err),
        PACKET_COMMITMENT_MISMATCH,
        "expected PACKET_COMMITMENT_MISMATCH"
    );
}
