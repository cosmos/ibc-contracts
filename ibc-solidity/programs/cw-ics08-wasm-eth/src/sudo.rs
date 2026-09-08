// SPDX-License-Identifier: Apache-2.0

//! This module contains the sudo message handlers

use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut};
use ethereum_light_client::update::update_consensus_state;
use ibc_proto::ibc::{
    core::client::v1::Height as IbcProtoHeight,
    lightclients::wasm::v1::ConsensusState as WasmConsensusState,
};

use crate::{
    custom_query::EthereumCustomQuery,
    msg::{
        Height, UpdateStateMsg, UpdateStateOnMisbehaviourMsg, UpdateStateResult,
        VerifyMembershipMsg, VerifyNonMembershipMsg,
    },
    state::{
        get_eth_client_state, get_eth_consensus_state, get_wasm_client_state, store_client_state,
        store_consensus_state,
    },
    ContractError,
};

/// Verify the membership of a value at a given height
/// # Errors
/// Returns an error if the membership proof verification fails
/// # Returns
/// An empty response
pub fn verify_membership(
    deps: Deps<EthereumCustomQuery>,
    verify_membership_msg: VerifyMembershipMsg,
) -> Result<Binary, ContractError> {
    let eth_client_state = get_eth_client_state(deps.storage)?;
    let eth_consensus_state =
        get_eth_consensus_state(deps.storage, verify_membership_msg.height.revision_height)?;

    ethereum_light_client::membership::verify_membership(
        eth_consensus_state,
        eth_client_state,
        verify_membership_msg.proof.into(),
        verify_membership_msg
            .merkle_path
            .key_path
            .into_iter()
            .map(Into::into)
            .collect(),
        verify_membership_msg.value.into(),
    )
    .map_err(ContractError::VerifyMembershipFailed)?;

    Ok(Binary::default())
}

/// Verify the non-membership of a value at a given height
/// # Errors
/// Returns an error if the non-membership proof verification fails
/// # Returns
/// An empty response
pub fn verify_non_membership(
    deps: Deps<EthereumCustomQuery>,
    verify_non_membership_msg: VerifyNonMembershipMsg,
) -> Result<Binary, ContractError> {
    let eth_client_state = get_eth_client_state(deps.storage)?;
    let eth_consensus_state = get_eth_consensus_state(
        deps.storage,
        verify_non_membership_msg.height.revision_height,
    )?;

    ethereum_light_client::membership::verify_non_membership(
        eth_consensus_state,
        eth_client_state,
        verify_non_membership_msg.proof.into(),
        verify_non_membership_msg
            .merkle_path
            .key_path
            .into_iter()
            .map(Into::into)
            .collect(),
    )
    .map_err(ContractError::VerifyNonMembershipFailed)?;

    Ok(Binary::default())
}

/// Update the state of the light client
/// This function is always called after the verify client message, so
/// we can assume the client message is valid and that the consensus state can be updated
/// # Errors
/// Returns an error if deserialization failes or if the light client update logic fails
/// # Returns
/// The updated slot (called height in regular IBC terms)
#[allow(clippy::needless_pass_by_value)]
pub fn update_state(
    deps: DepsMut<EthereumCustomQuery>,
    update_state_msg: UpdateStateMsg,
) -> Result<Binary, ContractError> {
    let header_bz: Vec<u8> = update_state_msg.client_message.into();
    let header = serde_json::from_slice(&header_bz)
        .map_err(ContractError::DeserializeClientMessageFailed)?;

    let eth_client_state = get_eth_client_state(deps.storage)?;
    let eth_consensus_state = get_eth_consensus_state(deps.storage, eth_client_state.latest_slot)?;

    let (updated_slot, updated_consensus_state, updated_client_state) =
        update_consensus_state(eth_consensus_state, eth_client_state, header)
            .map_err(ContractError::UpdateClientStateFailed)?;

    let consensus_state_bz: Vec<u8> = serde_json::to_vec(&updated_consensus_state)
        .map_err(ContractError::SerializeConsensusStateFailed)?;
    let wasm_consensus_state = WasmConsensusState {
        data: consensus_state_bz,
    };
    store_consensus_state(deps.storage, &wasm_consensus_state, updated_slot)?;

    if let Some(client_state) = updated_client_state {
        let client_state_bz: Vec<u8> =
            serde_json::to_vec(&client_state).map_err(ContractError::SerializeClientStateFailed)?;

        let mut wasm_client_state = get_wasm_client_state(deps.storage)?;
        wasm_client_state.data = client_state_bz;
        wasm_client_state.latest_height = Some(IbcProtoHeight {
            revision_number: 0,
            revision_height: updated_slot,
        });
        store_client_state(deps.storage, &wasm_client_state)?;
    }

    Ok(to_json_binary(&UpdateStateResult {
        heights: vec![Height {
            revision_number: 0,
            revision_height: updated_slot,
        }],
    })?)
}

/// Update the state of the light client on misbehaviour
/// # Errors
/// Returns an error if the misbehaviour verification fails
#[allow(clippy::needless_pass_by_value)]
pub fn misbehaviour(
    deps: DepsMut<EthereumCustomQuery>,
    _msg: UpdateStateOnMisbehaviourMsg,
) -> Result<Binary, ContractError> {
    let mut eth_client_state = get_eth_client_state(deps.storage)?;
    eth_client_state.is_frozen = true;

    let client_state_bz: Vec<u8> =
        serde_json::to_vec(&eth_client_state).map_err(ContractError::SerializeClientStateFailed)?;

    let mut wasm_client_state = get_wasm_client_state(deps.storage)?;
    wasm_client_state.data = client_state_bz;

    store_client_state(deps.storage, &wasm_client_state)?;

    Ok(Binary::default())
}

#[cfg(test)]
mod tests {
    use crate::{contract::instantiate, test::helpers::mk_deps};
    use cosmwasm_std::{
        coins, from_json,
        testing::{message_info, mock_env},
        Binary,
    };
    use ethereum_light_client::test_utils::fixtures::{self, InitialState, StepsFixture};

    #[test]
    fn test_misbehaviour() {
        let mut deps = mk_deps();
        let creator = deps.api.addr_make("creator");
        let info = message_info(&creator, &coins(1, "uatom"));

        let fixture: StepsFixture =
            fixtures::load("Test_ICS20TransferNativeCosmosCoinsToEthereumAndBack");

        let initial_state: InitialState = fixture.get_data_at_step(0);

        let client_state = initial_state.client_state;
        let consensus_state = initial_state.consensus_state;

        let client_state_bz: Vec<u8> = serde_json::to_vec(&client_state).unwrap();
        let consensus_state_bz: Vec<u8> = serde_json::to_vec(&consensus_state).unwrap();

        let msg = crate::msg::InstantiateMsg {
            client_state: Binary::from(client_state_bz),
            consensus_state: Binary::from(consensus_state_bz),
            checksum: b"checksum".into(),
        };
        instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

        let msg = crate::msg::UpdateStateOnMisbehaviourMsg {
            client_message: Binary::default(),
        };
        let res = crate::sudo::misbehaviour(deps.as_mut(), msg).unwrap();
        assert_eq!(0, res.len());

        let eth_client_state = crate::state::get_eth_client_state(deps.as_ref().storage).unwrap();
        assert!(eth_client_state.is_frozen);

        // Query status
        let res = crate::query::status(deps.as_ref()).unwrap();
        let status_result: crate::msg::StatusResult = from_json(res).unwrap();
        assert_eq!("Frozen", status_result.status);
    }
}
