// SPDX-License-Identifier: Apache-2.0

//! Integration tests for update client functionality

mod helpers;

use helpers::*;

const ONE_HOUR_IN_SECONDS: u64 = 3600;

#[test]
fn test_update_client_happy_path() {
    let fixture = load_happy_path_fixture();
    let ctx = setup_test_context(fixture);

    let output = execute_update_client(&ctx).expect("Expected success but failed for happy path");
    assert!(
        output.latest_height.revision_height() > output.trusted_height.revision_height(),
        "New height should be greater than trusted height"
    );
}

#[test]
fn test_update_client_malformed_message() {
    let fixture = load_happy_path_fixture();
    let mut ctx = setup_test_context(fixture);

    corrupt_header_signature(&mut ctx.proposed_header);

    execute_update_client(&ctx).expect_err(
        "Malformed message test should have failed but succeeded for malformed message",
    );
}

#[test]
fn test_update_client_expired_header() {
    let fixture = load_happy_path_fixture();
    let mut ctx = setup_test_context(fixture);

    let trusting_period_plus_buffer =
        ctx.client_state.trusting_period_seconds + ONE_HOUR_IN_SECONDS;
    set_header_timestamp_to_past(&mut ctx.proposed_header, trusting_period_plus_buffer);

    execute_update_client(&ctx).expect_err("Expected failure but succeeded for expired header");
}

#[test]
fn test_update_client_future_timestamp() {
    let fixture = load_happy_path_fixture();
    let mut ctx = setup_test_context(fixture);

    let max_clock_drift_plus_buffer =
        ctx.client_state.max_clock_drift_seconds + ONE_HOUR_IN_SECONDS;
    set_header_timestamp_to_future(&mut ctx.proposed_header, max_clock_drift_plus_buffer);

    execute_update_client(&ctx).expect_err("Expected failure but succeeded for future timestamp");
}

#[test]
fn test_update_client_wrong_trusted_height() {
    let fixture = load_happy_path_fixture();
    let mut ctx = setup_test_context(fixture);

    let non_existent_height = ctx.client_state.latest_height.revision_height() + 100;
    set_wrong_trusted_height(&mut ctx.proposed_header, non_existent_height);

    execute_update_client(&ctx)
        .expect_err("Expected failure but succeeded for wrong trusted height");
}
