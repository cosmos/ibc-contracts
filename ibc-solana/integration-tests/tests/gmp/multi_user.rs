// SPDX-License-Identifier: Apache-2.0

use super::*;
use solana_sdk::pubkey::Pubkey;

struct LifecycleTarget {
    gmp_pda: Pubkey,
    counter_pda: Pubkey,
    counter_app_state: Pubkey,
}

/// Run one full send -> recv -> ack lifecycle with attestation proofs.
///
/// `expected_counter` is the accumulated counter value *after* this increment
/// (the value that `test_gmp_app` returns as ack data).
#[allow(clippy::too_many_arguments)]
async fn lifecycle(
    user: &User,
    relayer: &Relayer,
    chain_a: &mut Chain,
    chain_b: &mut Chain,
    attestors_a: &Attestors,
    attestors_b: &Attestors,
    sequence: u64,
    amount: u64,
    expected_counter: u64,
    target: &LifecycleTarget,
) {
    let payload = gmp::encode_increment_payload(
        target.counter_app_state,
        target.counter_pda,
        target.gmp_pda,
        amount,
    );
    let packet = gmp::encode_gmp_packet(&user.pubkey(), &test_gmp_app::ID, &payload);

    let commitment_pda = user
        .send_call(
            chain_a,
            GmpSendCallParams {
                sequence,
                timeout_timestamp: GMP_TIMEOUT,
                receiver: &test_gmp_app::ID.to_string(),
                payload: payload.encode_to_vec(),
            },
        )
        .await
        .expect("send_call failed");

    // Build attestation proof for recv on chain_b
    let packet_commitment = read_commitment(chain_a, commitment_pda).await;
    let recv_entry = attestation::packet_commitment_entry(
        chain_b.counterparty_client_id(),
        sequence,
        packet_commitment,
    );
    let recv_proof =
        attestation::build_packet_membership_proof(attestors_b, PROOF_HEIGHT, &[recv_entry]);
    let recv_proof_bytes = attestation::serialize_proof(&recv_proof);

    let (b_payload_pda, b_proof_pda) = relayer
        .upload_chunks(chain_b, sequence, &packet, &recv_proof_bytes)
        .await
        .expect("upload recv chunks failed");

    let remaining = gmp::build_increment_remaining_accounts(
        target.gmp_pda,
        target.counter_app_state,
        target.counter_pda,
    );

    let recv = relayer
        .gmp_recv_packet(
            chain_b,
            GmpRecvPacketParams {
                sequence,
                payload_chunk_pda: b_payload_pda,
                proof_chunk_pda: b_proof_pda,
                remaining_accounts: remaining,
            },
        )
        .await
        .expect("recv_packet failed");

    // Build attestation proof for ack on chain_a
    let ack_commitment = extract_ack_data(chain_b, recv.ack_pda).await;
    let ack_entry = attestation::ack_commitment_entry(
        chain_a.counterparty_client_id(),
        sequence,
        ack_commitment
            .as_slice()
            .try_into()
            .expect("ack should be 32 bytes"),
    );
    let ack_proof =
        attestation::build_packet_membership_proof(attestors_a, PROOF_HEIGHT, &[ack_entry]);
    let ack_proof_bytes = attestation::serialize_proof(&ack_proof);

    let (a_payload_pda, a_proof_pda) = relayer
        .upload_chunks(chain_a, sequence, &packet, &ack_proof_bytes)
        .await
        .expect("upload ack chunks failed");

    let raw_ack = ics27_gmp::encoding::encode_gmp_ack(
        &expected_counter.to_le_bytes(),
        gmp::ICS27_ENCODING_PROTOBUF,
    )
    .expect("encode GMP ack");

    relayer
        .gmp_ack_packet(
            chain_a,
            GmpAckPacketParams {
                sequence,
                acknowledgement: raw_ack,
                payload_chunk_pda: a_payload_pda,
                proof_chunk_pda: a_proof_pda,
            },
        )
        .await
        .expect("ack_packet failed");

    relayer
        .cleanup_chunks(chain_a, sequence, a_payload_pda, a_proof_pda)
        .await
        .expect("cleanup ack chunks failed");
}

/// Two independent users send GMP calls through the same chain pair.
/// Each user gets their own `UserCounter` PDA and `GMPCallResultAccount`.
#[tokio::test]
#[allow(clippy::too_many_lines)]
async fn test_gmp_multi_user_isolation() {
    // ── Attestors (independent per chain) ──
    let attestors_a = Attestors::new(2);
    let attestors_b = Attestors::new(3);

    // ── Actors ──
    let deployer = Deployer::new();
    let admin = Admin::new();
    let relayer = Relayer::new();
    let user_a = User::new();
    let user_b = User::new();

    // ── Chains ──
    let attestation_lc_a = AttestationLc::new(&attestors_a);
    let programs_a: &[&dyn ChainProgram] = &[&Ics27Gmp, &TestGmpApp, &attestation_lc_a];

    let attestation_lc_b = AttestationLc::new(&attestors_b);
    let programs_b: &[&dyn ChainProgram] = &[&Ics27Gmp, &TestGmpApp, &attestation_lc_b];

    let (mut chain_a, mut chain_b) = Chain::pair(&deployer, programs_a, programs_b);
    chain_a.prefund(&[&admin, &relayer, &user_a, &user_b]);
    chain_b.prefund(&[&admin, &relayer]);

    let gmp_pda_a = gmp::derive_gmp_account_pda(chain_b.client_id(), &user_a.pubkey());
    let gmp_pda_b = gmp::derive_gmp_account_pda(chain_b.client_id(), &user_b.pubkey());
    chain_b.prefund_lamports(gmp_pda_a, GMP_ACCOUNT_PREFUND_LAMPORTS);
    chain_b.prefund_lamports(gmp_pda_b, GMP_ACCOUNT_PREFUND_LAMPORTS);

    // ── Init ──
    chain_a
        .init_with_attestation(&deployer, &admin, &relayer, programs_a, &attestors_a)
        .await;
    chain_b
        .init_with_attestation(&deployer, &admin, &relayer, programs_b, &attestors_b)
        .await;

    // ── Build targets ──
    let counter_pda_a = gmp::derive_user_counter_pda(&gmp_pda_a);
    let counter_pda_b = gmp::derive_user_counter_pda(&gmp_pda_b);
    let counter_app_state = chain_b.counter_app_state_pda();

    let target_a = LifecycleTarget {
        gmp_pda: gmp_pda_a,
        counter_pda: counter_pda_a,
        counter_app_state,
    };
    let target_b = LifecycleTarget {
        gmp_pda: gmp_pda_b,
        counter_pda: counter_pda_b,
        counter_app_state,
    };

    // ── user_a: seq=1, amount=5 → counter 0→5 ──
    lifecycle(
        &user_a,
        &relayer,
        &mut chain_a,
        &mut chain_b,
        &attestors_a,
        &attestors_b,
        1,
        5,
        5,
        &target_a,
    )
    .await;

    // ── user_a: seq=2, amount=7 → counter 5→12 ──
    lifecycle(
        &user_a,
        &relayer,
        &mut chain_a,
        &mut chain_b,
        &attestors_a,
        &attestors_b,
        2,
        7,
        12,
        &target_a,
    )
    .await;

    // ── user_b: seq=3, amount=3 → counter 0→3 ──
    lifecycle(
        &user_b,
        &relayer,
        &mut chain_a,
        &mut chain_b,
        &attestors_a,
        &attestors_b,
        3,
        3,
        3,
        &target_b,
    )
    .await;

    // ── Assertions ──

    let counter_a = read_user_counter(&chain_b, counter_pda_a).await;
    assert_eq!(counter_a.count, 12, "user_a counter should be 5 + 7 = 12");

    let counter_b = read_user_counter(&chain_b, counter_pda_b).await;
    assert_eq!(counter_b.count, 3, "user_b counter should be 3");

    let state = read_counter_app_state(&chain_b, counter_app_state).await;
    assert_eq!(state.total_counters, 2, "should have 2 separate counters");

    for seq in [1u64, 2, 3] {
        assert_gmp_result_exists(&chain_a, chain_a.client_id(), seq).await;
    }
}
