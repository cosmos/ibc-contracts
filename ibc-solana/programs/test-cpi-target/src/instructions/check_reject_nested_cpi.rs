// SPDX-License-Identifier: Apache-2.0

use anchor_lang::prelude::*;
use solana_ibc_types::cpi::reject_nested_cpi;

/// Accounts for [`reject_nested_cpi`](solana_ibc_types::cpi::reject_nested_cpi) wrapper.
#[derive(Accounts)]
pub struct CheckRejectNestedCpi<'info> {
    /// Transaction signer (required by Anchor but unused by the check).
    pub signer: Signer<'info>,
}

pub fn check_reject_nested_cpi(_ctx: Context<CheckRejectNestedCpi>) -> Result<()> {
    reject_nested_cpi().map_err(Into::into)
}

#[cfg(test)]
mod integration_tests {
    use anchor_lang::{InstructionData, ToAccountMetas};
    use rstest::{fixture, rstest};
    use solana_sdk::{instruction::Instruction, pubkey::Pubkey, signer::Signer};

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

    fn build_ix(signer: Pubkey) -> Instruction {
        let accounts = crate::accounts::CheckRejectNestedCpi { signer };
        Instruction {
            program_id: crate::ID,
            accounts: accounts.to_account_metas(None),
            data: crate::instruction::CheckRejectNestedCpi {}.data(),
        }
    }

    #[rstest]
    #[tokio::test]
    async fn test_reject_nested_allows_direct(#[future] ctx: TestContext) {
        let TestContext {
            banks_client,
            payer,
            recent_blockhash,
        } = ctx.await;

        let ix = build_ix(payer.pubkey());
        process_tx(&banks_client, &payer, recent_blockhash, &[ix])
            .await
            .unwrap();
    }

    #[rstest]
    #[tokio::test]
    async fn test_reject_nested_allows_single_cpi(#[future] ctx: TestContext) {
        let TestContext {
            banks_client,
            payer,
            recent_blockhash,
        } = ctx.await;

        let inner_ix = build_ix(payer.pubkey());
        let ix = build_single_cpi_ix(payer.pubkey(), &inner_ix);
        process_tx(&banks_client, &payer, recent_blockhash, &[ix])
            .await
            .unwrap();
    }

    #[rstest]
    #[tokio::test]
    async fn test_reject_nested_rejects_double_cpi(#[future] ctx: TestContext) {
        let TestContext {
            banks_client,
            payer,
            recent_blockhash,
        } = ctx.await;

        let inner_ix = build_ix(payer.pubkey());
        let ix = build_nested_cpi_ix(payer.pubkey(), &inner_ix);
        let result = process_tx(&banks_client, &payer, recent_blockhash, &[ix]).await;
        assert!(result.is_err(), "Nested CPI should be rejected");
    }
}
