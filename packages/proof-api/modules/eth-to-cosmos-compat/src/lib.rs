// SPDX-License-Identifier: Apache-2.0

//! Ethereum to Cosmos backwards compatible proof API module.

#![deny(
    clippy::nursery,
    clippy::pedantic,
    warnings,
    missing_docs,
    unused_crate_dependencies
)]

use ibc_eureka_relayer_core_v1_3::{
    api::{self as api_v1_3, relayer_service_server::RelayerService as RelayerServiceV1_3},
    modules::RelayerModule as RelayerModuleV1_3,
};
use ibc_eureka_relayer_eth_to_cosmos_v1_3::EthToCosmosRelayerModule as EthToCosmosRelayerModuleV1_3;
use ibc_eureka_utils::rpc::TendermintRpcExt;
use ibc_proto_eureka::ibc::lightclients::wasm::v1::ClientState;
use proof_api_core::{
    api::{self, proof_api_service_server::ProofApiService},
    modules::ProofApiModule,
};
use proof_api_eth_to_cosmos::{EthToCosmosConfig, EthToCosmosProofApiModule, TxBuilderMode};
use proof_api_lib::listener::cosmos_sdk;
use prost::Message;
use serde_json::Value;
use tendermint_rpc::HttpClient;
use tonic::{Request, Response};

/// The checksum for the v1.3 Ethereum wasm client.
const V1_3_CHECKSUM: &[u8] = &[
    175, 132, 204, 204, 163, 231, 70, 217, 196, 234, 152, 12, 109, 27, 69, 17, 222, 15, 169, 98,
    237, 80, 3, 222, 232, 203, 68, 237, 161, 14, 69, 104,
];

/// The key for the checksum hex in the parameters map.
const CHECKSUM_HEX: &str = "checksum_hex";

/// The Ethereum to Cosmos backwards compatible proof API module.
#[derive(Clone, Copy, Debug)]
pub struct EthToCosmosCompatProofApiModule;

/// The backwards compatible proof API service from Ethereum to Cosmos.
struct EthToCosmosCompatProofApiModuleService {
    tm_listener: cosmos_sdk::ChainListener,
    old_service: Box<dyn RelayerServiceV1_3>,
    new_service: Box<dyn ProofApiService>,
}

impl EthToCosmosCompatProofApiModuleService {
    fn new(
        config: &EthToCosmosConfig,
        old_service: Box<dyn RelayerServiceV1_3>,
        new_service: Box<dyn ProofApiService>,
    ) -> Self {
        let tm_client = HttpClient::from_rpc_url(&config.tm_rpc_url);
        let tm_listener = cosmos_sdk::ChainListener::new(tm_client);
        Self {
            tm_listener,
            old_service,
            new_service,
        }
    }

    async fn client_checksum(&self, dst_client_id: &str) -> anyhow::Result<Vec<u8>> {
        let client_state = self
            .tm_listener
            .client()
            .client_state(dst_client_id.to_string())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get client state of {dst_client_id}: {e}"))?;

        Ok(ClientState::decode(&*client_state.value)
            .map_err(|e| anyhow::anyhow!("Failed to decode client state of {dst_client_id}: {e}"))?
            .checksum)
    }
}

#[tonic::async_trait]
impl ProofApiService for EthToCosmosCompatProofApiModuleService {
    #[tracing::instrument(skip_all, err(Debug))]
    async fn info(
        &self,
        request: Request<api::InfoRequest>,
    ) -> Result<Response<api::InfoResponse>, tonic::Status> {
        self.new_service.info(request).await
    }

    #[tracing::instrument(skip_all, err(Debug))]
    async fn relay_by_tx(
        &self,
        request: Request<api::RelayByTxRequest>,
    ) -> Result<Response<api::RelayByTxResponse>, tonic::Status> {
        let req = request.get_ref();
        let checksum = self
            .client_checksum(&req.dst_client_id)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        if checksum == V1_3_CHECKSUM {
            tracing::info!("Using backwards compatible relay_by_tx");
            let inner = request.into_inner();
            let resp = self
                .old_service
                .relay_by_tx(Request::new(api_v1_3::RelayByTxRequest {
                    src_chain: inner.src_chain,
                    dst_chain: inner.dst_chain,
                    source_tx_ids: inner.source_tx_ids,
                    timeout_tx_ids: inner.timeout_tx_ids,
                    src_client_id: inner.src_client_id,
                    dst_client_id: inner.dst_client_id,
                    src_packet_sequences: inner.src_packet_sequences,
                    dst_packet_sequences: inner.dst_packet_sequences,
                }))
                .await?
                .into_inner();
            Ok(Response::new(api::RelayByTxResponse {
                tx: resp.tx,
                address: resp.address,
            }))
        } else {
            self.new_service.relay_by_tx(request).await
        }
    }

    #[tracing::instrument(skip_all, err(Debug))]
    async fn create_client(
        &self,
        request: Request<api::CreateClientRequest>,
    ) -> Result<Response<api::CreateClientResponse>, tonic::Status> {
        let checksum = hex::decode(request.get_ref().parameters.get(CHECKSUM_HEX).ok_or_else(
            || tonic::Status::invalid_argument("Checksum hex parameter is missing in request"),
        )?)
        .map_err(|e| {
            tonic::Status::invalid_argument(format!("Failed to decode checksum hex: {e}"))
        })?;

        if checksum == V1_3_CHECKSUM {
            tracing::info!("Using backwards compatible create_client");
            let inner = request.into_inner();
            let resp = self
                .old_service
                .create_client(Request::new(api_v1_3::CreateClientRequest {
                    src_chain: inner.src_chain,
                    dst_chain: inner.dst_chain,
                    parameters: inner.parameters,
                }))
                .await?
                .into_inner();
            Ok(Response::new(api::CreateClientResponse {
                tx: resp.tx,
                address: resp.address,
            }))
        } else {
            self.new_service.create_client(request).await
        }
    }

    #[tracing::instrument(skip_all, err(Debug))]
    async fn update_client(
        &self,
        request: Request<api::UpdateClientRequest>,
    ) -> Result<Response<api::UpdateClientResponse>, tonic::Status> {
        let req = request.get_ref();
        let checksum = self
            .client_checksum(&req.dst_client_id)
            .await
            .map_err(|e| tonic::Status::internal(e.to_string()))?;

        if checksum == V1_3_CHECKSUM {
            tracing::info!("Using backwards compatible update_client");
            let inner = request.into_inner();
            let resp = self
                .old_service
                .update_client(Request::new(api_v1_3::UpdateClientRequest {
                    src_chain: inner.src_chain,
                    dst_chain: inner.dst_chain,
                    dst_client_id: inner.dst_client_id,
                }))
                .await?
                .into_inner();
            Ok(Response::new(api::UpdateClientResponse {
                tx: resp.tx,
                address: resp.address,
            }))
        } else {
            self.new_service.update_client(request).await
        }
    }
}

#[tonic::async_trait]
impl ProofApiModule for EthToCosmosCompatProofApiModule {
    fn name(&self) -> &'static str {
        "eth_to_cosmos_compat"
    }

    #[tracing::instrument(skip_all, err(Debug))]
    async fn create_service(&self, config: Value) -> anyhow::Result<Box<dyn ProofApiService>> {
        let eth_to_cosmos_config = serde_json::from_value::<EthToCosmosConfig>(config.clone())
            .map_err(|e| anyhow::anyhow!("failed to parse config: {e}"))?;
        let legacy_config = legacy_config(&eth_to_cosmos_config)?;

        let old_service = EthToCosmosRelayerModuleV1_3
            .create_service(legacy_config)
            .await?;

        let new_service = EthToCosmosProofApiModule.create_service(config).await?;

        tracing::info!("Starting Ethereum to Cosmos backwards compatible proof API server.");
        Ok(Box::new(EthToCosmosCompatProofApiModuleService::new(
            &eth_to_cosmos_config,
            old_service,
            new_service,
        )))
    }
}

fn legacy_config(eth_to_cosmos_config: &EthToCosmosConfig) -> anyhow::Result<Value> {
    let mock = match &eth_to_cosmos_config.mode {
        TxBuilderMode::Real => false,
        TxBuilderMode::Mock => true,
        TxBuilderMode::Attested(_) => {
            anyhow::bail!("eth_to_cosmos_compat does not support attested mode")
        }
    };

    let mut config = serde_json::to_value(eth_to_cosmos_config)?;
    config["mock"] = Value::Bool(mock);
    config
        .as_object_mut()
        .ok_or_else(|| anyhow::anyhow!("eth_to_cosmos_compat config must be an object"))?
        .remove("mode");
    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::V1_3_CHECKSUM;

    #[test]
    fn v1_3_checksum_matches_expected_hex() {
        assert_eq!(
            hex::decode("af84cccca3e746d9c4ea980c6d1b4511de0fa962ed5003dee8cb44eda10e4568")
                .expect("valid checksum hex"),
            V1_3_CHECKSUM
        );
    }
}
