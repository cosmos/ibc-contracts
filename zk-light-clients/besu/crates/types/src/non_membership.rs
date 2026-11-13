use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

use crate::Eip1186ProofWitness;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NonMembershipWitness {
    pub schema_version: u32,
    pub proof_height: u64,
    pub state_root: [u8; 32],
    pub source_router: [u8; 20],
    pub path: Vec<Vec<u8>>,
    pub proof: Eip1186ProofWitness,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NonMembershipPublicValues {
    pub schema_version: u32,
    pub proof_height: u64,
    pub state_root: [u8; 32],
    pub source_router: [u8; 20],
    pub path_digest: [u8; 32],
}
