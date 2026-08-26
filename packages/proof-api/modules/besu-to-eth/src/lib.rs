// SPDX-License-Identifier: Apache-2.0

//! One-sided Besu-to-Ethereum proof API module.

#![deny(clippy::nursery, clippy::pedantic, warnings, unused_crate_dependencies)]
#![allow(missing_docs)]

mod tx_builder;

use std::collections::HashMap;

use alloy::{
    primitives::{Address, TxHash},
    providers::{Provider, RootProvider},
};
use proof_api_core::{
    api::{self, proof_api_service_server::ProofApiService},
    modules::ProofApiModule,
};
use proof_api_lib::{
    listener::{eth_eureka, ChainListenerService},
    service_utils::{parse_eth_tx_hashes, to_tonic_status},
    utils::RelayEventsParams,
};
use tonic::{Request, Response};
use tracing as _;
use tx_builder::TxBuilder;

#[derive(Clone, Copy, Debug)]
pub struct BesuToEthProofApiModule;

struct BesuToEthProofApiModuleService {
    src_ics26_address: Address,
    src_listener: eth_eureka::ChainListener<RootProvider>,
    dst_listener: eth_eureka::ChainListener<RootProvider>,
    tx_builder: TxBuilder,
}

#[derive(Clone, Copy, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BesuConsensusType {
    Qbft,
    Ibft2,
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct BesuToEthConfig {
    pub src_rpc_url: String,
    pub src_ics26_address: Address,
    pub dst_rpc_url: String,
    pub dst_ics26_address: Address,
    pub consensus_type: BesuConsensusType,
}

impl BesuToEthProofApiModuleService {
    async fn new(config: BesuToEthConfig) -> anyhow::Result<Self> {
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
        let tx_builder = TxBuilder::new(
            src_provider,
            dst_provider,
            config.src_ics26_address,
            config.dst_ics26_address,
            config.consensus_type,
        );

        Ok(Self {
            src_ics26_address: config.src_ics26_address,
            src_listener,
            dst_listener,
            tx_builder,
        })
    }
}

#[tonic::async_trait]
impl ProofApiService for BesuToEthProofApiModuleService {
    async fn info(
        &self,
        request: Request<api::InfoRequest>,
    ) -> Result<Response<api::InfoResponse>, tonic::Status> {
        let src_chain = request.into_inner().src_chain;
        Ok(Response::new(api::InfoResponse {
            target_chain: Some(api::Chain {
                chain_id: self
                    .dst_listener
                    .chain_id()
                    .await
                    .map_err(to_tonic_status)?,
                ibc_version: "2".to_string(),
                ibc_contract: self.tx_builder.ics26_router_address().to_string(),
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
        let inner_req = request.into_inner();

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
        let timeout_events = self
            .dst_listener
            .fetch_tx_events(timeout_txs)
            .await
            .map_err(to_tonic_status)?;

        let timeout_relay_height = if timeout_events.is_empty() {
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
                target_events: timeout_events,
                timeout_relay_height,
                src_client_id: inner_req.src_client_id,
                dst_client_id: inner_req.dst_client_id,
                src_packet_seqs: inner_req.src_packet_sequences,
                dst_packet_seqs: inner_req.dst_packet_sequences,
            })
            .await
            .map_err(to_tonic_status)?;

        Ok(Response::new(api::RelayByTxResponse {
            tx,
            address: self.tx_builder.ics26_router_address().to_string(),
        }))
    }

    async fn create_client(
        &self,
        request: Request<api::CreateClientRequest>,
    ) -> Result<Response<api::CreateClientResponse>, tonic::Status> {
        let inner_req = request.into_inner();
        let tx = self
            .tx_builder
            .create_client(&inner_req.parameters)
            .await
            .map_err(to_tonic_status)?;

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
            .update_client(&inner_req.dst_client_id)
            .await
            .map_err(to_tonic_status)?;

        Ok(Response::new(api::UpdateClientResponse {
            tx,
            address: self.tx_builder.ics26_router_address().to_string(),
        }))
    }
}

#[tonic::async_trait]
impl ProofApiModule for BesuToEthProofApiModule {
    fn name(&self) -> &'static str {
        "besu_to_eth"
    }

    async fn create_service(
        &self,
        config: serde_json::Value,
    ) -> anyhow::Result<Box<dyn ProofApiService>> {
        let config: BesuToEthConfig = serde_json::from_value(config)?;
        let service = BesuToEthProofApiModuleService::new(config).await?;
        Ok(Box::new(service))
    }
}

#[cfg(test)]
mod tests {
    use super::BesuToEthConfig;

    #[test]
    fn obsolete_src_chain_id_is_ignored() {
        let config: BesuToEthConfig = serde_json::from_value(serde_json::json!({
            "src_chain_id": "legacy",
            "src_rpc_url": "http://localhost:8545",
            "src_ics26_address": "0x0000000000000000000000000000000000000001",
            "dst_rpc_url": "http://localhost:9545",
            "dst_ics26_address": "0x0000000000000000000000000000000000000002",
            "consensus_type": "qbft"
        }))
        .unwrap();

        assert!(
            serde_json::to_value(config).unwrap()["src_chain_id"].is_null(),
            "obsolete field must be accepted but not emitted"
        );
    }
}
