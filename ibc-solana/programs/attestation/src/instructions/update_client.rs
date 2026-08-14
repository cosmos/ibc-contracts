// SPDX-License-Identifier: Apache-2.0

use crate::error::ErrorCode;
use crate::events::MisbehaviourDetected;
use crate::proof::deserialize_membership_proof;
use crate::state::ConsensusStateStore;
use crate::types::{AppState, ClientState, StateAttestation};
use crate::verification::verify_attestation;
use alloy_sol_types::SolValue;
use anchor_lang::prelude::*;

/// Stores a new consensus state at the given height after verifying attestor signatures.
/// Detects misbehaviour if a conflicting timestamp already exists.
#[derive(Accounts)]
#[instruction(new_height: u64, params: UpdateClientParams)]
pub struct UpdateClient<'info> {
    /// The attestation client state (updated with `latest_height` or frozen on misbehaviour).
    #[account(
        mut,
        seeds = [ClientState::SEED],
        bump
    )]
    pub client_state: Account<'info, ClientState>,

    /// The consensus state PDA for the new height (created if needed).
    #[account(
        init_if_needed,
        payer = submitter,
        space = 8 + ConsensusStateStore::INIT_SPACE,
        seeds = [ConsensusStateStore::SEED, &new_height.to_le_bytes()],
        bump
    )]
    pub consensus_state_store: Account<'info, ConsensusStateStore>,

    /// The attestation app state linking to the access manager.
    #[account(
        seeds = [AppState::SEED],
        bump
    )]
    pub app_state: Account<'info, AppState>,

    /// The access manager account for relayer role verification.
    /// CHECK: Validated by seeds constraint using stored `access_manager` program ID
    #[account(
        seeds = [access_manager::state::AccessManager::SEED],
        bump,
        seeds::program = app_state.am_state.access_manager
    )]
    pub access_manager: AccountInfo<'info>,

    /// Instructions sysvar for CPI and role validation.
    /// CHECK: Instructions sysvar for role verification
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instructions_sysvar: AccountInfo<'info>,

    /// The relayer submitting the attestation proof (must have relayer role).
    #[account(mut)]
    pub submitter: Signer<'info>,

    /// Required for creating the consensus state account if it doesn't exist.
    pub system_program: Program<'info, System>,
}

/// Parameters for `update_client` instruction
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct UpdateClientParams {
    pub proof: Vec<u8>,
}

pub fn update_client<'info>(
    ctx: Context<'_, '_, 'info, 'info, UpdateClient<'info>>,
    new_height: u64,
    params: UpdateClientParams,
) -> Result<()> {
    access_manager::require_role(
        &ctx.accounts.access_manager,
        solana_ibc_types::roles::RELAYER_ROLE,
        &ctx.accounts.submitter,
        &ctx.accounts.instructions_sysvar,
        &crate::ID,
    )?;

    let client_state = &mut ctx.accounts.client_state;

    require!(!client_state.is_frozen, ErrorCode::FrozenClientState);

    let proof = deserialize_membership_proof(&params.proof)?;
    let attestation = StateAttestation::abi_decode(&proof.attestation_data)
        .map_err(|_| error!(ErrorCode::InvalidAttestationData))?;

    verify_attestation(
        client_state,
        &proof.attestation_data,
        &proof.signatures,
        crate::crypto::AttestationType::State,
    )?;

    require!(
        attestation.height > 0 && attestation.timestamp > 0,
        ErrorCode::InvalidState
    );

    require!(new_height == attestation.height, ErrorCode::HeightMismatch);

    let consensus_state_store = &mut ctx.accounts.consensus_state_store;

    // Return Ok on misbehaviour so the client freeze persists (errors revert state in Solana)
    let existing_timestamp = consensus_state_store.timestamp;
    if existing_timestamp != 0 {
        if existing_timestamp != attestation.timestamp {
            client_state.is_frozen = true;
            emit!(MisbehaviourDetected {
                height: attestation.height,
                existing_timestamp,
                conflicting_timestamp: attestation.timestamp,
            });
        }
        return Ok(());
    }

    if attestation.height > client_state.latest_height {
        client_state.latest_height = attestation.height;
    }

    consensus_state_store.height = attestation.height;
    consensus_state_store.timestamp = attestation.timestamp;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::ConsensusStateStore;
    use crate::test_helpers::access_control::create_access_manager_account;
    use crate::test_helpers::accounts::{
        create_app_state_account, create_client_state_account, create_consensus_state_account,
        create_empty_account, create_instructions_sysvar_account, create_payer_account,
        create_system_program_account,
    };
    use crate::test_helpers::fixtures::default_client_state;
    use crate::test_helpers::signing::TestAttestor;
    use crate::test_helpers::PROGRAM_BINARY_PATH;
    use crate::types::{AppState, ClientState, MembershipProof};
    use crate::ETH_ADDRESS_LEN;
    use anchor_lang::solana_program::instruction::{AccountMeta, Instruction};
    use anchor_lang::InstructionData;
    use borsh::BorshSerialize;
    use mollusk_svm::result::Check;
    use mollusk_svm::Mollusk;
    use solana_sdk::account::Account;
    use solana_sdk::pubkey::Pubkey;
    use solana_sdk::system_program;

    const HEIGHT: u64 = 100;
    const NEW_HEIGHT: u64 = 200;

    struct TestAccounts {
        client_state_pda: Pubkey,
        consensus_state_pda: Pubkey,
        app_state_pda: Pubkey,
        access_manager_pda: Pubkey,
        instructions_sysvar: Pubkey,
        submitter: Pubkey,
        accounts: Vec<(Pubkey, Account)>,
    }

    fn setup_test_accounts(height: u64, client_state: ClientState) -> TestAccounts {
        setup_test_accounts_with_consensus(height, client_state, None)
    }

    fn setup_test_accounts_with_consensus(
        height: u64,
        client_state: ClientState,
        existing_consensus: Option<(u64, u64)>,
    ) -> TestAccounts {
        let client_state_pda = ClientState::pda();
        let consensus_state_pda = ConsensusStateStore::pda(height);
        let app_state_pda = AppState::pda();
        let submitter = Pubkey::new_unique();

        let (access_manager_pda, access_manager_account) =
            create_access_manager_account(submitter, vec![submitter]);

        let (instructions_sysvar, instructions_sysvar_account) =
            create_instructions_sysvar_account();

        let consensus_account = existing_consensus.map_or_else(create_empty_account, |(h, t)| {
            create_consensus_state_account(h, t)
        });

        let accounts = vec![
            (client_state_pda, create_client_state_account(&client_state)),
            (app_state_pda, create_app_state_account(access_manager::ID)),
            (access_manager_pda, access_manager_account),
            (instructions_sysvar, instructions_sysvar_account),
            (consensus_state_pda, consensus_account),
            (submitter, create_payer_account()),
            (system_program::ID, create_system_program_account()),
        ];

        TestAccounts {
            client_state_pda,
            consensus_state_pda,
            app_state_pda,
            access_manager_pda,
            instructions_sysvar,
            submitter,
            accounts,
        }
    }

    fn setup_default_test_accounts(new_height: u64) -> TestAccounts {
        setup_test_accounts(new_height, default_client_state(HEIGHT))
    }

    fn setup_attestor_accounts(
        attestor_addresses: Vec<[u8; ETH_ADDRESS_LEN]>,
        min_required_sigs: u8,
        new_height: u64,
    ) -> TestAccounts {
        let client_state = ClientState {
            version: crate::types::AccountVersion::V1,
            attestor_addresses,
            min_required_sigs,
            latest_height: HEIGHT,
            is_frozen: false,
        };
        setup_test_accounts(new_height, client_state)
    }

    fn create_update_client_instruction(
        test_accounts: &TestAccounts,
        height: u64,
        params: UpdateClientParams,
    ) -> Instruction {
        let instruction_data = crate::instruction::UpdateClient {
            new_height: height,
            params,
        };

        Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(test_accounts.client_state_pda, false),
                AccountMeta::new(test_accounts.consensus_state_pda, false),
                AccountMeta::new_readonly(test_accounts.app_state_pda, false),
                AccountMeta::new_readonly(test_accounts.access_manager_pda, false),
                AccountMeta::new_readonly(test_accounts.instructions_sysvar, false),
                AccountMeta::new(test_accounts.submitter, true),
                AccountMeta::new_readonly(system_program::ID, false),
            ],
            data: instruction_data.data(),
        }
    }

    fn expect_error(test_accounts: &TestAccounts, instruction: Instruction, error: ErrorCode) {
        let mollusk = Mollusk::new(&crate::ID, PROGRAM_BINARY_PATH);
        let checks = vec![Check::err(anchor_lang::error::Error::from(error).into())];
        mollusk.process_and_validate_instruction(&instruction, &test_accounts.accounts, &checks);
    }

    fn expect_success(test_accounts: &TestAccounts, instruction: Instruction) {
        let mollusk = Mollusk::new(&crate::ID, PROGRAM_BINARY_PATH);
        mollusk.process_and_validate_instruction(
            &instruction,
            &test_accounts.accounts,
            &[Check::success()],
        );
    }

    fn expect_bpf_crash(test_accounts: &TestAccounts, instruction: Instruction) {
        use solana_sdk::instruction::InstructionError;
        let mollusk = Mollusk::new(&crate::ID, PROGRAM_BINARY_PATH);
        let result = mollusk.process_instruction(&instruction, &test_accounts.accounts);
        assert_eq!(
            result.program_result,
            Err(InstructionError::ProgramFailedToComplete).into()
        );
    }

    fn build_signed_params(
        signers: &[&TestAttestor],
        attestation_height: u64,
        timestamp: u64,
    ) -> UpdateClientParams {
        let attestation_data =
            crate::test_helpers::fixtures::encode_state_attestation(attestation_height, timestamp);
        let signatures: Vec<_> = signers
            .iter()
            .map(|a| a.sign(&attestation_data, crate::crypto::AttestationType::State))
            .collect();
        UpdateClientParams {
            proof: MembershipProof {
                attestation_data,
                signatures,
            }
            .try_to_vec()
            .unwrap(),
        }
    }

    fn make_proof_params(
        attestation_data: Vec<u8>,
        signatures: Vec<Vec<u8>>,
    ) -> UpdateClientParams {
        UpdateClientParams {
            proof: MembershipProof {
                attestation_data,
                signatures,
            }
            .try_to_vec()
            .unwrap(),
        }
    }

    #[test]
    fn test_update_client_frozen() {
        let mut client_state = default_client_state(HEIGHT);
        client_state.is_frozen = true;
        let test_accounts = setup_test_accounts(NEW_HEIGHT, client_state);

        let params = make_proof_params(vec![], vec![]);
        let instruction = create_update_client_instruction(&test_accounts, NEW_HEIGHT, params);
        expect_error(&test_accounts, instruction, ErrorCode::FrozenClientState);
    }

    #[rstest::rstest]
    #[case::invalid_proof(vec![0xFF; 100], None)]
    #[case::attestation_data_too_short(
        MembershipProof { attestation_data: vec![0u8; 64], signatures: vec![vec![0u8; 65]] }.try_to_vec().unwrap(),
        Some(ErrorCode::InvalidSignature)
    )]
    fn test_update_client_rejects_bad_proof(
        #[case] proof: Vec<u8>,
        #[case] expected_error: Option<ErrorCode>,
    ) {
        let test_accounts = setup_default_test_accounts(NEW_HEIGHT);
        let params = UpdateClientParams { proof };
        let instruction = create_update_client_instruction(&test_accounts, NEW_HEIGHT, params);
        match expected_error {
            Some(err) => expect_error(&test_accounts, instruction, err),
            None => expect_bpf_crash(&test_accounts, instruction),
        }
    }

    #[test]
    fn test_update_client_no_signatures() {
        let test_accounts = setup_default_test_accounts(NEW_HEIGHT);
        let attestation_data =
            crate::test_helpers::fixtures::encode_state_attestation(NEW_HEIGHT, 1_700_000_000);
        let params = make_proof_params(attestation_data, vec![]);
        let instruction = create_update_client_instruction(&test_accounts, NEW_HEIGHT, params);
        expect_error(&test_accounts, instruction, ErrorCode::EmptySignatures);
    }

    #[test]
    fn test_update_client_too_few_signatures() {
        let test_accounts = setup_attestor_accounts(vec![[1u8; 20], [2u8; 20]], 2, NEW_HEIGHT);
        let attestation_data =
            crate::test_helpers::fixtures::encode_state_attestation(NEW_HEIGHT, 1_700_000_000);
        let params = make_proof_params(attestation_data, vec![vec![0u8; 65]]);
        let instruction = create_update_client_instruction(&test_accounts, NEW_HEIGHT, params);
        expect_error(&test_accounts, instruction, ErrorCode::ThresholdNotMet);
    }

    #[test]
    fn test_update_client_duplicate_signers() {
        let attestor = TestAttestor::new(1);
        let test_accounts = setup_attestor_accounts(vec![attestor.eth_address], 2, NEW_HEIGHT);

        let attestation_data =
            crate::test_helpers::fixtures::encode_state_attestation(NEW_HEIGHT, 1_700_000_000);
        let sig = attestor.sign(&attestation_data, crate::crypto::AttestationType::State);
        let params = make_proof_params(attestation_data, vec![sig.clone(), sig]);
        let instruction = create_update_client_instruction(&test_accounts, NEW_HEIGHT, params);
        expect_error(&test_accounts, instruction, ErrorCode::DuplicateSigner);
    }

    #[test]
    fn test_update_client_empty_proof_bytes() {
        let test_accounts = setup_default_test_accounts(NEW_HEIGHT);
        let params = UpdateClientParams { proof: vec![] };
        let instruction = create_update_client_instruction(&test_accounts, NEW_HEIGHT, params);
        expect_error(&test_accounts, instruction, ErrorCode::InvalidProof);
    }

    #[test]
    fn test_update_client_invalid_signature_length() {
        let test_accounts = setup_default_test_accounts(NEW_HEIGHT);
        let attestation_data =
            crate::test_helpers::fixtures::encode_state_attestation(NEW_HEIGHT, 1_700_000_000);
        let params = make_proof_params(attestation_data, vec![vec![0u8; 64]]);
        let instruction = create_update_client_instruction(&test_accounts, NEW_HEIGHT, params);
        expect_error(&test_accounts, instruction, ErrorCode::InvalidSignature);
    }

    #[test]
    fn test_update_client_max_attestors() {
        let addresses: Vec<[u8; ETH_ADDRESS_LEN]> = (0..10).map(|i| [i as u8; 20]).collect();
        let test_accounts = setup_attestor_accounts(addresses, 7, NEW_HEIGHT);
        let attestation_data =
            crate::test_helpers::fixtures::encode_state_attestation(NEW_HEIGHT, 1_700_000_000);
        let params = make_proof_params(
            attestation_data,
            vec![
                vec![1u8; 65],
                vec![2u8; 65],
                vec![3u8; 65],
                vec![4u8; 65],
                vec![5u8; 65],
            ],
        );
        let instruction = create_update_client_instruction(&test_accounts, NEW_HEIGHT, params);
        expect_error(&test_accounts, instruction, ErrorCode::ThresholdNotMet);
    }

    #[rstest::rstest]
    #[case::same_height(HEIGHT)]
    #[case::lower_height(50)]
    #[case::max_height(u64::MAX)]
    fn test_update_client_default_accounts_bad_height(#[case] height: u64) {
        let test_accounts = setup_test_accounts(height, default_client_state(HEIGHT));
        let attestation_data =
            crate::test_helpers::fixtures::encode_state_attestation(height, 1_700_000_000);
        let params = make_proof_params(attestation_data, vec![vec![0u8; 65]]);
        let instruction = create_update_client_instruction(&test_accounts, height, params);
        expect_error(&test_accounts, instruction, ErrorCode::InvalidSignature);
    }

    #[test]
    fn test_update_client_misbehaviour_different_timestamp() {
        let existing_timestamp = 1_700_000_000u64;
        let conflicting_timestamp = 1_800_000_000u64;
        let attestor = TestAttestor::new(1);

        let test_accounts = setup_test_accounts_with_consensus(
            HEIGHT,
            ClientState {
                version: crate::types::AccountVersion::V1,
                attestor_addresses: vec![attestor.eth_address],
                min_required_sigs: 1,
                latest_height: HEIGHT,
                is_frozen: false,
            },
            Some((HEIGHT, existing_timestamp)),
        );

        let params = build_signed_params(&[&attestor], HEIGHT, conflicting_timestamp);
        let instruction = create_update_client_instruction(&test_accounts, HEIGHT, params);

        let mollusk = Mollusk::new(&crate::ID, PROGRAM_BINARY_PATH);
        let result = mollusk.process_and_validate_instruction(
            &instruction,
            &test_accounts.accounts,
            &[Check::success()],
        );

        let client_state_account = result
            .get_account(&test_accounts.client_state_pda)
            .expect("Client state account should exist");
        let client_state: ClientState =
            anchor_lang::AccountDeserialize::try_deserialize(&mut &client_state_account.data[..])
                .expect("Failed to deserialize client state");
        assert!(
            client_state.is_frozen,
            "Client state should be frozen after misbehaviour detection"
        );
    }

    #[test]
    fn test_update_client_same_height_same_timestamp_noop() {
        let timestamp = 1_700_000_000u64;
        let attestor = TestAttestor::new(1);

        let test_accounts = setup_test_accounts_with_consensus(
            HEIGHT,
            ClientState {
                version: crate::types::AccountVersion::V1,
                attestor_addresses: vec![attestor.eth_address],
                min_required_sigs: 1,
                latest_height: HEIGHT,
                is_frozen: false,
            },
            Some((HEIGHT, timestamp)),
        );

        let params = build_signed_params(&[&attestor], HEIGHT, timestamp);
        let instruction = create_update_client_instruction(&test_accounts, HEIGHT, params);

        let mollusk = Mollusk::new(&crate::ID, PROGRAM_BINARY_PATH);
        let result = mollusk.process_and_validate_instruction(
            &instruction,
            &test_accounts.accounts,
            &[Check::success()],
        );

        let client_state_account = result
            .get_account(&test_accounts.client_state_pda)
            .expect("Client state account should exist");
        let client_state: ClientState =
            anchor_lang::AccountDeserialize::try_deserialize(&mut &client_state_account.data[..])
                .expect("Failed to deserialize client state");
        assert!(
            !client_state.is_frozen,
            "Client state should NOT be frozen for same timestamp"
        );
    }

    #[test]
    fn test_update_client_happy_path_new_height() {
        let attestor = TestAttestor::new(1);
        let test_accounts = setup_attestor_accounts(vec![attestor.eth_address], 1, NEW_HEIGHT);

        let new_timestamp = 1_800_000_000u64;
        let params = build_signed_params(&[&attestor], NEW_HEIGHT, new_timestamp);
        let instruction = create_update_client_instruction(&test_accounts, NEW_HEIGHT, params);

        let mollusk = Mollusk::new(&crate::ID, PROGRAM_BINARY_PATH);
        let result = mollusk.process_and_validate_instruction(
            &instruction,
            &test_accounts.accounts,
            &[Check::success()],
        );

        let client_state_account = result
            .get_account(&test_accounts.client_state_pda)
            .expect("Client state account should exist");
        let client_state: ClientState =
            anchor_lang::AccountDeserialize::try_deserialize(&mut &client_state_account.data[..])
                .expect("Failed to deserialize client state");
        assert_eq!(client_state.latest_height, NEW_HEIGHT);
        assert!(!client_state.is_frozen);

        let consensus_state_account = result
            .get_account(&test_accounts.consensus_state_pda)
            .expect("Consensus state account should exist");
        let consensus_state_store: crate::state::ConsensusStateStore =
            anchor_lang::AccountDeserialize::try_deserialize(
                &mut &consensus_state_account.data[..],
            )
            .expect("Failed to deserialize consensus state");
        assert_eq!(consensus_state_store.height, NEW_HEIGHT);
        assert_eq!(consensus_state_store.timestamp, new_timestamp);
    }

    #[test]
    fn test_update_client_height_mismatch() {
        let attestor = TestAttestor::new(1);
        let test_accounts = setup_attestor_accounts(vec![attestor.eth_address], 1, NEW_HEIGHT);

        // PDA seed uses NEW_HEIGHT (200), but attestation contains a different height (150)
        let params = build_signed_params(&[&attestor], 150, 1_800_000_000);
        let instruction = create_update_client_instruction(&test_accounts, NEW_HEIGHT, params);
        expect_error(&test_accounts, instruction, ErrorCode::HeightMismatch);
    }

    #[rstest::rstest]
    #[case::zero_height(0, 1_700_000_000)]
    #[case::zero_timestamp(NEW_HEIGHT, 0)]
    #[case::zero_height_and_timestamp(0, 0)]
    fn test_update_client_invalid_state(#[case] attestation_height: u64, #[case] timestamp: u64) {
        let attestor = TestAttestor::new(1);
        let test_accounts = setup_attestor_accounts(vec![attestor.eth_address], 1, NEW_HEIGHT);
        let params = build_signed_params(&[&attestor], attestation_height, timestamp);
        let instruction = create_update_client_instruction(&test_accounts, NEW_HEIGHT, params);
        expect_error(&test_accounts, instruction, ErrorCode::InvalidState);
    }

    #[test]
    fn test_update_client_lower_height_does_not_update_latest() {
        let attestor = TestAttestor::new(1);
        let lower_height = 50u64;
        let test_accounts = setup_attestor_accounts(vec![attestor.eth_address], 1, lower_height);

        let new_timestamp = 1_600_000_000u64;
        let params = build_signed_params(&[&attestor], lower_height, new_timestamp);
        let instruction = create_update_client_instruction(&test_accounts, lower_height, params);

        let mollusk = Mollusk::new(&crate::ID, PROGRAM_BINARY_PATH);
        let result = mollusk.process_and_validate_instruction(
            &instruction,
            &test_accounts.accounts,
            &[Check::success()],
        );

        let client_state_account = result
            .get_account(&test_accounts.client_state_pda)
            .expect("Client state account should exist");
        let client_state: ClientState =
            anchor_lang::AccountDeserialize::try_deserialize(&mut &client_state_account.data[..])
                .expect("Failed to deserialize client state");
        assert_eq!(
            client_state.latest_height, HEIGHT,
            "latest_height should remain unchanged"
        );

        let consensus_state_account = result
            .get_account(&test_accounts.consensus_state_pda)
            .expect("Consensus state account should exist");
        let consensus_state_store: crate::state::ConsensusStateStore =
            anchor_lang::AccountDeserialize::try_deserialize(
                &mut &consensus_state_account.data[..],
            )
            .expect("Failed to deserialize consensus state");
        assert_eq!(consensus_state_store.height, lower_height);
        assert_eq!(consensus_state_store.timestamp, new_timestamp);
    }

    #[test]
    fn test_update_client_multi_attestor_quorum() {
        use crate::test_helpers::signing::create_test_attestors;

        let attestors = create_test_attestors(3);
        let addresses: Vec<_> = attestors.iter().map(|a| a.eth_address).collect();
        let test_accounts = setup_attestor_accounts(addresses, 2, NEW_HEIGHT);

        // Sign with 2 out of 3 attestors
        let params =
            build_signed_params(&[&attestors[0], &attestors[2]], NEW_HEIGHT, 1_800_000_000);
        let instruction = create_update_client_instruction(&test_accounts, NEW_HEIGHT, params);
        expect_success(&test_accounts, instruction);
    }

    #[test]
    fn test_update_client_unknown_signer() {
        let trusted_attestor = TestAttestor::new(1);
        let untrusted_attestor = TestAttestor::new(99);
        let test_accounts =
            setup_attestor_accounts(vec![trusted_attestor.eth_address], 1, NEW_HEIGHT);

        let params = build_signed_params(&[&untrusted_attestor], NEW_HEIGHT, 1_800_000_000);
        let instruction = create_update_client_instruction(&test_accounts, NEW_HEIGHT, params);
        expect_error(&test_accounts, instruction, ErrorCode::UnknownSigner);
    }

    #[test]
    fn test_update_client_more_signatures_than_required() {
        let attestors: Vec<_> = (1..=5).map(TestAttestor::new).collect();
        let addresses: Vec<_> = attestors.iter().map(|a| a.eth_address).collect();
        let test_accounts = setup_attestor_accounts(addresses, 3, NEW_HEIGHT);

        let attestor_refs: Vec<_> = attestors.iter().collect();
        let params = build_signed_params(&attestor_refs, NEW_HEIGHT, 1_800_000_000);
        let instruction = create_update_client_instruction(&test_accounts, NEW_HEIGHT, params);
        expect_success(&test_accounts, instruction);
    }

    #[test]
    fn test_update_client_mixed_trusted_and_unknown_signer() {
        let trusted_attestor = TestAttestor::new(1);
        let unknown_attestor = TestAttestor::new(2);
        let test_accounts =
            setup_attestor_accounts(vec![trusted_attestor.eth_address], 2, NEW_HEIGHT);

        let params = build_signed_params(
            &[&trusted_attestor, &unknown_attestor],
            NEW_HEIGHT,
            1_800_000_000,
        );
        let instruction = create_update_client_instruction(&test_accounts, NEW_HEIGHT, params);
        expect_error(&test_accounts, instruction, ErrorCode::UnknownSigner);
    }

    #[test]
    fn test_update_client_wrong_client_state_pda() {
        let mut test_accounts = setup_default_test_accounts(NEW_HEIGHT);

        let wrong_client_pda = Pubkey::new_unique();
        if let Some(entry) = test_accounts
            .accounts
            .iter_mut()
            .find(|(k, _)| *k == test_accounts.client_state_pda)
        {
            entry.0 = wrong_client_pda;
        }
        test_accounts.client_state_pda = wrong_client_pda;

        let params = make_proof_params(vec![], vec![]);
        let instruction = create_update_client_instruction(&test_accounts, NEW_HEIGHT, params);

        let mollusk = Mollusk::new(&crate::ID, PROGRAM_BINARY_PATH);
        let checks = vec![Check::err(anchor_lang::prelude::ProgramError::Custom(
            anchor_lang::error::ErrorCode::ConstraintSeeds as u32,
        ))];
        mollusk.process_and_validate_instruction(&instruction, &test_accounts.accounts, &checks);
    }
}
