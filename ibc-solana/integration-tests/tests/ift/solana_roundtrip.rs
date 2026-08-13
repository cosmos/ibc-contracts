// SPDX-License-Identifier: Apache-2.0

use super::*;

/// Solana-to-Solana IFT roundtrip.
///
/// Two independent `ProgramTest` chains each run the IFT + GMP stack. The
/// source chain A burns the user's tokens and dispatches a `RawGmpSolanaPayload`
/// whose target is chain B's `ift_mint`. The relayer uploads chunks and
/// invokes `gmp_recv_packet` on B; the GMP program signs the CPI into IFT's
/// `ift_mint` which creates the receiver ATA and mints `TRANSFER_AMOUNT`
/// tokens. A success ack is delivered back to A, `ift_finalize_transfer`
/// closes the `PendingTransfer`, and A's token balance stays at
/// `INITIAL_BALANCE - TRANSFER_AMOUNT` (tokens remain burned on success).
#[tokio::test]
async fn test_ift_solana_roundtrip() {
    // ── Attestors ──
    let attestors_a = Attestors::new(2);
    let attestors_b = Attestors::new(3);

    // ── Actors ──
    let deployer = Deployer::new();
    let admin = Admin::new();
    let ift_admin = IftAdmin::new();
    let relayer = Relayer::new();
    let user = User::new();
    let mint_keypair_a = Keypair::new();
    let mint_keypair_b = Keypair::new();

    // ── Test data ──
    let sequence = 1u64;

    // ── Chains ──
    let attestation_lc_a = AttestationLc::new(&attestors_a);
    let attestation_lc_b = AttestationLc::new(&attestors_b);
    let all_programs_a: &[&dyn ChainProgram] = &[&Ics27Gmp, &Ift, &attestation_lc_a];
    let all_programs_b: &[&dyn ChainProgram] = &[&Ics27Gmp, &Ift, &attestation_lc_b];
    let (mut chain_a, mut chain_b) = Chain::pair(&deployer, all_programs_a, all_programs_b);
    chain_a.prefund(&[&admin, &ift_admin, &relayer, &user]);
    chain_b.prefund(&[&admin, &ift_admin, &relayer, &user]);

    // Prefund the destination-side GMP account PDA so it can pay ATA-init rent
    // for the receiver during `ift_mint`.
    let gmp_account_pda_on_b = gmp::derive_gmp_account_pda(chain_b.client_id(), &::ift::ID);
    chain_b.prefund_lamports(gmp_account_pda_on_b, GMP_ACCOUNT_PREFUND_LAMPORTS);

    // ── Init ──
    init_ift_chain(
        &mut chain_a,
        &deployer,
        &admin,
        &ift_admin,
        &relayer,
        &attestors_a,
        &attestation_lc_a,
    )
    .await;
    init_ift_chain(
        &mut chain_b,
        &deployer,
        &admin,
        &ift_admin,
        &relayer,
        &attestors_b,
        &attestation_lc_b,
    )
    .await;

    // ── Setup tokens ──
    let mint_a = mint_keypair_a.pubkey();
    let mint_b = mint_keypair_b.pubkey();

    create_spl_token(&mut chain_a, &ift_admin, &mint_keypair_a).await;
    create_spl_token(&mut chain_b, &ift_admin, &mint_keypair_b).await;

    let chain_a_client_id = chain_a.client_id().to_string();
    let chain_a_counterparty_client_id = chain_a.counterparty_client_id().to_string();
    let chain_b_client_id = chain_b.client_id().to_string();
    let chain_b_counterparty_client_id = chain_b.counterparty_client_id().to_string();

    register_solana_bridge(
        &mut chain_a,
        &ift_admin,
        mint_a,
        chain_a_client_id,
        mint_b,
        chain_a_counterparty_client_id,
    )
    .await;
    register_solana_bridge(
        &mut chain_b,
        &ift_admin,
        mint_b,
        chain_b_client_id,
        mint_a,
        chain_b_counterparty_client_id,
    )
    .await;

    admin_mint_to_user(&mut chain_a, &ift_admin, mint_a, user.pubkey()).await;
    let user_ata_a = TokenKind::Spl.get_ata(&user.pubkey(), &mint_a);
    assert_eq!(
        TokenKind::Spl.read_balance(&chain_a, user_ata_a).await,
        INITIAL_BALANCE
    );

    // ── Build the Solana-targeted mint payload + GMP packet bytes ──
    let solana_payload = ift::encode_ift_solana_mint_payload(
        ::ift::ID,
        mint_b,
        chain_b.client_id(),
        ::ift::ID,
        user.pubkey(),
        TRANSFER_AMOUNT,
    );
    let gmp_packet_bytes = ift::encode_ift_solana_gmp_packet(::ift::ID, ::ift::ID, &solana_payload);

    // ── User sends IFT transfer on chain A ──
    let result = user
        .ift_transfer(
            &mut chain_a,
            mint_a,
            TokenKind::Spl,
            IftTransferParams {
                sequence,
                receiver: user.pubkey().to_string(),
                amount: TRANSFER_AMOUNT,
                timeout_timestamp: IFT_TIMEOUT,
            },
        )
        .await
        .expect("ift_transfer on chain A failed");

    assert_commitment_set(&chain_a, result.commitment_pda).await;
    assert_eq!(
        TokenKind::Spl.read_balance(&chain_a, user_ata_a).await,
        INITIAL_BALANCE - TRANSFER_AMOUNT,
        "tokens must be burned on source after ift_transfer",
    );

    // ── Build recv proof (signed by B's attestors, proving A's commitment) ──
    let commitment_a = read_commitment(&chain_a, result.commitment_pda).await;
    let recv_entry = attestation::packet_commitment_entry(
        chain_b.counterparty_client_id(),
        sequence,
        commitment_a,
    );
    let recv_proof =
        attestation::build_packet_membership_proof(&attestors_b, PROOF_HEIGHT, &[recv_entry]);
    let recv_proof_bytes = attestation::serialize_proof(&recv_proof);

    // ── Relayer delivers recv_packet to chain B (mints tokens to user) ──
    let (b_recv_payload_chunk, b_recv_proof_chunk) = relayer
        .upload_chunks(&mut chain_b, sequence, &gmp_packet_bytes, &recv_proof_bytes)
        .await
        .expect("upload recv chunks on chain B failed");

    let remaining_accounts =
        ift::build_ift_solana_remaining_accounts(gmp_account_pda_on_b, ::ift::ID, &solana_payload);

    let recv = relayer
        .gmp_recv_packet(
            &mut chain_b,
            GmpRecvPacketParams {
                sequence,
                payload_chunk_pda: b_recv_payload_chunk,
                proof_chunk_pda: b_recv_proof_chunk,
                remaining_accounts,
            },
        )
        .await
        .expect("gmp_recv_packet on chain B failed");

    // ── Assert: receiver ATA was created and credited on chain B ──
    let user_ata_b = TokenKind::Spl.get_ata(&user.pubkey(), &mint_b);
    assert_eq!(
        TokenKind::Spl.read_balance(&chain_b, user_ata_b).await,
        TRANSFER_AMOUNT,
        "receiver should hold freshly-minted tokens on destination",
    );

    // ── Build ack proof (signed by A's attestors, proving B's ack) ──
    // `ift_mint` returns `Result<()>` (no return data), so GMP encodes an
    // empty-result success ack. Reconstruct the same bytes the on-chain GMP
    // program produced so the router's commitment check passes.
    let raw_ack = ics27_gmp::encoding::encode_gmp_ack(&[], gmp::ICS27_ENCODING_PROTOBUF)
        .expect("encode IFT GMP ack");
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

    // ── Relayer delivers success ack back to chain A ──
    let (a_ack_payload_chunk, a_ack_proof_chunk) = relayer
        .upload_chunks(&mut chain_a, sequence, &gmp_packet_bytes, &ack_proof_bytes)
        .await
        .expect("upload ack chunks on chain A failed");

    let ack_commitment_pda = relayer
        .gmp_ack_packet(
            &mut chain_a,
            GmpAckPacketParams {
                sequence,
                acknowledgement: raw_ack,
                payload_chunk_pda: a_ack_payload_chunk,
                proof_chunk_pda: a_ack_proof_chunk,
            },
        )
        .await
        .expect("gmp_ack_packet on chain A failed");

    assert_commitment_zeroed(&chain_a, ack_commitment_pda).await;

    // ── Relayer finalizes the transfer on chain A (no refund for success) ──
    let client_id = chain_a.client_id().to_string();
    relayer
        .ift_finalize_transfer(
            &mut chain_a,
            mint_a,
            user.pubkey(),
            &client_id,
            sequence,
            TokenKind::Spl,
        )
        .await
        .expect("ift_finalize_transfer on chain A failed");

    ift::assert_pending_transfer_closed(&chain_a, result.pending_transfer_pda).await;
    assert_eq!(
        TokenKind::Spl.read_balance(&chain_a, user_ata_a).await,
        INITIAL_BALANCE - TRANSFER_AMOUNT,
        "source balance must remain burned after successful finalize",
    );
    // Destination balance untouched by the ack/finalize round-trip.
    assert_eq!(
        TokenKind::Spl.read_balance(&chain_b, user_ata_b).await,
        TRANSFER_AMOUNT,
    );
}
