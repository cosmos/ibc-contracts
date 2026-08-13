// SPDX-License-Identifier: Apache-2.0

use super::*;

/// `recv_packet` by a relayer without `RELAYER_ROLE` is rejected by the
/// access manager CPI during the router's `require_role` check.
#[tokio::test]
async fn test_unauthorized_relayer_rejected() {
    // ── Attestors ──
    let attestors_a = Attestors::new(2);
    let attestors_b = Attestors::new(3);

    // ── Actors ──
    let deployer = Deployer::new();
    let admin = Admin::new();
    let relayer = Relayer::new();
    let user = User::new();
    let unauthorized = Relayer::new();

    // ── Test data ──
    let packet_data = b"unauthorized delivery";
    let sequence = 1u64;

    // ── Chains ──
    let attestation_lc_a = AttestationLc::new(&attestors_a);
    let attestation_lc_b = AttestationLc::new(&attestors_b);
    let programs_a: &[&dyn ChainProgram] = &[&TestIbcApp, &attestation_lc_a];
    let programs_b: &[&dyn ChainProgram] = &[&TestIbcApp, &attestation_lc_b];
    let (mut chain_a, mut chain_b) = Chain::pair(&deployer, programs_a, programs_b);
    chain_a.prefund(&[&admin, &relayer, &user]);
    chain_b.prefund(&[&admin, &relayer, &unauthorized]);

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

    // Authorized relayer uploads chunks on B (upload requires RELAYER_ROLE)
    let (payload_pda, proof_pda) = relayer
        .upload_chunks(&mut chain_b, sequence, packet_data, &recv_proof_bytes)
        .await
        .expect("authorized relayer upload_chunks should succeed");

    // Unauthorized relayer attempts recv_packet — access manager rejects
    let err = unauthorized
        .recv_packet(
            &mut chain_b,
            RecvPacketParams {
                sequence,
                payload_chunk_pda: payload_pda,
                proof_chunk_pda: proof_pda,
                app_program: test_ibc_app::ID,
                ..Default::default()
            },
        )
        .await
        .expect_err("recv_packet should fail for unauthorized relayer");

    let code = extract_custom_error(&err);
    assert_ne!(
        code, 0,
        "should have a non-zero error code from access manager rejection"
    );

    // Verify no receipt was created (recv_packet reverted)
    let receipt_pda = router::derive_receipt_pda(chain_b.client_id(), sequence);
    assert!(
        chain_b.get_account(receipt_pda).await.is_none(),
        "no receipt should exist after rejected recv_packet"
    );
}
