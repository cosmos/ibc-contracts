// SPDX-License-Identifier: Apache-2.0

use crate::constants::*;
use crate::errors::GMPError;
use crate::events::GMPAppInitialized;
use crate::state::{AccountVersion, GMPAppState};
use access_manager::AccessManagerState;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::bpf_loader_upgradeable;

/// Initializes the ICS27 GMP application by creating the global config PDA.
#[derive(Accounts)]
#[instruction(access_manager: Pubkey)]
pub struct Initialize<'info> {
    /// GMP program's global configuration PDA, created with a fixed seed.
    #[account(
        init,
        payer = payer,
        space = 8 + GMPAppState::INIT_SPACE,
        seeds = [GMPAppState::SEED],
        bump
    )]
    pub app_state: Account<'info, GMPAppState>,

    /// Fee payer that funds the `app_state` account creation.
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Solana system program used for account allocation.
    pub system_program: Program<'info, System>,

    /// BPF Loader Upgradeable `ProgramData` account for this program.
    #[account(
        seeds = [crate::ID.as_ref()],
        bump,
        seeds::program = bpf_loader_upgradeable::ID,
        constraint = program_data.upgrade_authority_address == Some(authority.key())
            @ GMPError::UnauthorizedDeployer
    )]
    pub program_data: Account<'info, ProgramData>,

    /// The program's upgrade authority — must sign to prove deployer identity.
    pub authority: Signer<'info>,
}

pub fn initialize(ctx: Context<Initialize>, access_manager: Pubkey) -> Result<()> {
    require!(
        access_manager != Pubkey::default(),
        GMPError::InvalidAccessManager
    );

    let app_state = &mut ctx.accounts.app_state;
    let clock = Clock::get()?;

    // Initialize app state
    app_state.version = AccountVersion::V1;
    app_state.paused = false;
    app_state.bump = ctx.bumps.app_state;
    app_state.am_state = AccessManagerState::new(access_manager);
    app_state._reserved = [0; 256];

    // Emit initialization event
    emit!(GMPAppInitialized {
        router_program: ics26_router::ID,
        port_id: GMP_PORT_ID.to_string(),
        timestamp: clock.unix_timestamp,
    });

    msg!(
        "ICS27 GMP app initialized with router: {}, port_id: {}",
        ics26_router::ID,
        GMP_PORT_ID
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::create_program_data_account;
    use anchor_lang::InstructionData;
    use mollusk_svm::result::Check;
    use mollusk_svm::Mollusk;
    use solana_sdk::instruction::{AccountMeta, Instruction};
    use solana_sdk::program_error::ProgramError;
    use solana_sdk::pubkey::Pubkey;
    use solana_sdk::system_program;

    fn create_initialize_instruction(
        app_state: Pubkey,
        payer: Pubkey,
        program_data: Pubkey,
        authority: Pubkey,
    ) -> Instruction {
        let instruction_data = crate::instruction::Initialize {
            access_manager: access_manager::ID,
        };

        Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(app_state, false),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(system_program::ID, false),
                AccountMeta::new_readonly(program_data, false),
                AccountMeta::new_readonly(authority, true),
            ],
            data: instruction_data.data(),
        }
    }

    #[test]
    fn test_initialize_success() {
        let payer = Pubkey::new_unique();
        let authority = Pubkey::new_unique();

        let (app_state_pda, _) = Pubkey::find_program_address(&[GMPAppState::SEED], &crate::ID);
        let (program_data_pda, program_data_account) =
            create_program_data_account(&crate::ID, Some(authority));

        let instruction =
            create_initialize_instruction(app_state_pda, payer, program_data_pda, authority);

        let accounts = vec![
            (app_state_pda, solana_sdk::account::Account::default()),
            (
                payer,
                solana_sdk::account::Account {
                    lamports: 1_000_000_000,
                    owner: system_program::ID,
                    ..Default::default()
                },
            ),
            (
                system_program::ID,
                solana_sdk::account::Account {
                    lamports: 1,
                    executable: true,
                    owner: solana_sdk::native_loader::ID,
                    ..Default::default()
                },
            ),
            (program_data_pda, program_data_account),
            (
                authority,
                solana_sdk::account::Account {
                    lamports: 1_000_000_000,
                    owner: system_program::ID,
                    ..Default::default()
                },
            ),
        ];

        let mollusk = Mollusk::new(&crate::ID, crate::get_gmp_program_path());

        let checks = vec![
            Check::success(),
            Check::account(&app_state_pda).owner(&crate::ID).build(),
        ];

        mollusk.process_and_validate_instruction(&instruction, &accounts, &checks);
    }

    #[test]
    fn test_initialize_cannot_reinitialize() {
        let payer = Pubkey::new_unique();
        let authority = Pubkey::new_unique();

        let (app_state_pda, _) = Pubkey::find_program_address(&[GMPAppState::SEED], &crate::ID);
        let (program_data_pda, program_data_account) =
            create_program_data_account(&crate::ID, Some(authority));

        let instruction =
            create_initialize_instruction(app_state_pda, payer, program_data_pda, authority);

        let accounts = vec![
            (
                app_state_pda,
                solana_sdk::account::Account {
                    lamports: 1_000_000,
                    data: vec![0; 100],
                    owner: crate::ID,
                    ..Default::default()
                },
            ),
            (
                payer,
                solana_sdk::account::Account {
                    lamports: 1_000_000_000,
                    owner: system_program::ID,
                    ..Default::default()
                },
            ),
            (
                system_program::ID,
                solana_sdk::account::Account {
                    lamports: 1,
                    executable: true,
                    owner: solana_sdk::native_loader::ID,
                    ..Default::default()
                },
            ),
            (program_data_pda, program_data_account),
            (
                authority,
                solana_sdk::account::Account {
                    lamports: 1_000_000_000,
                    owner: system_program::ID,
                    ..Default::default()
                },
            ),
        ];

        let mollusk = Mollusk::new(&crate::ID, crate::get_gmp_program_path());

        let checks = vec![Check::err(ProgramError::Custom(0))];

        mollusk.process_and_validate_instruction(&instruction, &accounts, &checks);
    }

    #[test]
    fn test_initialize_rejects_default_access_manager() {
        let payer = Pubkey::new_unique();
        let authority = Pubkey::new_unique();

        let (app_state_pda, _) = Pubkey::find_program_address(&[GMPAppState::SEED], &crate::ID);
        let (program_data_pda, program_data_account) =
            create_program_data_account(&crate::ID, Some(authority));

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(app_state_pda, false),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(system_program::ID, false),
                AccountMeta::new_readonly(program_data_pda, false),
                AccountMeta::new_readonly(authority, true),
            ],
            data: crate::instruction::Initialize {
                access_manager: Pubkey::default(),
            }
            .data(),
        };

        let accounts = vec![
            (app_state_pda, solana_sdk::account::Account::default()),
            (
                payer,
                solana_sdk::account::Account {
                    lamports: 1_000_000_000,
                    owner: system_program::ID,
                    ..Default::default()
                },
            ),
            (
                system_program::ID,
                solana_sdk::account::Account {
                    lamports: 1,
                    executable: true,
                    owner: solana_sdk::native_loader::ID,
                    ..Default::default()
                },
            ),
            (program_data_pda, program_data_account),
            (
                authority,
                solana_sdk::account::Account {
                    lamports: 1_000_000_000,
                    owner: system_program::ID,
                    ..Default::default()
                },
            ),
        ];

        let mollusk = Mollusk::new(&crate::ID, crate::get_gmp_program_path());

        let checks = vec![Check::err(ProgramError::Custom(
            crate::test_utils::ANCHOR_ERROR_OFFSET
                + crate::errors::GMPError::InvalidAccessManager as u32,
        ))];

        mollusk.process_and_validate_instruction(&instruction, &accounts, &checks);
    }

    #[test]
    fn test_initialize_wrong_authority_rejected() {
        let payer = Pubkey::new_unique();
        let real_authority = Pubkey::new_unique();
        let wrong_authority = Pubkey::new_unique();

        let (app_state_pda, _) = Pubkey::find_program_address(&[GMPAppState::SEED], &crate::ID);
        let (program_data_pda, program_data_account) =
            create_program_data_account(&crate::ID, Some(real_authority));

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(app_state_pda, false),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(system_program::ID, false),
                AccountMeta::new_readonly(program_data_pda, false),
                AccountMeta::new_readonly(wrong_authority, true),
            ],
            data: crate::instruction::Initialize {
                access_manager: access_manager::ID,
            }
            .data(),
        };

        let accounts = vec![
            (app_state_pda, solana_sdk::account::Account::default()),
            (
                payer,
                solana_sdk::account::Account {
                    lamports: 1_000_000_000,
                    owner: system_program::ID,
                    ..Default::default()
                },
            ),
            (
                system_program::ID,
                solana_sdk::account::Account {
                    lamports: 1,
                    executable: true,
                    owner: solana_sdk::native_loader::ID,
                    ..Default::default()
                },
            ),
            (program_data_pda, program_data_account),
            (
                wrong_authority,
                solana_sdk::account::Account {
                    lamports: 1_000_000_000,
                    owner: system_program::ID,
                    ..Default::default()
                },
            ),
        ];

        let mollusk = Mollusk::new(&crate::ID, crate::get_gmp_program_path());

        let checks = vec![Check::err(ProgramError::Custom(
            crate::test_utils::ANCHOR_ERROR_OFFSET
                + crate::errors::GMPError::UnauthorizedDeployer as u32,
        ))];

        mollusk.process_and_validate_instruction(&instruction, &accounts, &checks);
    }

    #[test]
    fn test_initialize_immutable_program_rejected() {
        let payer = Pubkey::new_unique();
        let authority = Pubkey::new_unique();

        let (app_state_pda, _) = Pubkey::find_program_address(&[GMPAppState::SEED], &crate::ID);
        let (program_data_pda, program_data_account) =
            create_program_data_account(&crate::ID, None);

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(app_state_pda, false),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(system_program::ID, false),
                AccountMeta::new_readonly(program_data_pda, false),
                AccountMeta::new_readonly(authority, true),
            ],
            data: crate::instruction::Initialize {
                access_manager: access_manager::ID,
            }
            .data(),
        };

        let accounts = vec![
            (app_state_pda, solana_sdk::account::Account::default()),
            (
                payer,
                solana_sdk::account::Account {
                    lamports: 1_000_000_000,
                    owner: system_program::ID,
                    ..Default::default()
                },
            ),
            (
                system_program::ID,
                solana_sdk::account::Account {
                    lamports: 1,
                    executable: true,
                    owner: solana_sdk::native_loader::ID,
                    ..Default::default()
                },
            ),
            (program_data_pda, program_data_account),
            (
                authority,
                solana_sdk::account::Account {
                    lamports: 1_000_000_000,
                    owner: system_program::ID,
                    ..Default::default()
                },
            ),
        ];

        let mollusk = Mollusk::new(&crate::ID, crate::get_gmp_program_path());

        let checks = vec![Check::err(ProgramError::Custom(
            crate::test_utils::ANCHOR_ERROR_OFFSET
                + crate::errors::GMPError::UnauthorizedDeployer as u32,
        ))];

        mollusk.process_and_validate_instruction(&instruction, &accounts, &checks);
    }

    #[test]
    fn test_initialize_cross_program_data_rejected() {
        let payer = Pubkey::new_unique();
        let authority = Pubkey::new_unique();
        let other_program_id = Pubkey::new_unique();

        let (app_state_pda, _) = Pubkey::find_program_address(&[GMPAppState::SEED], &crate::ID);
        let (wrong_program_data_pda, wrong_program_data_account) =
            create_program_data_account(&other_program_id, Some(authority));

        let instruction =
            create_initialize_instruction(app_state_pda, payer, wrong_program_data_pda, authority);

        let accounts = vec![
            (app_state_pda, solana_sdk::account::Account::default()),
            (
                payer,
                solana_sdk::account::Account {
                    lamports: 1_000_000_000,
                    owner: system_program::ID,
                    ..Default::default()
                },
            ),
            (
                system_program::ID,
                solana_sdk::account::Account {
                    lamports: 1,
                    executable: true,
                    owner: solana_sdk::native_loader::ID,
                    ..Default::default()
                },
            ),
            (wrong_program_data_pda, wrong_program_data_account),
            (
                authority,
                solana_sdk::account::Account {
                    lamports: 1_000_000_000,
                    owner: system_program::ID,
                    ..Default::default()
                },
            ),
        ];

        let mollusk = Mollusk::new(&crate::ID, crate::get_gmp_program_path());

        // Anchor ConstraintSeeds = 2006
        let checks = vec![Check::err(ProgramError::Custom(2006))];
        mollusk.process_and_validate_instruction(&instruction, &accounts, &checks);
    }
}
