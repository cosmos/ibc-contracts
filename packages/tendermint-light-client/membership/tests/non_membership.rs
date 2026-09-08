// SPDX-License-Identifier: Apache-2.0

//! Non-membership specific test cases

mod helpers;

use helpers::*;
use tendermint_light_client_membership::MembershipError;

/// Helper to create a test context where non-membership is treated as membership
fn create_context_non_membership_as_membership(
    fixture: MembershipVerificationFixture,
    fake_value: Vec<u8>,
) -> TestContext {
    let mut ctx = setup_test_context(fixture);
    ctx.kv_pair.value = fake_value; // Add fake value to make it look like membership
    ctx
}

#[test]
fn test_verify_non_membership_happy_path() {
    let fixture = load_non_membership_fixture_data();

    let ctx = setup_test_context(fixture);

    assert_membership_succeeds(&ctx, "non-membership happy path");
}

#[test]
fn test_verify_non_membership_with_membership_proof() {
    let non_membership_fixture = load_non_membership_fixture_data();
    let membership_fixture = load_membership_fixture_data();

    let ctx = setup_test_context(non_membership_fixture);

    let ctx = create_context_with_different_proof(ctx, membership_fixture);
    assert_membership_fails_with(
        &ctx,
        MembershipError::NonMembershipVerificationFailed,
        "wrong proof for non-membership",
    );
}

#[test]
fn test_verify_non_membership_wrong_app_hash() {
    let fixture = load_non_membership_fixture_data();

    let ctx = create_context_with_wrong_app_hash(fixture);

    assert_membership_fails_with(
        &ctx,
        MembershipError::NonMembershipVerificationFailed,
        "wrong app hash for non-membership",
    );
}

#[test]
fn test_verify_non_membership_as_membership() {
    let fixture = load_non_membership_fixture_data();

    let ctx = create_context_non_membership_as_membership(fixture, b"fake_value".to_vec());

    assert_membership_fails_with(
        &ctx,
        MembershipError::MembershipVerificationFailed,
        "non-membership proof treated as membership",
    );
}

#[test]
fn test_verify_non_membership_empty_proof() {
    let fixture = load_non_membership_fixture_data();

    let ctx = create_context_with_empty_proof(fixture);

    assert_membership_fails_with(
        &ctx,
        MembershipError::NonMembershipVerificationFailed,
        "empty proof for non-membership",
    );
}

#[test]
fn test_verify_non_membership_mismatched_path() {
    let fixture = load_non_membership_fixture_data();

    let ctx = create_context_with_mismatched_path(
        fixture,
        vec![b"different".to_vec(), b"nonexistent".to_vec()],
    );

    assert_membership_fails_with(
        &ctx,
        MembershipError::NonMembershipVerificationFailed,
        "mismatched path for non-membership",
    );
}

#[test]
fn test_verify_non_membership_malformed_proof() {
    let fixture = load_non_membership_fixture_data();

    let ctx = create_context_with_malformed_proof(fixture);

    assert_membership_fails_with(
        &ctx,
        MembershipError::NonMembershipVerificationFailed,
        "malformed proof for non-membership",
    );
}
