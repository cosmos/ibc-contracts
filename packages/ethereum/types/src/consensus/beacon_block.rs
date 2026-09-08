// SPDX-License-Identifier: Apache-2.0

//! This module defines types related to beacon's block api endpoints.

use alloy_primitives::{Address, Bloom, Bytes, B256, U256};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

use super::{bls::BlsSignature, sync_committee::SyncAggregate};

/// A beacon block
#[serde_as]
#[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct BeaconBlock {
    /// The beacon block message
    pub message: BeaconBlockMessage,
    /// The beacon block signature
    pub signature: BlsSignature,
}

/// A beacon block message
#[serde_as]
#[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct BeaconBlockMessage {
    /// The beacon block slot
    #[serde_as(as = "DisplayFromStr")]
    pub slot: u64,
    /// The beacon block proposer index
    #[serde_as(as = "DisplayFromStr")]
    pub proposer_index: u64,
    /// The beacon block parent root
    pub parent_root: B256,
    /// The beacon block state root
    pub state_root: B256,
    /// The beacon block body
    pub body: Body,
}

/// The body of a beacon block
#[serde_as]
#[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct Body {
    /// The sync aggregate of the beacon block
    pub sync_aggregate: SyncAggregate,
    /// The execution payload for the beacon block
    pub execution_payload: ExecutionPayload,
    // Values not represented in this type:
    // - randao_reveal
    // - eth1_data
    // - graffiti
    // - proposer_slashings
    // - attester_slashings
    // - attestations
    // - deposits
    // - voluntary_exits
    // - bls_to_execution_changes
    // - blob_kzg_commitments
    // - execution_requests
}

/// Execution payload for a beacon block
#[serde_as]
#[derive(Default, Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct ExecutionPayload {
    /// The parent hash of the execution payload header
    pub parent_hash: B256,
    /// Block fee recipient
    pub fee_recipient: Address,
    /// The state root
    pub state_root: B256,
    /// The root of the receipts trie
    pub receipts_root: B256,
    /// The logs bloom filter
    pub logs_bloom: Bloom,
    /// The previous Randao value, used to compute the randomness on the execution layer.
    pub prev_randao: B256,
    /// The block number of the execution payload
    #[serde_as(as = "DisplayFromStr")]
    pub block_number: u64,
    /// Execution block gas limit
    #[serde_as(as = "DisplayFromStr")]
    pub gas_limit: u64,
    /// Execution block gas used
    #[serde_as(as = "DisplayFromStr")]
    pub gas_used: u64,
    /// The timestamp of the execution payload
    #[serde_as(as = "DisplayFromStr")]
    pub timestamp: u64,
    /// The extra data of the execution payload
    pub extra_data: Bytes,
    /// Block base fee per gas
    pub base_fee_per_gas: U256,
    /// The block hash
    pub block_hash: B256,
    /// Blob gas used (new in Deneb)
    #[serde_as(as = "DisplayFromStr")]
    pub blob_gas_used: u64,
    /// Excess blob gas (new in Deneb)
    #[serde_as(as = "DisplayFromStr")]
    pub excess_blob_gas: u64,
    // Values not represented in this type:
    // - transactions
    // - withdrawals
}
