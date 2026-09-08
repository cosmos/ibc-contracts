// SPDX-License-Identifier: Apache-2.0

use anchor_lang::prelude::*;
use solana_ibc_types::reject_cpi;

use crate::constants::*;
use crate::errors::IFTError;
use crate::events::MintRateLimitUpdated;
use crate::state::{IFTAppMintState, IFTAppState, SetMintRateLimitMsg};

#[derive(Accounts)]
#[instruction(msg: SetMintRateLimitMsg)]
pub struct SetMintRateLimit<'info> {
    /// Global IFT app state (read-only, for admin check)
    #[account(
        seeds = [IFT_APP_STATE_SEED],
        bump = app_state.bump
    )]
    pub app_state: Account<'info, IFTAppState>,

    /// Per-mint IFT app state (mut, for `daily_mint_limit`)
    #[account(
        mut,
        seeds = [IFT_APP_MINT_STATE_SEED, app_mint_state.mint.as_ref()],
        bump = app_mint_state.bump
    )]
    pub app_mint_state: Account<'info, IFTAppMintState>,

    /// Admin authority, must match `app_state.admin`
    #[account(
        constraint = admin.key() == app_state.admin @ IFTError::UnauthorizedAdmin
    )]
    pub admin: Signer<'info>,

    /// CHECK: Address constraint verifies this is the instructions sysvar
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instructions_sysvar: AccountInfo<'info>,
}

pub fn set_mint_rate_limit(ctx: Context<SetMintRateLimit>, msg: SetMintRateLimitMsg) -> Result<()> {
    reject_cpi(&ctx.accounts.instructions_sysvar, &crate::ID).map_err(IFTError::from)?;

    ctx.accounts.app_mint_state.daily_mint_limit = msg.daily_mint_limit;

    let clock = Clock::get()?;
    emit!(MintRateLimitUpdated {
        mint: ctx.accounts.app_mint_state.mint,
        daily_mint_limit: msg.daily_mint_limit,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use anchor_lang::InstructionData;
    use rstest::rstest;
    use solana_sdk::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
    };

    use crate::errors::IFTError;
    use crate::state::SetMintRateLimitMsg;
    use crate::test_utils::*;

    fn run_set_mint_rate_limit_success_test(limit: u64) {
        let mollusk = setup_mollusk();

        let mint = Pubkey::new_unique();
        let admin = Pubkey::new_unique();
        let (app_state_pda, app_state_bump) = get_app_state_pda();
        let (app_mint_state_pda, app_mint_state_bump) = get_app_mint_state_pda(&mint);
        let (_, mint_authority_bump) = get_mint_authority_pda(&mint);
        let (sysvar_id, sysvar_account) = create_instructions_sysvar_account();

        let app_state_account = create_ift_app_state_account(app_state_bump, admin);

        let app_mint_state_account =
            create_ift_app_mint_state_account(mint, app_mint_state_bump, mint_authority_bump);

        let msg = SetMintRateLimitMsg {
            daily_mint_limit: limit,
        };

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new_readonly(app_state_pda, false),
                AccountMeta::new(app_mint_state_pda, false),
                AccountMeta::new_readonly(admin, true),
                AccountMeta::new_readonly(sysvar_id, false),
            ],
            data: crate::instruction::SetMintRateLimit { msg }.data(),
        };

        let accounts = vec![
            (app_state_pda, app_state_account),
            (app_mint_state_pda, app_mint_state_account),
            (admin, create_signer_account()),
            (sysvar_id, sysvar_account),
        ];

        let result = mollusk.process_instruction(&instruction, &accounts);
        assert!(
            !result.program_result.is_err(),
            "set_mint_rate_limit({limit}) should succeed: {:?}",
            result.program_result
        );

        let updated_account = result
            .resulting_accounts
            .iter()
            .find(|(k, _)| *k == app_mint_state_pda)
            .expect("app mint state should exist")
            .1
            .clone();
        let updated_state = deserialize_app_mint_state(&updated_account);
        assert_eq!(updated_state.daily_mint_limit, limit);
    }

    #[rstest]
    #[case::set_limit(1_000_000)]
    #[case::disable(0)]
    fn test_set_mint_rate_limit_success(#[case] limit: u64) {
        run_set_mint_rate_limit_success_test(limit);
    }

    #[test]
    fn test_set_mint_rate_limit_unauthorized() {
        let mollusk = setup_mollusk();

        let mint = Pubkey::new_unique();
        let admin = Pubkey::new_unique();
        let unauthorized = Pubkey::new_unique();

        let (app_state_pda, app_state_bump) = get_app_state_pda();
        let (app_mint_state_pda, app_mint_state_bump) = get_app_mint_state_pda(&mint);
        let (_, mint_authority_bump) = get_mint_authority_pda(&mint);
        let (sysvar_id, sysvar_account) = create_instructions_sysvar_account();

        let app_state_account = create_ift_app_state_account(app_state_bump, admin);

        let app_mint_state_account =
            create_ift_app_mint_state_account(mint, app_mint_state_bump, mint_authority_bump);

        let msg = SetMintRateLimitMsg {
            daily_mint_limit: 1_000_000,
        };

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new_readonly(app_state_pda, false),
                AccountMeta::new(app_mint_state_pda, false),
                AccountMeta::new_readonly(unauthorized, true),
                AccountMeta::new_readonly(sysvar_id, false),
            ],
            data: crate::instruction::SetMintRateLimit { msg }.data(),
        };

        let accounts = vec![
            (app_state_pda, app_state_account),
            (app_mint_state_pda, app_mint_state_account),
            (unauthorized, create_signer_account()),
            (sysvar_id, sysvar_account),
        ];

        let result = mollusk.process_instruction(&instruction, &accounts);
        assert_eq!(
            result.program_result,
            Err(solana_sdk::instruction::InstructionError::Custom(
                ANCHOR_ERROR_OFFSET + IFTError::UnauthorizedAdmin as u32,
            ))
            .into(),
        );
    }

    #[test]
    fn test_set_mint_rate_limit_cpi_rejected() {
        let mollusk = setup_mollusk();

        let mint = Pubkey::new_unique();
        let admin = Pubkey::new_unique();

        let (app_state_pda, app_state_bump) = get_app_state_pda();
        let (app_mint_state_pda, app_mint_state_bump) = get_app_mint_state_pda(&mint);
        let (_, mint_authority_bump) = get_mint_authority_pda(&mint);
        let (sysvar_id, sysvar_account) =
            create_cpi_instructions_sysvar_account(Pubkey::new_unique());

        let app_state_account = create_ift_app_state_account(app_state_bump, admin);

        let app_mint_state_account =
            create_ift_app_mint_state_account(mint, app_mint_state_bump, mint_authority_bump);

        let msg = SetMintRateLimitMsg {
            daily_mint_limit: 1_000_000,
        };

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new_readonly(app_state_pda, false),
                AccountMeta::new(app_mint_state_pda, false),
                AccountMeta::new_readonly(admin, true),
                AccountMeta::new_readonly(sysvar_id, false),
            ],
            data: crate::instruction::SetMintRateLimit { msg }.data(),
        };

        let accounts = vec![
            (app_state_pda, app_state_account),
            (app_mint_state_pda, app_mint_state_account),
            (admin, create_signer_account()),
            (sysvar_id, sysvar_account),
        ];

        let result = mollusk.process_instruction(&instruction, &accounts);
        assert_eq!(
            result.program_result,
            Err(solana_sdk::instruction::InstructionError::Custom(
                ANCHOR_ERROR_OFFSET + IFTError::CpiNotAllowed as u32,
            ))
            .into(),
        );
    }
}
