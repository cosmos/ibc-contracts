// SPDX-License-Identifier: Apache-2.0

use crate::state::RouterState;
use anchor_lang::prelude::*;

/// Proposes transferring the access manager to a new program.
/// Requires `ADMIN_ROLE` on the current access manager.
#[derive(Accounts)]
#[instruction(new_access_manager: Pubkey)]
pub struct ProposeAccessManagerTransfer<'info> {
    /// PDA holding router-level settings including the current `access_manager`.
    #[account(mut, seeds = [RouterState::SEED], bump)]
    pub router_state: Account<'info, RouterState>,

    /// Current access-manager state PDA used to verify the caller holds the admin role.
    /// CHECK: Validated via seeds constraint using the stored `access_manager` program ID
    #[account(
        seeds = [access_manager::state::AccessManager::SEED],
        bump,
        seeds::program = router_state.am_state.access_manager
    )]
    pub access_manager: AccountInfo<'info>,

    /// Admin signer authorized to propose the transfer.
    pub admin: Signer<'info>,

    /// Instructions sysvar used by the access manager to inspect the transaction.
    /// CHECK: Address constraint verifies this is the instructions sysvar
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instructions_sysvar: AccountInfo<'info>,
}

pub fn propose_access_manager_transfer(
    ctx: Context<ProposeAccessManagerTransfer>,
    new_access_manager: Pubkey,
) -> Result<()> {
    ctx.accounts.router_state.am_state.propose_transfer(
        new_access_manager,
        &ctx.accounts.access_manager,
        &ctx.accounts.admin,
        &ctx.accounts.instructions_sysvar,
        &crate::ID,
    )
}

/// Accepts a pending access manager transfer.
/// Requires `ADMIN_ROLE` on the **new** access manager.
#[derive(Accounts)]
pub struct AcceptAccessManagerTransfer<'info> {
    /// PDA holding router-level settings; the `access_manager` field is updated on success.
    #[account(
        mut,
        seeds = [RouterState::SEED],
        bump,
        constraint = router_state.am_state.pending_access_manager.is_some()
            @ access_manager::AccessManagerError::NoPendingAccessManagerTransfer
    )]
    pub router_state: Account<'info, RouterState>,

    /// Proposed access-manager state PDA derived from the pending program ID.
    /// CHECK: Validated via seeds constraint against `pending_access_manager`
    #[account(
        seeds = [access_manager::state::AccessManager::SEED],
        bump,
        seeds::program = router_state.am_state.pending_access_manager.unwrap()
    )]
    pub new_am_state: AccountInfo<'info>,

    /// Admin signer authorized on the **new** access manager.
    pub admin: Signer<'info>,

    /// Instructions sysvar used by the access manager to inspect the transaction.
    /// CHECK: Address constraint verifies this is the instructions sysvar
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instructions_sysvar: AccountInfo<'info>,
}

pub fn accept_access_manager_transfer(ctx: Context<AcceptAccessManagerTransfer>) -> Result<()> {
    ctx.accounts.router_state.am_state.accept_transfer(
        &ctx.accounts.new_am_state,
        &ctx.accounts.admin,
        &ctx.accounts.instructions_sysvar,
        &crate::ID,
    )
}

/// Cancels a pending access manager transfer.
/// Requires `ADMIN_ROLE` on the current access manager.
#[derive(Accounts)]
pub struct CancelAccessManagerTransfer<'info> {
    /// PDA holding router-level settings; the pending transfer is cleared on success.
    #[account(mut, seeds = [RouterState::SEED], bump)]
    pub router_state: Account<'info, RouterState>,

    /// Current access-manager state PDA used to verify the caller holds the admin role.
    /// CHECK: Validated via seeds constraint using the stored `access_manager` program ID
    #[account(
        seeds = [access_manager::state::AccessManager::SEED],
        bump,
        seeds::program = router_state.am_state.access_manager
    )]
    pub am_state: AccountInfo<'info>,

    /// Admin signer authorized to cancel the transfer.
    pub admin: Signer<'info>,

    /// Instructions sysvar used by the access manager to inspect the transaction.
    /// CHECK: Address constraint verifies this is the instructions sysvar
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instructions_sysvar: AccountInfo<'info>,
}

pub fn cancel_access_manager_transfer(ctx: Context<CancelAccessManagerTransfer>) -> Result<()> {
    ctx.accounts.router_state.am_state.cancel_transfer(
        &ctx.accounts.am_state,
        &ctx.accounts.admin,
        &ctx.accounts.instructions_sysvar,
        &crate::ID,
    )
}

#[cfg(test)]
mod tests {
    use crate::test_utils::*;
    use access_manager::{AccessManagerError, AccessManagerState};
    use mollusk_svm::result::Check;
    use solana_ibc_types::roles;
    use solana_sdk::instruction::AccountMeta;
    use solana_sdk::pubkey::Pubkey;

    fn build_propose_instruction(
        admin: Pubkey,
        new_access_manager: Pubkey,
    ) -> solana_sdk::instruction::Instruction {
        let (router_state_pda, _) =
            Pubkey::find_program_address(&[crate::state::RouterState::SEED], &crate::ID);
        let (access_manager_pda, _) = Pubkey::find_program_address(
            &[access_manager::state::AccessManager::SEED],
            &access_manager::ID,
        );

        build_instruction(
            crate::instruction::ProposeAccessManagerTransfer { new_access_manager },
            vec![
                AccountMeta::new(router_state_pda, false),
                AccountMeta::new_readonly(access_manager_pda, false),
                AccountMeta::new_readonly(admin, true),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
            ],
        )
    }

    fn build_cancel_instruction(admin: Pubkey) -> solana_sdk::instruction::Instruction {
        let (router_state_pda, _) =
            Pubkey::find_program_address(&[crate::state::RouterState::SEED], &crate::ID);
        let (access_manager_pda, _) = Pubkey::find_program_address(
            &[access_manager::state::AccessManager::SEED],
            &access_manager::ID,
        );

        build_instruction(
            crate::instruction::CancelAccessManagerTransfer {},
            vec![
                AccountMeta::new(router_state_pda, false),
                AccountMeta::new_readonly(access_manager_pda, false),
                AccountMeta::new_readonly(admin, true),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
            ],
        )
    }

    fn create_router_state_with_pending(
        pending: Option<Pubkey>,
    ) -> (Pubkey, solana_sdk::account::Account) {
        let (pda, _) = Pubkey::find_program_address(&[crate::state::RouterState::SEED], &crate::ID);
        let state = crate::state::RouterState {
            version: crate::state::AccountVersion::V1,
            am_state: pending.map_or_else(
                || AccessManagerState::new(access_manager::ID),
                |pending| AccessManagerState {
                    access_manager: access_manager::ID,
                    pending_access_manager: Some(pending),
                },
            ),
            paused: false,
            _reserved: [0; 256],
        };
        let data = create_account_data(&state);
        (
            pda,
            solana_sdk::account::Account {
                lamports: 1_000_000,
                data,
                owner: crate::ID,
                executable: false,
                rent_epoch: 0,
            },
        )
    }

    // ── Propose tests ──

    #[test]
    fn test_propose_succeeds() {
        let admin = Pubkey::new_unique();
        let new_am = Pubkey::new_unique();

        let (router_state_pda, router_state_account) = create_initialized_router_state();
        let (am_pda, am_account) = create_access_manager_with_role(admin, roles::ADMIN_ROLE, admin);

        let instruction = build_propose_instruction(admin, new_am);
        let accounts = vec![
            (router_state_pda, router_state_account),
            (am_pda, am_account),
            (admin, create_signer_account()),
            create_instructions_sysvar_account_with_caller(crate::ID),
        ];

        let mollusk = setup_mollusk();
        let result =
            mollusk.process_and_validate_instruction(&instruction, &accounts, &[Check::success()]);

        let state = get_router_state_from_result(&result, &router_state_pda);
        assert_eq!(state.am_state.pending_access_manager, Some(new_am));
        assert_eq!(state.am_state.access_manager, access_manager::ID);
    }

    #[test]
    fn test_propose_not_admin_fails() {
        let admin = Pubkey::new_unique();
        let non_admin = Pubkey::new_unique();
        let new_am = Pubkey::new_unique();

        let (router_state_pda, router_state_account) = create_initialized_router_state();
        let (am_pda, am_account) = create_access_manager_with_role(admin, roles::ADMIN_ROLE, admin);

        let instruction = build_propose_instruction(non_admin, new_am);
        let accounts = vec![
            (router_state_pda, router_state_account),
            (am_pda, am_account),
            (non_admin, create_signer_account()),
            create_instructions_sysvar_account_with_caller(crate::ID),
        ];

        let mollusk = setup_mollusk();
        mollusk.process_and_validate_instruction(
            &instruction,
            &accounts,
            &[Check::err(solana_sdk::program_error::ProgramError::Custom(
                ANCHOR_ERROR_OFFSET + AccessManagerError::Unauthorized as u32,
            ))],
        );
    }

    #[test]
    fn test_propose_zero_address_fails() {
        let admin = Pubkey::new_unique();

        let (router_state_pda, router_state_account) = create_initialized_router_state();
        let (am_pda, am_account) = create_access_manager_with_role(admin, roles::ADMIN_ROLE, admin);

        let instruction = build_propose_instruction(admin, Pubkey::default());
        let accounts = vec![
            (router_state_pda, router_state_account),
            (am_pda, am_account),
            (admin, create_signer_account()),
            create_instructions_sysvar_account_with_caller(crate::ID),
        ];

        let mollusk = setup_mollusk();
        mollusk.process_and_validate_instruction(
            &instruction,
            &accounts,
            &[Check::err(solana_sdk::program_error::ProgramError::Custom(
                ANCHOR_ERROR_OFFSET + AccessManagerError::InvalidProposedAccessManager as u32,
            ))],
        );
    }

    #[test]
    fn test_propose_self_transfer_fails() {
        let admin = Pubkey::new_unique();

        let (router_state_pda, router_state_account) = create_initialized_router_state();
        let (am_pda, am_account) = create_access_manager_with_role(admin, roles::ADMIN_ROLE, admin);

        let instruction = build_propose_instruction(admin, access_manager::ID);
        let accounts = vec![
            (router_state_pda, router_state_account),
            (am_pda, am_account),
            (admin, create_signer_account()),
            create_instructions_sysvar_account_with_caller(crate::ID),
        ];

        let mollusk = setup_mollusk();
        mollusk.process_and_validate_instruction(
            &instruction,
            &accounts,
            &[Check::err(solana_sdk::program_error::ProgramError::Custom(
                ANCHOR_ERROR_OFFSET + AccessManagerError::AccessManagerSelfTransfer as u32,
            ))],
        );
    }

    #[test]
    fn test_propose_already_pending_fails() {
        let admin = Pubkey::new_unique();
        let new_am = Pubkey::new_unique();

        let (router_state_pda, router_state_account) =
            create_router_state_with_pending(Some(Pubkey::new_unique()));
        let (am_pda, am_account) = create_access_manager_with_role(admin, roles::ADMIN_ROLE, admin);

        let instruction = build_propose_instruction(admin, new_am);
        let accounts = vec![
            (router_state_pda, router_state_account),
            (am_pda, am_account),
            (admin, create_signer_account()),
            create_instructions_sysvar_account_with_caller(crate::ID),
        ];

        let mollusk = setup_mollusk();
        mollusk.process_and_validate_instruction(
            &instruction,
            &accounts,
            &[Check::err(solana_sdk::program_error::ProgramError::Custom(
                ANCHOR_ERROR_OFFSET
                    + AccessManagerError::PendingAccessManagerTransferAlreadyExists as u32,
            ))],
        );
    }

    #[test]
    fn test_propose_fake_sysvar_fails() {
        let admin = Pubkey::new_unique();
        let new_am = Pubkey::new_unique();

        let (router_state_pda, router_state_account) = create_initialized_router_state();
        let (am_pda, am_account) = create_access_manager_with_role(admin, roles::ADMIN_ROLE, admin);

        let instruction = build_propose_instruction(admin, new_am);
        let (instruction, fake_sysvar_account) = setup_fake_sysvar_attack(instruction, crate::ID);

        let accounts = vec![
            (router_state_pda, router_state_account),
            (am_pda, am_account),
            (admin, create_signer_account()),
            fake_sysvar_account,
        ];

        let mollusk = setup_mollusk();
        mollusk.process_and_validate_instruction(
            &instruction,
            &accounts,
            &[expect_sysvar_attack_error()],
        );
    }

    #[test]
    fn test_propose_cpi_rejection() {
        let admin = Pubkey::new_unique();
        let new_am = Pubkey::new_unique();

        let (router_state_pda, router_state_account) = create_initialized_router_state();
        let (am_pda, am_account) = create_access_manager_with_role(admin, roles::ADMIN_ROLE, admin);

        let instruction = build_propose_instruction(admin, new_am);
        let malicious_program = Pubkey::new_unique();
        let (instruction, cpi_sysvar_account) = setup_cpi_call_test(instruction, malicious_program);

        let accounts = vec![
            (router_state_pda, router_state_account),
            (am_pda, am_account),
            (admin, create_signer_account()),
            cpi_sysvar_account,
        ];

        let mollusk = setup_mollusk();
        mollusk.process_and_validate_instruction(
            &instruction,
            &accounts,
            &[Check::err(solana_sdk::program_error::ProgramError::Custom(
                ANCHOR_ERROR_OFFSET + AccessManagerError::CpiNotAllowed as u32,
            ))],
        );
    }

    // ── Cancel tests ──

    #[test]
    fn test_cancel_succeeds() {
        let admin = Pubkey::new_unique();
        let pending = Pubkey::new_unique();

        let (router_state_pda, router_state_account) =
            create_router_state_with_pending(Some(pending));
        let (am_pda, am_account) = create_access_manager_with_role(admin, roles::ADMIN_ROLE, admin);

        let instruction = build_cancel_instruction(admin);
        let accounts = vec![
            (router_state_pda, router_state_account),
            (am_pda, am_account),
            (admin, create_signer_account()),
            create_instructions_sysvar_account_with_caller(crate::ID),
        ];

        let mollusk = setup_mollusk();
        let result =
            mollusk.process_and_validate_instruction(&instruction, &accounts, &[Check::success()]);

        let state = get_router_state_from_result(&result, &router_state_pda);
        assert_eq!(state.am_state.pending_access_manager, None);
    }

    #[test]
    fn test_cancel_not_admin_fails() {
        let admin = Pubkey::new_unique();
        let non_admin = Pubkey::new_unique();

        let (router_state_pda, router_state_account) =
            create_router_state_with_pending(Some(Pubkey::new_unique()));
        let (am_pda, am_account) = create_access_manager_with_role(admin, roles::ADMIN_ROLE, admin);

        let instruction = build_cancel_instruction(non_admin);
        let accounts = vec![
            (router_state_pda, router_state_account),
            (am_pda, am_account),
            (non_admin, create_signer_account()),
            create_instructions_sysvar_account_with_caller(crate::ID),
        ];

        let mollusk = setup_mollusk();
        mollusk.process_and_validate_instruction(
            &instruction,
            &accounts,
            &[Check::err(solana_sdk::program_error::ProgramError::Custom(
                ANCHOR_ERROR_OFFSET + AccessManagerError::Unauthorized as u32,
            ))],
        );
    }

    #[test]
    fn test_cancel_no_pending_fails() {
        let admin = Pubkey::new_unique();

        let (router_state_pda, router_state_account) = create_initialized_router_state();
        let (am_pda, am_account) = create_access_manager_with_role(admin, roles::ADMIN_ROLE, admin);

        let instruction = build_cancel_instruction(admin);
        let accounts = vec![
            (router_state_pda, router_state_account),
            (am_pda, am_account),
            (admin, create_signer_account()),
            create_instructions_sysvar_account_with_caller(crate::ID),
        ];

        let mollusk = setup_mollusk();
        mollusk.process_and_validate_instruction(
            &instruction,
            &accounts,
            &[Check::err(solana_sdk::program_error::ProgramError::Custom(
                ANCHOR_ERROR_OFFSET + AccessManagerError::NoPendingAccessManagerTransfer as u32,
            ))],
        );
    }
}

#[cfg(test)]
mod integration_tests {
    use crate::test_utils::*;
    use access_manager::AccessManagerState;
    use anchor_lang::{Discriminator, InstructionData};
    use solana_sdk::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        signature::Keypair,
        signer::Signer,
    };

    fn build_propose_ix(admin: Pubkey, new_access_manager: Pubkey) -> Instruction {
        let (router_state_pda, _) =
            Pubkey::find_program_address(&[crate::state::RouterState::SEED], &crate::ID);
        let (access_manager_pda, _) = Pubkey::find_program_address(
            &[access_manager::state::AccessManager::SEED],
            &access_manager::ID,
        );

        Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(router_state_pda, false),
                AccountMeta::new_readonly(access_manager_pda, false),
                AccountMeta::new_readonly(admin, true),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
            ],
            data: crate::instruction::ProposeAccessManagerTransfer { new_access_manager }.data(),
        }
    }

    fn build_cancel_ix(admin: Pubkey) -> Instruction {
        let (router_state_pda, _) =
            Pubkey::find_program_address(&[crate::state::RouterState::SEED], &crate::ID);
        let (access_manager_pda, _) = Pubkey::find_program_address(
            &[access_manager::state::AccessManager::SEED],
            &access_manager::ID,
        );

        Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(router_state_pda, false),
                AccountMeta::new_readonly(access_manager_pda, false),
                AccountMeta::new_readonly(admin, true),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
            ],
            data: crate::instruction::CancelAccessManagerTransfer {}.data(),
        }
    }

    fn build_accept_ix(admin: Pubkey, new_am_pda: Pubkey) -> Instruction {
        let (router_state_pda, _) =
            Pubkey::find_program_address(&[crate::state::RouterState::SEED], &crate::ID);

        Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(router_state_pda, false),
                AccountMeta::new_readonly(new_am_pda, false),
                AccountMeta::new_readonly(admin, true),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
            ],
            data: crate::instruction::AcceptAccessManagerTransfer {}.data(),
        }
    }

    #[tokio::test]
    async fn test_propose_direct_call_succeeds() {
        let admin = Keypair::new();
        let pt = setup_program_test_with_whitelist(&admin.pubkey(), &[TEST_CPI_TARGET_ID]);
        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let ix = build_propose_ix(admin.pubkey(), Pubkey::new_unique());

        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[ix],
            Some(&payer.pubkey()),
            &[&payer, &admin],
            recent_blockhash,
        );
        let result = banks_client.process_transaction(tx).await;
        assert!(result.is_ok(), "Direct propose by admin should succeed");
    }

    #[tokio::test]
    async fn test_propose_non_admin_rejected() {
        let admin = Keypair::new();
        let non_admin = Keypair::new();
        let pt = setup_program_test_with_whitelist(&admin.pubkey(), &[]);
        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let ix = build_propose_ix(non_admin.pubkey(), Pubkey::new_unique());

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
    async fn test_propose_whitelisted_cpi_succeeds() {
        let admin = Keypair::new();
        let pt = setup_program_test_with_whitelist(&admin.pubkey(), &[TEST_CPI_TARGET_ID]);
        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let inner_ix = build_propose_ix(admin.pubkey(), Pubkey::new_unique());
        let wrapped_ix = pt_wrap_in_test_cpi_target_proxy(admin.pubkey(), &inner_ix);

        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[wrapped_ix],
            Some(&payer.pubkey()),
            &[&payer, &admin],
            recent_blockhash,
        );
        let result = banks_client.process_transaction(tx).await;
        assert!(
            result.is_ok(),
            "Whitelisted CPI propose should succeed: {:?}",
            result.err()
        );
    }

    #[tokio::test]
    async fn test_propose_unauthorized_cpi_rejected() {
        let admin = Keypair::new();
        let pt = setup_program_test_with_whitelist(&admin.pubkey(), &[TEST_CPI_TARGET_ID]);
        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let inner_ix = build_propose_ix(admin.pubkey(), Pubkey::new_unique());
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

    #[tokio::test]
    async fn test_accept_succeeds() {
        use access_manager::state::AccessManager;
        use anchor_lang::AnchorSerialize;

        let admin = Keypair::new();
        let new_am_program_id = Pubkey::new_unique();
        let (new_am_pda, _) =
            Pubkey::find_program_address(&[AccessManager::SEED], &new_am_program_id);

        // Setup ProgramTest with a router state that has a pending transfer
        if std::env::var("SBF_OUT_DIR").is_err() {
            std::env::set_var("SBF_OUT_DIR", std::path::Path::new("../../target/deploy"));
        }
        let mut pt = solana_program_test::ProgramTest::new("ics26_router", crate::ID, None);
        pt.add_program("access_manager", access_manager::ID, None);

        // Router state with pending transfer
        let (router_state_pda, _) =
            Pubkey::find_program_address(&[crate::state::RouterState::SEED], &crate::ID);
        let router_state = crate::state::RouterState {
            version: crate::state::AccountVersion::V1,
            am_state: AccessManagerState {
                access_manager: access_manager::ID,
                pending_access_manager: Some(new_am_program_id),
            },
            paused: false,
            _reserved: [0; 256],
        };
        let router_data = create_account_data(&router_state);
        pt.add_account(
            router_state_pda,
            solana_sdk::account::Account {
                lamports: 1_000_000,
                data: router_data,
                owner: crate::ID,
                executable: false,
                rent_epoch: 0,
            },
        );

        // New AM account with admin role for the admin keypair
        let new_am = AccessManager {
            roles: vec![access_manager::RoleData {
                role_id: solana_ibc_types::roles::ADMIN_ROLE,
                members: vec![admin.pubkey()],
            }],
            whitelisted_programs: vec![],
            pending_authority_transfers: vec![],
        };
        let mut am_data = AccessManager::DISCRIMINATOR.to_vec();
        new_am.serialize(&mut am_data).unwrap();
        pt.add_account(
            new_am_pda,
            solana_sdk::account::Account {
                lamports: 1_000_000,
                data: am_data,
                owner: new_am_program_id,
                executable: false,
                rent_epoch: 0,
            },
        );

        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let ix = build_accept_ix(admin.pubkey(), new_am_pda);
        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[ix],
            Some(&payer.pubkey()),
            &[&payer, &admin],
            recent_blockhash,
        );
        banks_client
            .process_transaction(tx)
            .await
            .expect("accept should succeed");

        // Verify state updated
        let account = banks_client
            .get_account(router_state_pda)
            .await
            .unwrap()
            .unwrap();
        let state: crate::state::RouterState =
            anchor_lang::AccountDeserialize::try_deserialize(&mut &account.data[..]).unwrap();
        assert_eq!(state.am_state.access_manager, new_am_program_id);
        assert_eq!(state.am_state.pending_access_manager, None);
    }

    #[tokio::test]
    async fn test_accept_no_pending_fails() {
        let admin = Keypair::new();
        let pt = setup_program_test_with_whitelist(&admin.pubkey(), &[]);
        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let new_am_pda = Pubkey::new_unique();
        let ix = build_accept_ix(admin.pubkey(), new_am_pda);

        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[ix],
            Some(&payer.pubkey()),
            &[&payer, &admin],
            recent_blockhash,
        );
        let err = banks_client.process_transaction(tx).await.unwrap_err();
        assert_eq!(
            pt_extract_custom_error(&err),
            Some(
                ANCHOR_ERROR_OFFSET
                    + access_manager::AccessManagerError::NoPendingAccessManagerTransfer as u32
            ),
        );
    }

    #[tokio::test]
    async fn test_accept_wrong_am_account_fails() {
        use access_manager::state::AccessManager;
        use anchor_lang::AnchorSerialize;

        let admin = Keypair::new();
        let new_am_program_id = Pubkey::new_unique();
        let wrong_am_pda = Pubkey::new_unique();

        if std::env::var("SBF_OUT_DIR").is_err() {
            std::env::set_var("SBF_OUT_DIR", std::path::Path::new("../../target/deploy"));
        }
        let mut pt = solana_program_test::ProgramTest::new("ics26_router", crate::ID, None);
        pt.add_program("access_manager", access_manager::ID, None);

        let (router_state_pda, _) =
            Pubkey::find_program_address(&[crate::state::RouterState::SEED], &crate::ID);
        let router_state = crate::state::RouterState {
            version: crate::state::AccountVersion::V1,
            am_state: AccessManagerState {
                access_manager: access_manager::ID,
                pending_access_manager: Some(new_am_program_id),
            },
            paused: false,
            _reserved: [0; 256],
        };
        let router_data = create_account_data(&router_state);
        pt.add_account(
            router_state_pda,
            solana_sdk::account::Account {
                lamports: 1_000_000,
                data: router_data,
                owner: crate::ID,
                executable: false,
                rent_epoch: 0,
            },
        );

        // Wrong AM account (not matching the pending PDA derivation)
        let wrong_am = AccessManager {
            roles: vec![access_manager::RoleData {
                role_id: solana_ibc_types::roles::ADMIN_ROLE,
                members: vec![admin.pubkey()],
            }],
            whitelisted_programs: vec![],
            pending_authority_transfers: vec![],
        };
        let mut am_data = AccessManager::DISCRIMINATOR.to_vec();
        wrong_am.serialize(&mut am_data).unwrap();
        pt.add_account(
            wrong_am_pda,
            solana_sdk::account::Account {
                lamports: 1_000_000,
                data: am_data,
                owner: new_am_program_id,
                executable: false,
                rent_epoch: 0,
            },
        );

        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let ix = build_accept_ix(admin.pubkey(), wrong_am_pda);
        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[ix],
            Some(&payer.pubkey()),
            &[&payer, &admin],
            recent_blockhash,
        );
        let err = banks_client.process_transaction(tx).await.unwrap_err();
        assert_eq!(
            pt_extract_custom_error(&err),
            Some(anchor_lang::error::ErrorCode::ConstraintSeeds as u32),
        );
    }

    #[tokio::test]
    async fn test_cancel_direct_call_succeeds() {
        let admin = Keypair::new();
        let pt = setup_program_test_with_whitelist(&admin.pubkey(), &[]);

        // We need to override the router state with a pending transfer
        // Start the test, propose first, then cancel
        let (banks_client, payer, recent_blockhash) = pt.start().await;

        // Propose first
        let propose_ix = build_propose_ix(admin.pubkey(), Pubkey::new_unique());
        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[propose_ix],
            Some(&payer.pubkey()),
            &[&payer, &admin],
            recent_blockhash,
        );
        banks_client
            .process_transaction(tx)
            .await
            .expect("propose should succeed");

        // Cancel
        let recent_blockhash = banks_client.get_latest_blockhash().await.unwrap();
        let cancel_ix = build_cancel_ix(admin.pubkey());
        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[cancel_ix],
            Some(&payer.pubkey()),
            &[&payer, &admin],
            recent_blockhash,
        );
        banks_client
            .process_transaction(tx)
            .await
            .expect("cancel should succeed");

        // Verify pending cleared
        let (router_state_pda, _) =
            Pubkey::find_program_address(&[crate::state::RouterState::SEED], &crate::ID);
        let account = banks_client
            .get_account(router_state_pda)
            .await
            .unwrap()
            .unwrap();
        let state: crate::state::RouterState =
            anchor_lang::AccountDeserialize::try_deserialize(&mut &account.data[..]).unwrap();
        assert_eq!(state.am_state.pending_access_manager, None);
    }
}
