use alloc::vec::Vec;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MembershipEntryWitness {
    pub path: Vec<Vec<u8>>,
    pub expected_commitment: [u8; 32],
    pub storage_proof: Vec<Vec<u8>>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MembershipWitness {
    pub schema_version: u32,
    pub proof_height: u64,
    pub state_root: [u8; 32],
    pub source_router: [u8; 20],
    pub account_proof: Vec<Vec<u8>>,
    pub entries: Vec<MembershipEntryWitness>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MembershipPublicValues {
    pub schema_version: u32,
    pub proof_height: u64,
    pub state_root: [u8; 32],
    pub source_router: [u8; 20],
    pub batch_root: [u8; 32],
    pub entry_count: u32,
}
