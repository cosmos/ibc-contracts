// SPDX-License-Identifier: Apache-2.0

use anchor_lang::prelude::*;
use solana_ibc_types::CpiValidationError;

/// Custom errors for ICS27 IFT program
#[error_code]
pub enum IFTError {
    #[msg("Client ID cannot be empty")]
    EmptyClientId = 7000,

    #[msg("Counterparty address cannot be empty")]
    EmptyCounterpartyAddress,

    #[msg("Receiver cannot be empty")]
    EmptyReceiver,

    #[msg("Transfer amount must be greater than zero")]
    ZeroAmount,

    #[msg("IFT bridge not found for client")]
    BridgeNotFound,

    #[msg("IFT bridge mint does not match app state mint")]
    InvalidBridge,

    #[msg("IFT bridge is not active")]
    BridgeNotActive,

    #[msg("Invalid receiver address")]
    InvalidReceiver,

    #[msg("Timeout too soon")]
    TimeoutInPast,

    #[msg("Timeout too far in the future")]
    TimeoutTooLong,

    #[msg("Pending transfer not found")]
    PendingTransferNotFound,

    #[msg("Token account owner mismatch")]
    TokenAccountOwnerMismatch,

    #[msg("GMP call failed")]
    GmpCallFailed,

    #[msg("Invalid client ID length")]
    InvalidClientIdLength,

    #[msg("Invalid counterparty address length")]
    InvalidCounterpartyAddressLength,

    #[msg("Counterparty denom cannot be empty for Cosmos chains")]
    CosmosEmptyCounterpartyDenom,

    #[msg("Invalid counterparty denom length")]
    InvalidCounterpartyDenomLength,

    #[msg("Cosmos type URL cannot be empty for Cosmos chains")]
    CosmosEmptyTypeUrl,

    #[msg("Invalid Cosmos type URL length")]
    InvalidCosmosTypeUrlLength,

    #[msg("Cosmos ICA address cannot be empty for Cosmos chains")]
    CosmosEmptyIcaAddress,

    #[msg("Invalid Cosmos ICA address length")]
    InvalidCosmosIcaAddressLength,

    #[msg("Invalid Cosmos ICA address - not a valid bech32 address")]
    InvalidCosmosIcaAddress,

    #[msg("Invalid GMP account - not derived from expected counterparty bridge")]
    InvalidGmpAccount,

    #[msg("Invalid pending transfer account")]
    InvalidPendingTransfer,

    #[msg("GMP result client ID mismatch")]
    GmpResultClientMismatch,

    #[msg("GMP result sequence mismatch")]
    GmpResultSequenceMismatch,

    #[msg("GMP result sender mismatch")]
    GmpResultSenderMismatch,

    #[msg("Mint authority is not set on the token")]
    MintAuthorityNotSet,

    #[msg("Invalid mint authority - signer does not match current authority")]
    InvalidMintAuthority,

    #[msg("Daily mint rate limit exceeded")]
    MintRateLimitExceeded,

    #[msg("App is paused")]
    AppPaused,

    #[msg("Unauthorized: signer is not the admin")]
    UnauthorizedAdmin,

    #[msg("CPI calls not allowed for this instruction")]
    CpiNotAllowed,

    #[msg("Token program does not match the requested token type")]
    TokenProgramMismatch,

    #[msg("Arithmetic overflow")]
    ArithmeticOverflow,

    #[msg("No pending admin proposal")]
    NoPendingAdmin,

    #[msg("Signer is not the pending admin")]
    UnauthorizedPendingAdmin,

    #[msg("Cannot propose the zero address as admin")]
    InvalidProposedAdmin,

    #[msg("Cannot propose the current admin as the new admin")]
    SelfProposal,

    #[msg("A pending admin proposal already exists; cancel it first")]
    PendingAdminAlreadyExists,

    #[msg("Only the program's upgrade authority can call initialize")]
    UnauthorizedDeployer,
}

impl From<CpiValidationError> for IFTError {
    fn from(_: CpiValidationError) -> Self {
        Self::CpiNotAllowed
    }
}
