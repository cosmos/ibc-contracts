// SPDX-License-Identifier: Apache-2.0

use super::*;

/// Full lifecycle with a 2-chunk proof: the attestation proof exceeds
/// `CHUNK_DATA_SIZE` (900 bytes) when signed by 12 attestors, splitting
/// it across two chunk accounts.
#[tokio::test]
async fn test_multi_chunk_proof_lifecycle() {
    // ── Attestors ──
    // Use 12 attestors so the proof exceeds 900 bytes and requires 2 chunks.
    let attestors_a = Attestors::new(12);
    let attestors_b = Attestors::new(12);

    // ── Actors ──
    let deployer = Deployer::new();
    let admin = Admin::new();
    let relayer = Relayer::new();
    let user = User::new();

    // ── Test data ──
    let packet_data = b"multi-chunk proof test";
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
    // 12 ECDSA recoveries exceed the default 200K CU budget, so use manual
    // `init` + `attestation_update_client_with_budget` instead of `init_with_attestation`.
    chain_a.init(&deployer, &admin, &relayer, programs_a).await;
    relayer
        .attestation_update_client_with_budget(&mut chain_a, &attestors_a, PROOF_HEIGHT, 500_000)
        .await
        .expect("update_client on A with 12 attestors");
    chain_b.init(&deployer, &admin, &relayer, programs_b).await;
    relayer
        .attestation_update_client_with_budget(&mut chain_b, &attestors_b, PROOF_HEIGHT, 500_000)
        .await
        .expect("update_client on B with 12 attestors");

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

    // ── Build recv proof (12 attestors -> > 900 bytes) ──
    let commitment = read_commitment(&chain_a, send.commitment_pda).await;
    let recv_entry = attestation::packet_commitment_entry(
        chain_b.counterparty_client_id(),
        sequence,
        commitment,
    );
    let recv_proof =
        attestation::build_packet_membership_proof(&attestors_b, PROOF_HEIGHT, &[recv_entry]);
    let recv_proof_bytes = attestation::serialize_proof(&recv_proof);

    // Split proof into 900-byte chunks
    let chunk_size = 900;
    let recv_proof_chunks: Vec<Vec<u8>> = recv_proof_bytes
        .chunks(chunk_size)
        .map(<[u8]>::to_vec)
        .collect();
    assert!(
        recv_proof_chunks.len() >= 2,
        "12-attestor proof should exceed 900 bytes and need multiple chunks (got {} bytes)",
        recv_proof_bytes.len()
    );

    // Relayer uploads 1 payload chunk + N proof chunks to B
    let (b_payload, b_proof_pdas) = relayer
        .upload_chunks_with_multi_proof(&mut chain_b, sequence, packet_data, &recv_proof_chunks)
        .await
        .expect("upload multi-chunk proof failed on B");

    // Relayer delivers recv_packet with N proof chunks (needs extra CU for 12 attestors)
    let recv = relayer
        .recv_packet_multi_proof_with_budget(
            &mut chain_b,
            RecvPacketParams {
                sequence,
                payload_chunk_pda: b_payload,
                app_program: test_ibc_app::ID,
                ..Default::default()
            },
            &b_proof_pdas,
            600_000,
        )
        .await
        .expect("recv_packet with multi-chunk proof failed");

    assert_receipt_created(&chain_b, recv.receipt_pda).await;
    assert_commitment_set(&chain_b, recv.ack_pda).await;

    // ── Build ack proof (12 attestors -> > 900 bytes) ──
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

    let ack_proof_chunks: Vec<Vec<u8>> = ack_proof_bytes
        .chunks(chunk_size)
        .map(<[u8]>::to_vec)
        .collect();

    // Relayer uploads 1 payload chunk + N proof chunks to A for ack
    let (a_payload, a_proof_pdas) = relayer
        .upload_chunks_with_multi_proof(&mut chain_a, sequence, packet_data, &ack_proof_chunks)
        .await
        .expect("upload multi-chunk proof failed on A");

    // Relayer delivers ack_packet with N proof chunks
    let commitment_pda = relayer
        .ack_packet_multi_proof_with_budget(
            &mut chain_a,
            AckPacketParams {
                sequence,
                acknowledgement: successful_ack,
                payload_chunk_pda: a_payload,
                app_program: test_ibc_app::ID,
                ..Default::default()
            },
            &a_proof_pdas,
            600_000,
        )
        .await
        .expect("ack_packet with multi-chunk proof failed");

    assert_commitment_zeroed(&chain_a, commitment_pda).await;
}
