// SPDX-License-Identifier: Apache-2.0

use anchor_lang::prelude::*;
use solana_ibc_types::cpi::require_direct_call_or_whitelisted_caller;

/// Accounts for [`require_direct_call_or_whitelisted_caller`](solana_ibc_types::cpi::require_direct_call_or_whitelisted_caller) wrapper.
#[derive(Accounts)]
#[instruction(whitelisted_programs: Vec<Pubkey>)]
pub struct CheckDirectOrWhitelisted<'info> {
    /// CHECK: Validated inside the CPI validation functions
    pub instruction_sysvar: AccountInfo<'info>,
}

pub fn check_direct_or_whitelisted(
    ctx: Context<CheckDirectOrWhitelisted>,
    whitelisted_programs: Vec<Pubkey>,
) -> Result<()> {
    require_direct_call_or_whitelisted_caller(
        &ctx.accounts.instruction_sysvar,
        &whitelisted_programs,
        &crate::ID,
    )
    .map_err(Into::into)
}

#[cfg(test)]
mod integration_tests {
    use anchor_lang::{InstructionData, ToAccountMetas};
    use rstest::{fixture, rstest};
    use solana_sdk::{
        instruction::Instruction, pubkey::Pubkey, signer::Signer, sysvar::instructions as ix_sysvar,
    };

    use crate::test_utils::*;

    #[fixture]
    async fn ctx() -> TestContext {
        let pt = setup_program_test();
        let (banks_client, payer, recent_blockhash) = pt.start().await;
        TestContext {
            banks_client,
            payer,
            recent_blockhash,
        }
    }

    fn build_ix(whitelisted_programs: Vec<Pubkey>) -> Instruction {
        let accounts = crate::accounts::CheckDirectOrWhitelisted {
            instruction_sysvar: ix_sysvar::ID,
        };
        Instruction {
            program_id: crate::ID,
            accounts: accounts.to_account_metas(None),
            data: crate::instruction::CheckDirectOrWhitelisted {
                whitelisted_programs,
            }
            .data(),
        }
    }

    #[rstest]
    #[tokio::test]
    async fn test_whitelisted_allows_direct(#[future] ctx: TestContext) {
        let TestContext {
            banks_client,
            payer,
            recent_blockhash,
        } = ctx.await;

        let ix = build_ix(vec![]);
        process_tx(&banks_client, &payer, recent_blockhash, &[ix])
            .await
            .unwrap();
    }

    #[rstest]
    #[tokio::test]
    async fn test_whitelisted_allows_listed_cpi(#[future] ctx: TestContext) {
        let TestContext {
            banks_client,
            payer,
            recent_blockhash,
        } = ctx.await;

        let inner_ix = build_ix(vec![PROGRAM_A_ID]);
        let ix = build_single_cpi_ix(payer.pubkey(), &inner_ix);
        process_tx(&banks_client, &payer, recent_blockhash, &[ix])
            .await
            .unwrap();
    }

    #[rstest]
    #[tokio::test]
    async fn test_whitelisted_rejects_unlisted_cpi(#[future] ctx: TestContext) {
        let TestContext {
            banks_client,
            payer,
            recent_blockhash,
        } = ctx.await;

        let random_program = Pubkey::new_unique();
        let inner_ix = build_ix(vec![random_program]);
        let ix = build_single_cpi_ix(payer.pubkey(), &inner_ix);
        let result = process_tx(&banks_client, &payer, recent_blockhash, &[ix]).await;
        assert!(
            result.is_err(),
            "require_direct_call_or_whitelisted_caller should reject unlisted CPI caller"
        );
    }

    #[rstest]
    #[tokio::test]
    async fn test_whitelisted_rejects_nested(#[future] ctx: TestContext) {
        let TestContext {
            banks_client,
            payer,
            recent_blockhash,
        } = ctx.await;

        let inner_ix = build_ix(vec![PROGRAM_A_ID]);
        let ix = build_nested_cpi_ix(payer.pubkey(), &inner_ix);
        let result = process_tx(&banks_client, &payer, recent_blockhash, &[ix]).await;
        assert!(
            result.is_err(),
            "require_direct_call_or_whitelisted_caller should reject nested CPI"
        );
    }

    #[rstest]
    #[tokio::test]
    async fn test_whitelisted_rejects_fake_sysvar(#[future] ctx: TestContext) {
        let TestContext {
            banks_client,
            payer,
            recent_blockhash,
        } = ctx.await;

        let accounts = crate::accounts::CheckDirectOrWhitelisted {
            instruction_sysvar: Pubkey::new_unique(),
        };
        let ix = Instruction {
            program_id: crate::ID,
            accounts: accounts.to_account_metas(None),
            data: crate::instruction::CheckDirectOrWhitelisted {
                whitelisted_programs: vec![],
            }
            .data(),
        };
        let result = process_tx(&banks_client, &payer, recent_blockhash, &[ix]).await;
        assert!(
            result.is_err(),
            "require_direct_call_or_whitelisted_caller should reject fake sysvar account"
        );
    }

    #[rstest]
    #[tokio::test]
    async fn test_whitelisted_allows_non_first_entry(#[future] ctx: TestContext) {
        let TestContext {
            banks_client,
            payer,
            recent_blockhash,
        } = ctx.await;

        let inner_ix = build_ix(vec![Pubkey::new_unique(), PROGRAM_A_ID]);
        let ix = build_single_cpi_ix(payer.pubkey(), &inner_ix);
        process_tx(&banks_client, &payer, recent_blockhash, &[ix])
            .await
            .unwrap();
    }
}
