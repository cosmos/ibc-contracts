// SPDX-License-Identifier: Apache-2.0

use super::*;

/// After a successful ack, attempting to timeout the same packet fails.
#[tokio::test]
async fn test_timeout_after_ack_fails() {
    // ── Attestors ──
    let attestors_a = Attestors::new(2);
    let attestors_b = Attestors::new(3);

    // ── Actors ──
    let deployer = Deployer::new();
    let admin = Admin::new();
    let relayer = Relayer::new();
    let user = User::new();

    // ── Test data ──
    let packet_data = b"ack then timeout";
    let sequence = 1u64;
    let successful_ack = test_ibc_app::instructions::SUCCESSFUL_ACKNOWLEDGEMENT_JSON.to_vec();
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
    // Chain A needs timeout-compatible consensus timestamp for the final
    // timeout attempt. Use manual init + update_client with timeout ts.
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

    // ── Full lifecycle: send -> recv -> ack ──
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

    // ── Recv proof ──
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

    // ── Ack proof ──
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
    relayer
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
        .expect("ack failed");

    // ── Build timeout proof for the failed attempt ──
    let timeout_entry = attestation::receipt_commitment_entry(
        chain_a.counterparty_client_id(),
        sequence,
        [0u8; 32],
    );
    let timeout_proof =
        attestation::build_packet_membership_proof(&attestors_a, PROOF_HEIGHT, &[timeout_entry]);
    let timeout_proof_bytes = attestation::serialize_proof(&timeout_proof);

    // Cleanup consumed ack chunks, then upload fresh ones for the timeout attempt
    relayer
        .cleanup_chunks(&mut chain_a, sequence, a_payload, a_proof)
        .await
        .expect("cleanup chunks failed");
    let (t_payload, t_proof) = relayer
        .upload_chunks(&mut chain_a, sequence, packet_data, &timeout_proof_bytes)
        .await
        .expect("upload timeout chunks failed");

    // Now try to timeout — commitment is zeroed, should fail
    let err = relayer
        .timeout_packet(
            &mut chain_a,
            TimeoutPacketParams {
                sequence,
                payload_chunk_pda: t_payload,
                proof_chunk_pda: t_proof,
                app_program: test_ibc_app::ID,
                ..Default::default()
            },
        )
        .await
        .expect_err("timeout after ack should fail");

    assert_eq!(extract_custom_error(&err), PACKET_COMMITMENT_MISMATCH);
}
