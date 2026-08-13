// SPDX-License-Identifier: Apache-2.0

//! The crate that contains the types and utilities for `tendermint-light-client-update-client`
//! program.
#![deny(missing_docs, clippy::nursery, clippy::pedantic, warnings)]
#![cfg_attr(not(test), deny(unused_crate_dependencies))]

pub mod types;

use sha2 as _;

use std::{str::FromStr, time::Duration};

use ibc_client_tendermint::{
    client_state::verify_header,
    types::{ConsensusState, Header, TENDERMINT_CLIENT_TYPE},
};
use ibc_core_client_types::Height;
use ibc_core_host_types::identifiers::{ChainId, ClientId};
use tendermint::{crypto::Sha256, merkle::MerkleHash};
use tendermint_light_client_verifier::{
    options::Options, types::TrustThreshold as TmTrustThreshold,
};

#[cfg(not(feature = "solana"))]
use tendermint_light_client_verifier::ProdVerifier;

/// Trust threshold
#[derive(Clone, Debug)]
pub struct TrustThreshold {
    /// Numerator of the fraction
    pub numerator: u64,
    /// Denominator of the fraction
    pub denominator: u64,
}

impl TrustThreshold {
    /// Create a new trust threshold
    #[must_use]
    pub const fn new(numerator: u64, denominator: u64) -> Self {
        Self {
            numerator,
            denominator,
        }
    }
}

impl From<TrustThreshold> for TmTrustThreshold {
    fn from(tt: TrustThreshold) -> Self {
        Self::new(tt.numerator, tt.denominator)
            .expect("trust threshold numerator must be less than or equal to denominator")
    }
}

/// Client state
#[derive(Clone, Debug)]
pub struct ClientState {
    /// Chain ID
    pub chain_id: String,
    /// Trust level
    pub trust_level: TrustThreshold,
    /// Trusting period in seconds
    pub trusting_period_seconds: u64,
    /// Unbonding period in seconds
    pub unbonding_period_seconds: u64,
    /// Max clock drift in seconds
    pub max_clock_drift_seconds: u64,
    /// Frozen height (None if not frozen)
    pub is_frozen: bool,
    /// Latest height
    pub latest_height: Height,
}

impl ClientState {
    /// Default max clock drift in seconds
    pub const MAX_CLOCK_DRIFT_SECONDS: u64 = 15;
}

/// Output from update client verification
#[derive(Clone, Debug)]
pub struct UpdateClientOutput {
    /// New consensus state from the verified header
    pub new_consensus_state: ConsensusState,
    /// New latest height
    pub latest_height: Height,
    /// The trusted height used for verification
    pub trusted_height: Height,
}

/// Error type for update client
#[derive(Debug, thiserror::Error)]
pub enum UpdateClientError {
    /// Invalid client ID
    #[error("invalid client ID")]
    InvalidClientId,
    /// Invalid chain ID
    #[error("invalid chain ID: {0}")]
    InvalidChainId(String),
    /// Header verification failed
    #[error("header verification failed")]
    HeaderVerificationFailed,
}

/// IBC light client update client
///
/// # Errors
///
/// This function will return an error if:
/// - The client ID cannot be created
/// - The chain ID is invalid
/// - Header verification fails
#[cfg(not(feature = "solana"))]
pub fn update_client(
    client_state: &ClientState,
    trusted_consensus_state: &ConsensusState,
    proposed_header: Header,
    time: u128,
) -> Result<UpdateClientOutput, UpdateClientError> {
    update_client_impl(client_state, trusted_consensus_state, proposed_header, time)
}

/// IBC light client update client with Solana-optimized Ed25519 signature verification.
///
/// # Parameters
/// - `client_state` - On-chain client state (chain ID, trust level, periods, clock drift).
/// - `trusted_consensus_state` - Consensus state at `proposed_header.trusted_height`.
/// - `proposed_header` - Tendermint header to verify against the trusted state.
/// - `time` - Current time in **nanoseconds** since Unix epoch.
/// - `verification_accounts` - Pre-verified signature PDAs from `remaining_accounts`.
///   Derived as `PDA(["sig_verify", sha256(pk ‖ msg ‖ sig)], program_id)`.
///   Falls back to `brine-ed25519` when accounts are missing.
/// - `program_id` - Light-client program ID for signature PDA derivation.
///
/// # Errors
/// Returns error if client/chain ID is invalid or header verification fails.
#[cfg(feature = "solana")]
pub fn update_client<'a>(
    client_state: &ClientState,
    trusted_consensus_state: &ConsensusState,
    proposed_header: Header,
    time: u128,
    verification_accounts: &'a [anchor_lang::prelude::AccountInfo<'a>],
    program_id: &'a anchor_lang::prelude::Pubkey,
) -> Result<UpdateClientOutput, UpdateClientError> {
    update_client_impl(
        client_state,
        trusted_consensus_state,
        proposed_header,
        time,
        verification_accounts,
        program_id,
    )
}

#[cfg(not(feature = "solana"))]
fn update_client_impl(
    client_state: &ClientState,
    trusted_consensus_state: &ConsensusState,
    proposed_header: Header,
    time: u128,
) -> Result<UpdateClientOutput, UpdateClientError> {
    let verifier = ProdVerifier::default();
    verify_and_create_output::<_, sha2::Sha256>(
        client_state,
        trusted_consensus_state,
        proposed_header,
        time,
        &verifier,
    )
}

#[cfg(feature = "solana")]
fn update_client_impl<'a>(
    client_state: &ClientState,
    trusted_consensus_state: &ConsensusState,
    proposed_header: Header,
    time: u128,
    verification_accounts: &'a [anchor_lang::prelude::AccountInfo<'a>],
    program_id: &'a anchor_lang::prelude::Pubkey,
) -> Result<UpdateClientOutput, UpdateClientError> {
    let verifier = tendermint_light_client_solana::SolanaVerifier::new(
        tendermint_light_client_solana::SolanaPredicates,
        tendermint_light_client_solana::SolanaVotingPowerCalculator::new(
            tendermint_light_client_solana::SolanaSignatureVerifier::new(
                verification_accounts,
                program_id,
            ),
        ),
        tendermint_light_client_verifier::operations::commit_validator::ProdCommitValidator,
    );
    verify_and_create_output::<_, tendermint_light_client_solana::SolanaSha256>(
        client_state,
        trusted_consensus_state,
        proposed_header,
        time,
        &verifier,
    )
}

fn verify_and_create_output<V, H>(
    client_state: &ClientState,
    trusted_consensus_state: &ConsensusState,
    proposed_header: Header,
    time: u128,
    verifier: &V,
) -> Result<UpdateClientOutput, UpdateClientError>
where
    V: tendermint_light_client_verifier::Verifier,
    H: MerkleHash + Sha256 + Default,
{
    let client_id =
        ClientId::new(TENDERMINT_CLIENT_TYPE, 0).map_err(|_| UpdateClientError::InvalidClientId)?;
    let chain_id = ChainId::from_str(&client_state.chain_id)
        .map_err(|_| UpdateClientError::InvalidChainId(client_state.chain_id.clone()))?;

    let trust_threshold: TmTrustThreshold = client_state.trust_level.clone().into();

    let options = Options {
        trust_threshold,
        trusting_period: Duration::from_secs(client_state.trusting_period_seconds),
        clock_drift: Duration::from_secs(client_state.max_clock_drift_seconds),
    };

    let mut ctx = types::validation::ClientValidationCtx::new(time);

    ctx.insert_trusted_consensus_state(
        client_id.clone(),
        proposed_header.trusted_height.revision_number(),
        proposed_header.trusted_height.revision_height(),
        trusted_consensus_state,
    );

    verify_header::<_, H>(
        &ctx,
        &proposed_header,
        &client_id,
        &chain_id,
        &options,
        verifier,
    )
    .map_err(|_| UpdateClientError::HeaderVerificationFailed)?;

    let trusted_height = proposed_header.trusted_height;
    let latest_height = proposed_header.height();
    let new_consensus_state = ConsensusState::from(proposed_header);

    Ok(UpdateClientOutput {
        new_consensus_state,
        latest_height,
        trusted_height,
    })
}
