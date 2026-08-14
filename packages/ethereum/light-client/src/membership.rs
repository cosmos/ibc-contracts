// SPDX-License-Identifier: Apache-2.0

//! This module provides [`verify_membership`] function to verify the membership of a key in the
//! storage trie.

use alloy_primitives::{keccak256, Keccak256, U256};
use ethereum_trie_db::trie_db::{
    verify_account_storage_root, verify_storage_exclusion_proof, verify_storage_inclusion_proof,
};
use ethereum_types::execution::{account_proof::AccountProof, storage_proof::StorageProof};
use serde::{Deserialize, Serialize};

use crate::{client_state::ClientState, consensus_state::ConsensusState, error::EthereumIBCError};

/// The membership proof for the (non-)membership of a key in the execution state root.
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug, Default)]
pub struct MembershipProof {
    /// The inclusion proof of the contract's storage root in the account trie.
    pub account_proof: AccountProof,
    /// The inclusion/exculsion proof in the contract's storage trie.
    pub storage_proof: StorageProof,
}

/// Verifies the membership of a key in the storage trie.
/// # Errors
/// Returns an error if the proof cannot be verified.
#[allow(clippy::module_name_repetitions, clippy::needless_pass_by_value)]
pub fn verify_membership(
    trusted_consensus_state: ConsensusState,
    client_state: ClientState,
    proof: Vec<u8>,
    path: Vec<Vec<u8>>,
    raw_value: Vec<u8>,
) -> Result<(), EthereumIBCError> {
    let membership_proof: MembershipProof = serde_json::from_slice(proof.as_slice())
        .map_err(|_| EthereumIBCError::StorageProofDecode)?;

    // Verify the account proof first
    verify_account_storage_root(
        trusted_consensus_state.state_root,
        client_state.ibc_contract_address,
        &membership_proof.account_proof.proof,
        membership_proof.account_proof.storage_root,
    )
    .map_err(|err| EthereumIBCError::VerifyStorageProof(err.to_string()))?;

    // Verify the storage proof
    let verified_storage_root = membership_proof.account_proof.storage_root;

    check_commitment_path(
        &path,
        client_state.ibc_commitment_slot,
        membership_proof.storage_proof.key.into(),
    )?;

    ensure!(
        membership_proof.storage_proof.value.to_be_bytes_vec() == raw_value,
        EthereumIBCError::StoredValueMistmatch {
            expected: raw_value,
            actual: membership_proof.storage_proof.value.to_be_bytes_vec(),
        }
    );

    let rlp_value = alloy_rlp::encode_fixed_size(&membership_proof.storage_proof.value);
    verify_storage_inclusion_proof(
        &verified_storage_root,
        &membership_proof.storage_proof.key,
        &rlp_value,
        membership_proof.storage_proof.proof.iter(),
    )
    .map_err(|err| EthereumIBCError::VerifyStorageProof(err.to_string()))
}

/// Verifies the non-membership of a key in the storage trie.
/// # Errors
/// Returns an error if the proof cannot be verified.
#[allow(clippy::module_name_repetitions, clippy::needless_pass_by_value)]
pub fn verify_non_membership(
    trusted_consensus_state: ConsensusState,
    client_state: ClientState,
    proof: Vec<u8>,
    path: Vec<Vec<u8>>,
) -> Result<(), EthereumIBCError> {
    let membership_proof: MembershipProof = serde_json::from_slice(proof.as_slice())
        .map_err(|_| EthereumIBCError::StorageProofDecode)?;

    // Verify the account proof first
    verify_account_storage_root(
        trusted_consensus_state.state_root,
        client_state.ibc_contract_address,
        &membership_proof.account_proof.proof,
        membership_proof.account_proof.storage_root,
    )
    .map_err(|err| EthereumIBCError::VerifyStorageProof(err.to_string()))?;

    // Verify the storage proof
    let verified_storage_root = membership_proof.account_proof.storage_root;

    check_commitment_path(
        &path,
        client_state.ibc_commitment_slot,
        membership_proof.storage_proof.key.into(),
    )?;

    ensure!(
        membership_proof.storage_proof.value.is_zero(),
        EthereumIBCError::StoredValueMistmatch {
            expected: vec![0],
            actual: membership_proof.storage_proof.value.to_be_bytes_vec(),
        }
    );

    verify_storage_exclusion_proof(
        &verified_storage_root,
        &membership_proof.storage_proof.key,
        membership_proof.storage_proof.proof.iter(),
    )
    .map_err(|err| EthereumIBCError::VerifyStorageProof(err.to_string()))
}

fn check_commitment_path(
    path: &[Vec<u8>],
    ibc_commitment_slot: U256,
    key: U256,
) -> Result<(), EthereumIBCError> {
    ensure!(
        path.len() == 1,
        EthereumIBCError::InvalidPathLength {
            expected: 1,
            found: path.len()
        }
    );

    let expected_commitment_path = evm_ics26_commitment_path(&path[0], ibc_commitment_slot);
    ensure!(
        expected_commitment_path == key,
        EthereumIBCError::InvalidCommitmentKey(
            format!("0x{expected_commitment_path:x}"),
            format!("0x{key:x}"),
        )
    );

    Ok(())
}

// TODO: Unit test
/// Computes the commitment key for a given path and slot.
#[must_use = "calculating the commitment path has no effect"]
pub fn evm_ics26_commitment_path(ibc_path: &[u8], slot: U256) -> U256 {
    let path_hash = keccak256(ibc_path);

    let mut hasher = Keccak256::new();
    hasher.update(path_hash);
    hasher.update(slot.to_be_bytes_vec());

    hasher.finalize().into()
}

#[cfg(test)]
mod test {
    use crate::{
        header::Header,
        test_utils::fixtures::{self, get_packet_paths, InitialState, RelayerMessages},
        update::update_consensus_state,
    };

    use ibc_proto_eureka::ibc::lightclients::wasm::v1::ClientMessage;

    use prost::Message;

    use super::{verify_membership, verify_non_membership};

    #[test]
    fn test_verify_membership() {
        let fixture: fixtures::StepsFixture =
            fixtures::load("Test_ICS20TransferERC20TokenfromEthereumToCosmosAndBack");

        let initial_state: InitialState = fixture.get_data_at_step(0);

        let relayer_messages: RelayerMessages = fixture.get_data_at_step(1);
        let (update_client_msgs, recv_msgs, _, _) = relayer_messages.get_sdk_msgs();
        assert!(!update_client_msgs.is_empty());
        assert!(!recv_msgs.is_empty());

        let headers = update_client_msgs
            .iter()
            .map(|msg| {
                let client_msg =
                    ClientMessage::decode(msg.client_message.clone().unwrap().value.as_slice())
                        .unwrap();
                serde_json::from_slice(client_msg.data.as_slice()).unwrap()
            })
            .collect::<Vec<Header>>();

        let mut latest_consensus_state = initial_state.consensus_state;
        let mut latest_client_state = initial_state.client_state;
        for header in headers {
            let (_, updated_consensus_state, updated_client_state) =
                update_consensus_state(latest_consensus_state, latest_client_state, header.clone())
                    .unwrap();

            latest_consensus_state = updated_consensus_state;
            latest_client_state = updated_client_state.unwrap();
        }

        let trusted_consensus_state = latest_consensus_state;
        let client_state = latest_client_state;

        let packet = recv_msgs[0].packet.clone().unwrap();
        let storage_proof = recv_msgs[0].proof_commitment.clone();
        let (path, value, _) = get_packet_paths(packet);

        verify_membership(
            trusted_consensus_state,
            client_state,
            storage_proof,
            vec![path],
            value,
        )
        .unwrap();
    }

    #[test]
    fn test_verify_non_membership() {
        let fixture: fixtures::StepsFixture = fixtures::load("Test_TimeoutPacketFromCosmos");

        let initial_state: InitialState = fixture.get_data_at_step(0);

        let relayer_messages: RelayerMessages = fixture.get_data_at_step(1);
        let (update_client_msgs, _, _, timeout_msgs) = relayer_messages.get_sdk_msgs();
        assert!(!update_client_msgs.is_empty());
        assert!(!timeout_msgs.is_empty());

        let headers = update_client_msgs
            .iter()
            .map(|msg| {
                let client_msg =
                    ClientMessage::decode(msg.client_message.clone().unwrap().value.as_slice())
                        .unwrap();
                serde_json::from_slice(client_msg.data.as_slice()).unwrap()
            })
            .collect::<Vec<Header>>();

        let mut latest_consensus_state = initial_state.consensus_state;
        let mut latest_client_state = initial_state.client_state;
        for header in headers {
            let (_, updated_consensus_state, updated_client_state) =
                update_consensus_state(latest_consensus_state, latest_client_state, header.clone())
                    .unwrap();

            latest_consensus_state = updated_consensus_state;
            latest_client_state = updated_client_state.unwrap();
        }

        let trusted_consensus_state = latest_consensus_state;
        let client_state = latest_client_state;

        let packet = timeout_msgs[0].packet.clone().unwrap();
        let storage_proof = timeout_msgs[0].proof_unreceived.clone();
        let (_, _, path) = get_packet_paths(packet);

        verify_non_membership(
            trusted_consensus_state,
            client_state,
            storage_proof,
            vec![path],
        )
        .unwrap();
    }
}
