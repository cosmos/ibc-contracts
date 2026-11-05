// SPDX-License-Identifier: Apache-2.0

//! This module defines types related to beacon's light client functionality.

use alloy_primitives::{Address, Bloom, Bytes, B256, U256};
use schemars::{gen::SchemaGenerator, schema::Schema, JsonSchema};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use tree_hash_derive::TreeHash;

use super::{
    merkle::{
        floorlog2, EXECUTION_BLOCK_HASH_GINDEX_GLOAS, EXECUTION_PAYLOAD_GINDEX,
        FINALIZED_ROOT_GINDEX_ELECTRA, FINALIZED_ROOT_GINDEX_GLOAS,
        NEXT_SYNC_COMMITTEE_GINDEX_ELECTRA, NEXT_SYNC_COMMITTEE_GINDEX_GLOAS,
    },
    sync_committee::{SyncAggregate, SyncCommittee},
};

/// A light client update in the format of the fork of its attested header
#[derive(Serialize, Deserialize, JsonSchema, PartialEq, Eq, Clone, Debug)]
#[serde(untagged)]
#[allow(clippy::module_name_repetitions, clippy::large_enum_variant)]
pub enum LightClientUpdate {
    /// An update with a Gloas attested header
    Gloas(LightClientUpdateGloas),
    /// An update with a Fulu attested header
    Fulu(LightClientUpdateFulu),
}

/// A light client update with a Gloas attested header. Its finalized header can be from an earlier
/// fork, which is why the header doesn't determine the gindex of its execution branch.
#[allow(clippy::module_name_repetitions)]
pub type LightClientUpdateGloas = GenericLightClientUpdate<
    LightClientHeaderGloas,
    MerkleBranch<[B256; floorlog2(NEXT_SYNC_COMMITTEE_GINDEX_GLOAS)]>,
    MerkleBranch<[B256; floorlog2(FINALIZED_ROOT_GINDEX_GLOAS)]>,
>;

/// A light client update with a Fulu attested header
#[allow(clippy::module_name_repetitions)]
pub type LightClientUpdateFulu = GenericLightClientUpdate<
    LightClientHeaderFulu,
    MerkleBranch<[B256; floorlog2(NEXT_SYNC_COMMITTEE_GINDEX_ELECTRA)]>,
    MerkleBranch<[B256; floorlog2(FINALIZED_ROOT_GINDEX_ELECTRA)]>,
>;

/// A light client update, generic over the format of its headers and its fixed length branches.
/// Finality updates have the same format without the next sync committee.
#[serde_as]
#[derive(Serialize, Deserialize, JsonSchema, PartialEq, Eq, Clone, Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct GenericLightClientUpdate<H, NextSyncCommitteeBranch, FinalityBranch> {
    /// Header attested to by the sync committee
    pub attested_header: H,
    /// Next sync committee corresponding to `attested_header.state_root`
    pub next_sync_committee: Option<SyncCommittee>,
    /// The branch of the next sync committee
    pub next_sync_committee_branch: Option<NextSyncCommitteeBranch>,
    /// Finalized header corresponding to `attested_header.state_root`
    pub finalized_header: H,
    /// Branch of the finalized header
    pub finality_branch: FinalityBranch,
    /// Sync committee aggregate signature
    pub sync_aggregate: SyncAggregate,
    /// Slot at which the aggregate signature was created (untrusted)
    #[serde_as(as = "DisplayFromStr")]
    #[schemars(with = "String")]
    pub signature_slot: u64,
}

/// A merkle branch of a fixed length, which is enforced when decoding it
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, Default)]
#[serde(transparent)]
pub struct MerkleBranch<T>(pub T);

impl<T> JsonSchema for MerkleBranch<T> {
    fn schema_name() -> String {
        "MerkleBranch".to_string()
    }

    fn json_schema(generator: &mut SchemaGenerator) -> Schema {
        <Vec<String>>::json_schema(generator)
    }
}

impl LightClientUpdate {
    /// Returns the beacon block header attested to by the sync committee
    #[must_use]
    pub const fn attested_beacon(&self) -> &BeaconBlockHeader {
        match self {
            Self::Gloas(update) => &update.attested_header.beacon,
            Self::Fulu(update) => &update.attested_header.beacon,
        }
    }

    /// Returns the finalized beacon block header
    #[must_use]
    pub const fn finalized_beacon(&self) -> &BeaconBlockHeader {
        match self {
            Self::Gloas(update) => &update.finalized_header.beacon,
            Self::Fulu(update) => &update.finalized_header.beacon,
        }
    }

    /// Returns the hash of the execution block the attested header commits to
    #[must_use]
    pub const fn attested_block_hash(&self) -> B256 {
        match self {
            Self::Gloas(update) => update.attested_header.execution_block_hash,
            Self::Fulu(update) => update.attested_header.execution.block_hash,
        }
    }

    /// Returns the next sync committee, if the update contains it
    #[must_use]
    pub const fn next_sync_committee(&self) -> Option<&SyncCommittee> {
        match self {
            Self::Gloas(update) => update.next_sync_committee.as_ref(),
            Self::Fulu(update) => update.next_sync_committee.as_ref(),
        }
    }

    /// Returns whether the update contains a branch for the next sync committee
    #[must_use]
    pub const fn has_next_sync_committee_branch(&self) -> bool {
        match self {
            Self::Gloas(update) => update.next_sync_committee_branch.is_some(),
            Self::Fulu(update) => update.next_sync_committee_branch.is_some(),
        }
    }

    /// Returns the sync committee aggregate signature
    #[must_use]
    pub const fn sync_aggregate(&self) -> &SyncAggregate {
        match self {
            Self::Gloas(update) => &update.sync_aggregate,
            Self::Fulu(update) => &update.sync_aggregate,
        }
    }

    /// Returns the slot at which the aggregate signature was created (untrusted)
    #[must_use]
    pub const fn signature_slot(&self) -> u64 {
        match self {
            Self::Gloas(update) => update.signature_slot,
            Self::Fulu(update) => update.signature_slot,
        }
    }
}

/// The header of a light client, in the format of the light client data that contains it
#[derive(Serialize, Deserialize, JsonSchema, PartialEq, Eq, Clone, Debug)]
#[serde(untagged)]
#[allow(clippy::module_name_repetitions, clippy::large_enum_variant)]
pub enum LightClientHeader {
    /// A header in Gloas light client data
    Gloas(LightClientHeaderGloas),
    /// A header in Fulu light client data
    Fulu(LightClientHeaderFulu),
}

impl LightClientHeader {
    /// Returns the beacon block header
    #[must_use]
    pub const fn beacon(&self) -> &BeaconBlockHeader {
        match self {
            Self::Gloas(header) => &header.beacon,
            Self::Fulu(header) => &header.beacon,
        }
    }

    /// Returns the hash of the execution block this header commits to
    #[must_use]
    pub const fn block_hash(&self) -> B256 {
        match self {
            Self::Gloas(header) => header.execution_block_hash,
            Self::Fulu(header) => header.execution.block_hash,
        }
    }
}

/// The header of a light client in Gloas light client data
#[derive(Serialize, Deserialize, JsonSchema, PartialEq, Eq, Clone, Debug, Default)]
#[allow(clippy::module_name_repetitions)]
pub struct LightClientHeaderGloas {
    /// The beacon block header
    pub beacon: BeaconBlockHeader,
    /// The hash of the execution block
    #[schemars(with = "String")]
    pub execution_block_hash: B256,
    /// The execution branch, normalized to the Gloas depth for headers of pre-Gloas blocks
    #[schemars(with = "Vec<String>")]
    pub execution_branch: [B256; floorlog2(EXECUTION_BLOCK_HASH_GINDEX_GLOAS)],
}

/// The header of a light client in Fulu light client data
#[derive(Serialize, Deserialize, JsonSchema, PartialEq, Eq, Clone, Debug, Default)]
#[allow(clippy::module_name_repetitions)]
pub struct LightClientHeaderFulu {
    /// The beacon block header
    pub beacon: BeaconBlockHeader,
    /// The execution payload header
    pub execution: ExecutionPayloadHeader,
    /// The execution branch
    #[schemars(with = "Vec<String>")]
    pub execution_branch: [B256; floorlog2(EXECUTION_PAYLOAD_GINDEX)],
}

/// The beacon block header
#[serde_as]
#[derive(Serialize, Deserialize, JsonSchema, PartialEq, Eq, Clone, Debug, Default, TreeHash)]
pub struct BeaconBlockHeader {
    /// The slot to which this block corresponds
    #[serde_as(as = "DisplayFromStr")]
    #[schemars(with = "String")]
    pub slot: u64,
    /// The index of validator in validator registry
    #[serde_as(as = "DisplayFromStr")]
    #[schemars(with = "String")]
    pub proposer_index: u64,
    /// The signing merkle root of the parent `BeaconBlock`
    #[schemars(with = "String")]
    pub parent_root: B256,
    /// The tree hash merkle root of the `BeaconState` for the `BeaconBlock`
    #[schemars(with = "String")]
    pub state_root: B256,
    /// The tree hash merkle root of the `BeaconBlockBody` for the `BeaconBlock`
    #[schemars(with = "String")]
    pub body_root: B256,
}

/// Header to track the execution block
#[serde_as]
#[derive(Serialize, Deserialize, JsonSchema, PartialEq, Eq, Clone, Debug, Default, TreeHash)]
pub struct ExecutionPayloadHeader {
    /// The parent hash of the execution payload header
    #[schemars(with = "String")]
    pub parent_hash: B256,
    /// Block fee recipient
    #[schemars(with = "String")]
    pub fee_recipient: Address,
    /// The state root
    #[schemars(with = "String")]
    pub state_root: B256,
    /// The root of the receipts trie
    #[schemars(with = "String")]
    pub receipts_root: B256,
    /// The logs bloom filter
    #[schemars(with = "String")]
    pub logs_bloom: Bloom,
    /// The previous Randao value, used to compute the randomness on the execution layer.
    #[schemars(with = "String")]
    pub prev_randao: B256,
    /// The block number of the execution payload
    #[serde_as(as = "DisplayFromStr")]
    #[schemars(with = "String")]
    pub block_number: u64,
    /// Execution block gas limit
    #[serde_as(as = "DisplayFromStr")]
    #[schemars(with = "String")]
    pub gas_limit: u64,
    /// Execution block gas used
    #[serde_as(as = "DisplayFromStr")]
    #[schemars(with = "String")]
    pub gas_used: u64,
    /// The timestamp of the execution payload
    #[serde_as(as = "DisplayFromStr")]
    #[schemars(with = "String")]
    pub timestamp: u64,
    /// The extra data of the execution payload
    #[schemars(with = "String")]
    pub extra_data: Bytes,
    /// Block base fee per gas
    #[schemars(with = "String")]
    pub base_fee_per_gas: U256,
    /// The block hash
    #[schemars(with = "String")]
    pub block_hash: B256,
    /// SSZ hash tree root of the transaction list
    #[schemars(with = "String")]
    pub transactions_root: B256,
    /// Tree root of the withdrawals list
    #[schemars(with = "String")]
    pub withdrawals_root: B256,
    /// Blob gas used (new in Deneb)
    #[serde_as(as = "DisplayFromStr")]
    #[schemars(with = "String")]
    pub blob_gas_used: u64,
    /// Excess blob gas (new in Deneb)
    #[serde_as(as = "DisplayFromStr")]
    #[schemars(with = "String")]
    pub excess_blob_gas: u64,
}
