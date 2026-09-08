// SPDX-License-Identifier: Apache-2.0

use crate::errors::AccessManagerError;
use crate::events::RoleGrantedEvent;
use crate::helpers::require_admin;
use crate::state::AccessManager;
use anchor_lang::prelude::*;
use solana_ibc_types::roles;

/// Grants a role to the specified account. Requires admin authorization.
#[derive(Accounts)]
#[instruction(role_id: u64, account: Pubkey)]
pub struct GrantRole<'info> {
    /// The access manager PDA storing all role assignments.
    #[account(
        mut,
        seeds = [AccessManager::SEED],
        bump
    )]
    pub access_manager: Account<'info, AccessManager>,

    /// The admin signer authorizing this role grant.
    pub admin: Signer<'info>,

    /// Instructions sysvar for CPI validation
    /// CHECK: Address constraint verifies this is the instructions sysvar
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instructions_sysvar: AccountInfo<'info>,
}

pub fn grant_role(ctx: Context<GrantRole>, role_id: u64, account: Pubkey) -> Result<()> {
    require_admin(
        &ctx.accounts.access_manager.to_account_info(),
        &ctx.accounts.admin.to_account_info(),
        &ctx.accounts.instructions_sysvar,
        &crate::ID,
    )?;

    require!(
        account != Pubkey::default(),
        AccessManagerError::ZeroAccount
    );

    // Cannot grant PUBLIC_ROLE
    require!(
        role_id != roles::PUBLIC_ROLE,
        AccessManagerError::InvalidRoleId
    );

    ctx.accounts.access_manager.grant_role(role_id, account)?;

    emit!(RoleGrantedEvent {
        role_id,
        account,
        granted_by: ctx.accounts.admin.key(),
    });

    msg!(
        "Role {} granted to {} by {}",
        role_id,
        account,
        ctx.accounts.admin.key()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::AccessManagerError;
    use crate::test_utils::*;
    use mollusk_svm::result::Check;
    use solana_sdk::instruction::AccountMeta;

    #[test]
    fn test_grant_role_success() {
        let admin = Pubkey::new_unique();
        let relayer = Pubkey::new_unique();

        let (access_manager_pda, access_manager_account) = create_initialized_access_manager(admin);

        let instruction = build_instruction(
            crate::instruction::GrantRole {
                role_id: roles::RELAYER_ROLE,
                account: relayer,
            },
            vec![
                AccountMeta::new(access_manager_pda, false),
                AccountMeta::new_readonly(admin, true),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
            ],
        );

        let accounts = vec![
            (access_manager_pda, access_manager_account),
            (admin, create_signer_account()),
            (
                solana_sdk::sysvar::instructions::ID,
                create_instructions_sysvar_account(),
            ),
        ];

        let mollusk = setup_mollusk();
        let checks = vec![Check::success()];
        let result = mollusk.process_and_validate_instruction(&instruction, &accounts, &checks);

        let access_manager = get_access_manager_from_result(&result, &access_manager_pda);
        assert!(access_manager.has_role(roles::RELAYER_ROLE, &relayer));
    }

    #[test]
    fn test_grant_role_not_admin() {
        let admin = Pubkey::new_unique();
        let non_admin = Pubkey::new_unique();
        let relayer = Pubkey::new_unique();

        let (access_manager_pda, access_manager_account) = create_initialized_access_manager(admin);

        let instruction = build_instruction(
            crate::instruction::GrantRole {
                role_id: roles::RELAYER_ROLE,
                account: relayer,
            },
            vec![
                AccountMeta::new(access_manager_pda, false),
                AccountMeta::new_readonly(non_admin, true),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
            ],
        );

        let accounts = vec![
            (access_manager_pda, access_manager_account),
            (non_admin, create_signer_account()),
            (
                solana_sdk::sysvar::instructions::ID,
                create_instructions_sysvar_account(),
            ),
        ];

        let mollusk = setup_mollusk();
        let checks = vec![Check::err(solana_sdk::program_error::ProgramError::Custom(
            ANCHOR_ERROR_OFFSET + AccessManagerError::Unauthorized as u32,
        ))];

        mollusk.process_and_validate_instruction(&instruction, &accounts, &checks);
    }

    #[test]
    fn test_grant_role_invalid_role() {
        let admin = Pubkey::new_unique();
        let account = Pubkey::new_unique();

        let (access_manager_pda, access_manager_account) = create_initialized_access_manager(admin);

        let instruction = build_instruction(
            crate::instruction::GrantRole {
                role_id: roles::PUBLIC_ROLE, // Cannot grant PUBLIC_ROLE
                account,
            },
            vec![
                AccountMeta::new(access_manager_pda, false),
                AccountMeta::new_readonly(admin, true),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
            ],
        );

        let accounts = vec![
            (access_manager_pda, access_manager_account),
            (admin, create_signer_account()),
            (
                solana_sdk::sysvar::instructions::ID,
                create_instructions_sysvar_account(),
            ),
        ];

        let mollusk = setup_mollusk();
        let checks = vec![Check::err(solana_sdk::program_error::ProgramError::Custom(
            ANCHOR_ERROR_OFFSET + AccessManagerError::InvalidRoleId as u32,
        ))];

        mollusk.process_and_validate_instruction(&instruction, &accounts, &checks);
    }

    #[test]
    fn test_grant_role_zero_address_rejected() {
        let admin = Pubkey::new_unique();

        let (access_manager_pda, access_manager_account) = create_initialized_access_manager(admin);

        let instruction = build_instruction(
            crate::instruction::GrantRole {
                role_id: roles::RELAYER_ROLE,
                account: Pubkey::default(),
            },
            vec![
                AccountMeta::new(access_manager_pda, false),
                AccountMeta::new_readonly(admin, true),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
            ],
        );

        let accounts = vec![
            (access_manager_pda, access_manager_account),
            (admin, create_signer_account()),
            (
                solana_sdk::sysvar::instructions::ID,
                create_instructions_sysvar_account(),
            ),
        ];

        let mollusk = setup_mollusk();
        let checks = vec![Check::err(solana_sdk::program_error::ProgramError::Custom(
            ANCHOR_ERROR_OFFSET + AccessManagerError::ZeroAccount as u32,
        ))];

        mollusk.process_and_validate_instruction(&instruction, &accounts, &checks);
    }

    #[test]
    fn test_grant_role_fake_sysvar_wormhole_attack() {
        let admin = Pubkey::new_unique();
        let relayer = Pubkey::new_unique();

        let (access_manager_pda, access_manager_account) = create_initialized_access_manager(admin);

        let instruction = build_instruction(
            crate::instruction::GrantRole {
                role_id: roles::RELAYER_ROLE,
                account: relayer,
            },
            vec![
                AccountMeta::new(access_manager_pda, false),
                AccountMeta::new_readonly(admin, true),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
            ],
        );

        // Replace real sysvar with fake one (Wormhole-style attack)
        let (instruction, fake_sysvar_account) = setup_fake_sysvar_attack(instruction, crate::ID);

        let accounts = vec![
            (access_manager_pda, access_manager_account),
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
    fn test_grant_role_cpi_rejection() {
        let admin = Pubkey::new_unique();
        let member = Pubkey::new_unique();
        let role_id = 100;

        let (access_manager_pda, access_manager_account) =
            create_access_manager_with_role(admin, roles::ADMIN_ROLE, admin);

        let instruction = build_instruction(
            crate::instruction::GrantRole {
                role_id,
                account: member,
            },
            vec![
                AccountMeta::new(access_manager_pda, false),
                AccountMeta::new_readonly(admin, true),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
            ],
        );

        // Simulate CPI call from unauthorized program
        let malicious_program = Pubkey::new_unique();
        let (instruction, cpi_sysvar_account) = setup_cpi_call_test(instruction, malicious_program);

        let accounts = vec![
            (access_manager_pda, access_manager_account),
            (admin, create_signer_account()),
            cpi_sysvar_account,
        ];

        let mollusk = setup_mollusk();
        mollusk.process_and_validate_instruction(
            &instruction,
            &accounts,
            &[expect_access_manager_cpi_rejection_error()],
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

    fn build_grant_role_ix(admin: Pubkey, role_id: u64, account: Pubkey) -> Instruction {
        let (access_manager_pda, _) =
            Pubkey::find_program_address(&[crate::state::AccessManager::SEED], &crate::ID);

        Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(access_manager_pda, false),
                AccountMeta::new_readonly(admin, true),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
            ],
            data: crate::instruction::GrantRole { role_id, account }.data(),
        }
    }

    #[tokio::test]
    async fn test_direct_call_by_admin_succeeds() {
        let admin = Keypair::new();
        let relayer = Pubkey::new_unique();
        let pt = setup_program_test_with_whitelist(&admin.pubkey(), &[TEST_CPI_TARGET_ID]);
        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let ix = build_grant_role_ix(
            admin.pubkey(),
            solana_ibc_types::roles::RELAYER_ROLE,
            relayer,
        );

        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[ix],
            Some(&payer.pubkey()),
            &[&payer, &admin],
            recent_blockhash,
        );
        let result = banks_client.process_transaction(tx).await;
        assert!(result.is_ok(), "Direct call by admin should succeed");
    }

    #[tokio::test]
    async fn test_direct_call_by_non_admin_rejected() {
        let admin = Keypair::new();
        let non_admin = Keypair::new();
        let pt = setup_program_test_with_whitelist(&admin.pubkey(), &[]);
        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let ix = build_grant_role_ix(
            non_admin.pubkey(),
            solana_ibc_types::roles::RELAYER_ROLE,
            Pubkey::new_unique(),
        );

        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[ix],
            Some(&payer.pubkey()),
            &[&payer, &non_admin],
            recent_blockhash,
        );
        let err = banks_client.process_transaction(tx).await.unwrap_err();
        assert_eq!(
            extract_custom_error(&err),
            Some(ANCHOR_ERROR_OFFSET + crate::errors::AccessManagerError::Unauthorized as u32),
        );
    }

    #[tokio::test]
    async fn test_whitelisted_cpi_succeeds() {
        let admin = Keypair::new();
        let relayer = Pubkey::new_unique();
        let pt = setup_program_test_with_whitelist(&admin.pubkey(), &[TEST_CPI_TARGET_ID]);
        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let inner_ix = build_grant_role_ix(
            admin.pubkey(),
            solana_ibc_types::roles::RELAYER_ROLE,
            relayer,
        );
        let wrapped_ix = wrap_in_test_cpi_target_proxy(admin.pubkey(), &inner_ix);

        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[wrapped_ix],
            Some(&payer.pubkey()),
            &[&payer, &admin],
            recent_blockhash,
        );
        let result = banks_client.process_transaction(tx).await;
        assert!(
            result.is_ok(),
            "Whitelisted CPI should succeed: {:?}",
            result.err()
        );
    }

    #[tokio::test]
    async fn test_unauthorized_cpi_rejected() {
        let admin = Keypair::new();
        let pt = setup_program_test_with_whitelist(&admin.pubkey(), &[TEST_CPI_TARGET_ID]);
        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let inner_ix = build_grant_role_ix(
            admin.pubkey(),
            solana_ibc_types::roles::RELAYER_ROLE,
            Pubkey::new_unique(),
        );
        let wrapped_ix = wrap_in_test_cpi_proxy(admin.pubkey(), &inner_ix);

        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[wrapped_ix],
            Some(&payer.pubkey()),
            &[&payer, &admin],
            recent_blockhash,
        );
        let err = banks_client.process_transaction(tx).await.unwrap_err();
        assert_eq!(
            extract_custom_error(&err),
            Some(ANCHOR_ERROR_OFFSET + crate::errors::AccessManagerError::CpiNotAllowed as u32),
        );
    }

    #[tokio::test]
    async fn test_nested_cpi_rejected() {
        let admin = Keypair::new();
        let pt = setup_program_test_with_whitelist(&admin.pubkey(), &[TEST_CPI_TARGET_ID]);
        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let inner_ix = build_grant_role_ix(
            admin.pubkey(),
            solana_ibc_types::roles::RELAYER_ROLE,
            Pubkey::new_unique(),
        );
        let cpi_target_ix = wrap_in_test_cpi_target_proxy(admin.pubkey(), &inner_ix);
        let nested_ix = wrap_in_test_cpi_proxy(admin.pubkey(), &cpi_target_ix);

        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[nested_ix],
            Some(&payer.pubkey()),
            &[&payer, &admin],
            recent_blockhash,
        );
        let err = banks_client.process_transaction(tx).await.unwrap_err();
        assert_eq!(
            extract_custom_error(&err),
            Some(ANCHOR_ERROR_OFFSET + crate::errors::AccessManagerError::CpiNotAllowed as u32),
        );
    }
}
