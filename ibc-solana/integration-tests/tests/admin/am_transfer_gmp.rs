// SPDX-License-Identifier: Apache-2.0

use super::*;

/// Full propose -> accept lifecycle for GMP AM transfer.
///
/// The chain authority proposes transferring the GMP app's AM from
/// `access_manager` to `test_access_manager`. Then, using the same
/// authority (which holds `ADMIN_ROLE` on both AMs), the transfer is
/// accepted. Verifies `GMPAppState.am_state` at each step.
#[tokio::test]
async fn test_gmp_am_transfer_propose_accept() {
    // ── Attestors ──
    let attestors = Attestors::new(2);

    // ── Actors ──
    let deployer = Deployer::new();
    let admin = Admin::new();
    let relayer = Relayer::new();

    // ── Chain ──
    let attestation_lc = AttestationLc::new(&attestors);
    let programs: &[&dyn ChainProgram] = &[&Ics27Gmp, &TestAccessManager, &attestation_lc];
    let mut chain = Chain::single(&deployer, programs);
    chain.prefund(&[&admin, &relayer]);

    // ── Init ──
    chain
        .init_with_attestation(&deployer, &admin, &relayer, programs, &attestors)
        .await;

    // ── Verify initial state ──
    let state = gmp::read_gmp_app_state(&chain).await;
    assert_eq!(state.am_state.access_manager, access_manager::ID);
    assert!(state.am_state.pending_access_manager.is_none());

    // Propose transfer to test_access_manager
    admin
        .gmp_propose_am_transfer(&mut chain, test_access_manager::ID)
        .await
        .expect("propose should succeed");

    let state = gmp::read_gmp_app_state(&chain).await;
    assert_eq!(
        state.am_state.pending_access_manager,
        Some(test_access_manager::ID)
    );
    assert_eq!(state.am_state.access_manager, access_manager::ID);

    // Accept transfer (authority holds ADMIN_ROLE on test_access_manager too)
    admin
        .gmp_accept_am_transfer(&mut chain, test_access_manager::ID)
        .await
        .expect("accept should succeed");

    let state = gmp::read_gmp_app_state(&chain).await;
    assert_eq!(state.am_state.access_manager, test_access_manager::ID);
    assert!(state.am_state.pending_access_manager.is_none());
}

/// Propose then cancel AM transfer on GMP.
#[tokio::test]
async fn test_gmp_am_transfer_propose_cancel() {
    // ── Attestors ──
    let attestors = Attestors::new(2);

    // ── Actors ──
    let deployer = Deployer::new();
    let admin = Admin::new();
    let relayer = Relayer::new();

    // ── Chain ──
    let attestation_lc = AttestationLc::new(&attestors);
    let programs: &[&dyn ChainProgram] = &[&Ics27Gmp, &TestAccessManager, &attestation_lc];
    let mut chain = Chain::single(&deployer, programs);
    chain.prefund(&[&admin, &relayer]);

    // ── Init ──
    chain
        .init_with_attestation(&deployer, &admin, &relayer, programs, &attestors)
        .await;

    // ── Propose then cancel ──
    admin
        .gmp_propose_am_transfer(&mut chain, test_access_manager::ID)
        .await
        .expect("propose should succeed");

    let state = gmp::read_gmp_app_state(&chain).await;
    assert_eq!(
        state.am_state.pending_access_manager,
        Some(test_access_manager::ID)
    );

    admin
        .gmp_cancel_am_transfer(&mut chain)
        .await
        .expect("cancel should succeed");

    let state = gmp::read_gmp_app_state(&chain).await;
    assert!(state.am_state.pending_access_manager.is_none());
    assert_eq!(state.am_state.access_manager, access_manager::ID);
}

/// Non-admin cannot propose AM transfer on GMP.
#[tokio::test]
async fn test_gmp_am_transfer_unauthorized_propose() {
    // ── Attestors ──
    let attestors = Attestors::new(2);

    // ── Actors ──
    let deployer = Deployer::new();
    let admin = Admin::new();
    let relayer = Relayer::new();
    let non_admin = Admin::new();

    // ── Chain ──
    let attestation_lc = AttestationLc::new(&attestors);
    let programs: &[&dyn ChainProgram] = &[&Ics27Gmp, &TestAccessManager, &attestation_lc];
    let mut chain = Chain::single(&deployer, programs);
    chain.prefund(&[&admin, &relayer, &non_admin]);

    // ── Init ──
    chain
        .init_with_attestation(&deployer, &admin, &relayer, programs, &attestors)
        .await;

    // ── Unauthorized propose ──
    let err = non_admin
        .gmp_propose_am_transfer(&mut chain, test_access_manager::ID)
        .await
        .expect_err("non-admin propose should fail");

    assert_eq!(
        extract_custom_error(&err),
        anchor_error_code(AccessManagerError::Unauthorized as u32),
    );
}
