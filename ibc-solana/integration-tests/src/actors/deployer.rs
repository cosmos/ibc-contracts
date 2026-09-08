// SPDX-License-Identifier: Apache-2.0

//! Deployer / upgrade-authority actor.
//!
//! Orchestrates the full program initialization sequence (access manager,
//! router, light client, IBC apps and program-specific steps) and
//! transfers upgrade authority to the access manager PDA afterwards.

use super::Actor;
use crate::admin::Admin;
use crate::chain::{Chain, ChainProgram, InitStepSigner};
use crate::relayer::Relayer;
use solana_ibc_sdk::access_manager::instructions as am_sdk;
use solana_ibc_sdk::ics26_router::instructions as router_sdk;
use solana_ibc_sdk::ics26_router::types::CounterpartyInfo;
use solana_sdk::{
    bpf_loader_upgradeable, instruction::Instruction, pubkey::Pubkey, signature::Keypair,
    signer::Signer, transaction::Transaction,
};

/// Deployer / upgrade-authority actor.
pub struct Deployer {
    keypair: Keypair,
}

impl Default for Deployer {
    fn default() -> Self {
        Self::new()
    }
}

impl Actor for Deployer {
    fn keypair(&self) -> &Keypair {
        &self.keypair
    }
}

impl Deployer {
    /// Create a deployer with a fresh random keypair.
    pub fn new() -> Self {
        Self {
            keypair: Keypair::new(),
        }
    }

    // ── Public init methods ──────────────────────────────────────────────

    /// Initialize the full IBC stack on the chain.
    ///
    /// Must be called after `chain.start()`. Executes AM, router, light
    /// client, IBC app registration and program-specific initialization
    /// transactions.
    pub async fn init_ibc_stack(
        &self,
        chain: &mut Chain,
        admin: &Admin,
        relayer: &Relayer,
        programs: &[&dyn ChainProgram],
    ) {
        let steps = build_init_steps(
            self.keypair(),
            admin.pubkey(),
            relayer.pubkey(),
            chain.client_id(),
            chain.counterparty_client_id(),
            programs,
        );

        let admin_kp = admin.keypair();
        for (ixs, signer) in &steps {
            let signers: Vec<&Keypair> = match signer {
                InitStepSigner::DeployerOnly => vec![self.keypair()],
                InitStepSigner::WithAdmin => vec![self.keypair(), admin_kp],
            };
            submit_tx(chain, ixs, &signers).await;
        }
    }

    /// Run only the program-specific initialization steps.
    ///
    /// Unlike [`init_ibc_stack`](Self::init_ibc_stack), this skips the core
    /// infrastructure (AM, router, light client, IBC app registration).
    /// Use this when a program needs a different admin than the core stack.
    pub async fn init_programs(
        &self,
        chain: &mut Chain,
        admin: Pubkey,
        programs: &[&dyn ChainProgram],
    ) {
        for p in programs {
            for (ixs, _signer) in p.init_steps(self.keypair(), admin) {
                submit_tx(chain, &ixs, &[self.keypair()]).await;
            }
        }
    }

    /// Transfer upgrade authority of all deployed programs to the access
    /// manager PDA, reflecting a production deployment where governance
    /// controls upgrades.
    pub async fn transfer_upgrade_authority(
        &self,
        chain: &mut Chain,
        programs: &[&dyn ChainProgram],
    ) {
        let am_pda = derive_access_manager_pda();
        let ixs = build_transfer_upgrade_authority_ixs(self.keypair().pubkey(), am_pda, programs);
        if !ixs.is_empty() {
            submit_tx(chain, &ixs, &[self.keypair()]).await;
        }
    }

    /// Register an additional client/counterparty pair on the chain.
    ///
    /// Initializes the mock light client and calls `add_client` on the
    /// router. Used for multi-hop tests (e.g. three-chain roundtrip).
    /// Register an additional client/counterparty pair backed by an
    /// attestation LC instance.
    ///
    /// Calls `add_client` on the router referencing the given `lc_program_id`.
    /// The attestation LC's `initialize` is already handled by the
    /// `AttestationLc` `ChainProgram` in `init_steps`, so only the
    /// router-side registration is needed here.
    pub async fn add_counterparty_with_attestation(
        &self,
        chain: &mut Chain,
        admin: &Admin,
        client_id: &str,
        counterparty_client_id: &str,
        lc_program_id: Pubkey,
    ) {
        let add_ix = build_add_client_ix(
            admin.pubkey(),
            derive_access_manager_pda(),
            client_id,
            counterparty_client_id,
            lc_program_id,
        );
        submit_tx(chain, &[add_ix], &[self.keypair(), admin.keypair()]).await;
    }
}

// ── Helpers ─────────────────────────────────────────────────────────────

async fn submit_tx(chain: &mut Chain, ixs: &[Instruction], signers: &[&Keypair]) {
    let tx = Transaction::new_signed_with_payer(
        ixs,
        Some(&signers[0].pubkey()),
        signers,
        chain.blockhash(),
    );
    chain.process_transaction(tx).await.expect("init tx failed");
}

// ── PDA derivation ──────────────────────────────────────────────────────

fn derive_access_manager_pda() -> Pubkey {
    am_sdk::Initialize::access_manager_pda(&access_manager::ID).0
}

// ── Init step builder ───────────────────────────────────────────────────

/// Return the (`port_id`, `program_id`) for the first IBC application in the list.
fn get_port_and_app<'a>(programs: &'a [&'a dyn ChainProgram]) -> (&'a str, Pubkey) {
    for p in programs {
        if let Some(port_and_id) = p.ibc_port_and_id() {
            return port_and_id;
        }
    }
    panic!("no IBC application in programs list");
}

/// Build all initialization instructions grouped by transaction.
///
/// Returns `(instructions, signer)` tuples that must be executed
/// sequentially. The deployer always pays transaction fees; `WithAdmin`
/// steps additionally require the admin keypair as co-signer.
fn build_init_steps(
    deployer: &Keypair,
    admin_pubkey: Pubkey,
    relayer_pubkey: Pubkey,
    client_id: &str,
    counterparty_client_id: &str,
    programs: &[&dyn ChainProgram],
) -> Vec<(Vec<Instruction>, InitStepSigner)> {
    let deployer_pubkey = deployer.pubkey();
    let am_pda = derive_access_manager_pda();
    let (port_id, app_program_id) = get_port_and_app(programs);

    let mut steps = vec![
        // TX1: access_manager::initialize (deployer = upgrade authority + payer)
        (
            vec![build_am_initialize_ix(
                deployer_pubkey,
                deployer,
                admin_pubkey,
            )],
            InitStepSigner::DeployerOnly,
        ),
        // TX2: grant RELAYER_ROLE and ID_CUSTOMIZER_ROLE (admin = AM admin)
        (
            vec![
                build_am_grant_role_ix(
                    admin_pubkey,
                    solana_ibc_constants::roles::RELAYER_ROLE,
                    relayer_pubkey,
                ),
                build_am_grant_role_ix(
                    admin_pubkey,
                    solana_ibc_constants::roles::ID_CUSTOMIZER_ROLE,
                    admin_pubkey,
                ),
            ],
            InitStepSigner::WithAdmin,
        ),
        // TX3: ics26_router::initialize (deployer = upgrade authority + payer)
        (
            vec![build_router_initialize_ix(
                deployer_pubkey,
                deployer,
                access_manager::ID,
            )],
            InitStepSigner::DeployerOnly,
        ),
    ];

    // add_client + add_ibc_app (admin = ID_CUSTOMIZER_ROLE holder)
    steps.push((
        vec![
            build_add_client_ix(
                admin_pubkey,
                am_pda,
                client_id,
                counterparty_client_id,
                attestation::ID,
            ),
            build_add_ibc_app_ix(
                deployer_pubkey,
                admin_pubkey,
                am_pda,
                port_id,
                app_program_id,
            ),
        ],
        InitStepSigner::WithAdmin,
    ));

    // App-specific initialization (each program provides its own steps)
    for p in programs {
        steps.extend(p.init_steps(deployer, admin_pubkey));
    }

    steps
}

// ── Upgrade authority transfer ──────────────────────────────────────────

fn build_transfer_upgrade_authority_ixs(
    deployer: Pubkey,
    am_pda: Pubkey,
    programs: &[&dyn ChainProgram],
) -> Vec<Instruction> {
    let mut ixs = vec![
        bpf_loader_upgradeable::set_upgrade_authority(
            &access_manager::ID,
            &deployer,
            Some(&am_pda),
        ),
        bpf_loader_upgradeable::set_upgrade_authority(&ics26_router::ID, &deployer, Some(&am_pda)),
    ];

    for p in programs {
        if let Some(program_id) = p.upgrade_authority_program_id() {
            ixs.push(bpf_loader_upgradeable::set_upgrade_authority(
                &program_id,
                &deployer,
                Some(&am_pda),
            ));
        }
    }

    ixs
}

// ── Instruction builders for initialization ─────────────────────────────

fn build_am_initialize_ix(payer: Pubkey, deployer: &Keypair, admin: Pubkey) -> Instruction {
    am_sdk::Initialize::builder(&access_manager::ID)
        .accounts(am_sdk::InitializeAccounts {
            payer,
            program_data: am_sdk::Initialize::program_data_pda().0,
            authority: deployer.pubkey(),
        })
        .args(&am_sdk::InitializeArgs { admin })
        .build()
}

fn build_am_grant_role_ix(admin: Pubkey, role_id: u64, account: Pubkey) -> Instruction {
    am_sdk::GrantRole::builder(&access_manager::ID)
        .accounts(am_sdk::GrantRoleAccounts { admin })
        .args(&am_sdk::GrantRoleArgs { role_id, account })
        .build()
}

fn build_router_initialize_ix(
    payer: Pubkey,
    deployer: &Keypair,
    access_manager_program: Pubkey,
) -> Instruction {
    router_sdk::Initialize::builder(&ics26_router::ID)
        .accounts(router_sdk::InitializeAccounts {
            payer,
            program_data: router_sdk::Initialize::program_data_pda().0,
            authority: deployer.pubkey(),
        })
        .args(&router_sdk::InitializeArgs {
            access_manager: access_manager_program,
        })
        .build()
}

fn build_add_client_ix(
    admin: Pubkey,
    am_pda: Pubkey,
    client_id: &str,
    counterparty_client_id: &str,
    lc_program_id: Pubkey,
) -> Instruction {
    router_sdk::AddClient::builder(&ics26_router::ID)
        .accounts(router_sdk::AddClientAccounts {
            authority: admin,
            access_manager: am_pda,
            light_client_program: lc_program_id,
            client_id,
        })
        .args(&router_sdk::AddClientArgs {
            client_id: client_id.to_string(),
            counterparty_info: CounterpartyInfo {
                client_id: counterparty_client_id.to_string(),
                merkle_prefix: vec![vec![0x01, 0x02, 0x03]],
            },
        })
        .build()
}

fn build_add_ibc_app_ix(
    payer: Pubkey,
    admin: Pubkey,
    am_pda: Pubkey,
    port_id: &str,
    app_program_id: Pubkey,
) -> Instruction {
    router_sdk::AddIbcApp::builder(&ics26_router::ID)
        .accounts(router_sdk::AddIbcAppAccounts {
            access_manager: am_pda,
            app_program: app_program_id,
            payer,
            authority: admin,
            port_id,
        })
        .args(&router_sdk::AddIbcAppArgs {
            port_id: port_id.to_string(),
        })
        .build()
}
