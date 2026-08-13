// SPDX-License-Identifier: Apache-2.0

use crate::errors::RouterError;
use crate::state::*;
use anchor_lang::prelude::*;

/// Reclaims rent from previously uploaded payload and proof chunk accounts.
///
/// Chunk accounts to close are passed as remaining accounts, validated by
/// PDA derivation, zeroed and their lamports returned to the relayer.
#[derive(Accounts)]
#[instruction(msg: MsgCleanupChunks)]
pub struct CleanupChunks<'info> {
    /// Global router configuration PDA.
    #[account(
        seeds = [RouterState::SEED],
        bump
    )]
    pub router_state: Account<'info, RouterState>,

    /// Global access control state used for relayer role verification.
    /// CHECK: Validated by seeds constraint using stored `access_manager` program ID
    #[account(
        seeds = [access_manager::state::AccessManager::SEED],
        bump,
        seeds::program = router_state.am_state.access_manager,
    )]
    pub access_manager: AccountInfo<'info>,

    /// Relayer reclaiming rent; must hold the `RELAYER_ROLE`.
    /// Receives lamports from closed chunk accounts.
    #[account(mut)]
    pub relayer: Signer<'info>,

    /// Instructions sysvar used for CPI detection.
    /// CHECK: Address constraint verifies this is the instructions sysvar
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instructions_sysvar: AccountInfo<'info>,
}

pub fn cleanup_chunks<'info>(
    ctx: Context<'_, '_, '_, 'info, CleanupChunks<'info>>,
    msg: MsgCleanupChunks,
) -> Result<()> {
    access_manager::require_role(
        &ctx.accounts.access_manager,
        solana_ibc_types::roles::RELAYER_ROLE,
        &ctx.accounts.relayer,
        &ctx.accounts.instructions_sysvar,
        &crate::ID,
    )?;

    let relayer_key = ctx.accounts.relayer.key();
    let mut chunk_index = 0;

    // Clean payload chunks for each payload
    for (payload_idx, &total_chunks) in msg.payload_chunks.iter().enumerate() {
        for i in 0..total_chunks {
            require!(
                chunk_index < ctx.remaining_accounts.len(),
                RouterError::InvalidChunkCount
            );

            let chunk_account = &ctx.remaining_accounts[chunk_index];

            let expected_seeds = &[
                PayloadChunk::SEED,
                relayer_key.as_ref(),
                msg.client_id.as_bytes(),
                &msg.sequence.to_le_bytes(),
                &[payload_idx as u8],
                &[i],
            ];
            let (expected_pda, _) = Pubkey::find_program_address(expected_seeds, &crate::ID);

            require_keys_eq!(
                chunk_account.key(),
                expected_pda,
                RouterError::InvalidChunkAccount
            );

            require_keys_eq!(
                *chunk_account.owner,
                crate::ID,
                RouterError::InvalidAccountOwner
            );

            // Return rent to relayer
            cleanup_single_chunk(chunk_account, &ctx.accounts.relayer)?;
            chunk_index += 1;
        }
    }

    // Clean proof chunks
    for i in 0..msg.total_proof_chunks {
        require!(
            chunk_index < ctx.remaining_accounts.len(),
            RouterError::InvalidChunkCount
        );

        let chunk_account = &ctx.remaining_accounts[chunk_index];

        let expected_seeds = &[
            ProofChunk::SEED,
            relayer_key.as_ref(),
            msg.client_id.as_bytes(),
            &msg.sequence.to_le_bytes(),
            &[i],
        ];
        let (expected_pda, _) = Pubkey::find_program_address(expected_seeds, &crate::ID);

        require_keys_eq!(
            chunk_account.key(),
            expected_pda,
            RouterError::InvalidChunkAccount
        );

        require_keys_eq!(
            *chunk_account.owner,
            crate::ID,
            RouterError::InvalidAccountOwner
        );

        // Return rent to relayer
        cleanup_single_chunk(chunk_account, &ctx.accounts.relayer)?;
        chunk_index += 1;
    }

    Ok(())
}

fn cleanup_single_chunk<'info>(
    chunk_account: &AccountInfo<'info>,
    relayer: &Signer<'info>,
) -> Result<()> {
    {
        let mut data = chunk_account.try_borrow_mut_data()?;
        data.fill(0);
    }

    let mut chunk_lamports = chunk_account.try_borrow_mut_lamports()?;
    let mut relayer_lamports = relayer.try_borrow_mut_lamports()?;

    **relayer_lamports = relayer_lamports
        .checked_add(**chunk_lamports)
        .ok_or(RouterError::ArithmeticOverflow)?;
    **chunk_lamports = 0;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;
    use anchor_lang::InstructionData;
    use mollusk_svm::result::Check;
    use mollusk_svm::Mollusk;
    use solana_ibc_types::roles;
    use solana_sdk::account::Account;
    use solana_sdk::instruction::{AccountMeta, Instruction};
    use solana_sdk::program_error::ProgramError;
    use solana_sdk::pubkey::Pubkey;
    use solana_sdk::system_program;

    struct CleanupTestContext {
        instruction: Instruction,
        accounts: Vec<(Pubkey, Account)>,
        relayer: Pubkey,
        payload_chunk_pda: Pubkey,
        proof_chunk_pda: Pubkey,
        initial_relayer_balance: u64,
        chunk_rent: u64,
    }

    fn setup_cleanup_test(relayer_override: Option<Pubkey>) -> CleanupTestContext {
        let authority = Pubkey::new_unique();
        let relayer = relayer_override.unwrap_or(authority);
        let client_id = "test-client";
        let sequence = 42u64;
        let chunk_rent = 1_500_000u64;
        let initial_relayer_balance = 10_000_000_000u64;

        let (router_state_pda, router_state_data) = setup_router_state();
        let (access_manager_pda, access_manager_data) =
            setup_access_manager_with_roles(&[(roles::RELAYER_ROLE, &[authority])]);

        let payload_chunk_pda = Pubkey::find_program_address(
            &[
                PayloadChunk::SEED,
                relayer.as_ref(),
                client_id.as_bytes(),
                &sequence.to_le_bytes(),
                &[0u8],
                &[0u8],
            ],
            &crate::ID,
        )
        .0;

        let proof_chunk_pda = Pubkey::find_program_address(
            &[
                ProofChunk::SEED,
                relayer.as_ref(),
                client_id.as_bytes(),
                &sequence.to_le_bytes(),
                &[0u8],
            ],
            &crate::ID,
        )
        .0;

        let msg = MsgCleanupChunks {
            client_id: client_id.to_string(),
            sequence,
            payload_chunks: vec![1],
            total_proof_chunks: 1,
        };

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new_readonly(router_state_pda, false),
                AccountMeta::new_readonly(access_manager_pda, false),
                AccountMeta::new(relayer, true),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
                AccountMeta::new(payload_chunk_pda, false),
                AccountMeta::new(proof_chunk_pda, false),
            ],
            data: crate::instruction::CleanupChunks { msg }.data(),
        };

        let accounts = vec![
            create_account(router_state_pda, router_state_data, crate::ID),
            create_account(access_manager_pda, access_manager_data, access_manager::ID),
            (
                relayer,
                Account {
                    lamports: initial_relayer_balance,
                    data: vec![],
                    owner: system_program::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
            create_instructions_sysvar_account_with_caller(crate::ID),
            (
                payload_chunk_pda,
                Account {
                    lamports: chunk_rent,
                    data: vec![1u8; 100],
                    owner: crate::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
            (
                proof_chunk_pda,
                Account {
                    lamports: chunk_rent,
                    data: vec![2u8; 100],
                    owner: crate::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
            create_program_account(access_manager::ID),
        ];

        CleanupTestContext {
            instruction,
            accounts,
            relayer,
            payload_chunk_pda,
            proof_chunk_pda,
            initial_relayer_balance,
            chunk_rent,
        }
    }

    #[test]
    fn test_cleanup_chunks_success() {
        let ctx = setup_cleanup_test(None);

        let mollusk = Mollusk::new(&crate::ID, get_router_program_path());
        let result = mollusk.process_and_validate_instruction(
            &ctx.instruction,
            &ctx.accounts,
            &[Check::success()],
        );

        let relayer_account = result
            .get_account(&ctx.relayer)
            .expect("relayer account should exist");
        assert_eq!(
            relayer_account.lamports,
            ctx.initial_relayer_balance + (ctx.chunk_rent * 2),
        );

        let payload_chunk = result
            .get_account(&ctx.payload_chunk_pda)
            .expect("payload chunk account should exist");
        assert_eq!(payload_chunk.lamports, 0);
        assert!(payload_chunk.data.iter().all(|&b| b == 0));

        let proof_chunk = result
            .get_account(&ctx.proof_chunk_pda)
            .expect("proof chunk account should exist");
        assert_eq!(proof_chunk.lamports, 0);
        assert!(proof_chunk.data.iter().all(|&b| b == 0));
    }

    #[test]
    fn test_cleanup_chunks_unauthorized() {
        let ctx = setup_cleanup_test(Some(Pubkey::new_unique()));

        let mollusk = Mollusk::new(&crate::ID, get_router_program_path());

        let checks = vec![Check::err(ProgramError::Custom(
            ANCHOR_ERROR_OFFSET + access_manager::AccessManagerError::Unauthorized as u32,
        ))];

        mollusk.process_and_validate_instruction(&ctx.instruction, &ctx.accounts, &checks);
    }

    #[test]
    fn test_cleanup_chunks_cpi_rejection() {
        let mut ctx = setup_cleanup_test(None);

        let malicious_program = Pubkey::new_unique();
        let (instruction, cpi_sysvar_account) =
            setup_cpi_call_test(ctx.instruction, malicious_program);
        ctx.instruction = instruction;

        ctx.accounts
            .retain(|(pubkey, _)| *pubkey != solana_sdk::sysvar::instructions::ID);
        ctx.accounts.push(cpi_sysvar_account);

        let mollusk = Mollusk::new(&crate::ID, get_router_program_path());

        let checks = vec![Check::err(ProgramError::Custom(
            ANCHOR_ERROR_OFFSET + access_manager::AccessManagerError::CpiNotAllowed as u32,
        ))];

        mollusk.process_and_validate_instruction(&ctx.instruction, &ctx.accounts, &checks);
    }

    #[test]
    fn test_cleanup_chunks_fake_sysvar_wormhole_attack() {
        let mut ctx = setup_cleanup_test(None);

        let (instruction, fake_sysvar_account) =
            setup_fake_sysvar_attack(ctx.instruction, crate::ID);
        ctx.instruction = instruction;
        ctx.accounts.push(fake_sysvar_account);

        let mollusk = Mollusk::new(&crate::ID, get_router_program_path());
        mollusk.process_and_validate_instruction(
            &ctx.instruction,
            &ctx.accounts,
            &[expect_sysvar_attack_error()],
        );
    }
}

#[cfg(test)]
mod integration_tests {
    use crate::state::{MsgCleanupChunks, PayloadChunk, ProofChunk, RouterState};
    use crate::test_utils::*;
    use anchor_lang::InstructionData;
    use solana_sdk::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        signature::Keypair,
        signer::Signer,
    };

    fn build_cleanup_chunks_ix(relayer: Pubkey) -> Instruction {
        let client_id = "test-client";
        let sequence = 1u64;

        let (router_state_pda, _) = Pubkey::find_program_address(&[RouterState::SEED], &crate::ID);
        let (access_manager_pda, _) = Pubkey::find_program_address(
            &[access_manager::state::AccessManager::SEED],
            &access_manager::ID,
        );

        let payload_chunk_pda = Pubkey::find_program_address(
            &[
                PayloadChunk::SEED,
                relayer.as_ref(),
                client_id.as_bytes(),
                &sequence.to_le_bytes(),
                &[0u8],
                &[0u8],
            ],
            &crate::ID,
        )
        .0;

        let proof_chunk_pda = Pubkey::find_program_address(
            &[
                ProofChunk::SEED,
                relayer.as_ref(),
                client_id.as_bytes(),
                &sequence.to_le_bytes(),
                &[0u8],
            ],
            &crate::ID,
        )
        .0;

        Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new_readonly(router_state_pda, false),
                AccountMeta::new_readonly(access_manager_pda, false),
                AccountMeta::new(relayer, true),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
                AccountMeta::new(payload_chunk_pda, false),
                AccountMeta::new(proof_chunk_pda, false),
            ],
            data: crate::instruction::CleanupChunks {
                msg: MsgCleanupChunks {
                    client_id: client_id.to_string(),
                    sequence,
                    payload_chunks: vec![1],
                    total_proof_chunks: 1,
                },
            }
            .data(),
        }
    }

    #[tokio::test]
    async fn test_direct_call_without_relayer_role_rejected() {
        let relayer = Keypair::new();
        let non_relayer = Keypair::new();
        let mut pt = setup_program_test_with_roles_and_whitelist(
            &[(solana_ibc_types::roles::RELAYER_ROLE, &[relayer.pubkey()])],
            &[],
        );
        pt.add_account(
            non_relayer.pubkey(),
            solana_sdk::account::Account {
                lamports: 10_000_000_000,
                data: vec![],
                owner: solana_sdk::system_program::ID,
                executable: false,
                rent_epoch: 0,
            },
        );
        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let ix = build_cleanup_chunks_ix(non_relayer.pubkey());

        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[ix],
            Some(&payer.pubkey()),
            &[&payer, &non_relayer],
            recent_blockhash,
        );
        let err = banks_client.process_transaction(tx).await.unwrap_err();
        assert_eq!(
            pt_extract_custom_error(&err),
            Some(ANCHOR_ERROR_OFFSET + access_manager::AccessManagerError::Unauthorized as u32),
        );
    }

    #[tokio::test]
    async fn test_cpi_rejected() {
        let relayer = Keypair::new();
        let pt = setup_program_test_with_roles_and_whitelist(
            &[(solana_ibc_types::roles::RELAYER_ROLE, &[relayer.pubkey()])],
            &[TEST_CPI_TARGET_ID],
        );
        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let inner_ix = build_cleanup_chunks_ix(relayer.pubkey());
        let wrapped_ix = pt_wrap_in_test_cpi_proxy(relayer.pubkey(), &inner_ix);

        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[wrapped_ix],
            Some(&payer.pubkey()),
            &[&payer, &relayer],
            recent_blockhash,
        );
        let err = banks_client.process_transaction(tx).await.unwrap_err();
        assert_eq!(
            pt_extract_custom_error(&err),
            Some(ANCHOR_ERROR_OFFSET + access_manager::AccessManagerError::CpiNotAllowed as u32),
        );
    }
}
