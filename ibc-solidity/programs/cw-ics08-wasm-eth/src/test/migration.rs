// SPDX-License-Identifier: Apache-2.0

pub mod v1_3_0 {
    use std::marker::PhantomData;

    use alloy_primitives::B256;
    use cosmwasm_std::{
        coins,
        testing::{
            message_info, mock_dependencies, mock_env, MockApi, MockQuerier,
            MockQuerierCustomHandlerResult, MockStorage,
        },
        Binary, Env, OwnedDeps, SystemResult, Timestamp,
    };
    use ethereum_light_client::{
        header::Header,
        test_utils::{
            bls_verifier::{aggreagate, fast_aggregate_verify},
            fixtures::{self, get_packet_paths, RelayerMessages, StepsFixture},
        },
    };
    use ethereum_light_client_v1_3_0::{
        client_state::ClientState as ClientStateV1_3, header::Header as HeaderV1_3,
        test_utils::fixtures::RelayerMessages as RelayerMessagesV1_3,
    };
    use ethereum_types::consensus::{
        bls::{BlsPublicKey, BlsSignature},
        fork::ForkParameters,
    };
    use prost::Message;

    use cw_ics08_wasm_eth_v1_3_0::{
        contract as contract_v1_3, custom_query::EthereumCustomQuery, msg as msg_v1_3,
    };
    use ibc_proto::ibc::lightclients::wasm::v1::ClientMessage;

    use crate::{contract, msg, state, test::helpers::mk_deps};

    pub fn custom_query_handler(query: &EthereumCustomQuery) -> MockQuerierCustomHandlerResult {
        match query {
            EthereumCustomQuery::AggregateVerify {
                public_keys,
                message,
                signature,
            } => {
                let public_keys = public_keys
                    .iter()
                    .map(|pk| pk.as_ref().try_into().unwrap())
                    .collect::<Vec<BlsPublicKey>>();
                let message = B256::try_from(message.as_slice()).unwrap();
                let signature = BlsSignature::try_from(signature.as_slice()).unwrap();

                fast_aggregate_verify(&public_keys, message, signature).unwrap();

                SystemResult::Ok(cosmwasm_std::ContractResult::Ok::<Binary>(
                    serde_json::to_vec(&true).unwrap().into(),
                ))
            }
            EthereumCustomQuery::Aggregate { public_keys } => {
                let public_keys = public_keys
                    .iter()
                    .map(|pk| pk.as_ref().try_into().unwrap())
                    .collect::<Vec<BlsPublicKey>>();

                let aggregate_pubkey = aggreagate(&public_keys).unwrap();

                SystemResult::Ok(cosmwasm_std::ContractResult::Ok::<Binary>(
                    serde_json::to_vec(&Binary::from(aggregate_pubkey.as_slice()))
                        .unwrap()
                        .into(),
                ))
            }
        }
    }

    pub fn mk_deps_v1_3(
    ) -> OwnedDeps<MockStorage, MockApi, MockQuerier<EthereumCustomQuery>, EthereumCustomQuery>
    {
        let deps = mock_dependencies();

        OwnedDeps {
            storage: deps.storage,
            api: deps.api,
            querier: MockQuerier::<EthereumCustomQuery>::new(&[])
                .with_custom_handler(custom_query_handler),
            custom_query_type: PhantomData,
        }
    }

    fn env_at_signature_slot(client_state: &ClientStateV1_3, signature_slot: u64) -> Env {
        let mut env = mock_env();
        env.block.time =
            Timestamp::from_seconds(client_state.compute_timestamp_at_slot(signature_slot) + 1000);
        env
    }

    /// This test runs through the e2e test scenario defined in the interchaintest:
    /// `Test_Gloas_Fork`. A client running v1.3.0 on Fulu is given the Gloas fork parameters before
    /// the fork by the migration, and is then updated across the fork.
    #[allow(clippy::too_many_lines)]
    #[test]
    fn test_migrate_from_v1_3_0_and_update_across_gloas_fork() {
        let mut deps = mk_deps_v1_3();

        let creator = deps.api.addr_make("creator");
        let info = message_info(&creator, &coins(1, "uatom"));

        let fixture: StepsFixture = fixtures::load("Test_Gloas_Fork");

        // Initialize v1.3.0
        let initial_state: serde_json::Value = fixture.get_data_at_step(0);
        let client_state_v1_3: ClientStateV1_3 =
            serde_json::from_value(initial_state["client_state"].clone()).unwrap();
        // v1.3.0 requires storage_root to be present, which the fixture doesn't record
        let mut consensus_state = initial_state["consensus_state"].clone();
        consensus_state["storage_root"] = consensus_state["state_root"].clone();
        contract_v1_3::instantiate(
            deps.as_mut(),
            mock_env(),
            info,
            msg_v1_3::InstantiateMsg {
                client_state: serde_json::to_vec(&client_state_v1_3).unwrap().into(),
                consensus_state: serde_json::to_vec(&consensus_state).unwrap().into(),
                checksum: b"checksum".into(),
            },
        )
        .unwrap();

        // Update the client with v1.3.0 on Fulu
        let relayer_messages: RelayerMessagesV1_3 = fixture.get_data_at_step(1);
        let (update_client_msgs, ..) = relayer_messages.get_sdk_msgs();
        assert_ne!(update_client_msgs.len(), 0);
        for msg in update_client_msgs {
            let header_bz = ClientMessage::decode(msg.client_message.unwrap().value.as_slice())
                .unwrap()
                .data;
            let header: HeaderV1_3 = serde_json::from_slice(&header_bz).unwrap();
            let env =
                env_at_signature_slot(&client_state_v1_3, header.consensus_update.signature_slot);

            contract_v1_3::query(
                deps.as_ref(),
                env.clone(),
                msg_v1_3::QueryMsg::VerifyClientMessage(msg_v1_3::VerifyClientMessageMsg {
                    client_message: Binary::from(header_bz.clone()),
                }),
            )
            .unwrap();
            contract_v1_3::sudo(
                deps.as_mut(),
                env,
                msg_v1_3::SudoMsg::UpdateState(msg_v1_3::UpdateStateMsg {
                    client_message: Binary::from(header_bz),
                }),
            )
            .unwrap();
        }

        let mut new_deps = mk_deps();
        new_deps.storage = deps.storage;

        // Migrate to the current version with the Gloas fork parameters. The state of v1.3.0 doesn't
        // have the Gloas fork.
        let fork_parameters: ForkParameters = fixture.get_data_at_step(2);
        let gloas_fork_epoch = fork_parameters.gloas.epoch;
        contract::migrate(
            new_deps.as_mut(),
            mock_env(),
            msg::MigrateMsg {
                migration: msg::Migration::UpdateForkParameters(fork_parameters.clone()),
            },
        )
        .unwrap();

        let client_state = state::get_eth_client_state(&new_deps.storage).unwrap();
        assert_eq!(client_state.fork_parameters, fork_parameters);
        assert!(client_state.compute_epoch_at_slot(client_state.latest_slot) < gloas_fork_epoch);

        // Update the client across the fork
        let relayer_messages: RelayerMessages = fixture.get_data_at_step(3);
        let (update_client_msgs, recv_msgs, _, _) = relayer_messages.get_sdk_msgs();
        assert_ne!(update_client_msgs.len(), 0);
        assert_eq!(1, recv_msgs.len());

        let mut last_header = None;
        for msg in update_client_msgs {
            let header_bz = ClientMessage::decode(msg.client_message.unwrap().value.as_slice())
                .unwrap()
                .data;
            let header: Header = serde_json::from_slice(&header_bz).unwrap();
            let mut env = mock_env();
            env.block.time = Timestamp::from_seconds(
                client_state.compute_timestamp_at_slot(header.consensus_update.signature_slot())
                    + 1000,
            );

            contract::query(
                new_deps.as_ref(),
                env.clone(),
                msg::QueryMsg::VerifyClientMessage(msg::VerifyClientMessageMsg {
                    client_message: Binary::from(header_bz.clone()),
                }),
            )
            .unwrap();
            contract::sudo(
                new_deps.as_mut(),
                env,
                msg::SudoMsg::UpdateState(msg::UpdateStateMsg {
                    client_message: Binary::from(header_bz),
                }),
            )
            .unwrap();
            last_header = Some(header);
        }

        let stored_client_state = state::get_eth_client_state(&new_deps.storage).unwrap();
        assert!(
            client_state.compute_epoch_at_slot(stored_client_state.latest_slot) >= gloas_fork_epoch
        );
        assert_eq!(
            stored_client_state.latest_execution_block_number,
            last_header
                .unwrap()
                .finalized_execution()
                .unwrap()
                .block_number
        );

        // Verify the packet against the execution state of the Gloas consensus state
        let recv_msg = recv_msgs[0].clone();
        let (path, value, _) = get_packet_paths(recv_msg.packet.unwrap());
        contract::sudo(
            new_deps.as_mut(),
            mock_env(),
            msg::SudoMsg::VerifyMembership(msg::VerifyMembershipMsg {
                height: msg::Height {
                    revision_number: 0,
                    revision_height: recv_msg.proof_height.unwrap().revision_height,
                },
                delay_time_period: 0,
                delay_block_period: 0,
                proof: Binary::from(recv_msg.proof_commitment),
                merkle_path: msg::MerklePath {
                    key_path: vec![Binary::from(path)],
                },
                value: Binary::from(value),
            }),
        )
        .unwrap();
    }
}
