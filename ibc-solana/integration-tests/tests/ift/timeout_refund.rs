// SPDX-License-Identifier: Apache-2.0

use super::*;

/// IFT timeout refund: `ift_transfer` -> timeout -> `finalize_transfer`.
///
/// The user transfers tokens via IFT. The packet times out and the relayer
/// delivers a timeout. `finalize_transfer` refunds the burned tokens back
/// to the user.
#[tokio::test]
async fn test_ift_timeout_refund() {
    // ── Attestors ──
    let attestors = Attestors::new(2);

    // ── Actors ──
    let deployer = Deployer::new();
    let admin = Admin::new();
    let ift_admin = IftAdmin::new();
    let relayer = Relayer::new();
    let user = User::new();
    let mint_keypair = Keypair::new();

    // ── Test data ──
    let sequence = 1u64;

    // ── Chain ──
    let attestation_lc = AttestationLc::new(&attestors);
    let ibc_programs: &[&dyn ChainProgram] = &[&Ics27Gmp, &attestation_lc];
    let all_programs: &[&dyn ChainProgram] = &[&Ics27Gmp, &Ift, &attestation_lc];
    let mut chain = Chain::single(&deployer, all_programs);
    chain.prefund(&[&admin, &relayer, &user, &ift_admin]);

    // ── Init (manual update_client with timeout-compatible timestamp) ──
    chain.start().await;
    deployer
        .init_ibc_stack(&mut chain, &admin, &relayer, ibc_programs)
        .await;
    deployer
        .init_programs(&mut chain, ift_admin.pubkey(), &[&Ift])
        .await;
    deployer
        .transfer_upgrade_authority(&mut chain, all_programs)
        .await;
    let timeout_consensus_proof =
        attestation::build_state_membership_proof(&attestors, PROOF_HEIGHT, IFT_TIMEOUT);
    let update_ix = attestation::build_update_client_ix(
        relayer.pubkey(),
        PROOF_HEIGHT,
        timeout_consensus_proof,
    );
    relayer
        .send_tx(&mut chain, &[update_ix])
        .await
        .expect("update_client for timeout consensus failed");

    // ── Setup ──
    let (mint, user_ata) =
        setup_ift_chain(&mut chain, &ift_admin, &mint_keypair, user.pubkey()).await;
    let mint_call_payload = ift::encode_evm_mint_call(ift::EVM_RECEIVER, TRANSFER_AMOUNT);
    let gmp_packet_bytes =
        ift::encode_ift_gmp_packet(ift::COUNTERPARTY_IFT_ADDRESS, mint_call_payload);

    // ── User sends IFT transfer ──
    let result = user
        .ift_transfer(
            &mut chain,
            mint,
            TokenKind::Spl,
            IftTransferParams {
                sequence,
                receiver: ift::EVM_RECEIVER.to_string(),
                amount: TRANSFER_AMOUNT,
                timeout_timestamp: IFT_TIMEOUT,
            },
        )
        .await
        .expect("ift_transfer failed");

    assert_commitment_set(&chain, result.commitment_pda).await;

    // Tokens burned
    let balance = TokenKind::Spl.read_balance(&chain, user_ata).await;
    assert_eq!(balance, INITIAL_BALANCE - TRANSFER_AMOUNT);

    // ── Build timeout proof (receipt non-membership) ──
    let timeout_entry =
        attestation::receipt_commitment_entry(chain.counterparty_client_id(), sequence, [0u8; 32]);
    let timeout_proof =
        attestation::build_packet_membership_proof(&attestors, PROOF_HEIGHT, &[timeout_entry]);
    let timeout_proof_bytes = attestation::serialize_proof(&timeout_proof);

    // ── Relayer uploads chunks and delivers timeout ──
    let (timeout_payload_pda, timeout_proof_pda) = relayer
        .upload_chunks(
            &mut chain,
            sequence,
            &gmp_packet_bytes,
            &timeout_proof_bytes,
        )
        .await
        .expect("upload timeout chunks failed");

    let timeout_commitment_pda = relayer
        .ift_gmp_timeout_packet(
            &mut chain,
            IftGmpTimeoutPacketParams {
                sequence,
                payload_chunk_pda: timeout_payload_pda,
                proof_chunk_pda: timeout_proof_pda,
            },
        )
        .await
        .expect("ift_gmp_timeout_packet failed");

    assert_commitment_zeroed(&chain, timeout_commitment_pda).await;

    // ── Relayer finalizes transfer (refund for timeout) ──
    let client_id = chain.client_id().to_string();
    relayer
        .ift_finalize_transfer(
            &mut chain,
            mint,
            user.pubkey(),
            &client_id,
            sequence,
            TokenKind::Spl,
        )
        .await
        .expect("finalize_transfer failed");

    // PendingTransfer closed
    ift::assert_pending_transfer_closed(&chain, result.pending_transfer_pda).await;

    // Tokens refunded back to user
    let final_balance = TokenKind::Spl.read_balance(&chain, user_ata).await;
    assert_eq!(final_balance, INITIAL_BALANCE);
}
