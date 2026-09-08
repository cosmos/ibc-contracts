// SPDX-License-Identifier: Apache-2.0

use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    // Client state errors
    #[msg("Client is frozen")]
    ClientFrozen,
    #[msg("Client is already frozen")]
    ClientAlreadyFrozen,
    #[msg("Invalid chain ID: cannot be empty")]
    InvalidChainId,
    #[msg("Invalid trust level: numerator must be > 0 and <= denominator")]
    InvalidTrustLevel,
    #[msg("Invalid periods: trusting period must be positive and less than unbonding period")]
    InvalidPeriods,
    #[msg("Invalid max clock drift: must be positive")]
    InvalidMaxClockDrift,

    // Height errors
    #[msg("Invalid height: height cannot be zero")]
    InvalidHeight,
    #[msg("Height mismatch: expected height does not match provided height")]
    HeightMismatch,

    // Header and proof errors
    #[msg("Invalid header: failed to deserialize or validate header")]
    InvalidHeader,
    #[msg("Invalid proof: proof validation failed")]
    InvalidProof,

    // Update errors
    #[msg("Update client failed: header verification failed")]
    UpdateClientFailed,

    // Misbehaviour errors
    #[msg("Invalid misbehaviour: failed to deserialize or validate misbehaviour")]
    InvalidMisbehaviour,
    #[msg("Misbehaviour detected: conflicting consensus state at same height")]
    MisbehaviourConflictingState,
    #[msg("Misbehaviour detected: non-increasing timestamp")]
    MisbehaviourNonIncreasingTime,
    #[msg("Misbehaviour check failed: invalid misbehaviour proof")]
    MisbehaviourCheckFailed,

    // Verification errors
    #[msg("Membership verification failed: proof does not match commitment")]
    MembershipVerificationFailed,
    #[msg("Membership verification failed: value cannot be empty")]
    MembershipEmptyValue,
    #[msg("Non-membership verification failed: key exists when it should not")]
    NonMembershipVerificationFailed,
    #[msg("Invalid value: non-membership proof must have empty value")]
    InvalidValue,

    // Delay errors
    #[msg("Time and block delay must be zero")]
    NonZeroDelay,

    // Consensus state errors
    #[msg("Consensus state not found at the specified height")]
    ConsensusStateNotFound,

    // Chunking errors
    #[msg("Invalid chunk count: unexpected number of chunk accounts")]
    InvalidChunkCount,
    #[msg("Invalid chunk account: chunk account PDA mismatch")]
    InvalidChunkAccount,
    #[msg("Chunk data too large: exceeds maximum chunk size")]
    ChunkDataTooLarge,

    // Access control errors
    #[msg("Unauthorized: caller does not have required role")]
    UnauthorizedRole,
    // Signature verification errors
    #[msg("Invalid number of accounts: mismatch between signatures and PDAs")]
    InvalidNumberOfAccounts,
    #[msg("Invalid account data: account data is malformed or too small")]
    InvalidAccountData,

    // Other errors
    #[msg("Serialization error: failed to serialize/deserialize data")]
    SerializationError,
    #[msg("Account validation failed: invalid account or PDA")]
    AccountValidationFailed,
    #[msg("Invalid account owner: account is not owned by the expected program")]
    InvalidAccountOwner,
    #[msg("Arithmetic overflow detected")]
    ArithmeticOverflow,

    // Conversion errors
    #[msg("Conversion error: invalid header data")]
    ConversionError,

    #[msg("Invalid access manager")]
    InvalidAccessManager,

    #[msg("Only the program's upgrade authority can call initialize")]
    UnauthorizedDeployer,
}

impl From<crate::conversions::ConversionError> for ErrorCode {
    fn from(err: crate::conversions::ConversionError) -> Self {
        msg!("Conversion failed: {}", err.as_str());
        Self::ConversionError
    }
}
