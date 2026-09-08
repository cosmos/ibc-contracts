// SPDX-License-Identifier: Apache-2.0

//! This is a one-sided proof API module from Ethereum to a Cosmos SDK chain.

#![deny(
    clippy::nursery,
    clippy::pedantic,
    warnings,
    missing_docs,
    unused_crate_dependencies
)]

use tendermint as _;

pub mod tx_builder;

use std::collections::HashMap;

use alloy::{
    primitives::{Address, TxHash},
    providers::{Provider, RootProvider},
};
use ibc_eureka_utils::rpc::TendermintRpcExt;
use proof_api_lib::{
    aggregator::{Aggregator, Config as AggregatorConfig},
    listener::{cosmos_sdk, eth_eureka, ChainListenerService},
    service_utils::{parse_cosmos_tx_hashes, parse_eth_tx_hashes, to_tonic_status},
    tx_builder::TxBuilderService,
    utils::RelayEventsParams,
};
use tendermint_rpc::HttpClient;
use tonic::{Request, Response};

use proof_api_core::{
    api::{self, proof_api_service_server::ProofApiService},
    modules::ProofApiModule,
};

/// The `EthToCosmosProofApiModule` struct defines the Ethereum to Cosmos proof API module.
#[derive(Clone, Copy, Debug)]
pub struct EthToCosmosProofApiModule;

/// The `EthereumToCosmosProofApiModuleService` defines the proof API service from Ethereum to Cosmos.
struct EthToCosmosProofApiModuleService {
    /// The chain listener for `EthEureka`.
    pub eth_listener: eth_eureka::ChainListener<RootProvider>,
    /// The chain listener for Cosmos SDK.
    pub tm_listener: cosmos_sdk::ChainListener,
    /// The transaction builder for Ethereum to Cosmos.
    pub tx_builder: EthToCosmosTxBuilder,
}

enum EthToCosmosTxBuilder {
    Real(tx_builder::TxBuilder<RootProvider>),
    Mock(tx_builder::MockTxBuilder<RootProvider>),
    Attested(tx_builder::AttestedTxBuilder),
}

/// The configuration for the Ethereum to Cosmos proof API module.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct EthToCosmosConfig {
    /// The ICS26 address.
    pub ics26_address: Address,
    /// The tendermint RPC URL.
    pub tm_rpc_url: String,
    /// The EVM RPC URL.
    pub eth_rpc_url: String,
    /// The Ethereum Beacon API URL (required for Real mode).
    #[serde(default)]
    pub eth_beacon_api_url: String,
    /// The address of the submitter.
    /// Required since cosmos messages require a signer address.
    pub signer_address: String,
    /// Transaction builder mode.
    pub mode: TxBuilderMode,
}

/// Transaction builder mode configuration.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TxBuilderMode {
    /// Real mode using Ethereum beacon chain proofs.
    Real,
    /// Mock mode for testing without real proofs.
    Mock,
    /// Attested mode using aggregator attestations.
    Attested(AggregatorConfig),
}

impl EthToCosmosProofApiModuleService {
    async fn new(config: EthToCosmosConfig) -> anyhow::Result<Self> {
        let provider = RootProvider::builder()
            .connect(&config.eth_rpc_url)
            .await
            .map_err(|e| anyhow::anyhow!("failed to create provider: {e}"))?;
        let eth_listener = eth_eureka::ChainListener::new(config.ics26_address, provider.clone());

        let tm_client = HttpClient::from_rpc_url(&config.tm_rpc_url);
        let tm_listener = cosmos_sdk::ChainListener::new(tm_client.clone());

        let tx_builder = match config.mode {
            TxBuilderMode::Mock => EthToCosmosTxBuilder::Mock(tx_builder::MockTxBuilder::new(
                config.ics26_address,
                provider,
                config.signer_address,
            )),
            TxBuilderMode::Real => EthToCosmosTxBuilder::Real(tx_builder::TxBuilder::new(
                config.ics26_address,
                provider,
                config.eth_beacon_api_url,
                tm_client,
                config.signer_address,
            )),
            TxBuilderMode::Attested(aggregator_config) => {
                let aggregator = Aggregator::from_config(aggregator_config)
                    .await
                    .map_err(|e| anyhow::anyhow!("failed to create aggregator: {e}"))?;
                EthToCosmosTxBuilder::Attested(tx_builder::AttestedTxBuilder::new(
                    aggregator,
                    config.ics26_address,
                    config.signer_address,
                ))
            }
        };

        Ok(Self {
            eth_listener,
            tm_listener,
            tx_builder,
        })
    }
}

#[tonic::async_trait]
impl ProofApiService for EthToCosmosProofApiModuleService {
    #[tracing::instrument(skip_all, err(Debug))]
    async fn info(
        &self,
        _request: Request<api::InfoRequest>,
    ) -> Result<Response<api::InfoResponse>, tonic::Status> {
        tracing::info!("Handling info request for Eth to Cosmos...");
        Ok(Response::new(api::InfoResponse {
            target_chain: Some(api::Chain {
                chain_id: self.tm_listener.chain_id().await.map_err(to_tonic_status)?,
                ibc_version: "2".to_string(),
                ibc_contract: String::new(),
            }),
            source_chain: Some(api::Chain {
                chain_id: self
                    .eth_listener
                    .chain_id()
                    .await
                    .map_err(to_tonic_status)?,
                ibc_version: "2".to_string(),
                ibc_contract: self.tx_builder.ics26_router_address().to_string(),
            }),
            metadata: HashMap::default(),
        }))
    }

    #[tracing::instrument(skip_all, err(Debug))]
    async fn relay_by_tx(
        &self,
        request: Request<api::RelayByTxRequest>,
    ) -> Result<Response<api::RelayByTxResponse>, tonic::Status> {
        tracing::info!("Handling relay by tx request for Eth to Cosmos...");

        let inner_req = request.into_inner();
        tracing::info!("Got {} source tx IDs", inner_req.source_tx_ids.len());
        tracing::info!("Got {} timeout tx IDs", inner_req.timeout_tx_ids.len());
        let eth_tx_hashes = parse_eth_tx_hashes(inner_req.source_tx_ids)?;
        let eth_txs = eth_tx_hashes.into_iter().map(TxHash::from).collect();

        let timeout_txs = parse_cosmos_tx_hashes(inner_req.timeout_tx_ids)?;

        let eth_events = self
            .eth_listener
            .fetch_tx_events(eth_txs)
            .await
            .map_err(|e| tonic::Status::from_error(e.into()))?;

        tracing::debug!(eth_events = ?eth_events, "Fetched EVM events.");
        tracing::info!("Fetched {} eureka events from EVM.", eth_events.len());

        let timeout_events = self
            .tm_listener
            .fetch_tx_events(timeout_txs)
            .await
            .map_err(|e| tonic::Status::from_error(e.into()))?;

        tracing::debug!(timeout_events = ?timeout_events, "Fetched timeout events from Cosmos.");
        tracing::info!(
            "Fetched {} timeout eureka events from CosmosSDK.",
            timeout_events.len()
        );

        // For timeouts in attested mode, get the current height from the source chain (Eth)
        // where non-membership is proven
        let timeout_relay_height = if self.tx_builder.is_attested() && !timeout_events.is_empty() {
            Some(
                self.eth_listener
                    .get_block_number()
                    .await
                    .map_err(|e| tonic::Status::from_error(e.into()))?,
            )
        } else {
            None
        };

        let tx = self
            .tx_builder
            .relay_events(RelayEventsParams {
                src_events: eth_events,
                target_events: timeout_events,
                timeout_relay_height,
                src_client_id: inner_req.src_client_id,
                dst_client_id: inner_req.dst_client_id,
                src_packet_seqs: inner_req.src_packet_sequences,
                dst_packet_seqs: inner_req.dst_packet_sequences,
            })
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
        tracing::info!("Handling create client request for Eth to Cosmos...");

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
        tracing::info!("Handling update client request for Eth to Cosmos...");

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
impl ProofApiModule for EthToCosmosProofApiModule {
    fn name(&self) -> &'static str {
        "eth_to_cosmos"
    }

    #[tracing::instrument(skip_all, err(Debug))]
    async fn create_service(
        &self,
        config: serde_json::Value,
    ) -> anyhow::Result<Box<dyn ProofApiService>> {
        let config = serde_json::from_value::<EthToCosmosConfig>(config)
            .map_err(|e| anyhow::anyhow!("failed to parse config: {e}"))?;

        tracing::info!("Starting Ethereum to Cosmos proof API server.");
        Ok(Box::new(
            EthToCosmosProofApiModuleService::new(config).await?,
        ))
    }
}

impl EthToCosmosTxBuilder {
    async fn relay_events(&self, params: RelayEventsParams) -> anyhow::Result<Vec<u8>> {
        match self {
            Self::Real(tb) => {
                tb.relay_events(
                    params.src_events,
                    params.target_events,
                    params.src_client_id,
                    params.dst_client_id,
                    params.src_packet_seqs,
                    params.dst_packet_seqs,
                )
                .await
            }
            Self::Mock(tb) => {
                tb.relay_events(
                    params.src_events,
                    params.target_events,
                    params.src_client_id,
                    params.dst_client_id,
                    params.src_packet_seqs,
                    params.dst_packet_seqs,
                )
                .await
            }
            Self::Attested(tb) => tb.relay_events(params).await,
        }
    }

    async fn create_client(&self, parameters: &HashMap<String, String>) -> anyhow::Result<Vec<u8>> {
        match self {
            Self::Real(tb) => tb.create_client(parameters).await,
            Self::Mock(tb) => tb.create_client(parameters).await,
            Self::Attested(tb) => tb.create_client(parameters),
        }
    }

    async fn update_client(&self, dst_client_id: String) -> anyhow::Result<Vec<u8>> {
        match self {
            Self::Real(tb) => tb.update_client(dst_client_id).await,
            Self::Mock(tb) => tb.update_client(dst_client_id).await,
            Self::Attested(tb) => tb.update_client(&dst_client_id).await,
        }
    }

    const fn ics26_router_address(&self) -> &Address {
        match self {
            Self::Real(tb) => tb.ics26_router.address(),
            Self::Mock(tb) => tb.ics26_router.address(),
            Self::Attested(tb) => tb.ics26_router(),
        }
    }

    const fn is_attested(&self) -> bool {
        matches!(self, Self::Attested(_))
    }
}

#[cfg(test)]
mod tests {
    use alloy::providers::{Provider, RootProvider};

    #[tokio::test]
    async fn https_provider_can_fetch_chain_id() {
        let provider: RootProvider = RootProvider::builder()
            .connect("https://ethereum-rpc.publicnode.com")
            .await
            .expect("connect to HTTPS Ethereum RPC endpoint");

        assert_eq!(provider.get_chain_id().await.unwrap(), 1);
    }
}
