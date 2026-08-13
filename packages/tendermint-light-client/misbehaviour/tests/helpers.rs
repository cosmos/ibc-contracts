// SPDX-License-Identifier: Apache-2.0

use ibc_client_tendermint::types::{ConsensusState, Misbehaviour};
use ibc_core_client_types::Height;
use ibc_core_commitment_types::commitment::CommitmentRoot;
use tendermint::AppHash;
use tendermint_light_client_misbehaviour::{
    check_for_misbehaviour, ClientState, MisbehaviourError, MisbehaviourOutput, TrustThreshold,
};
use tendermint_testgen::{
    light_block::TmLightBlock, Generator, Header as TgHeader, LightBlock as TgLightBlock,
    Validator as TgValidator,
};

const UNBONDING_PERIOD_SECS: u64 = 1_814_400; // 21 days
const MAX_CLOCK_DRIFT_SECS: u64 = 15;
const NANOS_PER_SEC: u128 = 1_000_000_000;
const BASE_TIME_SECS: i64 = 1_000_000_000;

pub fn base_time(offset_secs: i64) -> tendermint::Time {
    tendermint::Time::from_unix_timestamp(BASE_TIME_SECS.saturating_add(offset_secs), 0)
        .expect("valid timestamp")
}

pub struct DoubleSignScenario {
    pub trusted_cs: ConsensusState,
    pub misbehaviour: Misbehaviour,
}

pub fn double_sign_scenario(chain_id: &str) -> DoubleSignScenario {
    let validators = default_validators();
    let trusted_block = generate_light_block(&validators, chain_id, 1, base_time(0));
    let trusted_cs = consensus_state_from_light_block(&trusted_block);

    let block_a = generate_light_block_with_app_hash(
        &validators,
        chain_id,
        2,
        base_time(10),
        AppHash::try_from(vec![1u8; 32]).expect("valid app hash"),
    );
    let block_b = generate_light_block_with_app_hash(
        &validators,
        chain_id,
        2,
        base_time(20),
        AppHash::try_from(vec![2u8; 32]).expect("valid app hash"),
    );

    let trusted_height = Height::new(0, 1).expect("valid height");
    let header_a = to_ibc_header(
        &block_a,
        trusted_height,
        trusted_block.next_validators.clone(),
    );
    let header_b = to_ibc_header(&block_b, trusted_height, trusted_block.next_validators);

    DoubleSignScenario {
        trusted_cs,
        misbehaviour: build_misbehaviour(header_a, header_b),
    }
}

pub fn default_validators() -> Vec<TgValidator> {
    vec![
        TgValidator::new("val1").voting_power(50),
        TgValidator::new("val2").voting_power(50),
    ]
}

pub fn generate_light_block(
    validators: &[TgValidator],
    chain_id: &str,
    height: u64,
    time: tendermint::Time,
) -> TmLightBlock {
    let header = TgHeader::new(validators)
        .height(height)
        .chain_id(chain_id)
        .next_validators(validators)
        .time(time);
    TgLightBlock::new_default_with_header(header)
        .generate()
        .expect("failed to generate light block")
}

pub fn generate_light_block_with_app_hash(
    validators: &[TgValidator],
    chain_id: &str,
    height: u64,
    time: tendermint::Time,
    app_hash: AppHash,
) -> TmLightBlock {
    let header = TgHeader::new(validators)
        .height(height)
        .chain_id(chain_id)
        .next_validators(validators)
        .time(time)
        .app_hash(app_hash);
    TgLightBlock::new_default_with_header(header)
        .generate()
        .expect("failed to generate light block")
}

pub fn consensus_state_from_light_block(block: &TmLightBlock) -> ConsensusState {
    let header = &block.signed_header.header;
    ConsensusState::new(
        CommitmentRoot::from_bytes(header.app_hash.as_ref()),
        header.time,
        header.next_validators_hash,
    )
}

pub fn to_ibc_header(
    block: &TmLightBlock,
    trusted_height: Height,
    trusted_next_validators: tendermint::validator::Set,
) -> ibc_client_tendermint::types::Header {
    ibc_client_tendermint::types::Header {
        signed_header: block.signed_header.clone(),
        validator_set: block.validators.clone(),
        trusted_height,
        trusted_next_validator_set: trusted_next_validators,
    }
}

pub fn create_client_state(
    chain_id: &str,
    trusting_period_secs: u64,
    latest_height: u64,
) -> ClientState {
    create_client_state_with_drift(
        chain_id,
        trusting_period_secs,
        latest_height,
        MAX_CLOCK_DRIFT_SECS,
    )
}

pub fn create_client_state_with_drift(
    chain_id: &str,
    trusting_period_secs: u64,
    latest_height: u64,
    max_clock_drift_secs: u64,
) -> ClientState {
    ClientState {
        chain_id: chain_id.to_string(),
        trust_level: TrustThreshold::new(1, 3),
        trusting_period_seconds: trusting_period_secs,
        unbonding_period_seconds: UNBONDING_PERIOD_SECS,
        max_clock_drift_seconds: max_clock_drift_secs,
        is_frozen: false,
        latest_height: Height::new(0, latest_height).expect("valid height"),
    }
}

pub fn time_nanos(time: tendermint::Time) -> u128 {
    time.unix_timestamp_nanos() as u128
}

pub fn secs_to_nanos(secs: u128) -> u128 {
    secs.saturating_mul(NANOS_PER_SEC)
}

pub fn build_misbehaviour(
    header1: ibc_client_tendermint::types::Header,
    header2: ibc_client_tendermint::types::Header,
) -> Misbehaviour {
    use ibc_client_tendermint::types::TENDERMINT_CLIENT_TYPE;
    use ibc_core_host_types::identifiers::ClientId;

    let client_id = ClientId::new(TENDERMINT_CLIENT_TYPE, 0).expect("valid client id");
    Misbehaviour::new(client_id, header1, header2)
}

pub fn execute_check(
    client_state: &ClientState,
    misbehaviour: &Misbehaviour,
    trusted_cs_1: ConsensusState,
    trusted_cs_2: ConsensusState,
    current_time: u128,
) -> Result<MisbehaviourOutput, MisbehaviourError> {
    check_for_misbehaviour(
        client_state,
        misbehaviour,
        trusted_cs_1,
        trusted_cs_2,
        current_time,
    )
}

pub fn assert_misbehaviour_error(
    result: Result<MisbehaviourOutput, MisbehaviourError>,
    expected: MisbehaviourError,
) {
    let actual = result.expect_err("expected an error");
    assert_eq!(
        std::mem::discriminant(&expected),
        std::mem::discriminant(&actual),
        "expected {expected:?} but got {actual:?}",
    );
}
