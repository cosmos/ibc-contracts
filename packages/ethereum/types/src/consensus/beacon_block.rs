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
    /// The execution payload for the beacon block (removed in Gloas)
    // TODO: Remove this field once mainnet has forked to Gloas (#1252)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution_payload: Option<ExecutionPayload>,
    /// The signed execution payload bid for the beacon block (new in Gloas)
    // TODO: Remove the optionality once mainnet has forked to Gloas (#1252)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub signed_execution_payload_bid: Option<SignedExecutionPayloadBid>,
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
    // - payload_attestations
    // - parent_execution_requests
}

impl Body {
    /// Returns the hash of the execution block whose state this beacon block commits to.
    /// From Gloas onwards this is the parent of the block bid on, since the execution payload of
    /// the bid is only revealed after the beacon block.
    #[must_use]
    pub fn execution_block_hash(&self) -> Option<B256> {
        self.execution_payload
            .as_ref()
            .map(|payload| payload.block_hash)
            .or_else(|| {
                self.signed_execution_payload_bid
                    .as_ref()
                    .map(|bid| bid.message.parent_block_hash)
            })
    }
}

/// A signed execution payload bid
#[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct SignedExecutionPayloadBid {
    /// The execution payload bid
    pub message: ExecutionPayloadBid,
    // Values not represented in this type:
    // - signature
}

/// An execution payload bid
#[derive(Default, Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct ExecutionPayloadBid {
    /// The hash of the execution block the bid builds on
    pub parent_block_hash: B256,
    // Values not represented in this type:
    // - parent_block_root
    // - block_hash
    // - prev_randao
    // - fee_recipient
    // - gas_limit
    // - builder_index
    // - slot
    // - value
    // - execution_payment
    // - blob_kzg_commitments
    // - execution_requests_root
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
