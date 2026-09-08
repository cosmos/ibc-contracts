// SPDX-License-Identifier: Apache-2.0

//! This module contains the instantiate helper functions

use cosmwasm_std::{ensure, Storage};
use ethereum_light_client::{
    client_state::ClientState as EthClientState,
    consensus_state::ConsensusState as EthConsensusState,
};
use ibc_proto::ibc::{
    core::client::v1::Height as IbcProtoHeight,
    lightclients::wasm::v1::{
        ClientState as WasmClientState, ConsensusState as WasmConsensusState,
    },
};

use crate::{
    msg::InstantiateMsg,
    state::{store_client_state, store_consensus_state},
    ContractError,
};

/// Initializes the client state and consensus state
/// # Errors
/// Will return an error if the client state or consensus state cannot be deserialized.
/// # Panics
/// Will panic if the client state latest height cannot be unwrapped
#[allow(clippy::needless_pass_by_value)]
pub fn client(storage: &mut dyn Storage, msg: InstantiateMsg) -> Result<(), ContractError> {
    let client_state_bz: Vec<u8> = msg.client_state.into();
    let client_state: EthClientState = serde_json::from_slice(&client_state_bz)
        .map_err(ContractError::DeserializeClientStateFailed)?;
    let wasm_client_state = WasmClientState {
        checksum: msg.checksum.into(),
        data: client_state_bz,
        latest_height: Some(IbcProtoHeight {
            revision_number: 0,
            revision_height: client_state.latest_slot,
        }),
    };

    let consensus_state_bz: Vec<u8> = msg.consensus_state.into();
    let consensus_state: EthConsensusState = serde_json::from_slice(&consensus_state_bz)
        .map_err(ContractError::DeserializeConsensusStateFailed)?;
    let wasm_consensus_state = WasmConsensusState {
        data: consensus_state_bz,
    };

    ensure!(
        wasm_client_state.latest_height.unwrap().revision_height == client_state.latest_slot,
        ContractError::ClientStateSlotMismatch
    );

    ensure!(
        client_state.latest_slot == consensus_state.slot,
        ContractError::ClientAndConsensusStateMismatch
    );

    client_state
        .verify_supported_fork_at_epoch(
            client_state.compute_epoch_at_slot(client_state.latest_slot),
        )
        .map_err(ContractError::UnsupportedForkVersion)?;

    store_client_state(storage, &wasm_client_state)?;
    store_consensus_state(storage, &wasm_consensus_state, consensus_state.slot)?;

    Ok(())
}
