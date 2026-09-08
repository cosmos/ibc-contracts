// SPDX-License-Identifier: Apache-2.0

use anchor_lang::prelude::*;

/// Accounts for [`require_role_with_whitelist`](access_manager::require_role_with_whitelist) wrapper.
#[derive(Accounts)]
#[instruction(role_id: u64)]
pub struct CheckRequireRoleWithWhitelist<'info> {
    /// CHECK: Passed to `access_manager::require_role_with_whitelist` for deserialization
    pub access_manager: AccountInfo<'info>,
    /// Must hold the requested role in the access-manager.
    pub signer: Signer<'info>,
    /// CHECK: Validated inside the helper
    pub instructions_sysvar: AccountInfo<'info>,
}

pub fn check_require_role_with_whitelist(
    ctx: Context<CheckRequireRoleWithWhitelist>,
    role_id: u64,
) -> Result<()> {
    access_manager::require_role_with_whitelist(
        &ctx.accounts.access_manager,
        role_id,
        &ctx.accounts.signer,
        &ctx.accounts.instructions_sysvar,
        &crate::ID,
    )
}

#[cfg(test)]
mod integration_tests {
    use anchor_lang::{InstructionData, ToAccountMetas};
    use solana_ibc_types::roles;
    use solana_sdk::{
        instruction::Instruction, pubkey::Pubkey, signature::Keypair, signer::Signer,
        sysvar::instructions as ix_sysvar,
    };

    use crate::test_utils::*;

    fn build_ix(access_manager: Pubkey, signer: Pubkey, role_id: u64) -> Instruction {
        let accounts = crate::accounts::CheckRequireRoleWithWhitelist {
            access_manager,
            signer,
            instructions_sysvar: ix_sysvar::ID,
        };
        Instruction {
            program_id: crate::ID,
            accounts: accounts.to_account_metas(None),
            data: crate::instruction::CheckRequireRoleWithWhitelist { role_id }.data(),
        }
    }

    #[tokio::test]
    async fn test_direct_call_with_role_succeeds() {
        let relayer = Keypair::new();
        let mut pt = setup_program_test();
        let am_pubkey =
            add_access_manager_account(&mut pt, relayer_roles(relayer.pubkey()), vec![]);
        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let ix = build_ix(am_pubkey, relayer.pubkey(), roles::RELAYER_ROLE);
        process_tx_with_signers(&banks_client, &payer, &[&relayer], recent_blockhash, &[ix])
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_direct_call_without_role_fails() {
        let non_relayer = Keypair::new();
        let mut pt = setup_program_test();
        let am_pubkey = add_access_manager_account(&mut pt, vec![], vec![]);
        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let ix = build_ix(am_pubkey, non_relayer.pubkey(), roles::RELAYER_ROLE);
        let err = process_tx_with_signers(
            &banks_client,
            &payer,
            &[&non_relayer],
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
    async fn test_whitelisted_cpi_with_role_succeeds() {
        let relayer = Keypair::new();
        let mut pt = setup_program_test();
        let am_pubkey = add_access_manager_account(
            &mut pt,
            relayer_roles(relayer.pubkey()),
            vec![PROGRAM_A_ID],
        );
        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let inner_ix = build_ix(am_pubkey, relayer.pubkey(), roles::RELAYER_ROLE);
        let ix = build_single_cpi_ix(relayer.pubkey(), &inner_ix);
        process_tx_with_signers(&banks_client, &payer, &[&relayer], recent_blockhash, &[ix])
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn test_non_whitelisted_cpi_fails() {
        let relayer = Keypair::new();
        let mut pt = setup_program_test();
        let am_pubkey =
            add_access_manager_account(&mut pt, relayer_roles(relayer.pubkey()), vec![]);
        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let inner_ix = build_ix(am_pubkey, relayer.pubkey(), roles::RELAYER_ROLE);
        let ix = build_single_cpi_ix(relayer.pubkey(), &inner_ix);
        let err =
            process_tx_with_signers(&banks_client, &payer, &[&relayer], recent_blockhash, &[ix])
                .await
                .unwrap_err();
        assert_eq!(
            extract_custom_error(&err),
            Some(ANCHOR_ERROR_OFFSET + access_manager::AccessManagerError::CpiNotAllowed as u32),
        );
    }
}
