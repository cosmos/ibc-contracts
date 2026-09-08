// SPDX-License-Identifier: Apache-2.0

//! This is a one-sided proof API module from Solana to a Cosmos SDK chain.

#![deny(clippy::nursery, clippy::pedantic, warnings, unused_crate_dependencies)]
#![allow(unused_crate_dependencies, missing_docs)]

pub mod tx_builder;

use std::collections::HashMap;

use ibc_eureka_utils::rpc::TendermintRpcExt;
use proof_api_lib::aggregator::{Aggregator, Config as AggregatorConfig};
use proof_api_lib::events::{EurekaEvent, EurekaEventWithHeight, SolanaEurekaEventWithHeight};
use proof_api_lib::listener::cosmos_sdk;
use proof_api_lib::listener::solana;
use proof_api_lib::listener::ChainListenerService;
use proof_api_lib::service_utils::parse_cosmos_tx_hashes;
use proof_api_lib::service_utils::parse_solana_tx_hashes;
use proof_api_lib::service_utils::to_tonic_status;
use proof_api_lib::tx_builder::TxBuilderService;
use proof_api_lib::utils::{wait_for_condition, RelayEventsParams};
use tendermint_rpc::HttpClient;
use tonic::{Request, Response};

use proof_api_core::{
    api::{self, proof_api_service_server::ProofApiService},
    modules::ProofApiModule,
};

enum SolanaToCosmosTxBuilder {
    Mock(tx_builder::MockTxBuilder),
    Attested(tx_builder::AttestedTxBuilder),
}

/// The `SolanaToCosmosProofApiModule` struct defines the Solana to Cosmos proof API module.
#[derive(Clone, Copy, Debug)]
pub struct SolanaToCosmosProofApiModule;

/// The `SolanaToCosmosProofApiModuleService` defines the proof API service from Solana to Cosmos.
struct SolanaToCosmosProofApiModuleService {
    /// The souce chain listener for Solana.
    pub src_listener: solana::ChainListener,
    /// The target chain listener for Cosmos SDK.
    pub target_listener: cosmos_sdk::ChainListener,
    /// The transaction builder from Solana to Cosmos.
    pub tx_builder: SolanaToCosmosTxBuilder,
}

/// The configuration for the Solana to Cosmos proof API module.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct SolanaToCosmosConfig {
    /// The Solana chain ID for identification.
    pub solana_chain_id: String,
    /// The Solana RPC URL.
    pub src_rpc_url: String,
    /// The target tendermint RPC URL.
    pub target_rpc_url: String,
    /// The address of the submitter on Cosmos.
    /// Required since cosmos messages require a signer address.
    pub signer_address: String,
    /// The Solana ICS26 router program ID.
    pub solana_ics26_program_id: String,
    /// Transaction builder mode.
    pub mode: TxBuilderMode,
}

/// Transaction builder mode configuration.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TxBuilderMode {
    /// Mock mode for testing without real proofs.
    Mock,
    /// Attested mode using aggregator attestations.
    Attested(AggregatorConfig),
}

impl SolanaToCosmosProofApiModuleService {
    async fn new(config: SolanaToCosmosConfig) -> anyhow::Result<Self> {
        let solana_ics26_program_id = config
            .solana_ics26_program_id
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid Solana program ID: {e}"))?;

        let src_listener =
            solana::ChainListener::new(config.src_rpc_url.clone(), solana_ics26_program_id);

        let target_listener =
            cosmos_sdk::ChainListener::new(HttpClient::from_rpc_url(&config.target_rpc_url));

        let tx_builder = match config.mode {
            TxBuilderMode::Mock => SolanaToCosmosTxBuilder::Mock(tx_builder::MockTxBuilder::new(
                src_listener.client().clone(),
                target_listener.client().clone(),
                config.signer_address,
                solana_ics26_program_id,
            )),
            TxBuilderMode::Attested(aggregator_config) => {
                let aggregator = Aggregator::from_config(aggregator_config)
                    .await
                    .map_err(|e| anyhow::anyhow!("failed to create aggregator: {e}"))?;
                SolanaToCosmosTxBuilder::Attested(tx_builder::AttestedTxBuilder::new(
                    aggregator,
                    config.signer_address,
                ))
            }
        };

        Ok(Self {
            src_listener,
            target_listener,
            tx_builder,
        })
    }
}

#[tonic::async_trait]
impl ProofApiService for SolanaToCosmosProofApiModuleService {
    async fn info(
        &self,
        _request: Request<api::InfoRequest>,
    ) -> Result<Response<api::InfoResponse>, tonic::Status> {
        tracing::debug!("Handling info request");

        Ok(Response::new(api::InfoResponse {
            target_chain: Some(api::Chain {
                chain_id: self
                    .target_listener
                    .chain_id()
                    .await
                    .map_err(to_tonic_status)?,
                ibc_version: "2".to_string(),
                ibc_contract: String::new(),
            }),
            source_chain: Some(api::Chain {
                chain_id: "solana-localnet".to_string(), // Solana doesn't have chain IDs like Cosmos
                ibc_version: "2".to_string(),
                ibc_contract: self.src_listener.ics26_program_id().to_string(),
            }),
            metadata: HashMap::default(),
        }))
    }

    async fn relay_by_tx(
        &self,
        request: Request<api::RelayByTxRequest>,
    ) -> Result<Response<api::RelayByTxResponse>, tonic::Status> {
        let inner_req = request.into_inner();
        tracing::debug!(
            "Relay request: {} source txs, {} timeout txs",
            inner_req.source_tx_ids.len(),
            inner_req.timeout_tx_ids.len()
        );

        let solana_tx_hashes = parse_solana_tx_hashes(inner_req.source_tx_ids)?;
        let timeout_txs = parse_cosmos_tx_hashes(inner_req.timeout_tx_ids)?;

        let solana_events = self
            .src_listener
            .fetch_tx_events(solana_tx_hashes)
            .await
            .map_err(to_tonic_status)?;

        tracing::debug!("Fetched {} src events", solana_events.len());

        let timeout_events = self
            .target_listener
            .fetch_tx_events(timeout_txs)
            .await
            .map_err(to_tonic_status)?;

        tracing::debug!("Fetched {} timeout events", timeout_events.len());

        // Solana finalization lags ~12s behind tip. We need a finalized slot
        // whose block time is past now so the non-membership proof is valid.
        let timeout_relay_height = if self.tx_builder.is_attested() && !timeout_events.is_empty() {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            let max_timeout = timeout_events
                .iter()
                .filter_map(|e| match &e.event {
                    EurekaEvent::SendPacket(packet) => Some(packet.timeoutTimestamp),
                    EurekaEvent::WriteAcknowledgement(..) => None,
                })
                .max()
                .unwrap_or(now);
            let cutoff = now.min(max_timeout);
            let src_listener = &self.src_listener;
            wait_for_condition(
                std::time::Duration::from_secs(24),
                std::time::Duration::from_secs(1),
                || async {
                    let (_, block_time) = src_listener.get_finalized_slot_with_time()?;
                    if block_time >= cutoff {
                        Ok(true)
                    } else {
                        tracing::debug!(
                            "Finalized block time ({block_time}) < cutoff ({cutoff}), waiting"
                        );
                        Ok(false)
                    }
                },
            )
            .await
            .map_err(to_tonic_status)?;
            let (slot, _) = self
                .src_listener
                .get_finalized_slot_with_time()
                .map_err(to_tonic_status)?;
            Some(slot)
        } else {
            None
        };

        let tx = self
            .tx_builder
            .relay_events(
                solana_events,
                timeout_events,
                timeout_relay_height,
                inner_req.src_client_id,
                inner_req.dst_client_id,
                inner_req.src_packet_sequences,
                inner_req.dst_packet_sequences,
            )
            .await
            .map_err(|e| tonic::Status::from_error(e.into()))?;

        tracing::debug!("Relay completed");

        Ok(Response::new(api::RelayByTxResponse {
            tx,
            address: String::new(),
        }))
    }

    async fn create_client(
        &self,
        request: Request<api::CreateClientRequest>,
    ) -> Result<Response<api::CreateClientResponse>, tonic::Status> {
        tracing::debug!("Handling create client request");

        let inner_req = request.into_inner();
        let tx = self
            .tx_builder
            .create_client(&inner_req.parameters)
            .await
            .map_err(|e| tonic::Status::from_error(e.into()))?;

        Ok(Response::new(api::CreateClientResponse {
            tx,
            address: String::new(),
        }))
    }

    async fn update_client(
        &self,
        request: Request<api::UpdateClientRequest>,
    ) -> Result<Response<api::UpdateClientResponse>, tonic::Status> {
        let inner_req = request.into_inner();
        let tx = self
            .tx_builder
            .update_client(inner_req.dst_client_id)
            .await
            .map_err(|e| tonic::Status::from_error(e.into()))?;

        Ok(Response::new(api::UpdateClientResponse {
            tx,
            address: String::new(),
        }))
    }
}

#[tonic::async_trait]
impl ProofApiModule for SolanaToCosmosProofApiModule {
    fn name(&self) -> &'static str {
        "solana_to_cosmos"
    }

    async fn create_service(
        &self,
        config: serde_json::Value,
    ) -> anyhow::Result<Box<dyn ProofApiService>> {
        let config: SolanaToCosmosConfig = serde_json::from_value(config)?;
        let service = SolanaToCosmosProofApiModuleService::new(config).await?;
        Ok(Box::new(service))
    }
}

impl SolanaToCosmosTxBuilder {
    #[allow(clippy::too_many_arguments)]
    async fn relay_events(
        &self,
        solana_src_events: Vec<SolanaEurekaEventWithHeight>,
        target_events: Vec<EurekaEventWithHeight>,
        timeout_relay_height: Option<u64>,
        src_client_id: String,
        dst_client_id: String,
        src_packet_seqs: Vec<u64>,
        dst_packet_seqs: Vec<u64>,
    ) -> anyhow::Result<Vec<u8>> {
        match self {
            Self::Mock(tb) => {
                tb.relay_events(
                    solana_src_events,
                    target_events,
                    src_client_id,
                    dst_client_id,
                    src_packet_seqs,
                    dst_packet_seqs,
                )
                .await
            }
            Self::Attested(tb) => {
                let src_events = solana_src_events
                    .into_iter()
                    .map(EurekaEventWithHeight::from)
                    .collect();
                tb.relay_events(RelayEventsParams {
                    src_events,
                    target_events,
                    timeout_relay_height,
                    src_client_id,
                    dst_client_id,
                    src_packet_seqs,
                    dst_packet_seqs,
                })
                .await
            }
        }
    }

    async fn create_client(&self, parameters: &HashMap<String, String>) -> anyhow::Result<Vec<u8>> {
        match self {
            Self::Mock(tb) => tb.create_client(parameters).await,
            Self::Attested(tb) => tb.create_client(parameters),
        }
    }

    async fn update_client(&self, dst_client_id: String) -> anyhow::Result<Vec<u8>> {
        match self {
            Self::Mock(tb) => tb.update_client(dst_client_id).await,
            Self::Attested(tb) => tb.update_client(&dst_client_id).await,
        }
    }

    const fn is_attested(&self) -> bool {
        matches!(self, Self::Attested(_))
    }
}
