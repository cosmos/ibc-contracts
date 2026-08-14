// SPDX-License-Identifier: Apache-2.0

use anchor_lang::{
    error_code,
    prelude::{AccountInfo, Pubkey},
    solana_program::{
        instruction::{get_stack_height, TRANSACTION_LEVEL_STACK_HEIGHT},
        sysvar::instructions::get_instruction_relative,
    },
    Key,
};

// # How `get_instruction_relative(0, ...)` works
//
// The Instructions sysvar stores only top-level transaction instructions,
// not nested CPI calls. During CPI, the instruction index doesn't change.
//
// ## Direct call - instruction IS our program:
//
//   Transaction
//   ├─ Instruction 0: Program B (we are here, direct call)
//   │                 get_instruction_relative(0) → Instruction 0
//   │                 current_ix.program_id == self_program_id ✓
//   │
//   └─ Instruction 1: Program C
//
// ## CPI call - instruction is the CALLER's program:
//
//   Transaction
//   ├─ Instruction 0: Program A ──┐
//   │                             │ CPI
//   │                             ▼
//   │                      Program B (we are here)
//   │                      get_instruction_relative(0) → Instruction 0
//   │                      current_ix.program_id == Program A (caller)
//   │
//   └─ Instruction 1: Program C
//
// This allows us to verify WHO initiated the call.

/// `TRANSACTION_LEVEL_STACK_HEIGHT` (1) is the stack height for top-level
/// transaction instructions. Each CPI hop adds 1, so single-level CPI = 2.
const SINGLE_LEVEL_CPI_STACK_HEIGHT: usize = TRANSACTION_LEVEL_STACK_HEIGHT + 1;

#[error_code]
pub enum CpiValidationError {
    #[msg("Invalid sysvar account")]
    InvalidSysvar,
    #[msg("Direct call not allowed - must be called via CPI")]
    DirectCallNotAllowed,
    #[msg("Unauthorized CPI caller")]
    UnauthorizedCaller,
    #[msg("Nested CPI not allowed - only single-level CPI is permitted")]
    NestedCpiNotAllowed,
}

/// Wrapper around `get_stack_height()` that asserts the result is non-zero
/// in the SBF runtime. Stack height is always >= 1 (direct call = 1, CPI = 2+).
/// Gated behind cfg because `get_stack_height()` returns 0 in `cargo test`
/// (the default syscall stub has no runtime context).
#[allow(
    unexpected_cfgs,
    reason = "solana is a valid target_os for the SBF toolchain but unknown to the relayer workspace"
)]
fn checked_stack_height() -> usize {
    let height = get_stack_height();
    #[cfg(target_os = "solana")]
    assert!(height > 0, "stack height must never be zero");
    height
}

/// Returns `true` if the current instruction is executing inside a CPI call.
pub fn is_cpi() -> bool {
    checked_stack_height() > TRANSACTION_LEVEL_STACK_HEIGHT
}

/// Rejects nested CPI chains (A → B → C). Only allows direct calls
/// (stack height 1) or single-level CPI (stack height 2).
///
/// This is required because `get_instruction_relative(0)` always returns the
/// top-level transaction instruction. In A → B → C, program C would see A as
/// the caller instead of B. Limiting to single-level CPI ensures the top-level
/// instruction IS the direct caller, making caller identity checks reliable.
pub fn reject_nested_cpi() -> core::result::Result<(), CpiValidationError> {
    if checked_stack_height() > SINGLE_LEVEL_CPI_STACK_HEIGHT {
        return Err(CpiValidationError::NestedCpiNotAllowed);
    }

    Ok(())
}

/// Rejects direct transaction calls. Only allows CPI (stack height >= 2).
pub fn reject_direct_calls() -> core::result::Result<(), CpiValidationError> {
    if !is_cpi() {
        return Err(CpiValidationError::DirectCallNotAllowed);
    }
    Ok(())
}

fn validate_instruction_sysvar(
    instruction_sysvar: &AccountInfo<'_>,
) -> core::result::Result<(), CpiValidationError> {
    if instruction_sysvar.key() != anchor_lang::solana_program::sysvar::instructions::ID {
        return Err(CpiValidationError::InvalidSysvar);
    }
    Ok(())
}

/// Validates that this instruction is called via CPI from the authorized program
///
/// Checks:
/// 1. Rejects nested CPI (stack height > 2)
/// 2. `instruction_sysvar` is the real sysvar (prevents [Wormhole-style attack])
/// 3. Current instruction's `program_id` is NOT self (rejects direct calls)
/// 4. Current instruction's `program_id` is the authorized program
///
/// [Wormhole-style attack]: https://github.com/sbellem/wormhole-attack-analysis
pub fn validate_cpi_caller(
    instruction_sysvar: &AccountInfo<'_>,
    authorized_program: &Pubkey,
    self_program_id: &Pubkey,
) -> Result<(), CpiValidationError> {
    reject_nested_cpi()?;
    validate_instruction_sysvar(instruction_sysvar)?;

    let current_ix = get_instruction_relative(0, instruction_sysvar)
        .map_err(|_| CpiValidationError::InvalidSysvar)?;

    if current_ix.program_id == *self_program_id {
        return Err(CpiValidationError::DirectCallNotAllowed);
    }

    if current_ix.program_id != *authorized_program {
        return Err(CpiValidationError::UnauthorizedCaller);
    }

    Ok(())
}

/// Validates that this instruction is either called directly OR via CPI from a whitelisted program
///
/// Checks:
/// 1. Rejects nested CPI (stack height > 2)
/// 2. `instruction_sysvar` is the real sysvar (prevents Wormhole-style attack)
/// 3. Current instruction's `program_id` is either self (direct call) or in the whitelist
///
/// Use this for instructions that can be called both directly by users and via CPI from trusted programs.
pub fn require_direct_call_or_whitelisted_caller(
    instruction_sysvar: &AccountInfo<'_>,
    whitelisted_programs: &[Pubkey],
    self_program_id: &Pubkey,
) -> core::result::Result<(), CpiValidationError> {
    reject_nested_cpi()?;
    validate_instruction_sysvar(instruction_sysvar)?;

    let current_ix = get_instruction_relative(0, instruction_sysvar)
        .map_err(|_| CpiValidationError::InvalidSysvar)?;

    if current_ix.program_id == *self_program_id {
        return Ok(());
    }

    // Allow CPI from whitelisted programs
    if whitelisted_programs.contains(&current_ix.program_id) {
        return Ok(());
    }

    Err(CpiValidationError::UnauthorizedCaller)
}

/// Validates that this program is called directly from the transaction (not via CPI)
///
/// Checks:
/// 1. Stack height confirms we are NOT in a CPI context (catches self-recursive A → A)
/// 2. `instruction_sysvar` is the real sysvar (prevents [Wormhole-style attack])
/// 3. Top-level instruction's `program_id` IS self (rejects external CPI callers)
pub fn reject_cpi(
    instruction_sysvar: &AccountInfo<'_>,
    self_program_id: &Pubkey,
) -> core::result::Result<(), CpiValidationError> {
    if is_cpi() {
        return Err(CpiValidationError::UnauthorizedCaller);
    }
    require_direct_call_or_whitelisted_caller(instruction_sysvar, &[], self_program_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use anchor_lang::solana_program::sysvar::instructions::ID as INSTRUCTIONS_SYSVAR_ID;
    use solana_sdk::sysvar::instructions::{
        construct_instructions_data, BorrowedAccountMeta, BorrowedInstruction,
    };

    fn create_instructions_sysvar_data(caller_program_id: &Pubkey) -> Vec<u8> {
        let account_pubkey = Pubkey::new_unique();
        let account = BorrowedAccountMeta {
            pubkey: &account_pubkey,
            is_signer: false,
            is_writable: true,
        };
        let instruction = BorrowedInstruction {
            program_id: caller_program_id,
            accounts: vec![account],
            data: &[],
        };
        construct_instructions_data(&[instruction])
    }

    fn create_test_account_info<'a>(
        key: &'a Pubkey,
        lamports: &'a mut u64,
        data: &'a mut [u8],
        owner: &'a Pubkey,
    ) -> AccountInfo<'a> {
        AccountInfo::new(key, false, false, lamports, data, owner, false, 0)
    }

    #[test]
    fn test_validate_cpi_caller_authorized_succeeds() {
        let authorized_program = Pubkey::new_unique();
        let self_program_id = Pubkey::new_unique();

        let mut data = create_instructions_sysvar_data(&authorized_program);
        let mut lamports = 1_000_000u64;
        let sysvar_owner = anchor_lang::solana_program::sysvar::ID;

        let account_info = create_test_account_info(
            &INSTRUCTIONS_SYSVAR_ID,
            &mut lamports,
            &mut data,
            &sysvar_owner,
        );

        let result = validate_cpi_caller(&account_info, &authorized_program, &self_program_id);

        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_cpi_caller_unauthorized_fails() {
        let authorized_program = Pubkey::new_unique();
        let unauthorized_caller = Pubkey::new_unique();
        let self_program_id = Pubkey::new_unique();

        let mut data = create_instructions_sysvar_data(&unauthorized_caller);
        let mut lamports = 1_000_000u64;
        let sysvar_owner = anchor_lang::solana_program::sysvar::ID;

        let account_info = create_test_account_info(
            &INSTRUCTIONS_SYSVAR_ID,
            &mut lamports,
            &mut data,
            &sysvar_owner,
        );

        let result = validate_cpi_caller(&account_info, &authorized_program, &self_program_id);

        assert!(matches!(
            result,
            Err(CpiValidationError::UnauthorizedCaller)
        ));
    }

    #[test]
    fn test_validate_cpi_caller_direct_call_fails() {
        let authorized_program = Pubkey::new_unique();
        let self_program_id = Pubkey::new_unique();

        let mut data = create_instructions_sysvar_data(&self_program_id);
        let mut lamports = 1_000_000u64;
        let sysvar_owner = anchor_lang::solana_program::sysvar::ID;

        let account_info = create_test_account_info(
            &INSTRUCTIONS_SYSVAR_ID,
            &mut lamports,
            &mut data,
            &sysvar_owner,
        );

        let result = validate_cpi_caller(&account_info, &authorized_program, &self_program_id);

        assert!(matches!(
            result,
            Err(CpiValidationError::DirectCallNotAllowed)
        ));
    }

    #[test]
    fn test_validate_direct_or_whitelisted_direct_call_succeeds() {
        let self_program_id = Pubkey::new_unique();
        let whitelisted: Vec<Pubkey> = vec![];

        let mut data = create_instructions_sysvar_data(&self_program_id);
        let mut lamports = 1_000_000u64;
        let sysvar_owner = anchor_lang::solana_program::sysvar::ID;

        let account_info = create_test_account_info(
            &INSTRUCTIONS_SYSVAR_ID,
            &mut lamports,
            &mut data,
            &sysvar_owner,
        );

        let result = require_direct_call_or_whitelisted_caller(
            &account_info,
            &whitelisted,
            &self_program_id,
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_direct_or_whitelisted_cpi_from_whitelist_succeeds() {
        let self_program_id = Pubkey::new_unique();
        let whitelisted_caller = Pubkey::new_unique();
        let whitelisted = vec![whitelisted_caller];

        let mut data = create_instructions_sysvar_data(&whitelisted_caller);
        let mut lamports = 1_000_000u64;
        let sysvar_owner = anchor_lang::solana_program::sysvar::ID;

        let account_info = create_test_account_info(
            &INSTRUCTIONS_SYSVAR_ID,
            &mut lamports,
            &mut data,
            &sysvar_owner,
        );

        let result = require_direct_call_or_whitelisted_caller(
            &account_info,
            &whitelisted,
            &self_program_id,
        );

        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_direct_or_whitelisted_unauthorized_fails() {
        let self_program_id = Pubkey::new_unique();
        let unauthorized_caller = Pubkey::new_unique();
        let whitelisted: Vec<Pubkey> = vec![];

        let mut data = create_instructions_sysvar_data(&unauthorized_caller);
        let mut lamports = 1_000_000u64;
        let sysvar_owner = anchor_lang::solana_program::sysvar::ID;

        let account_info = create_test_account_info(
            &INSTRUCTIONS_SYSVAR_ID,
            &mut lamports,
            &mut data,
            &sysvar_owner,
        );

        let result = require_direct_call_or_whitelisted_caller(
            &account_info,
            &whitelisted,
            &self_program_id,
        );

        assert!(matches!(
            result,
            Err(CpiValidationError::UnauthorizedCaller)
        ));
    }

    #[test]
    fn test_reject_cpi_direct_call_succeeds() {
        let self_program_id = Pubkey::new_unique();

        let mut data = create_instructions_sysvar_data(&self_program_id);
        let mut lamports = 1_000_000u64;
        let sysvar_owner = anchor_lang::solana_program::sysvar::ID;

        let account_info = create_test_account_info(
            &INSTRUCTIONS_SYSVAR_ID,
            &mut lamports,
            &mut data,
            &sysvar_owner,
        );

        let result = reject_cpi(&account_info, &self_program_id);

        assert!(result.is_ok());
    }

    #[test]
    fn test_reject_cpi_cpi_call_fails() {
        let self_program_id = Pubkey::new_unique();
        let cpi_caller = Pubkey::new_unique();

        let mut data = create_instructions_sysvar_data(&cpi_caller);
        let mut lamports = 1_000_000u64;
        let sysvar_owner = anchor_lang::solana_program::sysvar::ID;

        let account_info = create_test_account_info(
            &INSTRUCTIONS_SYSVAR_ID,
            &mut lamports,
            &mut data,
            &sysvar_owner,
        );

        let result = reject_cpi(&account_info, &self_program_id);

        assert!(matches!(
            result,
            Err(CpiValidationError::UnauthorizedCaller)
        ));
    }
}
