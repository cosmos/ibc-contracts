// SPDX-License-Identifier: Apache-2.0

use crate::errors::RouterError;
use crate::state::*;
use anchor_lang::prelude::*;

/// Uploads a single proof data chunk for multi-transaction packet delivery.
/// Chunks are assembled later during `recv_packet`, `ack_packet` or
/// `timeout_packet`.
#[derive(Accounts)]
#[instruction(msg: MsgUploadChunk)]
pub struct UploadProofChunk<'info> {
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

    /// Temporary storage for one proof chunk, keyed by relayer, client,
    /// sequence and chunk index.
    #[account(
        init_if_needed,
        payer = relayer,
        space = 8 + ProofChunk::INIT_SPACE,
        seeds = [
            ProofChunk::SEED,
            relayer.key().as_ref(),
            msg.client_id.as_bytes(),
            &msg.sequence.to_le_bytes(),
            &[msg.chunk_index]
        ],
        bump
    )]
    pub chunk: Account<'info, ProofChunk>,

    /// Relayer uploading the chunk; must hold the `RELAYER_ROLE` and pays rent.
    #[account(mut)]
    pub relayer: Signer<'info>,

    /// Solana system program used for account creation.
    pub system_program: Program<'info, System>,

    /// Instructions sysvar used for CPI detection.
    /// CHECK: Address constraint verifies this is the instructions sysvar
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instructions_sysvar: AccountInfo<'info>,
}

pub fn upload_proof_chunk(ctx: Context<UploadProofChunk>, msg: MsgUploadChunk) -> Result<()> {
    access_manager::require_role(
        &ctx.accounts.access_manager,
        solana_ibc_types::roles::RELAYER_ROLE,
        &ctx.accounts.relayer,
        &ctx.accounts.instructions_sysvar,
        &crate::ID,
    )?;

    let chunk = &mut ctx.accounts.chunk;

    require!(
        msg.chunk_data.len() <= CHUNK_DATA_SIZE,
        RouterError::ChunkDataTooLarge
    );

    chunk.client_id = msg.client_id;
    chunk.sequence = msg.sequence;
    chunk.chunk_index = msg.chunk_index;
    chunk.chunk_data = msg.chunk_data;

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
    use solana_sdk::instruction::{AccountMeta, Instruction};
    use solana_sdk::program_error::ProgramError;
    use solana_sdk::pubkey::Pubkey;
    use solana_sdk::system_program;

    struct UploadProofChunkTestContext {
        instruction: Instruction,
        accounts: Vec<(Pubkey, solana_sdk::account::Account)>,
        chunk_pda: Pubkey,
    }

    fn setup_upload_proof_chunk_test(
        relayer_override: Option<Pubkey>,
    ) -> UploadProofChunkTestContext {
        let authority = Pubkey::new_unique();
        let relayer = relayer_override.unwrap_or(authority);
        let client_id = "test-client";
        let sequence = 42u64;
        let chunk_index = 0u8;
        let chunk_data = vec![2u8; 200];

        let (router_state_pda, router_state_data) = setup_router_state();
        let (access_manager_pda, access_manager_data) =
            setup_access_manager_with_roles(&[(roles::RELAYER_ROLE, &[authority])]);

        let chunk_pda = Pubkey::find_program_address(
            &[
                ProofChunk::SEED,
                relayer.as_ref(),
                client_id.as_bytes(),
                &sequence.to_le_bytes(),
                &[chunk_index],
            ],
            &crate::ID,
        )
        .0;

        let instruction_data = crate::instruction::UploadProofChunk {
            msg: MsgUploadChunk {
                client_id: client_id.to_string(),
                sequence,
                payload_index: 0,
                chunk_index,
                chunk_data,
            },
        };

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new_readonly(router_state_pda, false),
                AccountMeta::new_readonly(access_manager_pda, false),
                AccountMeta::new(chunk_pda, false),
                AccountMeta::new(relayer, true),
                AccountMeta::new_readonly(system_program::ID, false),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
            ],
            data: instruction_data.data(),
        };

        let accounts = vec![
            create_account(router_state_pda, router_state_data, crate::ID),
            create_account(access_manager_pda, access_manager_data, access_manager::ID),
            create_uninitialized_account(chunk_pda, 0),
            create_system_account(relayer),
            create_program_account(system_program::ID),
            create_instructions_sysvar_account_with_caller(crate::ID),
            create_program_account(access_manager::ID),
        ];

        UploadProofChunkTestContext {
            instruction,
            accounts,
            chunk_pda,
        }
    }

    #[test]
    fn test_upload_proof_chunk_success() {
        let ctx = setup_upload_proof_chunk_test(None);

        let mollusk = Mollusk::new(&crate::ID, get_router_program_path());
        let result = mollusk.process_and_validate_instruction(
            &ctx.instruction,
            &ctx.accounts,
            &[Check::success()],
        );

        let chunk_account = result
            .get_account(&ctx.chunk_pda)
            .expect("chunk account should exist");
        assert_eq!(chunk_account.owner, crate::ID);
        assert!(chunk_account.lamports > 0);

        let chunk_data_raw = &chunk_account.data[8..];
        let chunk: ProofChunk = AnchorDeserialize::deserialize(&mut &chunk_data_raw[..])
            .expect("should deserialize chunk");
        assert_eq!(chunk.client_id, "test-client");
        assert_eq!(chunk.sequence, 42);
        assert_eq!(chunk.chunk_index, 0);
        assert_eq!(chunk.chunk_data, vec![2u8; 200]);
    }

    #[test]
    fn test_upload_proof_chunk_reupload_overwrites() {
        let authority = Pubkey::new_unique();
        let client_id = "test-client";
        let sequence = 42u64;
        let chunk_index = 0u8;
        let new_chunk_data = vec![9u8; 50];

        let (router_state_pda, router_state_data) = setup_router_state();
        let (access_manager_pda, access_manager_data) =
            setup_access_manager_with_roles(&[(roles::RELAYER_ROLE, &[authority])]);

        let chunk_pda = Pubkey::find_program_address(
            &[
                ProofChunk::SEED,
                authority.as_ref(),
                client_id.as_bytes(),
                &sequence.to_le_bytes(),
                &[chunk_index],
            ],
            &crate::ID,
        )
        .0;

        let existing_chunk = ProofChunk {
            client_id: client_id.to_string(),
            sequence,
            chunk_index,
            chunk_data: vec![2u8; 200],
        };
        let existing_data = create_account_data(&existing_chunk);
        let space = 8 + ProofChunk::INIT_SPACE;
        let mut padded_data = existing_data;
        padded_data.resize(space, 0);

        let instruction_data = crate::instruction::UploadProofChunk {
            msg: MsgUploadChunk {
                client_id: client_id.to_string(),
                sequence,
                payload_index: 0,
                chunk_index,
                chunk_data: new_chunk_data.clone(),
            },
        };

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new_readonly(router_state_pda, false),
                AccountMeta::new_readonly(access_manager_pda, false),
                AccountMeta::new(chunk_pda, false),
                AccountMeta::new(authority, true),
                AccountMeta::new_readonly(system_program::ID, false),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
            ],
            data: instruction_data.data(),
        };

        let accounts = vec![
            create_account(router_state_pda, router_state_data, crate::ID),
            create_account(access_manager_pda, access_manager_data, access_manager::ID),
            create_account(chunk_pda, padded_data, crate::ID),
            create_system_account(authority),
            create_program_account(system_program::ID),
            create_instructions_sysvar_account_with_caller(crate::ID),
            create_program_account(access_manager::ID),
        ];

        let mollusk = Mollusk::new(&crate::ID, get_router_program_path());
        let result =
            mollusk.process_and_validate_instruction(&instruction, &accounts, &[Check::success()]);

        let chunk_account = result
            .get_account(&chunk_pda)
            .expect("chunk account should exist");
        let chunk_data_raw = &chunk_account.data[8..];
        let chunk: ProofChunk = AnchorDeserialize::deserialize(&mut &chunk_data_raw[..])
            .expect("should deserialize chunk");
        assert_eq!(chunk.chunk_data, new_chunk_data);
    }

    #[test]
    fn test_upload_proof_chunk_data_too_large() {
        let authority = Pubkey::new_unique();
        let client_id = "test-client";
        let sequence = 42u64;
        let chunk_index = 0u8;
        let chunk_data = vec![2u8; CHUNK_DATA_SIZE + 1];

        let (router_state_pda, router_state_data) = setup_router_state();
        let (access_manager_pda, access_manager_data) =
            setup_access_manager_with_roles(&[(roles::RELAYER_ROLE, &[authority])]);

        let chunk_pda = Pubkey::find_program_address(
            &[
                ProofChunk::SEED,
                authority.as_ref(),
                client_id.as_bytes(),
                &sequence.to_le_bytes(),
                &[chunk_index],
            ],
            &crate::ID,
        )
        .0;

        let instruction_data = crate::instruction::UploadProofChunk {
            msg: MsgUploadChunk {
                client_id: client_id.to_string(),
                sequence,
                payload_index: 0,
                chunk_index,
                chunk_data,
            },
        };

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new_readonly(router_state_pda, false),
                AccountMeta::new_readonly(access_manager_pda, false),
                AccountMeta::new(chunk_pda, false),
                AccountMeta::new(authority, true),
                AccountMeta::new_readonly(system_program::ID, false),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
            ],
            data: instruction_data.data(),
        };

        let accounts = vec![
            create_account(router_state_pda, router_state_data, crate::ID),
            create_account(access_manager_pda, access_manager_data, access_manager::ID),
            create_uninitialized_account(chunk_pda, 0),
            create_system_account(authority),
            create_program_account(system_program::ID),
            create_instructions_sysvar_account_with_caller(crate::ID),
            create_program_account(access_manager::ID),
        ];

        let mollusk = Mollusk::new(&crate::ID, get_router_program_path());
        let result = mollusk.process_instruction(&instruction, &accounts);

        assert_error_code(
            result,
            RouterError::ChunkDataTooLarge,
            "upload_proof_chunk_data_too_large",
        );
    }

    #[test]
    fn test_upload_proof_chunk_unauthorized() {
        let ctx = setup_upload_proof_chunk_test(Some(Pubkey::new_unique()));

        let mollusk = Mollusk::new(&crate::ID, get_router_program_path());
        mollusk.process_and_validate_instruction(
            &ctx.instruction,
            &ctx.accounts,
            &[Check::err(ProgramError::Custom(
                ANCHOR_ERROR_OFFSET + access_manager::AccessManagerError::Unauthorized as u32,
            ))],
        );
    }

    #[test]
    fn test_upload_proof_chunk_cpi_rejection() {
        let mut ctx = setup_upload_proof_chunk_test(None);

        let malicious_program = Pubkey::new_unique();
        let (instruction, cpi_sysvar_account) =
            setup_cpi_call_test(ctx.instruction, malicious_program);
        ctx.instruction = instruction;

        ctx.accounts
            .retain(|(pubkey, _)| *pubkey != solana_sdk::sysvar::instructions::ID);
        ctx.accounts.push(cpi_sysvar_account);

        let mollusk = Mollusk::new(&crate::ID, get_router_program_path());
        mollusk.process_and_validate_instruction(
            &ctx.instruction,
            &ctx.accounts,
            &[Check::err(ProgramError::Custom(
                ANCHOR_ERROR_OFFSET + access_manager::AccessManagerError::CpiNotAllowed as u32,
            ))],
        );
    }

    #[test]
    fn test_upload_proof_chunk_fake_sysvar_attack() {
        let mut ctx = setup_upload_proof_chunk_test(None);

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
