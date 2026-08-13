// SPDX-License-Identifier: Apache-2.0

//! Integration tests for combined update client and membership functionality

mod helpers;

use helpers::*;
use tendermint_light_client_uc_and_membership::UcAndMembershipError;

const ONE_HOUR_IN_SECONDS: u64 = 3600;

#[test]
fn test_uc_and_membership_empty_membership_fails() {
    let fixture = load_combined_happy_path_fixture();
    let ctx = setup_test_context(fixture);

    let empty_request = vec![];

    let result = tendermint_light_client_uc_and_membership::update_client_and_membership(
        &ctx.client_state,
        &ctx.trusted_consensus_state,
        ctx.proposed_header.clone(),
        ctx.current_time,
        &empty_request,
    );

    assert!(
        matches!(
            result,
            Err(UcAndMembershipError::Membership(
                tendermint_light_client_membership::MembershipError::EmptyRequest,
            ))
        ),
        "Empty membership request should fail with EmptyRequest"
    );
}

#[test]
fn test_uc_and_membership_update_client_fails_expired_header() {
    let fixture = load_combined_expired_header_fixture();
    let mut ctx = setup_test_context(fixture);

    // Set the header to be expired
    let trusting_period_plus_buffer =
        ctx.client_state.trusting_period_seconds + ONE_HOUR_IN_SECONDS;
    set_header_timestamp_to_past(&mut ctx.proposed_header, trusting_period_plus_buffer);

    assert_uc_and_membership_failure_with_error(&ctx, "UpdateClient", "expired header");
}

#[test]
fn test_uc_and_membership_membership_fails_tampered_value() {
    let fixture = load_combined_invalid_membership_fixture();
    let mut ctx = setup_test_context(fixture);
    ctx.kv_pair.value = b"tampered_value".to_vec();
    assert_uc_and_membership_failure_with_error(&ctx, "Membership", "tampered value");
}

#[test]
fn test_uc_and_membership_membership_fails_empty_proof() {
    let fixture = load_combined_invalid_membership_fixture();
    let mut ctx = setup_test_context(fixture);
    ctx.merkle_proof = ibc_core_commitment_types::merkle::MerkleProof { proofs: vec![] };
    assert_uc_and_membership_failure_with_error(&ctx, "Membership", "empty proof");
}

#[test]
fn test_uc_and_membership_membership_fails_mismatched_path() {
    let fixture = load_combined_invalid_membership_fixture();
    let mut ctx = setup_test_context(fixture);
    ctx.kv_pair.path = vec![b"wrong".to_vec(), b"path".to_vec()];
    assert_uc_and_membership_failure_with_error(&ctx, "Membership", "mismatched path");
}

#[test]
fn test_uc_and_membership_empty_request() {
    let fixture = load_combined_happy_path_fixture();
    let ctx = setup_test_context(fixture);

    let empty_request = vec![];

    let result = tendermint_light_client_uc_and_membership::update_client_and_membership(
        &ctx.client_state,
        &ctx.trusted_consensus_state,
        ctx.proposed_header.clone(),
        ctx.current_time,
        &empty_request,
    );

    assert!(
        matches!(
            result,
            Err(UcAndMembershipError::Membership(
                tendermint_light_client_membership::MembershipError::EmptyRequest,
            ))
        ),
        "Empty membership request should fail with EmptyRequest"
    );
}

#[test]
fn test_uc_and_membership_sequence_validation() {
    // Test that both update client and membership validation occur in the correct order
    // First update client should succeed, then membership should be validated
    let fixture = load_combined_invalid_membership_fixture();

    // Create context where update client will succeed but membership will fail
    let mut ctx = setup_test_context(fixture);
    ctx.kv_pair.value = b"tampered_value".to_vec();

    // First verify that update client would succeed on its own
    let uc_result = tendermint_light_client_update_client::update_client(
        &ctx.client_state,
        &ctx.trusted_consensus_state,
        ctx.proposed_header.clone(),
        ctx.current_time,
    );

    uc_result.expect("Expected update client to succeed");

    // Now test the combined operation - should fail at membership step
    let error =
        execute_uc_and_membership(&ctx).expect_err("Expected failure due to invalid membership");

    assert!(
        matches!(
            error,
            UcAndMembershipError::Membership(
                tendermint_light_client_membership::MembershipError::MembershipVerificationFailed,
            )
        ),
        "Expected Membership::MembershipVerificationFailed but got: {:?}",
        error
    );
}

#[test]
fn test_uc_and_membership_update_client_error_type() {
    // Test specific UpdateClient error type for expired header
    let expired_fixture = load_combined_expired_header_fixture();
    let mut ctx = setup_test_context(expired_fixture);

    // Set the header to be expired
    let trusting_period_plus_buffer =
        ctx.client_state.trusting_period_seconds + ONE_HOUR_IN_SECONDS;
    set_header_timestamp_to_past(&mut ctx.proposed_header, trusting_period_plus_buffer);

    let error =
        execute_uc_and_membership(&ctx).expect_err("Expected UpdateClient error but succeeded");

    assert!(
        matches!(
            error,
            UcAndMembershipError::UpdateClient(
                tendermint_light_client_update_client::UpdateClientError::HeaderVerificationFailed,
            )
        ),
        "Expected UpdateClient::HeaderVerificationFailed but got: {:?}",
        error
    );
}

#[test]
fn test_uc_and_membership_membership_error_type() {
    // Test specific Membership error type for empty proof
    let membership_fixture = load_combined_invalid_membership_fixture();
    let mut ctx = setup_test_context(membership_fixture);
    ctx.merkle_proof = ibc_core_commitment_types::merkle::MerkleProof { proofs: vec![] };

    // First verify that update client would succeed on its own
    let uc_result = tendermint_light_client_update_client::update_client(
        &ctx.client_state,
        &ctx.trusted_consensus_state,
        ctx.proposed_header.clone(),
        ctx.current_time,
    );

    uc_result.expect("Update client must succeed for this test - membership errors should only be tested when update client works");

    let error =
        execute_uc_and_membership(&ctx).expect_err("Expected Membership error but succeeded");

    assert!(
        matches!(
            error,
            UcAndMembershipError::Membership(
                tendermint_light_client_membership::MembershipError::MembershipVerificationFailed,
            )
        ),
        "Expected Membership::MembershipVerificationFailed but got: {:?}",
        error
    );
}
