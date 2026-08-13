// SPDX-License-Identifier: Apache-2.0

//! Common test utilities and fixtures for update client tests

use serde::Deserialize;
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use ibc_client_tendermint::types::{ConsensusState, Header};
use ibc_core_client_types::Height;
use ibc_core_commitment_types::commitment::CommitmentRoot;
use prost::Message;
use tendermint_light_client_update_client::{
    update_client, ClientState, TrustThreshold, UpdateClientError,
};

#[derive(Debug, Deserialize, Clone)]
pub struct UpdateClientMessageFixture {
    pub client_message_hex: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateClientFixture {
    pub client_state_hex: String,
    pub consensus_state_hex: String,
    pub update_client_message: UpdateClientMessageFixture,
}

fn client_state_from_proto(
    proto: ibc_client_tendermint::types::proto::v1::ClientState,
) -> Result<ClientState, Box<dyn std::error::Error>> {
    let trust_level = proto
        .trust_level
        .ok_or("Missing trust level in client state")?;

    let trusting_period = proto
        .trusting_period
        .ok_or("Missing trusting period in client state")?;
    let unbonding_period = proto
        .unbonding_period
        .ok_or("Missing unbonding period in client state")?;
    let max_clock_drift = proto
        .max_clock_drift
        .ok_or("Missing max clock drift in client state")?;

    let latest_height = proto
        .latest_height
        .ok_or("Missing latest height in client state")?;

    Ok(ClientState {
        chain_id: proto.chain_id,
        trust_level: TrustThreshold::new(trust_level.numerator, trust_level.denominator),
        trusting_period_seconds: trusting_period.seconds as u64,
        unbonding_period_seconds: unbonding_period.seconds as u64,
        max_clock_drift_seconds: max_clock_drift.seconds as u64,
        is_frozen: proto.frozen_height.is_some(),
        latest_height: Height::new(latest_height.revision_number, latest_height.revision_height)
            .map_err(|e| format!("Invalid height: {}", e))?,
    })
}

pub fn client_state_from_hex(hex_str: &str) -> Result<ClientState, Box<dyn std::error::Error>> {
    let bytes =
        hex::decode(hex_str).map_err(|e| format!("Failed to decode client state hex: {}", e))?;

    let proto_client_state =
        ibc_client_tendermint::types::proto::v1::ClientState::decode(&bytes[..])
            .map_err(|e| format!("Failed to decode protobuf client state: {}", e))?;

    client_state_from_proto(proto_client_state)
}

fn consensus_state_from_proto(
    proto: ibc_client_tendermint::types::proto::v1::ConsensusState,
) -> Result<ConsensusState, Box<dyn std::error::Error>> {
    let timestamp = proto
        .timestamp
        .ok_or("Missing timestamp in consensus state")?;
    let root = proto.root.ok_or("Missing root in consensus state")?;

    let tm_timestamp =
        tendermint::Time::from_unix_timestamp(timestamp.seconds, timestamp.nanos as u32)
            .map_err(|e| format!("Failed to create timestamp: {}", e))?;

    let next_validators_hash = tendermint::Hash::from_bytes(
        tendermint::hash::Algorithm::Sha256,
        &proto.next_validators_hash,
    )
    .map_err(|e| format!("Failed to create next_validators_hash: {}", e))?;

    let commitment_root = CommitmentRoot::from_bytes(&root.hash);

    Ok(ConsensusState::new(
        commitment_root,
        tm_timestamp,
        next_validators_hash,
    ))
}

pub fn consensus_state_from_hex(
    hex_str: &str,
) -> Result<ConsensusState, Box<dyn std::error::Error>> {
    let bytes =
        hex::decode(hex_str).map_err(|e| format!("Failed to decode consensus state hex: {}", e))?;

    let proto_consensus_state =
        ibc_client_tendermint::types::proto::v1::ConsensusState::decode(&bytes[..])
            .map_err(|e| format!("Failed to decode protobuf consensus state: {}", e))?;

    consensus_state_from_proto(proto_consensus_state)
}

pub fn load_fixture(filename: &str) -> UpdateClientFixture {
    let fixture_path = Path::new("../fixtures").join(format!("{}.json", filename));
    let fixture_content = fs::read_to_string(&fixture_path)
        .unwrap_or_else(|_| panic!("Failed to read fixture: {}", fixture_path.display()));

    serde_json::from_str(&fixture_content)
        .unwrap_or_else(|_| panic!("Failed to parse fixture: {}", fixture_path.display()))
}

pub fn hex_to_header(hex_str: &str) -> Result<Header, Box<dyn std::error::Error>> {
    let bytes = hex::decode(hex_str).map_err(|e| format!("Failed to decode header hex: {}", e))?;

    let proto_header = ibc_client_tendermint::types::proto::v1::Header::decode(&bytes[..])
        .map_err(|e| format!("Failed to decode protobuf header: {}", e))?;

    Header::try_from(proto_header).map_err(|e| format!("Failed to convert header: {}", e).into())
}

pub struct TestContext {
    pub client_state: ClientState,
    pub trusted_consensus_state: ConsensusState,
    pub proposed_header: Header,
    pub current_time: u128,
}

pub fn setup_test_context(fixture: UpdateClientFixture) -> TestContext {
    let client_state = client_state_from_hex(&fixture.client_state_hex)
        .expect("Failed to create client state from fixture");

    let trusted_consensus_state = consensus_state_from_hex(&fixture.consensus_state_hex)
        .expect("Failed to create consensus state from fixture");

    let proposed_header = hex_to_header(&fixture.update_client_message.client_message_hex)
        .expect("Failed to parse header from fixture");

    let one_hour_nanos: u128 = 3600 * 1_000_000_000;

    let current_time = (trusted_consensus_state.timestamp.unix_timestamp_nanos() as u128)
        .saturating_add(one_hour_nanos);

    TestContext {
        client_state,
        trusted_consensus_state,
        proposed_header,
        current_time,
    }
}

pub fn execute_update_client(
    ctx: &TestContext,
) -> Result<tendermint_light_client_update_client::UpdateClientOutput, UpdateClientError> {
    #[cfg(not(feature = "solana"))]
    {
        update_client(
            &ctx.client_state,
            &ctx.trusted_consensus_state,
            ctx.proposed_header.clone(),
            ctx.current_time,
        )
    }
    #[cfg(feature = "solana")]
    {
        // For tests with solana feature, use empty verification accounts and a dummy program ID
        let dummy_program_id = anchor_lang::prelude::Pubkey::new_unique();
        update_client(
            &ctx.client_state,
            &ctx.trusted_consensus_state,
            ctx.proposed_header.clone(),
            ctx.current_time,
            &[],
            &dummy_program_id,
        )
    }
}

pub fn load_happy_path_fixture() -> UpdateClientFixture {
    load_fixture("update_client_happy_path")
}

pub fn corrupt_header_signature(header: &mut Header) {
    use tendermint::block::CommitSig;

    // Find the first signature and corrupt a single byte
    for sig in header.signed_header.commit.signatures.iter_mut() {
        let signature = match sig {
            CommitSig::BlockIdFlagCommit { signature, .. }
            | CommitSig::BlockIdFlagNil { signature, .. } => signature,
            _ => continue,
        };

        if let Some(sig_opt) = signature {
            let mut sig_bytes = sig_opt.as_bytes().to_vec();
            if !sig_bytes.is_empty() {
                // Flip a single bit in the middle of the signature
                let mid_pos = sig_bytes.len() / 2;
                sig_bytes[mid_pos] ^= 0x01;

                if let Ok(corrupted_sig) = tendermint::Signature::try_from(sig_bytes.as_slice()) {
                    *signature = Some(corrupted_sig);
                    break; // Only corrupt the first signature found
                }
            }
        }
    }
}

pub fn set_header_timestamp_to_past(header: &mut Header, seconds_ago: u64) {
    let past_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        .saturating_sub(seconds_ago);

    let tm_time = tendermint::Time::from_unix_timestamp(past_time as i64, 0)
        .expect("Failed to create past timestamp");
    header.signed_header.header.time = tm_time;
}

pub fn set_header_timestamp_to_future(header: &mut Header, seconds_ahead: u64) {
    let future_time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
        + seconds_ahead;

    let tm_time = tendermint::Time::from_unix_timestamp(future_time as i64, 0)
        .expect("Failed to create future timestamp");
    header.signed_header.header.time = tm_time;
}

pub fn set_wrong_trusted_height(header: &mut Header, wrong_height: u64) {
    header.trusted_height = Height::new(header.trusted_height.revision_number(), wrong_height)
        .expect("Failed to create wrong trusted height");
}
