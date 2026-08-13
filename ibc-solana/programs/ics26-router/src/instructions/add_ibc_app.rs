// SPDX-License-Identifier: Apache-2.0

use crate::errors::RouterError;
use crate::events::IBCAppAdded;
use crate::state::{AccountVersion, IBCApp, RouterState, MAX_PORT_ID_LENGTH, MIN_PORT_ID_LENGTH};
use crate::utils::validation::validate_custom_ibc_identifier;
use anchor_lang::prelude::*;

/// Registers an IBC application program under a given `port_id`.
#[derive(Accounts)]
#[instruction(port_id: String)]
pub struct AddIbcApp<'info> {
    /// Global router configuration PDA.
    #[account(
        seeds = [RouterState::SEED],
        bump
    )]
    pub router_state: Account<'info, RouterState>,

    /// Global access control state used for role verification.
    /// CHECK: Validated via seeds constraint using stored `access_manager` program ID
    #[account(
        seeds = [access_manager::state::AccessManager::SEED],
        bump,
        seeds::program = router_state.am_state.access_manager
    )]
    pub access_manager: AccountInfo<'info>,

    /// PDA mapping `port_id` to its IBC application program.
    #[account(
        init,
        payer = payer,
        space = 8 + IBCApp::INIT_SPACE,
        seeds = [IBCApp::SEED, port_id.as_bytes()],
        bump
    )]
    pub ibc_app: Account<'info, IBCApp>,

    /// IBC application program to register for this port.
    /// CHECK: No concrete type for `Program<'info, T>` since it can be any executable.
    #[account(constraint = app_program.executable @ RouterError::AppProgramNotExecutable)]
    pub app_program: UncheckedAccount<'info>,

    /// Pays for creating the `IBCApp` account.
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Signer with the `ID_CUSTOMIZER_ROLE`.
    pub authority: Signer<'info>,

    /// Solana system program used for account creation.
    pub system_program: Program<'info, System>,

    /// Instructions sysvar used for CPI detection.
    /// CHECK: Address constraint verifies this is the instructions sysvar
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instructions_sysvar: AccountInfo<'info>,
}

pub fn add_ibc_app(ctx: Context<AddIbcApp>, port_id: String) -> Result<()> {
    access_manager::require_role(
        &ctx.accounts.access_manager,
        solana_ibc_types::roles::ID_CUSTOMIZER_ROLE,
        &ctx.accounts.authority,
        &ctx.accounts.instructions_sysvar,
        &crate::ID,
    )?;

    require!(
        validate_custom_ibc_identifier(&port_id, MIN_PORT_ID_LENGTH, MAX_PORT_ID_LENGTH),
        RouterError::InvalidPortIdentifier
    );

    let ibc_app = &mut ctx.accounts.ibc_app;
    ibc_app.version = AccountVersion::V1;
    ibc_app.port_id = port_id;
    ibc_app.app_program_id = ctx.accounts.app_program.key();
    ibc_app._reserved = [0u8; 256];

    emit!(IBCAppAdded {
        port_id: ibc_app.port_id.clone(),
        app_program_id: ibc_app.app_program_id,
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;
    use anchor_lang::InstructionData;
    use mollusk_svm::result::Check;
    use mollusk_svm::Mollusk;
    use solana_ibc_types::roles;
    use solana_sdk::instruction::{AccountMeta, Instruction};
    use solana_sdk::program_error::ProgramError;
    use solana_sdk::pubkey::Pubkey;
    use solana_sdk::system_program;

    #[test]
    fn test_add_ibc_app_happy_path() {
        let authority = Pubkey::new_unique();
        let payer = authority;
        let port_id = "test-port";
        let app_program = Pubkey::new_unique();

        let (router_state_pda, router_state_data) = setup_router_state();

        let (access_manager_pda, access_manager_data) =
            setup_access_manager_with_roles(&[(roles::ID_CUSTOMIZER_ROLE, &[authority])]);

        let (ibc_app_pda, _) =
            Pubkey::find_program_address(&[IBCApp::SEED, port_id.as_bytes()], &crate::ID);

        let instruction_data = crate::instruction::AddIbcApp {
            port_id: port_id.to_string(),
        };

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new_readonly(router_state_pda, false),
                AccountMeta::new_readonly(access_manager_pda, false),
                AccountMeta::new(ibc_app_pda, false),
                AccountMeta::new_readonly(app_program, false),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(authority, true),
                AccountMeta::new_readonly(system_program::ID, false),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
            ],
            data: instruction_data.data(),
        };

        let accounts = vec![
            create_account(router_state_pda, router_state_data, crate::ID),
            create_account(access_manager_pda, access_manager_data, access_manager::ID),
            create_uninitialized_account(ibc_app_pda, 0),
            create_program_account(app_program),
            create_system_account(payer),
            create_system_account(authority),
            create_program_account(system_program::ID),
            create_instructions_sysvar_account_with_caller(crate::ID),
        ];

        let mollusk = Mollusk::new(&crate::ID, crate::test_utils::get_router_program_path());

        let checks = vec![
            Check::success(),
            Check::account(&ibc_app_pda).owner(&crate::ID).build(),
        ];

        mollusk.process_and_validate_instruction(&instruction, &accounts, &checks);
    }

    #[test]
    fn test_add_ibc_app_unauthorized_sender() {
        let authorized_user = Pubkey::new_unique();
        let unauthorized_sender = Pubkey::new_unique();
        let payer = unauthorized_sender;
        let port_id = "test-port";
        let app_program = Pubkey::new_unique();

        let (router_state_pda, router_state_data) = setup_router_state();

        // Setup access manager with authorized_user having ID_CUSTOMIZER_ROLE, but NOT unauthorized_sender
        let (access_manager_pda, access_manager_data) =
            setup_access_manager_with_roles(&[(roles::ID_CUSTOMIZER_ROLE, &[authorized_user])]);

        let (ibc_app_pda, _) =
            Pubkey::find_program_address(&[IBCApp::SEED, port_id.as_bytes()], &crate::ID);

        let instruction_data = crate::instruction::AddIbcApp {
            port_id: port_id.to_string(),
        };

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new_readonly(router_state_pda, false),
                AccountMeta::new_readonly(access_manager_pda, false),
                AccountMeta::new(ibc_app_pda, false),
                AccountMeta::new_readonly(app_program, false),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(unauthorized_sender, true),
                AccountMeta::new_readonly(system_program::ID, false),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
            ],
            data: instruction_data.data(),
        };

        let accounts = vec![
            create_account(router_state_pda, router_state_data, crate::ID),
            create_account(access_manager_pda, access_manager_data, access_manager::ID),
            create_uninitialized_account(ibc_app_pda, 0),
            create_program_account(app_program),
            create_system_account(payer),
            create_system_account(unauthorized_sender),
            create_program_account(system_program::ID),
            create_instructions_sysvar_account_with_caller(crate::ID),
        ];

        let mollusk = Mollusk::new(&crate::ID, crate::test_utils::get_router_program_path());

        let checks = vec![Check::err(ProgramError::Custom(
            ANCHOR_ERROR_OFFSET + RouterError::UnauthorizedSender as u32,
        ))];

        mollusk.process_and_validate_instruction(&instruction, &accounts, &checks);
    }

    fn test_add_ibc_app_with_port_id(port_id: &str) -> mollusk_svm::result::InstructionResult {
        let authority = Pubkey::new_unique();
        let payer = authority;
        let app_program = Pubkey::new_unique();

        let (router_state_pda, router_state_data) = setup_router_state();
        let (access_manager_pda, access_manager_data) =
            setup_access_manager_with_roles(&[(roles::ID_CUSTOMIZER_ROLE, &[authority])]);
        let (ibc_app_pda, _) =
            Pubkey::find_program_address(&[IBCApp::SEED, port_id.as_bytes()], &crate::ID);

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new_readonly(router_state_pda, false),
                AccountMeta::new_readonly(access_manager_pda, false),
                AccountMeta::new(ibc_app_pda, false),
                AccountMeta::new_readonly(app_program, false),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(authority, true),
                AccountMeta::new_readonly(system_program::ID, false),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
            ],
            data: crate::instruction::AddIbcApp {
                port_id: port_id.to_string(),
            }
            .data(),
        };

        let accounts = vec![
            create_account(router_state_pda, router_state_data, crate::ID),
            create_account(access_manager_pda, access_manager_data, access_manager::ID),
            create_uninitialized_account(ibc_app_pda, 0),
            create_program_account(app_program),
            create_system_account(payer),
            create_system_account(authority),
            create_program_account(system_program::ID),
            create_instructions_sysvar_account_with_caller(crate::ID),
        ];

        let mollusk = Mollusk::new(&crate::ID, crate::test_utils::get_router_program_path());
        mollusk.process_instruction(&instruction, &accounts)
    }

    #[rstest::rstest]
    #[case::empty("")]
    #[case::too_short("a")]
    #[case::invalid_chars("test@port")]
    #[case::reserved_prefix("client-myport")]
    fn test_add_ibc_app_invalid_port_id(#[case] port_id: &str) {
        let result = test_add_ibc_app_with_port_id(port_id);
        assert_error_code(
            result,
            RouterError::InvalidPortIdentifier,
            &format!("port_id={port_id:?}"),
        );
    }

    #[test]
    fn test_add_ibc_app_already_exists() {
        let authority = Pubkey::new_unique();
        let payer = authority;
        let port_id = "test-port";
        let app_program = Pubkey::new_unique();

        let (router_state_pda, router_state_data) = setup_router_state();
        let (access_manager_pda, access_manager_data) =
            setup_access_manager_with_roles(&[(roles::ID_CUSTOMIZER_ROLE, &[authority])]);
        // IBC app already exists
        let (ibc_app_pda, existing_ibc_app_data) = setup_ibc_app(port_id, Pubkey::new_unique());

        let instruction_data = crate::instruction::AddIbcApp {
            port_id: port_id.to_string(),
        };

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new_readonly(router_state_pda, false),
                AccountMeta::new_readonly(access_manager_pda, false),
                AccountMeta::new(ibc_app_pda, false),
                AccountMeta::new_readonly(app_program, false),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(authority, true),
                AccountMeta::new_readonly(system_program::ID, false),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
            ],
            data: instruction_data.data(),
        };

        let accounts = vec![
            create_account(router_state_pda, router_state_data, crate::ID),
            create_account(access_manager_pda, access_manager_data, access_manager::ID),
            create_account(ibc_app_pda, existing_ibc_app_data, crate::ID),
            create_program_account(app_program),
            create_system_account(payer),
            create_system_account(authority),
            create_program_account(system_program::ID),
            create_instructions_sysvar_account_with_caller(crate::ID),
        ];

        let mollusk = Mollusk::new(&crate::ID, crate::test_utils::get_router_program_path());

        // System program's Allocate fails with Custom(0) when account already in use
        let checks = vec![Check::err(ProgramError::Custom(0))];
        mollusk.process_and_validate_instruction(&instruction, &accounts, &checks);
    }

    #[test]
    fn test_add_ibc_app_fake_sysvar_wormhole_attack() {
        let authority = Pubkey::new_unique();
        let payer = authority;
        let port_id = "test-port";
        let app_program = Pubkey::new_unique();

        let (router_state_pda, router_state_data) = setup_router_state();

        let (access_manager_pda, access_manager_data) =
            setup_access_manager_with_roles(&[(roles::ID_CUSTOMIZER_ROLE, &[authority])]);

        let (ibc_app_pda, _) =
            Pubkey::find_program_address(&[IBCApp::SEED, port_id.as_bytes()], &crate::ID);

        let instruction_data = crate::instruction::AddIbcApp {
            port_id: port_id.to_string(),
        };

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new_readonly(router_state_pda, false),
                AccountMeta::new_readonly(access_manager_pda, false),
                AccountMeta::new(ibc_app_pda, false),
                AccountMeta::new_readonly(app_program, false),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(authority, true),
                AccountMeta::new_readonly(system_program::ID, false),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
            ],
            data: instruction_data.data(),
        };

        // Replace real sysvar with fake one (Wormhole-style attack)
        let (instruction, fake_sysvar_account) = setup_fake_sysvar_attack(instruction, crate::ID);

        let accounts = vec![
            create_account(router_state_pda, router_state_data, crate::ID),
            create_account(access_manager_pda, access_manager_data, access_manager::ID),
            create_uninitialized_account(ibc_app_pda, 0),
            create_program_account(app_program),
            create_system_account(payer),
            create_program_account(system_program::ID),
            fake_sysvar_account,
        ];

        let mollusk = Mollusk::new(&crate::ID, crate::test_utils::get_router_program_path());

        mollusk.process_and_validate_instruction(
            &instruction,
            &accounts,
            &[expect_sysvar_attack_error()],
        );
    }

    #[test]
    fn test_add_ibc_app_cpi_rejection() {
        let authority = Pubkey::new_unique();
        let payer = authority;
        let port_id = "test-port";
        let app_program = Pubkey::new_unique();

        let (router_state_pda, router_state_data) = setup_router_state();

        let (access_manager_pda, access_manager_data) =
            setup_access_manager_with_roles(&[(roles::ID_CUSTOMIZER_ROLE, &[authority])]);

        let (ibc_app_pda, _) =
            Pubkey::find_program_address(&[IBCApp::SEED, port_id.as_bytes()], &crate::ID);

        let instruction_data = crate::instruction::AddIbcApp {
            port_id: port_id.to_string(),
        };

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new_readonly(router_state_pda, false),
                AccountMeta::new_readonly(access_manager_pda, false),
                AccountMeta::new(ibc_app_pda, false),
                AccountMeta::new_readonly(app_program, false),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(authority, true),
                AccountMeta::new_readonly(system_program::ID, false),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
            ],
            data: instruction_data.data(),
        };

        // Simulate CPI call from unauthorized program
        let malicious_program = Pubkey::new_unique();
        let (instruction, cpi_sysvar_account) = setup_cpi_call_test(instruction, malicious_program);

        let accounts = vec![
            create_account(router_state_pda, router_state_data, crate::ID),
            create_account(access_manager_pda, access_manager_data, access_manager::ID),
            create_uninitialized_account(ibc_app_pda, 0),
            create_program_account(app_program),
            create_system_account(payer),
            create_system_account(authority),
            create_program_account(system_program::ID),
            cpi_sysvar_account,
        ];

        let mollusk = Mollusk::new(&crate::ID, crate::test_utils::get_router_program_path());

        // When CPI is detected by access_manager::require_role, it returns AccessManagerError::CpiNotAllowed (6005)
        let checks = vec![Check::err(ProgramError::Custom(
            ANCHOR_ERROR_OFFSET + access_manager::AccessManagerError::CpiNotAllowed as u32,
        ))];
        mollusk.process_and_validate_instruction(&instruction, &accounts, &checks);
    }

    #[test]
    fn test_add_ibc_app_non_executable_program() {
        let authority = Pubkey::new_unique();
        let payer = authority;
        let port_id = "test-port";
        let app_program = Pubkey::new_unique();

        let (router_state_pda, router_state_data) = setup_router_state();
        let (access_manager_pda, access_manager_data) =
            setup_access_manager_with_roles(&[(roles::ID_CUSTOMIZER_ROLE, &[authority])]);
        let (ibc_app_pda, _) =
            Pubkey::find_program_address(&[IBCApp::SEED, port_id.as_bytes()], &crate::ID);

        let instruction_data = crate::instruction::AddIbcApp {
            port_id: port_id.to_string(),
        };

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new_readonly(router_state_pda, false),
                AccountMeta::new_readonly(access_manager_pda, false),
                AccountMeta::new(ibc_app_pda, false),
                AccountMeta::new_readonly(app_program, false),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(authority, true),
                AccountMeta::new_readonly(system_program::ID, false),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
            ],
            data: instruction_data.data(),
        };

        let accounts = vec![
            create_account(router_state_pda, router_state_data, crate::ID),
            create_account(access_manager_pda, access_manager_data, access_manager::ID),
            create_uninitialized_account(ibc_app_pda, 0),
            create_account(app_program, vec![], system_program::ID),
            create_system_account(payer),
            create_system_account(authority),
            create_program_account(system_program::ID),
            create_instructions_sysvar_account_with_caller(crate::ID),
        ];

        let mollusk = Mollusk::new(&crate::ID, crate::test_utils::get_router_program_path());

        let checks = vec![Check::err(ProgramError::Custom(
            ANCHOR_ERROR_OFFSET + RouterError::AppProgramNotExecutable as u32,
        ))];
        mollusk.process_and_validate_instruction(&instruction, &accounts, &checks);
    }
}

#[cfg(test)]
mod integration_tests {
    use crate::state::{IBCApp, RouterState};
    use crate::test_utils::*;
    use anchor_lang::InstructionData;
    use solana_sdk::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        signature::Keypair,
        signer::Signer,
    };

    fn build_add_ibc_app_ix(payer: Pubkey, authority: Pubkey, port_id: &str) -> Instruction {
        let (router_state_pda, _) = Pubkey::find_program_address(&[RouterState::SEED], &crate::ID);
        let (access_manager_pda, _) = Pubkey::find_program_address(
            &[access_manager::state::AccessManager::SEED],
            &access_manager::ID,
        );
        let (ibc_app_pda, _) =
            Pubkey::find_program_address(&[IBCApp::SEED, port_id.as_bytes()], &crate::ID);

        Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new_readonly(router_state_pda, false),
                AccountMeta::new_readonly(access_manager_pda, false),
                AccountMeta::new(ibc_app_pda, false),
                AccountMeta::new_readonly(access_manager::ID, false),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(authority, true),
                AccountMeta::new_readonly(solana_sdk::system_program::ID, false),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
            ],
            data: crate::instruction::AddIbcApp {
                port_id: port_id.to_string(),
            }
            .data(),
        }
    }

    #[tokio::test]
    async fn test_add_ibc_app_direct_call_succeeds() {
        let admin = Keypair::new();
        let pt = setup_program_test_with_roles_and_whitelist(
            &[(
                solana_ibc_types::roles::ID_CUSTOMIZER_ROLE,
                &[admin.pubkey()],
            )],
            &[TEST_CPI_TARGET_ID],
        );
        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let ix = build_add_ibc_app_ix(payer.pubkey(), admin.pubkey(), "test-port");

        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[ix],
            Some(&payer.pubkey()),
            &[&payer, &admin],
            recent_blockhash,
        );
        let result = banks_client.process_transaction(tx).await;
        assert!(
            result.is_ok(),
            "Direct call with ID_CUSTOMIZER_ROLE should succeed: {:?}",
            result.err()
        );
    }

    #[tokio::test]
    async fn test_add_ibc_app_without_role_rejected() {
        let admin = Keypair::new();
        let non_admin = Keypair::new();
        let pt = setup_program_test_with_roles_and_whitelist(
            &[(
                solana_ibc_types::roles::ID_CUSTOMIZER_ROLE,
                &[admin.pubkey()],
            )],
            &[],
        );
        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let ix = build_add_ibc_app_ix(payer.pubkey(), non_admin.pubkey(), "test-port");

        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[ix],
            Some(&payer.pubkey()),
            &[&payer, &non_admin],
            recent_blockhash,
        );
        let err = banks_client.process_transaction(tx).await.unwrap_err();
        assert_eq!(
            pt_extract_custom_error(&err),
            Some(ANCHOR_ERROR_OFFSET + access_manager::AccessManagerError::Unauthorized as u32),
        );
    }

    #[tokio::test]
    async fn test_add_ibc_app_cpi_rejected() {
        let admin = Keypair::new();
        let pt = setup_program_test_with_roles_and_whitelist(
            &[(
                solana_ibc_types::roles::ID_CUSTOMIZER_ROLE,
                &[admin.pubkey()],
            )],
            &[TEST_CPI_TARGET_ID],
        );
        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let inner_ix = build_add_ibc_app_ix(payer.pubkey(), admin.pubkey(), "test-port");
        let wrapped_ix = pt_wrap_in_test_cpi_proxy(admin.pubkey(), &inner_ix);

        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[wrapped_ix],
            Some(&payer.pubkey()),
            &[&payer, &admin],
            recent_blockhash,
        );
        let err = banks_client.process_transaction(tx).await.unwrap_err();
        assert_eq!(
            pt_extract_custom_error(&err),
            Some(ANCHOR_ERROR_OFFSET + access_manager::AccessManagerError::CpiNotAllowed as u32),
        );
    }
}
