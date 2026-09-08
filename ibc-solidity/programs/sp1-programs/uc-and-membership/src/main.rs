// SPDX-License-Identifier: Apache-2.0

//! A program that verifies the membership or non-membership of a value in a commitment root.

#![deny(missing_docs, clippy::nursery, clippy::pedantic, warnings)]
#![allow(clippy::no_mangle_with_rust_abi)]
// These two lines are necessary for the program to properly compile.
//
// Under the hood, we wrap your main function with some extra code so that it behaves properly
// inside the zkVM.
#![no_main]
sp1_zkvm::entrypoint!(main);

use alloy_sol_types::SolValue;
use ibc_client_tendermint::types::Header;
use ibc_core_commitment_types::merkle::MerkleProof;
use ibc_eureka_solidity_types::msgs::{
    IICS07TendermintMsgs::{ClientState as SolClientState, ConsensusState as SolConsensusState},
    IMembershipMsgs::KVPair as SolKVPair,
    IUpdateClientAndMembershipMsgs::UcAndMembershipOutput as SolUcAndMembershipOutput,
    IUpdateClientMsgs::UpdateClientOutput as SolUpdateClientOutput,
};
use ibc_proto::{ibc::lightclients::tendermint::v1::Header as RawHeader, Protobuf};
use tendermint_light_client_membership::KVPair;
use tendermint_light_client_uc_and_membership::update_client_and_membership;
use tendermint_light_client_update_client::ClientState;

/// The main function of the program.
///
/// # Panics
/// Panics if the verification fails.
pub fn main() {
    let encoded_1 = sp1_zkvm::io::read_vec();
    let encoded_2 = sp1_zkvm::io::read_vec();
    let encoded_3 = sp1_zkvm::io::read_vec();
    let encoded_4 = sp1_zkvm::io::read_vec();
    // encoded_5 is the number of key-value pairs we want to verify
    let encoded_5 = sp1_zkvm::io::read_vec();
    let request_len = u16::from_le_bytes(encoded_5.try_into().unwrap());
    assert!(request_len != 0);

    // input 1: the client state
    let sol_client_state = SolClientState::abi_decode(&encoded_1).unwrap();
    let client_state: ClientState = (&sol_client_state).into();
    // input 2: the trusted consensus state
    let sol_consensus_state = SolConsensusState::abi_decode(&encoded_2).unwrap();
    let trusted_consensus_state = sol_consensus_state.clone().into();
    // input 3: the proposed header
    let proposed_header = <Header as Protobuf<RawHeader>>::decode_vec(&encoded_3).unwrap();
    // input 4: time
    let time = u128::from_le_bytes(encoded_4.try_into().unwrap());

    // Collect KVPairs and proofs separately
    let (sol_kv_pairs, kv_pairs): (Vec<_>, Vec<_>) = (0..request_len)
        .map(|_| {
            // loop_encoded_1 is the key-value pair we want to verify the membership of
            let loop_encoded_1 = sp1_zkvm::io::read_vec();
            let sol_kv_pair = SolKVPair::abi_decode(&loop_encoded_1).unwrap();
            let kv_pair = KVPair::new(
                sol_kv_pair.path.iter().map(|b| b.to_vec()).collect(),
                sol_kv_pair.value.to_vec(),
            );

            // loop_encoded_2 is the Merkle proof of the key-value pair
            let loop_encoded_2 = sp1_zkvm::io::read_vec();
            let merkle_proof = MerkleProof::decode_vec(&loop_encoded_2).unwrap();

            (sol_kv_pair, (kv_pair, merkle_proof))
        })
        .unzip();

    let output = update_client_and_membership(
        &client_state,
        &trusted_consensus_state,
        proposed_header,
        time,
        &kv_pairs,
    )
    .unwrap();

    // Convert output to Solidity format
    let sol_update_output = SolUpdateClientOutput::new(
        sol_client_state,
        sol_consensus_state,
        output.update_output,
        time,
    );
    let sol_output = SolUcAndMembershipOutput::new(sol_update_output, sol_kv_pairs);

    sp1_zkvm::io::commit_slice(&sol_output.abi_encode());
}
