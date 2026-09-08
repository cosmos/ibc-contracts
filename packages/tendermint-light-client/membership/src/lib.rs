// SPDX-License-Identifier: Apache-2.0

//! The crate that contains the types and utilities for `tendermint-light-client-membership` program.
#![deny(missing_docs, clippy::nursery, clippy::pedantic, warnings)]
#![cfg_attr(not(test), deny(unused_crate_dependencies))]

use ibc_core_commitment_types::{
    commitment::CommitmentRoot,
    merkle::{MerklePath, MerkleProof},
    specs::ProofSpecs,
};
use ibc_core_host_types::path::PathBytes;

#[cfg(feature = "solana")]
type HostFunctions = tendermint_light_client_solana::SolanaHostFunctionsManager;
#[cfg(not(feature = "solana"))]
type HostFunctions = ibc_core_commitment_types::proto::ics23::HostFunctionsManager;

/// Key-value pair for membership/non-membership proofs
#[derive(Clone, Debug)]
pub struct KVPair {
    /// Storage path segments
    pub path: Vec<Vec<u8>>,
    /// Value (empty for non-membership proofs)
    pub value: Vec<u8>,
}

impl KVPair {
    /// Create a new key-value pair
    #[must_use]
    pub const fn new(path: Vec<Vec<u8>>, value: Vec<u8>) -> Self {
        Self { path, value }
    }

    /// Check if this is a non-membership proof (empty value)
    /// NOTE: supress clippy due to compilation not stable in every target
    #[allow(clippy::missing_const_for_fn)]
    #[must_use]
    pub fn is_non_membership(&self) -> bool {
        self.value.is_empty()
    }

    /// Create a `MerklePath` from this `KVPair` path segments
    #[must_use]
    pub fn to_merkle_path(&self) -> MerklePath {
        MerklePath::new(
            self.path
                .clone()
                .iter()
                .map(PathBytes::from_bytes)
                .collect(),
        )
    }
}

/// Output for membership verification
#[derive(Clone, Debug)]
pub struct MembershipOutput {
    /// The commitment root (app hash) that was verified
    pub commitment_root: [u8; 32],
    /// The verified key-value pairs
    pub kv_pairs: Vec<KVPair>,
}

/// Error type for membership verification
#[derive(Debug, thiserror::Error)]
pub enum MembershipError {
    /// Empty request
    #[error("request must not be empty")]
    EmptyRequest,
    /// Non-membership verification failed
    #[error("non-membership verification failed")]
    NonMembershipVerificationFailed,
    /// Membership verification failed
    #[error("membership verification failed")]
    MembershipVerificationFailed,
}

/// IBC membership verification
///
/// # Errors
///
/// Returns `MembershipError::EmptyRequest` if the iterator is empty.
/// Returns `MembershipError::NonMembershipVerificationFailed` if non-membership proof verification fails.
/// Returns `MembershipError::MembershipVerificationFailed` if membership proof verification fails.
pub fn membership(
    app_hash: [u8; 32],
    request_iter: impl Iterator<Item = (KVPair, MerkleProof)>,
) -> Result<MembershipOutput, MembershipError> {
    let commitment_root = CommitmentRoot::from_bytes(&app_hash);

    let kv_pairs: Vec<KVPair> = request_iter
        .map(|(kv_pair, merkle_proof)| {
            let merkle_path = kv_pair.to_merkle_path();

            if kv_pair.is_non_membership() {
                merkle_proof
                    .verify_non_membership::<HostFunctions>(
                        &ProofSpecs::cosmos(),
                        commitment_root.clone().into(),
                        merkle_path,
                    )
                    .map_err(|_| MembershipError::NonMembershipVerificationFailed)?;
            } else {
                merkle_proof
                    .verify_membership::<HostFunctions>(
                        &ProofSpecs::cosmos(),
                        commitment_root.clone().into(),
                        merkle_path,
                        kv_pair.value.clone(),
                        0,
                    )
                    .map_err(|_| MembershipError::MembershipVerificationFailed)?;
            }

            Ok(kv_pair)
        })
        .collect::<Result<Vec<_>, _>>()?;

    if kv_pairs.is_empty() {
        return Err(MembershipError::EmptyRequest);
    }

    Ok(MembershipOutput {
        commitment_root: app_hash,
        kv_pairs,
    })
}
