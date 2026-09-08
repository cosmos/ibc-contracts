// SPDX-License-Identifier: Apache-2.0

use anchor_lang::prelude::*;

/// Custom errors for ICS27 GMP program
#[error_code]
pub enum GMPError {
    #[msg("App is currently paused")]
    AppPaused = 6000,

    #[msg("Invalid router program")]
    InvalidRouter,

    #[msg("Execution payload is empty")]
    EmptyPayload,

    #[msg("Invalid timeout timestamp")]
    InvalidTimeout,

    #[msg("Timeout too far in future")]
    TimeoutTooLong,

    #[msg("Timeout too soon")]
    TimeoutTooSoon,

    #[msg("Unauthorized sender")]
    UnauthorizedSender,

    #[msg("Wrong counterparty client")]
    WrongCounterpartyClient,

    #[msg("Invalid salt")]
    InvalidSalt,

    #[msg("Target program is not executable")]
    TargetNotExecutable,

    #[msg("Insufficient accounts provided")]
    InsufficientAccounts,

    #[msg("Account count mismatch")]
    AccountCountMismatch,

    #[msg("Account key mismatch")]
    AccountKeyMismatch,

    #[msg("Insufficient account permissions")]
    InsufficientAccountPermissions,

    #[msg("Account must not be a signer")]
    UnexpectedSigner,

    #[msg("Execution too expensive")]
    ExecutionTooExpensive,

    #[msg("Invalid account address derivation")]
    GMPAccountPDAMismatch,

    #[msg("Unauthorized admin operation")]
    UnauthorizedAdmin,

    #[msg("Invalid packet data format")]
    InvalidPacketData,

    #[msg("Unauthorized router calling")]
    UnauthorizedRouter,

    #[msg("Direct calls not allowed, must be called via CPI from router")]
    DirectCallNotAllowed,

    #[msg("Invalid port ID length")]
    InvalidPortId,

    #[msg("Invalid client ID length")]
    InvalidClientId,

    #[msg("Invalid sender address length")]
    InvalidSender,

    #[msg("Invalid memo length")]
    InvalidMemo,

    #[msg("Invalid Solana payload format")]
    InvalidSolanaPayload,

    #[msg("Account already exists")]
    AccountAlreadyExists,

    #[msg("Failed to parse packet data")]
    PacketDataParseError,

    #[msg("Packet data validation failed")]
    PacketDataValidationFailed,

    #[msg("Target program execution failed")]
    TargetExecutionFailed,

    #[msg("Invalid acknowledgement format")]
    InvalidAcknowledgement,

    #[msg("Compute budget exceeded")]
    ComputeBudgetExceeded,

    #[msg("Too many accounts in execution payload")]
    TooManyAccounts,

    #[msg("Invalid program ID format")]
    InvalidProgramId,

    #[msg("Invalid account key format")]
    InvalidAccountKey,

    #[msg("Invalid router CPI call")]
    InvalidRouterCall,

    #[msg("Insufficient funds for account creation")]
    InsufficientFunds,

    #[msg("Invalid IBC version")]
    InvalidVersion,

    #[msg("Invalid IBC port")]
    InvalidPort,

    #[msg("Invalid IBC encoding")]
    InvalidEncoding,

    #[msg("Failed to parse sequence from router account")]
    SequenceParseError,

    #[msg("Failed to decode protobuf message")]
    DecodeError,

    #[msg("Result account PDA mismatch")]
    ResultAccountPDAMismatch,

    #[msg("Invalid ABI encoding")]
    InvalidAbiEncoding,

    #[msg("Light client program does not match client registry")]
    InvalidLightClientProgram,

    #[msg("Account not owned by expected program")]
    InvalidAccountOwner,

    #[msg("Invalid access manager")]
    InvalidAccessManager,

    #[msg("Only the program's upgrade authority can call initialize")]
    UnauthorizedDeployer,
}

/// Convert GMP validation errors to GMP errors
impl From<solana_ibc_proto::GmpValidationError> for GMPError {
    fn from(err: solana_ibc_proto::GmpValidationError) -> Self {
        match err {
            solana_ibc_proto::GmpValidationError::DecodeError => Self::InvalidSolanaPayload,
            solana_ibc_proto::GmpValidationError::InvalidProgramId => Self::InvalidProgramId,
            solana_ibc_proto::GmpValidationError::EmptyPayload => Self::EmptyPayload,
            solana_ibc_proto::GmpValidationError::TooManyAccounts => Self::TooManyAccounts,
            solana_ibc_proto::GmpValidationError::InvalidAccountKey => Self::InvalidAccountKey,
        }
    }
}

/// Convert CPI validation errors to GMP errors
impl From<solana_ibc_types::CpiValidationError> for GMPError {
    fn from(err: solana_ibc_types::CpiValidationError) -> Self {
        match err {
            solana_ibc_types::CpiValidationError::DirectCallNotAllowed => {
                Self::DirectCallNotAllowed
            }
            solana_ibc_types::CpiValidationError::InvalidSysvar
            | solana_ibc_types::CpiValidationError::UnauthorizedCaller
            | solana_ibc_types::CpiValidationError::NestedCpiNotAllowed => Self::UnauthorizedRouter,
        }
    }
}
