#![forbid(unsafe_code)]

extern crate alloc;

mod client_state;
mod constants;
mod error;
mod hash;
mod membership;
mod mpt;
mod non_membership;
mod proof_envelope;
mod qbft_header;
mod update;

pub use client_state::ClientState;
pub use constants::*;
pub use error::TypeError;
pub use hash::{keccak256, sha256};
pub use membership::{MembershipEntryWitness, MembershipPublicValues, MembershipWitness};
pub use mpt::{Eip1186ProofWitness, ProofResult};
pub use non_membership::{NonMembershipPublicValues, NonMembershipWitness};
pub use proof_envelope::{MembershipBatchProofEnvelope, SP1ProofEnvelope};
pub use qbft_header::QbftHeaderInput;
pub use update::{UpdatePublicValues, UpdateWitness};

pub type Address = [u8; 20];
pub type Hash32 = [u8; 32];

#[must_use]
pub fn is_zero_address(address: &Address) -> bool {
    address.iter().all(|byte| *byte == 0)
}
