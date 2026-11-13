use core::fmt;
use evm_mpt::RlpError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QbftError {
    Rlp(RlpError),
    InvalidExtraDataShape,
    InvalidVanityLength(usize),
    InvalidValidatorLength(usize),
    EmptyValidatorSet,
    TooManyValidators,
    ZeroValidator,
    UnsortedValidators,
    DuplicateValidator,
    InvalidVote,
    RoundOverflow,
    InvalidSealLength(usize),
    InvalidSignatureScalar,
    InvalidRecoveryId(u8),
    SignatureRecovery,
    ZeroSigner,
    NonValidatorSigner,
    DuplicateSigner,
    InsufficientTargetQuorum { required: usize, actual: usize },
    InsufficientTrustedOverlap { required: usize, actual: usize },
    InvalidOptionalTail,
    UnsupportedSchema(u32),
    ZeroTrustedHeight,
    TargetHeightNotNewer,
    ZeroTargetTimestamp,
    TrustedValidatorHashMismatch,
    ArithmeticOverflow,
}

impl fmt::Display for QbftError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Rlp(error) => write!(formatter, "RLP error: {error}"),
            Self::InvalidExtraDataShape => formatter.write_str("invalid QBFT extraData shape"),
            Self::InvalidVanityLength(length) => {
                write!(formatter, "invalid vanity length {length}")
            }
            Self::InvalidValidatorLength(length) => {
                write!(formatter, "invalid validator length {length}")
            }
            Self::EmptyValidatorSet => formatter.write_str("validator set is empty"),
            Self::TooManyValidators => formatter.write_str("validator count does not fit u32"),
            Self::ZeroValidator => formatter.write_str("validator address is zero"),
            Self::UnsortedValidators => {
                formatter.write_str("validator addresses are not strictly ascending")
            }
            Self::DuplicateValidator => formatter.write_str("duplicate validator address"),
            Self::InvalidVote => formatter.write_str("invalid QBFT vote"),
            Self::RoundOverflow => formatter.write_str("QBFT round does not fit u32"),
            Self::InvalidSealLength(length) => {
                write!(formatter, "invalid commit-seal length {length}")
            }
            Self::InvalidSignatureScalar => {
                formatter.write_str("commit-seal scalar is out of range")
            }
            Self::InvalidRecoveryId(value) => {
                write!(formatter, "invalid commit-seal recovery id {value}")
            }
            Self::SignatureRecovery => {
                formatter.write_str("commit-seal public-key recovery failed")
            }
            Self::ZeroSigner => formatter.write_str("recovered signer is zero"),
            Self::NonValidatorSigner => formatter.write_str("committer is not a target validator"),
            Self::DuplicateSigner => formatter.write_str("duplicate recovered committer"),
            Self::InsufficientTargetQuorum { required, actual } => {
                write!(
                    formatter,
                    "insufficient target quorum: required {required}, got {actual}"
                )
            }
            Self::InsufficientTrustedOverlap { required, actual } => {
                write!(
                    formatter,
                    "insufficient trusted overlap: required {required}, got {actual}"
                )
            }
            Self::InvalidOptionalTail => formatter.write_str("invalid optional header-tail layout"),
            Self::UnsupportedSchema(version) => {
                write!(formatter, "unsupported schema version {version}")
            }
            Self::ZeroTrustedHeight => formatter.write_str("trusted height is zero"),
            Self::TargetHeightNotNewer => {
                formatter.write_str("target height is not newer than trusted height")
            }
            Self::ZeroTargetTimestamp => formatter.write_str("target timestamp is zero"),
            Self::TrustedValidatorHashMismatch => {
                formatter.write_str("trusted validator-set hash mismatch")
            }
            Self::ArithmeticOverflow => formatter.write_str("arithmetic overflow"),
        }
    }
}

impl core::error::Error for QbftError {}

impl From<RlpError> for QbftError {
    fn from(value: RlpError) -> Self {
        Self::Rlp(value)
    }
}
