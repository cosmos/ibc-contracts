#![forbid(unsafe_code)]

extern crate alloc;

mod error;
mod extra_data;
mod header;
mod quorum;
mod signature;
mod signing;
mod trust;
mod validator;

pub use error::QbftError;
pub use extra_data::{encode_extra_data, parse_extra_data, QbftExtraData, QbftVote, VoteKind};
pub use header::{encode_header, validate_optional_tail};
pub use quorum::{target_quorum, trusted_overlap_threshold};
pub use signature::{recover_address, validate_seal};
pub use signing::committed_seal_digest;
pub use trust::verify_update;
pub use validator::{validate_validator_set, validator_set_hash};
