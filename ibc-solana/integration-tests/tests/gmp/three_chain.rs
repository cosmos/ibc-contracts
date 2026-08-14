// SPDX-License-Identifier: Apache-2.0

use super::*;
use integration_tests::chain::attestation_lc_accounts as att_lc_accounts;
use integration_tests::programs::{ATTESTATION_PROGRAM_ID, TEST_ATTESTATION_ID};
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use solana_sdk::transaction::Transaction;

/// Three-chain full-circle roundtrip: A→B→C→A→C→B→A.
///
/// Six independent GMP legs traverse every edge of the A-B-C triangle in
/// both directions. Each leg is a complete send→recv→ack lifecycle.
///
/// Topology:
///   Chain A: `"a-to-b"` ↔ `"b-to-a"`, `"a-to-c"` ↔ `"c-to-a"`
///   Chain B: `"b-to-a"` ↔ `"a-to-b"`, `"b-to-c"` ↔ `"c-to-b"`
///   Chain C: `"c-to-b"` ↔ `"b-to-c"`, `"c-to-a"` ↔ `"a-to-c"`
///
/// Each chain hosts two attestation LC instances (`ATTESTATION_PROGRAM_ID` and
/// `TEST_ATTESTATION_ID`) with independent attestor sets per directed
/// client connection.
#[tokio::test]
async fn test_gmp_three_chain_roundtrip() {
    // ── Attestor sets (one per directed connection) ──
    let attestors_a_ab = Attestors::new(2);
    let attestors_a_ac = Attestors::new(2);
    let attestors_b_ba = Attestors::new(2);
    let attestors_b_bc = Attestors::new(2);
    let attestors_c_cb = Attestors::new(2);
    let attestors_c_ca = Attestors::new(2);

    // ── Actors ──
    let deployer = Deployer::new();
    let admin = Admin::new();
    let relayer = Relayer::new();
    let user = User::new();

    // ── Chains ──
    // Each chain has two attestation LC instances: primary (ATTESTATION_PROGRAM_ID)
    // and secondary (TEST_ATTESTATION_ID) with independent attestor sets.
    let att_lc_a_primary = AttestationLc::new(&attestors_a_ab);
    let att_lc_a_secondary =
        AttestationLc::with_program_id(&attestors_a_ac, TEST_ATTESTATION_ID, "test_attestation");
    let programs_a: &[&dyn ChainProgram] = &[
        &Ics27Gmp,
        &TestGmpApp,
        &att_lc_a_primary,
        &att_lc_a_secondary,
    ];

    let att_lc_b_primary = AttestationLc::new(&attestors_b_ba);
    let att_lc_b_secondary =
        AttestationLc::with_program_id(&attestors_b_bc, TEST_ATTESTATION_ID, "test_attestation");
    let programs_b: &[&dyn ChainProgram] = &[
        &Ics27Gmp,
        &TestGmpApp,
        &att_lc_b_primary,
        &att_lc_b_secondary,
    ];

    let att_lc_c_primary = AttestationLc::new(&attestors_c_cb);
    let att_lc_c_secondary =
        AttestationLc::with_program_id(&attestors_c_ca, TEST_ATTESTATION_ID, "test_attestation");
    let programs_c: &[&dyn ChainProgram] = &[
        &Ics27Gmp,
        &TestGmpApp,
        &att_lc_c_primary,
        &att_lc_c_secondary,
    ];

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
    chain_c.prefund(&[&admin, &relayer, &user]);

    // GMP account PDAs (one per receiving-chain × receiving-client × sender).
    let gmp_b_from_a = gmp::derive_gmp_account_pda("b-to-a", &user.pubkey());
    let gmp_b_from_c = gmp::derive_gmp_account_pda("b-to-c", &user.pubkey());
    let gmp_c_from_b = gmp::derive_gmp_account_pda("c-to-b", &user.pubkey());
    let gmp_c_from_a = gmp::derive_gmp_account_pda("c-to-a", &user.pubkey());
    let gmp_a_from_b = gmp::derive_gmp_account_pda("a-to-b", &user.pubkey());
    let gmp_a_from_c = gmp::derive_gmp_account_pda("a-to-c", &user.pubkey());

    chain_a.prefund_lamports(gmp_a_from_b, GMP_ACCOUNT_PREFUND_LAMPORTS);
    chain_a.prefund_lamports(gmp_a_from_c, GMP_ACCOUNT_PREFUND_LAMPORTS);
    chain_b.prefund_lamports(gmp_b_from_a, GMP_ACCOUNT_PREFUND_LAMPORTS);
    chain_b.prefund_lamports(gmp_b_from_c, GMP_ACCOUNT_PREFUND_LAMPORTS);
    chain_c.prefund_lamports(gmp_c_from_b, GMP_ACCOUNT_PREFUND_LAMPORTS);
    chain_c.prefund_lamports(gmp_c_from_a, GMP_ACCOUNT_PREFUND_LAMPORTS);

    // ── Init ──
    // Each chain: init primary client, add secondary client, update_client
    // on secondary LC.

    // Chain A: primary "a-to-b" (ATTESTATION_PROGRAM_ID), secondary "a-to-c" (TEST_ATTESTATION_ID)
    chain_a
        .init_with_attestation(&deployer, &admin, &relayer, programs_a, &attestors_a_ab)
        .await;
    deployer
        .add_counterparty_with_attestation(
            &mut chain_a,
            &admin,
            "a-to-c",
            "c-to-a",
            TEST_ATTESTATION_ID,
        )
        .await;
    relayer
        .attestation_update_client_for_program(
            &mut chain_a,
            &attestors_a_ac,
            PROOF_HEIGHT,
            TEST_ATTESTATION_ID,
        )
        .await
        .expect("update secondary LC on A");

    // Chain B: primary "b-to-a" (ATTESTATION_PROGRAM_ID), secondary "b-to-c" (TEST_ATTESTATION_ID)
    chain_b
        .init_with_attestation(&deployer, &admin, &relayer, programs_b, &attestors_b_ba)
        .await;
    deployer
        .add_counterparty_with_attestation(
            &mut chain_b,
            &admin,
            "b-to-c",
            "c-to-b",
            TEST_ATTESTATION_ID,
        )
        .await;
    relayer
        .attestation_update_client_for_program(
            &mut chain_b,
            &attestors_b_bc,
            PROOF_HEIGHT,
            TEST_ATTESTATION_ID,
        )
        .await
        .expect("update secondary LC on B");

    // Chain C: primary "c-to-b" (ATTESTATION_PROGRAM_ID), secondary "c-to-a" (TEST_ATTESTATION_ID)
    chain_c
        .init_with_attestation(&deployer, &admin, &relayer, programs_c, &attestors_c_cb)
        .await;
    deployer
        .add_counterparty_with_attestation(
            &mut chain_c,
            &admin,
            "c-to-a",
            "a-to-c",
            TEST_ATTESTATION_ID,
        )
        .await;
    relayer
        .attestation_update_client_for_program(
            &mut chain_c,
            &attestors_c_ca,
            PROOF_HEIGHT,
            TEST_ATTESTATION_ID,
        )
        .await
        .expect("update secondary LC on C");

    // ── Connection contexts ──
    let conn_a_ab = ConnCtx {
        lc_program_id: ATTESTATION_PROGRAM_ID,
        attestors: &attestors_a_ab,
    };
    let conn_a_ac = ConnCtx {
        lc_program_id: TEST_ATTESTATION_ID,
        attestors: &attestors_a_ac,
    };
    let conn_b_ba = ConnCtx {
        lc_program_id: ATTESTATION_PROGRAM_ID,
        attestors: &attestors_b_ba,
    };
    let conn_b_bc = ConnCtx {
        lc_program_id: TEST_ATTESTATION_ID,
        attestors: &attestors_b_bc,
    };
    let conn_c_cb = ConnCtx {
        lc_program_id: ATTESTATION_PROGRAM_ID,
        attestors: &attestors_c_cb,
    };
    let conn_c_ca = ConnCtx {
        lc_program_id: TEST_ATTESTATION_ID,
        attestors: &attestors_c_ca,
    };

    // ── Counter state PDAs ──
    let cs_a = chain_a.counter_app_state_pda();
    let cs_b = chain_b.counter_app_state_pda();
    let cs_c = chain_c.counter_app_state_pda();

    // ── User-counter PDAs ──
    let ctr_b_from_a = gmp::derive_user_counter_pda(&gmp_b_from_a);
    let ctr_b_from_c = gmp::derive_user_counter_pda(&gmp_b_from_c);
    let ctr_c_from_b = gmp::derive_user_counter_pda(&gmp_c_from_b);
    let ctr_c_from_a = gmp::derive_user_counter_pda(&gmp_c_from_a);
    let ctr_a_from_b = gmp::derive_user_counter_pda(&gmp_a_from_b);
    let ctr_a_from_c = gmp::derive_user_counter_pda(&gmp_a_from_c);

    // ── Leg 1: A → B ──
    let l1 = run_gmp_leg(
        &user,
        &relayer,
        &mut chain_a,
        "a-to-b",
        &conn_a_ab,
        &mut chain_b,
        "b-to-a",
        &conn_b_ba,
        gmp_b_from_a,
        cs_b,
        ctr_b_from_a,
        10,
        1,
        None,
        None,
    )
    .await;

    assert_eq!(read_user_counter(&chain_b, ctr_b_from_a).await.count, 10);
    assert_gmp_result_exists(&chain_a, "a-to-b", 1).await;

    // ── Leg 2: B → C ──
    let l2 = run_gmp_leg(
        &user,
        &relayer,
        &mut chain_b,
        "b-to-c",
        &conn_b_bc,
        &mut chain_c,
        "c-to-b",
        &conn_c_cb,
        gmp_c_from_b,
        cs_c,
        ctr_c_from_b,
        20,
        1,
        None,
        None,
    )
    .await;

    assert_eq!(read_user_counter(&chain_c, ctr_c_from_b).await.count, 20);
    assert_gmp_result_exists(&chain_b, "b-to-c", 1).await;

    // ── Leg 3: C → A ──
    let l3 = run_gmp_leg(
        &user,
        &relayer,
        &mut chain_c,
        "c-to-a",
        &conn_c_ca,
        &mut chain_a,
        "a-to-c",
        &conn_a_ac,
        gmp_a_from_c,
        cs_a,
        ctr_a_from_c,
        30,
        1,
        None,
        None,
    )
    .await;

    assert_eq!(read_user_counter(&chain_a, ctr_a_from_c).await.count, 30);
    assert_gmp_result_exists(&chain_c, "c-to-a", 1).await;

    // ── Leg 4: A → C  (reuses C("c-to-a",1) and A("a-to-c",1) slots) ──
    let l4 = run_gmp_leg(
        &user,
        &relayer,
        &mut chain_a,
        "a-to-c",
        &conn_a_ac,
        &mut chain_c,
        "c-to-a",
        &conn_c_ca,
        gmp_c_from_a,
        cs_c,
        ctr_c_from_a,
        40,
        1,
        Some(&l3.ack),
        Some(&l3.recv),
    )
    .await;
    let _ = l4;

    assert_eq!(read_user_counter(&chain_c, ctr_c_from_a).await.count, 40);
    assert_gmp_result_exists(&chain_a, "a-to-c", 1).await;

    // ── Leg 5: C → B  (reuses B("b-to-c",1) and C("c-to-b",1) slots) ──
    let l5 = run_gmp_leg(
        &user,
        &relayer,
        &mut chain_c,
        "c-to-b",
        &conn_c_cb,
        &mut chain_b,
        "b-to-c",
        &conn_b_bc,
        gmp_b_from_c,
        cs_b,
        ctr_b_from_c,
        50,
        1,
        Some(&l2.ack),
        Some(&l2.recv),
    )
    .await;
    let _ = l5;

    assert_eq!(read_user_counter(&chain_b, ctr_b_from_c).await.count, 50);
    assert_gmp_result_exists(&chain_c, "c-to-b", 1).await;

    // ── Leg 6: B → A  (reuses A("a-to-b",1) and B("b-to-a",1) slots) ──
    run_gmp_leg(
        &user,
        &relayer,
        &mut chain_b,
        "b-to-a",
        &conn_b_ba,
        &mut chain_a,
        "a-to-b",
        &conn_a_ab,
        gmp_a_from_b,
        cs_a,
        ctr_a_from_b,
        60,
        1,
        Some(&l1.ack),
        Some(&l1.recv),
    )
    .await;

    // ── Assertions ──

    // Chain A: received from C (30) and B (60)
    let counter = read_user_counter(&chain_a, ctr_a_from_c).await;
    assert_eq!(counter.count, 30);
    let counter = read_user_counter(&chain_a, ctr_a_from_b).await;
    assert_eq!(counter.count, 60);

    // Chain B: received from A (10) and C (50)
    let counter = read_user_counter(&chain_b, ctr_b_from_a).await;
    assert_eq!(counter.count, 10);
    let counter = read_user_counter(&chain_b, ctr_b_from_c).await;
    assert_eq!(counter.count, 50);

    // Chain C: received from B (20) and A (40)
    let counter = read_user_counter(&chain_c, ctr_c_from_b).await;
    assert_eq!(counter.count, 20);
    let counter = read_user_counter(&chain_c, ctr_c_from_a).await;
    assert_eq!(counter.count, 40);

    // All GMPCallResult accounts exist
    assert_gmp_result_exists(&chain_a, "a-to-b", 1).await;
    assert_gmp_result_exists(&chain_a, "a-to-c", 1).await;
    assert_gmp_result_exists(&chain_b, "b-to-c", 1).await;
    assert_gmp_result_exists(&chain_b, "b-to-a", 1).await;
    assert_gmp_result_exists(&chain_c, "c-to-a", 1).await;
    assert_gmp_result_exists(&chain_c, "c-to-b", 1).await;
}

// ── Helpers ──────────────────────────────────────────────────────────────

/// Connection context: LC program ID and attestor set for a directed link.
struct ConnCtx<'a> {
    lc_program_id: Pubkey,
    attestors: &'a Attestors,
}

struct ChunkPdas {
    payload: Pubkey,
    proof: Pubkey,
}

struct LegChunks {
    recv: ChunkPdas,
    ack: ChunkPdas,
}

/// Run a complete GMP leg: send → recv → ack with real attestation proofs.
///
/// `source_conn` / `dest_conn` provide the LC program ID and attestors for
/// each side. `stale_on_dest` / `stale_on_source` are chunk PDAs from a
/// previous leg that occupy the same (client, sequence) slot and must be
/// cleaned first.
#[allow(clippy::too_many_arguments)]
async fn run_gmp_leg(
    user: &User,
    relayer: &Relayer,
    source: &mut Chain,
    source_client: &str,
    source_conn: &ConnCtx<'_>,
    dest: &mut Chain,
    dest_client: &str,
    dest_conn: &ConnCtx<'_>,
    gmp_pda: Pubkey,
    counter_state: Pubkey,
    counter_pda: Pubkey,
    amount: u64,
    sequence: u64,
    stale_on_dest: Option<&ChunkPdas>,
    stale_on_source: Option<&ChunkPdas>,
) -> LegChunks {
    let payload = gmp::encode_increment_payload(counter_state, counter_pda, gmp_pda, amount);
    let packet = gmp::encode_gmp_packet(&user.pubkey(), &test_gmp_app::ID, &payload);

    // ── Send on source chain ──
    let source_lc = att_lc_accounts(source_conn.lc_program_id, PROOF_HEIGHT);
    let (send_ix, commitment_pda) = gmp::build_gmp_send_call_ix(
        user.pubkey(),
        user.pubkey(),
        source_client,
        &source_lc,
        GmpSendCallParams {
            sequence,
            timeout_timestamp: GMP_TIMEOUT,
            receiver: &test_gmp_app::ID.to_string(),
            payload: payload.encode_to_vec(),
        },
    );
    let tx = Transaction::new_signed_with_payer(
        &[send_ix],
        Some(&user.pubkey()),
        &[user.keypair()],
        source.blockhash(),
    );
    source
        .process_transaction(tx)
        .await
        .expect("send_call failed");

    // ── Build recv proof (signed by dest attestors) ──
    let packet_commitment = read_commitment(source, commitment_pda).await;
    let recv_entry = attestation::packet_commitment_entry(
        source_client, // counterparty from dest's perspective
        sequence,
        packet_commitment,
    );
    let recv_proof = attestation::build_packet_membership_proof(
        dest_conn.attestors,
        PROOF_HEIGHT,
        &[recv_entry],
    );
    let recv_proof_bytes = attestation::serialize_proof(&recv_proof);

    // ── Recv on dest chain ──
    if let Some(stale) = stale_on_dest {
        relayer
            .cleanup_chunks_for_client(dest, dest_client, sequence, stale.payload, stale.proof)
            .await
            .expect("cleanup stale dest chunks failed");
    }

    let (recv_pl, recv_pr) = relayer
        .upload_chunks_for_client(dest, dest_client, sequence, &packet, &recv_proof_bytes)
        .await
        .expect("upload recv chunks failed");

    let remaining = gmp::build_increment_remaining_accounts(gmp_pda, counter_state, counter_pda);
    let dest_lc = att_lc_accounts(dest_conn.lc_program_id, PROOF_HEIGHT);
    let recv_result = gmp::build_gmp_recv_packet_ix(
        relayer.pubkey(),
        dest_client,
        source_client,
        dest.clock_time(),
        &dest_lc,
        GmpRecvPacketParams {
            sequence,
            payload_chunk_pda: recv_pl,
            proof_chunk_pda: recv_pr,
            remaining_accounts: remaining,
        },
    );
    let budget_ix = ComputeBudgetInstruction::set_compute_unit_limit(300_000);
    let tx = Transaction::new_signed_with_payer(
        &[budget_ix, recv_result.ix],
        Some(&relayer.pubkey()),
        &[relayer.keypair()],
        dest.blockhash(),
    );
    dest.process_transaction(tx)
        .await
        .expect("recv_packet failed");

    // ── Build ack proof (signed by source attestors) ──
    let ack_commitment = extract_ack_data(dest, recv_result.ack_pda).await;
    let ack_entry = attestation::ack_commitment_entry(
        dest_client, // counterparty from source's perspective
        sequence,
        ack_commitment
            .as_slice()
            .try_into()
            .expect("ack should be 32 bytes"),
    );
    let ack_proof = attestation::build_packet_membership_proof(
        source_conn.attestors,
        PROOF_HEIGHT,
        &[ack_entry],
    );
    let ack_proof_bytes = attestation::serialize_proof(&ack_proof);

    // ── Ack on source chain ──
    if let Some(stale) = stale_on_source {
        relayer
            .cleanup_chunks_for_client(source, source_client, sequence, stale.payload, stale.proof)
            .await
            .expect("cleanup stale source chunks failed");
    }

    let (ack_pl, ack_pr) = relayer
        .upload_chunks_for_client(source, source_client, sequence, &packet, &ack_proof_bytes)
        .await
        .expect("upload ack chunks failed");

    let raw_ack =
        ics27_gmp::encoding::encode_gmp_ack(&amount.to_le_bytes(), gmp::ICS27_ENCODING_PROTOBUF)
            .expect("encode GMP ack");

    let (ack_ix, _) = gmp::build_gmp_ack_packet_ix(
        relayer.pubkey(),
        source_client,
        dest_client,
        source.clock_time(),
        &source_lc,
        GmpAckPacketParams {
            sequence,
            acknowledgement: raw_ack,
            payload_chunk_pda: ack_pl,
            proof_chunk_pda: ack_pr,
        },
    );
    let tx = Transaction::new_signed_with_payer(
        &[ack_ix],
        Some(&relayer.pubkey()),
        &[relayer.keypair()],
        source.blockhash(),
    );
    source
        .process_transaction(tx)
        .await
        .expect("ack_packet failed");

    LegChunks {
        recv: ChunkPdas {
            payload: recv_pl,
            proof: recv_pr,
        },
        ack: ChunkPdas {
            payload: ack_pl,
            proof: ack_pr,
        },
    }
}
