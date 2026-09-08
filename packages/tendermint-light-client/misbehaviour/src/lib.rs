// SPDX-License-Identifier: Apache-2.0

//! The crate that contains the types and utilities for `tendermint-light-client-misbehaviour`
//! program.
#![deny(missing_docs, clippy::nursery, clippy::pedantic, warnings)]
#![cfg_attr(not(test), deny(unused_crate_dependencies))]

use sha2 as _;

use ibc_client_tendermint::client_state::{
    check_for_misbehaviour_on_misbehavior, verify_misbehaviour,
};
use ibc_client_tendermint::types::{ConsensusState, Misbehaviour, TENDERMINT_CLIENT_TYPE};
use ibc_core_client_types::Height;
use ibc_core_host_types::identifiers::{ChainId, ClientId};
use std::time::Duration;
use tendermint::{crypto::Sha256, merkle::MerkleHash};
use tendermint_light_client_update_client::types::validation::ClientValidationCtx;
pub use tendermint_light_client_update_client::{ClientState, TrustThreshold};
use tendermint_light_client_verifier::{
    options::Options, types::TrustThreshold as TmTrustThreshold,
};

#[cfg(not(feature = "solana"))]
use tendermint_light_client_verifier::ProdVerifier;

/// Output from misbehaviour verification
#[derive(Clone, Debug)]
pub struct MisbehaviourOutput {
    /// The trusted height of header 1
    pub trusted_height_1: Height,
    /// The trusted height of header 2
    pub trusted_height_2: Height,
    /// The trusted consensus state of header 1
    pub trusted_consensus_state_1: ConsensusState,
    /// The trusted consensus state of header 2
    pub trusted_consensus_state_2: ConsensusState,
    /// The time which the misbehaviour was verified in unix nanoseconds
    pub time: u128,
}

/// Error type for misbehaviour detection
#[derive(Debug, thiserror::Error)]
pub enum MisbehaviourError {
    /// Invalid client ID
    #[error("invalid client ID")]
    InvalidClientId,
    /// Invalid chain ID
    #[error("invalid chain ID: {0}")]
    InvalidChainId(String),
    /// Chain ID mismatch
    #[error("chain ID mismatch: client state chain ID does not match misbehaviour header")]
    ChainIdMismatch,
    /// Misbehaviour verification failed
    #[error("misbehaviour verification failed")]
    MisbehaviourVerificationFailed,
    /// Check for misbehaviour failed
    #[error("check for misbehaviour failed")]
    CheckForMisbehaviourFailed,
    /// Misbehaviour is not detected
    #[error("misbehaviour is not detected")]
    MisbehaviourNotDetected,
}

/// IBC light client misbehaviour check
///
/// # Errors
///
/// Returns `MisbehaviourError::InvalidClientId` if client ID creation fails.
/// Returns `MisbehaviourError::InvalidChainId` if chain ID is invalid.
/// Returns `MisbehaviourError::ChainIdMismatch` if chain ID doesn't match between client state and misbehaviour header.
/// Returns `MisbehaviourError::MisbehaviourVerificationFailed` if misbehaviour verification fails.
/// Returns `MisbehaviourError::CheckForMisbehaviourFailed` if misbehaviour check fails.
/// Returns `MisbehaviourError::MisbehaviourNotDetected` if no misbehaviour is detected.
#[cfg(not(feature = "solana"))]
pub fn check_for_misbehaviour(
    client_state: &ClientState,
    misbehaviour: &Misbehaviour,
    trusted_consensus_state_1: ConsensusState,
    trusted_consensus_state_2: ConsensusState,
    time: u128,
) -> Result<MisbehaviourOutput, MisbehaviourError> {
    check_for_misbehaviour_impl::<_, sha2::Sha256>(
        client_state,
        misbehaviour,
        trusted_consensus_state_1,
        trusted_consensus_state_2,
        time,
        &ProdVerifier::default(),
    )
}

/// IBC light client misbehaviour check with Solana signature verification
///
/// # Errors
///
/// Returns `MisbehaviourError::InvalidClientId` if client ID creation fails.
/// Returns `MisbehaviourError::InvalidChainId` if chain ID is invalid.
/// Returns `MisbehaviourError::ChainIdMismatch` if chain ID doesn't match between client state and misbehaviour header.
/// Returns `MisbehaviourError::MisbehaviourVerificationFailed` if misbehaviour verification fails.
/// Returns `MisbehaviourError::CheckForMisbehaviourFailed` if misbehaviour check fails.
/// Returns `MisbehaviourError::MisbehaviourNotDetected` if no misbehaviour is detected.
#[cfg(feature = "solana")]
pub fn check_for_misbehaviour<'a>(
    client_state: &ClientState,
    misbehaviour: &Misbehaviour,
    trusted_consensus_state_1: ConsensusState,
    trusted_consensus_state_2: ConsensusState,
    time: u128,
    verification_accounts: &'a [anchor_lang::prelude::AccountInfo<'a>],
    program_id: &'a anchor_lang::prelude::Pubkey,
) -> Result<MisbehaviourOutput, MisbehaviourError> {
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
    check_for_misbehaviour_impl::<_, tendermint_light_client_solana::SolanaSha256>(
        client_state,
        misbehaviour,
        trusted_consensus_state_1,
        trusted_consensus_state_2,
        time,
        &verifier,
    )
}

fn check_for_misbehaviour_impl<V, H>(
    client_state: &ClientState,
    misbehaviour: &Misbehaviour,
    trusted_consensus_state_1: ConsensusState,
    trusted_consensus_state_2: ConsensusState,
    time: u128,
    verifier: &V,
) -> Result<MisbehaviourOutput, MisbehaviourError>
where
    V: tendermint_light_client_verifier::Verifier,
    H: MerkleHash + Sha256 + Default,
{
    let client_id =
        ClientId::new(TENDERMINT_CLIENT_TYPE, 0).map_err(|_| MisbehaviourError::InvalidClientId)?;
    let chain_id = ChainId::new(&client_state.chain_id)
        .map_err(|_| MisbehaviourError::InvalidChainId(client_state.chain_id.clone()))?;

    if client_state.chain_id
        != misbehaviour
            .header1()
            .signed_header
            .header
            .chain_id
            .to_string()
    {
        return Err(MisbehaviourError::ChainIdMismatch);
    } // header2 is checked by `verify_misbehaviour`

    // Insert the two trusted consensus states into the trusted consensus state map that exists in the ClientValidationContext that is expected by verifyMisbehaviour
    // Since we are mocking the existence of prior trusted consensus states, we are only filling in the two consensus states that are passed in into the map
    let mut ctx = ClientValidationCtx::new(time);

    ctx.insert_trusted_consensus_state(
        client_id.clone(),
        misbehaviour.header1().trusted_height.revision_number(),
        misbehaviour.header1().trusted_height.revision_height(),
        &trusted_consensus_state_1,
    );
    ctx.insert_trusted_consensus_state(
        client_id.clone(),
        misbehaviour.header2().trusted_height.revision_number(),
        misbehaviour.header2().trusted_height.revision_height(),
        &trusted_consensus_state_2,
    );

    let trust_threshold: TmTrustThreshold = client_state.trust_level.clone().into();

    let options = Options {
        trust_threshold,
        trusting_period: Duration::from_secs(client_state.trusting_period_seconds),
        clock_drift: Duration::from_secs(client_state.max_clock_drift_seconds),
    };

    // Call into ibc-rs verify_misbehaviour function to verify that both headers are valid given their respective trusted consensus states
    verify_misbehaviour::<_, H>(
        &ctx,
        misbehaviour,
        &client_id,
        &chain_id,
        &options,
        verifier,
    )
    .map_err(|_| MisbehaviourError::MisbehaviourVerificationFailed)?;

    // Call into ibc-rs check_for_misbehaviour_on_misbehaviour method to ensure that the misbehaviour is valid
    // i.e. the headers are same height but different commits, or headers are not monotonically increasing in time
    let is_misbehaviour =
        check_for_misbehaviour_on_misbehavior(misbehaviour.header1(), misbehaviour.header2())
            .map_err(|_| MisbehaviourError::CheckForMisbehaviourFailed)?;

    if !is_misbehaviour {
        return Err(MisbehaviourError::MisbehaviourNotDetected);
    }

    // The prover takes in the trusted headers as an input but does not maintain its own internal state
    // Thus, the verifier must ensure that the trusted headers that were used in the proof are trusted consensus
    // states stored in its own internal state before it can accept the misbehaviour proof as valid.
    Ok(MisbehaviourOutput {
        trusted_height_1: misbehaviour.header1().trusted_height,
        trusted_height_2: misbehaviour.header2().trusted_height,
        trusted_consensus_state_1,
        trusted_consensus_state_2,
        time,
    })
}
