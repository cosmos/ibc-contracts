#![forbid(unsafe_code)]

extern crate alloc;

mod account;
mod error;
mod hex_prefix;
mod inclusion;
mod nibble;
mod node;
mod proof;
pub mod rlp;
mod storage;

pub use account::{verify_account, verify_account_storage_root, AuthenticatedAccount};
pub use error::{MptError, RlpError};
pub use hex_prefix::{decode_compact_path, CompactPath};
pub use nibble::bytes_to_nibbles;
pub use storage::{
    verify_storage, verify_storage_membership, verify_storage_membership_expected,
    verify_storage_non_membership,
};
