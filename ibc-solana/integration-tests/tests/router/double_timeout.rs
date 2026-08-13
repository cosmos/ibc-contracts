// SPDX-License-Identifier: Apache-2.0

use super::*;

/// Timing out the same packet twice fails — the commitment is already zeroed.
#[tokio::test]
async fn test_double_timeout_fails() {
    // ── Attestors ──
    let attestors = Attestors::new(2);

    // ── Actors ──
    let deployer = Deployer::new();
    let admin = Admin::new();
    let relayer = Relayer::new();
    let user = User::new();

    // ── Test data ──
    let packet_data = b"double timeout";
    let sequence = 1u64;
    let timeout_timestamp = router::test_timeout(TEST_CLOCK_TIME);

    // ── Chain ──
    let attestation_lc = AttestationLc::new(&attestors);
    let programs: &[&dyn ChainProgram] = &[&TestIbcApp, &attestation_lc];
    let mut chain_a = Chain::single(&deployer, programs);
    chain_a.prefund(&[&admin, &relayer, &user]);

    // ── Init (manual: need timeout-compatible consensus timestamp) ──
    chain_a.init(&deployer, &admin, &relayer, programs).await;
    let timeout_consensus_proof =
        attestation::build_state_membership_proof(&attestors, PROOF_HEIGHT, timeout_timestamp);
    let update_ix = attestation::build_update_client_ix(
        relayer.pubkey(),
        PROOF_HEIGHT,
        timeout_consensus_proof,
    );
    relayer
        .send_tx(&mut chain_a, &[update_ix])
        .await
        .expect("update_client for timeout consensus failed");

    // ── Send ──
    user.send_packet(
        &mut chain_a,
        SendPacketParams {
            sequence,
            packet_data,
        },
    )
    .await
    .expect("send failed");

    // ── Build timeout proof ──
    let timeout_entry = attestation::receipt_commitment_entry(
        chain_a.counterparty_client_id(),
        sequence,
        [0u8; 32],
    );
    let timeout_proof =
        attestation::build_packet_membership_proof(&attestors, PROOF_HEIGHT, &[timeout_entry]);
    let timeout_proof_bytes = attestation::serialize_proof(&timeout_proof);

    let (payload_pda, proof_pda) = relayer
        .upload_chunks(&mut chain_a, sequence, packet_data, &timeout_proof_bytes)
        .await
        .expect("upload timeout chunks failed");

    let timeout_params = TimeoutPacketParams {
        sequence,
        payload_chunk_pda: payload_pda,
        proof_chunk_pda: proof_pda,
        app_program: test_ibc_app::ID,
        ..Default::default()
    };

    relayer
        .timeout_packet(&mut chain_a, timeout_params)
        .await
        .expect("first timeout failed");

    // Cleanup consumed chunks, then re-upload for second attempt
    relayer
        .cleanup_chunks(&mut chain_a, sequence, payload_pda, proof_pda)
        .await
        .expect("cleanup chunks failed");
    let (payload_pda, proof_pda) = relayer
        .upload_chunks(&mut chain_a, sequence, packet_data, &timeout_proof_bytes)
        .await
        .expect("re-upload timeout chunks failed");

    // Second timeout — commitment is zeroed, should fail
    let err = relayer
        .timeout_packet(
            &mut chain_a,
            TimeoutPacketParams {
                sequence,
                payload_chunk_pda: payload_pda,
                proof_chunk_pda: proof_pda,
                app_program: test_ibc_app::ID,
                ..Default::default()
            },
        )
        .await
        .expect_err("second timeout should fail");

    assert_eq!(extract_custom_error(&err), PACKET_COMMITMENT_MISMATCH);
}
