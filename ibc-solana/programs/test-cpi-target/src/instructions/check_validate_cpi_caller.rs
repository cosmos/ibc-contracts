// SPDX-License-Identifier: Apache-2.0

use anchor_lang::prelude::*;
use solana_ibc_types::cpi::validate_cpi_caller;

/// Accounts for [`validate_cpi_caller`](solana_ibc_types::cpi::validate_cpi_caller) wrapper.
#[derive(Accounts)]
#[instruction(authorized_program: Pubkey)]
pub struct CheckValidateCpiCaller<'info> {
    /// CHECK: Validated inside the CPI validation functions
    pub instruction_sysvar: AccountInfo<'info>,
}

pub fn check_validate_cpi_caller(
    ctx: Context<CheckValidateCpiCaller>,
    authorized_program: Pubkey,
) -> Result<()> {
    validate_cpi_caller(
        &ctx.accounts.instruction_sysvar,
        &authorized_program,
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

    fn build_ix(authorized_program: Pubkey) -> Instruction {
        let accounts = crate::accounts::CheckValidateCpiCaller {
            instruction_sysvar: ix_sysvar::ID,
        };
        Instruction {
            program_id: crate::ID,
            accounts: accounts.to_account_metas(None),
            data: crate::instruction::CheckValidateCpiCaller { authorized_program }.data(),
        }
    }

    #[rstest]
    #[tokio::test]
    async fn test_validate_cpi_rejects_direct_call(#[future] ctx: TestContext) {
        let TestContext {
            banks_client,
            payer,
            recent_blockhash,
        } = ctx.await;

        let ix = build_ix(PROGRAM_A_ID);
        let result = process_tx(&banks_client, &payer, recent_blockhash, &[ix]).await;
        assert!(
            result.is_err(),
            "validate_cpi_caller should reject direct calls"
        );
    }

    #[rstest]
    #[tokio::test]
    async fn test_validate_cpi_accepts_authorized(#[future] ctx: TestContext) {
        let TestContext {
            banks_client,
            payer,
            recent_blockhash,
        } = ctx.await;

        let inner_ix = build_ix(PROGRAM_A_ID);
        let ix = build_single_cpi_ix(payer.pubkey(), &inner_ix);
        process_tx(&banks_client, &payer, recent_blockhash, &[ix])
            .await
            .unwrap();
    }

    #[rstest]
    #[tokio::test]
    async fn test_validate_cpi_rejects_unauthorized(#[future] ctx: TestContext) {
        let TestContext {
            banks_client,
            payer,
            recent_blockhash,
        } = ctx.await;

        let random_program = Pubkey::new_unique();
        let inner_ix = build_ix(random_program);
        let ix = build_single_cpi_ix(payer.pubkey(), &inner_ix);
        let result = process_tx(&banks_client, &payer, recent_blockhash, &[ix]).await;
        assert!(
            result.is_err(),
            "validate_cpi_caller should reject unauthorized caller"
        );
    }

    #[rstest]
    #[tokio::test]
    async fn test_validate_cpi_rejects_nested(#[future] ctx: TestContext) {
        let TestContext {
            banks_client,
            payer,
            recent_blockhash,
        } = ctx.await;

        let inner_ix = build_ix(PROGRAM_A_ID);
        let ix = build_nested_cpi_ix(payer.pubkey(), &inner_ix);
        let result = process_tx(&banks_client, &payer, recent_blockhash, &[ix]).await;
        assert!(
            result.is_err(),
            "validate_cpi_caller should reject nested CPI"
        );
    }

    #[rstest]
    #[tokio::test]
    async fn test_validate_cpi_rejects_fake_sysvar(#[future] ctx: TestContext) {
        let TestContext {
            banks_client,
            payer,
            recent_blockhash,
        } = ctx.await;

        let accounts = crate::accounts::CheckValidateCpiCaller {
            instruction_sysvar: Pubkey::new_unique(),
        };
        let ix = Instruction {
            program_id: crate::ID,
            accounts: accounts.to_account_metas(None),
            data: crate::instruction::CheckValidateCpiCaller {
                authorized_program: PROGRAM_A_ID,
            }
            .data(),
        };
        let result = process_tx(&banks_client, &payer, recent_blockhash, &[ix]).await;
        assert!(
            result.is_err(),
            "validate_cpi_caller should reject fake sysvar account"
        );
    }
}
