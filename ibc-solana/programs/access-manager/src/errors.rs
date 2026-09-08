// SPDX-License-Identifier: Apache-2.0

use anchor_lang::prelude::*;
use solana_ibc_types::CpiValidationError;

#[error_code]
pub enum AccessManagerError {
    #[msg("Unauthorized: caller does not have required role")]
    Unauthorized,
    #[msg("Invalid role ID")]
    InvalidRoleId,
    #[msg("Cannot remove the last admin")]
    CannotRemoveLastAdmin,
    #[msg("Account does not have the specified role")]
    RoleNotGranted,
    #[msg("Invalid sysenv: cross-program invocation from unexpected program")]
    InvalidSysenv,
    #[msg("Account must be a signer")]
    SignerRequired,
    #[msg("CPI calls not allowed")]
    CpiNotAllowed,
    #[msg("Program account does not match target_program")]
    ProgramMismatch,
    #[msg("Zero account is not allowed")]
    ZeroAccount,
    #[msg("Duplicate entry in whitelisted programs list")]
    DuplicateWhitelistedProgram,
    #[msg("Only the program's upgrade authority can call initialize")]
    UnauthorizedDeployer,
    #[msg("New authority account does not match instruction parameter")]
    AuthorityMismatch,
    #[msg("A pending transfer for this target program already exists")]
    PendingTransferAlreadyExists,
    #[msg("No pending transfer for this target program")]
    NoPendingTransfer,
    #[msg("Cannot transfer upgrade authority to the current authority PDA")]
    SelfTransfer,
    #[msg("Maximum number of concurrent pending transfers reached")]
    TooManyPendingTransfers,
    #[msg("No pending access manager transfer to accept or cancel")]
    NoPendingAccessManagerTransfer,
    #[msg("A pending access manager transfer already exists")]
    PendingAccessManagerTransferAlreadyExists,
    #[msg("Proposed access manager address is invalid")]
    InvalidProposedAccessManager,
    #[msg("Cannot transfer access manager to the current access manager")]
    AccessManagerSelfTransfer,
}

impl From<CpiValidationError> for AccessManagerError {
    fn from(error: CpiValidationError) -> Self {
        match error {
            CpiValidationError::InvalidSysvar => Self::InvalidSysenv,
            CpiValidationError::UnauthorizedCaller
            | CpiValidationError::DirectCallNotAllowed
            | CpiValidationError::NestedCpiNotAllowed => Self::CpiNotAllowed,
        }
    }
}
