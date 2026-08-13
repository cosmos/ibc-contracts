// SPDX-License-Identifier: Apache-2.0

use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::solana_program::program::invoke;

declare_id!("CtQLLKbDMt1XVNXtLKJEt1K8cstbckjqE6zyFqR37KTc");

/// Generic CPI Proxy for Security Testing
///
/// This program acts as a flexible proxy that can execute arbitrary CPI calls.
/// It's designed for security testing to verify that target programs properly
/// validate their callers using the instructions sysvar.
///
/// Security Testing Pattern:
/// 1. E2E test constructs the exact CPI instruction to test
/// 2. Test passes the instruction to this proxy program
/// 3. Proxy executes the CPI, simulating an unauthorized caller
/// 4. Target program should reject the call by checking instructions sysvar
///
/// This generic approach allows testing any callback without hardcoding each one.
#[program]
pub mod test_cpi_proxy {
    use super::*;

    /// Execute an arbitrary CPI call from this program
    ///
    /// This is a generic CPI proxy that forwards any instruction to any program.
    /// The target program should validate the calling instruction's `program_id`
    /// using the instructions sysvar to reject unauthorized callers.
    ///
    /// # Arguments
    /// * `target_program` - The program to CPI into
    /// * `instruction_data` - The serialized instruction data to pass
    /// * `account_metas` - Metadata for accounts (`is_signer`, `is_writable`)
    ///
    /// # Security Note
    /// This program intentionally does NOT validate anything - it's designed
    /// to test that OTHER programs properly validate their CPI callers.
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
}

/// Account metadata for CPI calls (serializable version of `AccountMeta`)
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CpiAccountMeta {
    pub is_signer: bool,
    pub is_writable: bool,
}

/// Accounts for the generic CPI proxy
#[derive(Accounts)]
#[instruction(instruction_data: Vec<u8>, account_metas: Vec<CpiAccountMeta>)]
pub struct ProxyCpi<'info> {
    /// CHECK: The target program we're calling via CPI. This is intentionally
    /// unconstrained beyond executable check, as this is a test utility program
    /// designed to proxy arbitrary CPI calls for security testing.
    #[account(executable)]
    pub target_program: AccountInfo<'info>,

    /// Payer/signer for this transaction
    pub payer: Signer<'info>,
    // All other accounts are passed as remaining_accounts
    // The caller (e2e test) determines what accounts to pass
}
