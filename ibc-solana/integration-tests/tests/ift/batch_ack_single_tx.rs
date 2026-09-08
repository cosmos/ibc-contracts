// SPDX-License-Identifier: Apache-2.0

use super::*;
use ics26_router::ics24;

/// Two consecutive IFT transfers acked in a *single* router transaction.
///
/// Mirrors `batch_transfers` for the burn + commitment phase, then collapses
/// the per-sequence ack steps into one `relayer.ack_packets_batched(...)`
/// call. This exercises the relayer's batched submission path that the e2e
/// suite uses but is otherwise untested at the integration level. Finalize
/// is still per-sequence because `ift_finalize_transfer` is stateful per
/// `PendingTransfer` and is not batched at the program level.
#[tokio::test]
async fn test_ift_batch_ack_single_tx() {
    // ── Attestors ──
    let attestors = Attestors::new(2);

    // ── Actors ──
    let deployer = Deployer::new();
    let admin = Admin::new();
    let ift_admin = IftAdmin::new();
    let relayer = Relayer::new();
    let user = User::new();
    let mint_keypair = Keypair::new();

    // ── Chain ──
    let attestation_lc = AttestationLc::new(&attestors);
    let all_programs: &[&dyn ChainProgram] = &[&Ics27Gmp, &Ift, &attestation_lc];
    let mut chain = Chain::single(&deployer, all_programs);
    chain.prefund(&[&admin, &relayer, &user, &ift_admin]);

    // ── Init ──
    init_ift_chain(
        &mut chain,
        &deployer,
        &admin,
        &ift_admin,
        &relayer,
        &attestors,
        &attestation_lc,
    )
    .await;

    // ── Setup ──
    let (mint, user_ata) =
        setup_ift_chain(&mut chain, &ift_admin, &mint_keypair, user.pubkey()).await;
    assert_eq!(
        TokenKind::Spl.read_balance(&chain, user_ata).await,
        INITIAL_BALANCE,
    );

    // ── Two sequential burns (sequences 1 and 2) ──
    let result_1 = user
        .ift_transfer(
            &mut chain,
            mint,
            TokenKind::Spl,
            IftTransferParams {
                sequence: 1,
                receiver: ift::EVM_RECEIVER.to_string(),
                amount: TRANSFER_AMOUNT,
                timeout_timestamp: IFT_TIMEOUT,
            },
        )
        .await
        .expect("ift_transfer #1 failed");
    assert_commitment_set(&chain, result_1.commitment_pda).await;

    let result_2 = user
        .ift_transfer(
            &mut chain,
            mint,
            TokenKind::Spl,
            IftTransferParams {
                sequence: 2,
                receiver: ift::EVM_RECEIVER.to_string(),
                amount: TRANSFER_AMOUNT,
                timeout_timestamp: IFT_TIMEOUT,
            },
        )
        .await
        .expect("ift_transfer #2 failed");
    assert_commitment_set(&chain, result_2.commitment_pda).await;

    assert_eq!(
        TokenKind::Spl.read_balance(&chain, user_ata).await,
        INITIAL_BALANCE - 2 * TRANSFER_AMOUNT,
    );

    // ── Build ack proofs ──
    let ack_data = ift::success_ack();
    let ack_commitment =
        ics24::packet_acknowledgement_commitment_bytes32(std::slice::from_ref(&ack_data))
            .expect("compute ack commitment");

    let ack_entry_1 = attestation::ack_commitment_entry(
        chain.counterparty_client_id(),
        1,
        ack_commitment
            .as_slice()
            .try_into()
            .expect("ack should be 32 bytes"),
    );
    let ack_proof_1 =
        attestation::build_packet_membership_proof(&attestors, PROOF_HEIGHT, &[ack_entry_1]);
    let ack_proof_bytes_1 = attestation::serialize_proof(&ack_proof_1);

    let ack_entry_2 = attestation::ack_commitment_entry(
        chain.counterparty_client_id(),
        2,
        ack_commitment
            .as_slice()
            .try_into()
            .expect("ack should be 32 bytes"),
    );
    let ack_proof_2 =
        attestation::build_packet_membership_proof(&attestors, PROOF_HEIGHT, &[ack_entry_2]);
    let ack_proof_bytes_2 = attestation::serialize_proof(&ack_proof_2);

    // ── Upload chunks for both sequences (each in its own tx, as today) ──
    let mint_call_1 = ift::encode_evm_mint_call(ift::EVM_RECEIVER, TRANSFER_AMOUNT);
    let gmp_packet_1 = ift::encode_ift_gmp_packet(ift::COUNTERPARTY_IFT_ADDRESS, mint_call_1);
    let (payload_pda_1, proof_pda_1) = relayer
        .upload_chunks(&mut chain, 1, &gmp_packet_1, &ack_proof_bytes_1)
        .await
        .expect("upload chunks #1");

    let mint_call_2 = ift::encode_evm_mint_call(ift::EVM_RECEIVER, TRANSFER_AMOUNT);
    let gmp_packet_2 = ift::encode_ift_gmp_packet(ift::COUNTERPARTY_IFT_ADDRESS, mint_call_2);
    let (payload_pda_2, proof_pda_2) = relayer
        .upload_chunks(&mut chain, 2, &gmp_packet_2, &ack_proof_bytes_2)
        .await
        .expect("upload chunks #2");

    // ── Single tx: ack both sequences at once ──
    let client_id = chain.client_id().to_string();
    let ack_params_1 = ift::build_ift_gmp_ack_packet_params(
        &client_id,
        IftGmpAckPacketParams {
            sequence: 1,
            acknowledgement: ift::success_ack(),
            payload_chunk_pda: payload_pda_1,
            proof_chunk_pda: proof_pda_1,
        },
    );
    let ack_params_2 = ift::build_ift_gmp_ack_packet_params(
        &client_id,
        IftGmpAckPacketParams {
            sequence: 2,
            acknowledgement: ift::success_ack(),
            payload_chunk_pda: payload_pda_2,
            proof_chunk_pda: proof_pda_2,
        },
    );

    let commitment_pdas = relayer
        .ack_packets_batched(&mut chain, vec![ack_params_1, ack_params_2])
        .await
        .expect("batched ack tx failed");
    assert_eq!(commitment_pdas.len(), 2);
    assert_eq!(commitment_pdas[0], result_1.commitment_pda);
    assert_eq!(commitment_pdas[1], result_2.commitment_pda);

    // Both per-sequence commitments must be cleared by the single batched tx.
    assert_commitment_zeroed(&chain, result_1.commitment_pda).await;
    assert_commitment_zeroed(&chain, result_2.commitment_pda).await;

    // ── Finalize per-sequence (cannot be batched) ──
    relayer
        .ift_finalize_transfer(
            &mut chain,
            mint,
            user.pubkey(),
            &client_id,
            1,
            TokenKind::Spl,
        )
        .await
        .expect("finalize #1");
    ift::assert_pending_transfer_closed(&chain, result_1.pending_transfer_pda).await;

    relayer
        .ift_finalize_transfer(
            &mut chain,
            mint,
            user.pubkey(),
            &client_id,
            2,
            TokenKind::Spl,
        )
        .await
        .expect("finalize #2");
    ift::assert_pending_transfer_closed(&chain, result_2.pending_transfer_pda).await;

    // Final balance: both transfers burned, no refunds.
    assert_eq!(
        TokenKind::Spl.read_balance(&chain, user_ata).await,
        INITIAL_BALANCE - 2 * TRANSFER_AMOUNT,
    );
}
