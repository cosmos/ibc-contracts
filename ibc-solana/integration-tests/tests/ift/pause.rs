// SPDX-License-Identifier: Apache-2.0

use super::*;
use integration_tests::extract_custom_error;

/// Pause blocks transfers and admin-mint; unpause restores them.
///
/// After the admin pauses the IFT app, both `ift_transfer` and `admin_mint`
/// should fail with `AppPaused`. Unpausing should restore normal operation.
#[tokio::test]
async fn test_ift_pause() {
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

    // ── Pause the app ──
    ift_admin.set_paused(&mut chain, true).await.expect("pause");

    let state = ift::read_app_state(&chain).await;
    assert!(state.paused);

    // ── Transfer should fail while paused ──
    let err = user
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
        .expect_err("transfer should fail when paused");

    let app_paused_code =
        integration_tests::anchor_error_code(::ift::errors::IFTError::AppPaused as u32);
    assert_eq!(extract_custom_error(&err), app_paused_code);

    // ── Admin mint should fail while paused ──
    let err = ift_admin
        .admin_mint(&mut chain, mint, user.pubkey(), 100, TokenKind::Spl)
        .await
        .expect_err("admin_mint should fail when paused");
    assert_eq!(extract_custom_error(&err), app_paused_code);

    // Balance unchanged
    let balance = TokenKind::Spl.read_balance(&chain, user_ata).await;
    assert_eq!(balance, INITIAL_BALANCE);

    // ── Unpause ──
    ift_admin
        .set_paused(&mut chain, false)
        .await
        .expect("unpause");

    let state = ift::read_app_state(&chain).await;
    assert!(!state.paused);

    // ── Transfer should succeed after unpause ──
    user.ift_transfer(
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
    .expect("transfer should succeed after unpause");

    let balance = TokenKind::Spl.read_balance(&chain, user_ata).await;
    assert_eq!(balance, INITIAL_BALANCE - TRANSFER_AMOUNT);
}
