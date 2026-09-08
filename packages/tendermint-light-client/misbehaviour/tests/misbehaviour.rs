// SPDX-License-Identifier: Apache-2.0

mod helpers;

use helpers::*;
use ibc_core_client_types::Height;

use tendermint_light_client_misbehaviour::MisbehaviourError;

const CHAIN_ID: &str = "test-chain-0";
const TRUSTING_PERIOD_SECS: u64 = 1_209_600; // 14 days

#[test]
fn test_double_sign_misbehaviour() {
    let scenario = double_sign_scenario(CHAIN_ID);
    let client_state = create_client_state(CHAIN_ID, TRUSTING_PERIOD_SECS, 2);
    let current_time = time_nanos(base_time(3600));

    let output = execute_check(
        &client_state,
        &scenario.misbehaviour,
        scenario.trusted_cs.clone(),
        scenario.trusted_cs,
        current_time,
    )
    .expect("double-sign misbehaviour should succeed");

    assert_eq!(output.trusted_height_1.revision_height(), 1);
    assert_eq!(output.trusted_height_2.revision_height(), 1);
}

#[test]
fn test_time_monotonicity_misbehaviour() {
    let validators = default_validators();

    let trusted_block = generate_light_block(&validators, CHAIN_ID, 1, base_time(0));
    let trusted_cs = consensus_state_from_light_block(&trusted_block);

    // header1 at height 3 with earlier time (T+10)
    let block_1 = generate_light_block(&validators, CHAIN_ID, 3, base_time(10));
    // header2 at height 2 with later time (T+20)
    let block_2 = generate_light_block(&validators, CHAIN_ID, 2, base_time(20));

    // Misbehaviour requires header1.height >= header2.height (3 >= 2)
    // Time violation: header1.time (T+10) < header2.time (T+20) while height increased
    assert!(
        block_1.signed_header.header.time <= block_2.signed_header.header.time,
        "time monotonicity must be violated for this test"
    );

    let trusted_height = Height::new(0, 1).expect("valid height");
    let header_1 = to_ibc_header(
        &block_1,
        trusted_height,
        trusted_block.next_validators.clone(),
    );
    let header_2 = to_ibc_header(
        &block_2,
        trusted_height,
        trusted_block.next_validators.clone(),
    );

    let misbehaviour = build_misbehaviour(header_1, header_2);
    let client_state = create_client_state(CHAIN_ID, TRUSTING_PERIOD_SECS, 3);

    let current_time = time_nanos(base_time(3600));

    let output = execute_check(
        &client_state,
        &misbehaviour,
        trusted_cs.clone(),
        trusted_cs,
        current_time,
    )
    .expect("time monotonicity misbehaviour should succeed");

    assert_eq!(output.trusted_height_1.revision_height(), 1);
    assert_eq!(output.trusted_height_2.revision_height(), 1);
}

#[test]
fn test_chain_id_mismatch() {
    let scenario = double_sign_scenario("other-chain-0");
    // Client state uses a different chain_id than the headers
    let client_state = create_client_state(CHAIN_ID, TRUSTING_PERIOD_SECS, 2);
    let current_time = time_nanos(base_time(3600));

    let result = execute_check(
        &client_state,
        &scenario.misbehaviour,
        scenario.trusted_cs.clone(),
        scenario.trusted_cs,
        current_time,
    );

    assert_misbehaviour_error(result, MisbehaviourError::ChainIdMismatch);
}

#[test]
fn test_invalid_chain_id() {
    let scenario = double_sign_scenario(CHAIN_ID);
    // Empty chain_id is invalid
    let client_state = create_client_state("", TRUSTING_PERIOD_SECS, 2);
    let current_time = time_nanos(base_time(3600));

    let result = execute_check(
        &client_state,
        &scenario.misbehaviour,
        scenario.trusted_cs.clone(),
        scenario.trusted_cs,
        current_time,
    );

    assert_misbehaviour_error(result, MisbehaviourError::InvalidChainId(String::new()));
}

#[test]
fn test_no_misbehaviour_same_block() {
    let validators = default_validators();

    let trusted_block = generate_light_block(&validators, CHAIN_ID, 1, base_time(0));
    let trusted_cs = consensus_state_from_light_block(&trusted_block);

    let block = generate_light_block(&validators, CHAIN_ID, 2, base_time(10));

    let trusted_height = Height::new(0, 1).expect("valid height");
    let header = to_ibc_header(
        &block,
        trusted_height,
        trusted_block.next_validators.clone(),
    );

    // Same block used as both headers — identical block_id.hash
    let misbehaviour = build_misbehaviour(header.clone(), header);
    let client_state = create_client_state(CHAIN_ID, TRUSTING_PERIOD_SECS, 2);

    let current_time = time_nanos(base_time(3600));

    let result = execute_check(
        &client_state,
        &misbehaviour,
        trusted_cs.clone(),
        trusted_cs,
        current_time,
    );

    assert_misbehaviour_error(result, MisbehaviourError::MisbehaviourNotDetected);
}

#[test]
fn test_expired_trusted_state() {
    let scenario = double_sign_scenario(CHAIN_ID);
    let client_state = create_client_state(CHAIN_ID, TRUSTING_PERIOD_SECS, 2);

    // Set current_time well beyond the trusting period from the trusted state
    let expired_offset = (TRUSTING_PERIOD_SECS + 3600) as u128;
    let current_time = time_nanos(base_time(0)) + secs_to_nanos(expired_offset);

    let result = execute_check(
        &client_state,
        &scenario.misbehaviour,
        scenario.trusted_cs.clone(),
        scenario.trusted_cs,
        current_time,
    );

    assert_misbehaviour_error(result, MisbehaviourError::MisbehaviourVerificationFailed);
}

/// Proves that `max_clock_drift_seconds` in `ClientState` has no effect on
/// misbehaviour verification. The downstream verifier (`cometbft/tendermint-rs`)
/// intentionally skips the "header from the future" check in the misbehaviour
/// path to allow detecting FLA (Fake Light client Attack) evidence.
///
/// See: https://github.com/cometbft/tendermint-rs/blob/a2aa5bffcb2a2b62f775ba0c150819ff3341f881/light-client-verifier/src/verifier.rs#L310-L324
#[test]
fn test_clock_drift_is_unused_in_misbehaviour_verification() {
    let scenario = double_sign_scenario(CHAIN_ID);
    let current_time = time_nanos(base_time(3600));

    // Verify with zero drift — would reject any "future" header if the check ran
    let cs_zero = create_client_state_with_drift(CHAIN_ID, TRUSTING_PERIOD_SECS, 2, 0);
    let out_zero = execute_check(
        &cs_zero,
        &scenario.misbehaviour,
        scenario.trusted_cs.clone(),
        scenario.trusted_cs.clone(),
        current_time,
    )
    .expect("zero clock_drift should not affect misbehaviour");

    // Verify with an absurdly large drift
    let cs_huge = create_client_state_with_drift(CHAIN_ID, TRUSTING_PERIOD_SECS, 2, u64::MAX);
    let out_huge = execute_check(
        &cs_huge,
        &scenario.misbehaviour,
        scenario.trusted_cs.clone(),
        scenario.trusted_cs.clone(),
        current_time,
    )
    .expect("huge clock_drift should not affect misbehaviour");

    // Both produce identical outputs — the value is passed through but never read
    assert_eq!(
        out_zero.trusted_height_1, out_huge.trusted_height_1,
        "trusted_height_1 must be identical regardless of clock_drift"
    );
    assert_eq!(
        out_zero.trusted_height_2, out_huge.trusted_height_2,
        "trusted_height_2 must be identical regardless of clock_drift"
    );
    assert_eq!(out_zero.time, out_huge.time);
}
