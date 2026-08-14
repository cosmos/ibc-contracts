// SPDX-License-Identifier: Apache-2.0

use super::*;

/// Light client rejects proof: the attestation LC returns an error when the
/// proof bytes are invalid, causing the entire `recv_packet` transaction to
/// revert.
#[tokio::test]
async fn test_proof_verification_failure() {
    // ── Attestors ──
    let attestors_a = Attestors::new(2);
    let attestors_b = Attestors::new(3);

    // ── Actors ──
    let deployer = Deployer::new();
    let admin = Admin::new();
    let relayer = Relayer::new();
    let user = User::new();

    // ── Test data ──
    let packet_data = b"proof will be rejected";
    // Garbage bytes that fail deserialization in the attestation LC
    let bad_proof = vec![0u8; 32];
    let sequence = 1u64;

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

    // ── User sends on A ──
    user.send_packet(
        &mut chain_a,
        SendPacketParams {
            sequence,
            packet_data,
        },
    )
    .await
    .expect("send_packet failed");

    // Relayer uploads chunks with the bad proof
    let (b_payload, b_proof) = relayer
        .upload_chunks(&mut chain_b, sequence, packet_data, &bad_proof)
        .await
        .expect("upload chunks failed");

    // recv_packet should fail — light client CPI error aborts the transaction
    let err = relayer
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
        .expect_err("recv_packet with rejected proof should fail");

    // Verify it's a custom error (CPI failure propagates as custom error)
    let code = extract_custom_error(&err);
    assert_ne!(
        code, 0,
        "should have a non-zero error code from CPI failure"
    );
}
