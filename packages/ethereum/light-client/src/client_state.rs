// SPDX-License-Identifier: Apache-2.0

//! This module defines [`ClientState`].

use alloy_primitives::{Address, B256, U256};
use ethereum_types::consensus::fork::ForkParameters;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::EthereumIBCError;

/// The ethereum client state
#[derive(Serialize, Deserialize, JsonSchema, PartialEq, Eq, Clone, Debug, Default)]
pub struct ClientState {
    /// The chain ID
    pub chain_id: u64,
    /// The genesis validators root
    #[schemars(with = "String")]
    pub genesis_validators_root: B256,
    /// The minimum number of participants in the sync committee
    pub min_sync_committee_participants: u64,
    /// The size of the sync committee, maximum possible number of participants
    pub sync_committee_size: u64,
    /// The time of genesis (unix timestamp)
    pub genesis_time: u64,
    /// The genesis slot
    pub genesis_slot: u64,
    /// The fork parameters
    pub fork_parameters: ForkParameters,
    /// The slot duration in seconds
    pub seconds_per_slot: u64,
    /// The number of slots per epoch
    pub slots_per_epoch: u64,
    /// The number of epochs per sync committee period
    pub epochs_per_sync_committee_period: u64,
    /// The latest slot of this client
    pub latest_slot: u64,
    /// The latest execution block number, used for relayer convenience only
    pub latest_execution_block_number: u64,
    /// Whether the client is frozen
    pub is_frozen: bool,
    /// The address of the IBC contract being tracked on Ethereum
    #[schemars(with = "String")]
    pub ibc_contract_address: Address,
    /// The storage slot of the IBC commitment in the Ethereum contract
    #[schemars(with = "String")]
    pub ibc_commitment_slot: U256,
}

impl ClientState {
    /// Verifies that an epoch is within a supported fork for the light client.
    /// # Errors
    /// Returns an error if the slot is not supported.
    pub const fn verify_supported_fork_at_epoch(&self, epoch: u64) -> Result<(), EthereumIBCError> {
        if epoch < self.fork_parameters.electra.epoch {
            return Err(EthereumIBCError::MustBeElectraOrLater);
        }

        Ok(())
    }

    /// Returns the computed slot at a given `timestamp_seconds`.
    #[must_use]
    pub fn compute_slot_at_timestamp(&self, timestamp_seconds: u64) -> Option<u64> {
        timestamp_seconds
            .checked_sub(self.genesis_time)?
            .checked_div(self.seconds_per_slot)?
            .checked_add(self.genesis_slot)
    }

    /// Returns the epoch at a given `slot`.
    ///
    /// [See in consensus-spec](https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#compute_epoch_at_slot)
    #[allow(clippy::module_name_repetitions)]
    #[must_use]
    pub const fn compute_epoch_at_slot(&self, slot: u64) -> u64 {
        slot / self.slots_per_epoch
    }

    /// Returns the timestamp at a `slot`, respect to `genesis_time`.
    ///
    /// [See in consensus-spec](https://github.com/ethereum/consensus-specs/blob/dev/specs/bellatrix/beacon-chain.md#compute_timestamp_at_slot)
    #[allow(clippy::module_name_repetitions)]
    #[must_use]
    pub const fn compute_timestamp_at_slot(&self, slot: u64) -> u64 {
        let slots_since_genesis = slot - self.genesis_slot;
        self.genesis_time + (slots_since_genesis * self.seconds_per_slot)
    }

    /// Returns the sync committee period at a given `epoch`.
    ///
    /// [See in consensus-spec](https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/validator.md#sync-committee)
    #[must_use]
    pub const fn compute_sync_committee_period(&self, epoch: u64) -> u64 {
        epoch / self.epochs_per_sync_committee_period
    }

    /// Returns the sync committee period at a given `slot`.
    ///
    /// [See in consensus-spec](https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/light-client/sync-protocol.md#compute_sync_committee_period_at_slot)
    #[must_use]
    pub const fn compute_sync_committee_period_at_slot(&self, slot: u64) -> u64 {
        self.compute_sync_committee_period(self.compute_epoch_at_slot(slot))
    }
}
