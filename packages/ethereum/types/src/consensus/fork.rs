// SPDX-License-Identifier: Apache-2.0

//! This module defines types related to forks in Ethereum.

use alloy_primitives::{aliases::B32, B256};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tree_hash::TreeHash;
use tree_hash_derive::TreeHash;

/// Type alias Etheruem Version which is a fixed 4 byte array
pub type Version = B32;

/// The epoch of a fork that is not scheduled.
/// [See in consensus-spec](https://github.com/ethereum/consensus-specs/blob/master/specs/phase0/weak-subjectivity.md#constants)
pub const FAR_FUTURE_EPOCH: u64 = u64::MAX;

/// The fork data
#[derive(Serialize, Deserialize, JsonSchema, PartialEq, Eq, Clone, Debug, Default)]
pub struct Fork {
    /// The version of the fork
    #[schemars(with = "String")]
    pub version: Version,
    /// The epoch at which this fork is activated
    pub epoch: u64,
}

impl Fork {
    /// Returns a fork that is not scheduled.
    #[must_use]
    pub fn far_future() -> Self {
        Self {
            version: Version::default(),
            epoch: FAR_FUTURE_EPOCH,
        }
    }
}

/// The fork data
#[derive(Serialize, Deserialize, PartialEq, Clone, Debug, Default, TreeHash)]
struct ForkData {
    /// The current version
    pub current_version: Version,
    /// The genesis validators root
    pub genesis_validators_root: B256,
}

/// The fork parameters
#[derive(Serialize, Deserialize, JsonSchema, PartialEq, Eq, Clone, Debug, Default)]
#[allow(clippy::module_name_repetitions)]
pub struct ForkParameters {
    /// The genesis fork version
    #[schemars(with = "String")]
    pub genesis_fork_version: Version,
    /// The genesis slot
    pub genesis_slot: u64,
    /// The altair fork
    pub altair: Fork,
    /// The bellatrix fork
    pub bellatrix: Fork,
    /// The capella fork
    pub capella: Fork,
    /// The deneb fork
    pub deneb: Fork,
    /// The electra fork
    pub electra: Fork,
    /// The fulu fork
    pub fulu: Fork,
    /// The gloas fork
    // Client states stored before this fork was known do not have this field, and they are correct
    // to treat it as not scheduled until `Migration::UpdateForkParameters` sets the fork epoch.
    #[serde(default = "Fork::far_future")]
    pub gloas: Fork,
}

impl ForkParameters {
    /// Returns the fork version based on the `epoch`.
    /// [See in consensus-spec](https://github.com/ethereum/consensus-specs/blob/dev/specs/capella/fork.md#modified-compute_fork_version)
    #[must_use]
    pub const fn compute_fork_version(&self, epoch: u64) -> Version {
        match epoch {
            _ if epoch >= self.gloas.epoch => self.gloas.version,
            _ if epoch >= self.fulu.epoch => self.fulu.version,
            _ if epoch >= self.electra.epoch => self.electra.version,
            _ if epoch >= self.deneb.epoch => self.deneb.version,
            _ if epoch >= self.capella.epoch => self.capella.version,
            _ if epoch >= self.bellatrix.epoch => self.bellatrix.version,
            _ if epoch >= self.altair.epoch => self.altair.version,
            _ => self.genesis_fork_version,
        }
    }
}

/// Return the 32-byte fork data root for the `current_version` and `genesis_validators_root`.
/// This is used primarily in signature domains to avoid collisions across forks/chains.
///
/// [See in consensus-spec](https://github.com/ethereum/consensus-specs/blob/dev/specs/phase0/beacon-chain.md#compute_fork_data_root)
#[must_use]
pub fn compute_fork_data_root(current_version: Version, genesis_validators_root: B256) -> B256 {
    let fork_data = ForkData {
        current_version,
        genesis_validators_root,
    };

    fork_data.tree_hash_root()
}

#[cfg(test)]
mod test {
    use super::*;

    fn fork(version: u8, epoch: u64) -> Fork {
        Fork {
            version: Version::from([version, 0, 0, 0]),
            epoch,
        }
    }

    #[test]
    fn test_compute_fork_version_at_gloas() {
        let fork_parameters = ForkParameters {
            fulu: fork(6, 10),
            gloas: fork(7, 20),
            ..Default::default()
        };

        assert_eq!(
            fork_parameters.compute_fork_version(19),
            fork(6, 10).version
        );
        assert_eq!(
            fork_parameters.compute_fork_version(20),
            fork(7, 20).version
        );
    }

    #[test]
    fn test_gloas_is_not_scheduled_when_missing_from_stored_state() {
        let mut stored = serde_json::to_value(ForkParameters {
            fulu: fork(6, 10),
            gloas: fork(7, 20),
            ..Default::default()
        })
        .unwrap();
        stored.as_object_mut().unwrap().remove("gloas").unwrap();

        let fork_parameters: ForkParameters = serde_json::from_value(stored).unwrap();

        assert_eq!(fork_parameters.gloas, Fork::far_future());
        assert_eq!(
            fork_parameters.compute_fork_version(u64::MAX - 1),
            fork(6, 10).version
        );
    }
}
