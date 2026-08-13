// SPDX-License-Identifier: Apache-2.0

//! [`ChainProgram`] implementations for every on-chain program used in tests.
//!
//! Each struct knows how to register its `.so` binary on `ProgramTest`,
//! declare an IBC port (if applicable) and build program-specific
//! initialization instructions.

use crate::accounts::account_owned_by;
use crate::attestor::Attestors;
use crate::chain::{add_program_data, mock_ibc_app_state_pda, ChainProgram, InitStepSigner};
use anchor_lang::InstructionData;
use solana_ibc_sdk::access_manager::instructions as am_sdk;
use solana_ibc_sdk::attestation::instructions as attestation_sdk;
use solana_ibc_sdk::ics27_gmp::instructions as gmp_sdk;
use solana_ibc_sdk::ift::instructions as ift_sdk;
use solana_program_test::ProgramTest;
use solana_sdk::{
    instruction::{AccountMeta, Instruction},
    pubkey,
    pubkey::Pubkey,
    signature::Keypair,
    signer::Signer as _,
    system_program,
};

/// Program ID of the primary attestation light client.
pub const ATTESTATION_PROGRAM_ID: Pubkey = attestation::ID;

/// Program ID for the `test-attestation` instance
/// (built via `just solana::build-solana-test-instance attestation test-attestation`).
pub const TEST_ATTESTATION_ID: Pubkey = pubkey!("2kXQM1LsQeWLEB5GHBGDmUqNzNfejh3pVHaauiQv6FdV");

// ── TestIbcApp ──────────────────────────────────────────────────────────

/// Stateful `test_ibc_app` that counts packets sent/received/acked/timed-out.
pub struct TestIbcApp;

impl ChainProgram for TestIbcApp {
    fn register(&self, pt: &mut ProgramTest, _deployer: Pubkey) {
        pt.add_program("test_ibc_app", test_ibc_app::ID, None);
    }

    fn ibc_port_and_id(&self) -> Option<(&str, Pubkey)> {
        Some((crate::router::PORT_ID, test_ibc_app::ID))
    }

    fn init_steps(
        &self,
        deployer: &Keypair,
        _admin: Pubkey,
    ) -> Vec<(Vec<Instruction>, InitStepSigner)> {
        let payer = deployer.pubkey();
        let (app_state_pda, _) = solana_ibc_sdk::pda::ibc_app::app_state_pda(&test_ibc_app::ID);
        let ix = Instruction {
            program_id: test_ibc_app::ID,
            accounts: vec![
                AccountMeta::new(app_state_pda, false),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(system_program::ID, false),
            ],
            data: test_ibc_app::instruction::Initialize { authority: payer }.data(),
        };
        vec![(vec![ix], InitStepSigner::DeployerOnly)]
    }
}

// ── MockIbcApp ──────────────────────────────────────────────────────────

/// Stateless `mock_ibc_app` with magic-string ack control
/// (`RETURN_ERROR_ACK` / `RETURN_EMPTY_ACK`).
pub struct MockIbcApp;

impl ChainProgram for MockIbcApp {
    fn register(&self, pt: &mut ProgramTest, _deployer: Pubkey) {
        pt.add_program("mock_ibc_app", mock_ibc_app::ID, None);
        pt.add_account(
            mock_ibc_app_state_pda(),
            account_owned_by(vec![0u8; 100], mock_ibc_app::ID),
        );
    }

    fn ibc_port_and_id(&self) -> Option<(&str, Pubkey)> {
        Some((crate::router::PORT_ID, mock_ibc_app::ID))
    }
}

// ── Ics27Gmp ────────────────────────────────────────────────────────────

/// `ics27_gmp` — GMP IBC application registered on the GMP port.
pub struct Ics27Gmp;

impl ChainProgram for Ics27Gmp {
    fn register(&self, pt: &mut ProgramTest, deployer: Pubkey) {
        pt.add_program("ics27_gmp", ics27_gmp::ID, None);
        add_program_data(pt, ics27_gmp::ID, deployer);
    }

    fn ibc_port_and_id(&self) -> Option<(&str, Pubkey)> {
        Some((crate::gmp::GMP_PORT_ID, ics27_gmp::ID))
    }

    fn init_steps(
        &self,
        deployer: &Keypair,
        _admin: Pubkey,
    ) -> Vec<(Vec<Instruction>, InitStepSigner)> {
        let payer = deployer.pubkey();
        let ix = gmp_sdk::Initialize::builder(&ics27_gmp::ID)
            .accounts(gmp_sdk::InitializeAccounts {
                payer,
                program_data: gmp_sdk::Initialize::program_data_pda().0,
                authority: payer,
            })
            .args(&gmp_sdk::InitializeArgs {
                access_manager: access_manager::ID,
            })
            .build();
        vec![(vec![ix], InitStepSigner::DeployerOnly)]
    }

    fn upgrade_authority_program_id(&self) -> Option<Pubkey> {
        Some(ics27_gmp::ID)
    }
}

// ── TestGmpApp ──────────────────────────────────────────────────────────

/// `test_gmp_app` — counter app invoked by GMP via CPI.
pub struct TestGmpApp;

impl ChainProgram for TestGmpApp {
    fn register(&self, pt: &mut ProgramTest, _deployer: Pubkey) {
        pt.add_program("test_gmp_app", test_gmp_app::ID, None);
    }

    fn init_steps(
        &self,
        deployer: &Keypair,
        _admin: Pubkey,
    ) -> Vec<(Vec<Instruction>, InitStepSigner)> {
        let payer = deployer.pubkey();
        let (app_state_pda, _) = Pubkey::find_program_address(
            &[test_gmp_app::state::CounterAppState::SEED],
            &test_gmp_app::ID,
        );
        let ix = Instruction {
            program_id: test_gmp_app::ID,
            accounts: vec![
                AccountMeta::new(app_state_pda, false),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(system_program::ID, false),
            ],
            data: test_gmp_app::instruction::Initialize { authority: payer }.data(),
        };
        vec![(vec![ix], InitStepSigner::DeployerOnly)]
    }
}

// ── TestCpiProxy ────────────────────────────────────────────────────────

/// `test_cpi_proxy` — generic CPI proxy for security tests.
pub struct TestCpiProxy;

impl ChainProgram for TestCpiProxy {
    fn register(&self, pt: &mut ProgramTest, _deployer: Pubkey) {
        pt.add_program("test_cpi_proxy", test_cpi_proxy::ID, None);
    }
}

// ── Ift ─────────────────────────────────────────────────────────────────

/// `ift` — inter-chain fungible token transfers (uses GMP's port).
///
/// The IFT program has its own admin (stored in `IFTAppState.admin`) that is
/// independent from the access manager `ADMIN_ROLE`. Initialize IFT separately
/// via [`Deployer::init_programs`] to pass the desired IFT admin pubkey.
pub struct Ift;

impl ChainProgram for Ift {
    fn register(&self, pt: &mut ProgramTest, deployer: Pubkey) {
        pt.add_program("ift", ift::ID, None);
        add_program_data(pt, ift::ID, deployer);
    }

    fn init_steps(
        &self,
        deployer: &Keypair,
        admin: Pubkey,
    ) -> Vec<(Vec<Instruction>, InitStepSigner)> {
        let payer = deployer.pubkey();
        let ix = ift_sdk::Initialize::builder(&ift::ID)
            .accounts(ift_sdk::InitializeAccounts {
                payer,
                program_data: ift_sdk::Initialize::program_data_pda().0,
                authority: payer,
            })
            .args(&ift_sdk::InitializeArgs { admin })
            .build();
        vec![(vec![ix], InitStepSigner::DeployerOnly)]
    }

    fn upgrade_authority_program_id(&self) -> Option<Pubkey> {
        Some(ift::ID)
    }
}

// ── AttestationLc ──────────────────────────────────────────────────────

/// Attestation-based light client with ECDSA signature verification.
///
/// Unlike mock LC (auto-registered by the chain), the attestation LC must
/// be included in the `programs` slice and requires explicit attestor
/// addresses and a quorum threshold.
///
/// Supports multiple instances via [`with_program_id`](Self::with_program_id),
/// each backed by a separate `.so` binary with its own `declare_id!`.
pub struct AttestationLc {
    pub attestor_addresses: Vec<[u8; 20]>,
    pub min_required_sigs: u8,
    pub program_id: Pubkey,
    pub binary_name: &'static str,
}

impl AttestationLc {
    /// Build from an [`Attestors`] set, requiring all signatures (N-of-N quorum).
    ///
    /// Uses the default `attestation::ID` and `"attestation"` binary.
    pub fn new(attestors: &Attestors) -> Self {
        let addresses = attestors.eth_addresses();
        let min_required_sigs = addresses.len() as u8;
        Self {
            attestor_addresses: addresses,
            min_required_sigs,
            program_id: attestation::ID,
            binary_name: "attestation",
        }
    }

    /// Build from an [`Attestors`] set with a custom program ID and binary name.
    ///
    /// Use this when deploying multiple attestation instances on the same chain
    /// (e.g. the three-chain test where each chain needs two client connections).
    pub fn with_program_id(
        attestors: &Attestors,
        program_id: Pubkey,
        binary_name: &'static str,
    ) -> Self {
        let addresses = attestors.eth_addresses();
        let min_required_sigs = addresses.len() as u8;
        Self {
            attestor_addresses: addresses,
            min_required_sigs,
            program_id,
            binary_name,
        }
    }
}

impl ChainProgram for AttestationLc {
    fn register(&self, pt: &mut ProgramTest, deployer: Pubkey) {
        pt.add_program(self.binary_name, self.program_id, None);
        add_program_data(pt, self.program_id, deployer);
    }

    fn init_steps(
        &self,
        deployer: &Keypair,
        _admin: Pubkey,
    ) -> Vec<(Vec<Instruction>, InitStepSigner)> {
        let payer = deployer.pubkey();
        let pid = self.program_id;
        let (program_data_pda, _) =
            Pubkey::find_program_address(&[pid.as_ref()], &solana_sdk::bpf_loader_upgradeable::ID);

        let ix = attestation_sdk::Initialize::builder(&pid)
            .accounts(attestation_sdk::InitializeAccounts {
                payer,
                program_data: program_data_pda,
                authority: payer,
            })
            .args(&attestation_sdk::InitializeArgs {
                attestor_addresses: self.attestor_addresses.clone(),
                min_required_sigs: self.min_required_sigs,
                access_manager: access_manager::ID,
            })
            .build();
        vec![(vec![ix], InitStepSigner::DeployerOnly)]
    }

    fn upgrade_authority_program_id(&self) -> Option<Pubkey> {
        Some(self.program_id)
    }
}

// ── TestAccessManager ───────────────────────────────────────────────────

/// `test_access_manager` — second AM instance for AM migration tests.
pub struct TestAccessManager;

impl ChainProgram for TestAccessManager {
    fn register(&self, pt: &mut ProgramTest, deployer: Pubkey) {
        pt.add_program("test_access_manager", test_access_manager::ID, None);
        add_program_data(pt, test_access_manager::ID, deployer);
    }

    fn init_steps(
        &self,
        deployer: &Keypair,
        admin: Pubkey,
    ) -> Vec<(Vec<Instruction>, InitStepSigner)> {
        let payer = deployer.pubkey();
        let (program_data_pda, _) = Pubkey::find_program_address(
            &[test_access_manager::ID.as_ref()],
            &solana_sdk::bpf_loader_upgradeable::ID,
        );

        let init_ix = am_sdk::Initialize::builder(&test_access_manager::ID)
            .accounts(am_sdk::InitializeAccounts {
                payer,
                program_data: program_data_pda,
                authority: payer,
            })
            .args(&am_sdk::InitializeArgs { admin })
            .build();

        let grant_ix = am_sdk::GrantRole::builder(&test_access_manager::ID)
            .accounts(am_sdk::GrantRoleAccounts { admin })
            .args(&am_sdk::GrantRoleArgs {
                role_id: solana_ibc_constants::roles::ADMIN_ROLE,
                account: admin,
            })
            .build();

        vec![
            (vec![init_ix], InitStepSigner::DeployerOnly),
            (vec![grant_ix], InitStepSigner::WithAdmin),
        ]
    }

    fn upgrade_authority_program_id(&self) -> Option<Pubkey> {
        Some(test_access_manager::ID)
    }
}
