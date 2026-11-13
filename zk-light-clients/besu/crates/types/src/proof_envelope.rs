use alloc::vec::Vec;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SP1ProofEnvelope {
    pub public_values: Vec<u8>,
    pub proof_bytes: Vec<u8>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MembershipBatchProofEnvelope {
    pub mode: u8,
    pub public_values: Vec<u8>,
    pub proof_bytes: Vec<u8>,
    pub batch_root: [u8; 32],
    pub entry_count: u32,
    pub leaf_index: u32,
    pub merkle_proof: Vec<[u8; 32]>,
}
