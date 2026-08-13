// SPDX-License-Identifier: Apache-2.0

use crate::error::ErrorCode;
use crate::state::{HeaderChunk, CHUNK_DATA_SIZE};
use crate::test_helpers::access_control::create_access_manager_account;
use crate::test_helpers::{create_instructions_sysvar_account, PROGRAM_BINARY_PATH};
use crate::types::{AppState, ClientState, IbcHeight, UploadChunkParams};
use access_manager::AccessManagerState;
use anchor_lang::solana_program::{
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
};
use anchor_lang::{AccountDeserialize, AccountSerialize, InstructionData};
use mollusk_svm::{program::keyed_account_for_system_program, Mollusk};
use solana_sdk::account::Account;

struct TestAccounts {
    submitter: Pubkey,
    chunk_pda: Pubkey,
    client_state_pda: Pubkey,
    app_state_pda: Pubkey,
    access_manager_pda: Pubkey,
    accounts: Vec<(Pubkey, Account)>,
}

fn create_app_state_account() -> (Pubkey, Account) {
    let (app_state_pda, _) = Pubkey::find_program_address(&[AppState::SEED], &crate::ID);

    let app_state = AppState {
        am_state: AccessManagerState::new(access_manager::ID),
        _reserved: [0; 256],
    };

    let mut data = vec![];
    app_state.try_serialize(&mut data).unwrap();

    (
        app_state_pda,
        Account {
            lamports: 1_000_000,
            data,
            owner: crate::ID,
            executable: false,
            rent_epoch: 0,
        },
    )
}

fn setup_test_accounts(
    target_height: u64,
    chunk_index: u8,
    submitter: Pubkey,
    with_existing_client: bool,
) -> TestAccounts {
    let chunk_pda = Pubkey::find_program_address(
        &[
            crate::state::HeaderChunk::SEED,
            submitter.as_ref(),
            &target_height.to_le_bytes(),
            &[chunk_index],
        ],
        &crate::ID,
    )
    .0;

    let client_state_pda =
        Pubkey::find_program_address(&[crate::types::ClientState::SEED], &crate::ID).0;

    let (app_state_pda, app_state_account) = create_app_state_account();
    let (access_manager_pda, access_manager_account) =
        create_access_manager_account(submitter, vec![submitter]);
    let instructions_sysvar_account = create_instructions_sysvar_account();

    let mut accounts = vec![
        (
            chunk_pda,
            Account {
                lamports: 0,
                data: vec![],
                owner: system_program::ID,
                executable: false,
                rent_epoch: 0,
            },
        ),
        (
            submitter,
            Account {
                lamports: 10_000_000_000,
                data: vec![],
                owner: system_program::ID,
                executable: false,
                rent_epoch: 0,
            },
        ),
        keyed_account_for_system_program(),
    ];

    if with_existing_client {
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

        let mut client_data = vec![];
        client_state
            .try_serialize(&mut client_data)
            .expect("Failed to serialize client state");

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
    } else {
        accounts.push((
            client_state_pda,
            Account {
                lamports: 0,
                data: vec![],
                owner: system_program::ID,
                executable: false,
                rent_epoch: 0,
            },
        ));
    }

    accounts.push((app_state_pda, app_state_account));
    accounts.push((access_manager_pda, access_manager_account));
    accounts.push((
        anchor_lang::solana_program::sysvar::instructions::ID,
        instructions_sysvar_account,
    ));

    TestAccounts {
        submitter,
        chunk_pda,
        client_state_pda,
        app_state_pda,
        access_manager_pda,
        accounts,
    }
}

fn create_upload_chunk_params(
    target_height: u64,
    chunk_index: u8,
    chunk_data: Vec<u8>,
) -> UploadChunkParams {
    UploadChunkParams {
        target_height,
        chunk_index,
        chunk_data,
    }
}

fn create_upload_instruction(
    test_accounts: &TestAccounts,
    params: UploadChunkParams,
) -> Instruction {
    let instruction_data = crate::instruction::UploadHeaderChunk { params };

    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(test_accounts.chunk_pda, false),
            AccountMeta::new_readonly(test_accounts.client_state_pda, false),
            AccountMeta::new_readonly(test_accounts.app_state_pda, false),
            AccountMeta::new_readonly(test_accounts.access_manager_pda, false),
            AccountMeta::new(test_accounts.submitter, true),
            AccountMeta::new_readonly(anchor_lang::solana_program::sysvar::instructions::ID, false),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
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
) {
    let mollusk = Mollusk::new(&crate::ID, PROGRAM_BINARY_PATH);
    let result = mollusk.process_instruction(instruction, accounts);

    match result.program_result {
        mollusk_svm::result::ProgramResult::Success => {
            panic!("Expected instruction to fail with {expected_error:?}, but it succeeded");
        }
        mollusk_svm::result::ProgramResult::Failure(error) => {
            assert_eq!(
                error,
                anchor_lang::error::Error::from(expected_error).into()
            );
        }
        mollusk_svm::result::ProgramResult::UnknownError(error) => {
            panic!("Unknown error occurred: {error:?}");
        }
    }
}

#[test]
fn test_upload_first_chunk_success() {
    let target_height = 200;
    let chunk_index = 0;
    let submitter = Pubkey::new_unique();

    let test_accounts = setup_test_accounts(
        target_height,
        chunk_index,
        submitter,
        true, // with existing client
    );

    let chunk_data = vec![1u8; 100];
    let params = create_upload_chunk_params(target_height, chunk_index, chunk_data);

    let expected_data = params.chunk_data.clone();

    let instruction = create_upload_instruction(&test_accounts, params);
    let result = assert_instruction_succeeds(&instruction, &test_accounts.accounts);

    // Verify chunk account was created and populated
    let chunk_account = result
        .resulting_accounts
        .iter()
        .find(|(k, _)| *k == test_accounts.chunk_pda)
        .expect("chunk account should exist");

    assert!(chunk_account.1.lamports > 0, "chunk should be rent-exempt");
    assert_eq!(
        chunk_account.1.owner,
        crate::ID,
        "chunk should be owned by program"
    );

    // Deserialize and verify chunk data
    let chunk: HeaderChunk = HeaderChunk::try_deserialize(&mut &chunk_account.1.data[..])
        .expect("should deserialize chunk");

    assert_eq!(chunk.chunk_data, expected_data);
}

#[test]
fn test_reupload_chunk_overwrites_data() {
    let target_height = 200;
    let chunk_index = 0;
    let submitter = Pubkey::new_unique();

    let mut test_accounts = setup_test_accounts(target_height, chunk_index, submitter, true);

    // First upload
    let params1 = create_upload_chunk_params(target_height, chunk_index, vec![1u8; 100]);
    let instruction1 = create_upload_instruction(&test_accounts, params1);
    let result1 = assert_instruction_succeeds(&instruction1, &test_accounts.accounts);
    test_accounts.accounts = result1.resulting_accounts.into_iter().collect();

    // Second upload with different data should succeed and overwrite
    let new_data = vec![2u8; 80];
    let params2 = create_upload_chunk_params(target_height, chunk_index, new_data.clone());
    let instruction2 = create_upload_instruction(&test_accounts, params2);
    let result2 = assert_instruction_succeeds(&instruction2, &test_accounts.accounts);

    let chunk_account = result2
        .resulting_accounts
        .iter()
        .find(|(k, _)| *k == test_accounts.chunk_pda)
        .expect("chunk account should exist");

    let chunk: HeaderChunk = HeaderChunk::try_deserialize(&mut &chunk_account.1.data[..])
        .expect("should deserialize chunk");

    assert_eq!(chunk.chunk_data, new_data);
}

#[test]
fn test_upload_multiple_chunks_independently() {
    let target_height = 200;
    let submitter = Pubkey::new_unique();

    // Upload chunk 0
    let test_accounts0 = setup_test_accounts(target_height, 0, submitter, true);
    let params0 = create_upload_chunk_params(target_height, 0, vec![1u8; 100]);
    let instruction0 = create_upload_instruction(&test_accounts0, params0);
    let result0 = assert_instruction_succeeds(&instruction0, &test_accounts0.accounts);

    // Verify chunk 0 was created
    let chunk_account0 = result0
        .resulting_accounts
        .iter()
        .find(|(k, _)| *k == test_accounts0.chunk_pda)
        .expect("chunk 0 should exist");
    assert!(
        chunk_account0.1.lamports > 0,
        "chunk 0 should be rent-exempt"
    );

    // Upload chunk 1 independently
    let test_accounts1 = setup_test_accounts(target_height, 1, submitter, true);
    let params1 = create_upload_chunk_params(target_height, 1, vec![2u8; 100]);
    let instruction1 = create_upload_instruction(&test_accounts1, params1);
    let result1 = assert_instruction_succeeds(&instruction1, &test_accounts1.accounts);

    // Verify chunk 1 was created
    let chunk_account1 = result1
        .resulting_accounts
        .iter()
        .find(|(k, _)| *k == test_accounts1.chunk_pda)
        .expect("chunk 1 should exist");
    assert!(
        chunk_account1.1.lamports > 0,
        "chunk 1 should be rent-exempt"
    );
}

#[test]
fn test_upload_chunk_exceeding_max_size_fails() {
    let target_height = 200;
    let chunk_index = 0;
    let submitter = Pubkey::new_unique();

    let test_accounts = setup_test_accounts(target_height, chunk_index, submitter, true);

    // Create chunk data that exceeds max size
    let oversized_data = vec![1u8; CHUNK_DATA_SIZE + 1];

    let params = create_upload_chunk_params(target_height, chunk_index, oversized_data);

    let instruction = create_upload_instruction(&test_accounts, params);

    assert_instruction_fails_with_error(
        &instruction,
        &test_accounts.accounts,
        ErrorCode::ChunkDataTooLarge,
    );
}

#[test]
fn test_upload_chunk_with_frozen_client_fails() {
    let chain_id = "test-chain";
    let target_height = 200;
    let chunk_index = 0;
    let submitter = Pubkey::new_unique();

    let mut test_accounts = setup_test_accounts(target_height, chunk_index, submitter, true);

    // Freeze the client by setting frozen_height
    let client_state_pda =
        Pubkey::find_program_address(&[crate::types::ClientState::SEED], &crate::ID).0;

    if let Some((_, account)) = test_accounts
        .accounts
        .iter_mut()
        .find(|(key, _)| *key == client_state_pda)
    {
        let frozen_client_state = ClientState {
            chain_id: chain_id.to_string(),
            trust_level_numerator: 2,
            trust_level_denominator: 3,
            trusting_period: 86400,
            unbonding_period: 172_800,
            max_clock_drift: 600,
            frozen_height: IbcHeight {
                revision_number: 0,
                revision_height: 100, // Frozen at height 100
            },
            latest_height: IbcHeight {
                revision_number: 0,
                revision_height: 150,
            },
        };

        let mut data = vec![];
        frozen_client_state.try_serialize(&mut data).unwrap();
        account.data = data;
    }

    let chunk_data = vec![1u8; 100];
    let params = create_upload_chunk_params(target_height, chunk_index, chunk_data);
    let instruction = create_upload_instruction(&test_accounts, params);

    assert_instruction_fails_with_error(
        &instruction,
        &test_accounts.accounts,
        ErrorCode::ClientFrozen,
    );
}

#[test]
fn test_upload_without_relayer_role_rejected() {
    let target_height = 200;
    let chunk_index = 0;
    let submitter = Pubkey::new_unique();

    let mut test_accounts = setup_test_accounts(target_height, chunk_index, submitter, true);

    // Replace access manager with one that has no relayers
    let (access_manager_pda, access_manager_account) =
        create_access_manager_account(submitter, vec![]);
    if let Some((_, account)) = test_accounts
        .accounts
        .iter_mut()
        .find(|(key, _)| *key == access_manager_pda)
    {
        *account = access_manager_account;
    }

    let params = create_upload_chunk_params(target_height, chunk_index, vec![1u8; 100]);
    let instruction = create_upload_instruction(&test_accounts, params);

    let mollusk = Mollusk::new(&crate::ID, PROGRAM_BINARY_PATH);
    let result = mollusk.process_instruction(&instruction, &test_accounts.accounts);

    assert!(
        !matches!(
            result.program_result,
            mollusk_svm::result::ProgramResult::Success
        ),
        "should reject submitter without RELAYER_ROLE"
    );
}
