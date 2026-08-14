// SPDX-License-Identifier: Apache-2.0

//! This module defines [`TxBuilder`] which is responsible for building transactions to be sent to
//! the Cosmos SDK chain from events received from Ethereum.

use std::{collections::HashMap, time::Duration};

use alloy::{
    hex,
    network::Ethereum,
    primitives::{Address, U256},
    providers::Provider,
};
use anyhow::Result;
use ethereum_apis::{beacon_api::client::BeaconApiClient, eth_api::client::EthApiClient};
use ethereum_light_client::{
    client_state::ClientState,
    consensus_state::ConsensusState,
    header::{ActiveSyncCommittee, Header},
};
use ethereum_types::consensus::{
    light_client_header::{LightClientFinalityUpdate, LightClientUpdate},
    sync_committee::SyncCommittee,
};
use ibc_eureka_solidity_types::ics26::{router::routerInstance, ICS26_IBC_STORAGE_SLOT};
use ibc_eureka_utils::rpc::TendermintRpcExt;
use ibc_proto_eureka::{
    cosmos::tx::v1beta1::TxBody,
    google::protobuf::Any,
    ibc::{
        core::client::v1::{Height, MsgCreateClient, MsgUpdateClient},
        lightclients::wasm::v1::{
            ClientMessage, ClientState as WasmClientState, ConsensusState as WasmConsensusState,
        },
    },
};
use prost::Message;
use tendermint_rpc::{Client, HttpClient};

use proof_api_lib::{
    aggregator::Aggregator,
    chain::{CosmosSdk, EthEureka},
    events::EurekaEventWithHeight,
    tx_builder::TxBuilderService,
    utils::{
        cosmos,
        cosmos_attested::{
            build_attestor_create_client_tx, build_attestor_relay_events_tx,
            build_attestor_update_client_tx,
        },
        wait_for_condition, RelayEventsParams,
    },
};

/// The `TxBuilder` produces txs to [`CosmosSdk`] based on events from [`EthEureka`].
pub struct TxBuilder<P>
where
    P: Provider + Clone,
{
    /// The ETH API client.
    pub eth_client: EthApiClient<P>,
    /// The Beacon API client.
    pub beacon_api_client: BeaconApiClient,
    /// The IBC Eureka router instance.
    pub ics26_router: routerInstance<P, Ethereum>,
    /// The HTTP client for the Cosmos SDK.
    pub tm_client: HttpClient,
    /// The signer address for the Cosmos messages.
    pub signer_address: String,
}

/// The `MockTxBuilder` produces txs to [`CosmosSdk`] based on events from [`EthEureka`]
/// for testing purposes.
pub struct MockTxBuilder<P: Provider + Clone> {
    /// The ETH API client.
    pub eth_client: EthApiClient<P>,
    /// The IBC Eureka router instance.
    pub ics26_router: routerInstance<P, Ethereum>,
    /// The signer address for the Cosmos messages.
    pub signer_address: String,
}

impl<P> TxBuilder<P>
where
    P: Provider + Clone,
{
    /// Create a new [`TxBuilder`] instance.
    pub fn new(
        ics26_address: Address,
        provider: P,
        beacon_api_url: String,
        tm_client: HttpClient,
        signer_address: String,
    ) -> Self {
        Self {
            eth_client: EthApiClient::new(provider.clone()),
            beacon_api_client: BeaconApiClient::new(beacon_api_url),
            ics26_router: routerInstance::new(ics26_address, provider),
            tm_client,
            signer_address,
        }
    }

    /// Fetch the Ethereum client state from the light client on cosmos.
    /// # Errors
    /// Returns an error if the client state cannot be fetched or decoded.
    pub async fn ethereum_client_state(&self, client_id: String) -> Result<ClientState> {
        let wasm_client_state_any = self.tm_client.client_state(client_id).await?;
        let wasm_client_state = WasmClientState::decode(wasm_client_state_any.value.as_slice())?;
        Ok(serde_json::from_slice(&wasm_client_state.data)?)
    }

    async fn get_sync_commitee_for_finalized_slot(
        &self,
        finalized_slot: u64,
    ) -> Result<SyncCommittee> {
        let block_root = self
            .beacon_api_client
            .beacon_block_root(&format!("{finalized_slot}"))
            .await?;
        let light_client_bootstrap = self
            .beacon_api_client
            .light_client_bootstrap(&block_root)
            .await?
            .data;
        Ok(light_client_bootstrap.current_sync_committee)
    }

    /// Fetches light client updates from the Beacon API for synchronizing between the trusted and target periods.
    ///
    /// This function calculates the sync committee periods for both the trusted state and the finality update,
    /// then retrieves all light client updates needed to advance the light client from the trusted period
    /// to the target period. These updates contain validator signatures and sync committee data needed
    /// to verify the consensus transition.
    async fn get_light_client_updates(
        &self,
        client_state: &ClientState,
        finality_update: LightClientFinalityUpdate,
    ) -> Result<Vec<LightClientUpdate>> {
        let trusted_period =
            client_state.compute_sync_committee_period_at_slot(client_state.latest_slot);

        let target_period = client_state
            .compute_sync_committee_period_at_slot(finality_update.finalized_header.beacon.slot);

        tracing::debug!(
            "Getting light client updates from period {} to {}",
            trusted_period,
            target_period - trusted_period + 1
        );
        Ok(self
            .beacon_api_client
            .light_client_updates(trusted_period, target_period - trusted_period + 1)
            .await?
            .into_iter()
            .map(|resp| resp.data)
            .collect::<Vec<_>>())
    }

    async fn wait_for_light_client_readiness(&self, target_block_number: u64) -> Result<()> {
        // Wait until we find a finality update that meets our criteria and capture it
        // This way we avoid making an extra call at the end
        wait_for_condition(
            Duration::from_mins(45),
            Duration::from_secs(10),
            || async {
                tracing::debug!(
                    "Waiting for finality beyond target block number: {}",
                    target_block_number
                );

                let finality_update = self.beacon_api_client.finality_update().await?.data;
                if finality_update.finalized_header.execution.block_number < target_block_number {
                    tracing::info!(
                        "Waiting for finality: current finality execution block number: {}, Target execution block number: {}",
                        finality_update.finalized_header.execution.block_number,
                        target_block_number
                    );
                    return Ok(false);
                }

                // Return the update itself when the condition is met
                tracing::info!(
                    "Finality update found at execution block number: {}",
                    finality_update.finalized_header.execution.block_number
                );
                Ok(true)
            },
        )
        .await?;

        Ok(())
    }

    #[tracing::instrument(skip_all)]
    async fn get_update_headers(&self, ethereum_client_state: &ClientState) -> Result<Vec<Header>> {
        let finality_update = self.beacon_api_client.finality_update().await?.data;

        let mut headers = vec![];

        let light_client_updates = self
            .get_light_client_updates(ethereum_client_state, finality_update.clone())
            .await?;

        let mut latest_trusted_slot = ethereum_client_state.latest_slot;
        let mut latest_period =
            ethereum_client_state.compute_sync_committee_period_at_slot(latest_trusted_slot);
        tracing::debug!("Latest trusted sync committee period: {}", latest_period);

        for update in light_client_updates {
            let update_finalized_slot = update.finalized_header.beacon.slot;

            tracing::debug!(
                "Processing light client update for finalized slot {} with trusted slot {}",
                update_finalized_slot,
                latest_trusted_slot
            );

            if update_finalized_slot <= latest_trusted_slot {
                tracing::debug!(
                    "Skipping unnecessary update for slot {}",
                    update_finalized_slot
                );
                continue;
            }

            let update_period =
                ethereum_client_state.compute_sync_committee_period_at_slot(update_finalized_slot);
            if update_period == latest_period {
                tracing::debug!(
                    "Skipping header with same sync committee period for slot {}",
                    update_finalized_slot
                );
                continue;
            }

            let previous_next_sync_committee = self
                .get_sync_commitee_for_finalized_slot(update_finalized_slot)
                .await?;

            let active_sync_committee = ActiveSyncCommittee::Next(previous_next_sync_committee);
            let header = Header {
                active_sync_committee,
                consensus_update: update,
                trusted_slot: latest_trusted_slot,
            };
            tracing::debug!(
                "Added header for slot from light client updates {}
                Header: {:?}",
                update_finalized_slot,
                header
            );
            headers.push(header);
            latest_period = update_period;
            latest_trusted_slot = update_finalized_slot;
        }

        // If the latest header is earlier than the finality update, we need to add a header for the finality update.
        if headers.last().is_none_or(|last_header| {
            last_header.consensus_update.finalized_header.beacon.slot
                < finality_update.finalized_header.beacon.slot
        }) {
            let finality_update_finalized_slot = finality_update.finalized_header.beacon.slot;
            let finality_update_sync_committee = self
                .get_sync_commitee_for_finalized_slot(finality_update.attested_header.beacon.slot)
                .await?;
            let active_sync_committee =
                ActiveSyncCommittee::Current(finality_update_sync_committee);

            let header = Header {
                active_sync_committee,
                consensus_update: finality_update.into(),
                trusted_slot: latest_trusted_slot,
            };
            tracing::debug!(
                "Added header for slot from finality update {}: {}",
                finality_update_finalized_slot,
                serde_json::to_string(&header)?
            );
            headers.push(header);
        }

        Ok(headers)
    }

    async fn wait_for_cosmos_chain_to_catch_up(
        &self,
        ethereum_client_state: &ClientState,
        latest_signature_slot: u64,
    ) -> Result<(), anyhow::Error> {
        wait_for_condition(Duration::from_mins(15), Duration::from_secs(5), || async {
            let latests_tm_block = self.tm_client.latest_block().await?;
            let latest_onchain_timestamp = latests_tm_block.block.header.time.unix_timestamp();
            let calculated_slot = ethereum_client_state
                .compute_slot_at_timestamp(latest_onchain_timestamp.try_into().unwrap())
                .unwrap();
            tracing::debug!(
                "Waiting for target chain to catch up to slot {}",
                calculated_slot
            );
            Ok(calculated_slot > latest_signature_slot)
        })
        .await?;
        Ok(())
    }
}

/// The key for the checksum hex in the parameters map.
const CHECKSUM_HEX: &str = "checksum_hex";

#[async_trait::async_trait]
impl<P> TxBuilderService<EthEureka, CosmosSdk> for TxBuilder<P>
where
    P: Provider + Clone,
{
    #[tracing::instrument(skip_all, err(Debug))]
    async fn relay_events(
        &self,
        src_events: Vec<EurekaEventWithHeight>,
        dest_events: Vec<EurekaEventWithHeight>,
        src_client_id: String,
        dst_client_id: String,
        src_packet_seqs: Vec<u64>,
        dst_packet_seqs: Vec<u64>,
    ) -> Result<Vec<u8>> {
        let now_since_unix = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?;
        let mut ethereum_client_state = self.ethereum_client_state(dst_client_id.clone()).await?;
        let latest_block_number = self.eth_client.get_block_number().await?;

        let max_src_block_number = src_events.iter().map(|e| e.height).max();

        let mut timeout_msgs = cosmos::target_events_to_timeout_msgs(
            dest_events,
            &src_client_id,
            &dst_client_id,
            &dst_packet_seqs,
            &self.signer_address,
            now_since_unix.as_secs(),
        );

        let (mut recv_msgs, mut ack_msgs) = cosmos::src_events_to_recv_and_ack_msgs(
            src_events,
            &src_client_id,
            &dst_client_id,
            &src_packet_seqs,
            &dst_packet_seqs,
            &self.signer_address,
            now_since_unix.as_secs(),
        );

        let max_timeout_slot = timeout_msgs
            .iter()
            .filter_map(|e| {
                ethereum_client_state
                    .compute_slot_at_timestamp(e.packet.as_ref()?.timeout_timestamp)
                    .and_then(|slot| slot.checked_add(1))
            })
            .max();

        let max_timeout_block_number = if let Some(max_timeout_slot) = max_timeout_slot {
            Some(
                self.beacon_api_client
                    .beacon_block(&format!("{max_timeout_slot}"))
                    .await
                    .map_err(|e| {
                        anyhow::anyhow!(
                            "Failed to get beacon block for timeout slot {max_timeout_slot}: {e}",
                        )
                    })?
                    .message
                    .body
                    .execution_payload
                    .block_number,
            )
        } else {
            None
        };

        let minimum_block_number = max_src_block_number
            .into_iter()
            .chain(max_timeout_block_number.into_iter())
            .max()
            .unwrap_or(latest_block_number);

        tracing::info!(
            "Relaying events from Ethereum to Cosmos for client {}, target block number: {}, client state latest slot: {}",
            dst_client_id,
            minimum_block_number,
            ethereum_client_state.latest_slot,
        );

        // get updates if necessary
        let headers = if minimum_block_number > ethereum_client_state.latest_execution_block_number
        {
            self.wait_for_light_client_readiness(minimum_block_number)
                .await?;
            // Update the client state and consensus state, in case they have changed while we were waiting
            ethereum_client_state = self.ethereum_client_state(dst_client_id.clone()).await?;
            self.get_update_headers(&ethereum_client_state).await?
        } else {
            vec![]
        };

        let proof_slot = headers
            .last()
            .map_or(ethereum_client_state.latest_slot, |h| {
                h.consensus_update.finalized_header.beacon.slot
            });

        cosmos::inject_ethereum_proofs(
            &mut recv_msgs,
            &mut ack_msgs,
            &mut timeout_msgs,
            &self.eth_client,
            &self.beacon_api_client,
            &ethereum_client_state.ibc_contract_address.to_string(),
            ethereum_client_state.ibc_commitment_slot,
            proof_slot,
        )
        .await?;

        let update_msgs = headers
            .iter()
            .map(|header| -> Result<MsgUpdateClient> {
                let header_bz = serde_json::to_vec(&header)?;
                let client_msg = Any::from_msg(&ClientMessage { data: header_bz })?;
                Ok(MsgUpdateClient {
                    client_id: dst_client_id.clone(),
                    client_message: Some(client_msg),
                    signer: self.signer_address.clone(),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        let all_msgs = update_msgs
            .into_iter()
            .map(|m| Any::from_msg(&m))
            .chain(timeout_msgs.iter().map(Any::from_msg))
            .chain(recv_msgs.iter().map(Any::from_msg))
            .chain(ack_msgs.iter().map(Any::from_msg))
            .collect::<Result<Vec<_>, _>>()?;

        let tx_body = TxBody {
            messages: all_msgs,
            ..Default::default()
        };

        // If we have update clients, we do a final check to make sure the target chain
        // has caught up to update's signature slot
        if let Some(last_header) = headers.last() {
            self.wait_for_cosmos_chain_to_catch_up(
                &ethereum_client_state,
                last_header.consensus_update.signature_slot,
            )
            .await?;
        }

        let initial_period = ethereum_client_state
            .compute_sync_committee_period_at_slot(ethereum_client_state.latest_slot);
        let latest_period = ethereum_client_state.compute_sync_committee_period_at_slot(proof_slot);
        tracing::info!(
            "Relay events summary:
                client id: {},
                recv events processed: #{},
                ack events processed: #{},
                timeout events processed: #{},
                initial slot: {},
                latest trusted slot (after updates): {},
                initial period: {},
                latest period: {},
                number of headers: #{}",
            dst_client_id,
            recv_msgs.len(),
            ack_msgs.len(),
            timeout_msgs.len(),
            ethereum_client_state.latest_slot,
            proof_slot,
            initial_period,
            latest_period,
            headers.len()
        );

        Ok(tx_body.encode_to_vec())
    }

    #[tracing::instrument(skip_all, err(Debug))]
    async fn create_client(&self, parameters: &HashMap<String, String>) -> Result<Vec<u8>> {
        parameters
            .keys()
            .find(|k| k.as_str() != CHECKSUM_HEX)
            .map_or(Ok(()), |param| {
                Err(anyhow::anyhow!(
                    "Unexpected parameter: `{param}`, only `{CHECKSUM_HEX}` is allowed"
                ))
            })?;

        let genesis = self.beacon_api_client.genesis().await?.data;
        let spec = self.beacon_api_client.spec().await?.data;
        let beacon_block = self
            .beacon_api_client
            .beacon_block("finalized")
            .await?
            .message;

        tracing::info!("Creating client at slot: {}", beacon_block.slot);

        let block_root = self
            .beacon_api_client
            .beacon_block_root(&format!("{}", beacon_block.slot))
            .await?;

        let bootstrap = self
            .beacon_api_client
            .light_client_bootstrap(&block_root)
            .await?
            .data;

        if bootstrap.header.execution.block_number
            != beacon_block.body.execution_payload.block_number
        {
            anyhow::bail!(
                "Light client bootstrap block number does not match execution block number"
            );
        }

        let eth_client_state = ClientState {
            chain_id: self.ics26_router.provider().get_chain_id().await?,
            genesis_validators_root: genesis.genesis_validators_root,
            min_sync_committee_participants: spec.sync_committee_size.div_ceil(3),
            sync_committee_size: spec.sync_committee_size,
            genesis_time: genesis.genesis_time,
            genesis_slot: spec.genesis_slot,
            fork_parameters: spec.to_fork_parameters(),
            seconds_per_slot: spec.seconds_per_slot,
            slots_per_epoch: spec.slots_per_epoch,
            epochs_per_sync_committee_period: spec.epochs_per_sync_committee_period,
            latest_slot: bootstrap.header.beacon.slot,
            is_frozen: false,
            ibc_commitment_slot: U256::from_be_slice(&ICS26_IBC_STORAGE_SLOT),
            ibc_contract_address: *self.ics26_router.address(),
            latest_execution_block_number: bootstrap.header.execution.block_number,
        };
        let client_state = WasmClientState {
            data: serde_json::to_vec(&eth_client_state)?,
            checksum: hex::decode(
                parameters
                    .get(CHECKSUM_HEX)
                    .ok_or_else(|| anyhow::anyhow!("Missing `{CHECKSUM_HEX}` parameter"))?,
            )?,
            latest_height: Some(Height {
                revision_number: 0,
                revision_height: eth_client_state.latest_slot,
            }),
        };

        let latest_period =
            eth_client_state.compute_sync_committee_period_at_slot(eth_client_state.latest_slot);
        let next_sync_committee = self
            .beacon_api_client
            .light_client_updates(latest_period, 1)
            .await?
            .pop()
            .ok_or_else(|| anyhow::anyhow!("No light client updates found for the latest period"))?
            .data
            .next_sync_committee
            .ok_or_else(|| {
                anyhow::anyhow!("No next sync committee found in the light client update")
            })?;

        let eth_consensus_state = ConsensusState {
            slot: eth_client_state.latest_slot,
            state_root: bootstrap.header.execution.state_root,
            timestamp: bootstrap.header.execution.timestamp,
            current_sync_committee: bootstrap
                .current_sync_committee
                .to_summarized_sync_committee(),
            next_sync_committee: Some(next_sync_committee.to_summarized_sync_committee()),
        };
        let consensus_state = WasmConsensusState {
            data: serde_json::to_vec(&eth_consensus_state)?,
        };

        let msg = MsgCreateClient {
            client_state: Some(Any::from_msg(&client_state)?),
            consensus_state: Some(Any::from_msg(&consensus_state)?),
            signer: self.signer_address.clone(),
        };

        Ok(TxBody {
            messages: vec![Any::from_msg(&msg)?],
            ..Default::default()
        }
        .encode_to_vec())
    }

    #[tracing::instrument(skip_all, err(Debug))]
    async fn update_client(&self, dst_client_id: String) -> Result<Vec<u8>> {
        let ethereum_client_state = self.ethereum_client_state(dst_client_id.clone()).await?;
        let finality_update = self.beacon_api_client.finality_update().await?;
        let latest_finalized_block_number =
            finality_update.data.finalized_header.execution.block_number;

        if latest_finalized_block_number <= ethereum_client_state.latest_execution_block_number {
            tracing::warn!(
                "No updates needed for client {}: latest finalized block number: {}, latest execution block number: {}",
                dst_client_id,
                latest_finalized_block_number,
                ethereum_client_state.latest_execution_block_number
            );

            return Err(anyhow::anyhow!(
                "No updates needed for client {}: latest finalized block number: {}, latest execution block number: {}",
                dst_client_id,
                latest_finalized_block_number,
                ethereum_client_state.latest_execution_block_number
            ));
        }

        tracing::info!(
            "Generating tx to update client from block number: {} to block number: {}",
            ethereum_client_state.latest_execution_block_number,
            latest_finalized_block_number
        );

        let headers = self.get_update_headers(&ethereum_client_state).await?;
        let update_msgs = headers
            .iter()
            .map(|header| -> Result<MsgUpdateClient> {
                let header_bz = serde_json::to_vec(&header)?;
                let client_msg = Any::from_msg(&ClientMessage { data: header_bz })?;
                Ok(MsgUpdateClient {
                    client_id: dst_client_id.clone(),
                    client_message: Some(client_msg),
                    signer: self.signer_address.clone(),
                })
            })
            .collect::<Result<Vec<_>, _>>()?;

        // If we have update clients, we do a final check to make sure the target chain
        // has caught up to update's signature slot
        if let Some(last_header) = headers.last() {
            self.wait_for_cosmos_chain_to_catch_up(
                &ethereum_client_state,
                last_header.consensus_update.signature_slot,
            )
            .await?;
        }

        let proof_slot = headers
            .last()
            .map_or(ethereum_client_state.latest_slot, |h| {
                h.consensus_update.finalized_header.beacon.slot
            });
        let initial_period = ethereum_client_state
            .compute_sync_committee_period_at_slot(ethereum_client_state.latest_slot);
        let latest_period = ethereum_client_state.compute_sync_committee_period_at_slot(proof_slot);
        tracing::info!(
            "Update client summary:
                client id: {},
                initial slot: {},
                latest trusted slot (after updates): {},
                initial period: {},
                latest period: {},
                number of headers: #{}",
            dst_client_id,
            ethereum_client_state.latest_slot,
            proof_slot,
            initial_period,
            latest_period,
            headers.len()
        );

        Ok(TxBody {
            messages: update_msgs
                .into_iter()
                .map(|m| Any::from_msg(&m))
                .collect::<Result<Vec<_>, _>>()?,
            ..Default::default()
        }
        .encode_to_vec())
    }
}

impl<P: Provider + Clone> MockTxBuilder<P> {
    /// Create a new [`MockTxBuilder`] instance for testing.
    pub fn new(ics26_address: Address, provider: P, signer_address: String) -> Self {
        Self {
            eth_client: EthApiClient::new(provider.clone()),
            ics26_router: routerInstance::new(ics26_address, provider),
            signer_address,
        }
    }
}

#[async_trait::async_trait]
impl<P> TxBuilderService<EthEureka, CosmosSdk> for MockTxBuilder<P>
where
    P: Provider + Clone,
{
    #[tracing::instrument(skip_all, err(Debug))]
    async fn relay_events(
        &self,
        src_events: Vec<EurekaEventWithHeight>,
        dest_events: Vec<EurekaEventWithHeight>,
        src_client_id: String,
        dst_client_id: String,
        src_packet_seqs: Vec<u64>,
        dst_packet_seqs: Vec<u64>,
    ) -> Result<Vec<u8>> {
        tracing::info!(
            "Relaying events from Ethereum to Cosmos for client {}",
            dst_client_id
        );

        let now_since_unix = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?;

        let mut timeout_msgs = cosmos::target_events_to_timeout_msgs(
            dest_events,
            &src_client_id,
            &dst_client_id,
            &dst_packet_seqs,
            &self.signer_address,
            now_since_unix.as_secs(),
        );

        let (mut recv_msgs, mut ack_msgs) = cosmos::src_events_to_recv_and_ack_msgs(
            src_events,
            &src_client_id,
            &dst_client_id,
            &src_packet_seqs,
            &dst_packet_seqs,
            &self.signer_address,
            now_since_unix.as_secs(),
        );

        tracing::debug!("Timeout messages: #{}", timeout_msgs.len());
        tracing::debug!("Recv messages: #{}", recv_msgs.len());
        tracing::debug!("Ack messages: #{}", ack_msgs.len());

        cosmos::inject_mock_proofs(&mut recv_msgs, &mut ack_msgs, &mut timeout_msgs);

        let all_msgs = timeout_msgs
            .into_iter()
            .map(|m| Any::from_msg(&m))
            .chain(recv_msgs.into_iter().map(|m| Any::from_msg(&m)))
            .chain(ack_msgs.into_iter().map(|m| Any::from_msg(&m)))
            .collect::<Result<Vec<_>, _>>()?;

        let tx_body = TxBody {
            messages: all_msgs,
            ..Default::default()
        };
        Ok(tx_body.encode_to_vec())
    }

    #[tracing::instrument(skip_all, err(Debug))]
    async fn create_client(&self, parameters: &HashMap<String, String>) -> Result<Vec<u8>> {
        parameters
            .keys()
            .find(|k| k.as_str() != CHECKSUM_HEX)
            .map_or(Ok(()), |param| {
                Err(anyhow::anyhow!(
                    "Unexpected parameter: `{param}`, only `{CHECKSUM_HEX}` is allowed"
                ))
            })?;

        let client_state = WasmClientState {
            data: b"test".to_vec(),
            checksum: hex::decode(
                parameters
                    .get(CHECKSUM_HEX)
                    .ok_or_else(|| anyhow::anyhow!("Missing `{CHECKSUM_HEX}` parameter"))?,
            )?,
            latest_height: Some(Height {
                revision_number: 0,
                revision_height: 1,
            }),
        };
        let consensus_state = WasmConsensusState {
            data: b"test".to_vec(),
        };

        let msg = MsgCreateClient {
            client_state: Some(Any::from_msg(&client_state)?),
            consensus_state: Some(Any::from_msg(&consensus_state)?),
            signer: self.signer_address.clone(),
        };

        Ok(TxBody {
            messages: vec![Any::from_msg(&msg)?],
            ..Default::default()
        }
        .encode_to_vec())
    }

    #[tracing::instrument(skip_all, err(Debug))]
    async fn update_client(&self, dst_client_id: String) -> Result<Vec<u8>> {
        tracing::info!(
            "Generating tx to update mock light client: {}",
            dst_client_id
        );

        let consensus_state = WasmConsensusState {
            data: b"test".to_vec(),
        };
        let msg = MsgUpdateClient {
            client_id: dst_client_id,
            client_message: Some(Any::from_msg(&consensus_state)?),
            signer: self.signer_address.clone(),
        };

        Ok(TxBody {
            messages: vec![Any::from_msg(&msg)?],
            ..Default::default()
        }
        .encode_to_vec())
    }
}

/// Transaction builder for attested relay to Cosmos.
pub struct AttestedTxBuilder {
    aggregator: Aggregator,
    ics26_router: Address,
    signer_address: String,
}

impl AttestedTxBuilder {
    /// Create a new [`AttestedTxBuilder`] instance.
    #[must_use]
    pub const fn new(
        aggregator: Aggregator,
        ics26_router: Address,
        signer_address: String,
    ) -> Self {
        Self {
            aggregator,
            ics26_router,
            signer_address,
        }
    }

    /// Returns the ICS26 router address.
    #[must_use]
    pub const fn ics26_router(&self) -> &Address {
        &self.ics26_router
    }

    /// Relay events from source chain to Cosmos using attestations.
    ///
    /// # Errors
    /// Returns an error if attestation retrieval or transaction building fails.
    pub async fn relay_events(&self, params: RelayEventsParams) -> Result<Vec<u8>> {
        build_attestor_relay_events_tx(&self.aggregator, params, &self.signer_address).await
    }

    /// Create a client on Cosmos using attestations.
    ///
    /// # Errors
    /// Returns an error if transaction building fails.
    pub fn create_client(&self, parameters: &HashMap<String, String>) -> Result<Vec<u8>> {
        build_attestor_create_client_tx(parameters, &self.signer_address)
    }

    /// Update a client on Cosmos using attestations.
    ///
    /// # Errors
    /// Returns an error if attestation retrieval or transaction building fails.
    pub async fn update_client(&self, dst_client_id: &str) -> Result<Vec<u8>> {
        build_attestor_update_client_tx(&self.aggregator, dst_client_id, &self.signer_address).await
    }
}
