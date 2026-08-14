// SPDX-License-Identifier: Apache-2.0

//! This module contains the `CosmWasm` entrypoints for the 08-wasm smart contract

use cosmwasm_std::{entry_point, Binary, Deps, DepsMut, Env, MessageInfo, Response};

use crate::msg::{ExecuteMsg, InstantiateMsg, MigrateMsg, Migration, QueryMsg, SudoMsg};
use crate::{custom_query::EthereumCustomQuery, instantiate, query, state};
use crate::{sudo, ContractError};

/// The version of the contracts state.
/// It is used to determine if the state needs to be migrated in the migrate entry point.
const STATE_VERSION: &str = env!("CARGO_PKG_VERSION");
const CONTRACT_NAME: &str = env!("CARGO_PKG_NAME");

/// The instantiate entry point for the `CosmWasm` contract.
/// # Errors
/// Will return an error if the client state or consensus state cannot be deserialized.
/// # Panics
/// Will panic if the client state latest height cannot be unwrapped
#[entry_point]
#[allow(clippy::needless_pass_by_value)]
pub fn instantiate(
    deps: DepsMut<EthereumCustomQuery>,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, STATE_VERSION)?;

    instantiate::client(deps.storage, msg)?;

    Ok(Response::default())
}

/// The sudo entry point for the `CosmWasm` contract.
/// It routes the message to the appropriate handler.
/// # Errors
/// Will return an error if the handler returns an error.
#[entry_point]
#[allow(clippy::needless_pass_by_value)]
pub fn sudo(
    deps: DepsMut<EthereumCustomQuery>,
    _env: Env,
    msg: SudoMsg,
) -> Result<Response, ContractError> {
    let result = match msg {
        SudoMsg::VerifyMembership(verify_membership_msg) => {
            sudo::verify_membership(deps.as_ref(), verify_membership_msg)?
        }
        SudoMsg::VerifyNonMembership(verify_non_membership_msg) => {
            sudo::verify_non_membership(deps.as_ref(), verify_non_membership_msg)?
        }
        SudoMsg::UpdateState(update_state_msg) => sudo::update_state(deps, update_state_msg)?,
        SudoMsg::UpdateStateOnMisbehaviour(misbehaviour_msg) => {
            sudo::misbehaviour(deps, misbehaviour_msg)?
        }
        SudoMsg::VerifyUpgradeAndUpdateState(_) => todo!(),
        SudoMsg::MigrateClientStore(_) => todo!(),
    };

    Ok(Response::default().set_data(result))
}

/// Execute entry point is not used in this contract.
#[entry_point]
#[allow(clippy::needless_pass_by_value, clippy::missing_errors_doc)]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    unimplemented!()
}

/// The query entry point for the `CosmWasm` contract.
/// It routes the message to the appropriate handler.
/// # Errors
/// Will return an error if the handler returns an error.
#[entry_point]
pub fn query(
    deps: Deps<EthereumCustomQuery>,
    env: Env,
    msg: QueryMsg,
) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::VerifyClientMessage(verify_client_message_msg) => {
            query::verify_client_message(deps, env, verify_client_message_msg)
        }
        QueryMsg::CheckForMisbehaviour(check_for_misbehaviour_msg) => {
            query::check_for_misbehaviour(deps, env, check_for_misbehaviour_msg)
        }
        QueryMsg::TimestampAtHeight(timestamp_at_height_msg) => {
            query::timestamp_at_height(deps, timestamp_at_height_msg)
        }
        QueryMsg::Status(_) => query::status(deps),
    }
}

/// The migrate entry point for the `CosmWasm` contract.
/// # Errors
/// Will return an errror if the state version is not newer than the current one.
#[entry_point]
#[allow(clippy::needless_pass_by_value)]
pub fn migrate(
    deps: DepsMut<EthereumCustomQuery>,
    _env: Env,
    msg: MigrateMsg,
) -> Result<Response, ContractError> {
    // Check if the state version is older than the current one and update it
    cw2::ensure_from_older_version(deps.storage, CONTRACT_NAME, STATE_VERSION)?;

    // Perform the migration
    match msg.migration {
        Migration::CodeOnly => {} // do nothing here
        Migration::Reinstantiate(instantiate_msg) => {
            // Re-instantiate the client
            instantiate::client(deps.storage, instantiate_msg)?;
        }
        Migration::UpdateForkParameters(fork_parameters) => {
            // Change the fork parameters
            let mut client_state = state::get_eth_client_state(deps.storage)?;
            client_state.fork_parameters = fork_parameters;
            let client_state_bz: Vec<u8> = serde_json::to_vec(&client_state)
                .map_err(ContractError::SerializeClientStateFailed)?;
            let mut wasm_client_state = state::get_wasm_client_state(deps.storage)?;
            wasm_client_state.data = client_state_bz;
            state::store_client_state(deps.storage, &wasm_client_state)?;
        }
    }

    Ok(Response::default())
}

#[cfg(test)]
mod tests {
    mod instantiate_tests {
        use alloy_primitives::{Address, FixedBytes, B256, U256};
        use cosmwasm_std::{
            coins,
            testing::{message_info, mock_env},
            Storage,
        };
        use ethereum_light_client::{
            client_state::ClientState as EthClientState,
            consensus_state::ConsensusState as EthConsensusState,
        };
        use ethereum_types::consensus::{
            fork::{Fork, ForkParameters},
            sync_committee::SummarizedSyncCommittee,
        };
        use ibc_proto::{
            google::protobuf::Any,
            ibc::lightclients::wasm::v1::{
                ClientState as WasmClientState, ConsensusState as WasmConsensusState,
            },
        };
        use prost::{Message, Name};

        use crate::{
            contract::instantiate,
            msg::InstantiateMsg,
            state::{consensus_db_key, HOST_CLIENT_STATE_KEY},
            test::helpers::mk_deps,
        };

        #[test]
        fn test_instantiate() {
            let mut deps = mk_deps();
            let creator = deps.api.addr_make("creator");
            let info = message_info(&creator, &coins(1, "uatom"));

            let client_state = EthClientState {
                chain_id: 0,
                genesis_validators_root: B256::from([0; 32]),
                min_sync_committee_participants: 0,
                sync_committee_size: 0,
                genesis_time: 0,
                genesis_slot: 0,
                fork_parameters: ForkParameters {
                    genesis_fork_version: FixedBytes([0; 4]),
                    genesis_slot: 0,
                    altair: Fork {
                        version: FixedBytes([0; 4]),
                        epoch: 0,
                    },
                    bellatrix: Fork {
                        version: FixedBytes([0; 4]),
                        epoch: 0,
                    },
                    capella: Fork {
                        version: FixedBytes([0; 4]),
                        epoch: 0,
                    },
                    deneb: Fork {
                        version: FixedBytes([0; 4]),
                        epoch: 0,
                    },
                    electra: Fork {
                        version: FixedBytes([0; 4]),
                        epoch: 0,
                    },
                    fulu: Fork {
                        version: FixedBytes([0; 4]),
                        epoch: 0,
                    },
                },
                seconds_per_slot: 10,
                slots_per_epoch: 8,
                epochs_per_sync_committee_period: 0,
                latest_slot: 42,
                latest_execution_block_number: 38,
                ibc_commitment_slot: U256::from(0),
                ibc_contract_address: Address::default(),
                is_frozen: false,
            };
            let client_state_bz: Vec<u8> = serde_json::to_vec(&client_state).unwrap();

            let consensus_state = EthConsensusState {
                slot: 42,
                state_root: B256::from([0; 32]),
                timestamp: 0,
                current_sync_committee: SummarizedSyncCommittee::default(),
                next_sync_committee: None,
            };
            let consensus_state_bz: Vec<u8> = serde_json::to_vec(&consensus_state).unwrap();

            let msg = InstantiateMsg {
                client_state: client_state_bz.into(),
                consensus_state: consensus_state_bz.into(),
                checksum: b"also does not matter yet".into(),
            };

            let res = instantiate(deps.as_mut(), mock_env(), info, msg.clone()).unwrap();
            assert_eq!(0, res.messages.len());

            let actual_wasm_client_state_any_bz =
                deps.storage.get(HOST_CLIENT_STATE_KEY.as_bytes()).unwrap();
            let actual_wasm_client_state_any =
                Any::decode(actual_wasm_client_state_any_bz.as_slice()).unwrap();
            assert_eq!(
                WasmClientState::type_url(),
                actual_wasm_client_state_any.type_url
            );
            let actual_client_state =
                WasmClientState::decode(actual_wasm_client_state_any.value.as_slice()).unwrap();
            assert_eq!(msg.checksum, actual_client_state.checksum);
            assert_eq!(msg.client_state, actual_client_state.data);
            assert_eq!(
                0,
                actual_client_state.latest_height.unwrap().revision_number
            );
            assert_eq!(
                client_state.latest_slot,
                actual_client_state.latest_height.unwrap().revision_height
            );

            let actual_wasm_consensus_state_any_bz = deps
                .storage
                .get(consensus_db_key(consensus_state.slot).as_bytes())
                .unwrap();
            let actual_wasm_consensus_state_any =
                Any::decode(actual_wasm_consensus_state_any_bz.as_slice()).unwrap();
            assert_eq!(
                WasmConsensusState::type_url(),
                actual_wasm_consensus_state_any.type_url
            );
            let actual_consensus_state =
                WasmConsensusState::decode(actual_wasm_consensus_state_any.value.as_slice())
                    .unwrap();
            assert_eq!(msg.consensus_state, actual_consensus_state.data);
        }
    }

    mod integration_tests {
        use alloy_primitives::{Address, FixedBytes, B256, U256};
        use cosmwasm_std::{
            coins,
            testing::{message_info, mock_env},
            Binary, Storage, Timestamp,
        };
        use ethereum_light_client::{
            client_state::ClientState as EthClientState,
            consensus_state::ConsensusState as EthConsensusState,
            error::EthereumIBCError,
            header::{ActiveSyncCommittee, Header},
            test_utils::fixtures::{
                self, get_packet_paths, InitialState, RelayerMessages, StepsFixture,
            },
        };
        use ethereum_types::consensus::{
            fork::{Fork, ForkParameters},
            sync_committee::SummarizedSyncCommittee,
        };
        use ibc_proto::{
            google::protobuf::Any,
            ibc::lightclients::wasm::v1::{ClientMessage, ClientState as WasmClientState},
        };
        use prost::Message;

        use crate::{
            contract::{instantiate, migrate, query, sudo},
            msg::{
                Height, InstantiateMsg, MerklePath, MigrateMsg, Migration, QueryMsg, SudoMsg,
                UpdateStateMsg, UpdateStateResult, VerifyClientMessageMsg, VerifyMembershipMsg,
            },
            state::HOST_CLIENT_STATE_KEY,
            test::helpers::mk_deps,
            ContractError,
        };

        #[test]
        // This test runs throught the e2e test scenario defined in the interchaintest:
        // TestICS20TransferERC20TokenfromEthereumToCosmosAndBack_Groth16
        fn test_ics20_transfer_from_ethereum_to_cosmos_flow() {
            let mut deps = mk_deps();
            let creator = deps.api.addr_make("creator");
            let info = message_info(&creator, &coins(1, "uatom"));

            let fixture: StepsFixture =
                fixtures::load("Test_ICS20TransferERC20TokenfromEthereumToCosmosAndBack");

            let initial_state: InitialState = fixture.get_data_at_step(0);

            let client_state = initial_state.client_state;

            let consensus_state = initial_state.consensus_state;

            let client_state_bz: Vec<u8> = serde_json::to_vec(&client_state).unwrap();
            let consensus_state_bz: Vec<u8> = serde_json::to_vec(&consensus_state).unwrap();

            let msg = InstantiateMsg {
                client_state: Binary::from(client_state_bz),
                consensus_state: Binary::from(consensus_state_bz),
                checksum: b"checksum".into(),
            };

            instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

            // At this point, the light clients are initialized and the client state is stored
            // In the flow, an ICS20 transfer has been initiated from Ethereum to Cosmos
            // Next up we want to prove the packet on the Cosmos chain, so we start by updating the
            // light client (which is two steps: verify client message and update state)

            // Verify client message
            let relayer_messages: RelayerMessages = fixture.get_data_at_step(1);
            let (update_client_msgs, recv_msgs, _, _) = relayer_messages.get_sdk_msgs();
            assert_eq!(1, update_client_msgs.len()); // just to make sure
            assert_eq!(1, recv_msgs.len()); // just to make sure
            let client_msgs = update_client_msgs
                .iter()
                .map(|msg| {
                    ClientMessage::decode(msg.client_message.clone().unwrap().value.as_slice())
                        .unwrap()
                })
                .map(|msg| msg.data)
                .collect::<Vec<_>>();

            let mut env = mock_env();

            for header_bz in client_msgs {
                let header: Header = serde_json::from_slice(&header_bz).unwrap();
                env.block.time = Timestamp::from_seconds(
                    header.consensus_update.attested_header.execution.timestamp + 1000,
                );

                let query_verify_client_msg =
                    QueryMsg::VerifyClientMessage(VerifyClientMessageMsg {
                        client_message: Binary::from(header_bz.clone()),
                    });
                query(deps.as_ref(), env.clone(), query_verify_client_msg).unwrap();

                // Update state
                let sudo_update_state_msg = SudoMsg::UpdateState(UpdateStateMsg {
                    client_message: Binary::from(header_bz),
                });
                let update_res = sudo(deps.as_mut(), env.clone(), sudo_update_state_msg).unwrap();
                let update_state_result: UpdateStateResult =
                    serde_json::from_slice(&update_res.data.unwrap())
                        .expect("update state result should be deserializable");
                assert_eq!(1, update_state_result.heights.len());
                assert_eq!(0, update_state_result.heights[0].revision_number);
                assert_eq!(
                    header.consensus_update.finalized_header.beacon.slot,
                    update_state_result.heights[0].revision_height
                );
            }

            // The client has now been updated, and we would submit the packet to the cosmos chain,
            // along with the proof of th packet commitment. IBC will call verify_membership.

            // Verify memebership
            let packet = recv_msgs[0].packet.clone().unwrap();
            let storage_proof = recv_msgs[0].proof_commitment.clone();
            let (path, value, _) = get_packet_paths(packet);

            let query_verify_membership_msg = SudoMsg::VerifyMembership(VerifyMembershipMsg {
                height: Height {
                    revision_number: 0,
                    revision_height: recv_msgs[0].proof_height.unwrap().revision_height,
                },
                delay_time_period: 0,
                delay_block_period: 0,
                proof: Binary::from(storage_proof),
                merkle_path: MerklePath {
                    key_path: vec![Binary::from(path)],
                },
                value: Binary::from(value),
            });
            sudo(deps.as_mut(), env, query_verify_membership_msg).unwrap();
        }

        /// This test runs through a scenario where a malicious relayer sends an incorrect sync
        /// commitee whose aggregate pubkey is the same as the one in the header.
        #[test]
        fn test_aggragate_sync_committee_collision() {
            let mut deps = mk_deps();
            let creator = deps.api.addr_make("creator");
            let info = message_info(&creator, &coins(1, "uatom"));

            let fixture: StepsFixture =
                fixtures::load("Test_ICS20TransferERC20TokenfromEthereumToCosmosAndBack");

            let initial_state: InitialState = fixture.get_data_at_step(0);

            let client_state = initial_state.client_state;

            let consensus_state = initial_state.consensus_state;

            let client_state_bz: Vec<u8> = serde_json::to_vec(&client_state).unwrap();
            let consensus_state_bz: Vec<u8> = serde_json::to_vec(&consensus_state).unwrap();

            let msg = InstantiateMsg {
                client_state: Binary::from(client_state_bz),
                consensus_state: Binary::from(consensus_state_bz),
                checksum: b"checksum".into(),
            };

            instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

            // At this point, the light clients are initialized and the client state is stored
            // In the flow, an ICS20 transfer has been initiated from Ethereum to Cosmos
            // Next up we want to prove the packet on the Cosmos chain, so we start by updating the
            // light client (which is two steps: verify client message and update state)

            // Verify client message
            let relayer_messages: RelayerMessages = fixture.get_data_at_step(1);
            let (update_client_msgs, recv_msgs, _, _) = relayer_messages.get_sdk_msgs();
            assert_eq!(1, update_client_msgs.len()); // just to make sure
            assert_eq!(1, recv_msgs.len()); // just to make sure
            let client_msgs = update_client_msgs
                .iter()
                .map(|msg| {
                    ClientMessage::decode(msg.client_message.clone().unwrap().value.as_slice())
                        .unwrap()
                })
                .map(|msg| msg.data)
                .collect::<Vec<_>>();

            let mut env = mock_env();

            for header_bz in client_msgs {
                let mut header: Header = serde_json::from_slice(&header_bz).unwrap();

                let sync_committee = header.active_sync_committee;

                header.active_sync_committee = sync_committee.clone();
                if let ActiveSyncCommittee::Current(_x) = sync_committee {
                    panic!("shouldn't happen");
                } else if let ActiveSyncCommittee::Next(x) = sync_committee {
                    let mut m = x.clone();

                    let pk1 = FixedBytes([
                        140, 49, 208, 243, 132, 3, 164, 40, 45, 148, 208, 102, 241, 152, 252, 233,
                        211, 98, 140, 14, 252, 12, 218, 20, 119, 221, 237, 190, 104, 87, 99, 203,
                        84, 46, 133, 35, 12, 231, 182, 84, 204, 230, 21, 131, 156, 120, 141, 61,
                    ]);
                    let pk2 = FixedBytes([
                        172, 49, 208, 243, 132, 3, 164, 40, 45, 148, 208, 102, 241, 152, 252, 233,
                        211, 98, 140, 14, 252, 12, 218, 20, 119, 221, 237, 190, 104, 87, 99, 203,
                        84, 46, 133, 35, 12, 231, 182, 84, 204, 230, 21, 131, 156, 120, 141, 61,
                    ]);

                    m.pubkeys = vec![m.aggregate_pubkey, pk1, pk2];

                    let mut bits = vec![0xFF; 48];
                    bits[0] = 0b0000_0010;

                    header.consensus_update.sync_aggregate.sync_committee_bits = bits.into();
                    header
                        .consensus_update
                        .sync_aggregate
                        .sync_committee_signature = FixedBytes([
                        141, 213, 40, 198, 216, 4, 232, 6, 81, 233, 68, 218, 77, 6, 86, 182, 237,
                        151, 157, 194, 232, 232, 2, 229, 197, 81, 72, 102, 47, 198, 140, 250, 207,
                        60, 148, 124, 180, 228, 54, 236, 83, 56, 107, 245, 42, 98, 160, 150, 1,
                        238, 185, 147, 132, 245, 121, 184, 114, 109, 240, 147, 152, 17, 155, 245,
                        103, 165, 20, 131, 198, 158, 174, 20, 209, 57, 48, 219, 193, 164, 139, 206,
                        114, 40, 86, 54, 211, 231, 111, 231, 233, 198, 92, 154, 229, 100, 165, 215,
                    ]);

                    header.active_sync_committee = ActiveSyncCommittee::Next(m);
                }

                let header_manipulated = serde_json::to_vec(&header).unwrap();

                env.block.time = Timestamp::from_seconds(
                    header.consensus_update.attested_header.execution.timestamp + 1000,
                );

                let query_verify_client_msg =
                    QueryMsg::VerifyClientMessage(VerifyClientMessageMsg {
                        client_message: Binary::from(header_manipulated.clone()),
                    });
                // NOTE: It should error here if the vuln is patched
                let err = query(deps.as_ref(), env.clone(), query_verify_client_msg).unwrap_err();
                assert!(matches!(
                    err,
                    ContractError::VerifyClientMessageFailed(
                        EthereumIBCError::NextSyncCommitteeMismatch { .. }
                    )
                ));
            }
        }

        #[test]
        fn test_invalid_sync_committee_size() {
            let mut deps = mk_deps();
            let creator = deps.api.addr_make("creator");
            let info = message_info(&creator, &coins(1, "uatom"));

            let fixture: StepsFixture =
                fixtures::load("Test_ICS20TransferERC20TokenfromEthereumToCosmosAndBack");

            let initial_state: InitialState = fixture.get_data_at_step(0);

            let client_state = initial_state.client_state;

            let mut consensus_state = initial_state.consensus_state;

            // Verify client message
            let relayer_messages: RelayerMessages = fixture.get_data_at_step(1);
            let (update_client_msgs, recv_msgs, _, _) = relayer_messages.get_sdk_msgs();
            assert_eq!(1, update_client_msgs.len()); // just to make sure
            assert_eq!(1, recv_msgs.len()); // just to make sure
            let client_msgs = update_client_msgs
                .iter()
                .map(|msg| {
                    ClientMessage::decode(msg.client_message.clone().unwrap().value.as_slice())
                        .unwrap()
                })
                .map(|msg| msg.data)
                .collect::<Vec<_>>();

            let mut env = mock_env();

            for header_bz in client_msgs {
                let mut header: Header = serde_json::from_slice(&header_bz).unwrap();

                let sync_committee = header.active_sync_committee;

                header.active_sync_committee = sync_committee.clone();
                if let ActiveSyncCommittee::Current(_x) = sync_committee {
                    panic!("shouldn't happen");
                } else if let ActiveSyncCommittee::Next(x) = sync_committee {
                    let mut m = x.clone();

                    let pk1 = FixedBytes([
                        140, 49, 208, 243, 132, 3, 164, 40, 45, 148, 208, 102, 241, 152, 252, 233,
                        211, 98, 140, 14, 252, 12, 218, 20, 119, 221, 237, 190, 104, 87, 99, 203,
                        84, 46, 133, 35, 12, 231, 182, 84, 204, 230, 21, 131, 156, 120, 141, 61,
                    ]);
                    let pk2 = FixedBytes([
                        172, 49, 208, 243, 132, 3, 164, 40, 45, 148, 208, 102, 241, 152, 252, 233,
                        211, 98, 140, 14, 252, 12, 218, 20, 119, 221, 237, 190, 104, 87, 99, 203,
                        84, 46, 133, 35, 12, 231, 182, 84, 204, 230, 21, 131, 156, 120, 141, 61,
                    ]);

                    m.pubkeys = vec![m.aggregate_pubkey, pk1, pk2];

                    let mut bits = vec![0xFF; 48];
                    bits[0] = 0b0000_0010;

                    header.consensus_update.sync_aggregate.sync_committee_bits = bits.into();
                    header
                        .consensus_update
                        .sync_aggregate
                        .sync_committee_signature = FixedBytes([
                        141, 213, 40, 198, 216, 4, 232, 6, 81, 233, 68, 218, 77, 6, 86, 182, 237,
                        151, 157, 194, 232, 232, 2, 229, 197, 81, 72, 102, 47, 198, 140, 250, 207,
                        60, 148, 124, 180, 228, 54, 236, 83, 56, 107, 245, 42, 98, 160, 150, 1,
                        238, 185, 147, 132, 245, 121, 184, 114, 109, 240, 147, 152, 17, 155, 245,
                        103, 165, 20, 131, 198, 158, 174, 20, 209, 57, 48, 219, 193, 164, 139, 206,
                        114, 40, 86, 54, 211, 231, 111, 231, 233, 198, 92, 154, 229, 100, 165, 215,
                    ]);

                    consensus_state.next_sync_committee = Some(m.to_summarized_sync_committee());
                    header.active_sync_committee = ActiveSyncCommittee::Next(m);
                }

                let client_state_bz: Vec<u8> = serde_json::to_vec(&client_state).unwrap();
                let consensus_state_bz: Vec<u8> = serde_json::to_vec(&consensus_state).unwrap();

                let msg = InstantiateMsg {
                    client_state: Binary::from(client_state_bz),
                    consensus_state: Binary::from(consensus_state_bz),
                    checksum: b"checksum".into(),
                };

                instantiate(deps.as_mut(), mock_env(), info.clone(), msg).unwrap();

                let header_manipulated = serde_json::to_vec(&header).unwrap();

                env.block.time = Timestamp::from_seconds(
                    header.consensus_update.attested_header.execution.timestamp + 1000,
                );

                let query_verify_client_msg =
                    QueryMsg::VerifyClientMessage(VerifyClientMessageMsg {
                        client_message: Binary::from(header_manipulated.clone()),
                    });
                // NOTE: It should error here if the vuln is patched
                let err = query(deps.as_ref(), env.clone(), query_verify_client_msg).unwrap_err();
                assert!(matches!(
                    err,
                    ContractError::VerifyClientMessageFailed(
                        EthereumIBCError::InsufficientSyncCommitteeLength {
                            expected: 32,
                            found: 3
                        }
                    )
                ));
            }
        }

        #[test]
        fn test_update_with_period_change() {
            let mut deps = mk_deps();
            let creator = deps.api.addr_make("creator");
            let info = message_info(&creator, &coins(1, "uatom"));

            let fixture: StepsFixture = fixtures::load("Test_MultiPeriodClientUpdateToCosmos");

            let initial_state: InitialState = fixture.get_data_at_step(0);

            let client_state = initial_state.client_state;

            let consensus_state = initial_state.consensus_state;

            let client_state_bz: Vec<u8> = serde_json::to_vec(&client_state).unwrap();
            let consensus_state_bz: Vec<u8> = serde_json::to_vec(&consensus_state).unwrap();

            let msg = InstantiateMsg {
                client_state: Binary::from(client_state_bz),
                consensus_state: Binary::from(consensus_state_bz),
                checksum: b"checksum".into(),
            };

            instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

            // At this point, the light clients are initialized and the client state is stored
            // In the flow, an ICS20 transfer has been initiated from Ethereum to Cosmos
            // Next up we want to prove the packet on the Cosmos chain, so we start by updating the
            // light client (which is two steps: verify client message and update state)

            // Verify client message
            let relayer_messages: RelayerMessages = fixture.get_data_at_step(1);
            let (update_client_msgs, recv_msgs, _, _) = relayer_messages.get_sdk_msgs();
            assert!(update_client_msgs.len() >= 2); // just to make sure
            assert_eq!(1, recv_msgs.len()); // just to make sure
            let client_msgs = update_client_msgs
                .iter()
                .map(|msg| {
                    ClientMessage::decode(msg.client_message.clone().unwrap().value.as_slice())
                        .unwrap()
                })
                .map(|msg| msg.data)
                .collect::<Vec<_>>();

            let mut env = mock_env();

            for header_bz in client_msgs {
                let header: Header = serde_json::from_slice(&header_bz).unwrap();
                env.block.time = Timestamp::from_seconds(
                    header.consensus_update.attested_header.execution.timestamp + 1000,
                );

                let query_verify_client_msg =
                    QueryMsg::VerifyClientMessage(VerifyClientMessageMsg {
                        client_message: Binary::from(header_bz.clone()),
                    });
                query(deps.as_ref(), env.clone(), query_verify_client_msg).unwrap();

                // Update state
                let sudo_update_state_msg = SudoMsg::UpdateState(UpdateStateMsg {
                    client_message: Binary::from(header_bz),
                });
                let update_res = sudo(deps.as_mut(), env.clone(), sudo_update_state_msg).unwrap();
                let update_state_result: UpdateStateResult =
                    serde_json::from_slice(&update_res.data.unwrap())
                        .expect("update state result should be deserializable");
                assert_eq!(1, update_state_result.heights.len());
                assert_eq!(0, update_state_result.heights[0].revision_number);
                assert_eq!(
                    header.consensus_update.finalized_header.beacon.slot,
                    update_state_result.heights[0].revision_height
                );
            }

            // The client has now been updated, and we would submit the packet to the cosmos chain,
            // along with the proof of th packet commitment. IBC will call verify_membership.

            // Verify memebership
            let packet = recv_msgs[0].packet.clone().unwrap();
            let storage_proof = recv_msgs[0].proof_commitment.clone();
            let (path, value, _) = get_packet_paths(packet);

            let query_verify_membership_msg = SudoMsg::VerifyMembership(VerifyMembershipMsg {
                height: Height {
                    revision_number: 0,
                    revision_height: recv_msgs[0].proof_height.unwrap().revision_height,
                },
                delay_time_period: 0,
                delay_block_period: 0,
                proof: Binary::from(storage_proof),
                merkle_path: MerklePath {
                    key_path: vec![Binary::from(path)],
                },
                value: Binary::from(value),
            });
            sudo(deps.as_mut(), env, query_verify_membership_msg).unwrap();
        }

        #[test]
        fn test_migrate_with_same_state_version() {
            let mut deps = mk_deps();
            let creator = deps.api.addr_make("creator");
            let info = message_info(&creator, &coins(1, "uatom"));

            let fixture: StepsFixture =
                fixtures::load("Test_ICS20TransferERC20TokenfromEthereumToCosmosAndBack");

            let initial_state: InitialState = fixture.get_data_at_step(0);

            let client_state = initial_state.client_state;

            let consensus_state = initial_state.consensus_state;

            let client_state_bz: Vec<u8> = serde_json::to_vec(&client_state).unwrap();
            let consensus_state_bz: Vec<u8> = serde_json::to_vec(&consensus_state).unwrap();

            let msg = InstantiateMsg {
                client_state: Binary::from(client_state_bz),
                consensus_state: Binary::from(consensus_state_bz),
                checksum: b"checksum".into(),
            };

            instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();

            // Migrate without any changes (i.e. same state version)
            migrate(
                deps.as_mut(),
                mock_env(),
                MigrateMsg {
                    migration: Migration::CodeOnly,
                },
            )
            .unwrap();
        }

        #[test]
        fn test_migrate_with_instantiate() {
            let mut deps = mk_deps();
            let creator = deps.api.addr_make("creator");
            let info = message_info(&creator, &coins(1, "uatom"));

            let client_state = EthClientState {
                chain_id: 0,
                genesis_validators_root: B256::from([0; 32]),
                min_sync_committee_participants: 0,
                genesis_time: 0,
                genesis_slot: 0,
                fork_parameters: ForkParameters {
                    genesis_fork_version: FixedBytes([0; 4]),
                    genesis_slot: 0,
                    altair: Fork {
                        version: FixedBytes([0; 4]),
                        epoch: 0,
                    },
                    bellatrix: Fork {
                        version: FixedBytes([0; 4]),
                        epoch: 0,
                    },
                    capella: Fork {
                        version: FixedBytes([0; 4]),
                        epoch: 0,
                    },
                    deneb: Fork {
                        version: FixedBytes([0; 4]),
                        epoch: 0,
                    },
                    electra: Fork {
                        version: FixedBytes([0; 4]),
                        epoch: 0,
                    },
                    fulu: Fork {
                        version: FixedBytes([0; 4]),
                        epoch: 0,
                    },
                },
                sync_committee_size: 512,
                seconds_per_slot: 10,
                slots_per_epoch: 8,
                epochs_per_sync_committee_period: 0,
                latest_slot: 42,
                latest_execution_block_number: 38,
                ibc_commitment_slot: U256::from(0),
                ibc_contract_address: Address::default(),
                is_frozen: false,
            };
            let client_state_bz: Vec<u8> = serde_json::to_vec(&client_state).unwrap();

            let consensus_state = EthConsensusState {
                slot: 42,
                state_root: B256::from([0; 32]),
                timestamp: 0,
                current_sync_committee: SummarizedSyncCommittee::default(),
                next_sync_committee: None,
            };
            let consensus_state_bz: Vec<u8> = serde_json::to_vec(&consensus_state).unwrap();

            let msg = InstantiateMsg {
                client_state: client_state_bz.into(),
                consensus_state: consensus_state_bz.into(),
                checksum: b"does not matter yet".into(),
            };

            let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
            assert_eq!(0, res.messages.len());

            let fixture: StepsFixture =
                fixtures::load("Test_ICS20TransferERC20TokenfromEthereumToCosmosAndBack");

            // Initial state is at Electra fork
            let initial_state: InitialState = fixture.get_data_at_step(0);

            let client_state_fixture = initial_state.client_state;

            let consensus_state_fixture = initial_state.consensus_state;

            let client_state_fixture_bz: Vec<u8> =
                serde_json::to_vec(&client_state_fixture).unwrap();
            let consensus_state_fixture_bz: Vec<u8> =
                serde_json::to_vec(&consensus_state_fixture).unwrap();

            let msg = InstantiateMsg {
                client_state: Binary::from(client_state_fixture_bz),
                consensus_state: Binary::from(consensus_state_fixture_bz),
                checksum: b"checksum".into(),
            };

            let migrate_msg = MigrateMsg {
                migration: Migration::Reinstantiate(msg.clone()),
            };

            // Migrate without any changes (i.e. same state version)
            migrate(deps.as_mut(), mock_env(), migrate_msg).unwrap();

            let actual_wasm_client_state_any_bz =
                deps.storage.get(HOST_CLIENT_STATE_KEY.as_bytes()).unwrap();
            let actual_wasm_client_state_any =
                Any::decode(actual_wasm_client_state_any_bz.as_slice()).unwrap();
            let wasm_client_state =
                WasmClientState::decode(actual_wasm_client_state_any.value.as_slice()).unwrap();
            assert_eq!(msg.checksum, wasm_client_state.checksum);
            assert_ne!(
                wasm_client_state.latest_height.unwrap().revision_height,
                client_state.latest_slot
            );
        }

        #[allow(clippy::too_many_lines)]
        #[test]
        fn test_migrate_with_fork_parameters() {
            let mut deps = mk_deps();
            let creator = deps.api.addr_make("creator");
            let info = message_info(&creator, &coins(1, "uatom"));

            let client_state = EthClientState {
                chain_id: 0,
                genesis_validators_root: B256::from([0; 32]),
                min_sync_committee_participants: 0,
                genesis_time: 0,
                genesis_slot: 0,
                fork_parameters: ForkParameters {
                    genesis_fork_version: FixedBytes([0; 4]),
                    genesis_slot: 0,
                    altair: Fork {
                        version: FixedBytes([0; 4]),
                        epoch: 0,
                    },
                    bellatrix: Fork {
                        version: FixedBytes([0; 4]),
                        epoch: 0,
                    },
                    capella: Fork {
                        version: FixedBytes([0; 4]),
                        epoch: 0,
                    },
                    deneb: Fork {
                        version: FixedBytes([0; 4]),
                        epoch: 0,
                    },
                    electra: Fork {
                        version: FixedBytes([0; 4]),
                        epoch: 0,
                    },
                    fulu: Fork {
                        version: FixedBytes([0; 4]),
                        epoch: 0,
                    },
                },
                sync_committee_size: 512,
                seconds_per_slot: 10,
                slots_per_epoch: 8,
                epochs_per_sync_committee_period: 0,
                latest_slot: 42,
                latest_execution_block_number: 38,
                ibc_commitment_slot: U256::from(0),
                ibc_contract_address: Address::default(),
                is_frozen: false,
            };
            let client_state_bz: Vec<u8> = serde_json::to_vec(&client_state).unwrap();

            let consensus_state = EthConsensusState {
                slot: 42,
                state_root: B256::from([0; 32]),
                timestamp: 0,
                current_sync_committee: SummarizedSyncCommittee::default(),
                next_sync_committee: None,
            };
            let consensus_state_bz: Vec<u8> = serde_json::to_vec(&consensus_state).unwrap();

            let msg = InstantiateMsg {
                client_state: client_state_bz.into(),
                consensus_state: consensus_state_bz.into(),
                checksum: b"checksum".into(),
            };
            let msg_copy = msg.clone();

            let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
            assert_eq!(0, res.messages.len());

            let migrate_msg = MigrateMsg {
                migration: Migration::UpdateForkParameters(ForkParameters {
                    genesis_fork_version: FixedBytes([0; 4]),
                    genesis_slot: 0,
                    altair: Fork {
                        version: FixedBytes([0; 4]),
                        epoch: 0,
                    },
                    bellatrix: Fork {
                        version: FixedBytes([0; 4]),
                        epoch: 0,
                    },
                    capella: Fork {
                        version: FixedBytes([0; 4]),
                        epoch: 0,
                    },
                    deneb: Fork {
                        version: FixedBytes([0; 4]),
                        epoch: 0,
                    },
                    electra: Fork {
                        version: FixedBytes([0; 4]),
                        epoch: 5000,
                    },
                    fulu: Fork {
                        version: FixedBytes([0; 4]),
                        epoch: 0,
                    },
                }),
            };

            // Migrate without any changes and without reinitializing (i.e. same state version)
            migrate(deps.as_mut(), mock_env(), migrate_msg).unwrap();

            let actual_wasm_client_state_any_bz =
                deps.storage.get(HOST_CLIENT_STATE_KEY.as_bytes()).unwrap();
            let actual_wasm_client_state_any =
                Any::decode(actual_wasm_client_state_any_bz.as_slice()).unwrap();
            let wasm_client_state =
                WasmClientState::decode(actual_wasm_client_state_any.value.as_slice()).unwrap();
            // verify checksum hasn't changed
            assert_eq!(msg_copy.checksum, wasm_client_state.checksum);
            // verify latest height hasn't changed
            assert_eq!(
                wasm_client_state.latest_height.unwrap().revision_height,
                client_state.latest_slot
            );
            // verify fork parameters have changed
            let eth_client_state: EthClientState =
                serde_json::from_slice(&wasm_client_state.data).unwrap();
            assert_eq!(eth_client_state.latest_slot, client_state.latest_slot);
            assert_ne!(
                eth_client_state.fork_parameters.electra.epoch,
                client_state.fork_parameters.electra.epoch
            );
            assert_eq!(eth_client_state.fork_parameters.electra.epoch, 5000);
        }
    }
}
