// SPDX-License-Identifier: Apache-2.0

//! The crate that contains the types and utilities for `tendermint-light-client-uc-and-membership` program.
#![deny(missing_docs, clippy::nursery, clippy::pedantic, warnings)]
#![cfg_attr(not(test), deny(unused_crate_dependencies))]

use ibc_client_tendermint::types::{ConsensusState, Header};
use ibc_core_commitment_types::merkle::MerkleProof;
use tendermint_light_client_membership::KVPair;
use tendermint_light_client_update_client::{ClientState, UpdateClientOutput};

/// Output from combined update client and membership verification
#[derive(Clone, Debug)]
pub struct UcAndMembershipOutput {
    /// Output from update client verification
    pub update_output: UpdateClientOutput,
}

/// Error type for combined update client and membership
#[derive(Debug, thiserror::Error)]
pub enum UcAndMembershipError {
    /// Invalid app hash
    #[error("invalid app hash: expected 32 bytes, got {0} bytes")]
    InvalidAppHash(usize),
    /// Update client error
    #[error("update client error: {0}")]
    UpdateClient(#[from] tendermint_light_client_update_client::UpdateClientError),
    /// Membership error
    #[error("membership error: {0}")]
    Membership(#[from] tendermint_light_client_membership::MembershipError),
}

/// IBC light client combined update of client and membership verification
///
/// # Errors
///
/// Returns `UcAndMembershipError::InvalidAppHash` if the app hash is not 32 bytes.
/// Returns `UcAndMembershipError::UpdateClient` if update client verification fails.
/// Returns `UcAndMembershipError::Membership` if membership verification fails.
pub fn update_client_and_membership(
    client_state: &ClientState,
    trusted_consensus_state: &ConsensusState,
    proposed_header: Header,
    time: u128,
    request: &[(KVPair, MerkleProof)],
) -> Result<UcAndMembershipOutput, UcAndMembershipError> {
    let app_hash_bytes = proposed_header.signed_header.header().app_hash.as_bytes();
    let app_hash: [u8; 32] = app_hash_bytes
        .try_into()
        .map_err(|_| UcAndMembershipError::InvalidAppHash(app_hash_bytes.len()))?;

    let uc_output = tendermint_light_client_update_client::update_client(
        client_state,
        trusted_consensus_state,
        proposed_header,
        time,
    )?;

    tendermint_light_client_membership::membership(app_hash, request.iter().cloned())?;

    Ok(UcAndMembershipOutput {
        update_output: uc_output,
    })
}
