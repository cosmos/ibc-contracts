// SPDX-License-Identifier: Apache-2.0

//! CPI helpers for calling another access-manager instance (e.g. during
//! AM-to-AM upgrade authority migration).
//!
//! Anchor's auto-generated `cpi` module requires the `cpi` feature which also
//! enables `no-entrypoint`, so a program cannot use its own CPI module.
//! These types mirror the Anchor-generated CPI interface following the same
//! pattern used by `solana_ibc_types::ibc_app`.

use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::AccountMeta;
use anchor_lang::InstructionData;

/// CPI accounts for `accept_upgrade_authority_transfer`.
#[derive(Clone)]
pub struct AcceptUpgradeAuthorityTransferCpi<'info> {
    pub access_manager: AccountInfo<'info>,
    pub program_data: AccountInfo<'info>,
    pub upgrade_authority: AccountInfo<'info>,
    pub new_authority: AccountInfo<'info>,
    pub bpf_loader_upgradeable: AccountInfo<'info>,
}

impl anchor_lang::ToAccountMetas for AcceptUpgradeAuthorityTransferCpi<'_> {
    fn to_account_metas(&self, _is_signer: Option<bool>) -> Vec<AccountMeta> {
        vec![
            AccountMeta::new(*self.access_manager.key, false),
            AccountMeta::new(*self.program_data.key, false),
            AccountMeta::new_readonly(*self.upgrade_authority.key, false),
            AccountMeta::new_readonly(*self.new_authority.key, true),
            AccountMeta::new_readonly(*self.bpf_loader_upgradeable.key, false),
        ]
    }
}

impl<'info> anchor_lang::ToAccountInfos<'info> for AcceptUpgradeAuthorityTransferCpi<'info> {
    fn to_account_infos(&self) -> Vec<AccountInfo<'info>> {
        vec![
            self.access_manager.clone(),
            self.program_data.clone(),
            self.upgrade_authority.clone(),
            self.new_authority.clone(),
            self.bpf_loader_upgradeable.clone(),
        ]
    }
}

/// Invoke `accept_upgrade_authority_transfer` via CPI.
pub fn accept_upgrade_authority_transfer<'info>(
    ctx: CpiContext<'_, '_, '_, 'info, AcceptUpgradeAuthorityTransferCpi<'info>>,
    target_program: Pubkey,
) -> Result<()> {
    let ix_data = crate::instruction::AcceptUpgradeAuthorityTransfer { target_program }.data();

    let account_metas = ctx.accounts.to_account_metas(None);
    let instruction = anchor_lang::solana_program::instruction::Instruction {
        program_id: *ctx.program.key,
        accounts: account_metas,
        data: ix_data,
    };

    let mut account_infos = ctx.accounts.to_account_infos();
    account_infos.push(ctx.program.clone());

    anchor_lang::solana_program::program::invoke_signed(
        &instruction,
        &account_infos,
        ctx.signer_seeds,
    )?;

    Ok(())
}
