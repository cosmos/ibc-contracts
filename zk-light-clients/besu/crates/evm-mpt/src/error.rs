use core::fmt;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RlpError {
    EmptyInput,
    Truncated,
    LengthOverflow,
    NonCanonical,
    TrailingBytes,
    ExpectedBytes,
    ExpectedList,
    WrongListLength { expected: usize, actual: usize },
    IntegerLeadingZero,
    IntegerOverflow,
}

impl fmt::Display for RlpError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyInput => formatter.write_str("empty RLP input"),
            Self::Truncated => formatter.write_str("truncated RLP input"),
            Self::LengthOverflow => formatter.write_str("RLP length overflow"),
            Self::NonCanonical => formatter.write_str("non-canonical RLP"),
            Self::TrailingBytes => formatter.write_str("trailing bytes after RLP item"),
            Self::ExpectedBytes => formatter.write_str("expected RLP byte string"),
            Self::ExpectedList => formatter.write_str("expected RLP list"),
            Self::WrongListLength { expected, actual } => {
                write!(
                    formatter,
                    "expected {expected} RLP list items, got {actual}"
                )
            }
            Self::IntegerLeadingZero => formatter.write_str("RLP integer has a leading zero"),
            Self::IntegerOverflow => formatter.write_str("RLP integer overflow"),
        }
    }
}

impl core::error::Error for RlpError {}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MptError {
    Rlp(RlpError),
    InvalidNodeArity(usize),
    InvalidCompactPath,
    CompactPathTooLong,
    EmptyExtensionPath,
    InvalidChildReference,
    MissingNode,
    DuplicateNode,
    UnusedProofNode,
    RootMismatch,
    PathOverflow,
    AccountAbsent,
    InvalidAccount,
    StorageAbsent,
    StoragePresent,
    StorageValueOverflow,
    StorageValueMismatch,
}

impl fmt::Display for MptError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Rlp(error) => write!(formatter, "RLP error: {error}"),
            Self::InvalidNodeArity(actual) => write!(formatter, "invalid MPT node arity {actual}"),
            Self::InvalidCompactPath => formatter.write_str("invalid hex-prefix compact path"),
            Self::CompactPathTooLong => {
                formatter.write_str("compact path exceeds secure-key length")
            }
            Self::EmptyExtensionPath => formatter.write_str("extension path must not be empty"),
            Self::InvalidChildReference => formatter.write_str("invalid MPT child reference"),
            Self::MissingNode => formatter.write_str("missing referenced MPT node"),
            Self::DuplicateNode => formatter.write_str("duplicate supplied MPT node"),
            Self::UnusedProofNode => formatter.write_str("unused supplied MPT proof node"),
            Self::RootMismatch => formatter.write_str("MPT root mismatch"),
            Self::PathOverflow => formatter.write_str("MPT path overflow"),
            Self::AccountAbsent => formatter.write_str("account is absent"),
            Self::InvalidAccount => formatter.write_str("invalid authenticated account encoding"),
            Self::StorageAbsent => formatter.write_str("storage key is absent"),
            Self::StoragePresent => formatter.write_str("storage key is present"),
            Self::StorageValueOverflow => formatter.write_str("storage value exceeds U256"),
            Self::StorageValueMismatch => {
                formatter.write_str("authenticated storage value does not match expected word")
            }
        }
    }
}

impl core::error::Error for MptError {}

impl From<RlpError> for MptError {
    fn from(value: RlpError) -> Self {
        Self::Rlp(value)
    }
}
