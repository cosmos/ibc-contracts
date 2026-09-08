// SPDX-License-Identifier: Apache-2.0

use anchor_lang::prelude::*;
use solana_ibc_types::cpi::reject_direct_calls;

/// Accounts for [`reject_direct_calls`](solana_ibc_types::cpi::reject_direct_calls) wrapper.
#[derive(Accounts)]
pub struct CheckRejectDirectCalls<'info> {
    /// Transaction signer (required by Anchor but unused by the check).
    pub signer: Signer<'info>,
}

pub fn check_reject_direct_calls(_ctx: Context<CheckRejectDirectCalls>) -> Result<()> {
    reject_direct_calls().map_err(Into::into)
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
        let accounts = crate::accounts::CheckRejectDirectCalls { signer };
        Instruction {
            program_id: crate::ID,
            accounts: accounts.to_account_metas(None),
            data: crate::instruction::CheckRejectDirectCalls {}.data(),
        }
    }

    #[rstest]
    #[tokio::test]
    async fn test_reject_direct_calls_rejects_direct(#[future] ctx: TestContext) {
        let TestContext {
            banks_client,
            payer,
            recent_blockhash,
        } = ctx.await;

        let ix = build_ix(payer.pubkey());
        let result = process_tx(&banks_client, &payer, recent_blockhash, &[ix]).await;
        assert!(result.is_err(), "Direct call should be rejected");
    }

    #[rstest]
    #[tokio::test]
    async fn test_reject_direct_calls_allows_single_cpi(#[future] ctx: TestContext) {
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
    async fn test_reject_direct_calls_allows_nested_cpi(#[future] ctx: TestContext) {
        let TestContext {
            banks_client,
            payer,
            recent_blockhash,
        } = ctx.await;

        let inner_ix = build_ix(payer.pubkey());
        let ix = build_nested_cpi_ix(payer.pubkey(), &inner_ix);
        process_tx(&banks_client, &payer, recent_blockhash, &[ix])
            .await
            .unwrap();
    }
}
