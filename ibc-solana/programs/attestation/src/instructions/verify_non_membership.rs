// SPDX-License-Identifier: Apache-2.0

use crate::error::ErrorCode;
use crate::proof::deserialize_membership_proof;
use crate::state::ConsensusStateStore;
use crate::types::{ClientState, PacketAttestation};
use crate::verification::verify_attestation;
use alloy_sol_types::SolValue;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::set_return_data;
use ics25_handler::NonMembershipMsg;
use solana_keccak_hasher::{hash as keccak256, Hash};

/// Verifies that no commitment exists at the given path and height. Returns the consensus
/// timestamp via return data.
#[derive(Accounts)]
#[instruction(msg: ics25_handler::NonMembershipMsg)]
pub struct VerifyNonMembership<'info> {
    /// The attestation client state holding the trusted attestor set and frozen flag.
    #[account(
        seeds = [ClientState::SEED],
        bump,
        constraint = !client_state.is_frozen @ ErrorCode::FrozenClientState
    )]
    pub client_state: Account<'info, ClientState>,
    /// The consensus state at the requested proof height (must already exist on-chain).
    #[account(
        seeds = [ConsensusStateStore::SEED, &msg.height.to_le_bytes()],
        bump,
        constraint = consensus_state_at_height.timestamp != 0 @ ErrorCode::ConsensusTimestampNotFound
    )]
    pub consensus_state_at_height: Account<'info, ConsensusStateStore>,
}

pub fn verify_non_membership(
    ctx: Context<VerifyNonMembership>,
    msg: NonMembershipMsg,
) -> Result<()> {
    require!(msg.path.len() == 1, ErrorCode::InvalidPathLength);
    require!(msg.height > 0, ErrorCode::InvalidHeight);

    let client_state = &ctx.accounts.client_state;
    let consensus_state_store = &ctx.accounts.consensus_state_at_height;

    // Sanity check: already enforced by PDA seeds
    require!(
        msg.height == consensus_state_store.height,
        ErrorCode::HeightMismatch
    );

    require!(!client_state.is_frozen, ErrorCode::FrozenClientState);
    require!(
        consensus_state_store.timestamp != 0,
        ErrorCode::ConsensusTimestampNotFound
    );

    let proof = deserialize_membership_proof(&msg.proof)?;

    let attestation = PacketAttestation::abi_decode(&proof.attestation_data)
        .map_err(|_| error!(ErrorCode::InvalidAttestationData))?;

    require!(
        attestation.height == consensus_state_store.height,
        ErrorCode::HeightMismatch
    );

    verify_attestation(
        client_state,
        &proof.attestation_data,
        &proof.signatures,
        crate::crypto::AttestationType::Packet,
    )?;

    require!(!attestation.packets.is_empty(), ErrorCode::EmptyAttestation);

    let Hash(path_hash) = keccak256(&msg.path[0]);

    let packet = attestation
        .packets
        .iter()
        .find(|p| p.path == path_hash)
        .ok_or_else(|| error!(ErrorCode::NotMember))?;

    require!(packet.commitment == [0u8; 32], ErrorCode::NonZeroCommitment);

    // The router calls this light client via CPI and reads the returned timestamp
    // via get_return_data() to perform timeout checks against packet expiration
    let timestamp_bytes = consensus_state_store.timestamp.to_le_bytes();
    set_return_data(&timestamp_bytes);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::ConsensusStateStore;
    use crate::test_helpers::accounts::{
        create_client_state_account, create_consensus_state_account,
    };
    use crate::test_helpers::fixtures::{default_client_state, DEFAULT_TIMESTAMP};
    use crate::test_helpers::signing::TestAttestor;
    use crate::test_helpers::PROGRAM_BINARY_PATH;
    use crate::types::{ClientState, MembershipProof};
    use crate::ETH_ADDRESS_LEN;
    use anchor_lang::solana_program::instruction::{AccountMeta, Instruction};
    use anchor_lang::InstructionData;
    use borsh::BorshSerialize;
    use mollusk_svm::result::Check;
    use mollusk_svm::Mollusk;
    use solana_sdk::account::Account;
    use solana_sdk::pubkey::Pubkey;

    const HEIGHT: u64 = 100;

    struct TestAccounts {
        client_state_pda: Pubkey,
        consensus_state_pda: Pubkey,
        accounts: Vec<(Pubkey, Account)>,
    }

    fn setup_test_accounts(height: u64, client_state: ClientState, timestamp: u64) -> TestAccounts {
        let client_state_pda = ClientState::pda();
        let consensus_state_pda = ConsensusStateStore::pda(height);

        let accounts = vec![
            (client_state_pda, create_client_state_account(&client_state)),
            (
                consensus_state_pda,
                create_consensus_state_account(height, timestamp),
            ),
        ];

        TestAccounts {
            client_state_pda,
            consensus_state_pda,
            accounts,
        }
    }

    fn setup_default_test_accounts(height: u64) -> TestAccounts {
        setup_test_accounts(height, default_client_state(height), DEFAULT_TIMESTAMP)
    }

    fn setup_attestor_accounts(
        attestor_addresses: Vec<[u8; ETH_ADDRESS_LEN]>,
        min_required_sigs: u8,
    ) -> TestAccounts {
        let client_state = ClientState {
            version: crate::types::AccountVersion::V1,
            attestor_addresses,
            min_required_sigs,
            latest_height: HEIGHT,
            is_frozen: false,
        };
        setup_test_accounts(HEIGHT, client_state, DEFAULT_TIMESTAMP)
    }

    fn create_verify_non_membership_instruction(
        test_accounts: &TestAccounts,
        msg: NonMembershipMsg,
    ) -> Instruction {
        let instruction_data = crate::instruction::VerifyNonMembership { msg };

        Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new_readonly(test_accounts.client_state_pda, false),
                AccountMeta::new_readonly(test_accounts.consensus_state_pda, false),
            ],
            data: instruction_data.data(),
        }
    }

    fn expect_error(test_accounts: &TestAccounts, msg: NonMembershipMsg, error: ErrorCode) {
        let instruction = create_verify_non_membership_instruction(test_accounts, msg);
        let mollusk = Mollusk::new(&crate::ID, PROGRAM_BINARY_PATH);
        let checks = vec![Check::err(anchor_lang::error::Error::from(error).into())];
        mollusk.process_and_validate_instruction(&instruction, &test_accounts.accounts, &checks);
    }

    fn expect_success(test_accounts: &TestAccounts, msg: NonMembershipMsg) {
        let instruction = create_verify_non_membership_instruction(test_accounts, msg);
        let mollusk = Mollusk::new(&crate::ID, PROGRAM_BINARY_PATH);
        mollusk.process_and_validate_instruction(
            &instruction,
            &test_accounts.accounts,
            &[Check::success()],
        );
    }

    fn expect_bpf_crash(test_accounts: &TestAccounts, msg: NonMembershipMsg) {
        use solana_sdk::instruction::InstructionError;
        let instruction = create_verify_non_membership_instruction(test_accounts, msg);
        let mollusk = Mollusk::new(&crate::ID, PROGRAM_BINARY_PATH);
        let result = mollusk.process_instruction(&instruction, &test_accounts.accounts);
        assert_eq!(
            result.program_result,
            Err(InstructionError::ProgramFailedToComplete).into()
        );
    }

    fn build_signed_msg(
        signers: &[&TestAttestor],
        attestation_height: u64,
        packets: &[([u8; 32], [u8; 32])],
        verify_path: &[u8],
    ) -> NonMembershipMsg {
        let attestation_data =
            crate::test_helpers::fixtures::encode_packet_attestation(attestation_height, packets);
        let signatures: Vec<_> = signers
            .iter()
            .map(|a| a.sign(&attestation_data, crate::crypto::AttestationType::Packet))
            .collect();
        let proof = MembershipProof {
            attestation_data,
            signatures,
        };
        NonMembershipMsg {
            height: HEIGHT,
            proof: proof.try_to_vec().unwrap(),
            path: vec![verify_path.to_vec()],
        }
    }

    #[rstest::rstest]
    #[case::empty_path(vec![])]
    #[case::two_paths(vec![b"path1".to_vec(), b"path2".to_vec()])]
    #[case::three_paths(vec![b"path1".to_vec(), b"path2".to_vec(), b"path3".to_vec()])]
    fn test_verify_non_membership_invalid_path_length(#[case] path: Vec<Vec<u8>>) {
        let test_accounts = setup_default_test_accounts(HEIGHT);
        let msg = NonMembershipMsg {
            height: HEIGHT,
            proof: vec![],
            path,
        };
        expect_error(&test_accounts, msg, ErrorCode::InvalidPathLength);
    }

    #[test]
    fn test_verify_non_membership_frozen_client() {
        let mut client_state = default_client_state(HEIGHT);
        client_state.is_frozen = true;
        let test_accounts = setup_test_accounts(HEIGHT, client_state, DEFAULT_TIMESTAMP);
        let msg = NonMembershipMsg {
            height: HEIGHT,
            proof: vec![],
            path: vec![b"test/path".to_vec()],
        };
        expect_error(&test_accounts, msg, ErrorCode::FrozenClientState);
    }

    #[rstest::rstest]
    #[case::invalid_proof(vec![0xFF; 100], None)]
    #[case::attestation_data_too_short(
        MembershipProof { attestation_data: vec![0u8; 64], signatures: vec![vec![0u8; 65]] }.try_to_vec().unwrap(),
        Some(ErrorCode::HeightMismatch)
    )]
    fn test_verify_non_membership_rejects_bad_input(
        #[case] proof: Vec<u8>,
        #[case] expected_error: Option<ErrorCode>,
    ) {
        let test_accounts = setup_default_test_accounts(HEIGHT);
        let msg = NonMembershipMsg {
            height: HEIGHT,
            proof,
            path: vec![b"test/path".to_vec()],
        };
        match expected_error {
            Some(err) => expect_error(&test_accounts, msg, err),
            None => expect_bpf_crash(&test_accounts, msg),
        }
    }

    #[test]
    fn test_verify_non_membership_large_timestamp() {
        let test_accounts = setup_test_accounts(HEIGHT, default_client_state(HEIGHT), u64::MAX);
        let msg = NonMembershipMsg {
            height: HEIGHT,
            proof: vec![0xFF; 100],
            path: vec![b"test/path".to_vec()],
        };
        expect_bpf_crash(&test_accounts, msg);
    }

    #[test]
    fn test_verify_non_membership_height_mismatch() {
        let test_accounts = setup_default_test_accounts(HEIGHT);
        let attestation_data = crate::test_helpers::fixtures::encode_packet_attestation(200, &[]);
        let proof = MembershipProof {
            attestation_data,
            signatures: vec![],
        };
        let msg = NonMembershipMsg {
            height: HEIGHT,
            proof: proof.try_to_vec().unwrap(),
            path: vec![b"test/path".to_vec()],
        };
        expect_error(&test_accounts, msg, ErrorCode::HeightMismatch);
    }

    #[test]
    fn test_verify_non_membership_no_signatures() {
        let test_accounts = setup_default_test_accounts(HEIGHT);
        let attestation_data = crate::test_helpers::fixtures::encode_packet_attestation(
            HEIGHT,
            &[([0u8; 32], [0u8; 32])],
        );
        let proof = MembershipProof {
            attestation_data,
            signatures: vec![],
        };
        let msg = NonMembershipMsg {
            height: HEIGHT,
            proof: proof.try_to_vec().unwrap(),
            path: vec![b"test/path".to_vec()],
        };
        expect_error(&test_accounts, msg, ErrorCode::EmptySignatures);
    }

    #[test]
    fn test_verify_non_membership_empty_attestation() {
        let test_accounts = setup_attestor_accounts(vec![], 0);
        let attestation_data =
            crate::test_helpers::fixtures::encode_packet_attestation(HEIGHT, &[]);
        let proof = MembershipProof {
            attestation_data,
            signatures: vec![vec![0u8; 65]],
        };
        let msg = NonMembershipMsg {
            height: HEIGHT,
            proof: proof.try_to_vec().unwrap(),
            path: vec![b"test/path".to_vec()],
        };
        expect_error(&test_accounts, msg, ErrorCode::InvalidSignature);
    }

    #[test]
    fn test_verify_non_membership_too_few_signatures() {
        let test_accounts = setup_attestor_accounts(vec![[1u8; 20], [2u8; 20]], 2);
        let attestation_data = crate::test_helpers::fixtures::encode_packet_attestation(
            HEIGHT,
            &[([0u8; 32], [0u8; 32])],
        );
        let proof = MembershipProof {
            attestation_data,
            signatures: vec![vec![0u8; 65]],
        };
        let msg = NonMembershipMsg {
            height: HEIGHT,
            proof: proof.try_to_vec().unwrap(),
            path: vec![b"test/path".to_vec()],
        };
        expect_error(&test_accounts, msg, ErrorCode::ThresholdNotMet);
    }

    #[test]
    fn test_verify_non_membership_duplicate_signers() {
        let attestor = TestAttestor::new(1);
        let test_accounts = setup_attestor_accounts(vec![attestor.eth_address], 2);

        let attestation_data = crate::test_helpers::fixtures::encode_packet_attestation(
            HEIGHT,
            &[([0u8; 32], [0u8; 32])],
        );
        let sig = attestor.sign(&attestation_data, crate::crypto::AttestationType::Packet);
        let proof = MembershipProof {
            attestation_data,
            signatures: vec![sig.clone(), sig],
        };
        let msg = NonMembershipMsg {
            height: HEIGHT,
            proof: proof.try_to_vec().unwrap(),
            path: vec![b"test/path".to_vec()],
        };
        expect_error(&test_accounts, msg, ErrorCode::DuplicateSigner);
    }

    #[test]
    fn test_verify_non_membership_invalid_signature_length() {
        let test_accounts = setup_default_test_accounts(HEIGHT);
        let path = b"test/path";
        let Hash(path_hash) = solana_keccak_hasher::hash(path);
        let attestation_data = crate::test_helpers::fixtures::encode_packet_attestation(
            HEIGHT,
            &[(path_hash, [0u8; 32])],
        );
        let proof = MembershipProof {
            attestation_data,
            signatures: vec![vec![0u8; 64]],
        };
        let msg = NonMembershipMsg {
            height: HEIGHT,
            proof: proof.try_to_vec().unwrap(),
            path: vec![path.to_vec()],
        };
        expect_error(&test_accounts, msg, ErrorCode::InvalidSignature);
    }

    #[test]
    fn test_verify_non_membership_zero_timestamp() {
        let test_accounts = setup_test_accounts(HEIGHT, default_client_state(HEIGHT), 0);
        let msg = NonMembershipMsg {
            height: HEIGHT,
            proof: vec![],
            path: vec![b"test/path".to_vec()],
        };
        expect_error(&test_accounts, msg, ErrorCode::ConsensusTimestampNotFound);
    }

    #[test]
    fn test_verify_non_membership_happy_path() {
        let attestor = TestAttestor::new(1);
        let test_accounts = setup_attestor_accounts(vec![attestor.eth_address], 1);

        let path = b"ibc/commitments/channel-0/sequence/1";
        let Hash(path_hash) = solana_keccak_hasher::hash(path);
        let zero_commitment = [0u8; 32];

        let msg = build_signed_msg(&[&attestor], HEIGHT, &[(path_hash, zero_commitment)], path);
        expect_success(&test_accounts, msg);
    }

    #[test]
    fn test_verify_non_membership_non_zero_commitment() {
        let attestor = TestAttestor::new(1);
        let test_accounts = setup_attestor_accounts(vec![attestor.eth_address], 1);

        let path = b"ibc/commitments/channel-0/sequence/1";
        let Hash(path_hash) = solana_keccak_hasher::hash(path);
        let non_zero_commitment = [0xAB; 32];

        let msg = build_signed_msg(
            &[&attestor],
            HEIGHT,
            &[(path_hash, non_zero_commitment)],
            path,
        );
        expect_error(&test_accounts, msg, ErrorCode::NonZeroCommitment);
    }

    #[test]
    fn test_verify_non_membership_multi_attestor_quorum() {
        use crate::test_helpers::signing::create_test_attestors;

        let attestors = create_test_attestors(3);
        let addresses: Vec<_> = attestors.iter().map(|a| a.eth_address).collect();
        let test_accounts = setup_attestor_accounts(addresses, 2);

        let path = b"ibc/commitments/channel-0/sequence/1";
        let Hash(path_hash) = solana_keccak_hasher::hash(path);
        let zero_commitment = [0u8; 32];

        // Sign with 2 out of 3 attestors
        let msg = build_signed_msg(
            &[&attestors[0], &attestors[1]],
            HEIGHT,
            &[(path_hash, zero_commitment)],
            path,
        );
        expect_success(&test_accounts, msg);
    }

    #[test]
    fn test_verify_non_membership_unknown_signer() {
        let trusted_attestor = TestAttestor::new(1);
        let untrusted_attestor = TestAttestor::new(99);
        let test_accounts = setup_attestor_accounts(vec![trusted_attestor.eth_address], 1);

        let path = b"ibc/commitments/channel-0/sequence/1";
        let Hash(path_hash) = solana_keccak_hasher::hash(path);
        let zero_commitment = [0u8; 32];

        let msg = build_signed_msg(
            &[&untrusted_attestor],
            HEIGHT,
            &[(path_hash, zero_commitment)],
            path,
        );
        expect_error(&test_accounts, msg, ErrorCode::UnknownSigner);
    }

    #[test]
    fn test_verify_non_membership_path_not_in_attestation() {
        let attestor = TestAttestor::new(1);
        let test_accounts = setup_attestor_accounts(vec![attestor.eth_address], 1);

        let attested_path = b"ibc/commitments/channel-0/sequence/1";
        let Hash(attested_path_hash) = solana_keccak_hasher::hash(attested_path);

        let different_path = b"ibc/commitments/channel-0/sequence/999";
        let msg = build_signed_msg(
            &[&attestor],
            HEIGHT,
            &[(attested_path_hash, [0u8; 32])],
            different_path,
        );
        expect_error(&test_accounts, msg, ErrorCode::NotMember);
    }

    #[test]
    fn test_verify_non_membership_attestation_height_mismatch() {
        let attestor = TestAttestor::new(1);
        let test_accounts = setup_attestor_accounts(vec![attestor.eth_address], 1);

        let path = b"ibc/commitments/channel-0/sequence/1";
        let Hash(path_hash) = solana_keccak_hasher::hash(path);
        let zero_commitment = [0u8; 32];

        let msg = build_signed_msg(
            &[&attestor],
            HEIGHT + 10, // wrong attestation height
            &[(path_hash, zero_commitment)],
            path,
        );
        expect_error(&test_accounts, msg, ErrorCode::HeightMismatch);
    }

    #[test]
    fn test_verify_non_membership_more_signatures_than_required() {
        let attestors: Vec<_> = (1..=5).map(TestAttestor::new).collect();
        let addresses: Vec<_> = attestors.iter().map(|a| a.eth_address).collect();
        let test_accounts = setup_attestor_accounts(addresses, 3);

        let path = b"ibc/commitments/channel-0/sequence/1";
        let Hash(path_hash) = solana_keccak_hasher::hash(path);
        let zero_commitment = [0u8; 32];

        let attestor_refs: Vec<_> = attestors.iter().collect();
        let msg = build_signed_msg(
            &attestor_refs,
            HEIGHT,
            &[(path_hash, zero_commitment)],
            path,
        );
        expect_success(&test_accounts, msg);
    }

    #[test]
    fn test_verify_non_membership_mixed_trusted_and_unknown_signer() {
        let trusted_attestor = TestAttestor::new(1);
        let unknown_attestor = TestAttestor::new(2);
        let test_accounts = setup_attestor_accounts(vec![trusted_attestor.eth_address], 2);

        let path = b"ibc/commitments/channel-0/sequence/1";
        let Hash(path_hash) = solana_keccak_hasher::hash(path);
        let zero_commitment = [0u8; 32];

        let msg = build_signed_msg(
            &[&trusted_attestor, &unknown_attestor],
            HEIGHT,
            &[(path_hash, zero_commitment)],
            path,
        );
        expect_error(&test_accounts, msg, ErrorCode::UnknownSigner);
    }

    #[test]
    fn test_verify_non_membership_returns_timestamp() {
        let timestamp = 1_700_000_000u64;
        let attestor = TestAttestor::new(1);

        let client_state = ClientState {
            version: crate::types::AccountVersion::V1,
            attestor_addresses: vec![attestor.eth_address],
            min_required_sigs: 1,
            latest_height: HEIGHT,
            is_frozen: false,
        };

        let test_accounts = setup_test_accounts(HEIGHT, client_state, timestamp);

        let path = b"ibc/commitments/channel-0/sequence/1";
        let Hash(path_hash) = solana_keccak_hasher::hash(path);
        let zero_commitment = [0u8; 32];

        let msg = build_signed_msg(&[&attestor], HEIGHT, &[(path_hash, zero_commitment)], path);

        let instruction = create_verify_non_membership_instruction(&test_accounts, msg);
        let mollusk = Mollusk::new(&crate::ID, PROGRAM_BINARY_PATH);
        let result = mollusk.process_and_validate_instruction(
            &instruction,
            &test_accounts.accounts,
            &[Check::success()],
        );

        assert!(!result.return_data.is_empty());
        assert_eq!(result.return_data.len(), 8);
        let returned_timestamp = u64::from_le_bytes(result.return_data.try_into().unwrap());
        assert_eq!(returned_timestamp, timestamp);
    }

    #[test]
    fn test_verify_non_membership_wrong_client_state_pda() {
        let test_accounts = setup_default_test_accounts(HEIGHT);

        let wrong_client_pda = Pubkey::new_unique();
        let mut accounts = test_accounts.accounts.clone();
        accounts[0].0 = wrong_client_pda;

        let msg = NonMembershipMsg {
            height: HEIGHT,
            proof: vec![],
            path: vec![b"test/path".to_vec()],
        };

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new_readonly(wrong_client_pda, false),
                AccountMeta::new_readonly(test_accounts.consensus_state_pda, false),
            ],
            data: crate::instruction::VerifyNonMembership { msg }.data(),
        };

        let mollusk = Mollusk::new(&crate::ID, PROGRAM_BINARY_PATH);
        let checks = vec![Check::err(anchor_lang::prelude::ProgramError::Custom(
            anchor_lang::error::ErrorCode::ConstraintSeeds as u32,
        ))];
        mollusk.process_and_validate_instruction(&instruction, &accounts, &checks);
    }
}
