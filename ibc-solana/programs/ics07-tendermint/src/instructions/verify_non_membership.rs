// SPDX-License-Identifier: Apache-2.0

use crate::error::ErrorCode;
use crate::helpers::deserialize_merkle_proof;
use crate::state::ConsensusStateStore;
use crate::types::ClientState;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::set_return_data;
use ics25_handler::NonMembershipMsg;
use tendermint_light_client_membership::KVPair;

/// Verifies that a key does not exist in the Tendermint state at a given height using an absence proof.
#[derive(Accounts)]
#[instruction(msg: ics25_handler::NonMembershipMsg)]
pub struct VerifyNonMembership<'info> {
    /// PDA holding the light client configuration; used to check the frozen status.
    #[account(
        seeds = [ClientState::SEED],
        bump
    )]
    pub client_state: Account<'info, ClientState>,
    /// PDA storing the verified consensus state at the requested proof height.
    #[account(
        seeds = [ConsensusStateStore::SEED, &msg.height.to_le_bytes()],
        bump
    )]
    pub consensus_state_at_height: Account<'info, ConsensusStateStore>,
}

pub fn verify_non_membership(
    ctx: Context<VerifyNonMembership>,
    msg: NonMembershipMsg,
) -> Result<()> {
    let client_state = &ctx.accounts.client_state;
    let consensus_state_store = &ctx.accounts.consensus_state_at_height;

    // Sanity check: already enforced by PDA seeds
    require!(
        msg.height == consensus_state_store.height,
        ErrorCode::HeightMismatch
    );
    require!(!client_state.is_frozen(), ErrorCode::ClientFrozen);

    let proof = deserialize_merkle_proof(&msg.proof)?;
    let kv_pair = KVPair::new(msg.path, vec![]);
    let app_hash = consensus_state_store.consensus_state.root;

    tendermint_light_client_membership::membership(app_hash, std::iter::once((kv_pair, proof)))
        .map_err(|_| error!(ErrorCode::NonMembershipVerificationFailed))?;

    let timestamp_secs = crate::nanos_to_secs(consensus_state_store.consensus_state.timestamp);
    let timestamp_bytes = timestamp_secs.to_le_bytes();

    set_return_data(&timestamp_bytes);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::ConsensusStateStore;
    use crate::test_helpers::chunk_test_utils::{
        derive_client_state_pda, derive_consensus_state_pda,
    };
    use crate::test_helpers::fixtures::*;
    use crate::test_helpers::PROGRAM_BINARY_PATH;
    use crate::types::{ClientState, ConsensusState, IbcHeight};
    use anchor_lang::solana_program::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
    };
    use anchor_lang::AccountSerialize;
    use anchor_lang::InstructionData;
    use ics25_handler::MembershipMsg;
    use mollusk_svm::result::Check;
    use mollusk_svm::Mollusk;
    use solana_sdk::account::Account;

    struct TestAccounts {
        client_state_pda: Pubkey,
        consensus_state_pda: Pubkey,
        accounts: Vec<(Pubkey, Account)>,
    }

    fn setup_test_accounts(
        height: u64,
        client_state: ClientState,
        consensus_state: ConsensusState,
    ) -> TestAccounts {
        let client_state_pda = derive_client_state_pda();
        let consensus_state_pda = derive_consensus_state_pda(height);

        let mut client_data = vec![];
        client_state.try_serialize(&mut client_data).unwrap();

        let consensus_state_store = ConsensusStateStore {
            height,
            consensus_state,
        };

        let mut consensus_data = vec![];
        consensus_state_store
            .try_serialize(&mut consensus_data)
            .unwrap();

        let accounts = vec![
            (
                client_state_pda,
                Account {
                    lamports: 1_000_000,
                    data: client_data,
                    owner: crate::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
            (
                consensus_state_pda,
                Account {
                    lamports: 1_000_000,
                    data: consensus_data,
                    owner: crate::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
        ];

        TestAccounts {
            client_state_pda,
            consensus_state_pda,
            accounts,
        }
    }

    fn create_membership_msg(fixture: &MembershipMsgFixture) -> MembershipMsg {
        let path: Vec<Vec<u8>> = fixture.path.iter().map(|s| s.as_bytes().to_vec()).collect();
        let proof = hex_to_bytes(&fixture.proof);
        let value = hex_to_bytes(&fixture.value);

        MembershipMsg {
            height: fixture.height,
            proof,
            path,
            value,
        }
    }

    fn create_non_membership_msg(fixture: &MembershipMsgFixture) -> NonMembershipMsg {
        let path: Vec<Vec<u8>> = fixture.path.iter().map(|s| s.as_bytes().to_vec()).collect();
        let proof = hex_to_bytes(&fixture.proof);

        NonMembershipMsg {
            height: fixture.height,
            proof,
            path,
        }
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

    fn setup_non_membership_test() -> (
        MembershipVerificationFixture,
        TestAccounts,
        NonMembershipMsg,
    ) {
        let fixture = load_membership_verification_fixture("verify_non-membership_key_1");
        let client_state = decode_client_state_from_hex(&fixture.client_state_hex);
        let consensus_state = decode_consensus_state_from_hex(&fixture.consensus_state_hex);

        let test_accounts =
            setup_test_accounts(fixture.membership_msg.height, client_state, consensus_state);

        let msg = create_non_membership_msg(&fixture.membership_msg);

        (fixture, test_accounts, msg)
    }

    #[test]
    fn test_verify_non_membership_happy_path() {
        let (_fixture, test_accounts, msg) = setup_non_membership_test();

        let instruction = create_verify_non_membership_instruction(&test_accounts, msg);

        let mollusk = Mollusk::new(&crate::ID, PROGRAM_BINARY_PATH);
        let checks = vec![Check::success()];
        mollusk.process_and_validate_instruction(&instruction, &test_accounts.accounts, &checks);
    }

    #[test]
    fn test_verify_non_membership_with_value() {
        // NonMembershipMsg doesn't have a value field by design.
        // This test verifies that non-membership proofs work correctly without values.
        let (_fixture, test_accounts, msg) = setup_non_membership_test();

        let instruction = create_verify_non_membership_instruction(&test_accounts, msg);

        let mollusk = Mollusk::new(&crate::ID, PROGRAM_BINARY_PATH);
        let checks = vec![Check::success()];
        mollusk.process_and_validate_instruction(&instruction, &test_accounts.accounts, &checks);
    }

    #[test]
    fn test_verify_non_membership_wrong_app_hash() {
        let fixture = load_membership_verification_fixture("verify_non-membership_key_1");
        let client_state = decode_client_state_from_hex(&fixture.client_state_hex);
        let mut consensus_state = decode_consensus_state_from_hex(&fixture.consensus_state_hex);

        consensus_state.root = [0xFF; 32];

        let test_accounts =
            setup_test_accounts(fixture.membership_msg.height, client_state, consensus_state);

        let msg = create_non_membership_msg(&fixture.membership_msg);
        let instruction = create_verify_non_membership_instruction(&test_accounts, msg);

        let mollusk = Mollusk::new(&crate::ID, PROGRAM_BINARY_PATH);
        let checks = vec![Check::err(
            anchor_lang::error::Error::from(ErrorCode::NonMembershipVerificationFailed).into(),
        )];
        mollusk.process_and_validate_instruction(&instruction, &test_accounts.accounts, &checks);
    }

    #[test]
    fn test_verify_non_membership_wrong_path() {
        let (_fixture, test_accounts, mut msg) = setup_non_membership_test();
        msg.path = vec![b"wrong".to_vec(), b"path".to_vec()];

        let instruction = create_verify_non_membership_instruction(&test_accounts, msg);

        let mollusk = Mollusk::new(&crate::ID, PROGRAM_BINARY_PATH);
        let checks = vec![Check::err(
            anchor_lang::error::Error::from(ErrorCode::NonMembershipVerificationFailed).into(),
        )];
        mollusk.process_and_validate_instruction(&instruction, &test_accounts.accounts, &checks);
    }

    #[test]
    fn test_verify_non_membership_invalid_proof() {
        let (_fixture, test_accounts, mut msg) = setup_non_membership_test();
        msg.proof = vec![0xFF; 100];

        let instruction = create_verify_non_membership_instruction(&test_accounts, msg);

        let mollusk = Mollusk::new(&crate::ID, PROGRAM_BINARY_PATH);
        let checks = vec![Check::err(
            anchor_lang::error::Error::from(ErrorCode::InvalidProof).into(),
        )];
        mollusk.process_and_validate_instruction(&instruction, &test_accounts.accounts, &checks);
    }

    #[test]
    fn test_verify_non_membership_frozen_client() {
        let fixture = load_membership_verification_fixture("verify_non-membership_key_1");
        let mut client_state = decode_client_state_from_hex(&fixture.client_state_hex);
        let consensus_state = decode_consensus_state_from_hex(&fixture.consensus_state_hex);

        client_state.frozen_height = IbcHeight {
            revision_number: 0,
            revision_height: 1,
        };

        let test_accounts =
            setup_test_accounts(fixture.membership_msg.height, client_state, consensus_state);

        let msg = create_non_membership_msg(&fixture.membership_msg);
        let instruction = create_verify_non_membership_instruction(&test_accounts, msg);

        let mollusk = Mollusk::new(&crate::ID, PROGRAM_BINARY_PATH);
        let checks = vec![Check::err(
            anchor_lang::error::Error::from(ErrorCode::ClientFrozen).into(),
        )];
        mollusk.process_and_validate_instruction(&instruction, &test_accounts.accounts, &checks);
    }

    #[test]
    fn test_verify_non_membership_wrong_height_pda() {
        let fixture = load_membership_verification_fixture("verify_non-membership_key_1");
        let client_state = decode_client_state_from_hex(&fixture.client_state_hex);
        let consensus_state = decode_consensus_state_from_hex(&fixture.consensus_state_hex);

        let actual_height = fixture.membership_msg.height;
        let wrong_height = actual_height + 100;

        let test_accounts = setup_test_accounts(actual_height, client_state, consensus_state);

        let mut msg = create_non_membership_msg(&fixture.membership_msg);
        msg.height = wrong_height;

        let instruction = create_verify_non_membership_instruction(&test_accounts, msg);

        let mollusk = Mollusk::new(&crate::ID, PROGRAM_BINARY_PATH);
        let checks = vec![Check::err(anchor_lang::prelude::ProgramError::Custom(
            anchor_lang::error::ErrorCode::ConstraintSeeds as u32,
        ))];
        mollusk.process_and_validate_instruction(&instruction, &test_accounts.accounts, &checks);
    }

    #[test]
    fn test_verify_non_membership_nonexistent_height() {
        let fixture = load_membership_verification_fixture("verify_non-membership_key_1");
        let client_state = decode_client_state_from_hex(&fixture.client_state_hex);

        let existing_height = fixture.membership_msg.height;
        let nonexistent_height = existing_height + 999;

        let client_state_pda = derive_client_state_pda();

        let mut client_data = vec![];
        client_state.try_serialize(&mut client_data).unwrap();

        let nonexistent_consensus_pda = derive_consensus_state_pda(nonexistent_height);

        let accounts = vec![
            (
                client_state_pda,
                Account {
                    lamports: 1_000_000,
                    data: client_data,
                    owner: crate::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
            (
                nonexistent_consensus_pda,
                Account {
                    lamports: 0,
                    data: vec![],
                    owner: solana_sdk::system_program::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
        ];

        let mut msg = create_non_membership_msg(&fixture.membership_msg);
        msg.height = nonexistent_height;

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new_readonly(client_state_pda, false),
                AccountMeta::new_readonly(nonexistent_consensus_pda, false),
            ],
            data: crate::instruction::VerifyNonMembership { msg }.data(),
        };

        let mollusk = Mollusk::new(&crate::ID, PROGRAM_BINARY_PATH);
        let checks = vec![Check::err(anchor_lang::prelude::ProgramError::Custom(3012))];
        mollusk.process_and_validate_instruction(&instruction, &accounts, &checks);
    }

    #[test]
    fn test_verify_non_membership_wrong_client_state_pda() {
        let (_fixture, test_accounts, msg) = setup_non_membership_test();

        let wrong_client_pda = Pubkey::new_unique();
        let mut accounts = test_accounts.accounts.clone();
        accounts[0].0 = wrong_client_pda;

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
