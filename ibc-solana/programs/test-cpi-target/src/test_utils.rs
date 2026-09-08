// SPDX-License-Identifier: Apache-2.0

use access_manager::{state::AccessManager, RoleData};
use anchor_lang::{AnchorSerialize, Discriminator, InstructionData, Space, ToAccountMetas};
use solana_program_test::{BanksClient, BanksClientError, ProgramTest};
use solana_sdk::{
    account::Account,
    hash::Hash,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer,
    transaction::Transaction,
};

const PROGRAM_BINARY_PATH: &str = "../../target/deploy/test_cpi_target";

pub const PROGRAM_A_ID: Pubkey = test_cpi_proxy::ID;

pub const ANCHOR_ERROR_OFFSET: u32 = 6000;

pub fn setup_program_test() -> ProgramTest {
    if std::env::var("SBF_OUT_DIR").is_err() {
        let deploy_dir = std::path::Path::new(PROGRAM_BINARY_PATH)
            .parent()
            .expect("Invalid program path");
        std::env::set_var("SBF_OUT_DIR", deploy_dir);
    }

    let mut pt = ProgramTest::new("test_cpi_target", crate::ID, None);
    pt.add_program("test_cpi_proxy", PROGRAM_A_ID, None);
    pt
}

pub struct TestContext {
    pub banks_client: BanksClient,
    pub payer: Keypair,
    pub recent_blockhash: Hash,
}

fn wrap_in_test_cpi_proxy(payer: Pubkey, inner_ix: &Instruction) -> Instruction {
    let account_metas: Vec<test_cpi_proxy::CpiAccountMeta> = inner_ix
        .accounts
        .iter()
        .map(|m| test_cpi_proxy::CpiAccountMeta {
            is_signer: m.is_signer,
            is_writable: m.is_writable,
        })
        .collect();

    let ix_data = test_cpi_proxy::instruction::ProxyCpi {
        instruction_data: inner_ix.data.clone(),
        account_metas,
    };

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
        program_id: PROGRAM_A_ID,
        accounts,
        data: ix_data.data(),
    }
}

fn wrap_in_test_cpi_target_proxy(payer: Pubkey, inner_ix: &Instruction) -> Instruction {
    let account_metas: Vec<crate::CpiAccountMeta> = inner_ix
        .accounts
        .iter()
        .map(|m| crate::CpiAccountMeta {
            is_signer: m.is_signer,
            is_writable: m.is_writable,
        })
        .collect();

    let ix_data = crate::instruction::ProxyCpi {
        instruction_data: inner_ix.data.clone(),
        account_metas,
    };

    let accounts_struct = crate::accounts::ProxyCpi {
        target_program: inner_ix.program_id,
        payer,
    };
    let mut accounts = accounts_struct.to_account_metas(None);
    for meta in &inner_ix.accounts {
        accounts.push(if meta.is_writable {
            AccountMeta::new(meta.pubkey, false)
        } else {
            AccountMeta::new_readonly(meta.pubkey, false)
        });
    }

    Instruction {
        program_id: crate::ID,
        accounts,
        data: ix_data.data(),
    }
}

/// Wraps `inner_ix` in a single CPI hop: Tx -> A -> B
pub fn build_single_cpi_ix(payer: Pubkey, inner_ix: &Instruction) -> Instruction {
    wrap_in_test_cpi_proxy(payer, inner_ix)
}

/// Wraps `inner_ix` in a self-recursive CPI: Tx -> B(`proxy_cpi`) -> B(`check_*`)
pub fn build_self_cpi_ix(payer: Pubkey, inner_ix: &Instruction) -> Instruction {
    wrap_in_test_cpi_target_proxy(payer, inner_ix)
}

/// Wraps `inner_ix` in a nested CPI chain: Tx -> A -> B(`proxy_cpi`) -> B(`check_*`)
pub fn build_nested_cpi_ix(payer: Pubkey, inner_ix: &Instruction) -> Instruction {
    let proxy_wrapping_inner = wrap_in_test_cpi_target_proxy(payer, inner_ix);
    wrap_in_test_cpi_proxy(payer, &proxy_wrapping_inner)
}

pub async fn process_tx(
    banks_client: &BanksClient,
    payer: &Keypair,
    recent_blockhash: Hash,
    instructions: &[Instruction],
) -> Result<(), BanksClientError> {
    let tx = Transaction::new_signed_with_payer(
        instructions,
        Some(&payer.pubkey()),
        &[payer],
        recent_blockhash,
    );
    banks_client.process_transaction(tx).await
}

pub async fn process_tx_with_signers(
    banks_client: &BanksClient,
    payer: &Keypair,
    extra_signers: &[&Keypair],
    recent_blockhash: Hash,
    instructions: &[Instruction],
) -> Result<(), BanksClientError> {
    let mut signers: Vec<&Keypair> = vec![payer];
    signers.extend_from_slice(extra_signers);
    let tx = Transaction::new_signed_with_payer(
        instructions,
        Some(&payer.pubkey()),
        &signers,
        recent_blockhash,
    );
    banks_client.process_transaction(tx).await
}

pub fn extract_custom_error(err: &BanksClientError) -> Option<u32> {
    match err {
        BanksClientError::TransactionError(
            solana_sdk::transaction::TransactionError::InstructionError(
                _,
                solana_sdk::instruction::InstructionError::Custom(code),
            ),
        ) => Some(*code),
        _ => None,
    }
}

// ── AccessManager account helpers ──

pub fn admin_roles(admin: Pubkey) -> Vec<RoleData> {
    vec![RoleData {
        role_id: solana_ibc_types::roles::ADMIN_ROLE,
        members: vec![admin],
    }]
}

pub fn relayer_roles(relayer: Pubkey) -> Vec<RoleData> {
    vec![RoleData {
        role_id: solana_ibc_types::roles::RELAYER_ROLE,
        members: vec![relayer],
    }]
}

/// Adds a pre-populated `AccessManager` account to the `ProgramTest`.
/// Returns the account's pubkey for use in instructions.
pub fn add_access_manager_account(
    pt: &mut ProgramTest,
    roles: Vec<RoleData>,
    whitelisted_programs: Vec<Pubkey>,
) -> Pubkey {
    let pubkey = Pubkey::new_unique();

    let am = AccessManager {
        roles,
        whitelisted_programs,
        pending_authority_transfers: vec![],
    };

    let mut data = vec![0u8; 8 + AccessManager::INIT_SPACE];
    data[0..8].copy_from_slice(AccessManager::DISCRIMINATOR);
    am.serialize(&mut &mut data[8..]).unwrap();

    pt.add_account(
        pubkey,
        Account {
            lamports: 1_000_000,
            data,
            owner: access_manager::ID,
            executable: false,
            rent_epoch: 0,
        },
    );

    pubkey
}
