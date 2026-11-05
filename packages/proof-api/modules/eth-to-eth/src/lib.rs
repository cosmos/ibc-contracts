// SPDX-License-Identifier: Apache-2.0

//! One-sided proof API modules from an EVM chain to Ethereum: `eth_to_eth` relays with
//! attestations and `besu_to_eth` with Besu light client proofs.

#![deny(clippy::nursery, clippy::pedantic, warnings, unused_crate_dependencies)]
#![allow(unused_crate_dependencies, missing_docs)]

mod besu;

use std::collections::HashMap;

use alloy::{
    primitives::{Address, TxHash},
    providers::{Provider, RootProvider},
};
use proof_api_lib::{
    aggregator::{Aggregator, Config as AggregatorConfig},
    listener::{eth_eureka, ChainListenerService},
    service_utils::{parse_eth_tx_hashes, to_tonic_status},
    utils::{eth_attested::EthAttestedTxBuilder, RelayEventsParams},
};
use tonic::{Request, Response};

use proof_api_core::{
    api::{self, proof_api_service_server::ProofApiService},
    modules::ProofApiModule,
};

pub use besu::{BesuConsensusType, BesuToEthConfig, BesuToEthProofApiModule};

/// The `EthToEthProofApiModule` struct defines the Ethereum to Ethereum proof API module.
#[derive(Clone, Copy, Debug)]
pub struct EthToEthProofApiModule;

/// The `EthToEthProofApiModuleService` defines the proof API service from Ethereum to Ethereum.
struct EthToEthProofApiModuleService {
    /// The source ICS26 address.
    src_ics26_address: Address,
    /// The chain listener for source Ethereum.
    src_listener: eth_eureka::ChainListener<RootProvider>,
    /// The chain listener for destination Ethereum.
    dst_listener: eth_eureka::ChainListener<RootProvider>,
    /// The destination ICS26 address.
    dst_ics26_address: Address,
    /// The transaction builder.
    tx_builder: EthToEthTxBuilder,
    /// The destination `IBCMulticall` address that relay and update client transactions are
    /// submitted to.
    dst_ibc_multicall_address: Address,
}

enum EthToEthTxBuilder {
    Attested(EthAttestedTxBuilder<RootProvider>),
    Besu(besu::TxBuilder),
}

/// The configuration for the Ethereum to Ethereum proof API module.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct EthToEthConfig {
    /// The source Ethereum RPC URL.
    pub src_rpc_url: String,
    /// The source ICS26 address.
    pub src_ics26_address: Address,
    /// The destination Ethereum RPC URL.
    pub dst_rpc_url: String,
    /// The destination ICS26 address.
    pub dst_ics26_address: Address,
    /// The destination `IBCMulticall` address that relay and update client transactions are
    /// submitted to.
    pub dst_ibc_multicall_address: Address,
    /// Transaction builder mode.
    pub mode: TxBuilderMode,
}

/// Transaction builder mode configuration.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TxBuilderMode {
    /// Attested mode using aggregator attestations.
    Attested(AggregatorConfig),
    /// Besu light client mode, configured through the `besu_to_eth` module.
    #[serde(skip)]
    Besu(BesuConsensusType),
}

impl EthToEthProofApiModuleService {
    async fn new(config: EthToEthConfig) -> anyhow::Result<Self> {
        let src_provider = RootProvider::builder()
            .connect(&config.src_rpc_url)
            .await
            .map_err(|e| anyhow::anyhow!("failed to create source provider: {e}"))?;

        let dst_provider = RootProvider::builder()
            .connect(&config.dst_rpc_url)
            .await
            .map_err(|e| anyhow::anyhow!("failed to create destination provider: {e}"))?;

        let src_listener =
            eth_eureka::ChainListener::new(config.src_ics26_address, src_provider.clone());
        let dst_listener =
            eth_eureka::ChainListener::new(config.dst_ics26_address, dst_provider.clone());

        let tx_builder = match config.mode {
            TxBuilderMode::Attested(aggregator_config) => {
                let aggregator = Aggregator::from_config(aggregator_config)
                    .await
                    .map_err(|e| anyhow::anyhow!("failed to create aggregator: {e}"))?;
                EthToEthTxBuilder::Attested(EthAttestedTxBuilder::new(
                    aggregator,
                    config.dst_ics26_address,
                    dst_provider,
                ))
            }
            TxBuilderMode::Besu(consensus_type) => EthToEthTxBuilder::Besu(besu::TxBuilder::new(
                src_provider,
                dst_provider,
                config.src_ics26_address,
                config.dst_ics26_address,
                consensus_type,
            )),
        };

        Ok(Self {
            src_ics26_address: config.src_ics26_address,
            src_listener,
            dst_listener,
            dst_ics26_address: config.dst_ics26_address,
            tx_builder,
            dst_ibc_multicall_address: config.dst_ibc_multicall_address,
        })
    }
}

#[tonic::async_trait]
impl ProofApiService for EthToEthProofApiModuleService {
    async fn info(
        &self,
        request: Request<api::InfoRequest>,
    ) -> Result<Response<api::InfoResponse>, tonic::Status> {
        tracing::info!("Handling info request for Eth to Eth...");
        let src_chain = request.into_inner().src_chain;

        Ok(Response::new(api::InfoResponse {
            target_chain: Some(api::Chain {
                chain_id: self
                    .dst_listener
                    .chain_id()
                    .await
                    .map_err(to_tonic_status)?,
                ibc_version: "2".to_string(),
                ibc_contract: self.dst_ics26_address.to_string(),
            }),
            source_chain: Some(api::Chain {
                chain_id: src_chain,
                ibc_version: "2".to_string(),
                ibc_contract: self.src_ics26_address.to_string(),
            }),
            metadata: HashMap::default(),
        }))
    }

    async fn relay_by_tx(
        &self,
        request: Request<api::RelayByTxRequest>,
    ) -> Result<Response<api::RelayByTxResponse>, tonic::Status> {
        tracing::info!("Handling relay by tx request for Eth to Eth...");

        let inner_req = request.into_inner();
        tracing::info!("Got {} source tx IDs", inner_req.source_tx_ids.len());
        tracing::info!("Got {} timeout tx IDs", inner_req.timeout_tx_ids.len());

        let src_txs = parse_eth_tx_hashes(inner_req.source_tx_ids)?
            .into_iter()
            .map(TxHash::from)
            .collect();
        let timeout_txs = parse_eth_tx_hashes(inner_req.timeout_tx_ids)?
            .into_iter()
            .map(TxHash::from)
            .collect();

        let src_events = self
            .src_listener
            .fetch_tx_events(src_txs)
            .await
            .map_err(to_tonic_status)?;

        tracing::debug!(?src_events, "Fetched source Eth events.");
        tracing::info!(
            "Fetched {} eureka events from source Eth.",
            src_events.len()
        );

        let timeout_txs = self
            .dst_listener
            .fetch_tx_events(timeout_txs)
            .await
            .map_err(to_tonic_status)?;

        tracing::debug!(?timeout_txs, "Fetched destination Eth events.");
        tracing::info!(
            "Fetched {} eureka events from destination Eth.",
            timeout_txs.len()
        );

        // For timeouts, get the current height from the source chain (where non-membership is
        // proven).
        let timeout_relay_height = if timeout_txs.is_empty() {
            None
        } else {
            Some(
                self.src_listener
                    .get_block_number()
                    .await
                    .map_err(to_tonic_status)?,
            )
        };

        let tx = self
            .tx_builder
            .relay_events(RelayEventsParams {
                src_events,
                target_events: timeout_txs,
                timeout_relay_height,
                src_client_id: inner_req.src_client_id,
                dst_client_id: inner_req.dst_client_id,
                src_packet_seqs: inner_req.src_packet_sequences,
                dst_packet_seqs: inner_req.dst_packet_sequences,
            })
            .await
            .map_err(to_tonic_status)?;

        tracing::info!("Relay by tx request completed.");

        Ok(Response::new(api::RelayByTxResponse {
            tx,
            address: self.dst_ibc_multicall_address.to_string(),
        }))
    }

    async fn create_client(
        &self,
        request: Request<api::CreateClientRequest>,
    ) -> Result<Response<api::CreateClientResponse>, tonic::Status> {
        tracing::info!("Handling create client request for Eth to Eth...");

        let inner_req = request.into_inner();
        let tx = self
            .tx_builder
            .create_client(&inner_req.parameters)
            .await
            .map_err(to_tonic_status)?;

        tracing::info!("Create client request completed.");

        Ok(Response::new(api::CreateClientResponse {
            tx,
            address: String::new(),
        }))
    }

    async fn update_client(
        &self,
        request: Request<api::UpdateClientRequest>,
    ) -> Result<Response<api::UpdateClientResponse>, tonic::Status> {
        tracing::info!("Handling update client request for Eth to Eth...");

        let inner_req = request.into_inner();
        let tx = self
            .tx_builder
            .update_client(inner_req.dst_client_id)
            .await
            .map_err(to_tonic_status)?;

        tracing::info!("Update client request completed.");

        Ok(Response::new(api::UpdateClientResponse {
            tx,
            address: self.dst_ibc_multicall_address.to_string(),
        }))
    }
}

#[tonic::async_trait]
impl ProofApiModule for EthToEthProofApiModule {
    fn name(&self) -> &'static str {
        "eth_to_eth"
    }

    async fn create_service(
        &self,
        config: serde_json::Value,
    ) -> anyhow::Result<Box<dyn ProofApiService>> {
        let config: EthToEthConfig = serde_json::from_value(config)?;
        let service = EthToEthProofApiModuleService::new(config).await?;
        Ok(Box::new(service))
    }
}

impl EthToEthTxBuilder {
    async fn relay_events(&self, params: RelayEventsParams) -> anyhow::Result<Vec<u8>> {
        match self {
            Self::Attested(tb) => tb.relay_events(params).await,
            Self::Besu(tb) => tb.relay_events(params).await,
        }
    }

    async fn create_client(&self, parameters: &HashMap<String, String>) -> anyhow::Result<Vec<u8>> {
        match self {
            Self::Attested(tb) => tb.create_client(parameters),
            Self::Besu(tb) => tb.create_client(parameters).await,
        }
    }

    async fn update_client(&self, dst_client_id: String) -> anyhow::Result<Vec<u8>> {
        match self {
            Self::Attested(tb) => tb.update_client(dst_client_id).await,
            Self::Besu(tb) => tb.update_client(&dst_client_id).await,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{BesuConsensusType, BesuToEthConfig, EthToEthConfig, TxBuilderMode};
    use alloy::providers::{Provider, RootProvider};

    #[test]
    fn besu_to_eth_config_selects_besu_mode() {
        let config: BesuToEthConfig = serde_json::from_value(serde_json::json!({
            "src_rpc_url": "http://localhost:8545",
            "src_ics26_address": "0x0000000000000000000000000000000000000001",
            "dst_rpc_url": "http://localhost:9545",
            "dst_ics26_address": "0x0000000000000000000000000000000000000002",
            "dst_ibc_multicall_address": "0x0000000000000000000000000000000000000003",
            "consensus_type": "ibft2"
        }))
        .unwrap();

        let config = EthToEthConfig::from(config);
        assert_eq!(config.src_rpc_url, "http://localhost:8545");
        assert_eq!(config.dst_rpc_url, "http://localhost:9545");
        assert_eq!(
            config.dst_ibc_multicall_address,
            "0x0000000000000000000000000000000000000003"
                .parse::<alloy::primitives::Address>()
                .unwrap()
        );
        assert!(matches!(
            config.mode,
            TxBuilderMode::Besu(BesuConsensusType::Ibft2)
        ));
    }

    #[test]
    fn eth_to_eth_config_rejects_besu_mode() {
        let result = serde_json::from_value::<EthToEthConfig>(serde_json::json!({
            "src_rpc_url": "http://localhost:8545",
            "src_ics26_address": "0x0000000000000000000000000000000000000001",
            "dst_rpc_url": "http://localhost:9545",
            "dst_ics26_address": "0x0000000000000000000000000000000000000002",
            "dst_ibc_multicall_address": "0x0000000000000000000000000000000000000003",
            "mode": { "besu": "qbft" }
        }));

        assert!(
            result.is_err(),
            "Besu mode is configured through besu_to_eth"
        );
    }

    #[test]
    fn obsolete_src_chain_id_is_ignored() {
        let config: EthToEthConfig = serde_json::from_value(serde_json::json!({
            "src_chain_id": "legacy",
            "src_rpc_url": "http://localhost:8545",
            "src_ics26_address": "0x0000000000000000000000000000000000000001",
            "dst_rpc_url": "http://localhost:9545",
            "dst_ics26_address": "0x0000000000000000000000000000000000000002",
            "dst_ibc_multicall_address": "0x0000000000000000000000000000000000000003",
            "mode": { "attested": {
                "attestor": {
                    "attestor_query_timeout_ms": 5000,
                    "quorum_threshold": 1,
                    "attestor_endpoints": ["http://localhost:2025"]
                }
            }}
        }))
        .unwrap();

        assert!(
            serde_json::to_value(config).unwrap()["src_chain_id"].is_null(),
            "obsolete field must be accepted but not emitted"
        );
    }

    #[tokio::test]
    async fn https_provider_can_fetch_chain_id() {
        let provider: RootProvider = RootProvider::builder()
            .connect("https://ethereum-rpc.publicnode.com")
            .await
            .expect("connect to HTTPS Ethereum RPC endpoint");

        assert_eq!(provider.get_chain_id().await.unwrap(), 1);
    }
}
