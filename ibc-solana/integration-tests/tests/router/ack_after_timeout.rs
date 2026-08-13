// SPDX-License-Identifier: Apache-2.0

use super::*;

/// After a successful timeout, attempting to ack the same packet fails.
#[tokio::test]
async fn test_ack_after_timeout_fails() {
    // ── Attestors ──
    let attestors = Attestors::new(2);

    // ── Actors ──
    let deployer = Deployer::new();
    let admin = Admin::new();
    let relayer = Relayer::new();
    let user = User::new();

    // ── Test data ──
    let packet_data = b"timeout then ack";
    let sequence = 1u64;
    let successful_ack = test_ibc_app::instructions::SUCCESSFUL_ACKNOWLEDGEMENT_JSON.to_vec();
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

    // Timeout the packet
    let (payload_pda, proof_pda) = relayer
        .upload_chunks(&mut chain_a, sequence, packet_data, &timeout_proof_bytes)
        .await
        .expect("upload timeout chunks failed");
    relayer
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
        .expect("timeout failed");

    // ── Build ack proof (for the failed ack attempt) ──
    let ack_commitment =
        ics24::packet_acknowledgement_commitment_bytes32(std::slice::from_ref(&successful_ack))
            .expect("compute ack commitment");
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

    // Cleanup consumed timeout chunks, then upload fresh ones for the ack attempt
    relayer
        .cleanup_chunks(&mut chain_a, sequence, payload_pda, proof_pda)
        .await
        .expect("cleanup chunks failed");
    let (a_payload, a_proof) = relayer
        .upload_chunks(&mut chain_a, sequence, packet_data, &ack_proof_bytes)
        .await
        .expect("upload ack chunks failed");

    // Now try to ack — commitment is zeroed, should fail
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
        .expect_err("ack after timeout should fail");

    assert_eq!(extract_custom_error(&err), PACKET_COMMITMENT_MISMATCH);
}
