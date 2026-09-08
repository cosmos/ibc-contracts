// SPDX-License-Identifier: Apache-2.0

use super::*;

/// Timeout lifecycle: send on A -> timeout on A (packet never delivered to B).
#[tokio::test]
async fn test_timeout_packet() {
    // ── Attestors ──
    let attestors = Attestors::new(2);

    // ── Actors ──
    let deployer = Deployer::new();
    let admin = Admin::new();
    let relayer = Relayer::new();
    let user = User::new();

    // ── Test data ──
    let packet_data = b"this packet will time out";
    let sequence = 1u64;
    let timeout_timestamp = router::test_timeout(TEST_CLOCK_TIME);

    // ── Chain ──
    let attestation_lc = AttestationLc::new(&attestors);
    let programs: &[&dyn ChainProgram] = &[&TestIbcApp, &attestation_lc];
    let mut chain_a = Chain::single(&deployer, programs);
    chain_a.prefund(&[&admin, &relayer, &user]);

    // ── Init ──
    // Manual init: we need update_client with a timestamp >= timeout so the
    // router's timeout check passes.
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

    // ── User sends packet on Chain A ──
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

    assert_commitment_set(&chain_a, send.commitment_pda).await;

    // ── Build attestation non-membership proof for timeout ──
    let timeout_entry = attestation::receipt_commitment_entry(
        chain_a.counterparty_client_id(),
        sequence,
        [0u8; 32],
    );
    let timeout_proof =
        attestation::build_packet_membership_proof(&attestors, PROOF_HEIGHT, &[timeout_entry]);
    let timeout_proof_bytes = attestation::serialize_proof(&timeout_proof);

    // ── Relayer uploads chunks and delivers timeout on Chain A ──
    let (timeout_payload, timeout_proof_pda) = relayer
        .upload_chunks(&mut chain_a, sequence, packet_data, &timeout_proof_bytes)
        .await
        .expect("upload timeout chunks failed");
    let commitment_pda = relayer
        .timeout_packet(
            &mut chain_a,
            TimeoutPacketParams {
                sequence,
                payload_chunk_pda: timeout_payload,
                proof_chunk_pda: timeout_proof_pda,
                app_program: test_ibc_app::ID,
                ..Default::default()
            },
        )
        .await
        .expect("timeout_packet failed");

    assert_commitment_zeroed(&chain_a, commitment_pda).await;

    // Verify app state reflects the timeout
    let a_state = read_app_state(&chain_a).await;
    assert_eq!(a_state.packets_sent, 1);
    assert_eq!(a_state.packets_timed_out, 1);
    assert_eq!(a_state.packets_acknowledged, 0);
}
