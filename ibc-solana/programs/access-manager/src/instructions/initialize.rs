// SPDX-License-Identifier: Apache-2.0

use crate::errors::AccessManagerError;
use crate::state::AccessManager;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::bpf_loader_upgradeable;
use solana_ibc_types::{reject_cpi, roles};

/// Creates the global [`AccessManager`] PDA and assigns the first admin.
#[derive(Accounts)]
#[instruction(admin: Pubkey)]
pub struct Initialize<'info> {
    /// The access manager PDA to initialize (created here).
    #[account(
        init,
        payer = payer,
        space = 8 + AccessManager::INIT_SPACE,
        seeds = [AccessManager::SEED],
        bump
    )]
    pub access_manager: Account<'info, AccessManager>,

    /// Pays for account creation rent.
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Required for PDA account creation.
    pub system_program: Program<'info, System>,

    /// Instructions sysvar for CPI validation
    /// CHECK: Address constraint verifies this is the instructions sysvar
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instructions_sysvar: AccountInfo<'info>,

    /// BPF Loader Upgradeable `ProgramData` account for this program.
    #[account(
        seeds = [crate::ID.as_ref()],
        bump,
        seeds::program = bpf_loader_upgradeable::ID,
        constraint = program_data.upgrade_authority_address == Some(authority.key())
            @ AccessManagerError::UnauthorizedDeployer
    )]
    pub program_data: Account<'info, ProgramData>,

    /// The program's upgrade authority — must sign to prove deployer identity.
    pub authority: Signer<'info>,
}

pub fn initialize(ctx: Context<Initialize>, admin: Pubkey) -> Result<()> {
    reject_cpi(&ctx.accounts.instructions_sysvar, &crate::ID).map_err(AccessManagerError::from)?;

    let access_manager = &mut ctx.accounts.access_manager;
    access_manager.roles = vec![];
    access_manager.whitelisted_programs = vec![];
    access_manager.pending_authority_transfers = vec![];

    access_manager.grant_role(roles::ADMIN_ROLE, admin)?;

    msg!("Global access control initialized with admin: {}", admin);
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
    use solana_sdk::pubkey::Pubkey;
    use solana_sdk::{native_loader, system_program};

    #[test]
    fn test_initialize_happy_path() {
        let admin = Pubkey::new_unique();
        let payer = Pubkey::new_unique();
        let authority = Pubkey::new_unique();

        let (access_manager_pda, _) =
            Pubkey::find_program_address(&[AccessManager::SEED], &crate::ID);
        let (program_data_pda, program_data_account) =
            create_program_data_account(&crate::ID, Some(authority));

        let instruction_data = crate::instruction::Initialize { admin };

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(access_manager_pda, false),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(system_program::ID, false),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
                AccountMeta::new_readonly(program_data_pda, false),
                AccountMeta::new_readonly(authority, true),
            ],
            data: instruction_data.data(),
        };

        let payer_lamports = 10_000_000_000;
        let accounts = vec![
            (
                access_manager_pda,
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
            (
                solana_sdk::sysvar::instructions::ID,
                crate::test_utils::create_instructions_sysvar_account(),
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

        let mollusk = Mollusk::new(&crate::ID, crate::get_access_manager_program_path());

        let checks = vec![
            Check::success(),
            Check::account(&access_manager_pda)
                .owner(&crate::ID)
                .build(),
        ];

        let result = mollusk.process_and_validate_instruction(&instruction, &accounts, &checks);

        let access_manager_account = result
            .resulting_accounts
            .iter()
            .find(|(pubkey, _)| pubkey == &access_manager_pda)
            .map(|(_, account)| account)
            .expect("Access control account not found");

        let deserialized_access_manager: AccessManager =
            anchor_lang::AccountDeserialize::try_deserialize(&mut &access_manager_account.data[..])
                .expect("Failed to deserialize access control");

        // Verify admin has ADMIN_ROLE
        assert!(deserialized_access_manager.has_role(roles::ADMIN_ROLE, &admin));
        assert_eq!(deserialized_access_manager.roles.len(), 1);
    }

    #[test]
    fn test_initialize_fake_sysvar_wormhole_attack() {
        let admin = Pubkey::new_unique();
        let payer = Pubkey::new_unique();
        let authority = Pubkey::new_unique();

        let (access_manager_pda, _) =
            Pubkey::find_program_address(&[AccessManager::SEED], &crate::ID);
        let (program_data_pda, program_data_account) =
            create_program_data_account(&crate::ID, Some(authority));

        let instruction_data = crate::instruction::Initialize { admin };

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(access_manager_pda, false),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(system_program::ID, false),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
                AccountMeta::new_readonly(program_data_pda, false),
                AccountMeta::new_readonly(authority, true),
            ],
            data: instruction_data.data(),
        };

        // Replace real sysvar with fake one (Wormhole-style attack)
        let (instruction, fake_sysvar_account) = setup_fake_sysvar_attack(instruction, crate::ID);

        let payer_lamports = 10_000_000_000;
        let accounts = vec![
            (
                access_manager_pda,
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
            fake_sysvar_account,
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

        let mollusk = Mollusk::new(&crate::ID, crate::get_access_manager_program_path());

        mollusk.process_and_validate_instruction(
            &instruction,
            &accounts,
            &[expect_sysvar_attack_error()],
        );
    }

    #[test]
    fn test_initialize_cpi_rejection() {
        let payer = Pubkey::new_unique();
        let admin = Pubkey::new_unique();
        let authority = Pubkey::new_unique();

        let (access_manager_pda, _) =
            Pubkey::find_program_address(&[AccessManager::SEED], &crate::ID);
        let (program_data_pda, program_data_account) =
            create_program_data_account(&crate::ID, Some(authority));

        let instruction_data = crate::instruction::Initialize { admin };

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(access_manager_pda, false),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(system_program::ID, false),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
                AccountMeta::new_readonly(program_data_pda, false),
                AccountMeta::new_readonly(authority, true),
            ],
            data: instruction_data.data(),
        };

        // Simulate CPI call from unauthorized program
        let malicious_program = Pubkey::new_unique();
        let (instruction, cpi_sysvar_account) = setup_cpi_call_test(instruction, malicious_program);

        let payer_lamports = 10_000_000_000;
        let accounts = vec![
            (
                access_manager_pda,
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
            cpi_sysvar_account,
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

        let mollusk = Mollusk::new(&crate::ID, crate::get_access_manager_program_path());

        mollusk.process_and_validate_instruction(
            &instruction,
            &accounts,
            &[expect_access_manager_cpi_rejection_error()],
        );
    }

    #[test]
    fn test_initialize_cannot_reinitialize() {
        let admin = Pubkey::new_unique();
        let payer = Pubkey::new_unique();
        let authority = Pubkey::new_unique();

        // Use helper to create an already-initialized access manager
        let (access_manager_pda, access_manager_account) = create_initialized_access_manager(admin);
        let (program_data_pda, program_data_account) =
            create_program_data_account(&crate::ID, Some(authority));

        let instruction_data = crate::instruction::Initialize { admin };

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(access_manager_pda, false),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(system_program::ID, false),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
                AccountMeta::new_readonly(program_data_pda, false),
                AccountMeta::new_readonly(authority, true),
            ],
            data: instruction_data.data(),
        };

        let payer_lamports = 10_000_000_000;
        let accounts = vec![
            (access_manager_pda, access_manager_account),
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
            (
                solana_sdk::sysvar::instructions::ID,
                create_instructions_sysvar_account(),
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

        let mollusk = Mollusk::new(&crate::ID, crate::get_access_manager_program_path());

        // Anchor's `init` constraint fails when account already exists
        // Error code 0 means the account is already in use
        let checks = vec![Check::err(solana_sdk::program_error::ProgramError::Custom(
            0,
        ))];

        mollusk.process_and_validate_instruction(&instruction, &accounts, &checks);
    }

    #[test]
    fn test_initialize_wrong_authority_rejected() {
        let admin = Pubkey::new_unique();
        let payer = Pubkey::new_unique();
        let real_authority = Pubkey::new_unique();
        let wrong_authority = Pubkey::new_unique();

        let (access_manager_pda, _) =
            Pubkey::find_program_address(&[AccessManager::SEED], &crate::ID);
        let (program_data_pda, program_data_account) =
            create_program_data_account(&crate::ID, Some(real_authority));

        let instruction_data = crate::instruction::Initialize { admin };

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(access_manager_pda, false),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(system_program::ID, false),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
                AccountMeta::new_readonly(program_data_pda, false),
                AccountMeta::new_readonly(wrong_authority, true),
            ],
            data: instruction_data.data(),
        };

        let payer_lamports = 10_000_000_000;
        let accounts = vec![
            (
                access_manager_pda,
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
            (
                solana_sdk::sysvar::instructions::ID,
                create_instructions_sysvar_account(),
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

        let mollusk = Mollusk::new(&crate::ID, crate::get_access_manager_program_path());

        let checks = vec![Check::err(solana_sdk::program_error::ProgramError::Custom(
            ANCHOR_ERROR_OFFSET + AccessManagerError::UnauthorizedDeployer as u32,
        ))];

        mollusk.process_and_validate_instruction(&instruction, &accounts, &checks);
    }

    #[test]
    fn test_initialize_immutable_program_rejected() {
        let admin = Pubkey::new_unique();
        let payer = Pubkey::new_unique();
        let authority = Pubkey::new_unique();

        let (access_manager_pda, _) =
            Pubkey::find_program_address(&[AccessManager::SEED], &crate::ID);
        let (program_data_pda, program_data_account) =
            create_program_data_account(&crate::ID, None);

        let instruction_data = crate::instruction::Initialize { admin };

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(access_manager_pda, false),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(system_program::ID, false),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
                AccountMeta::new_readonly(program_data_pda, false),
                AccountMeta::new_readonly(authority, true),
            ],
            data: instruction_data.data(),
        };

        let payer_lamports = 10_000_000_000;
        let accounts = vec![
            (
                access_manager_pda,
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
            (
                solana_sdk::sysvar::instructions::ID,
                create_instructions_sysvar_account(),
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

        let mollusk = Mollusk::new(&crate::ID, crate::get_access_manager_program_path());

        let checks = vec![Check::err(solana_sdk::program_error::ProgramError::Custom(
            ANCHOR_ERROR_OFFSET + AccessManagerError::UnauthorizedDeployer as u32,
        ))];

        mollusk.process_and_validate_instruction(&instruction, &accounts, &checks);
    }
    #[test]
    fn test_initialize_cross_program_data_rejected() {
        let admin = Pubkey::new_unique();
        let payer = Pubkey::new_unique();
        let authority = Pubkey::new_unique();
        let other_program_id = Pubkey::new_unique();

        let (access_manager_pda, _) =
            Pubkey::find_program_address(&[AccessManager::SEED], &crate::ID);
        // Derive ProgramData from a *different* program — should be rejected by the seeds constraint.
        let (wrong_program_data_pda, wrong_program_data_account) =
            create_program_data_account(&other_program_id, Some(authority));

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(access_manager_pda, false),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(system_program::ID, false),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
                AccountMeta::new_readonly(wrong_program_data_pda, false),
                AccountMeta::new_readonly(authority, true),
            ],
            data: crate::instruction::Initialize { admin }.data(),
        };

        let payer_lamports = 10_000_000_000;
        let accounts = vec![
            (
                access_manager_pda,
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
            (
                solana_sdk::sysvar::instructions::ID,
                create_instructions_sysvar_account(),
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

        let mollusk = Mollusk::new(&crate::ID, crate::get_access_manager_program_path());

        // Anchor ConstraintSeeds = 2006
        let checks = vec![Check::err(solana_sdk::program_error::ProgramError::Custom(
            2006,
        ))];

        mollusk.process_and_validate_instruction(&instruction, &accounts, &checks);
    }
}

#[cfg(test)]
mod integration_tests {
    use crate::test_utils::*;
    use anchor_lang::InstructionData;
    use solana_sdk::{
        bpf_loader_upgradeable::{self, UpgradeableLoaderState},
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        signature::Keypair,
        signer::Signer,
    };

    fn setup_program_test_without_am(authority: &Keypair) -> solana_program_test::ProgramTest {
        if std::env::var("SBF_OUT_DIR").is_err() {
            let deploy_dir = std::path::Path::new("../../target/deploy");
            std::env::set_var("SBF_OUT_DIR", deploy_dir);
        }

        let mut pt = solana_program_test::ProgramTest::new(
            crate::test_config::PROGRAM_BINARY_NAME,
            crate::ID,
            None,
        );
        pt.add_program("test_cpi_proxy", TEST_CPI_PROXY_ID, None);
        pt.add_program("test_cpi_target", TEST_CPI_TARGET_ID, None);

        let (program_data_pda, _) =
            Pubkey::find_program_address(&[crate::ID.as_ref()], &bpf_loader_upgradeable::ID);
        let state = UpgradeableLoaderState::ProgramData {
            slot: 0,
            upgrade_authority_address: Some(authority.pubkey()),
        };
        pt.add_account(
            program_data_pda,
            solana_sdk::account::Account {
                lamports: 1_000_000,
                data: bincode::serialize(&state).unwrap(),
                owner: bpf_loader_upgradeable::ID,
                executable: false,
                rent_epoch: 0,
            },
        );

        pt
    }

    fn build_initialize_ix(payer: Pubkey, admin: Pubkey, authority: Pubkey) -> Instruction {
        let (access_manager_pda, _) =
            Pubkey::find_program_address(&[crate::state::AccessManager::SEED], &crate::ID);
        let (program_data_pda, _) =
            Pubkey::find_program_address(&[crate::ID.as_ref()], &bpf_loader_upgradeable::ID);

        Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(access_manager_pda, false),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(solana_sdk::system_program::ID, false),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
                AccountMeta::new_readonly(program_data_pda, false),
                AccountMeta::new_readonly(authority, true),
            ],
            data: crate::instruction::Initialize { admin }.data(),
        }
    }

    #[tokio::test]
    async fn test_direct_call_succeeds() {
        let authority = Keypair::new();
        let pt = setup_program_test_without_am(&authority);
        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let admin = Pubkey::new_unique();
        let ix = build_initialize_ix(payer.pubkey(), admin, authority.pubkey());

        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[ix],
            Some(&payer.pubkey()),
            &[&payer, &authority],
            recent_blockhash,
        );
        let result = banks_client.process_transaction(tx).await;
        assert!(result.is_ok(), "Direct initialize should succeed");
    }

    #[tokio::test]
    async fn test_cpi_from_whitelisted_program_rejected() {
        let authority = Keypair::new();
        let pt = setup_program_test_without_am(&authority);
        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let admin = Pubkey::new_unique();
        let inner_ix = build_initialize_ix(payer.pubkey(), admin, authority.pubkey());
        let mut wrapped_ix = wrap_in_test_cpi_target_proxy(payer.pubkey(), &inner_ix);
        // Mark authority as signer in the outer instruction so the transaction
        // can include it as a signer and the runtime propagates its status via CPI.
        for meta in &mut wrapped_ix.accounts {
            if meta.pubkey == authority.pubkey() {
                meta.is_signer = true;
            }
        }

        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[wrapped_ix],
            Some(&payer.pubkey()),
            &[&payer, &authority],
            recent_blockhash,
        );
        let err = banks_client.process_transaction(tx).await.unwrap_err();
        assert_eq!(
            extract_custom_error(&err),
            Some(ANCHOR_ERROR_OFFSET + crate::errors::AccessManagerError::CpiNotAllowed as u32),
        );
    }

    #[tokio::test]
    async fn test_cpi_from_unauthorized_program_rejected() {
        let authority = Keypair::new();
        let pt = setup_program_test_without_am(&authority);
        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let admin = Pubkey::new_unique();
        let inner_ix = build_initialize_ix(payer.pubkey(), admin, authority.pubkey());
        let mut wrapped_ix = wrap_in_test_cpi_proxy(payer.pubkey(), &inner_ix);
        for meta in &mut wrapped_ix.accounts {
            if meta.pubkey == authority.pubkey() {
                meta.is_signer = true;
            }
        }

        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[wrapped_ix],
            Some(&payer.pubkey()),
            &[&payer, &authority],
            recent_blockhash,
        );
        let err = banks_client.process_transaction(tx).await.unwrap_err();
        assert_eq!(
            extract_custom_error(&err),
            Some(ANCHOR_ERROR_OFFSET + crate::errors::AccessManagerError::CpiNotAllowed as u32),
        );
    }
}
