// SPDX-License-Identifier: Apache-2.0

//! Local/dev/testing-only one-sided proof API module from EVM dummy to EVM dummy.

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
use tx_builder::TxBuilder;

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct EvmDummyToEvmDummyConfig {
    pub src_chain_id: String,
    pub src_rpc_url: String,
    pub src_ics26_address: Address,
    pub dst_rpc_url: String,
    pub dst_ics26_address: Address,
}

#[derive(Clone, Copy, Debug)]
pub struct EvmDummyToEvmDummyProofApiModule;

struct EvmDummyToEvmDummyProofApiModuleService {
    src_chain_id: String,
    src_ics26_address: Address,
    src_listener: eth_eureka::ChainListener<RootProvider>,
    dst_listener: eth_eureka::ChainListener<RootProvider>,
    tx_builder: TxBuilder,
}

impl EvmDummyToEvmDummyProofApiModuleService {
    async fn new(config: EvmDummyToEvmDummyConfig) -> anyhow::Result<Self> {
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
        let tx_builder = TxBuilder::new(src_provider, dst_provider, config.dst_ics26_address);

        Ok(Self {
            src_chain_id: config.src_chain_id,
            src_ics26_address: config.src_ics26_address,
            src_listener,
            dst_listener,
            tx_builder,
        })
    }
}

#[tonic::async_trait]
impl ProofApiService for EvmDummyToEvmDummyProofApiModuleService {
    async fn info(
        &self,
        _request: Request<api::InfoRequest>,
    ) -> Result<Response<api::InfoResponse>, tonic::Status> {
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
                chain_id: self.src_chain_id.clone(),
                ibc_version: "2".to_string(),
                ibc_contract: self.src_ics26_address.to_string(),
            }),
            metadata: HashMap::default(),
        }))
    }

    async fn create_client(
        &self,
        _request: Request<api::CreateClientRequest>,
    ) -> Result<Response<api::CreateClientResponse>, tonic::Status> {
        Ok(Response::new(api::CreateClientResponse {
            tx: self.tx_builder.create_client(),
            address: String::new(),
        }))
    }

    async fn update_client(
        &self,
        request: Request<api::UpdateClientRequest>,
    ) -> Result<Response<api::UpdateClientResponse>, tonic::Status> {
        let inner = request.into_inner();
        let tx = self
            .tx_builder
            .update_client(&inner.dst_client_id)
            .await
            .map_err(to_tonic_status)?;
        Ok(Response::new(api::UpdateClientResponse {
            tx,
            address: self.tx_builder.ics26_router_address().to_string(),
        }))
    }

    async fn relay_by_tx(
        &self,
        request: Request<api::RelayByTxRequest>,
    ) -> Result<Response<api::RelayByTxResponse>, tonic::Status> {
        let inner = request.into_inner();
        let src_txs = parse_eth_tx_hashes(inner.source_tx_ids)?
            .into_iter()
            .map(TxHash::from)
            .collect();
        let timeout_txs = parse_eth_tx_hashes(inner.timeout_tx_ids)?
            .into_iter()
            .map(TxHash::from)
            .collect();
        let src_events = self
            .src_listener
            .fetch_tx_events(src_txs)
            .await
            .map_err(to_tonic_status)?;
        let target_events = self
            .dst_listener
            .fetch_tx_events(timeout_txs)
            .await
            .map_err(to_tonic_status)?;
        let tx = self
            .tx_builder
            .relay_events(RelayEventsParams {
                src_events,
                target_events,
                timeout_relay_height: None,
                src_client_id: inner.src_client_id,
                dst_client_id: inner.dst_client_id,
                src_packet_seqs: inner.src_packet_sequences,
                dst_packet_seqs: inner.dst_packet_sequences,
            })
            .await
            .map_err(to_tonic_status)?;
        Ok(Response::new(api::RelayByTxResponse {
            tx,
            address: self.tx_builder.ics26_router_address().to_string(),
        }))
    }
}

#[tonic::async_trait]
impl ProofApiModule for EvmDummyToEvmDummyProofApiModule {
    fn name(&self) -> &'static str {
        "evmdummy_to_evmdummy"
    }

    async fn create_service(
        &self,
        config: serde_json::Value,
    ) -> anyhow::Result<Box<dyn ProofApiService>> {
        let config: EvmDummyToEvmDummyConfig = serde_json::from_value(config)?;
        Ok(Box::new(
            EvmDummyToEvmDummyProofApiModuleService::new(config).await?,
        ))
    }
}
