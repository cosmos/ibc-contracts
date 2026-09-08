// SPDX-License-Identifier: Apache-2.0

use super::*;
use ics26_router::ics24;

/// IFT error ack refund: `ift_transfer` -> error ack -> `finalize_transfer`.
///
/// The user transfers tokens via IFT. The counterparty returns a universal
/// error acknowledgement. `finalize_transfer` detects the error ack and
/// refunds the burned tokens back to the user.
#[tokio::test]
async fn test_ift_error_ack_refund() {
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

    // ── Relayer uploads chunks and delivers error ack ──
    let ack_data = ift::universal_error_ack();
    let ack_commitment =
        ics24::packet_acknowledgement_commitment_bytes32(std::slice::from_ref(&ack_data))
            .expect("compute ack commitment");
    let ack_entry = attestation::ack_commitment_entry(
        chain.counterparty_client_id(),
        sequence,
        ack_commitment
            .as_slice()
            .try_into()
            .expect("ack should be 32 bytes"),
    );
    let ack_proof =
        attestation::build_packet_membership_proof(&attestors, PROOF_HEIGHT, &[ack_entry]);
    let ack_proof_bytes = attestation::serialize_proof(&ack_proof);

    let (ack_payload_pda, ack_proof_pda) = relayer
        .upload_chunks(&mut chain, sequence, &gmp_packet_bytes, &ack_proof_bytes)
        .await
        .expect("upload ack chunks failed");

    let ack_commitment_pda = relayer
        .ift_gmp_ack_packet(
            &mut chain,
            IftGmpAckPacketParams {
                sequence,
                acknowledgement: ift::universal_error_ack(),
                payload_chunk_pda: ack_payload_pda,
                proof_chunk_pda: ack_proof_pda,
            },
        )
        .await
        .expect("ift_gmp_ack_packet with error ack failed");

    assert_commitment_zeroed(&chain, ack_commitment_pda).await;

    // ── Relayer finalizes transfer (refund for error ack) ──
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
