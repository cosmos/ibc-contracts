// SPDX-License-Identifier: Apache-2.0

//! This module defines types related to the genesis endpoint of the Beacon API.

use alloy_primitives::B256;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};

use super::fork::Version;

/// Genesis provides information about the genesis of a chain.
#[serde_as]
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, Default)]
pub struct Genesis {
    /// The genesis time (in unix seconds)
    #[serde_as(as = "DisplayFromStr")]
    pub genesis_time: u64,
    /// The genesis validators root
    pub genesis_validators_root: B256,
    /// The genesis fork version
    pub genesis_fork_version: Version,
}
