// SPDX-License-Identifier: Apache-2.0

use super::*;
use integration_tests::chain::ChainConfig;

/// Full send -> recv -> ack lifecycle using the attestation light client
/// with two attestors (2-of-2 quorum).
#[tokio::test]
async fn test_attestation_send_recv_ack_roundtrip() {
    // ── Attestors ──
    let attestors = Attestors::new(2);

    // ── Actors ──
    let deployer = Deployer::new();
    let admin = Admin::new();
    let relayer = Relayer::new();
    let user = User::new();

    // ── Test data ──
    let packet_data = b"attestation roundtrip";
    let sequence = 1u64;
    let successful_ack = test_ibc_app::instructions::SUCCESSFUL_ACKNOWLEDGEMENT_JSON.to_vec();

    // ── Chains ──
    let attestation_lc = AttestationLc::new(&attestors);
    let programs: &[&dyn ChainProgram] = &[&TestIbcApp, &attestation_lc];

    let mut chain_a = Chain::new(ChainConfig {
        client_id: "chain-a-client",
        counterparty_client_id: "chain-b-client",
        deployer: &deployer,
        programs,
    });
    chain_a.prefund(&[&admin, &relayer, &user]);

    let mut chain_b = Chain::new(ChainConfig {
        client_id: "chain-b-client",
        counterparty_client_id: "chain-a-client",
        deployer: &deployer,
        programs,
    });
    chain_b.prefund(&[&admin, &relayer]);

    // ── Init (includes update_client at PROOF_HEIGHT) ──
    chain_a
        .init_with_attestation(&deployer, &admin, &relayer, programs, &attestors)
        .await;
    chain_b
        .init_with_attestation(&deployer, &admin, &relayer, programs, &attestors)
        .await;

    // ── User sends on Chain A ──
    let send = user
        .send_packet(
            &mut chain_a,
            SendPacketParams {
                sequence,
                packet_data,
            },
        )
        .await
        .expect("send_packet on A failed");

    assert_commitment_set(&chain_a, send.commitment_pda).await;

    // ── Build attestation proof for recv on Chain B ──
    // The router verifies: path = prefixed(commitment_path(source_client, seq))
    // where source_client = packet.source_client = chain A's client_id
    let packet_commitment = ics24::packet_commitment_bytes32(&send.packet);
    let recv_entry = attestation::packet_commitment_entry(
        chain_b.counterparty_client_id(),
        sequence,
        packet_commitment,
    );
    let recv_proof =
        attestation::build_packet_membership_proof(&attestors, PROOF_HEIGHT, &[recv_entry]);
    let recv_proof_bytes = attestation::serialize_proof(&recv_proof);

    // ── Relayer uploads chunks and delivers to Chain B ──
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
                ..Default::default()
            },
        )
        .await
        .expect("recv_packet on B failed");

    assert_receipt_created(&chain_b, recv.receipt_pda).await;
    assert_commitment_set(&chain_b, recv.ack_pda).await;

    // ── Build attestation proof for ack on Chain A ──
    // The router verifies: path = prefixed(ack_commitment_path(dest_client, seq))
    // where dest_client = chain A's counterparty_client_id
    let ack_data = extract_ack_data(&chain_b, recv.ack_pda).await;
    let ack_entry = attestation::ack_commitment_entry(
        chain_a.counterparty_client_id(),
        sequence,
        ack_data
            .as_slice()
            .try_into()
            .expect("ack should be 32 bytes"),
    );
    let ack_proof =
        attestation::build_packet_membership_proof(&attestors, PROOF_HEIGHT, &[ack_entry]);
    let ack_proof_bytes = attestation::serialize_proof(&ack_proof);

    // ── Relayer uploads chunks and delivers ack back to Chain A ──
    let (a_ack_payload, a_ack_proof) = relayer
        .upload_chunks(&mut chain_a, sequence, packet_data, &ack_proof_bytes)
        .await
        .expect("upload ack chunks on A failed");

    let commitment_pda = relayer
        .ack_packet(
            &mut chain_a,
            AckPacketParams {
                sequence,
                acknowledgement: successful_ack,
                payload_chunk_pda: a_ack_payload,
                proof_chunk_pda: a_ack_proof,
                ..Default::default()
            },
        )
        .await
        .expect("ack_packet on A failed");

    assert_commitment_zeroed(&chain_a, commitment_pda).await;

    // ── Final assertions ──
    let a_state = router::read_test_ibc_app_state(&chain_a).await;
    assert_eq!(a_state.packets_sent, 1);
    assert_eq!(a_state.packets_acknowledged, 1);

    let b_state = router::read_test_ibc_app_state(&chain_b).await;
    assert_eq!(b_state.packets_received, 1);
}
