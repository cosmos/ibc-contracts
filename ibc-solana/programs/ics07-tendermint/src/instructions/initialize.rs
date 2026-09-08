// SPDX-License-Identifier: Apache-2.0

use crate::error::ErrorCode;
use crate::state::ConsensusStateStore;
use crate::types::{AppState, ClientState, ConsensusState};
use access_manager::AccessManagerState;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::bpf_loader_upgradeable;

/// Initializes the ICS-07 Tendermint light client with its initial state and configuration.
#[derive(Accounts)]
#[instruction(client_state: ClientState, consensus_state: ConsensusState, access_manager: Pubkey)]
pub struct Initialize<'info> {
    /// PDA holding the Tendermint light client configuration and tracking state.
    #[account(
        init,
        payer = payer,
        space = 8 + ClientState::INIT_SPACE,
        seeds = [ClientState::SEED],
        bump
    )]
    pub client_state_account: Account<'info, ClientState>,
    /// PDA storing the verified consensus state at the initial height.
    #[account(
        init,
        payer = payer,
        space = 8 + ConsensusStateStore::INIT_SPACE,
        seeds = [ConsensusStateStore::SEED, &client_state.latest_height.revision_height.to_le_bytes()],
        bump
    )]
    pub consensus_state_store: Account<'info, ConsensusStateStore>,
    /// PDA holding program-level settings such as the `access_manager` address and `chain_id`.
    #[account(
        init,
        payer = payer,
        space = 8 + AppState::INIT_SPACE,
        seeds = [AppState::SEED],
        bump
    )]
    pub app_state: Account<'info, AppState>,
    /// Signer that pays for PDA account creation.
    #[account(mut)]
    pub payer: Signer<'info>,
    /// Required by Anchor for PDA creation via the System Program.
    pub system_program: Program<'info, System>,

    /// BPF Loader Upgradeable `ProgramData` account for this program.
    #[account(
        seeds = [crate::ID.as_ref()],
        bump,
        seeds::program = bpf_loader_upgradeable::ID,
        constraint = program_data.upgrade_authority_address == Some(authority.key())
            @ ErrorCode::UnauthorizedDeployer
    )]
    pub program_data: Account<'info, ProgramData>,

    /// The program's upgrade authority — must sign to prove deployer identity.
    pub authority: Signer<'info>,
}

pub fn initialize(
    ctx: Context<Initialize>,
    client_state: ClientState,
    consensus_state: ConsensusState,
    access_manager: Pubkey,
) -> Result<()> {
    require!(
        access_manager != Pubkey::default(),
        ErrorCode::InvalidAccessManager
    );

    require!(!client_state.chain_id.is_empty(), ErrorCode::InvalidChainId);

    require!(
        client_state.trust_level_numerator > 0
            && client_state.trust_level_numerator <= client_state.trust_level_denominator
            && client_state.trust_level_denominator > 0,
        ErrorCode::InvalidTrustLevel
    );

    require!(
        client_state.trusting_period > 0
            && client_state.unbonding_period > 0
            && client_state.trusting_period < client_state.unbonding_period,
        ErrorCode::InvalidPeriods
    );

    require!(
        client_state.max_clock_drift > 0,
        ErrorCode::InvalidMaxClockDrift
    );

    require!(
        client_state.latest_height.revision_height > 0,
        ErrorCode::InvalidHeight
    );

    let latest_height = client_state.latest_height;
    ctx.accounts.client_state_account.set_inner(client_state);

    let consensus_state_store = &mut ctx.accounts.consensus_state_store;
    consensus_state_store.height = latest_height.revision_height;
    consensus_state_store.consensus_state = consensus_state;

    let app_state = &mut ctx.accounts.app_state;
    app_state.am_state = AccessManagerState::new(access_manager);
    app_state._reserved = [0; 256];

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::ConsensusStateStore;
    use crate::test_helpers::{fixtures::*, PROGRAM_BINARY_PATH};
    use anchor_lang::InstructionData;
    use mollusk_svm::result::Check;
    use mollusk_svm::Mollusk;
    use solana_sdk::account::Account;
    use solana_sdk::instruction::{AccountMeta, Instruction};
    use solana_sdk::pubkey::Pubkey;
    use solana_sdk::{native_loader, system_program};

    struct TestAccounts {
        payer: Pubkey,
        authority: Pubkey,
        client_state_pda: Pubkey,
        consensus_state_store_pda: Pubkey,
        app_state_pda: Pubkey,
        accounts: Vec<(Pubkey, Account)>,
    }

    fn setup_test_accounts(latest_height: u64) -> TestAccounts {
        let payer = Pubkey::new_unique();
        let authority = Pubkey::new_unique();
        let (client_state_pda, _) =
            Pubkey::find_program_address(&[crate::types::ClientState::SEED], &crate::ID);
        let (consensus_state_store_pda, _) = Pubkey::find_program_address(
            &[
                crate::state::ConsensusStateStore::SEED,
                &latest_height.to_le_bytes(),
            ],
            &crate::ID,
        );
        let (app_state_pda, _) =
            Pubkey::find_program_address(&[crate::types::AppState::SEED], &crate::ID);
        let (program_data_pda, program_data_account) =
            crate::test_helpers::create_program_data_account(&crate::ID, Some(authority));

        let accounts = vec![
            (
                client_state_pda,
                Account {
                    lamports: 0,
                    data: vec![],
                    owner: system_program::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
            (
                consensus_state_store_pda,
                Account {
                    lamports: 0,
                    data: vec![],
                    owner: system_program::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
            (
                app_state_pda,
                Account {
                    lamports: 0,
                    data: vec![],
                    owner: system_program::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
            (
                payer,
                Account {
                    lamports: 10_000_000_000,
                    data: vec![],
                    owner: system_program::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
            (
                system_program::ID,
                Account {
                    lamports: 0,
                    data: vec![],
                    owner: native_loader::ID,
                    executable: true,
                    rent_epoch: 0,
                },
            ),
            (program_data_pda, program_data_account),
            (
                authority,
                Account {
                    lamports: 1_000_000_000,
                    data: vec![],
                    owner: system_program::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
        ];

        TestAccounts {
            payer,
            authority,
            client_state_pda,
            consensus_state_store_pda,
            app_state_pda,
            accounts,
        }
    }

    fn create_initialize_instruction(
        test_accounts: &TestAccounts,
        client_state: &ClientState,
        consensus_state: &ConsensusState,
    ) -> Instruction {
        let (program_data_pda, _) = Pubkey::find_program_address(
            &[crate::ID.as_ref()],
            &anchor_lang::solana_program::bpf_loader_upgradeable::ID,
        );

        let instruction_data = crate::instruction::Initialize {
            client_state: client_state.clone(),
            consensus_state: consensus_state.clone(),
            access_manager: access_manager::ID,
        };

        Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(test_accounts.client_state_pda, false),
                AccountMeta::new(test_accounts.consensus_state_store_pda, false),
                AccountMeta::new(test_accounts.app_state_pda, false),
                AccountMeta::new(test_accounts.payer, true),
                AccountMeta::new_readonly(system_program::ID, false),
                AccountMeta::new_readonly(program_data_pda, false),
                AccountMeta::new_readonly(test_accounts.authority, true),
            ],
            data: instruction_data.data(),
        }
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

    fn test_initialize_validation_failure(
        mut client_state: ClientState,
        consensus_state: ConsensusState,
        setup_invalid_state: impl FnOnce(&mut ClientState),
        expected_error: ErrorCode,
    ) {
        setup_invalid_state(&mut client_state);

        let test_accounts = setup_test_accounts(client_state.latest_height.revision_height);
        let instruction =
            create_initialize_instruction(&test_accounts, &client_state, &consensus_state);

        assert_instruction_fails_with_error(&instruction, &test_accounts.accounts, expected_error);
    }

    #[test]
    fn test_initialize_happy_path() {
        use crate::types::AppState;

        // Load all fixtures efficiently (single JSON parse)
        let (client_state, consensus_state, _) = load_primary_fixtures();

        let payer = Pubkey::new_unique();
        let authority = Pubkey::new_unique();

        let (client_state_pda, _) =
            Pubkey::find_program_address(&[crate::types::ClientState::SEED], &crate::ID);

        let latest_height = client_state.latest_height.revision_height;
        let (consensus_state_store_pda, _) = Pubkey::find_program_address(
            &[
                crate::state::ConsensusStateStore::SEED,
                &latest_height.to_le_bytes(),
            ],
            &crate::ID,
        );
        let (app_state_pda, _) =
            Pubkey::find_program_address(&[crate::types::AppState::SEED], &crate::ID);
        let (program_data_pda, program_data_account) =
            crate::test_helpers::create_program_data_account(&crate::ID, Some(authority));

        let instruction_data = crate::instruction::Initialize {
            client_state: client_state.clone(),
            consensus_state: consensus_state.clone(),
            access_manager: access_manager::ID,
        };

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(client_state_pda, false),
                AccountMeta::new(consensus_state_store_pda, false),
                AccountMeta::new(app_state_pda, false),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(system_program::ID, false),
                AccountMeta::new_readonly(program_data_pda, false),
                AccountMeta::new_readonly(authority, true),
            ],
            data: instruction_data.data(),
        };

        let payer_lamports = 10_000_000_000;
        let accounts = vec![
            (
                client_state_pda,
                Account {
                    lamports: 0,
                    data: vec![],
                    owner: system_program::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
            (
                consensus_state_store_pda,
                Account {
                    lamports: 0,
                    data: vec![],
                    owner: system_program::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
            (
                app_state_pda,
                Account {
                    lamports: 0,
                    data: vec![],
                    owner: system_program::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
            (
                payer,
                Account {
                    lamports: payer_lamports,
                    data: vec![],
                    owner: system_program::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
            (
                system_program::ID,
                Account {
                    lamports: 0,
                    data: vec![],
                    owner: native_loader::ID,
                    executable: true,
                    rent_epoch: 0,
                },
            ),
            (program_data_pda, program_data_account),
            (
                authority,
                Account {
                    lamports: 1_000_000_000,
                    data: vec![],
                    owner: system_program::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
        ];

        let mollusk = Mollusk::new(&crate::ID, PROGRAM_BINARY_PATH);

        let checks = vec![
            Check::success(),
            Check::account(&client_state_pda).owner(&crate::ID).build(),
            Check::account(&consensus_state_store_pda)
                .owner(&crate::ID)
                .build(),
            Check::account(&app_state_pda).owner(&crate::ID).build(),
        ];

        let result = mollusk.process_and_validate_instruction(&instruction, &accounts, &checks);

        let payer_account = result
            .resulting_accounts
            .iter()
            .find(|(pubkey, _)| pubkey == &payer)
            .map(|(_, account)| account)
            .expect("Payer account not found");

        assert!(
            payer_account.lamports < payer_lamports,
            "Payer should have paid for account creation"
        );

        let client_state_account = result
            .resulting_accounts
            .iter()
            .find(|(pubkey, _)| pubkey == &client_state_pda)
            .map(|(_, account)| account)
            .expect("Client state account not found");

        assert!(
            client_state_account.lamports > 0,
            "Client state account should be rent-exempt"
        );
        assert!(
            client_state_account.data.len() > 8,
            "Client state account should have data"
        );

        let deserialized_client_state: ClientState =
            ClientState::try_deserialize(&mut &client_state_account.data[..])
                .expect("Failed to deserialize client state");

        assert_eq!(deserialized_client_state.chain_id, client_state.chain_id);
        assert_eq!(
            deserialized_client_state.trust_level_numerator,
            client_state.trust_level_numerator
        );
        assert_eq!(
            deserialized_client_state.trust_level_denominator,
            client_state.trust_level_denominator
        );
        assert_eq!(
            deserialized_client_state.trusting_period,
            client_state.trusting_period
        );
        assert_eq!(
            deserialized_client_state.unbonding_period,
            client_state.unbonding_period
        );
        assert_eq!(
            deserialized_client_state.max_clock_drift,
            client_state.max_clock_drift
        );
        assert_eq!(
            deserialized_client_state.frozen_height.revision_number,
            client_state.frozen_height.revision_number
        );
        assert_eq!(
            deserialized_client_state.frozen_height.revision_height,
            client_state.frozen_height.revision_height
        );
        assert_eq!(
            deserialized_client_state.latest_height.revision_number,
            client_state.latest_height.revision_number
        );
        assert_eq!(
            deserialized_client_state.latest_height.revision_height,
            client_state.latest_height.revision_height
        );

        let consensus_state_account = result
            .resulting_accounts
            .iter()
            .find(|(pubkey, _)| pubkey == &consensus_state_store_pda)
            .map(|(_, account)| account)
            .expect("Consensus state store account not found");

        assert!(
            consensus_state_account.lamports > 0,
            "Consensus state store account should be rent-exempt"
        );
        assert!(
            consensus_state_account.data.len() > 8,
            "Consensus state store account should have data"
        );

        let deserialized_consensus_store: ConsensusStateStore =
            ConsensusStateStore::try_deserialize(&mut &consensus_state_account.data[..])
                .expect("Failed to deserialize consensus state store");

        assert_eq!(deserialized_consensus_store.height, latest_height);
        assert_eq!(
            deserialized_consensus_store.consensus_state.timestamp,
            consensus_state.timestamp
        );
        assert_eq!(
            deserialized_consensus_store.consensus_state.root,
            consensus_state.root
        );
        assert_eq!(
            deserialized_consensus_store
                .consensus_state
                .next_validators_hash,
            consensus_state.next_validators_hash
        );

        // Verify app_state was created correctly
        let app_state_account = result
            .resulting_accounts
            .iter()
            .find(|(pubkey, _)| pubkey == &app_state_pda)
            .map(|(_, account)| account)
            .expect("App state account not found");

        assert!(
            app_state_account.lamports > 0,
            "App state account should be rent-exempt"
        );
        assert!(
            app_state_account.data.len() > 8,
            "App state account should have data"
        );

        let deserialized_app_state: AppState =
            AppState::try_deserialize(&mut &app_state_account.data[..])
                .expect("Failed to deserialize app state");

        assert_eq!(
            deserialized_app_state.am_state.access_manager,
            access_manager::ID
        );
    }

    #[test]
    fn test_initialize_invalid_chain_id() {
        let (client_state, consensus_state, _) = load_primary_fixtures();

        test_initialize_validation_failure(
            client_state,
            consensus_state,
            |cs| cs.chain_id = String::new(),
            ErrorCode::InvalidChainId,
        );
    }

    #[test]
    fn test_initialize_invalid_trust_level_zero_numerator() {
        let (client_state, consensus_state, _) = load_primary_fixtures();

        test_initialize_validation_failure(
            client_state,
            consensus_state,
            |cs| cs.trust_level_numerator = 0,
            ErrorCode::InvalidTrustLevel,
        );
    }

    #[test]
    fn test_initialize_invalid_trust_level_numerator_greater_than_denominator() {
        let (client_state, consensus_state, _) = load_primary_fixtures();

        test_initialize_validation_failure(
            client_state,
            consensus_state,
            |cs| {
                cs.trust_level_numerator = 5;
                cs.trust_level_denominator = 3;
            },
            ErrorCode::InvalidTrustLevel,
        );
    }

    #[test]
    fn test_initialize_invalid_trust_level_zero_denominator() {
        let (client_state, consensus_state, _) = load_primary_fixtures();

        test_initialize_validation_failure(
            client_state,
            consensus_state,
            |cs| cs.trust_level_denominator = 0,
            ErrorCode::InvalidTrustLevel,
        );
    }

    #[test]
    fn test_initialize_invalid_periods_zero_trusting_period() {
        let (client_state, consensus_state, _) = load_primary_fixtures();

        test_initialize_validation_failure(
            client_state,
            consensus_state,
            |cs| cs.trusting_period = 0,
            ErrorCode::InvalidPeriods,
        );
    }

    #[test]
    fn test_initialize_invalid_periods_zero_unbonding_period() {
        let (client_state, consensus_state, _) = load_primary_fixtures();

        test_initialize_validation_failure(
            client_state,
            consensus_state,
            |cs| cs.unbonding_period = 0,
            ErrorCode::InvalidPeriods,
        );
    }

    #[test]
    fn test_initialize_invalid_periods_trusting_greater_than_unbonding() {
        let (client_state, consensus_state, _) = load_primary_fixtures();

        test_initialize_validation_failure(
            client_state,
            consensus_state,
            |cs| {
                cs.trusting_period = 1000;
                cs.unbonding_period = 500;
            },
            ErrorCode::InvalidPeriods,
        );
    }

    #[test]
    fn test_initialize_invalid_max_clock_drift() {
        let (client_state, consensus_state, _) = load_primary_fixtures();

        test_initialize_validation_failure(
            client_state,
            consensus_state,
            |cs| cs.max_clock_drift = 0,
            ErrorCode::InvalidMaxClockDrift,
        );
    }

    #[test]
    fn test_initialize_invalid_height() {
        let (client_state, consensus_state, _) = load_primary_fixtures();

        test_initialize_validation_failure(
            client_state,
            consensus_state,
            |cs| cs.latest_height.revision_height = 0,
            ErrorCode::InvalidHeight,
        );
    }

    #[test]
    fn test_initialize_rejects_default_access_manager() {
        let (client_state, consensus_state, _) = load_primary_fixtures();

        let test_accounts = setup_test_accounts(client_state.latest_height.revision_height);

        let (program_data_pda, _) = Pubkey::find_program_address(
            &[crate::ID.as_ref()],
            &anchor_lang::solana_program::bpf_loader_upgradeable::ID,
        );

        let instruction_data = crate::instruction::Initialize {
            client_state,
            consensus_state,
            access_manager: Pubkey::default(),
        };

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(test_accounts.client_state_pda, false),
                AccountMeta::new(test_accounts.consensus_state_store_pda, false),
                AccountMeta::new(test_accounts.app_state_pda, false),
                AccountMeta::new(test_accounts.payer, true),
                AccountMeta::new_readonly(system_program::ID, false),
                AccountMeta::new_readonly(program_data_pda, false),
                AccountMeta::new_readonly(test_accounts.authority, true),
            ],
            data: instruction_data.data(),
        };

        assert_instruction_fails_with_error(
            &instruction,
            &test_accounts.accounts,
            ErrorCode::InvalidAccessManager,
        );
    }

    #[test]
    fn test_initialize_cannot_reinitialize() {
        let (client_state, consensus_state, _) = load_primary_fixtures();

        let latest_height = client_state.latest_height.revision_height;
        let payer = Pubkey::new_unique();
        let authority = Pubkey::new_unique();

        let (client_state_pda, _) =
            Pubkey::find_program_address(&[crate::types::ClientState::SEED], &crate::ID);
        let (consensus_state_store_pda, _) = Pubkey::find_program_address(
            &[
                crate::state::ConsensusStateStore::SEED,
                &latest_height.to_le_bytes(),
            ],
            &crate::ID,
        );
        let (app_state_pda, _) =
            Pubkey::find_program_address(&[crate::types::AppState::SEED], &crate::ID);
        let (program_data_pda, program_data_account) =
            crate::test_helpers::create_program_data_account(&crate::ID, Some(authority));

        let instruction_data = crate::instruction::Initialize {
            client_state,
            consensus_state,
            access_manager: access_manager::ID,
        };

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(client_state_pda, false),
                AccountMeta::new(consensus_state_store_pda, false),
                AccountMeta::new(app_state_pda, false),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(system_program::ID, false),
                AccountMeta::new_readonly(program_data_pda, false),
                AccountMeta::new_readonly(authority, true),
            ],
            data: instruction_data.data(),
        };

        // Create already-initialized accounts (owned by program, with data)
        let accounts = vec![
            (
                client_state_pda,
                Account {
                    lamports: 1_000_000,
                    data: vec![0; 256],
                    owner: crate::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
            (
                consensus_state_store_pda,
                Account {
                    lamports: 1_000_000,
                    data: vec![0; 128],
                    owner: crate::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
            (
                app_state_pda,
                Account {
                    lamports: 1_000_000,
                    data: vec![0; 128],
                    owner: crate::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
            (
                payer,
                Account {
                    lamports: 10_000_000_000,
                    data: vec![],
                    owner: system_program::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
            (
                system_program::ID,
                Account {
                    lamports: 0,
                    data: vec![],
                    owner: native_loader::ID,
                    executable: true,
                    rent_epoch: 0,
                },
            ),
            (program_data_pda, program_data_account),
            (
                authority,
                Account {
                    lamports: 1_000_000_000,
                    data: vec![],
                    owner: system_program::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
        ];

        let mollusk = Mollusk::new(&crate::ID, PROGRAM_BINARY_PATH);

        // Anchor's `init` constraint fails when account already exists
        // Error code 0 means the account is already in use
        let checks = vec![Check::err(solana_sdk::program_error::ProgramError::Custom(
            0,
        ))];

        mollusk.process_and_validate_instruction(&instruction, &accounts, &checks);
    }

    #[test]
    fn test_initialize_wrong_authority_rejected() {
        let (client_state, consensus_state, _) = load_primary_fixtures();

        let test_accounts = setup_test_accounts(client_state.latest_height.revision_height);

        let wrong_authority = Pubkey::new_unique();
        let (program_data_pda, _) = Pubkey::find_program_address(
            &[crate::ID.as_ref()],
            &anchor_lang::solana_program::bpf_loader_upgradeable::ID,
        );

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(test_accounts.client_state_pda, false),
                AccountMeta::new(test_accounts.consensus_state_store_pda, false),
                AccountMeta::new(test_accounts.app_state_pda, false),
                AccountMeta::new(test_accounts.payer, true),
                AccountMeta::new_readonly(system_program::ID, false),
                AccountMeta::new_readonly(program_data_pda, false),
                AccountMeta::new_readonly(wrong_authority, true),
            ],
            data: crate::instruction::Initialize {
                client_state,
                consensus_state,
                access_manager: access_manager::ID,
            }
            .data(),
        };

        let mut accounts = test_accounts.accounts;
        accounts.pop(); // remove the correct authority
        accounts.push((
            wrong_authority,
            Account {
                lamports: 1_000_000_000,
                data: vec![],
                owner: system_program::ID,
                executable: false,
                rent_epoch: 0,
            },
        ));

        assert_instruction_fails_with_error(
            &instruction,
            &accounts,
            ErrorCode::UnauthorizedDeployer,
        );
    }

    #[test]
    fn test_initialize_immutable_program_rejected() {
        let (client_state, consensus_state, _) = load_primary_fixtures();

        let payer = Pubkey::new_unique();
        let authority = Pubkey::new_unique();
        let (client_state_pda, _) =
            Pubkey::find_program_address(&[crate::types::ClientState::SEED], &crate::ID);
        let (consensus_state_store_pda, _) = Pubkey::find_program_address(
            &[
                crate::state::ConsensusStateStore::SEED,
                &client_state.latest_height.revision_height.to_le_bytes(),
            ],
            &crate::ID,
        );
        let (app_state_pda, _) =
            Pubkey::find_program_address(&[crate::types::AppState::SEED], &crate::ID);
        let (program_data_pda, program_data_account) =
            crate::test_helpers::create_program_data_account(&crate::ID, None);

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(client_state_pda, false),
                AccountMeta::new(consensus_state_store_pda, false),
                AccountMeta::new(app_state_pda, false),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(system_program::ID, false),
                AccountMeta::new_readonly(program_data_pda, false),
                AccountMeta::new_readonly(authority, true),
            ],
            data: crate::instruction::Initialize {
                client_state,
                consensus_state,
                access_manager: access_manager::ID,
            }
            .data(),
        };

        let accounts = vec![
            (
                client_state_pda,
                Account {
                    lamports: 0,
                    data: vec![],
                    owner: system_program::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
            (
                consensus_state_store_pda,
                Account {
                    lamports: 0,
                    data: vec![],
                    owner: system_program::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
            (
                app_state_pda,
                Account {
                    lamports: 0,
                    data: vec![],
                    owner: system_program::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
            (
                payer,
                Account {
                    lamports: 10_000_000_000,
                    data: vec![],
                    owner: system_program::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
            (
                system_program::ID,
                Account {
                    lamports: 0,
                    data: vec![],
                    owner: native_loader::ID,
                    executable: true,
                    rent_epoch: 0,
                },
            ),
            (program_data_pda, program_data_account),
            (
                authority,
                Account {
                    lamports: 1_000_000_000,
                    data: vec![],
                    owner: system_program::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
        ];

        assert_instruction_fails_with_error(
            &instruction,
            &accounts,
            ErrorCode::UnauthorizedDeployer,
        );
    }

    #[test]
    fn test_initialize_cross_program_data_rejected() {
        let (client_state, consensus_state, _) = load_primary_fixtures();

        let payer = Pubkey::new_unique();
        let authority = Pubkey::new_unique();
        let other_program_id = Pubkey::new_unique();

        let (client_state_pda, _) =
            Pubkey::find_program_address(&[crate::types::ClientState::SEED], &crate::ID);
        let (consensus_state_store_pda, _) = Pubkey::find_program_address(
            &[
                crate::state::ConsensusStateStore::SEED,
                &client_state.latest_height.revision_height.to_le_bytes(),
            ],
            &crate::ID,
        );
        let (app_state_pda, _) =
            Pubkey::find_program_address(&[crate::types::AppState::SEED], &crate::ID);
        let (wrong_program_data_pda, wrong_program_data_account) =
            crate::test_helpers::create_program_data_account(&other_program_id, Some(authority));

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(client_state_pda, false),
                AccountMeta::new(consensus_state_store_pda, false),
                AccountMeta::new(app_state_pda, false),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(system_program::ID, false),
                AccountMeta::new_readonly(wrong_program_data_pda, false),
                AccountMeta::new_readonly(authority, true),
            ],
            data: crate::instruction::Initialize {
                client_state,
                consensus_state,
                access_manager: access_manager::ID,
            }
            .data(),
        };

        let accounts = vec![
            (
                client_state_pda,
                Account {
                    lamports: 0,
                    data: vec![],
                    owner: system_program::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
            (
                consensus_state_store_pda,
                Account {
                    lamports: 0,
                    data: vec![],
                    owner: system_program::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
            (
                app_state_pda,
                Account {
                    lamports: 0,
                    data: vec![],
                    owner: system_program::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
            (
                payer,
                Account {
                    lamports: 10_000_000_000,
                    data: vec![],
                    owner: system_program::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
            (
                system_program::ID,
                Account {
                    lamports: 0,
                    data: vec![],
                    owner: native_loader::ID,
                    executable: true,
                    rent_epoch: 0,
                },
            ),
            (wrong_program_data_pda, wrong_program_data_account),
            (
                authority,
                Account {
                    lamports: 1_000_000_000,
                    data: vec![],
                    owner: system_program::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
        ];

        let mollusk = Mollusk::new(&crate::ID, PROGRAM_BINARY_PATH);
        // Anchor ConstraintSeeds = 2006
        let checks = vec![Check::err(solana_sdk::program_error::ProgramError::Custom(
            2006,
        ))];
        mollusk.process_and_validate_instruction(&instruction, &accounts, &checks);
    }
}
