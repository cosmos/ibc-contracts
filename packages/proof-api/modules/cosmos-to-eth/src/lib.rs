// SPDX-License-Identifier: Apache-2.0

//! This is a one-sided proof API module from a Cosmos SDK chain to Ethereum.

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
    service_utils::{parse_cosmos_tx_hashes, to_tonic_status},
    tx_builder::TxBuilderService,
    utils::RelayEventsParams,
};
use sp1_ics07_tendermint_prover::programs::{
    MembershipProgram, MisbehaviourProgram, SP1ICS07TendermintPrograms,
    UpdateClientAndMembershipProgram, UpdateClientProgram,
};
use sp1_ics07_tendermint_prover::prover::Sp1Prover;
use sp1_sdk::{
    network::{FulfillmentStrategy, NetworkMode},
    ProverClient,
};
use tendermint_rpc::HttpClient;
use tonic::{Request, Response};
use tx_builder::TxBuilder;

use proof_api_core::{
    api::{self, proof_api_service_server::ProofApiService},
    modules::ProofApiModule,
};

/// The `CosmosToEthProofApiModule` struct defines the Cosmos to Ethereum proof API module.
#[derive(Clone, Copy, Debug)]
pub struct CosmosToEthProofApiModule;

/// The `CosmosToEthProofApiModuleService` defines the proof API service from Cosmos to Ethereum.
struct CosmosToEthProofApiModuleService {
    /// The chain listener for Cosmos SDK.
    tm_listener: cosmos_sdk::ChainListener,
    /// The chain listener for `EthEureka`.
    eth_listener: eth_eureka::ChainListener<RootProvider>,
    /// The transaction builder for `EthEureka`.
    tx_builder: CosmosToEthTxBuilder,
}

enum CosmosToEthTxBuilder {
    SP1(Box<TxBuilder<RootProvider>>),
    Attested(Box<tx_builder::AttestedTxBuilder<RootProvider>>),
}

/// The configuration for the Cosmos to Ethereum proof API module.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct CosmosToEthConfig {
    /// The tendermint RPC URL.
    pub tm_rpc_url: String,
    /// The ICS26 address.
    pub ics26_address: Address,
    /// The EVM RPC URL.
    pub eth_rpc_url: String,
    /// Transaction builder mode.
    pub mode: TxBuilderMode,
}

/// Transaction builder mode configuration.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TxBuilderMode {
    /// SP1 prover mode using zero-knowledge proofs.
    Sp1(Sp1ModeConfig),
    /// Attested mode using aggregator attestations.
    Attested(AggregatorConfig),
}

/// Configuration for SP1 prover mode.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct Sp1ModeConfig {
    /// The SP1 prover configuration.
    pub sp1_prover: SP1Config,
    /// The SP1 program paths.
    pub sp1_programs: SP1ProgramPaths,
}

/// The paths to the SP1 programs.
/// This is relative to the current working directory.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct SP1ProgramPaths {
    /// The path to the update client program.
    pub update_client: String,
    /// The path to the membership program.
    pub membership: String,
    /// The path to the update client and membership program.
    pub update_client_and_membership: String,
    /// The path to the misbehaviour program.
    pub misbehaviour: String,
}

impl SP1ProgramPaths {
    /// Get the ELF bytes for the programs.
    ///
    /// # Errors
    /// Returns an error if the programs cannot be read.
    pub fn read_programs(&self) -> anyhow::Result<SP1ICS07TendermintPrograms> {
        let update_client = std::fs::read(&self.update_client)?;
        let membership = std::fs::read(&self.membership)?;
        let uc_and_membership = std::fs::read(&self.update_client_and_membership)?;
        let misbehaviour = std::fs::read(&self.misbehaviour)?;

        Ok(SP1ICS07TendermintPrograms {
            update_client: UpdateClientProgram::new(update_client),
            membership: MembershipProgram::new(membership),
            update_client_and_membership: UpdateClientAndMembershipProgram::new(uc_and_membership),
            misbehaviour: MisbehaviourProgram::new(misbehaviour),
        })
    }
}

/// The configuration for the SP1 prover.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum SP1Config {
    /// Mock prover.
    Mock,
    /// Prover from environment variables. (Usually a network prover)
    ///
    /// `sp1-sdk` expects the following environment variables:
    /// - `SP1_PROVER`: The prover type. (mock, network, cpu, cuda)
    /// - `NETWORK_PRIVATE_KEY`: The private key for the network prover. (only network)
    /// - `NETWORK_RPC_URL`: The RPC URL for prover network. (only network, default exists if empty)
    Env,
    /// The network prover.
    Network {
        /// The optional private key for the network prover.
        /// `NETWORK_PRIVATE_KEY` environment variable is used if not provided.
        #[serde(default)]
        network_private_key: Option<String>,
        /// The optional RPC URL for the network prover.
        /// `NETWORK_RPC_URL` environment variable is used if not provided.
        #[serde(default)]
        network_rpc_url: Option<String>,
        /// Whether to use a private cluster.
        #[serde(default)]
        private_cluster: bool,
    },
    /// The local CPU prover.
    Cpu,
    /// The local CUDA prover.
    Cuda,
}

impl CosmosToEthProofApiModuleService {
    async fn new(config: CosmosToEthConfig) -> anyhow::Result<Self> {
        let tm_client = HttpClient::from_rpc_url(&config.tm_rpc_url);
        let tm_listener = cosmos_sdk::ChainListener::new(tm_client.clone());

        let provider = RootProvider::builder()
            .connect(&config.eth_rpc_url)
            .await
            .map_err(|e| anyhow::anyhow!("failed to create provider: {e}"))?;

        let eth_listener = eth_eureka::ChainListener::new(config.ics26_address, provider.clone());

        let tx_builder = match config.mode {
            TxBuilderMode::Sp1(sp1_config) => {
                let programs = sp1_config
                    .sp1_programs
                    .read_programs()
                    .map_err(|e| anyhow::anyhow!("failed to read SP1 programs: {e}"))?;

                let sp1_prover = match sp1_config.sp1_prover {
                    SP1Config::Mock => {
                        Sp1Prover::Mock(ProverClient::builder().mock().build().await)
                    }
                    SP1Config::Env => Sp1Prover::Env(ProverClient::from_env().await),
                    SP1Config::Cpu => Sp1Prover::Cpu(ProverClient::builder().cpu().build().await),
                    SP1Config::Cuda => {
                        Sp1Prover::Cuda(ProverClient::builder().cuda().build().await)
                    }
                    SP1Config::Network {
                        network_private_key,
                        network_rpc_url,
                        private_cluster,
                    } => {
                        let mut builder = if private_cluster {
                            ProverClient::builder().network_for(NetworkMode::Reserved)
                        } else {
                            ProverClient::builder().network()
                        };
                        let strategy = if private_cluster {
                            FulfillmentStrategy::Reserved
                        } else {
                            FulfillmentStrategy::Hosted
                        };
                        if let Some(private_key) = network_private_key {
                            builder = builder.private_key(&private_key);
                        }
                        if let Some(rpc_url) = network_rpc_url {
                            builder = builder.rpc_url(&rpc_url);
                        }
                        Sp1Prover::Network(builder.build().await, strategy)
                    }
                };
                let sp1_tx_builder = TxBuilder::new(
                    config.ics26_address,
                    provider,
                    tm_client,
                    sp1_prover,
                    programs,
                );
                CosmosToEthTxBuilder::SP1(Box::new(sp1_tx_builder))
            }
            TxBuilderMode::Attested(aggregator_config) => {
                let aggregator = Aggregator::from_config(aggregator_config)
                    .await
                    .map_err(|e| anyhow::anyhow!("failed to create aggregator: {e}"))?;
                CosmosToEthTxBuilder::Attested(Box::new(tx_builder::AttestedTxBuilder::new(
                    aggregator,
                    config.ics26_address,
                    provider,
                )))
            }
        };

        Ok(Self {
            tm_listener,
            eth_listener,
            tx_builder,
        })
    }
}

#[tonic::async_trait]
impl ProofApiService for CosmosToEthProofApiModuleService {
    #[tracing::instrument(skip_all, err(Debug))]
    async fn info(
        &self,
        _request: Request<api::InfoRequest>,
    ) -> Result<Response<api::InfoResponse>, tonic::Status> {
        tracing::info!("Handling info request for Cosmos to Eth...");
        Ok(Response::new(api::InfoResponse {
            target_chain: Some(api::Chain {
                chain_id: self
                    .eth_listener
                    .chain_id()
                    .await
                    .map_err(to_tonic_status)?,
                ibc_version: "2".to_string(),
                ibc_contract: self.tx_builder.ics26_router_address().to_string(),
            }),
            source_chain: Some(api::Chain {
                chain_id: self.tm_listener.chain_id().await.map_err(to_tonic_status)?,
                ibc_version: "2".to_string(),
                ibc_contract: String::new(),
            }),
            metadata: self.tx_builder.metadata(),
        }))
    }

    #[tracing::instrument(skip_all, err(Debug))]
    async fn relay_by_tx(
        &self,
        request: Request<api::RelayByTxRequest>,
    ) -> Result<Response<api::RelayByTxResponse>, tonic::Status> {
        tracing::info!("Handling relay by tx request for Cosmos to Eth...");

        let inner_req = request.into_inner();
        tracing::info!("Got {} source tx IDs", inner_req.source_tx_ids.len());
        tracing::info!("Got {} timeout tx IDs", inner_req.timeout_tx_ids.len());
        let cosmos_txs = parse_cosmos_tx_hashes(inner_req.source_tx_ids)?;

        let timeout_txs = inner_req
            .timeout_tx_ids
            .into_iter()
            .map(TryInto::<[u8; 32]>::try_into)
            .map(|tx_hash| tx_hash.map(TxHash::from))
            .collect::<Result<Vec<_>, _>>()
            .map_err(|tx| tonic::Status::from_error(format!("invalid tx hash: {tx:?}").into()))?;

        let cosmos_events = self
            .tm_listener
            .fetch_tx_events(cosmos_txs)
            .await
            .map_err(|e| tonic::Status::from_error(e.into()))?;

        tracing::debug!(cosmos_events = ?cosmos_events, "Fetched Cosmos events.");
        tracing::info!(
            "Fetched {} eureka events from CosmosSDK.",
            cosmos_events.len()
        );

        let timeout_events = self
            .eth_listener
            .fetch_tx_events(timeout_txs)
            .await
            .map_err(|e| tonic::Status::from_error(e.into()))?;

        tracing::debug!(timeout_events = ?timeout_events, "Fetched timeout events from EVM.");
        tracing::info!("Fetched {} timeout events from EVM.", timeout_events.len());

        // For timeouts in attested mode, get the current height from the source chain (Cosmos)
        // where non-membership is proven
        let timeout_relay_height = if self.tx_builder.is_attested() && !timeout_events.is_empty() {
            Some(
                self.tm_listener
                    .get_block_height()
                    .await
                    .map_err(|e| tonic::Status::from_error(e.into()))?,
            )
        } else {
            None
        };

        let multicall_tx = self
            .tx_builder
            .relay_events(RelayEventsParams {
                src_events: cosmos_events,
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
            tx: multicall_tx,
            address: self.tx_builder.ics26_router_address().to_string(),
        }))
    }

    #[tracing::instrument(skip_all, err(Debug))]
    async fn create_client(
        &self,
        request: Request<api::CreateClientRequest>,
    ) -> Result<Response<api::CreateClientResponse>, tonic::Status> {
        tracing::info!("Handling create client request for Cosmos to Eth...");

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
        tracing::info!("Handling update client request for Cosmos to Eth...");

        let inner_req = request.into_inner();
        let tx = self
            .tx_builder
            .update_client(inner_req.dst_client_id)
            .await
            .map_err(|e| tonic::Status::from_error(e.into()))?;

        tracing::info!("Update client request completed.");

        Ok(Response::new(api::UpdateClientResponse {
            tx,
            address: self.tx_builder.ics26_router_address().to_string(),
        }))
    }
}

#[tonic::async_trait]
impl ProofApiModule for CosmosToEthProofApiModule {
    fn name(&self) -> &'static str {
        "cosmos_to_eth"
    }

    #[tracing::instrument(skip_all, err(Debug))]
    async fn create_service(
        &self,
        config: serde_json::Value,
    ) -> anyhow::Result<Box<dyn ProofApiService>> {
        let config = serde_json::from_value::<CosmosToEthConfig>(config)
            .map_err(|e| anyhow::anyhow!("failed to parse config: {e}"))?;

        tracing::info!("Starting Cosmos to Ethereum proof API server.");
        Ok(Box::new(
            CosmosToEthProofApiModuleService::new(config).await?,
        ))
    }
}

impl CosmosToEthTxBuilder {
    async fn relay_events(&self, params: RelayEventsParams) -> anyhow::Result<Vec<u8>> {
        match self {
            Self::SP1(tb) => {
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
            Self::SP1(tb) => tb.create_client(parameters).await,
            Self::Attested(tb) => tb.create_client(parameters),
        }
    }

    async fn update_client(&self, dst_client_id: String) -> anyhow::Result<Vec<u8>> {
        match self {
            Self::SP1(tb) => tb.update_client(dst_client_id.clone()).await,
            Self::Attested(tb) => tb.update_client(dst_client_id).await,
        }
    }

    const fn ics26_router_address(&self) -> &Address {
        match self {
            Self::SP1(tb) => tb.ics26_router.address(),
            Self::Attested(tb) => tb.ics26_address(),
        }
    }

    fn metadata(&self) -> HashMap<String, String> {
        match self {
            Self::SP1(tb) => tb.metadata(),
            Self::Attested(_) => HashMap::default(),
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
