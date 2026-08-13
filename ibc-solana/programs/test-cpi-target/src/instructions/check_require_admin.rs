// SPDX-License-Identifier: Apache-2.0

use anchor_lang::prelude::*;

/// Accounts for [`require_admin`](access_manager::require_admin) wrapper.
#[derive(Accounts)]
pub struct CheckRequireAdmin<'info> {
    /// CHECK: Passed to `access_manager::require_admin` for deserialization
    pub access_manager: AccountInfo<'info>,
    /// Must hold the admin role in the access-manager.
    pub signer: Signer<'info>,
    /// CHECK: Validated inside the helper
    pub instructions_sysvar: AccountInfo<'info>,
}

pub fn check_require_admin(ctx: Context<CheckRequireAdmin>) -> Result<()> {
    access_manager::require_admin(
        &ctx.accounts.access_manager,
        &ctx.accounts.signer,
        &ctx.accounts.instructions_sysvar,
        &crate::ID,
    )
}

#[cfg(test)]
mod integration_tests {
    use anchor_lang::{InstructionData, ToAccountMetas};
    use solana_sdk::{
        instruction::Instruction, pubkey::Pubkey, signature::Keypair, signer::Signer,
        sysvar::instructions as ix_sysvar,
    };

    use crate::test_utils::*;

    fn build_ix(access_manager: Pubkey, signer: Pubkey) -> Instruction {
        let accounts = crate::accounts::CheckRequireAdmin {
            access_manager,
            signer,
            instructions_sysvar: ix_sysvar::ID,
        };
        Instruction {
            program_id: crate::ID,
            accounts: accounts.to_account_metas(None),
            data: crate::instruction::CheckRequireAdmin {}.data(),
        }
    }

    #[tokio::test]
    async fn test_direct_call_by_admin_succeeds() {
        let admin = Keypair::new();
        let mut pt = setup_program_test();
        let am_pubkey = add_access_manager_account(&mut pt, admin_roles(admin.pubkey()), vec![]);
        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let ix = build_ix(am_pubkey, admin.pubkey());
        process_tx_with_signers(&banks_client, &payer, &[&admin], recent_blockhash, &[ix])
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_direct_call_by_non_admin_fails() {
        let non_admin = Keypair::new();
        let mut pt = setup_program_test();
        let am_pubkey = add_access_manager_account(&mut pt, vec![], vec![]);
        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let ix = build_ix(am_pubkey, non_admin.pubkey());
        let err = process_tx_with_signers(
            &banks_client,
            &payer,
            &[&non_admin],
            recent_blockhash,
            &[ix],
        )
        .await
        .unwrap_err();
        assert_eq!(
            extract_custom_error(&err),
            Some(ANCHOR_ERROR_OFFSET + access_manager::AccessManagerError::Unauthorized as u32),
        );
    }

    #[tokio::test]
    async fn test_whitelisted_cpi_by_admin_succeeds() {
        let admin = Keypair::new();
        let mut pt = setup_program_test();
        let am_pubkey =
            add_access_manager_account(&mut pt, admin_roles(admin.pubkey()), vec![PROGRAM_A_ID]);
        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let inner_ix = build_ix(am_pubkey, admin.pubkey());
        let ix = build_single_cpi_ix(admin.pubkey(), &inner_ix);
        process_tx_with_signers(&banks_client, &payer, &[&admin], recent_blockhash, &[ix])
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_non_whitelisted_cpi_fails() {
        let admin = Keypair::new();
        let mut pt = setup_program_test();
        let am_pubkey = add_access_manager_account(&mut pt, admin_roles(admin.pubkey()), vec![]);
        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let inner_ix = build_ix(am_pubkey, admin.pubkey());
        let ix = build_single_cpi_ix(admin.pubkey(), &inner_ix);
        let err =
            process_tx_with_signers(&banks_client, &payer, &[&admin], recent_blockhash, &[ix])
                .await
                .unwrap_err();
        assert_eq!(
            extract_custom_error(&err),
            Some(ANCHOR_ERROR_OFFSET + access_manager::AccessManagerError::CpiNotAllowed as u32),
        );
    }
}
