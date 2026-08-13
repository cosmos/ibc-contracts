// SPDX-License-Identifier: Apache-2.0

use access_manager::AccessManagerState;

pub const PROGRAM_BINARY_PATH: &str = "../../target/deploy/attestation";

pub mod accounts {
    use crate::state::ConsensusStateStore;
    use crate::types::{AppState, ClientState};
    use access_manager::AccessManagerState;
    use anchor_lang::AccountSerialize;
    use solana_sdk::account::Account;
    use solana_sdk::pubkey::Pubkey;
    use solana_sdk::{native_loader, system_program};

    pub fn create_client_state_account(client_state: &ClientState) -> Account {
        let mut data = vec![];
        client_state.try_serialize(&mut data).unwrap();
        Account {
            lamports: 1_000_000,
            data,
            owner: crate::ID,
            executable: false,
            rent_epoch: 0,
        }
    }

    pub fn create_consensus_state_account(height: u64, timestamp: u64) -> Account {
        let consensus_state_store = ConsensusStateStore { height, timestamp };
        let mut data = vec![];
        consensus_state_store.try_serialize(&mut data).unwrap();
        Account {
            lamports: 10_000_000,
            data,
            owner: crate::ID,
            executable: false,
            rent_epoch: u64::MAX, // Required for Mollusk rent exemption checks
        }
    }

    pub fn create_app_state_account(access_manager: Pubkey) -> Account {
        use anchor_lang::Space;

        let app_state = AppState {
            version: crate::types::AccountVersion::V1,
            am_state: AccessManagerState::new(access_manager),
            _reserved: [0; 256],
        };
        let mut data = vec![0u8; 8 + AppState::INIT_SPACE];
        app_state.try_serialize(&mut &mut data[..]).unwrap();
        Account {
            lamports: 1_000_000,
            data,
            owner: crate::ID,
            executable: false,
            rent_epoch: 0,
        }
    }

    pub fn create_payer_account() -> Account {
        Account {
            lamports: 10_000_000_000,
            data: vec![],
            owner: system_program::ID,
            executable: false,
            rent_epoch: 0,
        }
    }

    pub fn create_empty_account() -> Account {
        Account {
            lamports: 0,
            data: vec![],
            owner: system_program::ID,
            executable: false,
            rent_epoch: 0,
        }
    }

    pub fn create_system_program_account() -> Account {
        Account {
            lamports: 0,
            data: vec![],
            owner: native_loader::ID,
            executable: true,
            rent_epoch: 0,
        }
    }

    pub fn create_instructions_sysvar_account() -> (Pubkey, Account) {
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
            program_id: &crate::ID,
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

    /// Create a BPF Loader Upgradeable `ProgramData` account for testing.
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
}

pub mod fixtures {
    use crate::types::{ClientState, PacketAttestation, PacketCompact, StateAttestation};
    use crate::ETH_ADDRESS_LEN;
    use alloy_sol_types::SolValue;

    pub const DEFAULT_TIMESTAMP: u64 = 1_700_000_000;

    pub fn default_client_state(height: u64) -> ClientState {
        ClientState {
            version: crate::types::AccountVersion::V1,
            attestor_addresses: vec![[1u8; 20]],
            min_required_sigs: 1,
            latest_height: height,
            is_frozen: false,
        }
    }

    pub fn create_test_client_state(
        attestor_addresses: Vec<[u8; ETH_ADDRESS_LEN]>,
        min_required_sigs: u8,
        latest_height: u64,
    ) -> ClientState {
        ClientState {
            version: crate::types::AccountVersion::V1,
            attestor_addresses,
            min_required_sigs,
            latest_height,
            is_frozen: false,
        }
    }

    pub fn encode_packet_attestation(height: u64, packets: &[([u8; 32], [u8; 32])]) -> Vec<u8> {
        PacketAttestation {
            height,
            packets: packets
                .iter()
                .map(|(path, commitment)| PacketCompact {
                    path: (*path).into(),
                    commitment: (*commitment).into(),
                })
                .collect(),
        }
        .abi_encode()
    }

    pub fn encode_state_attestation(height: u64, timestamp: u64) -> Vec<u8> {
        StateAttestation { height, timestamp }.abi_encode()
    }

    pub fn create_test_signature() -> Vec<u8> {
        let mut sig = vec![0u8; 65];
        sig[64] = 27; // recovery_id
        sig
    }
}

/// Test signing utilities using alloy.
pub mod signing {
    use crate::crypto::AttestationType;
    use crate::ETH_ADDRESS_LEN;
    use alloy_signer::SignerSync;
    use alloy_signer_local::PrivateKeySigner;
    use sha2::{Digest, Sha256};

    /// Test attestor with deterministic keys for unit and integration testing.
    pub struct TestAttestor {
        signer: PrivateKeySigner,
        pub eth_address: [u8; ETH_ADDRESS_LEN],
    }

    impl TestAttestor {
        /// Create a test attestor from a deterministic seed
        pub fn new(seed: u8) -> Self {
            let mut key_bytes = [0u8; 32];
            key_bytes[0] = seed;
            key_bytes[31] = 1; // Ensure non-zero

            let signer = PrivateKeySigner::from_bytes(&key_bytes.into())
                .expect("Valid key bytes for testing");
            let eth_address = signer.address().0 .0;

            Self {
                signer,
                eth_address,
            }
        }

        /// Sign attestation data with domain separation and return 65-byte signature.
        ///
        /// Computes `sha256(type_tag || sha256(data))` then signs the result.
        pub fn sign(&self, data: &[u8], attestation_type: AttestationType) -> Vec<u8> {
            let inner_hash: [u8; 32] = Sha256::digest(data).into();
            let mut tagged = Vec::with_capacity(crate::crypto::DOMAIN_SEPARATED_PREIMAGE_LEN);
            tagged.push(attestation_type as u8);
            tagged.extend_from_slice(&inner_hash);
            let message_hash: [u8; 32] = Sha256::digest(&tagged).into();

            let sig = self
                .signer
                .sign_hash_sync(&message_hash.into())
                .expect("Signing should succeed");

            let mut result = Vec::with_capacity(65);
            result.extend_from_slice(&sig.r().to_be_bytes::<32>());
            result.extend_from_slice(&sig.s().to_be_bytes::<32>());
            result.push(u8::from(sig.v()) + 27);
            result
        }
    }

    /// Create a set of test attestors
    pub fn create_test_attestors(count: usize) -> Vec<TestAttestor> {
        (1..=count as u8).map(TestAttestor::new).collect()
    }
}

// ── ProgramTest (BPF runtime) integration test helpers ──

pub const TEST_CPI_PROXY_ID: solana_sdk::pubkey::Pubkey =
    solana_sdk::pubkey!("CtQLLKbDMt1XVNXtLKJEt1K8cstbckjqE6zyFqR37KTc");
pub const TEST_CPI_TARGET_ID: solana_sdk::pubkey::Pubkey =
    solana_sdk::pubkey!("GHB99UGVmKFeNrtSLsuzL2QhZZgaqcASvTjotQd2dZzu");
const DEPLOY_DIR: &str = "../../target/deploy";

pub const ANCHOR_ERROR_OFFSET: u32 = 6000;

pub fn anchor_discriminator(instruction_name: &str) -> [u8; 8] {
    let hash = solana_sdk::hash::hash(format!("global:{instruction_name}").as_bytes());
    let mut disc = [0u8; 8];
    disc.copy_from_slice(&hash.to_bytes()[..8]);
    disc
}

pub fn setup_program_test_with_whitelist(
    admin: &solana_sdk::pubkey::Pubkey,
    whitelisted_programs: &[solana_sdk::pubkey::Pubkey],
) -> solana_program_test::ProgramTest {
    use anchor_lang::{AccountSerialize, AnchorSerialize, Discriminator, Space};

    if std::env::var("SBF_OUT_DIR").is_err() {
        let deploy_dir = std::path::Path::new(DEPLOY_DIR);
        std::env::set_var("SBF_OUT_DIR", deploy_dir);
    }

    let mut pt = solana_program_test::ProgramTest::new("attestation", crate::ID, None);
    pt.add_program("test_cpi_proxy", TEST_CPI_PROXY_ID, None);
    pt.add_program("test_cpi_target", TEST_CPI_TARGET_ID, None);
    pt.add_program("access_manager", access_manager::ID, None);

    // Pre-create AppState PDA
    let app_state_pda = crate::types::AppState::pda();
    let app_state = crate::types::AppState {
        version: crate::types::AccountVersion::V1,
        am_state: AccessManagerState::new(access_manager::ID),
        _reserved: [0; 256],
    };
    let mut app_data = vec![0u8; 8 + crate::types::AppState::INIT_SPACE];
    app_state.try_serialize(&mut &mut app_data[..]).unwrap();

    pt.add_account(
        app_state_pda,
        solana_sdk::account::Account {
            lamports: 1_000_000,
            data: app_data,
            owner: crate::ID,
            executable: false,
            rent_epoch: 0,
        },
    );

    // Pre-create AccessManager PDA with admin role and whitelist
    let (access_manager_pda, _) = solana_sdk::pubkey::Pubkey::find_program_address(
        &[access_manager::state::AccessManager::SEED],
        &access_manager::ID,
    );
    let am = access_manager::state::AccessManager {
        roles: vec![access_manager::RoleData {
            role_id: solana_ibc_types::roles::ADMIN_ROLE,
            members: vec![*admin],
        }],
        whitelisted_programs: whitelisted_programs.to_vec(),
        pending_authority_transfers: vec![],
    };
    let mut am_data = access_manager::state::AccessManager::DISCRIMINATOR.to_vec();
    am.serialize(&mut am_data).unwrap();

    pt.add_account(
        access_manager_pda,
        solana_sdk::account::Account {
            lamports: 1_000_000,
            data: am_data,
            owner: access_manager::ID,
            executable: false,
            rent_epoch: 0,
        },
    );

    pt
}

pub fn wrap_in_test_cpi_proxy(
    payer: solana_sdk::pubkey::Pubkey,
    inner_ix: &solana_sdk::instruction::Instruction,
) -> solana_sdk::instruction::Instruction {
    use anchor_lang::AnchorSerialize;
    use solana_sdk::instruction::{AccountMeta, Instruction};

    let mut data = Vec::new();
    data.extend_from_slice(&anchor_discriminator("proxy_cpi"));
    inner_ix.data.serialize(&mut data).unwrap();
    let meta_count = inner_ix.accounts.len() as u32;
    meta_count.serialize(&mut data).unwrap();
    for meta in &inner_ix.accounts {
        meta.is_signer.serialize(&mut data).unwrap();
        meta.is_writable.serialize(&mut data).unwrap();
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

pub fn wrap_in_test_cpi_target_proxy(
    payer: solana_sdk::pubkey::Pubkey,
    inner_ix: &solana_sdk::instruction::Instruction,
) -> solana_sdk::instruction::Instruction {
    use anchor_lang::AnchorSerialize;
    use solana_sdk::instruction::{AccountMeta, Instruction};

    let mut data = Vec::new();
    data.extend_from_slice(&anchor_discriminator("proxy_cpi"));
    inner_ix.data.serialize(&mut data).unwrap();
    let meta_count = inner_ix.accounts.len() as u32;
    meta_count.serialize(&mut data).unwrap();
    for meta in &inner_ix.accounts {
        meta.is_signer.serialize(&mut data).unwrap();
        meta.is_writable.serialize(&mut data).unwrap();
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

pub mod access_control {
    use access_manager::RoleData;
    use anchor_lang::prelude::Pubkey;
    use anchor_lang::{AnchorSerialize, Discriminator};

    pub fn setup_access_manager(admin: Pubkey, relayers: Vec<Pubkey>) -> (Pubkey, Vec<u8>) {
        let (access_manager_pda, _) = Pubkey::find_program_address(
            &[access_manager::state::AccessManager::SEED],
            &access_manager::ID,
        );

        let mut roles = vec![RoleData {
            role_id: solana_ibc_types::roles::ADMIN_ROLE,
            members: vec![admin],
        }];

        if !relayers.is_empty() {
            roles.push(RoleData {
                role_id: solana_ibc_types::roles::RELAYER_ROLE,
                members: relayers,
            });
        }

        let access_manager = access_manager::state::AccessManager {
            roles,
            whitelisted_programs: vec![],
            pending_authority_transfers: vec![],
        };

        let mut data = access_manager::state::AccessManager::DISCRIMINATOR.to_vec();
        access_manager.serialize(&mut data).unwrap();

        (access_manager_pda, data)
    }

    pub fn create_access_manager_account(
        admin: Pubkey,
        relayers: Vec<Pubkey>,
    ) -> (Pubkey, solana_sdk::account::Account) {
        let (pda, data) = setup_access_manager(admin, relayers);

        let account = solana_sdk::account::Account {
            lamports: 10_000_000,
            data,
            owner: access_manager::ID,
            executable: false,
            rent_epoch: 0,
        };

        (pda, account)
    }
}
