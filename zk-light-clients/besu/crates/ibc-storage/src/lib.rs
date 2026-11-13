#![forbid(unsafe_code)]

extern crate alloc;

mod error;
mod path;
mod slot;
mod verify;

pub use error::IbcStorageError;
pub use path::{
    parse_membership_path, parse_receipt_path, path_digest, terminal_path, PathKind, TerminalPath,
};
pub use slot::logical_storage_slot;
pub use verify::{verify_membership, verify_non_membership};
