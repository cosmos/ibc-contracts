// SPDX-License-Identifier: Apache-2.0

//! One-sided proof API module from Solana to Ethereum.
//!
//! Listens for IBC events on Solana, and builds ABI-encoded EVM multicall
//! transactions with attestation proofs. Payloads from Solana are already
//! ABI-encoded by the IFT program, so no translation is needed.

#![deny(clippy::nursery, clippy::pedantic, warnings)]
#![allow(missing_docs, unused_crate_dependencies)]

pub mod tx_builder;

use std::collections::HashMap;

use alloy::{
    primitives::{Address, TxHash},
    providers::{Provider, RootProvider},
};
use proof_api_lib::{
    aggregator::Config as AggregatorConfig,
    events::EurekaEventWithHeight,
    listener::{eth_eureka, solana, ChainListenerService},
    service_utils::{parse_eth_tx_hashes, parse_solana_tx_hashes, to_tonic_status},
    utils::RelayEventsParams,
};
use solana_sdk::commitment_config::CommitmentConfig;
use tonic::{Request, Response};

use proof_api_core::{
    api::{self, proof_api_service_server::ProofApiService},
    modules::ProofApiModule,
};

/// The `SolanaToEthProofApiModule` defines the Solana to Ethereum proof API module.
#[derive(Clone, Copy, Debug)]
pub struct SolanaToEthProofApiModule;

/// The proof API service from Solana to Ethereum.
struct SolanaToEthProofApiModuleService {
    /// Source chain listener for Solana.
    src_listener: solana::ChainListener,
    /// Target chain listener for Ethereum (for timeout events).
    target_listener: eth_eureka::ChainListener<RootProvider>,
    /// Transaction builder.
    tx_builder: SolanaToEthTxBuilder,
    /// ICS26 contract address on Ethereum.
    ics26_eth_address: Address,
}

/// Enum wrapping transaction builders for different modes.
enum SolanaToEthTxBuilder {
    /// Attestation light client mode.
    Attested(tx_builder::AttestedTxBuilder),
}

/// Configuration for the Solana to Ethereum proof API module.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct SolanaToEthConfig {
    /// The Solana RPC URL.
    pub solana_rpc_url: String,
    /// The Solana ICS26 router program ID.
    pub solana_ics26_program_id: String,
    /// The EVM RPC URL.
    pub eth_rpc_url: String,
    /// The ICS26 contract address on Ethereum.
    pub ics26_address: Address,
    /// Transaction builder mode.
    pub mode: SolanaToEthTxBuilderMode,
}

/// Transaction builder mode for Solana to Eth relay.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SolanaToEthTxBuilderMode {
    /// Attestation light client mode.
    Attested(AggregatorConfig),
}

impl SolanaToEthProofApiModuleService {
    async fn new(config: SolanaToEthConfig) -> anyhow::Result<Self> {
        let solana_ics26_program_id = config
            .solana_ics26_program_id
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid Solana ICS26 program ID: {e}"))?;

        let src_listener =
            solana::ChainListener::new(config.solana_rpc_url.clone(), solana_ics26_program_id);

        let provider = RootProvider::builder()
            .connect(&config.eth_rpc_url)
            .await
            .map_err(|e| anyhow::anyhow!("failed to create EVM provider: {e}"))?;

        let target_listener =
            eth_eureka::ChainListener::new(config.ics26_address, provider.clone());

        let tx_builder = match config.mode {
            SolanaToEthTxBuilderMode::Attested(aggregator_config) => {
                let attested =
                    tx_builder::AttestedTxBuilder::new(aggregator_config, provider).await?;
                SolanaToEthTxBuilder::Attested(attested)
            }
        };

        Ok(Self {
            src_listener,
            target_listener,
            tx_builder,
            ics26_eth_address: config.ics26_address,
        })
    }
}

#[tonic::async_trait]
impl ProofApiService for SolanaToEthProofApiModuleService {
    async fn info(
        &self,
        _request: Request<api::InfoRequest>,
    ) -> Result<Response<api::InfoResponse>, tonic::Status> {
        tracing::debug!("Handling info request for Solana to Eth");

        Ok(Response::new(api::InfoResponse {
            source_chain: Some(api::Chain {
                chain_id: "solana-localnet".to_string(),
                ibc_version: "2".to_string(),
                ibc_contract: self.src_listener.ics26_program_id().to_string(),
            }),
            target_chain: Some(api::Chain {
                chain_id: self
                    .target_listener
                    .chain_id()
                    .await
                    .map_err(to_tonic_status)?,
                ibc_version: "2".to_string(),
                ibc_contract: self.ics26_eth_address.to_string(),
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
        let eth_timeout_tx_hashes = parse_eth_tx_hashes(inner_req.timeout_tx_ids)?;
        let eth_timeout_txs = eth_timeout_tx_hashes
            .into_iter()
            .map(TxHash::from)
            .collect();

        let solana_src_events = self
            .src_listener
            .fetch_tx_events(solana_tx_hashes)
            .await
            .map_err(to_tonic_status)?;

        tracing::debug!("Fetched {} src events from Solana", solana_src_events.len());

        let target_events = self
            .target_listener
            .fetch_tx_events(eth_timeout_txs)
            .await
            .map_err(|e| tonic::Status::from_error(e.into()))?;

        tracing::debug!("Fetched {} timeout events from EVM", target_events.len());

        // Convert Solana events to generic EurekaEventWithHeight
        let src_events: Vec<EurekaEventWithHeight> = solana_src_events
            .into_iter()
            .map(EurekaEventWithHeight::from)
            .collect();

        // For timeouts in attested mode, get the current Solana slot
        // where non-membership is proven
        let timeout_relay_height = if target_events.is_empty() {
            None
        } else {
            let slot = self
                .src_listener
                .client()
                .get_slot_with_commitment(CommitmentConfig::finalized())
                .map_err(|e| tonic::Status::internal(format!("Failed to get Solana slot: {e}")))?;
            Some(slot)
        };

        let tx = self
            .tx_builder
            .relay_events(RelayEventsParams {
                src_events,
                target_events,
                timeout_relay_height,
                src_client_id: inner_req.src_client_id,
                dst_client_id: inner_req.dst_client_id,
                src_packet_seqs: inner_req.src_packet_sequences,
                dst_packet_seqs: inner_req.dst_packet_sequences,
            })
            .await
            .map_err(|e| tonic::Status::from_error(e.into()))?;

        tracing::debug!("Relay completed");

        Ok(Response::new(api::RelayByTxResponse {
            tx,
            address: self.ics26_eth_address.to_string(),
        }))
    }

    async fn create_client(
        &self,
        request: Request<api::CreateClientRequest>,
    ) -> Result<Response<api::CreateClientResponse>, tonic::Status> {
        tracing::debug!("Handling create client request for Solana to Eth");

        let inner_req = request.into_inner();
        let tx = self
            .tx_builder
            .create_client(&inner_req.parameters)
            .map_err(|e| tonic::Status::from_error(e.into()))?;

        Ok(Response::new(api::CreateClientResponse {
            tx,
            address: self.ics26_eth_address.to_string(),
        }))
    }

    async fn update_client(
        &self,
        request: Request<api::UpdateClientRequest>,
    ) -> Result<Response<api::UpdateClientResponse>, tonic::Status> {
        tracing::info!("Handling update client request for Solana to Eth");

        let inner_req = request.into_inner();
        let tx = self
            .tx_builder
            .update_client(&inner_req.dst_client_id)
            .await
            .map_err(|e| tonic::Status::from_error(e.into()))?;

        Ok(Response::new(api::UpdateClientResponse {
            tx,
            address: self.ics26_eth_address.to_string(),
        }))
    }
}

#[tonic::async_trait]
impl ProofApiModule for SolanaToEthProofApiModule {
    fn name(&self) -> &'static str {
        "solana_to_eth"
    }

    async fn create_service(
        &self,
        config: serde_json::Value,
    ) -> anyhow::Result<Box<dyn ProofApiService>> {
        let config: SolanaToEthConfig = serde_json::from_value(config)?;
        let service = SolanaToEthProofApiModuleService::new(config).await?;
        Ok(Box::new(service))
    }
}

impl SolanaToEthTxBuilder {
    async fn relay_events(&self, params: RelayEventsParams) -> anyhow::Result<Vec<u8>> {
        match self {
            Self::Attested(tb) => tb.relay_events(params).await,
        }
    }

    fn create_client(&self, parameters: &HashMap<String, String>) -> anyhow::Result<Vec<u8>> {
        match self {
            Self::Attested(tb) => tb.create_client(parameters),
        }
    }

    async fn update_client(&self, dst_client_id: &str) -> anyhow::Result<Vec<u8>> {
        match self {
            Self::Attested(tb) => tb.update_client(dst_client_id).await,
        }
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
