// SPDX-License-Identifier: Apache-2.0

use super::*;

/// Send 3 packets A->B, recv all on B, ack all on A.
#[tokio::test]
async fn test_multiple_sequential_packets() {
    // ── Attestors ──
    let attestors_a = Attestors::new(2);
    let attestors_b = Attestors::new(3);

    // ── Actors ──
    let deployer = Deployer::new();
    let admin = Admin::new();
    let relayer = Relayer::new();
    let user = User::new();

    // ── Test data ──
    let successful_ack = test_ibc_app::instructions::SUCCESSFUL_ACKNOWLEDGEMENT_JSON.to_vec();
    let packets: [(u64, &[u8]); 3] = [(1, b"packet one"), (2, b"packet two"), (3, b"packet three")];

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

    // ── User sends all 3 packets on A ──
    let mut send_results = Vec::new();
    for &(seq, data) in &packets {
        let send = user
            .send_packet(
                &mut chain_a,
                SendPacketParams {
                    sequence: seq,
                    packet_data: data,
                },
            )
            .await
            .unwrap_or_else(|e| panic!("send seq={seq} failed: {e:?}"));
        send_results.push(send);
    }

    let a_state = read_app_state(&chain_a).await;
    assert_eq!(a_state.packets_sent, 3);

    // ── Relayer uploads chunks and delivers all 3 packets to B ──
    let mut recv_results = Vec::new();
    for (i, &(seq, data)) in packets.iter().enumerate() {
        let commitment = read_commitment(&chain_a, send_results[i].commitment_pda).await;
        let recv_entry =
            attestation::packet_commitment_entry(chain_b.counterparty_client_id(), seq, commitment);
        let recv_proof =
            attestation::build_packet_membership_proof(&attestors_b, PROOF_HEIGHT, &[recv_entry]);
        let recv_proof_bytes = attestation::serialize_proof(&recv_proof);

        let (payload, proof) = relayer
            .upload_chunks(&mut chain_b, seq, data, &recv_proof_bytes)
            .await
            .unwrap_or_else(|e| panic!("upload B recv chunks seq={seq} failed: {e:?}"));
        let recv = relayer
            .recv_packet(
                &mut chain_b,
                RecvPacketParams {
                    sequence: seq,
                    payload_chunk_pda: payload,
                    proof_chunk_pda: proof,
                    app_program: test_ibc_app::ID,
                    ..Default::default()
                },
            )
            .await
            .unwrap_or_else(|e| panic!("recv seq={seq} failed: {e:?}"));
        recv_results.push(recv);
    }

    let b_state = read_app_state(&chain_b).await;
    assert_eq!(b_state.packets_received, 3);

    // ── Relayer uploads chunks and delivers all 3 acks back to A ──
    for (i, &(seq, data)) in packets.iter().enumerate() {
        let ack_commitment = extract_ack_data(&chain_b, recv_results[i].ack_pda).await;
        let ack_entry = attestation::ack_commitment_entry(
            chain_a.counterparty_client_id(),
            seq,
            ack_commitment
                .as_slice()
                .try_into()
                .expect("ack should be 32 bytes"),
        );
        let ack_proof =
            attestation::build_packet_membership_proof(&attestors_a, PROOF_HEIGHT, &[ack_entry]);
        let ack_proof_bytes = attestation::serialize_proof(&ack_proof);

        let (payload, proof) = relayer
            .upload_chunks(&mut chain_a, seq, data, &ack_proof_bytes)
            .await
            .unwrap_or_else(|e| panic!("upload A ack chunks seq={seq} failed: {e:?}"));
        let commitment_pda = relayer
            .ack_packet(
                &mut chain_a,
                AckPacketParams {
                    sequence: seq,
                    acknowledgement: successful_ack.clone(),
                    payload_chunk_pda: payload,
                    proof_chunk_pda: proof,
                    app_program: test_ibc_app::ID,
                    ..Default::default()
                },
            )
            .await
            .unwrap_or_else(|e| panic!("ack seq={seq} failed: {e:?}"));

        assert_commitment_zeroed(&chain_a, commitment_pda).await;
    }

    // ── Verify final counters ──
    let a_state = read_app_state(&chain_a).await;
    assert_eq!(a_state.packets_sent, 3);
    assert_eq!(a_state.packets_acknowledged, 3);

    let b_state = read_app_state(&chain_b).await;
    assert_eq!(b_state.packets_received, 3);
}
