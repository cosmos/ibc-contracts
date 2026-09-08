// SPDX-License-Identifier: Apache-2.0

use crate::types::AppState;
use anchor_lang::prelude::*;

/// Proposes transferring the access manager to a new program.
/// Requires `ADMIN_ROLE` on the current access manager.
#[derive(Accounts)]
#[instruction(new_access_manager: Pubkey)]
pub struct ProposeAccessManagerTransfer<'info> {
    /// PDA holding program-level settings including the current `access_manager`.
    #[account(mut, seeds = [AppState::SEED], bump)]
    pub app_state: Account<'info, AppState>,

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
    /// PDA holding program-level settings; the `access_manager` field is updated on success.
    #[account(
        mut,
        seeds = [AppState::SEED],
        bump,
        constraint = app_state.am_state.pending_access_manager.is_some()
            @ access_manager::AccessManagerError::NoPendingAccessManagerTransfer
    )]
    pub app_state: Account<'info, AppState>,

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
    /// PDA holding program-level settings; the pending transfer is cleared on success.
    #[account(mut, seeds = [AppState::SEED], bump)]
    pub app_state: Account<'info, AppState>,

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
    use crate::test_helpers::PROGRAM_BINARY_PATH;
    use crate::types::AppState;
    use access_manager::{AccessManagerError, AccessManagerState};
    use anchor_lang::{AccountDeserialize, AccountSerialize, InstructionData, Space};
    use mollusk_svm::result::Check;
    use mollusk_svm::Mollusk;
    use solana_ibc_types::roles;
    use solana_sdk::account::Account as SolanaAccount;
    use solana_sdk::instruction::{AccountMeta, Instruction};
    use solana_sdk::pubkey::Pubkey;

    const ANCHOR_ERROR_OFFSET: u32 = 6000;

    fn create_signer_account() -> SolanaAccount {
        SolanaAccount {
            lamports: 1_000_000_000,
            data: vec![],
            owner: solana_sdk::system_program::ID,
            executable: false,
            rent_epoch: 0,
        }
    }

    fn create_app_state_account(access_manager: Pubkey, pending: Option<Pubkey>) -> SolanaAccount {
        let app_state = AppState {
            am_state: pending.map_or_else(
                || AccessManagerState::new(access_manager),
                |pending| AccessManagerState {
                    access_manager,
                    pending_access_manager: Some(pending),
                },
            ),
            _reserved: [0; 256],
        };

        let mut data = vec![0u8; 8 + AppState::INIT_SPACE];
        app_state.try_serialize(&mut &mut data[..]).unwrap();

        SolanaAccount {
            lamports: 10_000_000,
            data,
            owner: crate::ID,
            executable: false,
            rent_epoch: 0,
        }
    }

    fn create_access_manager_account(admin: Pubkey, role: u64) -> SolanaAccount {
        use access_manager::state::AccessManager;
        use access_manager::types::RoleData;

        let access_manager = AccessManager {
            roles: vec![RoleData {
                role_id: role,
                members: vec![admin],
            }],
            whitelisted_programs: vec![],
            pending_authority_transfers: vec![],
        };

        let mut data = vec![0u8; 8 + 10000];
        access_manager.try_serialize(&mut &mut data[..]).unwrap();

        SolanaAccount {
            lamports: 10_000_000,
            data,
            owner: access_manager::ID,
            executable: false,
            rent_epoch: 0,
        }
    }

    fn create_instructions_sysvar_account() -> (Pubkey, SolanaAccount) {
        use solana_sdk::sysvar::instructions::{
            construct_instructions_data, BorrowedAccountMeta, BorrowedInstruction,
        };

        let account_pubkey = Pubkey::new_unique();
        let account = BorrowedAccountMeta {
            pubkey: &account_pubkey,
            is_signer: false,
            is_writable: true,
        };
        let mock_instruction = BorrowedInstruction {
            program_id: &crate::ID,
            accounts: vec![account],
            data: &[],
        };

        let ixs_data = construct_instructions_data(&[mock_instruction]);

        (
            solana_sdk::sysvar::instructions::ID,
            SolanaAccount {
                lamports: 1_000_000,
                data: ixs_data,
                owner: solana_sdk::sysvar::ID,
                executable: false,
                rent_epoch: 0,
            },
        )
    }

    #[test]
    fn test_propose_succeeds() {
        let admin = Pubkey::new_unique();
        let new_access_manager = Pubkey::new_unique();

        let (app_state_pda, _) = Pubkey::find_program_address(&[AppState::SEED], &crate::ID);

        let (access_manager_pda, _) = Pubkey::find_program_address(
            &[access_manager::state::AccessManager::SEED],
            &access_manager::ID,
        );

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(app_state_pda, false),
                AccountMeta::new_readonly(access_manager_pda, false),
                AccountMeta::new_readonly(admin, true),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
            ],
            data: crate::instruction::ProposeAccessManagerTransfer { new_access_manager }.data(),
        };

        let app_state_account = create_app_state_account(access_manager::ID, None);
        let access_manager_account = create_access_manager_account(admin, roles::ADMIN_ROLE);
        let (instructions_sysvar_pubkey, instructions_sysvar_account) =
            create_instructions_sysvar_account();

        let accounts = vec![
            (app_state_pda, app_state_account),
            (access_manager_pda, access_manager_account),
            (admin, create_signer_account()),
            (instructions_sysvar_pubkey, instructions_sysvar_account),
        ];

        let mollusk = Mollusk::new(&crate::ID, PROGRAM_BINARY_PATH);
        let result =
            mollusk.process_and_validate_instruction(&instruction, &accounts, &[Check::success()]);

        let app_state_account = result
            .get_account(&app_state_pda)
            .expect("App state account not found");
        let app_state: AppState = AppState::try_deserialize(&mut &app_state_account.data[..])
            .expect("Failed to deserialize app state");

        assert_eq!(
            app_state.am_state.pending_access_manager,
            Some(new_access_manager)
        );
        assert_eq!(app_state.am_state.access_manager, access_manager::ID);
    }

    #[test]
    fn test_propose_not_admin_fails() {
        let admin = Pubkey::new_unique();
        let non_admin = Pubkey::new_unique();
        let new_access_manager = Pubkey::new_unique();

        let (app_state_pda, _) = Pubkey::find_program_address(&[AppState::SEED], &crate::ID);

        let (access_manager_pda, _) = Pubkey::find_program_address(
            &[access_manager::state::AccessManager::SEED],
            &access_manager::ID,
        );

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(app_state_pda, false),
                AccountMeta::new_readonly(access_manager_pda, false),
                AccountMeta::new_readonly(non_admin, true),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
            ],
            data: crate::instruction::ProposeAccessManagerTransfer { new_access_manager }.data(),
        };

        let app_state_account = create_app_state_account(access_manager::ID, None);
        let access_manager_account = create_access_manager_account(admin, roles::ADMIN_ROLE);
        let (instructions_sysvar_pubkey, instructions_sysvar_account) =
            create_instructions_sysvar_account();

        let accounts = vec![
            (app_state_pda, app_state_account),
            (access_manager_pda, access_manager_account),
            (non_admin, create_signer_account()),
            (instructions_sysvar_pubkey, instructions_sysvar_account),
        ];

        let mollusk = Mollusk::new(&crate::ID, PROGRAM_BINARY_PATH);
        mollusk.process_and_validate_instruction(
            &instruction,
            &accounts,
            &[Check::err(solana_sdk::program_error::ProgramError::Custom(
                ANCHOR_ERROR_OFFSET + AccessManagerError::Unauthorized as u32,
            ))],
        );
    }
}

#[cfg(test)]
mod integration_tests {
    use crate::test_helpers::*;
    use anchor_lang::InstructionData;
    use solana_sdk::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        signature::Keypair,
        signer::Signer,
    };

    fn build_propose_ix(admin: Pubkey, new_access_manager: Pubkey) -> Instruction {
        let (app_state_pda, _) =
            Pubkey::find_program_address(&[crate::types::AppState::SEED], &crate::ID);
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
}
