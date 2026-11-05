// SPDX-License-Identifier: Apache-2.0

//! Relay from a Besu BFT chain, proving the source router storage against the destination Besu
//! light client.

mod tx_builder;

use alloy::primitives::Address;
use proof_api_core::{api::proof_api_service_server::ProofApiService, modules::ProofApiModule};

use crate::{EthToEthConfig, EthToEthProofApiModuleService, TxBuilderMode};

pub use tx_builder::TxBuilder;

/// The `BesuToEthProofApiModule` struct defines the Besu to Ethereum proof API module.
#[derive(Clone, Copy, Debug)]
pub struct BesuToEthProofApiModule;

/// The consensus protocol of the source Besu chain.
#[derive(Clone, Copy, Debug, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum BesuConsensusType {
    Qbft,
    Ibft2,
}

/// The configuration for the Besu to Ethereum proof API module.
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct BesuToEthConfig {
    pub src_rpc_url: String,
    pub src_ics26_address: Address,
    pub dst_rpc_url: String,
    pub dst_ics26_address: Address,
    /// The destination `IBCMulticall` address that relay and update client transactions are
    /// submitted to.
    pub dst_ibc_multicall_address: Address,
    /// The source consensus type, which selects the commit-seal signing form of the headers
    /// submitted in client updates.
    pub consensus_type: BesuConsensusType,
}

impl From<BesuToEthConfig> for EthToEthConfig {
    fn from(config: BesuToEthConfig) -> Self {
        Self {
            src_rpc_url: config.src_rpc_url,
            src_ics26_address: config.src_ics26_address,
            dst_rpc_url: config.dst_rpc_url,
            dst_ics26_address: config.dst_ics26_address,
            dst_ibc_multicall_address: config.dst_ibc_multicall_address,
            mode: TxBuilderMode::Besu(config.consensus_type),
        }
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
        let service = EthToEthProofApiModuleService::new(config.into()).await?;
        Ok(Box::new(service))
    }
}
