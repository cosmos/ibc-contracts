// SPDX-License-Identifier: Apache-2.0

use crate::errors::RouterError;
use crate::state::{AccountVersion, RouterState};
use access_manager::AccessManagerState;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::bpf_loader_upgradeable;

/// Initializes the ICS26 router by creating the global `RouterState` PDA.
#[derive(Accounts)]
#[instruction(access_manager: Pubkey)]
pub struct Initialize<'info> {
    /// Global router configuration PDA storing the access manager address.
    #[account(
        init,
        payer = payer,
        space = 8 + RouterState::INIT_SPACE,
        seeds = [RouterState::SEED],
        bump
    )]
    pub router_state: Account<'info, RouterState>,

    /// Pays for creating the `RouterState` account.
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Solana system program used for account creation.
    pub system_program: Program<'info, System>,

    /// BPF Loader Upgradeable `ProgramData` account for this program.
    #[account(
        seeds = [crate::ID.as_ref()],
        bump,
        seeds::program = bpf_loader_upgradeable::ID,
        constraint = program_data.upgrade_authority_address == Some(authority.key())
            @ RouterError::UnauthorizedDeployer
    )]
    pub program_data: Account<'info, ProgramData>,

    /// The program's upgrade authority — must sign to prove deployer identity.
    pub authority: Signer<'info>,
}

pub fn initialize(ctx: Context<Initialize>, access_manager: Pubkey) -> Result<()> {
    require!(
        access_manager != Pubkey::default(),
        RouterError::InvalidAccessManager
    );

    let router_state = &mut ctx.accounts.router_state;
    router_state.version = AccountVersion::V1;
    router_state.am_state = AccessManagerState::new(access_manager);
    router_state.paused = false;
    router_state._reserved = [0u8; 256];
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;
    use anchor_lang::InstructionData;
    use mollusk_svm::result::Check;
    use mollusk_svm::Mollusk;
    use solana_sdk::account::Account;
    use solana_sdk::instruction::{AccountMeta, Instruction};
    use solana_sdk::program_error::ProgramError;
    use solana_sdk::pubkey::Pubkey;
    use solana_sdk::{native_loader, system_program};

    const ANCHOR_ERROR_OFFSET: u32 = 6000;

    #[test]
    fn test_initialize_happy_path() {
        let payer = Pubkey::new_unique();
        let authority = Pubkey::new_unique();

        let (router_state_pda, _) = Pubkey::find_program_address(&[RouterState::SEED], &crate::ID);
        let (program_data_pda, program_data_account) =
            create_program_data_account(&crate::ID, Some(authority));

        let instruction_data = crate::instruction::Initialize {
            access_manager: access_manager::ID,
        };

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(router_state_pda, false),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(system_program::ID, false),
                AccountMeta::new_readonly(program_data_pda, false),
                AccountMeta::new_readonly(authority, true),
            ],
            data: instruction_data.data(),
        };

        let payer_lamports = 10_000_000_000;
        let accounts = vec![
            (
                router_state_pda,
                Account {
                    lamports: 0,
                    data: vec![],
                    owner: system_program::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
            (
                payer,
                Account {
                    lamports: payer_lamports,
                    data: vec![],
                    owner: system_program::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
            (
                system_program::ID,
                Account {
                    lamports: 0,
                    data: vec![],
                    owner: native_loader::ID,
                    executable: true,
                    rent_epoch: 0,
                },
            ),
            (program_data_pda, program_data_account),
            (
                authority,
                Account {
                    lamports: 1_000_000_000,
                    data: vec![],
                    owner: system_program::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
        ];

        let mollusk = Mollusk::new(&crate::ID, crate::test_utils::get_router_program_path());

        let checks = vec![
            Check::success(),
            Check::account(&router_state_pda).owner(&crate::ID).build(),
        ];

        let result = mollusk.process_and_validate_instruction(&instruction, &accounts, &checks);

        let payer_account = result
            .resulting_accounts
            .iter()
            .find(|(pubkey, _)| pubkey == &payer)
            .map(|(_, account)| account)
            .expect("Payer account not found");

        assert!(
            payer_account.lamports < payer_lamports,
            "Payer should have paid for account creation"
        );

        let router_state_account = result
            .resulting_accounts
            .iter()
            .find(|(pubkey, _)| pubkey == &router_state_pda)
            .map(|(_, account)| account)
            .expect("Router state account not found");

        assert!(
            router_state_account.lamports > 0,
            "Router state account should be rent-exempt"
        );
        assert!(
            router_state_account.data.len() > 8,
            "Router state account should have data"
        );

        let deserialized_router_state: RouterState =
            RouterState::try_deserialize(&mut &router_state_account.data[..])
                .expect("Failed to deserialize router state");

        assert_eq!(deserialized_router_state.version, AccountVersion::V1);
        assert_eq!(
            deserialized_router_state.am_state.access_manager,
            access_manager::ID
        );
    }

    #[test]
    fn test_initialize_cannot_reinitialize() {
        let payer = Pubkey::new_unique();
        let authority = Pubkey::new_unique();

        // Create an already-initialized router state
        let (router_state_pda, router_state_account) = create_initialized_router_state();
        let (program_data_pda, program_data_account) =
            create_program_data_account(&crate::ID, Some(authority));

        let instruction_data = crate::instruction::Initialize {
            access_manager: access_manager::ID,
        };

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(router_state_pda, false),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(system_program::ID, false),
                AccountMeta::new_readonly(program_data_pda, false),
                AccountMeta::new_readonly(authority, true),
            ],
            data: instruction_data.data(),
        };

        let payer_lamports = 10_000_000_000;
        let accounts = vec![
            (router_state_pda, router_state_account),
            (
                payer,
                Account {
                    lamports: payer_lamports,
                    data: vec![],
                    owner: system_program::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
            (
                system_program::ID,
                Account {
                    lamports: 0,
                    data: vec![],
                    owner: native_loader::ID,
                    executable: true,
                    rent_epoch: 0,
                },
            ),
            (program_data_pda, program_data_account),
            (
                authority,
                Account {
                    lamports: 1_000_000_000,
                    data: vec![],
                    owner: system_program::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
        ];

        let mollusk = Mollusk::new(&crate::ID, get_router_program_path());

        // Anchor's `init` constraint fails when account already exists
        // Error code 0 means the account is already in use
        let checks = vec![Check::err(ProgramError::Custom(0))];

        mollusk.process_and_validate_instruction(&instruction, &accounts, &checks);
    }

    #[test]
    fn test_initialize_rejects_default_access_manager() {
        let payer = Pubkey::new_unique();
        let authority = Pubkey::new_unique();

        let (router_state_pda, _) = Pubkey::find_program_address(&[RouterState::SEED], &crate::ID);
        let (program_data_pda, program_data_account) =
            create_program_data_account(&crate::ID, Some(authority));

        let instruction_data = crate::instruction::Initialize {
            access_manager: Pubkey::default(),
        };

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(router_state_pda, false),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(system_program::ID, false),
                AccountMeta::new_readonly(program_data_pda, false),
                AccountMeta::new_readonly(authority, true),
            ],
            data: instruction_data.data(),
        };

        let accounts = vec![
            (
                router_state_pda,
                Account {
                    lamports: 0,
                    data: vec![],
                    owner: system_program::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
            (
                payer,
                Account {
                    lamports: 10_000_000_000,
                    data: vec![],
                    owner: system_program::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
            (
                system_program::ID,
                Account {
                    lamports: 0,
                    data: vec![],
                    owner: native_loader::ID,
                    executable: true,
                    rent_epoch: 0,
                },
            ),
            (program_data_pda, program_data_account),
            (
                authority,
                Account {
                    lamports: 1_000_000_000,
                    data: vec![],
                    owner: system_program::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
        ];

        let mollusk = Mollusk::new(&crate::ID, get_router_program_path());

        let checks = vec![Check::err(ProgramError::Custom(
            ANCHOR_ERROR_OFFSET + RouterError::InvalidAccessManager as u32,
        ))];

        mollusk.process_and_validate_instruction(&instruction, &accounts, &checks);
    }

    #[test]
    fn test_initialize_wrong_authority_rejected() {
        let payer = Pubkey::new_unique();
        let real_authority = Pubkey::new_unique();
        let wrong_authority = Pubkey::new_unique();

        let (router_state_pda, _) = Pubkey::find_program_address(&[RouterState::SEED], &crate::ID);
        let (program_data_pda, program_data_account) =
            create_program_data_account(&crate::ID, Some(real_authority));

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(router_state_pda, false),
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
            (
                router_state_pda,
                Account {
                    lamports: 0,
                    data: vec![],
                    owner: system_program::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
            (
                payer,
                Account {
                    lamports: 10_000_000_000,
                    data: vec![],
                    owner: system_program::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
            (
                system_program::ID,
                Account {
                    lamports: 0,
                    data: vec![],
                    owner: native_loader::ID,
                    executable: true,
                    rent_epoch: 0,
                },
            ),
            (program_data_pda, program_data_account),
            (
                wrong_authority,
                Account {
                    lamports: 1_000_000_000,
                    data: vec![],
                    owner: system_program::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
        ];

        let mollusk = Mollusk::new(&crate::ID, get_router_program_path());
        let checks = vec![Check::err(ProgramError::Custom(
            ANCHOR_ERROR_OFFSET + RouterError::UnauthorizedDeployer as u32,
        ))];
        mollusk.process_and_validate_instruction(&instruction, &accounts, &checks);
    }

    #[test]
    fn test_initialize_immutable_program_rejected() {
        let payer = Pubkey::new_unique();
        let authority = Pubkey::new_unique();

        let (router_state_pda, _) = Pubkey::find_program_address(&[RouterState::SEED], &crate::ID);
        let (program_data_pda, program_data_account) =
            create_program_data_account(&crate::ID, None);

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(router_state_pda, false),
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
            (
                router_state_pda,
                Account {
                    lamports: 0,
                    data: vec![],
                    owner: system_program::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
            (
                payer,
                Account {
                    lamports: 10_000_000_000,
                    data: vec![],
                    owner: system_program::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
            (
                system_program::ID,
                Account {
                    lamports: 0,
                    data: vec![],
                    owner: native_loader::ID,
                    executable: true,
                    rent_epoch: 0,
                },
            ),
            (program_data_pda, program_data_account),
            (
                authority,
                Account {
                    lamports: 1_000_000_000,
                    data: vec![],
                    owner: system_program::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
        ];

        let mollusk = Mollusk::new(&crate::ID, get_router_program_path());
        let checks = vec![Check::err(ProgramError::Custom(
            ANCHOR_ERROR_OFFSET + RouterError::UnauthorizedDeployer as u32,
        ))];
        mollusk.process_and_validate_instruction(&instruction, &accounts, &checks);
    }

    #[test]
    fn test_initialize_cross_program_data_rejected() {
        let payer = Pubkey::new_unique();
        let authority = Pubkey::new_unique();
        let other_program_id = Pubkey::new_unique();

        let (router_state_pda, _) = Pubkey::find_program_address(&[RouterState::SEED], &crate::ID);
        let (wrong_program_data_pda, wrong_program_data_account) =
            create_program_data_account(&other_program_id, Some(authority));

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(router_state_pda, false),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(system_program::ID, false),
                AccountMeta::new_readonly(wrong_program_data_pda, false),
                AccountMeta::new_readonly(authority, true),
            ],
            data: crate::instruction::Initialize {
                access_manager: access_manager::ID,
            }
            .data(),
        };

        let accounts = vec![
            (
                router_state_pda,
                Account {
                    lamports: 0,
                    data: vec![],
                    owner: system_program::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
            (
                payer,
                Account {
                    lamports: 10_000_000_000,
                    data: vec![],
                    owner: system_program::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
            (
                system_program::ID,
                Account {
                    lamports: 0,
                    data: vec![],
                    owner: native_loader::ID,
                    executable: true,
                    rent_epoch: 0,
                },
            ),
            (wrong_program_data_pda, wrong_program_data_account),
            (
                authority,
                Account {
                    lamports: 1_000_000_000,
                    data: vec![],
                    owner: system_program::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
        ];

        let mollusk = Mollusk::new(&crate::ID, get_router_program_path());
        // Anchor ConstraintSeeds = 2006
        let checks = vec![Check::err(ProgramError::Custom(2006))];
        mollusk.process_and_validate_instruction(&instruction, &accounts, &checks);
    }
}
