// SPDX-License-Identifier: Apache-2.0

//! This is a one-sided proof API module from a Cosmos SDK chain to a Cosmos SDK chain.

#![deny(
    clippy::nursery,
    clippy::pedantic,
    warnings,
    missing_docs,
    unused_crate_dependencies
)]

use ics23 as _;
use tendermint as _;

pub mod tx_builder;

use std::collections::HashMap;

use ibc_eureka_utils::rpc::TendermintRpcExt;
use proof_api_lib::{
    listener::{cosmos_sdk, ChainListenerService},
    service_utils::{parse_cosmos_tx_hashes, to_tonic_status},
    tx_builder::TxBuilderService,
};
use tendermint_rpc::HttpClient;
use tonic::{Request, Response};

use proof_api_core::{
    api::{self, proof_api_service_server::ProofApiService},
    modules::ProofApiModule,
};

/// The `CosmosToCosmosProofApiModule` struct defines the Cosmos to Cosmos proof API module.
#[derive(Clone, Copy, Debug)]
pub struct CosmosToCosmosProofApiModule;

/// The `CosmosToCosmosProofApiModuleService` defines the proof API service from Cosmos to Cosmos.
#[allow(dead_code)]
struct CosmosToCosmosProofApiModuleService {
    /// The souce chain listener for Cosmos SDK.
    pub src_listener: cosmos_sdk::ChainListener,
    /// The target chain listener for Cosmos SDK.
    pub target_listener: cosmos_sdk::ChainListener,
    /// The transaction builder from Cosmos to Cosmos.
    pub tx_builder: tx_builder::TxBuilder,
}

/// The configuration for the Cosmos to Cosmos proof API module.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct CosmosToCosmosConfig {
    /// The source tendermint RPC URL.
    pub src_rpc_url: String,
    /// The target tendermint RPC URL.
    pub target_rpc_url: String,
    /// The address of the submitter.
    /// Required since cosmos messages require a signer address.
    pub signer_address: String,
}

impl CosmosToCosmosProofApiModuleService {
    fn new(config: CosmosToCosmosConfig) -> Self {
        let src_client = HttpClient::from_rpc_url(&config.src_rpc_url);
        let src_listener = cosmos_sdk::ChainListener::new(src_client.clone());
        let target_client = HttpClient::from_rpc_url(&config.target_rpc_url);
        let target_listener = cosmos_sdk::ChainListener::new(target_client.clone());

        let tx_builder =
            tx_builder::TxBuilder::new(src_client, target_client, config.signer_address);

        Self {
            src_listener,
            target_listener,
            tx_builder,
        }
    }
}

#[tonic::async_trait]
impl ProofApiService for CosmosToCosmosProofApiModuleService {
    #[tracing::instrument(skip_all, err(Debug))]
    async fn info(
        &self,
        _request: Request<api::InfoRequest>,
    ) -> Result<Response<api::InfoResponse>, tonic::Status> {
        tracing::info!("Handling info request for Cosmos to Cosmos...");
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
                chain_id: self
                    .src_listener
                    .chain_id()
                    .await
                    .map_err(to_tonic_status)?,
                ibc_version: "2".to_string(),
                ibc_contract: String::new(),
            }),
            metadata: HashMap::default(),
        }))
    }

    #[tracing::instrument(skip_all, err(Debug))]
    async fn relay_by_tx(
        &self,
        request: Request<api::RelayByTxRequest>,
    ) -> Result<Response<api::RelayByTxResponse>, tonic::Status> {
        tracing::info!("Handling relay by tx request for Cosmos to Cosmos...");

        let inner_req = request.into_inner();
        tracing::info!("Got {} source tx IDs", inner_req.source_tx_ids.len());
        tracing::info!("Got {} timeout tx IDs", inner_req.timeout_tx_ids.len());
        let src_txs = parse_cosmos_tx_hashes(inner_req.source_tx_ids)?;

        let target_txs = parse_cosmos_tx_hashes(inner_req.timeout_tx_ids)?;

        let src_events = self
            .src_listener
            .fetch_tx_events(src_txs)
            .await
            .map_err(|e| tonic::Status::from_error(e.into()))?;

        tracing::debug!(cosmos_src_events = ?src_events, "Fetched source cosmos events.");
        tracing::info!(
            "Fetched {} source eureka events from CosmosSDK.",
            src_events.len()
        );

        let target_events = self
            .target_listener
            .fetch_tx_events(target_txs)
            .await
            .map_err(|e| tonic::Status::from_error(e.into()))?;

        tracing::debug!(cosmos_target_events = ?target_events, "Fetched target cosmos events.");
        tracing::info!(
            "Fetched {} target eureka events from CosmosSDK.",
            target_events.len()
        );

        let tx = self
            .tx_builder
            .relay_events(
                src_events,
                target_events,
                inner_req.src_client_id,
                inner_req.dst_client_id,
                inner_req.src_packet_sequences,
                inner_req.dst_packet_sequences,
            )
            .await
            .map_err(|e| tonic::Status::from_error(e.into()))?;

        tracing::info!("Relay by tx request completed.");

        Ok(Response::new(api::RelayByTxResponse {
            tx,
            address: String::new(),
        }))
    }

    #[tracing::instrument(skip_all, err(Debug))]
    async fn create_client(
        &self,
        request: Request<api::CreateClientRequest>,
    ) -> Result<Response<api::CreateClientResponse>, tonic::Status> {
        tracing::info!("Handling create client request for Cosmos to Cosmos...");

        let inner_req = request.into_inner();
        let tx = self
            .tx_builder
            .create_client(&inner_req.parameters)
            .await
            .map_err(|e| tonic::Status::from_error(e.into()))?;

        tracing::info!("Create client request completed.");

        Ok(Response::new(api::CreateClientResponse {
            tx,
            address: String::new(),
        }))
    }

    #[tracing::instrument(skip_all, err(Debug))]
    async fn update_client(
        &self,
        request: Request<api::UpdateClientRequest>,
    ) -> Result<Response<api::UpdateClientResponse>, tonic::Status> {
        tracing::info!("Handling update client request for Cosmos to Cosmos...");

        let inner_req = request.into_inner();
        let tx = self
            .tx_builder
            .update_client(inner_req.dst_client_id)
            .await
            .map_err(|e| tonic::Status::from_error(e.into()))?;

        tracing::info!("Update client request completed.");

        Ok(Response::new(api::UpdateClientResponse {
            tx,
            address: String::new(),
        }))
    }
}

#[tonic::async_trait]
impl ProofApiModule for CosmosToCosmosProofApiModule {
    fn name(&self) -> &'static str {
        "cosmos_to_cosmos"
    }

    #[tracing::instrument(skip_all, err(Debug))]
    async fn create_service(
        &self,
        config: serde_json::Value,
    ) -> anyhow::Result<Box<dyn ProofApiService>> {
        let config = serde_json::from_value::<CosmosToCosmosConfig>(config)
            .map_err(|e| anyhow::anyhow!("failed to parse config: {e}"))?;

        tracing::info!("Starting Cosmos to Cosmos proof API server.");
        Ok(Box::new(CosmosToCosmosProofApiModuleService::new(config)))
    }
}
