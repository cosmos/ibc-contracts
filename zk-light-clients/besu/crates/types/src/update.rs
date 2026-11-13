use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::QbftHeaderInput;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdateWitness {
    pub schema_version: u32,
    pub trusted_height: u64,
    pub trusted_validator_set_hash: [u8; 32],
    pub trusted_validators: Vec<[u8; 20]>,
    pub target_header: QbftHeaderInput,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpdatePublicValues {
    pub schema_version: u32,
    pub trusted_height: u64,
    pub trusted_validator_set_hash: [u8; 32],
    pub verified_height: u64,
    pub state_root: [u8; 32],
    pub block_timestamp: u64,
    pub new_validator_set_hash: [u8; 32],
}
