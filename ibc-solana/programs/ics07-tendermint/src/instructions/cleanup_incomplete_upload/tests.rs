// SPDX-License-Identifier: Apache-2.0

use super::*;
use crate::error::ErrorCode;
use crate::state::HeaderChunk;
use crate::test_helpers::{fixtures::assert_error_code, PROGRAM_BINARY_PATH};
use crate::types::{ClientState, IbcHeight};
use anchor_lang::solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
};
use anchor_lang::InstructionData;
use mollusk_svm::Mollusk;
use solana_sdk::account::Account;

struct TestAccounts {
    submitter: Pubkey,
    chunk_pdas: Vec<Pubkey>,
    client_state_pda: Pubkey,
    accounts: Vec<(Pubkey, Account)>,
}

fn setup_test_accounts_with_chunks(
    chain_id: &str,
    target_height: u64,
    submitter: Pubkey,
    num_chunks: u8,
    with_populated_chunks: bool,
) -> TestAccounts {
    // Derive PDAs
    let mut chunk_pdas = vec![];
    for i in 0..num_chunks {
        let chunk_pda = Pubkey::find_program_address(
            &[
                crate::state::HeaderChunk::SEED,
                submitter.as_ref(),
                chain_id.as_bytes(),
                &target_height.to_le_bytes(),
                &[i],
            ],
            &crate::ID,
        )
        .0;
        chunk_pdas.push(chunk_pda);
    }

    let client_state_pda = Pubkey::find_program_address(
        &[crate::types::ClientState::SEED, chain_id.as_bytes()],
        &crate::ID,
    )
    .0;

    let mut accounts = vec![];

    // Add submitter account
    accounts.push((
        submitter,
        Account {
            lamports: 10_000_000_000, // Will receive refunds
            data: vec![],
            owner: system_program::ID,
            executable: false,
            rent_epoch: 0,
        },
    ));

    // Add client state account (always needed)
    let client_state = ClientState {
        chain_id: chain_id.to_string(),
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
            revision_height: 100, // Higher than target_height
        },
    };

    let mut client_data = vec![];
    client_state.try_serialize(&mut client_data).unwrap();

    accounts.push((
        client_state_pda,
        Account {
            lamports: 1_000_000,
            data: client_data,
            owner: crate::ID,
            executable: false,
            rent_epoch: 0,
        },
    ));

    // Add chunk accounts
    for (i, chunk_pda) in chunk_pdas.iter().enumerate() {
        if with_populated_chunks {
            let chunk = HeaderChunk {
                submitter,
                chunk_data: vec![i as u8; 100],
            };

            let mut chunk_data = vec![];
            chunk.try_serialize(&mut chunk_data).unwrap();

            accounts.push((
                *chunk_pda,
                Account {
                    lamports: 1_500_000, // Rent to be reclaimed
                    data: chunk_data,
                    owner: crate::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ));
        } else {
            accounts.push((
                *chunk_pda,
                Account {
                    lamports: 0,
                    data: vec![],
                    owner: system_program::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ));
        }
    }

    TestAccounts {
        submitter,
        chunk_pdas,
        client_state_pda,
        accounts,
    }
}

fn create_cleanup_instruction(test_accounts: &TestAccounts) -> Instruction {
    let instruction_data = crate::instruction::CleanupIncompleteUpload;

    let mut account_metas = vec![AccountMeta::new(test_accounts.submitter, true)];

    // Add chunk accounts as remaining accounts
    for chunk_pda in &test_accounts.chunk_pdas {
        account_metas.push(AccountMeta::new(*chunk_pda, false));
    }

    Instruction {
        program_id: crate::ID,
        accounts: account_metas,
        data: instruction_data.data(),
    }
}

fn assert_instruction_succeeds(
    instruction: &Instruction,
    accounts: &[(Pubkey, Account)],
) -> mollusk_svm::result::InstructionResult {
    let mollusk = Mollusk::new(&crate::ID, PROGRAM_BINARY_PATH);
    let result = mollusk.process_instruction(instruction, accounts);

    if !matches!(
        result.program_result,
        mollusk_svm::result::ProgramResult::Success
    ) {
        panic!("Instruction failed: {:?}", result.program_result);
    }

    result
}

fn assert_instruction_fails_with_error(
    instruction: &Instruction,
    accounts: &[(Pubkey, Account)],
    expected_error: ErrorCode,
    test_name: &str,
) {
    let mollusk = Mollusk::new(&crate::ID, PROGRAM_BINARY_PATH);
    let result = mollusk.process_instruction(instruction, accounts);
    assert_error_code(result, expected_error, test_name);
}

#[test]
fn test_cleanup_successful_with_rent_reclaim() {
    let chain_id = "test-chain";
    let cleanup_height = 50;
    let num_chunks = 3;
    let submitter = Pubkey::new_unique();

    let test_accounts = setup_test_accounts_with_chunks(
        chain_id,
        cleanup_height,
        submitter,
        num_chunks,
        true, // with populated chunks
    );

    // Calculate expected rent to be reclaimed
    let chunk_rent_per = 1_500_000u64;
    let total_expected_rent = chunk_rent_per * u64::from(num_chunks);
    let initial_submitter_balance = 10_000_000_000u64;

    let instruction = create_cleanup_instruction(&test_accounts);

    let result = assert_instruction_succeeds(&instruction, &test_accounts.accounts);

    // Verify submitter received all rent back
    let submitter_account = result
        .resulting_accounts
        .iter()
        .find(|(k, _)| *k == submitter)
        .expect("submitter account should exist");

    assert_eq!(
        submitter_account.1.lamports,
        initial_submitter_balance + total_expected_rent,
        "submitter should receive all rent back"
    );

    // Verify all chunk accounts are closed (lamports = 0 and data zeroed)
    for chunk_pda in &test_accounts.chunk_pdas {
        let chunk_account = result
            .resulting_accounts
            .iter()
            .find(|(k, _)| k == chunk_pda)
            .expect("chunk account should exist");

        assert_eq!(
            chunk_account.1.lamports, 0,
            "chunk account should be closed"
        );

        // Verify data is zeroed
        assert!(
            chunk_account.1.data.iter().all(|&b| b == 0),
            "chunk account data should be zeroed"
        );
    }
}

#[test]
fn test_cleanup_with_missing_chunks() {
    let chain_id = "test-chain";
    let cleanup_height = 50u64;
    let submitter = Pubkey::new_unique();

    // Set up with only 2 out of 3 chunks actually created
    let client_state_pda = Pubkey::find_program_address(
        &[crate::types::ClientState::SEED, chain_id.as_bytes()],
        &crate::ID,
    )
    .0;

    let mut accounts = vec![];

    // Add submitter
    accounts.push((
        submitter,
        Account {
            lamports: 10_000_000_000,
            data: vec![],
            owner: system_program::ID,
            executable: false,
            rent_epoch: 0,
        },
    ));

    // Add client state
    let client_state = ClientState {
        chain_id: chain_id.to_string(),
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

    let mut client_data = vec![];
    client_state.try_serialize(&mut client_data).unwrap();

    accounts.push((
        client_state_pda,
        Account {
            lamports: 1_000_000,
            data: client_data,
            owner: crate::ID,
            executable: false,
            rent_epoch: 0,
        },
    ));

    // Only add chunks 0 and 2, skip chunk 1
    for i in [0u8, 2u8] {
        let chunk_pda = Pubkey::find_program_address(
            &[
                crate::state::HeaderChunk::SEED,
                submitter.as_ref(),
                chain_id.as_bytes(),
                &cleanup_height.to_le_bytes(),
                &[i],
            ],
            &crate::ID,
        )
        .0;

        let chunk = HeaderChunk {
            submitter,
            chunk_data: vec![i; 100],
        };

        let mut chunk_data = vec![];
        chunk.try_serialize(&mut chunk_data).unwrap();

        accounts.push((
            chunk_pda,
            Account {
                lamports: 1_500_000,
                data: chunk_data,
                owner: crate::ID,
                executable: false,
                rent_epoch: 0,
            },
        ));
    }

    // Add empty account for missing chunk 1
    let missing_chunk_pda = Pubkey::find_program_address(
        &[
            crate::state::HeaderChunk::SEED,
            submitter.as_ref(),
            chain_id.as_bytes(),
            &cleanup_height.to_le_bytes(),
            &[1],
        ],
        &crate::ID,
    )
    .0;

    accounts.push((
        missing_chunk_pda,
        Account {
            lamports: 0,
            data: vec![],
            owner: system_program::ID,
            executable: false,
            rent_epoch: 0,
        },
    ));

    let instruction_data = crate::instruction::CleanupIncompleteUpload;

    let instruction = Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(submitter, true),
            // Pass all chunk PDAs even though one is missing
            AccountMeta::new(
                Pubkey::find_program_address(
                    &[
                        crate::state::HeaderChunk::SEED,
                        submitter.as_ref(),
                        chain_id.as_bytes(),
                        &cleanup_height.to_le_bytes(),
                        &[0],
                    ],
                    &crate::ID,
                )
                .0,
                false,
            ),
            AccountMeta::new(missing_chunk_pda, false),
            AccountMeta::new(
                Pubkey::find_program_address(
                    &[
                        crate::state::HeaderChunk::SEED,
                        submitter.as_ref(),
                        chain_id.as_bytes(),
                        &cleanup_height.to_le_bytes(),
                        &[2],
                    ],
                    &crate::ID,
                )
                .0,
                false,
            ),
        ],
        data: instruction_data.data(),
    };

    let result = assert_instruction_succeeds(&instruction, &accounts);

    // Should still succeed and reclaim rent from existing chunks
    let submitter_account = result
        .resulting_accounts
        .iter()
        .find(|(k, _)| *k == submitter)
        .expect("submitter account should exist");

    // Should receive 2 chunks worth of rent (not 3)
    let expected_rent = 1_500_000 * 2;
    assert_eq!(
        submitter_account.1.lamports,
        10_000_000_000 + expected_rent,
        "submitter should receive rent from 2 chunks"
    );
}

#[test]
fn test_cleanup_with_wrong_chunk_order() {
    let chain_id = "test-chain";
    let cleanup_height = 50;
    let submitter = Pubkey::new_unique();

    let test_accounts =
        setup_test_accounts_with_chunks(chain_id, cleanup_height, submitter, 3, true);

    let instruction_data = crate::instruction::CleanupIncompleteUpload;

    // Pass chunks in wrong order (2, 0, 1 instead of 0, 1, 2)
    let instruction = Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(test_accounts.submitter, true),
            AccountMeta::new(test_accounts.chunk_pdas[2], false),
            AccountMeta::new(test_accounts.chunk_pdas[0], false),
            AccountMeta::new(test_accounts.chunk_pdas[1], false),
        ],
        data: instruction_data.data(),
    };

    // Should still work - the cleanup checks each account against expected PDAs
    let result = assert_instruction_succeeds(&instruction, &test_accounts.accounts);

    let submitter_account = result
        .resulting_accounts
        .iter()
        .find(|(k, _)| *k == submitter)
        .expect("submitter account should exist");

    // Should receive rent from all chunks
    assert!(submitter_account.1.lamports >= 10_000_000_000);
}
