// SPDX-License-Identifier: Apache-2.0

//! This module defines [`EthereumIBCError`].

use alloy_primitives::B256;
use ethereum_types::consensus::bls::BlsPublicKey;

#[derive(thiserror::Error, Debug, Clone, PartialEq)]
#[allow(missing_docs, clippy::module_name_repetitions)]
pub enum EthereumIBCError {
    #[error("invalid path length, expected {expected} but found {found}")]
    InvalidPathLength { expected: usize, found: usize },

    #[error("unable to decode storage proof")]
    StorageProofDecode,

    #[error("invalid commitment key, expected ({0}) but found ({1})")]
    InvalidCommitmentKey(String, String),

    #[error("expected value ({expected}) and stored value ({actual}) don't match", 
        expected = hex::encode(expected),
        actual = hex::encode(actual)
    )]
    StoredValueMistmatch { expected: Vec<u8>, actual: Vec<u8> },

    #[error("verify storage proof error: {0}")]
    VerifyStorageProof(String),

    #[error("insufficient number of sync committee participants ({0})")]
    InsufficientSyncCommitteeParticipants(u64),

    #[error("insufficient number of sync committee addresses ({found}) but expected ({expected})")]
    InsufficientSyncCommitteeLength { expected: u64, found: u64 },

    #[error("unsupported fork version, must be electra or later")]
    MustBeElectraOrLater,

    #[error(transparent)]
    InvalidMerkleBranch(#[from] Box<InvalidMerkleBranch>), // boxed to decrease enum size

    #[error(
        "invalid normalized merkle branch, expected {num_extra} empty bytes in {normalized_branch}",
        normalized_branch = .normalized_branch.iter().map(ToString::to_string).collect::<Vec<_>>().join(", ")
    )]
    InvalidNormalizedMerkleBranch {
        num_extra: usize,
        normalized_branch: Vec<B256>,
    },

    #[error("finalized slot cannot be the genesis slot")]
    FinalizedSlotIsGenesis,

    #[error(
        "update signature slot {update_signature_slot} is more recent than the \
        calculated current slot {current_slot}"
    )]
    UpdateSignatureSlotMoreRecentThanCurrentSlot {
        current_slot: u64,
        update_signature_slot: u64,
    },

    #[error(
        "(update_signature_slot > update_attested_slot >= update_finalized_slot) must hold, \
        found: ({update_signature_slot} > {update_attested_slot} >= {update_finalized_slot})"
    )]
    InvalidSlots {
        update_signature_slot: u64,
        update_attested_slot: u64,
        update_finalized_slot: u64,
    },

    #[error(
        "signature period ({signature_period}) must be equal to `store_period` \
        ({stored_period}) or `store_period + 1` when the next sync committee is stored"
    )]
    InvalidSignaturePeriodWhenNextSyncCommitteeExists {
        signature_period: u64,
        stored_period: u64,
    },

    #[error(
        "signature period ({signature_period}) must be equal to `store_period` \
        ({stored_period}) when the next sync committee is not stored"
    )]
    InvalidSignaturePeriodWhenNextSyncCommitteeDoesNotExist {
        signature_period: u64,
        stored_period: u64,
    },

    #[error(
        "irrelevant update since the order of the slots in the update data, and stored data is not correct. \
        either the update_attested_slot (found {update_attested_slot}) must be > the trusted_finalized_slot \
        (found {trusted_finalized_slot}) or if it is not, then the update_attested_period \
        (found {update_attested_period}) must be the same as the store_period (found {stored_period}) and \
        the update_sync_committee must be set (was set: {update_sync_committee_is_set}) and the trusted \
        next_sync_committee must be unset (was set: {trusted_next_sync_committee_is_set})"
    )]
    IrrelevantUpdate {
        update_attested_slot: u64,
        trusted_finalized_slot: u64,
        update_attested_period: u64,
        stored_period: u64,
        update_sync_committee_is_set: bool,
        trusted_next_sync_committee_is_set: bool,
    },

    #[error(
        "next sync committee ({found}) does not match with the one in the current state ({expected})"
    )]
    NextSyncCommitteeMismatch {
        expected: BlsPublicKey,
        found: BlsPublicKey,
    },

    #[error(
        "current sync committee ({found}) does not match with the one in the current state ({expected})"
    )]
    CurrenttSyncCommitteeMismatch {
        expected: BlsPublicKey,
        found: BlsPublicKey,
    },

    #[error("aggregate public key mismatch: expected {expected} but found {found}")]
    AggregatePubkeyMismatch {
        expected: BlsPublicKey,
        found: BlsPublicKey,
    },

    #[error(
        "expected current sync committee to be provided since `update_period == current_period`"
    )]
    ExpectedCurrentSyncCommittee,

    #[error("expected next sync committee to be provided for signature verification`")]
    ExpectedNextSyncCommittee,

    #[error("expected next sync committee to be provided in the update since `update_period > current_period`")]
    ExpectedNextSyncCommitteeUpdate,

    #[error("expected next sync committee to be known and stored in state")]
    NextSyncCommitteeUnknown,

    #[error("unexpected next sync committee in the update")]
    UnexpectedNextSyncCommittee,

    #[error("bls aggregate error: {0}")]
    BlsAggregateError(String),

    #[error("fast aggregate verify error: {0}")]
    FastAggregateVerifyError(String),

    #[error("not enough signatures")]
    NotEnoughSignatures,

    #[error("failed to verify finalized_header is finalized: {0}")]
    ValidateFinalizedHeaderFailed(#[source] Box<Self>),

    #[error("failed to verify next sync committee against attested header: {0}")]
    ValidateNextSyncCommitteeFailed(#[source] Box<Self>),

    #[error("client's store period must be equal to update's finalized period")]
    StorePeriodMustBeEqualToFinalizedPeriod,

    #[error("failed to compute slot at timestamp with  \
        (timestamp ({timestamp}) - genesis ({genesis})) / seconds_per_slot ({seconds_per_slot}) + genesis_slot ({genesis_slot})"
    )]
    FailedToComputeSlotAtTimestamp {
        timestamp: u64,
        genesis: u64,
        seconds_per_slot: u64,
        genesis_slot: u64,
    },

    #[error("conflicting updates are for different slots: {0} != {1}")]
    MisbehaviourSlotMismatch(u64, u64),

    #[error("storage roots are not conflicting: {0} == {0}")]
    MisbehaviourStorageRootsMatch(B256),

    #[error(
        "update must be against a previous consensus state: \
        stored consensus state slot: {consensus_state_slot}, \
        update finalized header slot: {update_finalized_slot}"
    )]
    InvalidUpdateSlot {
        consensus_state_slot: u64,
        update_finalized_slot: u64,
    },

    #[error(
        "client and consensus slot mismatch: \
        client state slot: {client_state_slot}, \
        consensus state slot: {consensus_state_slot}"
    )]
    ClientAndConsensusSlotMismatch {
        client_state_slot: u64,
        consensus_state_slot: u64,
    },
}

#[derive(Debug, PartialEq, Eq, Clone, thiserror::Error)]
#[error("invalid merkle branch \
    (leaf: {leaf}, branch: [{branch}], \
    depth: {depth}, index: {index}, root: {root}, found: {found})",
    branch = .branch.iter().map(ToString::to_string).collect::<Vec<_>>().join(", ")
)]
#[allow(missing_docs)]
pub struct InvalidMerkleBranch {
    pub leaf: B256,
    pub branch: Vec<B256>,
    pub depth: usize,
    pub index: u64,
    pub root: B256,
    pub found: B256,
}

impl EthereumIBCError {
    /// Constructs an [`EthereumIBCError::InvalidMerkleBranch`] variant.
    #[must_use]
    pub fn invalid_merkle_branch(
        leaf: B256,
        branch: Vec<B256>,
        depth: usize,
        index: u64,
        root: B256,
        found: B256,
    ) -> Self {
        Self::InvalidMerkleBranch(Box::new(InvalidMerkleBranch {
            leaf,
            branch,
            depth,
            index,
            root,
            found,
        }))
    }
}
