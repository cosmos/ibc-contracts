// SPDX-License-Identifier: Apache-2.0

use anchor_lang::prelude::*;
use anchor_lang::solana_program::bpf_loader_upgradeable;

use crate::constants::*;
use crate::errors::IFTError;
use crate::events::IFTInitialized;
use crate::state::{AccountVersion, IFTAppState};

#[derive(Accounts)]
#[instruction(admin: Pubkey)]
pub struct Initialize<'info> {
    /// Global IFT app state PDA (to be created, singleton)
    #[account(
        init,
        payer = payer,
        space = 8 + IFTAppState::INIT_SPACE,
        seeds = [IFT_APP_STATE_SEED],
        bump
    )]
    pub app_state: Account<'info, IFTAppState>,

    /// Pays for account creation and transaction fees
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Required for PDA account creation
    pub system_program: Program<'info, System>,

    /// BPF Loader Upgradeable `ProgramData` account for this program.
    #[account(
        seeds = [crate::ID.as_ref()],
        bump,
        seeds::program = bpf_loader_upgradeable::ID,
        constraint = program_data.upgrade_authority_address == Some(authority.key())
            @ IFTError::UnauthorizedDeployer
    )]
    pub program_data: Account<'info, ProgramData>,

    /// The program's upgrade authority — must sign to prove deployer identity.
    pub authority: Signer<'info>,
}

pub fn initialize(ctx: Context<Initialize>, admin: Pubkey) -> Result<()> {
    let app_state = &mut ctx.accounts.app_state;
    app_state.version = AccountVersion::V1;
    app_state.bump = ctx.bumps.app_state;
    app_state.admin = admin;
    app_state.pending_admin = None;

    let clock = Clock::get()?;
    emit!(IFTInitialized {
        admin,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use anchor_lang::InstructionData;
    use solana_sdk::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
    };

    use crate::test_utils::*;

    #[test]
    fn test_initialize_success() {
        let mollusk = setup_mollusk();

        let admin = Pubkey::new_unique();
        let payer = Pubkey::new_unique();
        let authority = Pubkey::new_unique();

        let (app_state_pda, _) = get_app_state_pda();
        let (system_program, system_account) = create_system_program_account();
        let (program_data_pda, program_data_account) =
            create_program_data_account(&crate::ID, Some(authority));

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(app_state_pda, false),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(system_program, false),
                AccountMeta::new_readonly(program_data_pda, false),
                AccountMeta::new_readonly(authority, true),
            ],
            data: crate::instruction::Initialize { admin }.data(),
        };

        let accounts = vec![
            (app_state_pda, create_uninitialized_pda()),
            (payer, create_signer_account()),
            (system_program, system_account),
            (program_data_pda, program_data_account),
            (authority, create_signer_account()),
        ];

        let result = mollusk.process_instruction(&instruction, &accounts);
        assert!(
            !result.program_result.is_err(),
            "initialize should succeed: {:?}",
            result.program_result
        );

        let updated_account = result
            .resulting_accounts
            .iter()
            .find(|(k, _)| *k == app_state_pda)
            .expect("app state should exist")
            .1
            .clone();
        let state = deserialize_app_state(&updated_account);
        assert_eq!(state.admin, admin);
        assert!(!state.paused);
        assert_eq!(state.pending_admin, None);
    }

    #[test]
    fn test_initialize_already_initialized_fails() {
        let mollusk = setup_mollusk();

        let admin = Pubkey::new_unique();
        let payer = Pubkey::new_unique();
        let authority = Pubkey::new_unique();

        let (app_state_pda, app_state_bump) = get_app_state_pda();
        let (system_program, system_account) = create_system_program_account();
        let (program_data_pda, program_data_account) =
            create_program_data_account(&crate::ID, Some(authority));

        let app_state_account = create_ift_app_state_account(app_state_bump, admin);

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(app_state_pda, false),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(system_program, false),
                AccountMeta::new_readonly(program_data_pda, false),
                AccountMeta::new_readonly(authority, true),
            ],
            data: crate::instruction::Initialize { admin }.data(),
        };

        let accounts = vec![
            (app_state_pda, app_state_account),
            (payer, create_signer_account()),
            (system_program, system_account),
            (program_data_pda, program_data_account),
            (authority, create_signer_account()),
        ];

        let result = mollusk.process_instruction(&instruction, &accounts);
        // System program returns AccountAlreadyInUse (0) when init tries to
        // create_account for an account that already has data/lamports
        assert_eq!(
            result.program_result,
            Err(solana_sdk::instruction::InstructionError::Custom(0)).into(),
        );
    }

    #[test]
    fn test_initialize_wrong_authority_rejected() {
        use crate::errors::IFTError;

        let mollusk = setup_mollusk();

        let admin = Pubkey::new_unique();
        let payer = Pubkey::new_unique();
        let real_authority = Pubkey::new_unique();
        let wrong_authority = Pubkey::new_unique();

        let (app_state_pda, _) = get_app_state_pda();
        let (system_program, system_account) = create_system_program_account();
        let (program_data_pda, program_data_account) =
            create_program_data_account(&crate::ID, Some(real_authority));

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(app_state_pda, false),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(system_program, false),
                AccountMeta::new_readonly(program_data_pda, false),
                AccountMeta::new_readonly(wrong_authority, true),
            ],
            data: crate::instruction::Initialize { admin }.data(),
        };

        let accounts = vec![
            (app_state_pda, create_uninitialized_pda()),
            (payer, create_signer_account()),
            (system_program, system_account),
            (program_data_pda, program_data_account),
            (wrong_authority, create_signer_account()),
        ];

        let result = mollusk.process_instruction(&instruction, &accounts);
        assert_eq!(
            result.program_result,
            Err(solana_sdk::instruction::InstructionError::Custom(
                ANCHOR_ERROR_OFFSET + IFTError::UnauthorizedDeployer as u32,
            ))
            .into(),
        );
    }

    #[test]
    fn test_initialize_immutable_program_rejected() {
        use crate::errors::IFTError;

        let mollusk = setup_mollusk();

        let admin = Pubkey::new_unique();
        let payer = Pubkey::new_unique();
        let authority = Pubkey::new_unique();

        let (app_state_pda, _) = get_app_state_pda();
        let (system_program, system_account) = create_system_program_account();
        let (program_data_pda, program_data_account) =
            create_program_data_account(&crate::ID, None);

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(app_state_pda, false),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(system_program, false),
                AccountMeta::new_readonly(program_data_pda, false),
                AccountMeta::new_readonly(authority, true),
            ],
            data: crate::instruction::Initialize { admin }.data(),
        };

        let accounts = vec![
            (app_state_pda, create_uninitialized_pda()),
            (payer, create_signer_account()),
            (system_program, system_account),
            (program_data_pda, program_data_account),
            (authority, create_signer_account()),
        ];

        let result = mollusk.process_instruction(&instruction, &accounts);
        assert_eq!(
            result.program_result,
            Err(solana_sdk::instruction::InstructionError::Custom(
                ANCHOR_ERROR_OFFSET + IFTError::UnauthorizedDeployer as u32,
            ))
            .into(),
        );
    }

    #[test]
    fn test_initialize_cross_program_data_rejected() {
        let mollusk = setup_mollusk();

        let admin = Pubkey::new_unique();
        let payer = Pubkey::new_unique();
        let authority = Pubkey::new_unique();
        let other_program_id = Pubkey::new_unique();

        let (app_state_pda, _) = get_app_state_pda();
        let (system_program, system_account) = create_system_program_account();
        let (wrong_program_data_pda, wrong_program_data_account) =
            create_program_data_account(&other_program_id, Some(authority));

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(app_state_pda, false),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(system_program, false),
                AccountMeta::new_readonly(wrong_program_data_pda, false),
                AccountMeta::new_readonly(authority, true),
            ],
            data: crate::instruction::Initialize { admin }.data(),
        };

        let accounts = vec![
            (app_state_pda, create_uninitialized_pda()),
            (payer, create_signer_account()),
            (system_program, system_account),
            (wrong_program_data_pda, wrong_program_data_account),
            (authority, create_signer_account()),
        ];

        let result = mollusk.process_instruction(&instruction, &accounts);
        // Anchor ConstraintSeeds = 2006
        assert_eq!(
            result.program_result,
            Err(solana_sdk::instruction::InstructionError::Custom(2006)).into(),
        );
    }
}
