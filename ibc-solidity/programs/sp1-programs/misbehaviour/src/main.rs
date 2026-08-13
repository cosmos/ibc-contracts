// SPDX-License-Identifier: Apache-2.0

//! A program that verifies a misbehaviour evidence.

#![deny(missing_docs, clippy::nursery, clippy::pedantic, warnings)]
#![allow(clippy::no_mangle_with_rust_abi)]
// These two lines are necessary for the program to properly compile.
//
// Under the hood, we wrap your main function with some extra code so that it behaves properly
// inside the zkVM.
#![no_main]
sp1_zkvm::entrypoint!(main);

use alloy_sol_types::SolValue;
use ibc_client_tendermint::types::ConsensusState;
use ibc_client_tendermint::types::Misbehaviour;
use ibc_eureka_solidity_types::msgs::{
    IICS07TendermintMsgs::{ClientState as SolClientState, ConsensusState as SolConsensusState},
    IMisbehaviourMsgs::MisbehaviourOutput as SolMisbehaviourOutput,
};
use ibc_proto::{ibc::lightclients::tendermint::v1::Misbehaviour as RawMisbehaviour, Protobuf};
use tendermint_light_client_misbehaviour::check_for_misbehaviour;
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
    let encoded_5 = sp1_zkvm::io::read_vec();

    // input 1: client state
    let sol_client_state = SolClientState::abi_decode(&encoded_1).unwrap();
    let client_state: ClientState = (&sol_client_state).into();
    // input 2: the misbehaviour evidence
    let misbehaviour = <Misbehaviour as Protobuf<RawMisbehaviour>>::decode_vec(&encoded_2).unwrap();
    // input 3: header 1 trusted consensus state
    let sol_trusted_consensus_state_1 = SolConsensusState::abi_decode(&encoded_3).unwrap();
    let trusted_consensus_state_1: ConsensusState = sol_trusted_consensus_state_1.clone().into();
    // input 4: header 2 trusted consensus state
    let sol_trusted_consensus_state_2 = SolConsensusState::abi_decode(&encoded_4).unwrap();
    let trusted_consensus_state_2: ConsensusState = sol_trusted_consensus_state_2.clone().into();
    // input 5: time
    let time = u128::from_le_bytes(encoded_5.try_into().unwrap());

    let output = match check_for_misbehaviour(
        &client_state,
        &misbehaviour,
        trusted_consensus_state_1,
        trusted_consensus_state_2,
        time,
    ) {
        Ok(output) => output,
        Err(e) => panic!("{}", e),
    };

    // Convert output to Solidity format
    let sol_output = SolMisbehaviourOutput {
        clientState: sol_client_state,
        trustedHeight1: output.trusted_height_1.into(),
        trustedHeight2: output.trusted_height_2.into(),
        trustedConsensusState1: sol_trusted_consensus_state_1,
        trustedConsensusState2: sol_trusted_consensus_state_2,
        time,
    };

    sp1_zkvm::io::commit_slice(&sol_output.abi_encode());
}
