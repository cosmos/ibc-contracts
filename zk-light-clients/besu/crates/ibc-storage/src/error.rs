use core::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum IbcStorageError {
    UnsupportedSchemaVersion(u32),
    ZeroProofHeight,
    ZeroSourceRouter,
    WrongPathElementCount(usize),
    TerminalPathTooShort,
    PathEncodingOverflow,
    InvalidClientIdentifier,
    WrongPathType(u8),
    Mpt(evm_mpt::MptError),
    MissingCommitment,
    CommitmentMismatch,
    DuplicatePathDigest,
    NonCanonicalBatchOrder,
    EntryCountOverflow,
    BatchMerkle(batch_merkle::BatchMerkleError),
}

impl fmt::Display for IbcStorageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedSchemaVersion(version) => {
                write!(formatter, "unsupported witness schema version {version}")
            }
            Self::ZeroProofHeight => formatter.write_str("proof height must be non-zero"),
            Self::ZeroSourceRouter => formatter.write_str("source Router address must be non-zero"),
            Self::WrongPathElementCount(actual) => {
                write!(formatter, "expected one path element, got {actual}")
            }
            Self::TerminalPathTooShort => formatter.write_str("terminal IBC path is too short"),
            Self::PathEncodingOverflow => {
                formatter.write_str("terminal IBC path ABI encoding length overflow")
            }
            Self::InvalidClientIdentifier => formatter.write_str("invalid IBC client identifier"),
            Self::WrongPathType(value) => {
                write!(formatter, "unsupported IBC path type {value:#04x}")
            }
            Self::Mpt(error) => write!(formatter, "invalid authenticated state proof: {error}"),
            Self::MissingCommitment => {
                formatter.write_str("the path-derived Router storage slot is absent")
            }
            Self::CommitmentMismatch => {
                formatter.write_str("authenticated Router commitment does not match")
            }
            Self::DuplicatePathDigest => formatter.write_str("duplicate membership path digest"),
            Self::NonCanonicalBatchOrder => {
                formatter.write_str("membership entries are not strictly ordered by path digest")
            }
            Self::EntryCountOverflow => {
                formatter.write_str("membership entry count does not fit u32")
            }
            Self::BatchMerkle(error) => write!(formatter, "invalid membership batch: {error}"),
        }
    }
}

impl From<batch_merkle::BatchMerkleError> for IbcStorageError {
    fn from(error: batch_merkle::BatchMerkleError) -> Self {
        Self::BatchMerkle(error)
    }
}

impl core::error::Error for IbcStorageError {}

impl From<evm_mpt::MptError> for IbcStorageError {
    fn from(error: evm_mpt::MptError) -> Self {
        Self::Mpt(error)
    }
}
