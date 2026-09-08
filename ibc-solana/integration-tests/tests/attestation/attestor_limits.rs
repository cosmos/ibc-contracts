// SPDX-License-Identifier: Apache-2.0

use super::*;
use integration_tests::{chain::ChainConfig, Actor};

/// Solana's maximum transaction size (1280 IPv6 MTU - 40 IP header - 8 UDP header).
const PACKET_DATA_SIZE: usize = 1232;

/// Maximum attestors whose signatures fit in a single `update_client` transaction.
const MAX_ATTESTORS_WITHOUT_BATCHING: usize = 11;

/// Compute budget for transactions that verify many ECDSA signatures.
/// The ack path is the most expensive: proof verification + commitment zeroing.
const ATTESTATION_COMPUTE_UNITS: u32 = 500_000;

/// Full send -> recv -> ack lifecycle with 11 attestors (11-of-11 quorum),
/// the maximum count whose signatures fit within a single `update_client`
/// transaction without batching.
///
/// Proofs with 11 signatures exceed the 900-byte chunk limit and require
/// multi-chunk delivery; signature verification needs an elevated compute
/// budget.
#[tokio::test]
async fn test_11_attestors_send_recv_ack() {
    // ── Attestors ──
    let attestors = Attestors::new(MAX_ATTESTORS_WITHOUT_BATCHING);

    // ── Actors ──
    let deployer = Deployer::new();
    let admin = Admin::new();
    let relayer = Relayer::new();
    let user = User::new();

    // ── Test data ──
    let packet_data = b"11 attestors roundtrip";
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

    // ── Init ──
    chain_a.init(&deployer, &admin, &relayer, programs).await;
    chain_b.init(&deployer, &admin, &relayer, programs).await;

    // ── Update client (create consensus state at PROOF_HEIGHT) ──
    // 11 ECDSA recoveries exceed the default 200K CU budget.
    relayer
        .attestation_update_client_with_budget(
            &mut chain_a,
            &attestors,
            PROOF_HEIGHT,
            ATTESTATION_COMPUTE_UNITS,
        )
        .await
        .expect("update_client with 11 attestors on A failed");

    relayer
        .attestation_update_client_with_budget(
            &mut chain_b,
            &attestors,
            PROOF_HEIGHT,
            ATTESTATION_COMPUTE_UNITS,
        )
        .await
        .expect("update_client with 11 attestors on B failed");

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
    let packet_commitment = ics24::packet_commitment_bytes32(&send.packet);
    let recv_entry = attestation::packet_commitment_entry(
        chain_b.counterparty_client_id(),
        sequence,
        packet_commitment,
    );
    let recv_proof =
        attestation::build_packet_membership_proof(&attestors, PROOF_HEIGHT, &[recv_entry]);
    let recv_proof_bytes = attestation::serialize_proof(&recv_proof);

    // Proof with 11 signatures exceeds the 900-byte chunk limit.
    let recv_proof_chunks = router::split_into_chunks(&recv_proof_bytes);

    // ── Relayer uploads chunks and delivers to Chain B ──
    let (b_recv_payload, b_recv_proof_pdas) = relayer
        .upload_chunks_with_multi_proof(&mut chain_b, sequence, packet_data, &recv_proof_chunks)
        .await
        .expect("upload recv chunks on B failed");

    let recv = relayer
        .recv_packet_multi_proof_with_budget(
            &mut chain_b,
            RecvPacketParams {
                sequence,
                payload_chunk_pda: b_recv_payload,
                ..Default::default()
            },
            &b_recv_proof_pdas,
            ATTESTATION_COMPUTE_UNITS,
        )
        .await
        .expect("recv_packet on B failed");

    assert_receipt_created(&chain_b, recv.receipt_pda).await;
    assert_commitment_set(&chain_b, recv.ack_pda).await;

    // ── Build attestation proof for ack on Chain A ──
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

    let ack_proof_chunks = router::split_into_chunks(&ack_proof_bytes);

    // ── Relayer uploads chunks and delivers ack back to Chain A ──
    let (a_ack_payload, a_ack_proof_pdas) = relayer
        .upload_chunks_with_multi_proof(&mut chain_a, sequence, packet_data, &ack_proof_chunks)
        .await
        .expect("upload ack chunks on A failed");

    let commitment_pda = relayer
        .ack_packet_multi_proof_with_budget(
            &mut chain_a,
            AckPacketParams {
                sequence,
                acknowledgement: successful_ack,
                payload_chunk_pda: a_ack_payload,
                ..Default::default()
            },
            &a_ack_proof_pdas,
            ATTESTATION_COMPUTE_UNITS,
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

/// Verify that 12 attestors produce an `update_client` transaction that
/// cannot be submitted and exceeds Solana's 1232-byte packet size limit.
///
/// Each ECDSA signature adds 69 bytes (4-byte Borsh length prefix + 65-byte
/// signature) to the instruction data. With 11 attestors the serialized
/// transaction fits (~1220 bytes); with 12 it overflows (~1289 bytes).
#[tokio::test]
async fn test_12_attestors_update_client_exceeds_tx_size() {
    // ── Attestors ──
    let attestors = Attestors::new(MAX_ATTESTORS_WITHOUT_BATCHING.saturating_add(1));

    // ── Actors ──
    let deployer = Deployer::new();
    let admin = Admin::new();
    let relayer = Relayer::new();

    // ── Chain ──
    let attestation_lc = AttestationLc::new(&attestors);
    let programs: &[&dyn ChainProgram] = &[&TestIbcApp, &attestation_lc];

    let mut chain = Chain::new(ChainConfig {
        client_id: "chain-a-client",
        counterparty_client_id: "chain-b-client",
        deployer: &deployer,
        programs,
    });
    chain.prefund(&[&admin, &relayer]);
    chain.init(&deployer, &admin, &relayer, programs).await;

    // ── Submit update_client — should fail ──
    let result = relayer
        .attestation_update_client(&mut chain, &attestors, PROOF_HEIGHT)
        .await;

    assert!(
        result.is_err(),
        "update_client with 12 attestors should fail"
    );

    // ── Verify the serialized tx exceeds the network packet limit ──
    // BanksClient processes transactions in-memory and does not enforce
    // the 1232-byte wire-format limit. On a real cluster the transaction
    // would be rejected before reaching the runtime.
    let proof = attestation::build_state_membership_proof(
        &attestors,
        PROOF_HEIGHT,
        chain.clock_time() as u64,
    );
    let update_ix = attestation::build_update_client_ix(relayer.pubkey(), PROOF_HEIGHT, proof);
    let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
        &[update_ix],
        Some(&relayer.pubkey()),
        &[relayer.keypair()],
        chain.blockhash(),
    );
    let serialized = bincode::serialize(&tx).expect("transaction serialization");

    assert!(
        serialized.len() > PACKET_DATA_SIZE,
        "12-attestor update_client tx ({} bytes) should exceed the {PACKET_DATA_SIZE}-byte limit",
        serialized.len(),
    );
}
