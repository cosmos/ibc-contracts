// SPDX-License-Identifier: Apache-2.0

use anchor_lang::{AnchorSerialize, Discriminator, InstructionData, Space};
use mollusk_svm::result::Check;
use mollusk_svm::Mollusk;
use solana_sdk::{
    account::Account,
    instruction::{AccountMeta, Instruction},
    native_loader,
    program_error::ProgramError,
    pubkey::Pubkey,
    system_program,
};
use test_gmp_app::{state::*, ID};

const fn get_test_gmp_app_program_path() -> &'static str {
    "../../target/deploy/test_gmp_app"
}

// Helper functions for account preparation
fn create_counter_app_state_account(
    pubkey: Pubkey,
    authority: Pubkey,
    total_counters: u64,
    total_gmp_calls: u64,
    bump: u8,
) -> (Pubkey, Account) {
    let app_state = CounterAppState {
        authority,
        total_counters,
        total_gmp_calls,
        bump,
    };

    let mut data = Vec::new();
    data.extend_from_slice(CounterAppState::DISCRIMINATOR);
    app_state.serialize(&mut data).unwrap();

    (
        pubkey,
        Account {
            lamports: 1_000_000, // Standard lamports for initialized accounts
            data,
            owner: ID,
            executable: false,
            rent_epoch: 0,
        },
    )
}

fn create_user_counter_account(
    pubkey: Pubkey,
    user: Pubkey,
    count: u64,
    increments: u64,
    decrements: u64,
    last_updated: i64,
    bump: u8,
) -> (Pubkey, Account) {
    let user_counter = UserCounter {
        user,
        count,
        increments,
        decrements,
        last_updated,
        bump,
    };

    let mut data = Vec::new();
    data.extend_from_slice(UserCounter::DISCRIMINATOR);
    user_counter.serialize(&mut data).unwrap();

    (
        pubkey,
        Account {
            lamports: 1_000_000,
            data,
            owner: ID,
            executable: false,
            rent_epoch: 0,
        },
    )
}

fn create_uninitialized_account(pubkey: Pubkey) -> (Pubkey, Account) {
    (
        pubkey,
        Account {
            lamports: {
                use solana_sdk::rent::Rent;
                let account_size = 8 + UserCounter::INIT_SPACE;
                Rent::default().minimum_balance(account_size)
            },
            data: vec![],
            owner: system_program::ID,
            executable: false,
            rent_epoch: 0,
        },
    )
}

const fn create_pda_for_init(pubkey: Pubkey) -> (Pubkey, Account) {
    (
        pubkey,
        Account {
            lamports: 0,
            data: vec![],
            owner: system_program::ID,
            executable: false,
            rent_epoch: 0,
        },
    )
}

const fn create_payer_account(pubkey: Pubkey) -> (Pubkey, Account) {
    (
        pubkey,
        Account {
            lamports: 1_000_000_000,
            data: vec![],
            owner: system_program::ID,
            executable: false,
            rent_epoch: 0,
        },
    )
}

const fn create_system_program_account() -> (Pubkey, Account) {
    (
        system_program::ID,
        Account {
            lamports: 0,
            data: vec![],
            owner: native_loader::ID,
            executable: true,
            rent_epoch: 0,
        },
    )
}

#[test]
fn test_initialize_success() {
    let mollusk = Mollusk::new(&ID, get_test_gmp_app_program_path());

    let authority = Pubkey::new_unique();
    let (app_state_pda, _bump) = Pubkey::find_program_address(&[CounterAppState::SEED], &ID);
    let payer = Pubkey::new_unique();

    let instruction_data = test_gmp_app::instruction::Initialize { authority };

    let instruction = Instruction {
        program_id: ID,
        accounts: vec![
            AccountMeta::new(app_state_pda, false),
            AccountMeta::new(payer, true),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: instruction_data.data(),
    };

    let accounts = vec![
        create_pda_for_init(app_state_pda),
        create_payer_account(payer),
        create_system_program_account(),
    ];

    let result = mollusk.process_instruction(&instruction, &accounts);
    assert!(!result.program_result.is_err());
}

#[test]
fn test_increment_counter_new_user() {
    let mollusk = Mollusk::new(&ID, get_test_gmp_app_program_path());

    let authority = Pubkey::new_unique();
    let user_authority = Pubkey::new_unique(); // The ICS27 gmp_account PDA would be here
    let (app_state_pda, app_state_bump) =
        Pubkey::find_program_address(&[CounterAppState::SEED], &ID);
    let (user_counter_pda, _) =
        Pubkey::find_program_address(&[UserCounter::SEED, user_authority.as_ref()], &ID);
    let payer = Pubkey::new_unique();

    let instruction_data = test_gmp_app::instruction::Increment { amount: 5 };

    let instruction = Instruction {
        program_id: ID,
        accounts: vec![
            AccountMeta::new(app_state_pda, false),
            AccountMeta::new(user_counter_pda, false),
            AccountMeta::new_readonly(user_authority, true), // user_authority must be signer
            AccountMeta::new(payer, true),
            AccountMeta::new_readonly(system_program::ID, false),
        ],
        data: instruction_data.data(),
    };

    let accounts = vec![
        create_counter_app_state_account(app_state_pda, authority, 0, 0, app_state_bump),
        create_uninitialized_account(user_counter_pda),
        create_payer_account(user_authority), // user_authority needs lamports to sign
        create_payer_account(payer),
        create_system_program_account(),
    ];

    let result = mollusk.process_instruction(&instruction, &accounts);
    assert!(!result.program_result.is_err());

    // Verify return data contains the new counter value (5)
    assert!(!result.return_data.is_empty());
    assert_eq!(result.return_data.len(), 8);
    let counter_value = u64::from_le_bytes(result.return_data.try_into().unwrap());
    assert_eq!(counter_value, 5);
}

#[test]
fn test_decrement_counter_existing_user() {
    let mollusk = Mollusk::new(&ID, get_test_gmp_app_program_path());

    let authority = Pubkey::new_unique();
    let user = Pubkey::new_unique();
    let (app_state_pda, app_state_bump) =
        Pubkey::find_program_address(&[CounterAppState::SEED], &ID);
    let (user_counter_pda, user_counter_bump) =
        Pubkey::find_program_address(&[UserCounter::SEED, user.as_ref()], &ID);

    let instruction_data = test_gmp_app::instruction::Decrement { user, amount: 3 };

    let instruction = Instruction {
        program_id: ID,
        accounts: vec![
            AccountMeta::new(app_state_pda, false),
            AccountMeta::new(user_counter_pda, false),
        ],
        data: instruction_data.data(),
    };

    let accounts = vec![
        create_counter_app_state_account(app_state_pda, authority, 1, 0, app_state_bump),
        create_user_counter_account(
            user_counter_pda,
            user,
            10,
            5,
            2,
            1_600_000_000,
            user_counter_bump,
        ),
    ];

    let result = mollusk.process_instruction(&instruction, &accounts);
    assert!(!result.program_result.is_err());

    // Verify return data contains the new counter value (7)
    assert!(!result.return_data.is_empty());
    let counter_value = u64::from_le_bytes(result.return_data.try_into().unwrap());
    assert_eq!(counter_value, 7);
}

#[test]
fn test_counter_underflow_fails() {
    let mollusk = Mollusk::new(&ID, get_test_gmp_app_program_path());

    let authority = Pubkey::new_unique();
    let user = Pubkey::new_unique();
    let (app_state_pda, app_state_bump) =
        Pubkey::find_program_address(&[CounterAppState::SEED], &ID);
    let (user_counter_pda, user_counter_bump) =
        Pubkey::find_program_address(&[UserCounter::SEED, user.as_ref()], &ID);

    let instruction_data = test_gmp_app::instruction::Decrement { user, amount: 10 };

    let instruction = Instruction {
        program_id: ID,
        accounts: vec![
            AccountMeta::new(app_state_pda, false),
            AccountMeta::new(user_counter_pda, false),
        ],
        data: instruction_data.data(),
    };

    let accounts = vec![
        create_counter_app_state_account(app_state_pda, authority, 1, 0, app_state_bump),
        create_user_counter_account(
            user_counter_pda,
            user,
            5,
            2,
            1,
            1_600_000_000,
            user_counter_bump,
        ),
    ];

    let checks = vec![Check::err(ProgramError::Custom(
        anchor_lang::error::ERROR_CODE_OFFSET
            + test_gmp_app::errors::CounterError::CounterUnderflow as u32,
    ))];
    mollusk.process_and_validate_instruction(&instruction, &accounts, &checks);
}

#[test]
fn test_get_counter() {
    let mollusk = Mollusk::new(&ID, get_test_gmp_app_program_path());

    let user = Pubkey::new_unique();
    let (user_counter_pda, user_counter_bump) =
        Pubkey::find_program_address(&[UserCounter::SEED, user.as_ref()], &ID);

    let instruction_data = test_gmp_app::instruction::GetCounter { user };

    let instruction = Instruction {
        program_id: ID,
        accounts: vec![AccountMeta::new_readonly(user_counter_pda, false)],
        data: instruction_data.data(),
    };

    let accounts = vec![create_user_counter_account(
        user_counter_pda,
        user,
        42,
        10,
        3,
        1_600_000_000,
        user_counter_bump,
    )];

    let result = mollusk.process_instruction(&instruction, &accounts);
    assert!(!result.program_result.is_err());

    // Verify return data contains the counter value (42)
    assert!(!result.return_data.is_empty());
    let counter_value = u64::from_le_bytes(result.return_data.try_into().unwrap());
    assert_eq!(counter_value, 42);
}
