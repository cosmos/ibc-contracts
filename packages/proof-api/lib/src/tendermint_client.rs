// SPDX-License-Identifier: Apache-2.0

//! Utilities for Tendermint light client configuration
//! This module provides common functionality for creating and configuring Tendermint client states

use ibc_proto_eureka::{
    google::protobuf::Duration,
    ibc::{
        core::client::v1::Height,
        lightclients::tendermint::v1::{ClientState, Fraction},
    },
};

/// Default trust level for Tendermint light clients (1/3)
#[must_use]
pub const fn default_trust_level() -> Fraction {
    Fraction {
        numerator: 1,
        denominator: 3,
    }
}

/// Default max clock drift for Tendermint light clients (15 seconds)
#[must_use]
pub const fn default_max_clock_drift() -> Duration {
    Duration {
        seconds: 15,
        nanos: 0,
    }
}

/// Build a Tendermint client state with common defaults
///
/// # Arguments
/// * `chain_id` - The chain ID
/// * `height` - The latest height
/// * `trusting_period` - The trusting period
/// * `unbonding_period` - The unbonding period
/// * `proof_specs` - The proof specifications for ICS23
///
/// Returns a `ClientState` with default trust level, max clock drift, and the provided proof specs
#[must_use]
pub fn build_tendermint_client_state(
    chain_id: String,
    height: Height,
    trusting_period: Duration,
    unbonding_period: Duration,
    proof_specs: Vec<ics23::ProofSpec>,
) -> ClientState {
    build_tendermint_client_state_with_trust_level(
        chain_id,
        height,
        trusting_period,
        unbonding_period,
        proof_specs,
        None,
    )
}

/// Build a Tendermint client state with configurable trust level
///
/// # Arguments
/// * `chain_id` - The chain ID
/// * `height` - The latest height
/// * `trusting_period` - The trusting period
/// * `unbonding_period` - The unbonding period
/// * `proof_specs` - The proof specifications for ICS23
/// * `trust_level` - Optional trust level (defaults to 1/3 if None)
///
/// Returns a `ClientState` with the specified trust level, max clock drift, and the provided proof specs
#[must_use]
pub fn build_tendermint_client_state_with_trust_level(
    chain_id: String,
    height: Height,
    trusting_period: Duration,
    unbonding_period: Duration,
    proof_specs: Vec<ics23::ProofSpec>,
    trust_level: Option<Fraction>,
) -> ClientState {
    ClientState {
        chain_id,
        trust_level: Some(trust_level.unwrap_or_else(default_trust_level)),
        trusting_period: Some(trusting_period),
        unbonding_period: Some(unbonding_period),
        max_clock_drift: Some(default_max_clock_drift()),
        latest_height: Some(height),
        proof_specs,
        upgrade_path: vec!["upgrade".to_string(), "upgradedIBCState".to_string()],
        ..Default::default()
    }
}
