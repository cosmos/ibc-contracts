// SPDX-License-Identifier: Apache-2.0

//! Simulated Solana chain backed by `ProgramTest` / `BanksClient`.
//!
//! Handles program registration, deterministic clock setup, actor funding,
//! transaction submission and account reads. Each [`Chain`] instance
//! represents an independent local validator used by a single test.

use crate::actors::admin::Admin;
use crate::actors::deployer::Deployer;
use crate::actors::relayer::Relayer;
use crate::Actor;
use solana_program_test::{BanksClient, BanksClientError, ProgramTest};
use solana_sdk::{
    account::Account,
    bpf_loader_upgradeable::{self, UpgradeableLoaderState},
    hash::Hash,
    instruction::Instruction,
    pubkey::Pubkey,
    signature::Keypair,
    system_program,
    sysvar::Sysvar as _,
    transaction::Transaction,
};

const DEPLOY_DIR: &str = "../target/deploy";
/// Deterministic Unix timestamp used for every test chain clock.
pub const TEST_CLOCK_TIME: i64 = 1_700_000_000;
const DEFAULT_PREFUND_LAMPORTS: u64 = 10_000_000_000;
// ── Light-client account references ─────────────────────────────────────

/// Light-client account references passed to router instruction builders.
#[derive(Clone)]
pub struct LcAccounts {
    pub program_id: Pubkey,
    pub client_state: Pubkey,
    pub consensus_state: Pubkey,
}

/// Derive attestation light client [`LcAccounts`] for a given program and proof height.
///
/// The client state PDA is height-independent (seeds: `["client"]`), and the
/// consensus state PDA depends on the proof height
/// (seeds: `["consensus_state", height.to_le_bytes()]`).
pub fn attestation_lc_accounts(program_id: Pubkey, proof_height: u64) -> LcAccounts {
    use solana_ibc_sdk::attestation::instructions as att_sdk;
    let (client_state, _) = att_sdk::Initialize::client_state_pda(&program_id);
    let (consensus_state, _) =
        att_sdk::VerifyMembership::consensus_state_at_height_pda(proof_height, &program_id);
    LcAccounts {
        program_id,
        client_state,
        consensus_state,
    }
}

/// Deterministic PDA for `mock_ibc_app`'s dummy state account.
///
/// `mock_ibc_app` has no `initialize` instruction, so we pre-create a dummy
/// account at this address during `ProgramTest` setup. The same PDA must be
/// used everywhere the router needs the app's state account.
pub fn mock_ibc_app_state_pda() -> Pubkey {
    Pubkey::find_program_address(&[b"state"], &mock_ibc_app::ID).0
}

// ── ChainProgram trait ──────────────────────────────────────────────────

/// Which signers a particular init step requires.
///
/// The deployer always pays transaction fees and signs as the fee payer.
/// `WithAdmin` adds the admin keypair as a co-signer for AM-role-gated steps.
#[derive(Clone, Copy)]
pub enum InitStepSigner {
    DeployerOnly,
    WithAdmin,
}

/// A program that can be loaded onto a test chain.
///
/// Each implementor knows how to register itself on `ProgramTest`, provide
/// IBC port info and build its own initialization / upgrade-authority steps.
pub trait ChainProgram: Sync {
    /// Register this program on a `ProgramTest` instance.
    fn register(&self, pt: &mut ProgramTest, deployer: Pubkey);

    /// Return `(port_id, program_id)` if this program is an IBC application
    /// that should be registered on the router via `add_ibc_app`.
    fn ibc_port_and_id(&self) -> Option<(&str, Pubkey)> {
        None
    }

    /// Return program-specific initialization steps (run after the common
    /// AM + router + light-client + `add_client` + `add_ibc_app` transactions).
    fn init_steps(
        &self,
        _deployer: &Keypair,
        _admin: Pubkey,
    ) -> Vec<(Vec<Instruction>, InitStepSigner)> {
        vec![]
    }

    /// Return the program ID whose upgrade authority should be transferred
    /// to the access manager PDA after initialization.
    fn upgrade_authority_program_id(&self) -> Option<Pubkey> {
        None
    }
}

// ── Chain config & runtime ──────────────────────────────────────────────

/// Parameters for constructing a new [`Chain`].
pub struct ChainConfig<'a> {
    /// Light-client identifier on this chain (e.g. `"08-wasm-0"`).
    pub client_id: &'a str,
    /// Light-client identifier on the counterparty chain.
    pub counterparty_client_id: &'a str,
    /// Deployer actor that holds upgrade authority.
    pub deployer: &'a Deployer,
    /// Application programs to register alongside the core stack.
    pub programs: &'a [&'a dyn ChainProgram],
}

/// Simulated Solana validator for a single test.
///
/// Has two phases: **setup** (before [`Chain::start`]) where programs and
/// accounts are registered, and **runtime** (after start) where
/// transactions can be submitted.
pub struct Chain {
    pt: Option<ProgramTest>,
    client_id: String,
    counterparty_client_id: String,
    clock_time: i64,
    banks: Option<BanksClient>,
    blockhash: Hash,
}

impl Chain {
    /// Create a new chain from the given config (setup phase).
    pub fn new(config: ChainConfig<'_>) -> Self {
        let pt = build_program_test(&config);

        Self {
            pt: Some(pt),
            client_id: config.client_id.to_string(),
            counterparty_client_id: config.counterparty_client_id.to_string(),
            clock_time: TEST_CLOCK_TIME,
            banks: None,
            blockhash: Hash::default(),
        }
    }

    /// Create a single chain with default client IDs.
    pub fn single(deployer: &Deployer, programs: &[&dyn ChainProgram]) -> Self {
        Self::new(ChainConfig {
            client_id: "chain-a-client",
            counterparty_client_id: "chain-b-client",
            deployer,
            programs,
        })
    }

    /// Create two chains with mirrored client IDs and different programs.
    pub fn pair(
        deployer: &Deployer,
        programs_a: &[&dyn ChainProgram],
        programs_b: &[&dyn ChainProgram],
    ) -> (Self, Self) {
        let chain_a = Self::new(ChainConfig {
            client_id: "chain-a-client",
            counterparty_client_id: "chain-b-client",
            deployer,
            programs: programs_a,
        });
        let chain_b = Self::new(ChainConfig {
            client_id: "chain-b-client",
            counterparty_client_id: "chain-a-client",
            deployer,
            programs: programs_b,
        });
        (chain_a, chain_b)
    }

    // ── Setup phase (before start) ──────────────────────────────────────

    /// Pre-fund actor accounts with the default amount (10 SOL each).
    pub fn prefund(&mut self, actors: &[&dyn Actor]) {
        for actor in actors {
            self.prefund_lamports(actor.pubkey(), DEFAULT_PREFUND_LAMPORTS);
        }
    }

    /// Pre-fund an account with a specific lamport amount.
    pub fn prefund_lamports(&mut self, pubkey: Pubkey, lamports: u64) {
        self.pt().add_account(pubkey, system_account(lamports));
    }

    /// Start the `ProgramTest` runtime, producing a `BanksClient`.
    ///
    /// After calling `start()`, use `Deployer::init_ibc_stack` and
    /// `Deployer::transfer_upgrade_authority` to initialize on-chain state.
    pub async fn start(&mut self) {
        let pt = self.pt.take().expect("chain already started");
        let (banks, _payer, blockhash) = pt.start().await;
        self.banks = Some(banks);
        self.blockhash = blockhash;
    }

    /// Start the chain and initialize the full IBC stack, transferring
    /// upgrade authority to the access manager PDA.
    ///
    /// Combines `start`, `Deployer::init_ibc_stack` and
    /// `Deployer::transfer_upgrade_authority` into the canonical setup
    /// sequence used by most tests.
    pub async fn init(
        &mut self,
        deployer: &Deployer,
        admin: &Admin,
        relayer: &Relayer,
        programs: &[&dyn ChainProgram],
    ) {
        self.start().await;
        deployer
            .init_ibc_stack(self, admin, relayer, programs)
            .await;
        deployer.transfer_upgrade_authority(self, programs).await;
    }

    /// Like [`init`](Self::init) but also submits an `update_client`
    /// transaction for the attestation LC, creating a consensus state at
    /// [`PROOF_HEIGHT`](crate::router::PROOF_HEIGHT).
    ///
    /// Attestation LC requires an explicit `update_client` after
    /// initialization (unlike mock LC which is pre-seeded with a
    /// consensus state). This method bundles both steps so attestation
    /// tests don't need a separate `update_client` call.
    pub async fn init_with_attestation(
        &mut self,
        deployer: &Deployer,
        admin: &Admin,
        relayer: &Relayer,
        programs: &[&dyn ChainProgram],
        attestors: &crate::attestor::Attestors,
    ) {
        self.init(deployer, admin, relayer, programs).await;
        relayer
            .attestation_update_client(self, attestors, crate::router::PROOF_HEIGHT)
            .await
            .expect("attestation update_client during init failed");
    }

    // ── Runtime phase (after start) ─────────────────────────────────────

    /// Light-client identifier on this chain.
    pub fn client_id(&self) -> &str {
        &self.client_id
    }

    /// Light-client identifier on the counterparty chain.
    pub fn counterparty_client_id(&self) -> &str {
        &self.counterparty_client_id
    }

    /// Deterministic Unix timestamp set during chain construction.
    pub const fn clock_time(&self) -> i64 {
        self.clock_time
    }

    /// Default [`LcAccounts`] for this chain's primary attestation client at
    /// [`PROOF_HEIGHT`](crate::router::PROOF_HEIGHT).
    pub fn lc_accounts(&self) -> LcAccounts {
        attestation_lc_accounts(attestation::ID, crate::router::PROOF_HEIGHT)
    }

    /// [`LcAccounts`] for a specific proof height on the primary attestation client.
    pub fn lc_accounts_at_height(&self, height: u64) -> LcAccounts {
        attestation_lc_accounts(attestation::ID, height)
    }

    /// Derive the `test_gmp_app` `CounterAppState` PDA.
    pub fn counter_app_state_pda(&self) -> Pubkey {
        Pubkey::find_program_address(
            &[test_gmp_app::state::CounterAppState::SEED],
            &test_gmp_app::ID,
        )
        .0
    }

    /// Derive the IFT `IFTAppState` PDA.
    pub fn ift_app_state_pda(&self) -> Pubkey {
        solana_ibc_sdk::ift::instructions::Initialize::app_state_pda(&ift::ID).0
    }

    /// Latest blockhash, refreshed after every transaction.
    pub const fn blockhash(&self) -> Hash {
        self.blockhash
    }

    /// Submit a transaction and auto-refresh the blockhash.
    pub async fn process_transaction(&mut self, tx: Transaction) -> Result<(), BanksClientError> {
        let banks = self.banks.as_mut().expect("chain not started yet");
        banks.process_transaction(tx).await?;
        self.blockhash = banks.get_latest_blockhash().await.unwrap();
        Ok(())
    }

    /// Read an account from the chain, returning `None` if it doesn't exist.
    pub async fn get_account(&self, pubkey: Pubkey) -> Option<Account> {
        self.banks
            .as_ref()
            .expect("chain not started yet")
            .get_account(pubkey)
            .await
            .unwrap()
    }

    // ── Helpers ─────────────────────────────────────────────────────────

    const fn pt(&mut self) -> &mut ProgramTest {
        self.pt.as_mut().expect("chain already started")
    }
}

// ── Helpers ──────────────────────────────────────────────────────────────

const fn system_account(lamports: u64) -> Account {
    Account {
        lamports,
        data: vec![],
        owner: system_program::ID,
        executable: false,
        rent_epoch: 0,
    }
}

// ── Internal ProgramTest builder ────────────────────────────────────────

fn ensure_sbf_out_dir() {
    if std::env::var("SBF_OUT_DIR").is_err() {
        std::env::set_var("SBF_OUT_DIR", std::path::Path::new(DEPLOY_DIR));
    }
}

pub(crate) fn add_program_data(pt: &mut ProgramTest, program_id: Pubkey, deployer_pubkey: Pubkey) {
    let (program_data_pda, _) =
        Pubkey::find_program_address(&[program_id.as_ref()], &bpf_loader_upgradeable::ID);
    let state = UpgradeableLoaderState::ProgramData {
        slot: 0,
        upgrade_authority_address: Some(deployer_pubkey),
    };
    pt.add_account(
        program_data_pda,
        Account {
            lamports: 1_000_000,
            data: bincode::serialize(&state).unwrap(),
            owner: bpf_loader_upgradeable::ID,
            executable: false,
            rent_epoch: 0,
        },
    );
}

fn build_program_test(config: &ChainConfig<'_>) -> ProgramTest {
    ensure_sbf_out_dir();

    let deployer_pubkey = config.deployer.pubkey();

    let mut pt = ProgramTest::new("ics26_router", ics26_router::ID, None);
    pt.add_program("access_manager", access_manager::ID, None);

    // ProgramData accounts for programs that verify upgrade authority
    add_program_data(&mut pt, access_manager::ID, deployer_pubkey);
    add_program_data(&mut pt, ics26_router::ID, deployer_pubkey);

    for program in config.programs {
        program.register(&mut pt, deployer_pubkey);
    }

    // Pre-fund deployer (admin and relayer are prefunded via chain.prefund())
    pt.add_account(deployer_pubkey, system_account(DEFAULT_PREFUND_LAMPORTS));

    // Deterministic clock
    let clock = solana_sdk::clock::Clock {
        slot: 1,
        epoch_start_timestamp: TEST_CLOCK_TIME,
        epoch: 0,
        leader_schedule_epoch: 0,
        unix_timestamp: TEST_CLOCK_TIME,
    };
    let mut clock_data = vec![0u8; solana_sdk::clock::Clock::size_of()];
    bincode::serialize_into(&mut clock_data[..], &clock).unwrap();
    pt.add_account(
        solana_sdk::sysvar::clock::ID,
        Account {
            lamports: 1,
            data: clock_data,
            owner: solana_sdk::sysvar::ID,
            executable: false,
            rent_epoch: 0,
        },
    );

    pt
}
