// SPDX-License-Identifier: Apache-2.0

use crate::error::ErrorCode;
use crate::state::{MisbehaviourChunk, CHUNK_DATA_SIZE};
use crate::types::{AppState, ClientState, UploadMisbehaviourChunkParams};
use anchor_lang::prelude::*;

/// Uploads a single chunk of serialized misbehaviour evidence.
///
/// Misbehaviour data that exceeds a single transaction is split into chunks and
/// reassembled later by `assemble_and_submit_misbehaviour`.
#[derive(Accounts)]
#[instruction(params: UploadMisbehaviourChunkParams)]
pub struct UploadMisbehaviourChunk<'info> {
    /// PDA storing one segment of the serialized misbehaviour, keyed by submitter and chunk index.
    #[account(
        init_if_needed,
        payer = submitter,
        space = 8 + MisbehaviourChunk::INIT_SPACE,
        seeds = [
            MisbehaviourChunk::SEED,
            submitter.key().as_ref(),
            &[params.chunk_index]
        ],
        bump
    )]
    pub chunk: Account<'info, MisbehaviourChunk>,

    /// PDA holding the light client configuration; used to check the frozen status.
    #[account(
        seeds = [ClientState::SEED],
        bump
    )]
    pub client_state: Account<'info, ClientState>,

    /// PDA holding program-level settings; provides the `access_manager` address for role checks.
    #[account(
        seeds = [AppState::SEED],
        bump
    )]
    pub app_state: Account<'info, AppState>,

    /// Access-manager PDA used to verify the submitter holds the relayer role.
    /// CHECK: Validated by seeds constraint using stored `access_manager` program ID
    #[account(
        seeds = [access_manager::state::AccessManager::SEED],
        bump,
        seeds::program = app_state.am_state.access_manager
    )]
    pub access_manager: AccountInfo<'info>,

    /// Relayer that signs the transaction and pays for chunk account creation.
    #[account(mut)]
    pub submitter: Signer<'info>,

    /// Instructions sysvar used by the access manager to inspect the transaction.
    /// CHECK: Address constraint verifies this is the instructions sysvar
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instructions_sysvar: AccountInfo<'info>,

    /// Required by Anchor for PDA creation via the System Program.
    pub system_program: Program<'info, System>,
}

pub fn upload_misbehaviour_chunk(
    ctx: Context<UploadMisbehaviourChunk>,
    params: UploadMisbehaviourChunkParams,
) -> Result<()> {
    access_manager::require_role(
        &ctx.accounts.access_manager,
        solana_ibc_types::roles::RELAYER_ROLE,
        &ctx.accounts.submitter,
        &ctx.accounts.instructions_sysvar,
        &crate::ID,
    )?;

    require!(
        !ctx.accounts.client_state.is_frozen(),
        ErrorCode::ClientFrozen
    );
    let chunk = &mut ctx.accounts.chunk;

    require!(
        params.chunk_data.len() <= CHUNK_DATA_SIZE,
        ErrorCode::ChunkDataTooLarge
    );

    chunk.chunk_data = params.chunk_data;

    Ok(())
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod integration_tests {
    use crate::test_helpers::*;
    use crate::types::{ClientState, IbcHeight, UploadMisbehaviourChunkParams};
    use anchor_lang::{AccountSerialize, InstructionData};
    use solana_sdk::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        signature::Keypair,
        signer::Signer,
    };

    fn add_client_state(pt: &mut solana_program_test::ProgramTest) {
        let (client_state_pda, _) = Pubkey::find_program_address(&[ClientState::SEED], &crate::ID);

        let client_state = ClientState {
            chain_id: "test-chain".to_string(),
            trust_level_numerator: 2,
            trust_level_denominator: 3,
            trusting_period: 86400,
            unbonding_period: 172_800,
            max_clock_drift: 600,
            frozen_height: IbcHeight {
                revision_number: 0,
                revision_height: 0,
            },
            latest_height: IbcHeight {
                revision_number: 0,
                revision_height: 100,
            },
        };

        let mut data = vec![];
        client_state.try_serialize(&mut data).unwrap();

        pt.add_account(
            client_state_pda,
            solana_sdk::account::Account {
                lamports: 1_000_000,
                data,
                owner: crate::ID,
                executable: false,
                rent_epoch: 0,
            },
        );
    }

    fn build_upload_misbehaviour_chunk_ix(submitter: Pubkey, chunk_index: u8) -> Instruction {
        let (chunk_pda, _) = Pubkey::find_program_address(
            &[
                crate::state::MisbehaviourChunk::SEED,
                submitter.as_ref(),
                &[chunk_index],
            ],
            &crate::ID,
        );
        let (client_state_pda, _) = Pubkey::find_program_address(&[ClientState::SEED], &crate::ID);
        let (app_state_pda, _) =
            Pubkey::find_program_address(&[crate::types::AppState::SEED], &crate::ID);
        let (access_manager_pda, _) = Pubkey::find_program_address(
            &[access_manager::state::AccessManager::SEED],
            &access_manager::ID,
        );

        let params = UploadMisbehaviourChunkParams {
            chunk_index,
            chunk_data: vec![1u8; 100],
        };

        Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(chunk_pda, false),
                AccountMeta::new_readonly(client_state_pda, false),
                AccountMeta::new_readonly(app_state_pda, false),
                AccountMeta::new_readonly(access_manager_pda, false),
                AccountMeta::new(submitter, true),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
                AccountMeta::new_readonly(solana_sdk::system_program::ID, false),
            ],
            data: crate::instruction::UploadMisbehaviourChunk { params }.data(),
        }
    }

    #[tokio::test]
    async fn test_direct_call_by_relayer_succeeds() {
        let relayer = Keypair::new();
        let mut pt = setup_program_test_with_relayer(&relayer.pubkey());
        add_client_state(&mut pt);
        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let ix = build_upload_misbehaviour_chunk_ix(relayer.pubkey(), 0);

        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[ix],
            Some(&payer.pubkey()),
            &[&payer, &relayer],
            recent_blockhash,
        );
        let result = banks_client.process_transaction(tx).await;
        assert!(
            result.is_ok(),
            "Direct call by relayer should succeed: {:?}",
            result.err()
        );
    }

    #[tokio::test]
    async fn test_direct_call_without_relayer_role_rejected() {
        let relayer = Keypair::new();
        let non_relayer = Keypair::new();
        let mut pt = setup_program_test_with_relayer(&relayer.pubkey());
        add_client_state(&mut pt);
        fund_account(&mut pt, &non_relayer.pubkey());
        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let ix = build_upload_misbehaviour_chunk_ix(non_relayer.pubkey(), 0);

        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[ix],
            Some(&payer.pubkey()),
            &[&payer, &non_relayer],
            recent_blockhash,
        );
        let err = banks_client.process_transaction(tx).await.unwrap_err();
        assert_eq!(
            extract_custom_error(&err),
            Some(ANCHOR_ERROR_OFFSET + access_manager::AccessManagerError::Unauthorized as u32),
        );
    }

    #[tokio::test]
    async fn test_cpi_rejected() {
        let relayer = Keypair::new();
        let mut pt = setup_program_test_with_relayer(&relayer.pubkey());
        add_client_state(&mut pt);
        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let inner_ix = build_upload_misbehaviour_chunk_ix(relayer.pubkey(), 0);
        let wrapped_ix = wrap_in_test_cpi_proxy(relayer.pubkey(), &inner_ix);

        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[wrapped_ix],
            Some(&payer.pubkey()),
            &[&payer, &relayer],
            recent_blockhash,
        );
        let err = banks_client.process_transaction(tx).await.unwrap_err();
        assert_eq!(
            extract_custom_error(&err),
            Some(ANCHOR_ERROR_OFFSET + access_manager::AccessManagerError::CpiNotAllowed as u32),
        );
    }
}
