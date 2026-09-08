// SPDX-License-Identifier: Apache-2.0

use anchor_lang::prelude::*;
use solana_ibc_types::reject_cpi;

use crate::constants::*;
use crate::errors::IFTError;
use crate::events::TokenPausedUpdated;
use crate::state::{IFTAppState, SetPausedMsg};

#[derive(Accounts)]
#[instruction(msg: SetPausedMsg)]
pub struct SetPaused<'info> {
    /// Global IFT app state (mut, `paused` field will be updated)
    #[account(
        mut,
        seeds = [IFT_APP_STATE_SEED],
        bump = app_state.bump
    )]
    pub app_state: Account<'info, IFTAppState>,

    /// Admin authority, must match `app_state.admin`
    #[account(
        constraint = admin.key() == app_state.admin @ IFTError::UnauthorizedAdmin
    )]
    pub admin: Signer<'info>,

    /// CHECK: Address constraint verifies this is the instructions sysvar
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instructions_sysvar: AccountInfo<'info>,
}

pub fn set_paused(ctx: Context<SetPaused>, msg: SetPausedMsg) -> Result<()> {
    reject_cpi(&ctx.accounts.instructions_sysvar, &crate::ID).map_err(IFTError::from)?;

    ctx.accounts.app_state.paused = msg.paused;

    let clock = Clock::get()?;
    emit!(TokenPausedUpdated {
        paused: msg.paused,
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
    use crate::state::SetPausedMsg;
    use crate::test_utils::*;

    fn run_set_paused_success_test(paused: bool) {
        let mollusk = setup_mollusk();

        let admin = Pubkey::new_unique();
        let (app_state_pda, app_state_bump) = get_app_state_pda();
        let (sysvar_id, sysvar_account) = create_instructions_sysvar_account();

        let app_state_account = create_ift_app_state_account(app_state_bump, admin);

        let msg = SetPausedMsg { paused };

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(app_state_pda, false),
                AccountMeta::new_readonly(admin, true),
                AccountMeta::new_readonly(sysvar_id, false),
            ],
            data: crate::instruction::SetPaused { msg }.data(),
        };

        let accounts = vec![
            (app_state_pda, app_state_account),
            (admin, create_signer_account()),
            (sysvar_id, sysvar_account),
        ];

        let result = mollusk.process_instruction(&instruction, &accounts);
        assert!(
            !result.program_result.is_err(),
            "set_paused({paused}) should succeed: {:?}",
            result.program_result
        );

        let updated_account = result
            .resulting_accounts
            .iter()
            .find(|(k, _)| *k == app_state_pda)
            .expect("app state should exist")
            .1
            .clone();
        let updated_state = deserialize_app_state(&updated_account);
        assert_eq!(updated_state.paused, paused);
    }

    #[rstest]
    #[case::pause(true)]
    #[case::unpause(false)]
    fn test_set_paused_success(#[case] paused: bool) {
        run_set_paused_success_test(paused);
    }

    #[test]
    fn test_set_paused_unauthorized() {
        let mollusk = setup_mollusk();

        let admin = Pubkey::new_unique();
        let unauthorized = Pubkey::new_unique();

        let (app_state_pda, app_state_bump) = get_app_state_pda();
        let (sysvar_id, sysvar_account) = create_instructions_sysvar_account();

        let app_state_account = create_ift_app_state_account(app_state_bump, admin);

        let msg = SetPausedMsg { paused: true };

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(app_state_pda, false),
                AccountMeta::new_readonly(unauthorized, true),
                AccountMeta::new_readonly(sysvar_id, false),
            ],
            data: crate::instruction::SetPaused { msg }.data(),
        };

        let accounts = vec![
            (app_state_pda, app_state_account),
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
    fn test_set_paused_cpi_rejected() {
        let mollusk = setup_mollusk();

        let admin = Pubkey::new_unique();
        let (app_state_pda, app_state_bump) = get_app_state_pda();
        let (sysvar_id, sysvar_account) =
            create_cpi_instructions_sysvar_account(Pubkey::new_unique());

        let app_state_account = create_ift_app_state_account(app_state_bump, admin);

        let msg = SetPausedMsg { paused: true };

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(app_state_pda, false),
                AccountMeta::new_readonly(admin, true),
                AccountMeta::new_readonly(sysvar_id, false),
            ],
            data: crate::instruction::SetPaused { msg }.data(),
        };

        let accounts = vec![
            (app_state_pda, app_state_account),
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
