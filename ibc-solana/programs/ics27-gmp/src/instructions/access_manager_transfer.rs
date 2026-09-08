// SPDX-License-Identifier: Apache-2.0

use crate::state::GMPAppState;
use anchor_lang::prelude::*;

/// Proposes transferring the access manager to a new program.
/// Requires `ADMIN_ROLE` on the current access manager.
#[derive(Accounts)]
#[instruction(new_access_manager: Pubkey)]
pub struct ProposeAccessManagerTransfer<'info> {
    /// PDA holding GMP app settings including the current `access_manager`.
    #[account(mut, seeds = [GMPAppState::SEED], bump = app_state.bump)]
    pub app_state: Account<'info, GMPAppState>,

    /// Current access-manager state PDA used to verify the caller holds the admin role.
    /// CHECK: Validated via seeds constraint using the stored `access_manager` program ID
    #[account(
        seeds = [access_manager::state::AccessManager::SEED],
        bump,
        seeds::program = app_state.am_state.access_manager
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
    ctx.accounts.app_state.am_state.propose_transfer(
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
    /// PDA holding GMP app settings; the `access_manager` field is updated on success.
    #[account(
        mut,
        seeds = [GMPAppState::SEED],
        bump = app_state.bump,
        constraint = app_state.am_state.pending_access_manager.is_some()
            @ access_manager::AccessManagerError::NoPendingAccessManagerTransfer
    )]
    pub app_state: Account<'info, GMPAppState>,

    /// Proposed access-manager state PDA derived from the pending program ID.
    /// CHECK: Validated via seeds constraint against `pending_access_manager`
    #[account(
        seeds = [access_manager::state::AccessManager::SEED],
        bump,
        seeds::program = app_state.am_state.pending_access_manager.unwrap()
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
    ctx.accounts.app_state.am_state.accept_transfer(
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
    /// PDA holding GMP app settings; the pending transfer is cleared on success.
    #[account(mut, seeds = [GMPAppState::SEED], bump = app_state.bump)]
    pub app_state: Account<'info, GMPAppState>,

    /// Current access-manager state PDA used to verify the caller holds the admin role.
    /// CHECK: Validated via seeds constraint using the stored `access_manager` program ID
    #[account(
        seeds = [access_manager::state::AccessManager::SEED],
        bump,
        seeds::program = app_state.am_state.access_manager
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
    ctx.accounts.app_state.am_state.cancel_transfer(
        &ctx.accounts.am_state,
        &ctx.accounts.admin,
        &ctx.accounts.instructions_sysvar,
        &crate::ID,
    )
}

#[cfg(test)]
mod tests {
    use crate::state::GMPAppState;
    use crate::test_utils::*;
    use access_manager::{AccessManagerError, AccessManagerState};
    use anchor_lang::{AnchorSerialize, Discriminator};
    use mollusk_svm::result::Check;
    use solana_ibc_types::roles;
    use solana_sdk::instruction::AccountMeta;
    use solana_sdk::pubkey::Pubkey;

    fn build_propose_instruction(
        admin: Pubkey,
        new_access_manager: Pubkey,
    ) -> solana_sdk::instruction::Instruction {
        let (app_state_pda, _) = Pubkey::find_program_address(&[GMPAppState::SEED], &crate::ID);
        let (access_manager_pda, _) = Pubkey::find_program_address(
            &[access_manager::state::AccessManager::SEED],
            &access_manager::ID,
        );

        build_instruction(
            crate::instruction::ProposeAccessManagerTransfer { new_access_manager },
            vec![
                AccountMeta::new(app_state_pda, false),
                AccountMeta::new_readonly(access_manager_pda, false),
                AccountMeta::new_readonly(admin, true),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
            ],
        )
    }

    fn build_cancel_instruction(admin: Pubkey) -> solana_sdk::instruction::Instruction {
        let (app_state_pda, _) = Pubkey::find_program_address(&[GMPAppState::SEED], &crate::ID);
        let (access_manager_pda, _) = Pubkey::find_program_address(
            &[access_manager::state::AccessManager::SEED],
            &access_manager::ID,
        );

        build_instruction(
            crate::instruction::CancelAccessManagerTransfer {},
            vec![
                AccountMeta::new(app_state_pda, false),
                AccountMeta::new_readonly(access_manager_pda, false),
                AccountMeta::new_readonly(admin, true),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
            ],
        )
    }

    fn create_app_state_with_pending(
        pending: Option<Pubkey>,
    ) -> (Pubkey, solana_sdk::account::Account) {
        let (pda, bump) = Pubkey::find_program_address(&[GMPAppState::SEED], &crate::ID);
        let state = GMPAppState {
            version: crate::state::AccountVersion::V1,
            paused: false,
            bump,
            am_state: pending.map_or_else(
                || AccessManagerState::new(access_manager::ID),
                |pending| AccessManagerState {
                    access_manager: access_manager::ID,
                    pending_access_manager: Some(pending),
                },
            ),
            _reserved: [0; 256],
        };
        let mut data = Vec::new();
        data.extend_from_slice(GMPAppState::DISCRIMINATOR);
        state.serialize(&mut data).unwrap();
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

    // -- Propose tests --

    #[test]
    fn test_propose_succeeds() {
        let admin = Pubkey::new_unique();
        let new_am = Pubkey::new_unique();

        let (app_state_pda, app_state_account) = create_initialized_app_state(access_manager::ID);
        let (am_pda, am_account) = create_access_manager_with_role(admin, roles::ADMIN_ROLE, admin);

        let instruction = build_propose_instruction(admin, new_am);
        let accounts = vec![
            (app_state_pda, app_state_account),
            (am_pda, am_account),
            (admin, create_signer_account()),
            create_instructions_sysvar_account_with_caller(crate::ID),
        ];

        let mollusk = setup_mollusk();
        let result =
            mollusk.process_and_validate_instruction(&instruction, &accounts, &[Check::success()]);

        let state = get_app_state_from_result(&result, &app_state_pda);
        assert_eq!(state.am_state.pending_access_manager, Some(new_am));
        assert_eq!(state.am_state.access_manager, access_manager::ID);
    }

    #[test]
    fn test_propose_not_admin_fails() {
        let admin = Pubkey::new_unique();
        let non_admin = Pubkey::new_unique();
        let new_am = Pubkey::new_unique();

        let (app_state_pda, app_state_account) = create_initialized_app_state(access_manager::ID);
        let (am_pda, am_account) = create_access_manager_with_role(admin, roles::ADMIN_ROLE, admin);

        let instruction = build_propose_instruction(non_admin, new_am);
        let accounts = vec![
            (app_state_pda, app_state_account),
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

        let (app_state_pda, app_state_account) = create_initialized_app_state(access_manager::ID);
        let (am_pda, am_account) = create_access_manager_with_role(admin, roles::ADMIN_ROLE, admin);

        let instruction = build_propose_instruction(admin, Pubkey::default());
        let accounts = vec![
            (app_state_pda, app_state_account),
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

        let (app_state_pda, app_state_account) = create_initialized_app_state(access_manager::ID);
        let (am_pda, am_account) = create_access_manager_with_role(admin, roles::ADMIN_ROLE, admin);

        let instruction = build_propose_instruction(admin, access_manager::ID);
        let accounts = vec![
            (app_state_pda, app_state_account),
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

        let (app_state_pda, app_state_account) =
            create_app_state_with_pending(Some(Pubkey::new_unique()));
        let (am_pda, am_account) = create_access_manager_with_role(admin, roles::ADMIN_ROLE, admin);

        let instruction = build_propose_instruction(admin, new_am);
        let accounts = vec![
            (app_state_pda, app_state_account),
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

    // -- Cancel tests --

    #[test]
    fn test_cancel_succeeds() {
        let admin = Pubkey::new_unique();
        let pending = Pubkey::new_unique();

        let (app_state_pda, app_state_account) = create_app_state_with_pending(Some(pending));
        let (am_pda, am_account) = create_access_manager_with_role(admin, roles::ADMIN_ROLE, admin);

        let instruction = build_cancel_instruction(admin);
        let accounts = vec![
            (app_state_pda, app_state_account),
            (am_pda, am_account),
            (admin, create_signer_account()),
            create_instructions_sysvar_account_with_caller(crate::ID),
        ];

        let mollusk = setup_mollusk();
        let result =
            mollusk.process_and_validate_instruction(&instruction, &accounts, &[Check::success()]);

        let state = get_app_state_from_result(&result, &app_state_pda);
        assert_eq!(state.am_state.pending_access_manager, None);
    }

    #[test]
    fn test_cancel_not_admin_fails() {
        let admin = Pubkey::new_unique();
        let non_admin = Pubkey::new_unique();

        let (app_state_pda, app_state_account) =
            create_app_state_with_pending(Some(Pubkey::new_unique()));
        let (am_pda, am_account) = create_access_manager_with_role(admin, roles::ADMIN_ROLE, admin);

        let instruction = build_cancel_instruction(non_admin);
        let accounts = vec![
            (app_state_pda, app_state_account),
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

        let (app_state_pda, app_state_account) = create_initialized_app_state(access_manager::ID);
        let (am_pda, am_account) = create_access_manager_with_role(admin, roles::ADMIN_ROLE, admin);

        let instruction = build_cancel_instruction(admin);
        let accounts = vec![
            (app_state_pda, app_state_account),
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
    use anchor_lang::InstructionData;
    use solana_sdk::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        signature::Keypair,
        signer::Signer,
    };

    fn build_propose_ix(admin: Pubkey, new_access_manager: Pubkey) -> Instruction {
        let (app_state_pda, _) =
            Pubkey::find_program_address(&[crate::state::GMPAppState::SEED], &crate::ID);
        let (access_manager_pda, _) = Pubkey::find_program_address(
            &[access_manager::state::AccessManager::SEED],
            &access_manager::ID,
        );

        Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(app_state_pda, false),
                AccountMeta::new_readonly(access_manager_pda, false),
                AccountMeta::new_readonly(admin, true),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
            ],
            data: crate::instruction::ProposeAccessManagerTransfer { new_access_manager }.data(),
        }
    }

    #[tokio::test]
    async fn test_propose_direct_call_succeeds() {
        let admin = Keypair::new();
        let pt = setup_program_test_with_access_manager(&admin.pubkey(), &[TEST_CPI_TARGET_ID]);
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
}
