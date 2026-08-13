// SPDX-License-Identifier: Apache-2.0

use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::solana_program::program::invoke;

/// Serializable account metadata passed alongside `proxy_cpi` instruction data.
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CpiAccountMeta {
    pub is_signer: bool,
    pub is_writable: bool,
}

/// Accounts for `proxy_cpi`, a whitelisted CPI forwarder used to test
/// authorized-caller code paths from within a real BPF runtime.
#[derive(Accounts)]
#[instruction(instruction_data: Vec<u8>, account_metas: Vec<CpiAccountMeta>)]
pub struct ProxyCpi<'info> {
    /// CHECK: The target program to CPI into
    #[account(executable)]
    pub target_program: AccountInfo<'info>,

    /// Transaction fee payer and signer.
    pub payer: Signer<'info>,
}

pub fn proxy_cpi<'info>(
    ctx: Context<'_, '_, '_, 'info, ProxyCpi<'info>>,
    instruction_data: Vec<u8>,
    account_metas: Vec<CpiAccountMeta>,
) -> Result<()> {
    if account_metas.len() > ctx.remaining_accounts.len() {
        return Err(ProgramError::NotEnoughAccountKeys.into());
    }

    let account_infos = ctx.remaining_accounts[..account_metas.len()].to_vec();

    let instruction = Instruction {
        program_id: ctx.accounts.target_program.key(),
        accounts: account_metas
            .iter()
            .enumerate()
            .map(
                |(i, meta)| anchor_lang::solana_program::instruction::AccountMeta {
                    pubkey: account_infos[i].key(),
                    is_signer: meta.is_signer,
                    is_writable: meta.is_writable,
                },
            )
            .collect(),
        data: instruction_data,
    };

    invoke(&instruction, &account_infos)?;
    Ok(())
}
