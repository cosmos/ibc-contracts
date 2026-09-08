// SPDX-License-Identifier: Apache-2.0

use super::*;
use integration_tests::chain::{attestation_lc_accounts, ChainConfig};
use integration_tests::gmp::{self, GmpAckPacketParams, GmpRecvPacketParams};
use integration_tests::programs::{AttestationLc, ATTESTATION_PROGRAM_ID, TEST_ATTESTATION_ID};
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use solana_sdk::transaction::Transaction;

/// Three-chain full-circle IFT roundtrip: A→B→C→A→C→B→A.
///
/// Six independent IFT transfer legs traverse every edge of the A-B-C triangle
/// in both directions. Each leg is a complete lifecycle:
/// `ift_transfer` (burn) → `gmp_recv_packet` (mint) → `gmp_ack_packet` →
/// `ift_finalize_transfer`.
///
/// Topology (mirrors `tests/gmp/three_chain.rs`):
///   Chain A: `"a-to-b"` ↔ `"b-to-a"`, `"a-to-c"` ↔ `"c-to-a"`
///   Chain B: `"b-to-a"` ↔ `"a-to-b"`, `"b-to-c"` ↔ `"c-to-b"`
///   Chain C: `"c-to-b"` ↔ `"b-to-c"`, `"c-to-a"` ↔ `"a-to-c"`
///
/// Each chain hosts two attestation LC instances (`ATTESTATION_PROGRAM_ID` and
/// `TEST_ATTESTATION_ID`) with independent attestor sets per directed
/// client connection. Each chain has its own SPL token mint with two bridges
/// linking it to the other two chains' mints.
#[tokio::test]
async fn test_ift_three_chain_roundtrip() {
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
    let ift_admin = IftAdmin::new();
    let relayer = Relayer::new();
    let user = User::new();
    let mint_keypair_a = Keypair::new();
    let mint_keypair_b = Keypair::new();
    let mint_keypair_c = Keypair::new();

    // ── Chains ──
    let att_lc_a_primary = AttestationLc::new(&attestors_a_ab);
    let att_lc_a_secondary =
        AttestationLc::with_program_id(&attestors_a_ac, TEST_ATTESTATION_ID, "test_attestation");
    let att_lc_b_primary = AttestationLc::new(&attestors_b_ba);
    let att_lc_b_secondary =
        AttestationLc::with_program_id(&attestors_b_bc, TEST_ATTESTATION_ID, "test_attestation");
    let att_lc_c_primary = AttestationLc::new(&attestors_c_cb);
    let att_lc_c_secondary =
        AttestationLc::with_program_id(&attestors_c_ca, TEST_ATTESTATION_ID, "test_attestation");

    let all_programs_a: &[&dyn ChainProgram] =
        &[&Ics27Gmp, &Ift, &att_lc_a_primary, &att_lc_a_secondary];
    let all_programs_b: &[&dyn ChainProgram] =
        &[&Ics27Gmp, &Ift, &att_lc_b_primary, &att_lc_b_secondary];
    let all_programs_c: &[&dyn ChainProgram] =
        &[&Ics27Gmp, &Ift, &att_lc_c_primary, &att_lc_c_secondary];

    let mut chain_a = Chain::new(ChainConfig {
        client_id: "a-to-b",
        counterparty_client_id: "b-to-a",
        deployer: &deployer,
        programs: all_programs_a,
    });
    chain_a.prefund(&[&admin, &ift_admin, &relayer, &user]);

    let mut chain_b = Chain::new(ChainConfig {
        client_id: "b-to-a",
        counterparty_client_id: "a-to-b",
        deployer: &deployer,
        programs: all_programs_b,
    });
    chain_b.prefund(&[&admin, &ift_admin, &relayer, &user]);

    let mut chain_c = Chain::new(ChainConfig {
        client_id: "c-to-b",
        counterparty_client_id: "b-to-c",
        deployer: &deployer,
        programs: all_programs_c,
    });
    chain_c.prefund(&[&admin, &ift_admin, &relayer, &user]);

    // GMP account PDAs (one per receiving-client × IFT program).
    let gmp_b_from_a = gmp::derive_gmp_account_pda("b-to-a", &::ift::ID);
    let gmp_b_from_c = gmp::derive_gmp_account_pda("b-to-c", &::ift::ID);
    let gmp_c_from_b = gmp::derive_gmp_account_pda("c-to-b", &::ift::ID);
    let gmp_c_from_a = gmp::derive_gmp_account_pda("c-to-a", &::ift::ID);
    let gmp_a_from_b = gmp::derive_gmp_account_pda("a-to-b", &::ift::ID);
    let gmp_a_from_c = gmp::derive_gmp_account_pda("a-to-c", &::ift::ID);

    chain_a.prefund_lamports(gmp_a_from_b, GMP_ACCOUNT_PREFUND_LAMPORTS);
    chain_a.prefund_lamports(gmp_a_from_c, GMP_ACCOUNT_PREFUND_LAMPORTS);
    chain_b.prefund_lamports(gmp_b_from_a, GMP_ACCOUNT_PREFUND_LAMPORTS);
    chain_b.prefund_lamports(gmp_b_from_c, GMP_ACCOUNT_PREFUND_LAMPORTS);
    chain_c.prefund_lamports(gmp_c_from_b, GMP_ACCOUNT_PREFUND_LAMPORTS);
    chain_c.prefund_lamports(gmp_c_from_a, GMP_ACCOUNT_PREFUND_LAMPORTS);

    // ── Init chains ──
    // Each chain: start → init IBC stack → init IFT → add secondary client →
    // transfer upgrade authority → update both LCs.
    init_ift_three_chain(
        &mut chain_a,
        &deployer,
        &admin,
        &ift_admin,
        &relayer,
        &att_lc_a_primary,
        &att_lc_a_secondary,
        &attestors_a_ab,
        &attestors_a_ac,
        "a-to-c",
        "c-to-a",
    )
    .await;
    init_ift_three_chain(
        &mut chain_b,
        &deployer,
        &admin,
        &ift_admin,
        &relayer,
        &att_lc_b_primary,
        &att_lc_b_secondary,
        &attestors_b_ba,
        &attestors_b_bc,
        "b-to-c",
        "c-to-b",
    )
    .await;
    init_ift_three_chain(
        &mut chain_c,
        &deployer,
        &admin,
        &ift_admin,
        &relayer,
        &att_lc_c_primary,
        &att_lc_c_secondary,
        &attestors_c_cb,
        &attestors_c_ca,
        "c-to-a",
        "a-to-c",
    )
    .await;

    // ── Setup tokens ──
    let mint_a = mint_keypair_a.pubkey();
    let mint_b = mint_keypair_b.pubkey();
    let mint_c = mint_keypair_c.pubkey();

    create_spl_token(&mut chain_a, &ift_admin, &mint_keypair_a).await;
    create_spl_token(&mut chain_b, &ift_admin, &mint_keypair_b).await;
    create_spl_token(&mut chain_c, &ift_admin, &mint_keypair_c).await;

    // Register 2 bridges per chain (6 total), linking each local mint to both
    // counterparty mints.
    // Chain A: mint_a via "a-to-b" ↔ mint_b, mint_a via "a-to-c" ↔ mint_c
    register_solana_bridge(
        &mut chain_a,
        &ift_admin,
        mint_a,
        "a-to-b".into(),
        mint_b,
        "b-to-a".into(),
    )
    .await;
    register_solana_bridge(
        &mut chain_a,
        &ift_admin,
        mint_a,
        "a-to-c".into(),
        mint_c,
        "c-to-a".into(),
    )
    .await;
    // Chain B: mint_b via "b-to-a" ↔ mint_a, mint_b via "b-to-c" ↔ mint_c
    register_solana_bridge(
        &mut chain_b,
        &ift_admin,
        mint_b,
        "b-to-a".into(),
        mint_a,
        "a-to-b".into(),
    )
    .await;
    register_solana_bridge(
        &mut chain_b,
        &ift_admin,
        mint_b,
        "b-to-c".into(),
        mint_c,
        "c-to-b".into(),
    )
    .await;
    // Chain C: mint_c via "c-to-b" ↔ mint_b, mint_c via "c-to-a" ↔ mint_a
    register_solana_bridge(
        &mut chain_c,
        &ift_admin,
        mint_c,
        "c-to-b".into(),
        mint_b,
        "b-to-c".into(),
    )
    .await;
    register_solana_bridge(
        &mut chain_c,
        &ift_admin,
        mint_c,
        "c-to-a".into(),
        mint_a,
        "a-to-c".into(),
    )
    .await;

    // Mint initial balance on chain A only.
    admin_mint_to_user(&mut chain_a, &ift_admin, mint_a, user.pubkey()).await;

    let user_ata_a = TokenKind::Spl.get_ata(&user.pubkey(), &mint_a);
    let user_ata_b = TokenKind::Spl.get_ata(&user.pubkey(), &mint_b);
    let user_ata_c = TokenKind::Spl.get_ata(&user.pubkey(), &mint_c);

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

    // ── Leg 1: A → B ──
    let l1 = run_ift_leg(
        &user,
        &relayer,
        &mut chain_a,
        "a-to-b",
        &conn_a_ab,
        mint_a,
        &mut chain_b,
        "b-to-a",
        &conn_b_ba,
        mint_b,
        TRANSFER_AMOUNT,
        1,
        None,
        None,
    )
    .await;

    assert_eq!(
        TokenKind::Spl.read_balance(&chain_a, user_ata_a).await,
        INITIAL_BALANCE - TRANSFER_AMOUNT,
    );
    assert_eq!(
        TokenKind::Spl.read_balance(&chain_b, user_ata_b).await,
        TRANSFER_AMOUNT,
    );

    // ── Leg 2: B → C ──
    let l2 = run_ift_leg(
        &user,
        &relayer,
        &mut chain_b,
        "b-to-c",
        &conn_b_bc,
        mint_b,
        &mut chain_c,
        "c-to-b",
        &conn_c_cb,
        mint_c,
        TRANSFER_AMOUNT,
        1,
        None,
        None,
    )
    .await;

    assert_eq!(TokenKind::Spl.read_balance(&chain_b, user_ata_b).await, 0);
    assert_eq!(
        TokenKind::Spl.read_balance(&chain_c, user_ata_c).await,
        TRANSFER_AMOUNT,
    );

    // ── Leg 3: C → A ──
    let l3 = run_ift_leg(
        &user,
        &relayer,
        &mut chain_c,
        "c-to-a",
        &conn_c_ca,
        mint_c,
        &mut chain_a,
        "a-to-c",
        &conn_a_ac,
        mint_a,
        TRANSFER_AMOUNT,
        1,
        None,
        None,
    )
    .await;

    assert_eq!(TokenKind::Spl.read_balance(&chain_c, user_ata_c).await, 0);
    assert_eq!(
        TokenKind::Spl.read_balance(&chain_a, user_ata_a).await,
        INITIAL_BALANCE,
    );

    // ── Leg 4: A → C (reuses (c-to-a,1) on C and (a-to-c,1) on A) ──
    let l4 = run_ift_leg(
        &user,
        &relayer,
        &mut chain_a,
        "a-to-c",
        &conn_a_ac,
        mint_a,
        &mut chain_c,
        "c-to-a",
        &conn_c_ca,
        mint_c,
        TRANSFER_AMOUNT,
        1,
        Some(&l3.ack),
        Some(&l3.recv),
    )
    .await;
    let _ = l4;

    assert_eq!(
        TokenKind::Spl.read_balance(&chain_a, user_ata_a).await,
        INITIAL_BALANCE - TRANSFER_AMOUNT,
    );
    assert_eq!(
        TokenKind::Spl.read_balance(&chain_c, user_ata_c).await,
        TRANSFER_AMOUNT,
    );

    // ── Leg 5: C → B (reuses (b-to-c,1) on B and (c-to-b,1) on C) ──
    let l5 = run_ift_leg(
        &user,
        &relayer,
        &mut chain_c,
        "c-to-b",
        &conn_c_cb,
        mint_c,
        &mut chain_b,
        "b-to-c",
        &conn_b_bc,
        mint_b,
        TRANSFER_AMOUNT,
        1,
        Some(&l2.ack),
        Some(&l2.recv),
    )
    .await;
    let _ = l5;

    assert_eq!(TokenKind::Spl.read_balance(&chain_c, user_ata_c).await, 0);
    assert_eq!(
        TokenKind::Spl.read_balance(&chain_b, user_ata_b).await,
        TRANSFER_AMOUNT,
    );

    // ── Leg 6: B → A (reuses (a-to-b,1) on A and (b-to-a,1) on B) ──
    run_ift_leg(
        &user,
        &relayer,
        &mut chain_b,
        "b-to-a",
        &conn_b_ba,
        mint_b,
        &mut chain_a,
        "a-to-b",
        &conn_a_ab,
        mint_a,
        TRANSFER_AMOUNT,
        1,
        Some(&l1.ack),
        Some(&l1.recv),
    )
    .await;

    // ── Final assertions ──
    assert_eq!(
        TokenKind::Spl.read_balance(&chain_a, user_ata_a).await,
        INITIAL_BALANCE,
        "full circle: all tokens back on chain A",
    );
    assert_eq!(
        TokenKind::Spl.read_balance(&chain_b, user_ata_b).await,
        0,
        "chain B drained",
    );
    assert_eq!(
        TokenKind::Spl.read_balance(&chain_c, user_ata_c).await,
        0,
        "chain C drained",
    );

    // All GMPCallResult accounts exist.
    assert_gmp_result_exists(&chain_a, "a-to-b", 1).await;
    assert_gmp_result_exists(&chain_a, "a-to-c", 1).await;
    assert_gmp_result_exists(&chain_b, "b-to-a", 1).await;
    assert_gmp_result_exists(&chain_b, "b-to-c", 1).await;
    assert_gmp_result_exists(&chain_c, "c-to-b", 1).await;
    assert_gmp_result_exists(&chain_c, "c-to-a", 1).await;
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

struct LegResult {
    recv: ChunkPdas,
    ack: ChunkPdas,
}

/// Initialize a chain with two attestation LC instances for three-chain topology.
///
/// Handles IBC stack + IFT program init + secondary client registration +
/// upgrade authority transfer + `update_client` on both LCs.
#[allow(clippy::too_many_arguments)]
async fn init_ift_three_chain(
    chain: &mut Chain,
    deployer: &Deployer,
    admin: &Admin,
    ift_admin: &IftAdmin,
    relayer: &Relayer,
    primary_lc: &AttestationLc,
    secondary_lc: &AttestationLc,
    primary_attestors: &Attestors,
    secondary_attestors: &Attestors,
    secondary_client: &str,
    secondary_counterparty: &str,
) {
    let ibc_programs: &[&dyn ChainProgram] = &[&Ics27Gmp, primary_lc, secondary_lc];
    let all_programs: &[&dyn ChainProgram] = &[&Ics27Gmp, &Ift, primary_lc, secondary_lc];

    chain.start().await;
    deployer
        .init_ibc_stack(chain, admin, relayer, ibc_programs)
        .await;
    deployer
        .init_programs(chain, ift_admin.pubkey(), &[&Ift])
        .await;
    deployer
        .add_counterparty_with_attestation(
            chain,
            admin,
            secondary_client,
            secondary_counterparty,
            TEST_ATTESTATION_ID,
        )
        .await;
    deployer
        .transfer_upgrade_authority(chain, all_programs)
        .await;
    relayer
        .attestation_update_client(chain, primary_attestors, PROOF_HEIGHT)
        .await
        .expect("primary update_client failed");
    relayer
        .attestation_update_client_for_program(
            chain,
            secondary_attestors,
            PROOF_HEIGHT,
            TEST_ATTESTATION_ID,
        )
        .await
        .expect("secondary update_client failed");
}

/// Run a complete IFT transfer leg: burn on source → mint on dest → ack → finalize.
#[allow(clippy::too_many_arguments)]
async fn run_ift_leg(
    user: &User,
    relayer: &Relayer,
    source: &mut Chain,
    source_client: &str,
    source_conn: &ConnCtx<'_>,
    source_mint: Pubkey,
    dest: &mut Chain,
    dest_client: &str,
    dest_conn: &ConnCtx<'_>,
    dest_mint: Pubkey,
    amount: u64,
    sequence: u64,
    stale_on_dest: Option<&ChunkPdas>,
    stale_on_source: Option<&ChunkPdas>,
) -> LegResult {
    let source_lc = attestation_lc_accounts(source_conn.lc_program_id, PROOF_HEIGHT);
    let dest_lc = attestation_lc_accounts(dest_conn.lc_program_id, PROOF_HEIGHT);

    // ── Build payload ──
    let solana_payload = ift::encode_ift_solana_mint_payload(
        ::ift::ID,
        dest_mint,
        dest_client,
        ::ift::ID,
        user.pubkey(),
        amount,
    );
    let gmp_packet_bytes = ift::encode_ift_solana_gmp_packet(::ift::ID, ::ift::ID, &solana_payload);

    // ── IFT transfer (burn on source) ──
    let transfer_result = ift::build_ift_transfer_ix(
        user.pubkey(),
        user.pubkey(),
        source_client,
        source_mint,
        TokenKind::Spl,
        &source_lc,
        IftTransferParams {
            sequence,
            receiver: user.pubkey().to_string(),
            amount,
            timeout_timestamp: IFT_TIMEOUT,
        },
    );
    let tx = Transaction::new_signed_with_payer(
        &[transfer_result.ix],
        Some(&user.pubkey()),
        &[user.keypair()],
        source.blockhash(),
    );
    source
        .process_transaction(tx)
        .await
        .expect("ift_transfer failed");
    assert_commitment_set(source, transfer_result.commitment_pda).await;

    // ── Build recv proof (dest attestors prove source commitment) ──
    let recv_proof_bytes = attestation::build_recv_proof_bytes(
        source,
        transfer_result.commitment_pda,
        source_client,
        sequence,
        dest_conn.attestors,
    )
    .await;

    // ── Deliver recv_packet to dest (mints tokens) ──
    if let Some(stale) = stale_on_dest {
        relayer
            .cleanup_chunks_for_client(dest, dest_client, sequence, stale.payload, stale.proof)
            .await
            .expect("cleanup stale dest chunks failed");
    }

    let (recv_pl, recv_pr) = relayer
        .upload_chunks_for_client(
            dest,
            dest_client,
            sequence,
            &gmp_packet_bytes,
            &recv_proof_bytes,
        )
        .await
        .expect("upload recv chunks failed");

    let gmp_account_pda = gmp::derive_gmp_account_pda(dest_client, &::ift::ID);
    let remaining =
        ift::build_ift_solana_remaining_accounts(gmp_account_pda, ::ift::ID, &solana_payload);

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
        .expect("gmp_recv_packet failed");

    // ── Build ack proof (source attestors prove dest ack) ──
    let raw_ack = ics27_gmp::encoding::encode_gmp_ack(&[], gmp::ICS27_ENCODING_PROTOBUF)
        .expect("encode GMP ack");

    let ack_proof_bytes = attestation::build_ack_proof_bytes(
        dest,
        recv_result.ack_pda,
        dest_client,
        sequence,
        source_conn.attestors,
    )
    .await;

    // ── Deliver ack to source ──
    if let Some(stale) = stale_on_source {
        relayer
            .cleanup_chunks_for_client(source, source_client, sequence, stale.payload, stale.proof)
            .await
            .expect("cleanup stale source chunks failed");
    }

    let (ack_pl, ack_pr) = relayer
        .upload_chunks_for_client(
            source,
            source_client,
            sequence,
            &gmp_packet_bytes,
            &ack_proof_bytes,
        )
        .await
        .expect("upload ack chunks failed");

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
        .expect("gmp_ack_packet failed");

    // ── Finalize transfer on source (closes PendingTransfer) ──
    let finalize_ix = ift::build_finalize_transfer_ix(
        relayer.pubkey(),
        source_mint,
        user.pubkey(),
        source_client,
        sequence,
        TokenKind::Spl,
    );
    let tx = Transaction::new_signed_with_payer(
        &[finalize_ix],
        Some(&relayer.pubkey()),
        &[relayer.keypair()],
        source.blockhash(),
    );
    source
        .process_transaction(tx)
        .await
        .expect("ift_finalize_transfer failed");

    ift::assert_pending_transfer_closed(source, transfer_result.pending_transfer_pda).await;

    LegResult {
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

async fn assert_gmp_result_exists(chain: &Chain, client_id: &str, sequence: u64) {
    let (pda, _) = solana_ibc_gmp_types::GMPCallResult::pda(client_id, sequence, &ics27_gmp::ID);
    let account = chain
        .get_account(pda)
        .await
        .expect("GMPCallResult should exist");
    assert_eq!(account.owner, ics27_gmp::ID);
}
