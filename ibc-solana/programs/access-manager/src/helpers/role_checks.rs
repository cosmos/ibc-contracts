// SPDX-License-Identifier: Apache-2.0

use crate::errors::AccessManagerError;
use crate::state::AccessManager;
use anchor_lang::prelude::*;

/// Verifies the caller has the `ADMIN_ROLE`.
///
/// Allows direct calls and whitelisted CPI callers (e.g. multisig) so admin
/// operations can go through governance. Reads the whitelist from the
/// `AccessManager` account state.
pub fn require_admin(
    access_manager_account: &AccountInfo,
    signer_account: &AccountInfo,
    instructions_sysvar: &AccountInfo,
    program_id: &Pubkey,
) -> Result<()> {
    let access_manager = deserialize_access_manager(access_manager_account)?;

    require_role_with_whitelist_inner(
        &access_manager,
        solana_ibc_types::roles::ADMIN_ROLE,
        signer_account,
        instructions_sysvar,
        program_id,
    )
}

/// Verifies the caller has the given role. Rejects all CPI calls — only direct
/// transactions are allowed.
pub fn require_role(
    access_manager_account: &AccountInfo,
    role_id: u64,
    signer_account: &AccountInfo,
    instructions_sysvar: &AccountInfo,
    program_id: &Pubkey,
) -> Result<()> {
    require!(signer_account.is_signer, AccessManagerError::SignerRequired);

    solana_ibc_types::reject_cpi(instructions_sysvar, program_id)
        .map_err(|_| error!(AccessManagerError::CpiNotAllowed))?;

    let access_manager = deserialize_access_manager(access_manager_account)?;

    require!(
        access_manager.has_role(role_id, &signer_account.key()),
        AccessManagerError::Unauthorized
    );

    Ok(())
}

/// Verifies the caller has the given role. Allows direct calls and whitelisted
/// CPI callers. Reads the whitelist from the `AccessManager` account state.
pub fn require_role_with_whitelist(
    access_manager_account: &AccountInfo,
    role_id: u64,
    signer_account: &AccountInfo,
    instructions_sysvar: &AccountInfo,
    program_id: &Pubkey,
) -> Result<()> {
    let access_manager = deserialize_access_manager(access_manager_account)?;

    require_role_with_whitelist_inner(
        &access_manager,
        role_id,
        signer_account,
        instructions_sysvar,
        program_id,
    )
}

fn deserialize_access_manager(account: &AccountInfo) -> Result<AccessManager> {
    let data = account.try_borrow_data()?;
    anchor_lang::AccountDeserialize::try_deserialize(&mut &data[..])
}

fn require_role_with_whitelist_inner(
    access_manager: &AccessManager,
    role_id: u64,
    signer_account: &AccountInfo,
    instructions_sysvar: &AccountInfo,
    program_id: &Pubkey,
) -> Result<()> {
    require!(signer_account.is_signer, AccessManagerError::SignerRequired);

    solana_ibc_types::require_direct_call_or_whitelisted_caller(
        instructions_sysvar,
        &access_manager.whitelisted_programs,
        program_id,
    )
    .map_err(|_| error!(AccessManagerError::CpiNotAllowed))?;

    require!(
        access_manager.has_role(role_id, &signer_account.key()),
        AccessManagerError::Unauthorized
    );

    Ok(())
}
