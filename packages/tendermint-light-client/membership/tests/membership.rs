// SPDX-License-Identifier: Apache-2.0

//! Integration tests for membership verification functionality

mod helpers;

use helpers::*;
use tendermint_light_client_membership::{membership, MembershipError};

/// Helper to create a test context with tampered value
fn create_context_with_tampered_value(fixture: MembershipVerificationFixture) -> TestContext {
    let mut ctx = setup_test_context(fixture);
    ctx.kv_pair.value.push(0xFF); // Tamper with the value
    ctx
}

/// Helper to create a test context where membership is treated as non-membership
fn create_context_membership_as_non_membership(
    fixture: MembershipVerificationFixture,
) -> TestContext {
    let mut ctx = setup_test_context(fixture);
    ctx.kv_pair.value.clear(); // Clear value to make it look like non-membership
    ctx
}

#[test]
fn test_verify_membership_happy_path() {
    let fixture = load_membership_fixture_data();
    let ctx = setup_test_context(fixture);
    assert_membership_succeeds(&ctx, "membership happy path");
}

#[test]
fn test_verify_non_membership_happy_path() {
    let fixture = load_non_membership_fixture_data();
    let ctx = setup_test_context(fixture);
    assert_membership_succeeds(&ctx, "membership happy path");
}

#[test]
fn test_verify_membership_wrong_app_hash() {
    let fixture = load_membership_fixture_data();
    let ctx = create_context_with_wrong_app_hash(fixture);

    assert_membership_fails_with(
        &ctx,
        MembershipError::MembershipVerificationFailed,
        "wrong app hash",
    );
}

#[test]
fn test_verify_non_membership_wrong_app_hash() {
    let fixture = load_non_membership_fixture_data();
    let mut ctx = setup_test_context(fixture);

    // Use a completely different app hash
    ctx.app_hash = [0xFF; 32];

    let error = execute_membership(&ctx)
        .expect_err("Non-membership verification should have failed with wrong app hash");
    assert!(matches!(
        error,
        MembershipError::NonMembershipVerificationFailed
    ));
}

#[test]
fn test_verify_membership_with_non_membership_proof() {
    let membership_fixture = load_membership_fixture_data();
    let non_membership_fixture = load_non_membership_fixture_data();
    let ctx = setup_test_context(membership_fixture);
    let ctx = create_context_with_different_proof(ctx, non_membership_fixture);
    assert_membership_fails_with(
        &ctx,
        MembershipError::MembershipVerificationFailed,
        "wrong proof",
    );
}

#[test]
fn test_verify_multiple_kv_pairs() {
    // Test verifying multiple key-value pairs in a single call
    let membership_fixture = load_membership_fixture_data();
    let non_membership_fixture = load_non_membership_fixture_data();
    let membership_ctx = setup_test_context(membership_fixture);
    let non_membership_ctx = setup_test_context(non_membership_fixture);

    // Create a request with both membership and non-membership proofs
    let request = vec![
        (
            membership_ctx.kv_pair.clone(),
            membership_ctx.merkle_proof.clone(),
        ),
        (
            non_membership_ctx.kv_pair.clone(),
            non_membership_ctx.merkle_proof.clone(),
        ),
    ];

    membership(membership_ctx.app_hash, request.into_iter())
        .expect("Multiple KV pairs verification should succeed");
}

#[test]
fn test_verify_membership_empty_proof() {
    let fixture = load_membership_fixture_data();
    let ctx = create_context_with_empty_proof(fixture);
    assert_membership_fails_with(
        &ctx,
        MembershipError::MembershipVerificationFailed,
        "empty proof",
    );
}

#[test]
fn test_verify_membership_mismatched_path() {
    let fixture = load_membership_fixture_data();
    let ctx =
        create_context_with_mismatched_path(fixture, vec![b"different".to_vec(), b"path".to_vec()]);
    assert_membership_fails_with(
        &ctx,
        MembershipError::MembershipVerificationFailed,
        "mismatched path",
    );
}

#[test]
fn test_verify_membership_tampered_value() {
    let fixture: MembershipVerificationFixture = load_membership_fixture_data();
    let ctx = create_context_with_tampered_value(fixture);
    assert_membership_fails_with(
        &ctx,
        MembershipError::MembershipVerificationFailed,
        "tampered value",
    );
}

#[test]
fn test_verify_membership_as_non_membership() {
    let fixture = load_membership_fixture_data();
    let ctx = create_context_membership_as_non_membership(fixture);
    assert_membership_fails_with(
        &ctx,
        MembershipError::NonMembershipVerificationFailed,
        "membership proof treated as non-membership",
    );
}

#[test]
fn test_verify_membership_malformed_proof() {
    let fixture = load_membership_fixture_data();
    let ctx = create_context_with_malformed_proof(fixture);
    assert_membership_fails_with(
        &ctx,
        MembershipError::MembershipVerificationFailed,
        "malformed proof for membership",
    );
}
