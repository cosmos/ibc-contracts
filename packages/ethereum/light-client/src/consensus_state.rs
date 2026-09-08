// SPDX-License-Identifier: Apache-2.0

//! This module defines [`ConsensusState`] and [`TrustedConsensusState`].

use alloy_primitives::B256;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use ethereum_types::consensus::sync_committee::{SummarizedSyncCommittee, SyncCommittee};

use crate::{
    client_state::ClientState, error::EthereumIBCError, header::ActiveSyncCommittee,
    verify::BlsVerify,
};

/// The consensus state of the Ethereum light client corresponding to a finalized header
#[derive(Serialize, Deserialize, JsonSchema, PartialEq, Eq, Debug, Clone)]
pub struct ConsensusState {
    /// The slot number of the finalized header
    pub slot: u64,
    /// The state merkle root of the finalized header
    #[schemars(with = "String")]
    pub state_root: B256,
    /// The execution timestamp of the finalized header
    pub timestamp: u64,
    /// aggregate public key of current sync committee at the finalized header
    pub current_sync_committee: SummarizedSyncCommittee,
    /// aggregate public key of next sync committee at the finalized header if known
    pub next_sync_committee: Option<SummarizedSyncCommittee>,
}

/// The trusted consensus state of the Ethereum light client
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct TrustedConsensusState {
    /// The consensus state
    pub state: ConsensusState,
    /// Full sync committee data which corresponds to the aggregate key that we
    /// store at the client.
    ///
    /// This sync committee can either be the current sync committee or the next sync
    /// committee. That's because the verifier uses next or current sync committee's
    /// public keys to verify the signature against.
    sync_committee: ActiveSyncCommittee,
}

impl TrustedConsensusState {
    /// Creates a new trusted consensus state
    /// # Errors
    /// Returns an error if the untrusted sync committee does not match the trusted state
    pub fn new<V: BlsVerify>(
        client_state: &ClientState,
        trusted_state: ConsensusState,
        untrusted_sync_committee: ActiveSyncCommittee,
        bls_verifier: &V,
    ) -> Result<Self, EthereumIBCError> {
        let full_committee = match untrusted_sync_committee {
            ActiveSyncCommittee::Current(ref committee) => {
                ensure!(
                    committee.to_summarized_sync_committee()
                        == trusted_state.current_sync_committee,
                    EthereumIBCError::CurrenttSyncCommitteeMismatch {
                        expected: trusted_state.current_sync_committee.aggregate_pubkey,
                        found: committee.aggregate_pubkey
                    }
                );
                committee
            }
            ActiveSyncCommittee::Next(ref committee) => {
                let trusted_next_sync_committee = trusted_state
                    .next_sync_committee
                    .as_ref()
                    .ok_or(EthereumIBCError::NextSyncCommitteeUnknown)?;
                ensure!(
                    committee.to_summarized_sync_committee() == *trusted_next_sync_committee,
                    EthereumIBCError::NextSyncCommitteeMismatch {
                        expected: trusted_next_sync_committee.aggregate_pubkey,
                        found: committee.aggregate_pubkey
                    }
                );
                committee
            }
        };

        // Verify the sync committee size
        ensure!(
            full_committee.pubkeys.len() as u64 == client_state.sync_committee_size,
            EthereumIBCError::InsufficientSyncCommitteeLength {
                expected: client_state.sync_committee_size,
                found: full_committee.pubkeys.len() as u64
            }
        );

        let aggregate_pubkey = bls_verifier
            .aggregate(&full_committee.pubkeys)
            .map_err(|e| EthereumIBCError::BlsAggregateError(e.to_string()))?;
        ensure!(
            aggregate_pubkey == full_committee.aggregate_pubkey,
            EthereumIBCError::AggregatePubkeyMismatch {
                expected: aggregate_pubkey,
                found: full_committee.aggregate_pubkey
            }
        );

        Ok(Self {
            state: trusted_state,
            sync_committee: untrusted_sync_committee,
        })
    }

    /// Returns the finalized slot of the trusted consensus state
    #[must_use]
    pub const fn finalized_slot(&self) -> u64 {
        self.state.slot
    }

    /// Returns the current slot of the trusted consensus state if it is available
    #[must_use]
    pub const fn current_sync_committee(&self) -> Option<&SyncCommittee> {
        if let ActiveSyncCommittee::Current(committee) = &self.sync_committee {
            Some(committee)
        } else {
            None
        }
    }

    /// Returns the next sync committee if it is available
    #[must_use]
    pub const fn next_sync_committee(&self) -> Option<&SyncCommittee> {
        if let ActiveSyncCommittee::Next(committee) = &self.sync_committee {
            Some(committee)
        } else {
            None
        }
    }
}
