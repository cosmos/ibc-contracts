// SPDX-License-Identifier: Apache-2.0

use super::*;
use integration_tests::programs::ATTESTATION_PROGRAM_ID;

/// Three-chain plain-router roundtrip: A->B then B->C, with independent IBC
/// packet lifecycles on each hop.
///
/// Mirrors `tests/gmp/three_chain.rs` but exercises the bare router via
/// `test_ibc_app` instead of the GMP application stack. Verifies that a chain
/// in the middle of a multi-hop path can both receive on its primary client
/// and dispatch a fresh send via a secondary client.
///
/// Chain A: client `"a-to-b"` <-> counterparty `"b-to-a"`
/// Chain B: primary `"b-to-a"` <-> `"a-to-b"`, additional `"b-to-c"` <-> `"c-to-b"`
/// Chain C: client `"c-to-b"` <-> counterparty `"b-to-c"`
#[tokio::test]
async fn test_router_three_chain_roundtrip() {
    // ── Attestors ──
    let attestors_a = Attestors::new(2);
    let attestors_b = Attestors::new(3);
    let attestors_c = Attestors::new(2);

    // ── Actors ──
    let deployer = Deployer::new();
    let admin = Admin::new();
    let relayer = Relayer::new();
    let user = User::new();

    // ── Test data ──
    let packet_data_ab = b"hello from chain A";
    let packet_data_bc = b"hello from chain B";
    let successful_ack = test_ibc_app::instructions::SUCCESSFUL_ACKNOWLEDGEMENT_JSON.to_vec();

    // ── Chains ──
    let attestation_lc_a = AttestationLc::new(&attestors_a);
    let attestation_lc_b = AttestationLc::new(&attestors_b);
    let attestation_lc_c = AttestationLc::new(&attestors_c);

    let programs_a: &[&dyn ChainProgram] = &[&TestIbcApp, &attestation_lc_a];
    let programs_b: &[&dyn ChainProgram] = &[&TestIbcApp, &attestation_lc_b];
    let programs_c: &[&dyn ChainProgram] = &[&TestIbcApp, &attestation_lc_c];

    let mut chain_a = Chain::new(ChainConfig {
        client_id: "a-to-b",
        counterparty_client_id: "b-to-a",
        deployer: &deployer,
        programs: programs_a,
    });
    chain_a.prefund(&[&admin, &relayer, &user]);

    let mut chain_b = Chain::new(ChainConfig {
        client_id: "b-to-a",
        counterparty_client_id: "a-to-b",
        deployer: &deployer,
        programs: programs_b,
    });
    chain_b.prefund(&[&admin, &relayer, &user]);

    let mut chain_c = Chain::new(ChainConfig {
        client_id: "c-to-b",
        counterparty_client_id: "b-to-c",
        deployer: &deployer,
        programs: programs_c,
    });
    chain_c.prefund(&[&admin, &relayer]);

    // ── Init ──
    chain_a
        .init_with_attestation(&deployer, &admin, &relayer, programs_a, &attestors_a)
        .await;

    // Chain B needs the standard init plus a second client (`b-to-c`) wired
    // up via `add_counterparty_with_attestation`.
    chain_b.start().await;
    deployer
        .init_ibc_stack(&mut chain_b, &admin, &relayer, programs_b)
        .await;
    deployer
        .add_counterparty_with_attestation(
            &mut chain_b,
            &admin,
            "b-to-c",
            "c-to-b",
            ATTESTATION_PROGRAM_ID,
        )
        .await;
    deployer
        .transfer_upgrade_authority(&mut chain_b, programs_b)
        .await;
    relayer
        .attestation_update_client(&mut chain_b, &attestors_b, PROOF_HEIGHT)
        .await
        .expect("update_client on chain B failed");

    chain_c
        .init_with_attestation(&deployer, &admin, &relayer, programs_c, &attestors_c)
        .await;

    // LC accounts for the secondary client on B (same attestation program)
    let lc_b_secondary = attestation_lc_accounts(ATTESTATION_PROGRAM_ID, PROOF_HEIGHT);

    // ── Leg 1: A -> B ──
    let send_a = user
        .send_packet(
            &mut chain_a,
            SendPacketParams {
                sequence: 1,
                packet_data: packet_data_ab,
            },
        )
        .await
        .expect("A->B send_packet failed");
    assert_commitment_set(&chain_a, send_a.commitment_pda).await;

    let recv_proof_ab_bytes = attestation::build_recv_proof_bytes(
        &chain_a,
        send_a.commitment_pda,
        chain_b.counterparty_client_id(),
        1,
        &attestors_b,
    )
    .await;

    let (b_recv_payload, b_recv_proof) = relayer
        .upload_chunks(&mut chain_b, 1, packet_data_ab, &recv_proof_ab_bytes)
        .await
        .expect("upload A->B recv chunks failed");
    let recv_on_b = relayer
        .recv_packet(
            &mut chain_b,
            RecvPacketParams {
                sequence: 1,
                payload_chunk_pda: b_recv_payload,
                proof_chunk_pda: b_recv_proof,
                ..Default::default()
            },
        )
        .await
        .expect("A->B recv_packet failed");
    assert_receipt_created(&chain_b, recv_on_b.receipt_pda).await;
    assert_commitment_set(&chain_b, recv_on_b.ack_pda).await;

    let ack_proof_ab_bytes = attestation::build_ack_proof_bytes(
        &chain_b,
        recv_on_b.ack_pda,
        chain_a.counterparty_client_id(),
        1,
        &attestors_a,
    )
    .await;

    let (a_ack_payload, a_ack_proof) = relayer
        .upload_chunks(&mut chain_a, 1, packet_data_ab, &ack_proof_ab_bytes)
        .await
        .expect("upload A->B ack chunks failed");
    let ack_commitment_a = relayer
        .ack_packet(
            &mut chain_a,
            AckPacketParams {
                sequence: 1,
                acknowledgement: successful_ack.clone(),
                payload_chunk_pda: a_ack_payload,
                proof_chunk_pda: a_ack_proof,
                ..Default::default()
            },
        )
        .await
        .expect("A->B ack_packet failed");
    assert_commitment_zeroed(&chain_a, ack_commitment_a).await;

    // Chain A: 1 sent, 1 acked. Chain B: 1 received.
    let a_state = read_app_state(&chain_a).await;
    assert_eq!(a_state.packets_sent, 1);
    assert_eq!(a_state.packets_acknowledged, 1);
    let b_state = read_app_state(&chain_b).await;
    assert_eq!(b_state.packets_received, 1);

    // ── Leg 2: B -> C ──
    let send_bc = user
        .send_packet_for_client(
            &mut chain_b,
            "b-to-c",
            "c-to-b",
            &lc_b_secondary,
            SendPacketParams {
                sequence: 1,
                packet_data: packet_data_bc,
            },
        )
        .await
        .expect("B->C send_packet failed");
    assert_commitment_set(&chain_b, send_bc.commitment_pda).await;

    let recv_proof_bc_bytes = attestation::build_recv_proof_bytes(
        &chain_b,
        send_bc.commitment_pda,
        chain_c.counterparty_client_id(),
        1,
        &attestors_c,
    )
    .await;

    let (c_recv_payload, c_recv_proof) = relayer
        .upload_chunks(&mut chain_c, 1, packet_data_bc, &recv_proof_bc_bytes)
        .await
        .expect("upload B->C recv chunks failed");
    let recv_on_c = relayer
        .recv_packet(
            &mut chain_c,
            RecvPacketParams {
                sequence: 1,
                payload_chunk_pda: c_recv_payload,
                proof_chunk_pda: c_recv_proof,
                ..Default::default()
            },
        )
        .await
        .expect("B->C recv_packet failed");
    assert_receipt_created(&chain_c, recv_on_c.receipt_pda).await;
    assert_commitment_set(&chain_c, recv_on_c.ack_pda).await;

    let ack_proof_bc_bytes =
        attestation::build_ack_proof_bytes(&chain_c, recv_on_c.ack_pda, "c-to-b", 1, &attestors_b)
            .await;

    let (b_ack_payload, b_ack_proof) = relayer
        .upload_chunks_for_client(
            &mut chain_b,
            "b-to-c",
            1,
            packet_data_bc,
            &ack_proof_bc_bytes,
        )
        .await
        .expect("upload B->C ack chunks failed");
    let ack_commitment_b = relayer
        .ack_packet_for_client(
            &mut chain_b,
            "b-to-c",
            "c-to-b",
            &lc_b_secondary,
            AckPacketParams {
                sequence: 1,
                acknowledgement: successful_ack,
                payload_chunk_pda: b_ack_payload,
                proof_chunk_pda: b_ack_proof,
                ..Default::default()
            },
        )
        .await
        .expect("B->C ack_packet failed");
    assert_commitment_zeroed(&chain_b, ack_commitment_b).await;

    // ── Final assertions ──
    // Chain A: 1 sent, 1 acked.
    let a_state = read_app_state(&chain_a).await;
    assert_eq!(a_state.packets_sent, 1);
    assert_eq!(a_state.packets_acknowledged, 1);

    // Chain B: 1 received (leg 1) + 1 sent + 1 acked (leg 2).
    let b_state = read_app_state(&chain_b).await;
    assert_eq!(b_state.packets_sent, 1);
    assert_eq!(b_state.packets_received, 1);
    assert_eq!(b_state.packets_acknowledged, 1);

    // Chain C: 1 received (leg 2).
    let c_state = read_app_state(&chain_c).await;
    assert_eq!(c_state.packets_received, 1);
}
