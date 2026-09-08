// SPDX-License-Identifier: Apache-2.0

use super::*;

const EXTRA_PREFUND_LAMPORTS: u64 = 50_000_000;

/// Pre-existing lamports on the GMP account PDA do not break `init_if_needed`
/// or `invoke_signed` during the recv flow.
#[tokio::test]
async fn test_gmp_prefunded_pda_not_blocked() {
    // ── Attestors (independent per chain) ──
    let attestors_a = Attestors::new(2);
    let attestors_b = Attestors::new(3);

    // ── Actors ──
    let deployer = Deployer::new();
    let admin = Admin::new();
    let relayer = Relayer::new();
    let user = User::new();

    // ── Test data ──
    let sequence = 1u64;
    let increment_amount = 42u64;

    // ── Chains ──
    let attestation_lc_a = AttestationLc::new(&attestors_a);
    let programs_a: &[&dyn ChainProgram] = &[&Ics27Gmp, &TestGmpApp, &attestation_lc_a];

    let attestation_lc_b = AttestationLc::new(&attestors_b);
    let programs_b: &[&dyn ChainProgram] = &[&Ics27Gmp, &TestGmpApp, &attestation_lc_b];

    let (mut chain_a, mut chain_b) = Chain::pair(&deployer, programs_a, programs_b);
    chain_a.prefund(&[&admin, &relayer, &user]);
    chain_b.prefund(&[&admin, &relayer]);

    let gmp_account_pda = gmp::derive_gmp_account_pda(chain_b.client_id(), &user.pubkey());
    chain_b.prefund_lamports(gmp_account_pda, EXTRA_PREFUND_LAMPORTS);

    // ── Init ──
    chain_a
        .init_with_attestation(&deployer, &admin, &relayer, programs_a, &attestors_a)
        .await;
    chain_b
        .init_with_attestation(&deployer, &admin, &relayer, programs_b, &attestors_b)
        .await;

    // ── Build payload ──
    let user_counter_pda = gmp::derive_user_counter_pda(&gmp_account_pda);
    let counter_app_state = chain_b.counter_app_state_pda();

    let solana_payload = gmp::encode_increment_payload(
        counter_app_state,
        user_counter_pda,
        gmp_account_pda,
        increment_amount,
    );
    let gmp_packet_bytes =
        gmp::encode_gmp_packet(&user.pubkey(), &test_gmp_app::ID, &solana_payload);

    // ── Send ──
    let commitment_pda = user
        .send_call(
            &mut chain_a,
            GmpSendCallParams {
                sequence,
                timeout_timestamp: GMP_TIMEOUT,
                receiver: &test_gmp_app::ID.to_string(),
                payload: solana_payload.encode_to_vec(),
            },
        )
        .await
        .expect("send_call failed");

    // ── Build attestation proof for recv ──
    let packet_commitment = read_commitment(&chain_a, commitment_pda).await;
    let recv_entry = attestation::packet_commitment_entry(
        chain_b.counterparty_client_id(),
        sequence,
        packet_commitment,
    );
    let recv_proof =
        attestation::build_packet_membership_proof(&attestors_b, PROOF_HEIGHT, &[recv_entry]);
    let recv_proof_bytes = attestation::serialize_proof(&recv_proof);

    // ── Recv ──
    let (b_payload, b_proof) = relayer
        .upload_chunks(&mut chain_b, sequence, &gmp_packet_bytes, &recv_proof_bytes)
        .await
        .expect("upload recv chunks failed");

    let remaining = gmp::build_increment_remaining_accounts(
        gmp_account_pda,
        counter_app_state,
        user_counter_pda,
    );

    let recv = relayer
        .gmp_recv_packet(
            &mut chain_b,
            GmpRecvPacketParams {
                sequence,
                payload_chunk_pda: b_payload,
                proof_chunk_pda: b_proof,
                remaining_accounts: remaining,
            },
        )
        .await
        .expect("recv_packet should succeed despite pre-funded PDA");

    let user_counter = read_user_counter(&chain_b, user_counter_pda).await;
    assert_eq!(user_counter.count, increment_amount);

    // ── Build attestation proof for ack ──
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

    // ── Ack ──
    let (a_payload, a_proof) = relayer
        .upload_chunks(&mut chain_a, sequence, &gmp_packet_bytes, &ack_proof_bytes)
        .await
        .expect("upload ack chunks failed");

    let raw_ack = ics27_gmp::encoding::encode_gmp_ack(
        &increment_amount.to_le_bytes(),
        gmp::ICS27_ENCODING_PROTOBUF,
    )
    .expect("encode GMP ack");

    let ack_commitment_pda = relayer
        .gmp_ack_packet(
            &mut chain_a,
            GmpAckPacketParams {
                sequence,
                acknowledgement: raw_ack,
                payload_chunk_pda: a_payload,
                proof_chunk_pda: a_proof,
            },
        )
        .await
        .expect("ack_packet failed");

    assert_commitment_zeroed(&chain_a, ack_commitment_pda).await;
    assert_gmp_result_exists(&chain_a, chain_a.client_id(), sequence).await;
}
