// SPDX-License-Identifier: Apache-2.0

//! This module implements the `BeaconApiClient` to interact with the Ethereum Beacon API.

use ethereum_types::consensus::{
    beacon_block::BeaconBlock,
    bootstrap::LightClientBootstrap,
    genesis::Genesis,
    light_client_header::{LightClientFinalityUpdate, LightClientUpdate},
    spec::Spec,
};
use reqwest::{Client, StatusCode};
use serde::de::DeserializeOwned;
use tracing::debug;

use super::{
    error::{BeaconApiClientError, InternalServerError, NotFoundError},
    response::{BeaconBlockRoot, Response, Version},
};

const SPEC_PATH: &str = "/eth/v1/config/spec";
const GENESIS_PATH: &str = "/eth/v1/beacon/genesis";
const BEACON_BLOCKS_V1_PATH: &str = "/eth/v1/beacon/blocks";
const BEACON_BLOCKS_V2_PATH: &str = "/eth/v2/beacon/blocks";
const LIGHT_CLIENT_BOOTSTRAP_PATH: &str = "/eth/v1/beacon/light_client/bootstrap";
const LIGHT_CLIENT_FINALITY_UPDATE_PATH: &str = "/eth/v1/beacon/light_client/finality_update";
const LIGHT_CLIENT_UPDATES_PATH: &str = "/eth/v1/beacon/light_client/updates";

/// The api client for interacting with the Beacon API
#[allow(clippy::module_name_repetitions)]
pub struct BeaconApiClient {
    client: Client,
    base_url: String,
}

impl BeaconApiClient {
    /// Create new `BeaconApiClient`
    #[must_use]
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
        }
    }

    /// Fetches the Beacon spec
    /// # Errors
    /// Returns an error if the request fails or the response is not successful deserialized
    pub async fn spec(&self) -> Result<Response<Spec>, BeaconApiClientError> {
        self.get_json(SPEC_PATH).await
    }

    /// Retrieve details of the chain's genesis which can be used to identify chain.
    /// # Errors
    /// Returns an error if the request fails or the response is not successful deserialized
    pub async fn genesis(&self) -> Result<Response<Genesis>, BeaconApiClientError> {
        self.get_json(GENESIS_PATH).await
    }

    /// Fetches the `LigthClientBootstrap` for a given beacon block root
    /// # Errors
    /// Returns an error if the request fails or the response is not successful deserialized
    pub async fn light_client_bootstrap(
        &self,
        beacon_block_root: &str,
    ) -> Result<Response<LightClientBootstrap>, BeaconApiClientError> {
        self.get_json(&format!(
            "{LIGHT_CLIENT_BOOTSTRAP_PATH}/{beacon_block_root}"
        ))
        .await
    }

    /// Fetches the Beacon block for a given block id
    /// # Errors
    /// Returns an error if the request fails or the response is not successful deserialized
    pub async fn beacon_block(&self, block_id: &str) -> Result<BeaconBlock, BeaconApiClientError> {
        let resp: Response<BeaconBlock> = self
            .get_json(&format!("{BEACON_BLOCKS_V2_PATH}/{block_id}"))
            .await?;
        Ok(resp.data)
    }

    /// Fetches the Beacon block root for a given block id
    /// # Errors
    /// Returns an error if the request fails or the response is not successful deserialized
    pub async fn beacon_block_root(&self, block_id: &str) -> Result<String, BeaconApiClientError> {
        let resp: Response<BeaconBlockRoot> = self
            .get_json(&format!("{BEACON_BLOCKS_V1_PATH}/{block_id}/root"))
            .await?;

        Ok(resp.data.root)
    }

    /// Fetches the latest Beacon light client finality update
    /// # Errors
    /// Returns an error if the request fails or the response is not successful deserialized
    pub async fn finality_update(
        &self,
    ) -> Result<Response<LightClientFinalityUpdate, Version>, BeaconApiClientError> {
        self.get_json(LIGHT_CLIENT_FINALITY_UPDATE_PATH).await
    }

    /// Fetches Beacon light client updates starting from a given period
    /// # Errors
    /// Returns an error if the request fails or the response is not successful deserialized
    pub async fn light_client_updates(
        &self,
        start_period: u64,
        count: u64,
    ) -> Result<Vec<Response<LightClientUpdate>>, BeaconApiClientError> {
        self.get_json(&format!(
            "{LIGHT_CLIENT_UPDATES_PATH}?start_period={start_period}&count={count}"
        ))
        .await
    }

    // Helper functions
    #[tracing::instrument(skip_all)]
    async fn get_json<T: DeserializeOwned>(&self, path: &str) -> Result<T, BeaconApiClientError> {
        let url = format!("{}{}", self.base_url, path);

        debug!(%url, "get_json");

        let res = self.client.get(url).send().await?;

        match res.status() {
            StatusCode::OK => {
                let bytes = res.bytes().await?;

                debug!(response = %String::from_utf8_lossy(&bytes), "get_json");

                Ok(serde_json::from_slice(&bytes).map_err(BeaconApiClientError::Json)?)
            }
            StatusCode::NOT_FOUND => Err(BeaconApiClientError::NotFound(
                res.json::<NotFoundError>().await?,
            )),
            StatusCode::INTERNAL_SERVER_ERROR => Err(BeaconApiClientError::Internal(
                res.json::<InternalServerError>().await?,
            )),
            code => Err(BeaconApiClientError::Other {
                code,
                text: res.text().await?,
            }),
        }
    }
}
