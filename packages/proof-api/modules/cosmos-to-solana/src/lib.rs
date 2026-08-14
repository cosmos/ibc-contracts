// SPDX-License-Identifier: Apache-2.0

//! This is a one-sided proof API module from a Cosmos SDK chain to Solana.

#![deny(clippy::nursery, clippy::pedantic, warnings, unused_crate_dependencies)]
#![allow(missing_docs, unused_crate_dependencies)]

pub mod borsh_conversions;
pub mod constants;
pub mod gmp;
pub mod ift;
pub mod proto;
pub mod tx_builder;

use std::collections::HashMap;

use ibc_eureka_utils::rpc::TendermintRpcExt;
use proof_api_lib::aggregator::Config as AggregatorConfig;
use proof_api_lib::events::SolanaEurekaEvent;
use proof_api_lib::listener::cosmos_sdk;
use proof_api_lib::listener::solana;
use proof_api_lib::listener::ChainListenerService;
use proof_api_lib::service_utils::parse_cosmos_tx_hashes;
use proof_api_lib::service_utils::parse_solana_tx_hashes;
use proof_api_lib::service_utils::to_tonic_status;
use proof_api_lib::utils::wait_for_condition;
use tendermint_rpc::HttpClient;
use tonic::{Request, Response};

use proof_api_core::{
    api::{self, proof_api_service_server::ProofApiService},
    modules::ProofApiModule,
};

/// The `CosmosToSolanaProofApiModule` struct defines the Cosmos to Solana proof API module.
#[derive(Clone, Copy, Debug)]
pub struct CosmosToSolanaProofApiModule;

/// The `CosmosToSolanaProofApiModuleService` defines the proof API service from Cosmos to Solana.
#[allow(dead_code)]
struct CosmosToSolanaProofApiModuleService {
    /// The souce chain listener for Cosmos.
    src_listener: cosmos_sdk::ChainListener,
    /// The target chain listener for Solana.
    target_listener: solana::ChainListener,
    /// The transaction builder (either ICS07 Tendermint or Attested).
    tx_builder: CosmosToSolanaTxBuilder,
}

/// Enum wrapping transaction builders for different modes.
#[allow(clippy::large_enum_variant)]
enum CosmosToSolanaTxBuilder {
    /// ICS07 Tendermint light client mode.
    Ics07Tendermint(tx_builder::TxBuilder),
    /// Attestation light client mode.
    Attested(tx_builder::AttestedTxBuilder),
}

/// The configuration for the Cosmos to Solana proof API module.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct CosmosToSolanaConfig {
    /// The source tendermint RPC URL.
    pub source_rpc_url: String,
    /// The Solana RPC URL.
    pub target_rpc_url: String,
    /// The Solana ICS26 router program ID.
    pub solana_ics26_program_id: String,
    /// The Solana fee payer address.
    pub solana_fee_payer: String,
    /// Address Lookup Table address for reducing transaction size (optional).
    pub solana_alt_address: Option<String>,
    /// Whitelisted Solana IFT program IDs. Only IFT instructions targeting
    /// these programs are relayed (defense-in-depth against untrusted GMP
    /// sender fields). Empty means IFT relay is disabled.
    #[serde(default)]
    pub solana_ift_program_ids: Vec<String>,
    /// Signature threshold below which pre-verification is skipped.
    /// None = always use pre-verification, Some(n) = skip when signatures ≤ n.
    /// Default: Some(50)
    #[serde(default = "default_skip_pre_verify_threshold")]
    pub skip_pre_verify_threshold: Option<usize>,
    /// Transaction builder mode. Defaults to ICS07 Tendermint.
    #[serde(default)]
    pub mode: SolanaTxBuilderMode,
}

/// Transaction builder mode for Cosmos to Solana relay.
#[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SolanaTxBuilderMode {
    /// ICS07 Tendermint light client mode (default).
    /// Uses Tendermint header verification on Solana.
    #[default]
    Ics07Tendermint,
    /// Attestation light client mode.
    /// Uses attestor signatures for state verification via aggregator.
    Attested(AggregatorConfig),
}

#[allow(
    clippy::unnecessary_wraps,
    reason = "Option required for serde default to match field type"
)]
const fn default_skip_pre_verify_threshold() -> Option<usize> {
    Some(50)
}

impl CosmosToSolanaProofApiModuleService {
    async fn new(config: &CosmosToSolanaConfig) -> anyhow::Result<Self> {
        let src_listener =
            cosmos_sdk::ChainListener::new(HttpClient::from_rpc_url(&config.source_rpc_url));

        let solana_ics26_program_id = config
            .solana_ics26_program_id
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid Solana ICS26 program ID: {e}"))?;

        let target_listener =
            solana::ChainListener::new(config.target_rpc_url.clone(), solana_ics26_program_id);

        let fee_payer = config
            .solana_fee_payer
            .parse()
            .map_err(|e| anyhow::anyhow!("Invalid fee payer address: {e}"))?;

        let alt_address = config
            .solana_alt_address
            .as_ref()
            .map(|addr| addr.parse())
            .transpose()
            .map_err(|e| anyhow::anyhow!("Invalid ALT address: {e}"))?;

        let ift_program_ids = config
            .solana_ift_program_ids
            .iter()
            .map(|addr| addr.parse())
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| anyhow::anyhow!("Invalid IFT program ID: {e}"))?;

        let tx_builder = match &config.mode {
            SolanaTxBuilderMode::Ics07Tendermint => {
                let builder = tx_builder::TxBuilder::new(
                    src_listener.client().clone(),
                    target_listener.client().clone(),
                    solana_ics26_program_id,
                    fee_payer,
                    alt_address,
                    config.skip_pre_verify_threshold,
                    ift_program_ids,
                )?;
                CosmosToSolanaTxBuilder::Ics07Tendermint(builder)
            }
            SolanaTxBuilderMode::Attested(aggregator_config) => {
                let base_builder = tx_builder::TxBuilder::new(
                    src_listener.client().clone(),
                    target_listener.client().clone(),
                    solana_ics26_program_id,
                    fee_payer,
                    alt_address,
                    config.skip_pre_verify_threshold,
                    ift_program_ids,
                )?;
                let attested_builder =
                    tx_builder::AttestedTxBuilder::new(aggregator_config.clone(), base_builder)
                        .await
                        .map_err(|e| {
                            anyhow::anyhow!("Failed to create attested tx builder: {e}")
                        })?;
                CosmosToSolanaTxBuilder::Attested(attested_builder)
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
impl ProofApiService for CosmosToSolanaProofApiModuleService {
    async fn info(
        &self,
        _request: Request<api::InfoRequest>,
    ) -> Result<Response<api::InfoResponse>, tonic::Status> {
        tracing::debug!("Handling info request");

        Ok(Response::new(api::InfoResponse {
            source_chain: Some(api::Chain {
                chain_id: self
                    .src_listener
                    .chain_id()
                    .await
                    .map_err(to_tonic_status)?,
                ibc_version: "2".to_string(),
                ibc_contract: String::new(),
            }),
            target_chain: Some(api::Chain {
                chain_id: "solana-localnet".to_string(),
                ibc_version: "2".to_string(),
                ibc_contract: self.target_listener.ics26_program_id().to_string(),
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
        let src_txs = parse_cosmos_tx_hashes(inner_req.source_tx_ids)?;
        let target_txs = parse_solana_tx_hashes(inner_req.timeout_tx_ids)?;

        let src_events = self
            .src_listener
            .fetch_tx_events(src_txs)
            .await
            .map_err(|e| tonic::Status::from_error(e.into()))?;

        tracing::debug!(
            "Fetched {} src events, fetching target events...",
            src_events.len()
        );

        let target_events = self
            .target_listener
            .fetch_tx_events(target_txs)
            .await
            .map_err(|e| tonic::Status::from_error(e.into()))?;

        tracing::debug!("Fetched {} target events", target_events.len());

        // For timeouts in attested mode, find a source chain height whose
        // block time is past the packet timeout. ICS07 Tendermint does not
        // use this value.
        let timeout_relay_height = if self.tx_builder.is_attested() {
            let max_timeout = target_events
                .iter()
                .filter_map(|e| match &e.event {
                    SolanaEurekaEvent::SendPacket(event) => Some(event.timeout_timestamp),
                    SolanaEurekaEvent::WriteAcknowledgement(_) => None,
                })
                .max();

            if let Some(max_timeout) = max_timeout {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .expect("system clock before UNIX epoch")
                    .as_secs();
                let cutoff = now.min(max_timeout);
                let src_listener = &self.src_listener;
                wait_for_condition(
                    std::time::Duration::from_secs(24),
                    std::time::Duration::from_secs(1),
                    || async {
                        let (_, block_time) = src_listener.get_block_height_with_time().await?;
                        if block_time >= cutoff {
                            Ok(true)
                        } else {
                            tracing::debug!(
                                "Cosmos block time ({block_time}) < cutoff ({cutoff}), waiting"
                            );
                            Ok(false)
                        }
                    },
                )
                .await
                .map_err(to_tonic_status)?;
                let (height, _) = self
                    .src_listener
                    .get_block_height_with_time()
                    .await
                    .map_err(to_tonic_status)?;
                Some(height)
            } else {
                None
            }
        } else {
            None
        };

        let (packet_txs, update_client) = self
            .tx_builder
            .relay_events(
                src_events,
                target_events,
                &inner_req.src_client_id,
                &inner_req.dst_client_id,
                &inner_req.src_packet_sequences,
                &inner_req.dst_packet_sequences,
                timeout_relay_height,
            )
            .await
            .map_err(to_tonic_status)?;

        tracing::debug!(
            "Relay completed: {} packets{}",
            packet_txs.len(),
            if update_client.is_some() {
                " +update"
            } else {
                ""
            }
        );

        let batch = api::SolanaRelayPacketBatch {
            packets: packet_txs,
            update_client,
        };
        let tx = prost::Message::encode_to_vec(&batch);

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
        tracing::info!("Handling update client request for Cosmos to Solana...");

        let inner_req = request.into_inner();
        let solana_update_client = self
            .tx_builder
            .update_client(&inner_req.dst_client_id)
            .await
            .map_err(|e| tonic::Status::from_error(e.into()))?;

        tracing::debug!(
            "Update client: {} chunks",
            solana_update_client.chunk_txs.len()
        );

        let tx = prost::Message::encode_to_vec(&solana_update_client);

        Ok(Response::new(api::UpdateClientResponse {
            tx,
            address: String::new(),
        }))
    }
}

#[tonic::async_trait]
impl ProofApiModule for CosmosToSolanaProofApiModule {
    fn name(&self) -> &'static str {
        "cosmos_to_solana"
    }

    async fn create_service(
        &self,
        config: serde_json::Value,
    ) -> anyhow::Result<Box<dyn ProofApiService>> {
        let config: CosmosToSolanaConfig = serde_json::from_value(config)?;
        let service = CosmosToSolanaProofApiModuleService::new(&config).await?;
        Ok(Box::new(service))
    }
}

// Implement dispatch methods on the enum
impl CosmosToSolanaTxBuilder {
    #[allow(clippy::too_many_arguments)]
    async fn relay_events(
        &self,
        src_events: Vec<proof_api_lib::events::EurekaEventWithHeight>,
        target_events: Vec<proof_api_lib::events::SolanaEurekaEventWithHeight>,
        src_client_id: &str,
        dst_client_id: &str,
        src_packet_seqs: &[u64],
        dst_packet_seqs: &[u64],
        timeout_relay_height: Option<u64>,
    ) -> anyhow::Result<(
        Vec<proof_api_core::api::SolanaPacketTxs>,
        Option<proof_api_core::api::SolanaUpdateClient>,
    )> {
        match self {
            Self::Ics07Tendermint(tb) => {
                tb.relay_events_with_update(tx_builder::RelayParams {
                    src_events,
                    dest_events: target_events,
                    src_client_id: src_client_id.to_string(),
                    dst_client_id: dst_client_id.to_string(),
                    src_packet_seqs: src_packet_seqs.to_vec(),
                    dst_packet_seqs: dst_packet_seqs.to_vec(),
                    timeout_relay_height,
                })
                .await
            }
            Self::Attested(tb) => {
                tb.relay_events(tx_builder::RelayParams {
                    src_events,
                    dest_events: target_events,
                    src_client_id: src_client_id.to_string(),
                    dst_client_id: dst_client_id.to_string(),
                    src_packet_seqs: src_packet_seqs.to_vec(),
                    dst_packet_seqs: dst_packet_seqs.to_vec(),
                    timeout_relay_height,
                })
                .await
            }
        }
    }

    async fn create_client(&self, parameters: &HashMap<String, String>) -> anyhow::Result<Vec<u8>> {
        match self {
            Self::Ics07Tendermint(tb) => tb.create_client(parameters).await,
            Self::Attested(tb) => tb.tx_builder().create_client(parameters).await,
        }
    }

    async fn update_client(
        &self,
        dst_client_id: &str,
    ) -> anyhow::Result<proof_api_core::api::SolanaUpdateClient> {
        match self {
            Self::Ics07Tendermint(tb) => tb.update_client(dst_client_id).await,
            Self::Attested(tb) => tb.update_client(dst_client_id).await,
        }
    }

    const fn is_attested(&self) -> bool {
        matches!(self, Self::Attested(_))
    }
}
