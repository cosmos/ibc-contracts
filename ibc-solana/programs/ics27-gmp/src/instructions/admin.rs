// SPDX-License-Identifier: Apache-2.0

use crate::events::{GMPAppPaused, GMPAppUnpaused};
use crate::state::GMPAppState;
use anchor_lang::prelude::*;

/// Pauses the entire GMP application. Requires the `PAUSER_ROLE`.
#[derive(Accounts)]
pub struct PauseApp<'info> {
    /// GMP program's global configuration PDA that holds the paused flag.
    #[account(
        mut,
        seeds = [GMPAppState::SEED],
        bump = app_state.bump
    )]
    pub app_state: Account<'info, GMPAppState>,

    /// Access manager account used for role-based authorization.
    /// CHECK: Validated via seeds constraint using stored `access_manager` program ID
    #[account(
        seeds = [access_manager::state::AccessManager::SEED],
        bump,
        seeds::program = app_state.am_state.access_manager
    )]
    pub access_manager: AccountInfo<'info>,

    /// Signer whose `PAUSER_ROLE` membership is verified by the access manager.
    pub authority: Signer<'info>,

    /// Instructions sysvar used to detect and reject unauthorized CPI callers.
    /// CHECK: Address constraint verifies this is the instructions sysvar
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instructions_sysvar: AccountInfo<'info>,
}

pub fn pause_app(ctx: Context<PauseApp>) -> Result<()> {
    access_manager::require_role(
        &ctx.accounts.access_manager,
        solana_ibc_types::roles::PAUSER_ROLE,
        &ctx.accounts.authority,
        &ctx.accounts.instructions_sysvar,
        &crate::ID,
    )?;

    let clock = Clock::get()?;
    let app_state = &mut ctx.accounts.app_state;

    app_state.paused = true;

    emit!(GMPAppPaused {
        admin: ctx.accounts.authority.key(),
        timestamp: clock.unix_timestamp,
    });

    msg!("GMP app paused by admin: {}", ctx.accounts.authority.key());

    Ok(())
}

/// Unpauses the entire GMP application. Requires the `UNPAUSER_ROLE`.
#[derive(Accounts)]
pub struct UnpauseApp<'info> {
    /// GMP program's global configuration PDA that holds the paused flag.
    #[account(
        mut,
        seeds = [GMPAppState::SEED],
        bump = app_state.bump
    )]
    pub app_state: Account<'info, GMPAppState>,

    /// Access manager account used for role-based authorization.
    /// CHECK: Validated via seeds constraint using stored `access_manager` program ID
    #[account(
        seeds = [access_manager::state::AccessManager::SEED],
        bump,
        seeds::program = app_state.am_state.access_manager
    )]
    pub access_manager: AccountInfo<'info>,

    /// Signer whose `UNPAUSER_ROLE` membership is verified by the access manager.
    pub authority: Signer<'info>,

    /// Instructions sysvar used to detect and reject unauthorized CPI callers.
    /// CHECK: Address constraint verifies this is the instructions sysvar
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instructions_sysvar: AccountInfo<'info>,
}

pub fn unpause_app(ctx: Context<UnpauseApp>) -> Result<()> {
    access_manager::require_role(
        &ctx.accounts.access_manager,
        solana_ibc_types::roles::UNPAUSER_ROLE,
        &ctx.accounts.authority,
        &ctx.accounts.instructions_sysvar,
        &crate::ID,
    )?;

    let clock = Clock::get()?;
    let app_state = &mut ctx.accounts.app_state;

    app_state.paused = false;

    emit!(GMPAppUnpaused {
        admin: ctx.accounts.authority.key(),
        timestamp: clock.unix_timestamp,
    });

    msg!(
        "GMP app unpaused by admin: {}",
        ctx.accounts.authority.key()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{AccountVersion, GMPAppState};
    use crate::test_utils::*;
    use access_manager::AccessManagerState;
    use anchor_lang::InstructionData;
    use mollusk_svm::result::Check;
    use mollusk_svm::Mollusk;
    use solana_ibc_types::roles;
    use solana_sdk::program_error::ProgramError;
    use solana_sdk::{
        account::Account as SolanaAccount,
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        system_program,
    };

    #[test]
    fn test_initialize_success() {
        let mollusk = Mollusk::new(&crate::ID, crate::get_gmp_program_path());

        let payer = Pubkey::new_unique();
        let authority = Pubkey::new_unique();
        let (app_state_pda, _bump) = Pubkey::find_program_address(&[GMPAppState::SEED], &crate::ID);
        let (program_data_pda, program_data_account) =
            create_program_data_account(&crate::ID, Some(authority));

        let instruction_data = crate::instruction::Initialize {
            access_manager: access_manager::ID,
        };

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(app_state_pda, false),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(system_program::ID, false),
                AccountMeta::new_readonly(program_data_pda, false),
                AccountMeta::new_readonly(authority, true),
            ],
            data: instruction_data.data(),
        };

        let accounts = vec![
            create_pda_for_init(app_state_pda),
            create_payer_account(payer),
            create_system_program_account(),
            (program_data_pda, program_data_account),
            create_payer_account(authority),
        ];

        let result = mollusk.process_instruction(&instruction, &accounts);
        assert!(!result.program_result.is_err(), "Initialize should succeed");
    }

    #[test]
    fn test_pause_app_success() {
        let mollusk = Mollusk::new(&crate::ID, crate::get_gmp_program_path());

        let authority = Pubkey::new_unique();
        let (access_manager_pda, access_manager_account) =
            setup_access_manager_with_roles(&[(roles::PAUSER_ROLE, &[authority])]);
        let (app_state_pda, app_state_bump) =
            Pubkey::find_program_address(&[GMPAppState::SEED], &crate::ID);

        let instruction_data = crate::instruction::PauseApp {};

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(app_state_pda, false),
                AccountMeta::new_readonly(access_manager_pda, false),
                AccountMeta::new_readonly(authority, true),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
            ],
            data: instruction_data.data(),
        };

        let accounts = vec![
            create_gmp_app_state_account(
                app_state_pda,
                app_state_bump,
                false, // not paused
            ),
            (access_manager_pda, access_manager_account),
            create_authority_account(authority),
            create_instructions_sysvar_account_with_caller(crate::ID),
        ];

        let result = mollusk.process_instruction(&instruction, &accounts);
        assert!(
            !result.program_result.is_err(),
            "Pause app should succeed: {:?}",
            result.program_result
        );

        let app_state_account = result.get_account(&app_state_pda).unwrap();
        let app_state = GMPAppState::try_deserialize(&mut &app_state_account.data[..]).unwrap();
        assert!(app_state.paused, "App should be paused");
    }

    #[test]
    fn test_unpause_app_success() {
        let mollusk = Mollusk::new(&crate::ID, crate::get_gmp_program_path());

        let authority = Pubkey::new_unique();
        let (access_manager_pda, access_manager_account) =
            setup_access_manager_with_roles(&[(roles::UNPAUSER_ROLE, &[authority])]);
        let (app_state_pda, app_state_bump) =
            Pubkey::find_program_address(&[GMPAppState::SEED], &crate::ID);

        let app_state = GMPAppState {
            version: AccountVersion::V1,
            paused: true,
            bump: app_state_bump,
            am_state: AccessManagerState::new(access_manager::ID),
            _reserved: [0; 256],
        };

        let mut data = Vec::new();
        data.extend_from_slice(GMPAppState::DISCRIMINATOR);
        app_state.serialize(&mut data).unwrap();

        let instruction_data = crate::instruction::UnpauseApp {};

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(app_state_pda, false),
                AccountMeta::new_readonly(access_manager_pda, false),
                AccountMeta::new_readonly(authority, true),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
            ],
            data: instruction_data.data(),
        };

        let accounts = vec![
            (
                app_state_pda,
                SolanaAccount {
                    lamports: 1_000_000,
                    data,
                    owner: crate::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
            (access_manager_pda, access_manager_account),
            create_authority_account(authority),
            create_instructions_sysvar_account_with_caller(crate::ID),
        ];

        let result = mollusk.process_instruction(&instruction, &accounts);
        assert!(
            !result.program_result.is_err(),
            "Unpause app should succeed: {:?}",
            result.program_result
        );

        let app_state_account = result.get_account(&app_state_pda).unwrap();
        let app_state = GMPAppState::try_deserialize(&mut &app_state_account.data[..]).unwrap();
        assert!(!app_state.paused, "App should be unpaused");
    }

    #[test]
    fn test_pause_app_unauthorized() {
        let mollusk = Mollusk::new(&crate::ID, crate::get_gmp_program_path());

        let authority = Pubkey::new_unique();
        let wrong_authority = Pubkey::new_unique();
        let (access_manager_pda, access_manager_account) =
            setup_access_manager_with_roles(&[(roles::PAUSER_ROLE, &[authority])]);
        let (app_state_pda, app_state_bump) =
            Pubkey::find_program_address(&[GMPAppState::SEED], &crate::ID);

        let instruction_data = crate::instruction::PauseApp {};

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(app_state_pda, false),
                AccountMeta::new_readonly(access_manager_pda, false),
                AccountMeta::new_readonly(wrong_authority, true),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
            ],
            data: instruction_data.data(),
        };

        let accounts = vec![
            create_gmp_app_state_account(
                app_state_pda,
                app_state_bump,
                false, // not paused
            ),
            (access_manager_pda, access_manager_account),
            create_authority_account(wrong_authority),
            create_instructions_sysvar_account(),
        ];

        let result = mollusk.process_instruction(&instruction, &accounts);
        assert!(
            result.program_result.is_err(),
            "Pause app should fail with wrong authority"
        );
    }

    // ========================================================================

    #[test]
    fn test_pause_app_invalid_pda() {
        let mollusk = Mollusk::new(&crate::ID, crate::get_gmp_program_path());

        let authority = Pubkey::new_unique();
        let (access_manager_pda, access_manager_account) =
            setup_access_manager_with_roles(&[(roles::PAUSER_ROLE, &[authority])]);
        let (_correct_app_state_pda, _correct_bump) =
            Pubkey::find_program_address(&[GMPAppState::SEED], &crate::ID);

        // Use wrong PDA
        let wrong_app_state_pda = Pubkey::new_unique();

        let instruction_data = crate::instruction::PauseApp {};

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(wrong_app_state_pda, false), // Wrong PDA!
                AccountMeta::new_readonly(access_manager_pda, false),
                AccountMeta::new_readonly(authority, true),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
            ],
            data: instruction_data.data(),
        };

        let accounts = vec![
            // Create account state at wrong PDA for testing
            create_gmp_app_state_account(
                wrong_app_state_pda,
                255u8,
                false, // not paused
            ),
            (access_manager_pda, access_manager_account),
            create_authority_account(authority),
            create_instructions_sysvar_account(),
        ];

        let result = mollusk.process_instruction(&instruction, &accounts);
        assert!(
            result.program_result.is_err(),
            "PauseApp should fail with invalid app_state PDA"
        );
    }

    #[test]
    fn test_pause_app_fake_sysvar_wormhole_attack() {
        let mollusk = Mollusk::new(&crate::ID, crate::get_gmp_program_path());

        let authority = Pubkey::new_unique();
        let (access_manager_pda, access_manager_account) =
            setup_access_manager_with_roles(&[(roles::PAUSER_ROLE, &[authority])]);
        let (app_state_pda, app_state_bump) =
            Pubkey::find_program_address(&[GMPAppState::SEED], &crate::ID);

        let instruction_data = crate::instruction::PauseApp {};

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(app_state_pda, false),
                AccountMeta::new_readonly(access_manager_pda, false),
                AccountMeta::new_readonly(authority, true),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
            ],
            data: instruction_data.data(),
        };

        // Replace real sysvar with fake one (Wormhole-style attack)
        let (instruction, fake_sysvar_account) = setup_fake_sysvar_attack(instruction, crate::ID);

        let accounts = vec![
            create_gmp_app_state_account(app_state_pda, app_state_bump, false),
            (access_manager_pda, access_manager_account),
            create_authority_account(authority),
            fake_sysvar_account,
        ];

        mollusk.process_and_validate_instruction(
            &instruction,
            &accounts,
            &[expect_sysvar_attack_error()],
        );
    }

    #[test]
    fn test_pause_app_cpi_rejection() {
        let mollusk = Mollusk::new(&crate::ID, crate::get_gmp_program_path());

        let authority = Pubkey::new_unique();
        let (access_manager_pda, access_manager_account) =
            setup_access_manager_with_roles(&[(roles::PAUSER_ROLE, &[authority])]);
        let (app_state_pda, app_state_bump) =
            Pubkey::find_program_address(&[GMPAppState::SEED], &crate::ID);

        let instruction_data = crate::instruction::PauseApp {};

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(app_state_pda, false),
                AccountMeta::new_readonly(access_manager_pda, false),
                AccountMeta::new_readonly(authority, true),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
            ],
            data: instruction_data.data(),
        };

        // Simulate CPI call from unauthorized program
        let malicious_program = Pubkey::new_unique();
        let (instruction, cpi_sysvar_account) = setup_cpi_call_test(instruction, malicious_program);

        let accounts = vec![
            create_gmp_app_state_account(app_state_pda, app_state_bump, false),
            (access_manager_pda, access_manager_account),
            create_authority_account(authority),
            cpi_sysvar_account,
        ];

        let checks = vec![Check::err(ProgramError::Custom(
            ANCHOR_ERROR_OFFSET + access_manager::AccessManagerError::CpiNotAllowed as u32,
        ))];
        mollusk.process_and_validate_instruction(&instruction, &accounts, &checks);
    }

    #[test]
    fn test_unpause_app_fake_sysvar_wormhole_attack() {
        let mollusk = Mollusk::new(&crate::ID, crate::get_gmp_program_path());

        let authority = Pubkey::new_unique();
        let (access_manager_pda, access_manager_account) =
            setup_access_manager_with_roles(&[(roles::UNPAUSER_ROLE, &[authority])]);
        let (app_state_pda, app_state_bump) =
            Pubkey::find_program_address(&[GMPAppState::SEED], &crate::ID);

        let instruction_data = crate::instruction::UnpauseApp {};

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(app_state_pda, false),
                AccountMeta::new_readonly(access_manager_pda, false),
                AccountMeta::new_readonly(authority, true),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
            ],
            data: instruction_data.data(),
        };

        // Replace real sysvar with fake one (Wormhole-style attack)
        let (instruction, fake_sysvar_account) = setup_fake_sysvar_attack(instruction, crate::ID);

        let accounts = vec![
            create_gmp_app_state_account(app_state_pda, app_state_bump, true), // paused
            (access_manager_pda, access_manager_account),
            create_authority_account(authority),
            fake_sysvar_account,
        ];

        mollusk.process_and_validate_instruction(
            &instruction,
            &accounts,
            &[expect_sysvar_attack_error()],
        );
    }

    #[test]
    fn test_unpause_app_cpi_rejection() {
        let mollusk = Mollusk::new(&crate::ID, crate::get_gmp_program_path());

        let authority = Pubkey::new_unique();
        let (access_manager_pda, access_manager_account) =
            setup_access_manager_with_roles(&[(roles::UNPAUSER_ROLE, &[authority])]);
        let (app_state_pda, app_state_bump) =
            Pubkey::find_program_address(&[GMPAppState::SEED], &crate::ID);

        let instruction_data = crate::instruction::UnpauseApp {};

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(app_state_pda, false),
                AccountMeta::new_readonly(access_manager_pda, false),
                AccountMeta::new_readonly(authority, true),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
            ],
            data: instruction_data.data(),
        };

        // Simulate CPI call from unauthorized program
        let malicious_program = Pubkey::new_unique();
        let (instruction, cpi_sysvar_account) = setup_cpi_call_test(instruction, malicious_program);

        let accounts = vec![
            create_gmp_app_state_account(app_state_pda, app_state_bump, true), // paused
            (access_manager_pda, access_manager_account),
            create_authority_account(authority),
            cpi_sysvar_account,
        ];

        let checks = vec![Check::err(ProgramError::Custom(
            ANCHOR_ERROR_OFFSET + access_manager::AccessManagerError::CpiNotAllowed as u32,
        ))];
        mollusk.process_and_validate_instruction(&instruction, &accounts, &checks);
    }
}

#[cfg(test)]
mod integration_tests {
    use crate::state::GMPAppState;
    use crate::test_utils::*;
    use access_manager::AccessManagerState;
    use anchor_lang::InstructionData;
    use solana_sdk::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        signature::Keypair,
        signer::Signer,
    };

    fn build_pause_app_ix(authority: Pubkey) -> Instruction {
        let (app_state_pda, _) = Pubkey::find_program_address(&[GMPAppState::SEED], &crate::ID);
        let (access_manager_pda, _) = Pubkey::find_program_address(
            &[access_manager::state::AccessManager::SEED],
            &access_manager::ID,
        );

        Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(app_state_pda, false),
                AccountMeta::new_readonly(access_manager_pda, false),
                AccountMeta::new_readonly(authority, true),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
            ],
            data: crate::instruction::PauseApp {}.data(),
        }
    }

    fn build_unpause_app_ix(authority: Pubkey) -> Instruction {
        let (app_state_pda, _) = Pubkey::find_program_address(&[GMPAppState::SEED], &crate::ID);
        let (access_manager_pda, _) = Pubkey::find_program_address(
            &[access_manager::state::AccessManager::SEED],
            &access_manager::ID,
        );

        Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(app_state_pda, false),
                AccountMeta::new_readonly(access_manager_pda, false),
                AccountMeta::new_readonly(authority, true),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
            ],
            data: crate::instruction::UnpauseApp {}.data(),
        }
    }

    fn setup_with_pauser_role(pauser: &Pubkey) -> solana_program_test::ProgramTest {
        use anchor_lang::{AnchorSerialize, Discriminator};

        if std::env::var("SBF_OUT_DIR").is_err() {
            std::env::set_var("SBF_OUT_DIR", std::path::Path::new("../../target/deploy"));
        }

        let mut pt = solana_program_test::ProgramTest::new("ics27_gmp", crate::ID, None);
        pt.add_program("test_cpi_proxy", TEST_CPI_PROXY_ID, None);
        pt.add_program("test_cpi_target", TEST_CPI_TARGET_ID, None);
        pt.add_program("ics26_router", ics26_router::ID, None);
        pt.add_program("access_manager", access_manager::ID, None);

        // Pre-create GMP app_state PDA
        let (app_state_pda, bump) = Pubkey::find_program_address(&[GMPAppState::SEED], &crate::ID);
        let app_state = GMPAppState {
            version: crate::state::AccountVersion::V1,
            paused: false,
            bump,
            am_state: AccessManagerState::new(access_manager::ID),
            _reserved: [0; 256],
        };
        let mut data = Vec::new();
        data.extend_from_slice(GMPAppState::DISCRIMINATOR);
        app_state.serialize(&mut data).unwrap();

        pt.add_account(
            app_state_pda,
            solana_sdk::account::Account {
                lamports: 1_000_000,
                data,
                owner: crate::ID,
                executable: false,
                rent_epoch: 0,
            },
        );

        // Pre-create AccessManager PDA with pauser + unpauser roles
        let (access_manager_pda, _) =
            solana_ibc_types::access_manager::AccessManager::pda(access_manager::ID);

        let am = access_manager::state::AccessManager {
            roles: vec![
                access_manager::RoleData {
                    role_id: solana_ibc_types::roles::ADMIN_ROLE,
                    members: vec![Pubkey::new_unique()],
                },
                access_manager::RoleData {
                    role_id: solana_ibc_types::roles::PAUSER_ROLE,
                    members: vec![*pauser],
                },
                access_manager::RoleData {
                    role_id: solana_ibc_types::roles::UNPAUSER_ROLE,
                    members: vec![*pauser],
                },
            ],
            whitelisted_programs: vec![],
            pending_authority_transfers: vec![],
        };
        let mut am_data = access_manager::state::AccessManager::DISCRIMINATOR.to_vec();
        am.serialize(&mut am_data).unwrap();

        pt.add_account(
            access_manager_pda,
            solana_sdk::account::Account {
                lamports: 1_000_000,
                data: am_data,
                owner: access_manager::ID,
                executable: false,
                rent_epoch: 0,
            },
        );

        pt
    }

    #[tokio::test]
    async fn test_pause_direct_call_succeeds() {
        let pauser = Keypair::new();
        let pt = setup_with_pauser_role(&pauser.pubkey());
        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let ix = build_pause_app_ix(pauser.pubkey());
        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[ix],
            Some(&payer.pubkey()),
            &[&payer, &pauser],
            recent_blockhash,
        );
        let result = banks_client.process_transaction(tx).await;
        assert!(result.is_ok(), "Direct pause by pauser should succeed");
    }

    #[tokio::test]
    async fn test_pause_without_role_rejected() {
        let pauser = Keypair::new();
        let non_pauser = Keypair::new();
        let pt = setup_with_pauser_role(&pauser.pubkey());
        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let ix = build_pause_app_ix(non_pauser.pubkey());
        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[ix],
            Some(&payer.pubkey()),
            &[&payer, &non_pauser],
            recent_blockhash,
        );
        let err = banks_client.process_transaction(tx).await.unwrap_err();
        assert_eq!(
            extract_custom_error(&err),
            Some(ANCHOR_ERROR_OFFSET + access_manager::AccessManagerError::Unauthorized as u32),
        );
    }

    #[tokio::test]
    async fn test_pause_cpi_rejected() {
        let pauser = Keypair::new();
        let pt = setup_with_pauser_role(&pauser.pubkey());
        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let inner_ix = build_pause_app_ix(pauser.pubkey());
        let wrapped_ix = wrap_in_test_cpi_proxy(pauser.pubkey(), &inner_ix);

        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[wrapped_ix],
            Some(&payer.pubkey()),
            &[&payer, &pauser],
            recent_blockhash,
        );
        let err = banks_client.process_transaction(tx).await.unwrap_err();
        assert_eq!(
            extract_custom_error(&err),
            Some(ANCHOR_ERROR_OFFSET + access_manager::AccessManagerError::CpiNotAllowed as u32),
        );
    }

    #[tokio::test]
    async fn test_unpause_direct_call_succeeds() {
        let pauser = Keypair::new();
        let pt = setup_with_pauser_role(&pauser.pubkey());
        let (banks_client, payer, recent_blockhash) = pt.start().await;

        // Pause first
        let pause_ix = build_pause_app_ix(pauser.pubkey());
        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[pause_ix],
            Some(&payer.pubkey()),
            &[&payer, &pauser],
            recent_blockhash,
        );
        banks_client.process_transaction(tx).await.unwrap();

        // Then unpause
        let unpause_ix = build_unpause_app_ix(pauser.pubkey());
        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[unpause_ix],
            Some(&payer.pubkey()),
            &[&payer, &pauser],
            recent_blockhash,
        );
        let result = banks_client.process_transaction(tx).await;
        assert!(result.is_ok(), "Direct unpause by unpauser should succeed");
    }

    #[tokio::test]
    async fn test_unpause_without_role_rejected() {
        let pauser = Keypair::new();
        let non_unpauser = Keypair::new();
        let pt = setup_with_pauser_role(&pauser.pubkey());
        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let ix = build_unpause_app_ix(non_unpauser.pubkey());
        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[ix],
            Some(&payer.pubkey()),
            &[&payer, &non_unpauser],
            recent_blockhash,
        );
        let err = banks_client.process_transaction(tx).await.unwrap_err();
        assert_eq!(
            extract_custom_error(&err),
            Some(ANCHOR_ERROR_OFFSET + access_manager::AccessManagerError::Unauthorized as u32),
        );
    }

    #[tokio::test]
    async fn test_unpause_cpi_rejected() {
        let pauser = Keypair::new();
        let pt = setup_with_pauser_role(&pauser.pubkey());
        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let inner_ix = build_unpause_app_ix(pauser.pubkey());
        let wrapped_ix = wrap_in_test_cpi_proxy(pauser.pubkey(), &inner_ix);

        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[wrapped_ix],
            Some(&payer.pubkey()),
            &[&payer, &pauser],
            recent_blockhash,
        );
        let err = banks_client.process_transaction(tx).await.unwrap_err();
        assert_eq!(
            extract_custom_error(&err),
            Some(ANCHOR_ERROR_OFFSET + access_manager::AccessManagerError::CpiNotAllowed as u32),
        );
    }
}
