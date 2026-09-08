// SPDX-License-Identifier: Apache-2.0

use crate::errors::AccessManagerError;
use crate::events::RoleRevokedEvent;
use crate::state::AccessManager;
use anchor_lang::prelude::*;
use solana_ibc_types::{require_direct_call_or_whitelisted_caller, roles};

/// Allows a caller to voluntarily give up one of their own roles.
#[derive(Accounts)]
#[instruction(role_id: u64)]
pub struct RenounceRole<'info> {
    /// The access manager PDA storing all role assignments.
    #[account(
        mut,
        seeds = [AccessManager::SEED],
        bump
    )]
    pub access_manager: Account<'info, AccessManager>,

    /// The signer renouncing their own role.
    pub caller: Signer<'info>,

    /// Instructions sysvar for CPI validation
    /// CHECK: Address constraint verifies this is the instructions sysvar
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instructions_sysvar: AccountInfo<'info>,
}

pub fn renounce_role(ctx: Context<RenounceRole>, role_id: u64) -> Result<()> {
    require_direct_call_or_whitelisted_caller(
        &ctx.accounts.instructions_sysvar,
        &ctx.accounts.access_manager.whitelisted_programs,
        &crate::ID,
    )
    .map_err(AccessManagerError::from)?;

    // Cannot renounce PUBLIC_ROLE
    require!(
        role_id != roles::PUBLIC_ROLE,
        AccessManagerError::InvalidRoleId
    );

    let caller_key = ctx.accounts.caller.key();

    // Verify caller has the role they're trying to renounce
    require!(
        ctx.accounts.access_manager.has_role(role_id, &caller_key),
        AccessManagerError::Unauthorized
    );

    // Revoke the role from caller (will fail if trying to remove last admin)
    ctx.accounts
        .access_manager
        .revoke_role(role_id, &caller_key)?;

    emit!(RoleRevokedEvent {
        role_id,
        account: caller_key,
        revoked_by: caller_key, // Self-revocation
    });

    msg!("Role {} renounced by {}", role_id, caller_key);

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
    fn test_renounce_role_success() {
        let relayer = Pubkey::new_unique();
        let admin = Pubkey::new_unique();

        let (access_manager_pda, access_manager_account) =
            create_access_manager_with_role(admin, roles::RELAYER_ROLE, relayer);

        let instruction = build_instruction(
            crate::instruction::RenounceRole {
                role_id: roles::RELAYER_ROLE,
            },
            vec![
                AccountMeta::new(access_manager_pda, false),
                AccountMeta::new_readonly(relayer, true),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
            ],
        );

        let accounts = vec![
            (access_manager_pda, access_manager_account),
            (relayer, create_signer_account()),
            (
                solana_sdk::sysvar::instructions::ID,
                create_instructions_sysvar_account(),
            ),
        ];

        let mollusk = setup_mollusk();
        let checks = vec![Check::success()];
        let result = mollusk.process_and_validate_instruction(&instruction, &accounts, &checks);

        let access_manager = get_access_manager_from_result(&result, &access_manager_pda);
        assert!(!access_manager.has_role(roles::RELAYER_ROLE, &relayer));
    }

    #[test]
    fn test_renounce_role_without_having_role() {
        let relayer = Pubkey::new_unique();
        let admin = Pubkey::new_unique();

        let (access_manager_pda, access_manager_account) = create_initialized_access_manager(admin);

        let instruction = build_instruction(
            crate::instruction::RenounceRole {
                role_id: roles::RELAYER_ROLE,
            },
            vec![
                AccountMeta::new(access_manager_pda, false),
                AccountMeta::new_readonly(relayer, true),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
            ],
        );

        let accounts = vec![
            (access_manager_pda, access_manager_account),
            (relayer, create_signer_account()),
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
    fn test_renounce_role_cannot_remove_last_admin() {
        let admin = Pubkey::new_unique();

        let (access_manager_pda, access_manager_account) = create_initialized_access_manager(admin);

        let instruction = build_instruction(
            crate::instruction::RenounceRole {
                role_id: roles::ADMIN_ROLE,
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
            ANCHOR_ERROR_OFFSET + AccessManagerError::CannotRemoveLastAdmin as u32,
        ))];

        mollusk.process_and_validate_instruction(&instruction, &accounts, &checks);
    }

    #[test]
    fn test_renounce_role_fake_sysvar_wormhole_attack() {
        let admin = Pubkey::new_unique();
        let relayer = Pubkey::new_unique();
        let (access_manager_pda, access_manager_account) =
            create_access_manager_with_role(admin, roles::RELAYER_ROLE, relayer);

        let instruction = build_instruction(
            crate::instruction::RenounceRole {
                role_id: roles::RELAYER_ROLE,
            },
            vec![
                AccountMeta::new(access_manager_pda, false),
                AccountMeta::new_readonly(relayer, true),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
            ],
        );

        // Replace real sysvar with fake one (Wormhole-style attack)
        let (instruction, fake_sysvar_account) = setup_fake_sysvar_attack(instruction, crate::ID);

        let accounts = vec![
            (access_manager_pda, access_manager_account),
            (relayer, create_signer_account()),
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
    fn test_renounce_role_cpi_rejection() {
        let relayer = Pubkey::new_unique();
        let role_id = 100;

        let (access_manager_pda, access_manager_account) =
            create_access_manager_with_role(relayer, role_id, relayer);

        let instruction = build_instruction(
            crate::instruction::RenounceRole { role_id },
            vec![
                AccountMeta::new(access_manager_pda, false),
                AccountMeta::new_readonly(relayer, true),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
            ],
        );

        // Simulate CPI call from unauthorized program
        let malicious_program = Pubkey::new_unique();
        let (instruction, cpi_sysvar_account) = setup_cpi_call_test(instruction, malicious_program);

        let accounts = vec![
            (access_manager_pda, access_manager_account),
            (relayer, create_signer_account()),
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

    fn build_renounce_role_ix(caller: Pubkey, role_id: u64) -> Instruction {
        let (access_manager_pda, _) =
            Pubkey::find_program_address(&[crate::state::AccessManager::SEED], &crate::ID);

        Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(access_manager_pda, false),
                AccountMeta::new_readonly(caller, true),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
            ],
            data: crate::instruction::RenounceRole { role_id }.data(),
        }
    }

    #[tokio::test]
    async fn test_direct_call_succeeds() {
        let admin = Keypair::new();
        let relayer = Keypair::new();
        let pt = setup_program_test_with_whitelist(&admin.pubkey(), &[TEST_CPI_TARGET_ID]);
        let (banks_client, payer, recent_blockhash) = pt.start().await;

        // Grant relayer role first
        let grant_ix = build_grant_role_ix(
            admin.pubkey(),
            solana_ibc_types::roles::RELAYER_ROLE,
            relayer.pubkey(),
        );
        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[grant_ix],
            Some(&payer.pubkey()),
            &[&payer, &admin],
            recent_blockhash,
        );
        banks_client.process_transaction(tx).await.unwrap();

        // Relayer renounces their own role
        let renounce_ix =
            build_renounce_role_ix(relayer.pubkey(), solana_ibc_types::roles::RELAYER_ROLE);
        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[renounce_ix],
            Some(&payer.pubkey()),
            &[&payer, &relayer],
            recent_blockhash,
        );
        let result = banks_client.process_transaction(tx).await;
        assert!(result.is_ok(), "Direct renounce should succeed");
    }

    #[tokio::test]
    async fn test_direct_call_without_role_rejected() {
        let admin = Keypair::new();
        let non_member = Keypair::new();
        let pt = setup_program_test_with_whitelist(&admin.pubkey(), &[]);
        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let ix = build_renounce_role_ix(non_member.pubkey(), solana_ibc_types::roles::RELAYER_ROLE);

        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[ix],
            Some(&payer.pubkey()),
            &[&payer, &non_member],
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
        let relayer = Keypair::new();
        let pt = setup_program_test_with_whitelist(&admin.pubkey(), &[TEST_CPI_TARGET_ID]);
        let (banks_client, payer, recent_blockhash) = pt.start().await;

        // Grant relayer role first
        let grant_ix = build_grant_role_ix(
            admin.pubkey(),
            solana_ibc_types::roles::RELAYER_ROLE,
            relayer.pubkey(),
        );
        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[grant_ix],
            Some(&payer.pubkey()),
            &[&payer, &admin],
            recent_blockhash,
        );
        banks_client.process_transaction(tx).await.unwrap();

        // Renounce via whitelisted CPI
        let inner_ix =
            build_renounce_role_ix(relayer.pubkey(), solana_ibc_types::roles::RELAYER_ROLE);
        let wrapped_ix = wrap_in_test_cpi_target_proxy(relayer.pubkey(), &inner_ix);

        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[wrapped_ix],
            Some(&payer.pubkey()),
            &[&payer, &relayer],
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

        let inner_ix =
            build_renounce_role_ix(admin.pubkey(), solana_ibc_types::roles::RELAYER_ROLE);
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

        let inner_ix =
            build_renounce_role_ix(admin.pubkey(), solana_ibc_types::roles::RELAYER_ROLE);
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
