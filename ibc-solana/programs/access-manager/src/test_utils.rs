// SPDX-License-Identifier: Apache-2.0

use crate::state::AccessManager;
use crate::types::{PendingAuthorityTransfer, RoleData};
use anchor_lang::prelude::*;
use mollusk_svm::{result::InstructionResult, Mollusk};
use solana_ibc_types::roles;
use solana_sdk::{
    account::Account,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    system_program,
};

pub fn serialize_access_manager(access_manager: &AccessManager) -> Vec<u8> {
    let mut data = AccessManager::DISCRIMINATOR.to_vec();
    access_manager.serialize(&mut data).unwrap();
    data
}

pub fn create_initialized_access_manager(admin: Pubkey) -> (Pubkey, Account) {
    let (access_manager_pda, _) = Pubkey::find_program_address(&[AccessManager::SEED], &crate::ID);

    let access_manager = AccessManager {
        roles: vec![RoleData {
            role_id: roles::ADMIN_ROLE,
            members: vec![admin],
        }],
        whitelisted_programs: vec![],
        pending_authority_transfers: vec![],
    };

    // Use INIT_SPACE to ensure account has enough space for max roles
    let mut data = vec![0u8; 8 + AccessManager::INIT_SPACE];
    data[0..8].copy_from_slice(AccessManager::DISCRIMINATOR);
    access_manager.serialize(&mut &mut data[8..]).unwrap();

    (
        access_manager_pda,
        Account {
            lamports: 1_000_000,
            data,
            owner: crate::ID,
            executable: false,
            rent_epoch: 0,
        },
    )
}

pub fn create_access_manager_with_role(
    admin: Pubkey,
    role_id: u64,
    member: Pubkey,
) -> (Pubkey, Account) {
    let (access_manager_pda, _) = Pubkey::find_program_address(&[AccessManager::SEED], &crate::ID);

    let mut roles_vec = vec![RoleData {
        role_id: roles::ADMIN_ROLE,
        members: vec![admin],
    }];

    // Add the requested role if it's not already ADMIN_ROLE or if member is different
    if role_id != roles::ADMIN_ROLE || member != admin {
        if let Some(existing_role) = roles_vec.iter_mut().find(|r| r.role_id == role_id) {
            if !existing_role.members.contains(&member) {
                existing_role.members.push(member);
            }
        } else {
            roles_vec.push(RoleData {
                role_id,
                members: vec![member],
            });
        }
    }

    let access_manager = AccessManager {
        roles: roles_vec,
        whitelisted_programs: vec![],
        pending_authority_transfers: vec![],
    };

    // Use INIT_SPACE to ensure account has enough space for max roles
    let mut data = vec![0u8; 8 + AccessManager::INIT_SPACE];
    data[0..8].copy_from_slice(AccessManager::DISCRIMINATOR);
    access_manager.serialize(&mut &mut data[8..]).unwrap();

    (
        access_manager_pda,
        Account {
            lamports: 1_000_000,
            data,
            owner: crate::ID,
            executable: false,
            rent_epoch: 0,
        },
    )
}

pub fn create_signer_account() -> Account {
    Account {
        lamports: 1_000_000_000,
        data: vec![],
        owner: system_program::ID,
        executable: false,
        rent_epoch: 0,
    }
}

pub fn setup_mollusk() -> Mollusk {
    Mollusk::new(&crate::ID, crate::get_access_manager_program_path())
}

/// Anchor error code offset
pub const ANCHOR_ERROR_OFFSET: u32 = 6000;

/// Build instruction for access-manager program
pub fn build_instruction<T: anchor_lang::InstructionData>(
    instruction_data: T,
    accounts: Vec<anchor_lang::prelude::AccountMeta>,
) -> Instruction {
    Instruction {
        program_id: crate::ID,
        accounts,
        data: instruction_data.data(),
    }
}

/// Get the `access_manager` PDA
pub fn get_access_manager_pda() -> Pubkey {
    Pubkey::find_program_address(&[AccessManager::SEED], &crate::ID).0
}

/// Deserialize `access_manager` from instruction result
pub fn get_access_manager_from_result(result: &InstructionResult, pda: &Pubkey) -> AccessManager {
    let account = result
        .resulting_accounts
        .iter()
        .find(|(pubkey, _)| pubkey == pda)
        .map(|(_, account)| account)
        .expect("Access manager account not found");

    anchor_lang::AccountDeserialize::try_deserialize(&mut &account.data[..])
        .expect("Failed to deserialize access manager")
}

/// Create instructions sysvar account for direct call (not CPI)
pub fn create_instructions_sysvar_account() -> Account {
    use solana_sdk::sysvar::instructions::{
        construct_instructions_data, BorrowedAccountMeta, BorrowedInstruction,
    };

    // Create minimal mock instruction to simulate direct call
    // Current instruction has this program as the program_id
    let account_pubkey = Pubkey::new_unique();
    let account = BorrowedAccountMeta {
        pubkey: &account_pubkey,
        is_signer: false,
        is_writable: true,
    };
    let mock_instruction = BorrowedInstruction {
        program_id: &crate::ID, // Direct call to our program
        accounts: vec![account],
        data: &[],
    };

    let ixs_data = construct_instructions_data(&[mock_instruction]);

    Account {
        lamports: 1_000_000,
        data: ixs_data,
        owner: solana_sdk::sysvar::ID,
        executable: false,
        rent_epoch: 0,
    }
}

/// Create fake instructions sysvar for testing Wormhole-style attack
/// Returns (`fake_pubkey`, account) where `fake_pubkey` is NOT the real instructions sysvar ID
pub fn create_fake_instructions_sysvar_account(caller_program_id: Pubkey) -> (Pubkey, Account) {
    use solana_sdk::sysvar::instructions::{
        construct_instructions_data, BorrowedAccountMeta, BorrowedInstruction,
    };

    let account_pubkey = Pubkey::new_unique();
    let account = BorrowedAccountMeta {
        pubkey: &account_pubkey,
        is_signer: false,
        is_writable: true,
    };
    let mock_instruction = BorrowedInstruction {
        program_id: &caller_program_id, // Fake caller program
        accounts: vec![account],
        data: &[],
    };

    let ixs_data = construct_instructions_data(&[mock_instruction]);

    // Use a FAKE pubkey (not the real instructions sysvar ID)
    let fake_sysvar_pubkey = Pubkey::new_unique();

    (
        fake_sysvar_pubkey,
        Account {
            lamports: 1_000_000,
            data: ixs_data,
            owner: solana_sdk::sysvar::ID,
            executable: false,
            rent_epoch: 0,
        },
    )
}

/// Helper for testing Wormhole-style fake sysvar attacks
/// Automatically finds and replaces the instructions sysvar with a fake one
/// Returns (`modified_instruction`, `fake_sysvar_account_tuple`)
pub fn setup_fake_sysvar_attack(
    mut instruction: Instruction,
    program_id: Pubkey,
) -> (Instruction, (Pubkey, Account)) {
    let (fake_sysvar_pubkey, fake_sysvar_account) =
        create_fake_instructions_sysvar_account(program_id);

    // Find the instructions sysvar account and replace it with the fake one
    let sysvar_account_index = instruction
        .accounts
        .iter()
        .position(|acc| acc.pubkey == solana_sdk::sysvar::instructions::ID)
        .expect("Instructions sysvar account not found in instruction");

    instruction.accounts[sysvar_account_index] =
        AccountMeta::new_readonly(fake_sysvar_pubkey, false);

    (instruction, (fake_sysvar_pubkey, fake_sysvar_account))
}

/// Expected error for Wormhole-style sysvar attacks (Anchor's address constraint violation)
pub fn expect_sysvar_attack_error() -> mollusk_svm::result::Check<'static> {
    mollusk_svm::result::Check::err(solana_sdk::program_error::ProgramError::Custom(
        anchor_lang::error::ErrorCode::ConstraintAddress as u32,
    ))
}

/// Create instructions sysvar that simulates a CPI call from another program
/// Uses the REAL sysvar address but with a different `program_id` to simulate CPI context
pub fn create_cpi_instructions_sysvar_account(caller_program_id: Pubkey) -> Account {
    use solana_sdk::sysvar::instructions::{
        construct_instructions_data, BorrowedAccountMeta, BorrowedInstruction,
    };

    let account_pubkey = Pubkey::new_unique();
    let account = BorrowedAccountMeta {
        pubkey: &account_pubkey,
        is_signer: false,
        is_writable: true,
    };
    let mock_instruction = BorrowedInstruction {
        program_id: &caller_program_id, // Different program calling via CPI
        accounts: vec![account],
        data: &[],
    };

    let ixs_data = construct_instructions_data(&[mock_instruction]);

    Account {
        lamports: 1_000_000,
        data: ixs_data,
        owner: solana_sdk::sysvar::ID,
        executable: false,
        rent_epoch: 0,
    }
}

/// Helper for testing CPI rejection
/// Replaces the instructions sysvar with one that simulates a CPI call
/// Returns (`modified_instruction`, `cpi_sysvar_account_tuple`)
pub fn setup_cpi_call_test(
    instruction: Instruction,
    caller_program_id: Pubkey,
) -> (Instruction, (Pubkey, Account)) {
    let cpi_sysvar_account = create_cpi_instructions_sysvar_account(caller_program_id);

    // Use the REAL sysvar address (unlike Wormhole attack which uses fake)
    (
        instruction,
        (solana_sdk::sysvar::instructions::ID, cpi_sysvar_account),
    )
}

/// Expected error for CPI rejection (`UnauthorizedCaller` from `reject_cpi`)
/// This is for instructions that DON'T map the error (like router/gmp callback instructions)
pub fn expect_cpi_rejection_error() -> mollusk_svm::result::Check<'static> {
    use solana_ibc_types::CpiValidationError;
    mollusk_svm::result::Check::err(solana_sdk::program_error::ProgramError::Custom(
        anchor_lang::error::ERROR_CODE_OFFSET + CpiValidationError::UnauthorizedCaller as u32,
    ))
}

/// Expected error for CPI rejection in access-manager instructions
/// These instructions map `CpiValidationError::UnauthorizedCaller` to `AccessManagerError::CpiNotAllowed`
pub fn expect_access_manager_cpi_rejection_error() -> mollusk_svm::result::Check<'static> {
    use crate::errors::AccessManagerError;
    mollusk_svm::result::Check::err(solana_sdk::program_error::ProgramError::Custom(
        ANCHOR_ERROR_OFFSET + AccessManagerError::CpiNotAllowed as u32,
    ))
}

/// Create instructions sysvar account for direct call (alias for `create_instructions_sysvar_account`)
pub fn create_instructions_sysvar_account_with_caller(
    caller_program_id: Pubkey,
) -> (Pubkey, Account) {
    use solana_sdk::sysvar::instructions::{
        construct_instructions_data, BorrowedAccountMeta, BorrowedInstruction,
    };

    let account_pubkey = Pubkey::new_unique();
    let account = BorrowedAccountMeta {
        pubkey: &account_pubkey,
        is_signer: false,
        is_writable: true,
    };
    let mock_instruction = BorrowedInstruction {
        program_id: &caller_program_id,
        accounts: vec![account],
        data: &[],
    };

    let ixs_data = construct_instructions_data(&[mock_instruction]);

    (
        solana_sdk::sysvar::instructions::ID,
        Account {
            lamports: 1_000_000,
            data: ixs_data,
            owner: solana_sdk::sysvar::ID,
            executable: false,
            rent_epoch: 0,
        },
    )
}

// ── ProgramTest (BPF runtime) integration test helpers ──

pub const TEST_CPI_PROXY_ID: Pubkey =
    solana_sdk::pubkey!("CtQLLKbDMt1XVNXtLKJEt1K8cstbckjqE6zyFqR37KTc");
pub const TEST_CPI_TARGET_ID: Pubkey =
    solana_sdk::pubkey!("GHB99UGVmKFeNrtSLsuzL2QhZZgaqcASvTjotQd2dZzu");
const DEPLOY_DIR: &str = "../../target/deploy";

pub fn anchor_discriminator(instruction_name: &str) -> [u8; 8] {
    let hash = solana_sdk::hash::hash(format!("global:{instruction_name}").as_bytes());
    let mut disc = [0u8; 8];
    disc.copy_from_slice(&hash.to_bytes()[..8]);
    disc
}

pub fn setup_program_test_with_whitelist(
    admin: &Pubkey,
    whitelisted_programs: &[Pubkey],
) -> solana_program_test::ProgramTest {
    if std::env::var("SBF_OUT_DIR").is_err() {
        let deploy_dir = std::path::Path::new(DEPLOY_DIR);
        std::env::set_var("SBF_OUT_DIR", deploy_dir);
    }

    let mut pt = solana_program_test::ProgramTest::new(
        crate::test_config::PROGRAM_BINARY_NAME,
        crate::ID,
        None,
    );
    pt.add_program("test_cpi_proxy", TEST_CPI_PROXY_ID, None);
    pt.add_program("test_cpi_target", TEST_CPI_TARGET_ID, None);

    // Pre-create AccessManager PDA with admin role and whitelist
    let (access_manager_pda, _) = Pubkey::find_program_address(&[AccessManager::SEED], &crate::ID);

    let am = AccessManager {
        roles: vec![RoleData {
            role_id: roles::ADMIN_ROLE,
            members: vec![*admin],
        }],
        whitelisted_programs: whitelisted_programs.to_vec(),
        pending_authority_transfers: vec![],
    };
    let mut am_data = vec![0u8; 8 + AccessManager::INIT_SPACE];
    am_data[0..8].copy_from_slice(AccessManager::DISCRIMINATOR);
    am.serialize(&mut &mut am_data[8..]).unwrap();

    pt.add_account(
        access_manager_pda,
        Account {
            lamports: 1_000_000,
            data: am_data,
            owner: crate::ID,
            executable: false,
            rent_epoch: 0,
        },
    );

    pt
}

pub fn wrap_in_test_cpi_proxy(payer: Pubkey, inner_ix: &Instruction) -> Instruction {
    let mut data = Vec::new();
    data.extend_from_slice(&anchor_discriminator("proxy_cpi"));

    AnchorSerialize::serialize(&inner_ix.data, &mut data).unwrap();

    let meta_count = inner_ix.accounts.len() as u32;
    AnchorSerialize::serialize(&meta_count, &mut data).unwrap();
    for meta in &inner_ix.accounts {
        AnchorSerialize::serialize(&meta.is_signer, &mut data).unwrap();
        AnchorSerialize::serialize(&meta.is_writable, &mut data).unwrap();
    }

    let mut accounts = vec![
        AccountMeta::new_readonly(inner_ix.program_id, false),
        AccountMeta::new_readonly(payer, true),
    ];
    for meta in &inner_ix.accounts {
        accounts.push(if meta.is_writable {
            AccountMeta::new(meta.pubkey, false)
        } else {
            AccountMeta::new_readonly(meta.pubkey, false)
        });
    }

    Instruction {
        program_id: TEST_CPI_PROXY_ID,
        accounts,
        data,
    }
}

pub fn wrap_in_test_cpi_target_proxy(payer: Pubkey, inner_ix: &Instruction) -> Instruction {
    let mut data = Vec::new();
    data.extend_from_slice(&anchor_discriminator("proxy_cpi"));

    AnchorSerialize::serialize(&inner_ix.data, &mut data).unwrap();

    let meta_count = inner_ix.accounts.len() as u32;
    AnchorSerialize::serialize(&meta_count, &mut data).unwrap();
    for meta in &inner_ix.accounts {
        AnchorSerialize::serialize(&meta.is_signer, &mut data).unwrap();
        AnchorSerialize::serialize(&meta.is_writable, &mut data).unwrap();
    }

    let mut accounts = vec![
        AccountMeta::new_readonly(inner_ix.program_id, false),
        AccountMeta::new_readonly(payer, true),
    ];
    for meta in &inner_ix.accounts {
        accounts.push(if meta.is_writable {
            AccountMeta::new(meta.pubkey, false)
        } else {
            AccountMeta::new_readonly(meta.pubkey, false)
        });
    }

    Instruction {
        program_id: TEST_CPI_TARGET_ID,
        accounts,
        data,
    }
}

pub fn extract_custom_error(err: &solana_program_test::BanksClientError) -> Option<u32> {
    match err {
        solana_program_test::BanksClientError::TransactionError(
            solana_sdk::transaction::TransactionError::InstructionError(
                _,
                solana_sdk::instruction::InstructionError::Custom(code),
            ),
        ) => Some(*code),
        _ => None,
    }
}

/// Deserialize account data into typed struct
pub fn get_account_data<T: anchor_lang::AccountDeserialize>(account: &Account) -> T {
    anchor_lang::AccountDeserialize::try_deserialize(&mut &account.data[..])
        .expect("Failed to deserialize account")
}

/// Serialize account struct to data
pub fn serialize_account<T: anchor_lang::Discriminator + anchor_lang::AnchorSerialize>(
    account: &T,
) -> Vec<u8> {
    let mut data = T::DISCRIMINATOR.to_vec();
    account.serialize(&mut data).unwrap();
    data
}

/// Create rent sysvar account
pub fn create_rent_sysvar_account() -> (Pubkey, Account) {
    (
        solana_sdk::sysvar::rent::ID,
        Account {
            lamports: 1_000_000,
            data: vec![0; 17], // Rent sysvar is 17 bytes
            owner: solana_sdk::sysvar::ID,
            executable: false,
            rent_epoch: 0,
        },
    )
}

/// Create clock sysvar account
pub fn create_clock_sysvar_account() -> (Pubkey, Account) {
    (
        solana_sdk::sysvar::clock::ID,
        Account {
            lamports: 1_000_000,
            data: vec![0; 40], // Clock sysvar is 40 bytes
            owner: solana_sdk::sysvar::ID,
            executable: false,
            rent_epoch: 0,
        },
    )
}

/// Create a BPF Loader Upgradeable `ProgramData` account for testing.
///
/// The account is placed at the correct PDA derived from the program ID
/// and owned by `bpf_loader_upgradeable::ID`.
pub fn create_program_data_account(
    program_id: &Pubkey,
    authority: Option<Pubkey>,
) -> (Pubkey, Account) {
    use solana_sdk::bpf_loader_upgradeable::{self, UpgradeableLoaderState};

    let (program_data_pda, _) =
        Pubkey::find_program_address(&[program_id.as_ref()], &bpf_loader_upgradeable::ID);

    let state = UpgradeableLoaderState::ProgramData {
        slot: 0,
        upgrade_authority_address: authority,
    };
    let data = bincode::serialize(&state).unwrap();

    (
        program_data_pda,
        Account {
            lamports: 1_000_000,
            data,
            owner: bpf_loader_upgradeable::ID,
            executable: false,
            rent_epoch: 0,
        },
    )
}

// ── Mollusk helpers for upgrade authority transfer tests ──

/// Creates an `AccessManager` Mollusk account with pre-populated pending transfers.
pub fn create_access_manager_with_transfers(
    admin: Pubkey,
    transfers: Vec<PendingAuthorityTransfer>,
) -> (Pubkey, Account) {
    let (pda, _) = Pubkey::find_program_address(&[AccessManager::SEED], &crate::ID);
    let am = AccessManager {
        roles: vec![RoleData {
            role_id: roles::ADMIN_ROLE,
            members: vec![admin],
        }],
        whitelisted_programs: vec![],
        pending_authority_transfers: transfers,
    };
    let mut data = vec![0u8; 8 + AccessManager::INIT_SPACE];
    data[0..8].copy_from_slice(AccessManager::DISCRIMINATOR);
    am.serialize(&mut &mut data[8..]).unwrap();
    (
        pda,
        Account {
            lamports: 1_000_000,
            data,
            owner: crate::ID,
            executable: false,
            rent_epoch: 0,
        },
    )
}

// ── ProgramTest helpers for upgrade authority transfer tests ──

/// Adds a target program's `ProgramData` and upgrade authority PDA accounts
/// to a `ProgramTest`, with this access manager's PDA as upgrade authority.
pub fn add_target_program_accounts(
    pt: &mut solana_program_test::ProgramTest,
    target_program: &Pubkey,
) {
    use solana_sdk::bpf_loader_upgradeable::{self, UpgradeableLoaderState};

    let (upgrade_authority_pda, _) =
        AccessManager::upgrade_authority_pda(target_program, &crate::ID);

    let (program_data_pda, _) =
        Pubkey::find_program_address(&[target_program.as_ref()], &bpf_loader_upgradeable::ID);

    let pd_account = Account::new_data_with_space(
        10_000_000_000,
        &UpgradeableLoaderState::ProgramData {
            slot: 0,
            upgrade_authority_address: Some(upgrade_authority_pda),
        },
        UpgradeableLoaderState::size_of_programdata_metadata(),
        &bpf_loader_upgradeable::ID,
    )
    .unwrap();
    pt.add_account(program_data_pda, pd_account);

    pt.add_account(
        upgrade_authority_pda,
        Account {
            lamports: 1_000_000,
            owner: system_program::ID,
            ..Default::default()
        },
    );
}

pub fn build_propose_ix(
    admin: Pubkey,
    target_program: Pubkey,
    new_authority: Pubkey,
) -> Instruction {
    use anchor_lang::InstructionData;

    let (access_manager_pda, _) = Pubkey::find_program_address(&[AccessManager::SEED], &crate::ID);

    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(access_manager_pda, false),
            AccountMeta::new_readonly(admin, true),
            AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
        ],
        data: crate::instruction::ProposeUpgradeAuthorityTransfer {
            target_program,
            new_authority,
        }
        .data(),
    }
}

pub fn build_accept_ix(target_program: Pubkey, new_authority: Pubkey) -> Instruction {
    use anchor_lang::InstructionData;
    use solana_sdk::bpf_loader_upgradeable;

    let (access_manager_pda, _) = Pubkey::find_program_address(&[AccessManager::SEED], &crate::ID);
    let (program_data, _) =
        Pubkey::find_program_address(&[target_program.as_ref()], &bpf_loader_upgradeable::ID);
    let (upgrade_authority_pda, _) =
        AccessManager::upgrade_authority_pda(&target_program, &crate::ID);

    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(access_manager_pda, false),
            AccountMeta::new(program_data, false),
            AccountMeta::new_readonly(upgrade_authority_pda, false),
            AccountMeta::new_readonly(new_authority, true),
            AccountMeta::new_readonly(bpf_loader_upgradeable::ID, false),
        ],
        data: crate::instruction::AcceptUpgradeAuthorityTransfer { target_program }.data(),
    }
}

pub fn build_cancel_ix(admin: Pubkey, target_program: Pubkey) -> Instruction {
    use anchor_lang::InstructionData;

    let (access_manager_pda, _) = Pubkey::find_program_address(&[AccessManager::SEED], &crate::ID);

    Instruction {
        program_id: crate::ID,
        accounts: vec![
            AccountMeta::new(access_manager_pda, false),
            AccountMeta::new_readonly(admin, true),
            AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
        ],
        data: crate::instruction::CancelUpgradeAuthorityTransfer { target_program }.data(),
    }
}

/// Reads the upgrade authority from a target program's `ProgramData` account.
pub async fn get_program_data_authority(
    banks_client: &solana_program_test::BanksClient,
    target_program: Pubkey,
) -> Option<Pubkey> {
    use solana_sdk::bpf_loader_upgradeable::{self, UpgradeableLoaderState};

    let (program_data_pda, _) =
        Pubkey::find_program_address(&[target_program.as_ref()], &bpf_loader_upgradeable::ID);
    let account = banks_client
        .get_account(program_data_pda)
        .await
        .unwrap()
        .unwrap();
    let state: UpgradeableLoaderState = bincode::deserialize(&account.data).unwrap();
    match state {
        UpgradeableLoaderState::ProgramData {
            upgrade_authority_address,
            ..
        } => upgrade_authority_address,
        _ => panic!("unexpected state"),
    }
}

/// Reads pending authority transfers from this crate's `AccessManager` state.
pub async fn get_pending_transfers(
    banks_client: &solana_program_test::BanksClient,
) -> Vec<PendingAuthorityTransfer> {
    let (pda, _) = Pubkey::find_program_address(&[AccessManager::SEED], &crate::ID);
    let account = banks_client.get_account(pda).await.unwrap().unwrap();
    let am: AccessManager =
        anchor_lang::AccountDeserialize::try_deserialize(&mut &account.data[..]).unwrap();
    am.pending_authority_transfers
}
