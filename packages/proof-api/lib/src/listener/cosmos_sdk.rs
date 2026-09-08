// SPDX-License-Identifier: Apache-2.0

//! This module defines the chain listener for 'ibc-go-eureka'.

use futures::future;
use tendermint::{block::Height, Hash};
use tendermint_rpc::{Client, HttpClient};

use anyhow::Result;
use tracing::instrument;

use crate::{
    chain::CosmosSdk,
    events::{EurekaEvent, EurekaEventWithHeight},
};

use super::ChainListenerService;

/// The `ChainListener` listens for events on the Cosmos SDK chain.
pub struct ChainListener(HttpClient);

impl ChainListener {
    /// Create a new [`Self`] instance.
    #[must_use]
    pub const fn new(tm_client: HttpClient) -> Self {
        Self(tm_client)
    }

    /// Get the HTTP client for tendermint.
    #[must_use]
    pub const fn client(&self) -> &HttpClient {
        &self.0
    }

    /// Get the chain ID.
    ///
    /// # Errors
    /// Returns an error if the chain ID cannot be fetched.
    #[instrument(skip(self), err)]
    pub async fn chain_id(&self) -> Result<String> {
        Ok(self
            .client()
            .latest_block()
            .await?
            .block
            .header
            .chain_id
            .into())
    }

    /// Get the latest block height.
    ///
    /// # Errors
    /// Returns an error if the block height cannot be fetched.
    #[instrument(skip(self), err)]
    pub async fn get_block_height(&self) -> Result<u64> {
        Ok(self
            .client()
            .latest_block()
            .await?
            .block
            .header
            .height
            .value())
    }

    /// Get the latest block height and timestamp (seconds since epoch).
    ///
    /// # Errors
    /// Returns an error if the block cannot be fetched or the timestamp is
    /// negative.
    #[instrument(skip(self), err)]
    pub async fn get_block_height_with_time(&self) -> Result<(u64, u64)> {
        let block = self.client().latest_block().await?.block;
        let height = block.header.height.value();
        let time = u64::try_from(block.header.time.unix_timestamp())
            .map_err(|_| anyhow::anyhow!("block timestamp is negative"))?;
        Ok((height, time))
    }
}

#[async_trait::async_trait]
impl ChainListenerService<CosmosSdk> for ChainListener {
    #[instrument(skip(self), fields(tx_count = tx_ids.len()), err(Debug))]
    async fn fetch_tx_events(&self, tx_ids: Vec<Hash>) -> Result<Vec<EurekaEventWithHeight>> {
        Ok(
            future::try_join_all(tx_ids.into_iter().map(|tx_id| async move {
                let tx_response = self.client().tx(tx_id, false).await?;
                let height = tx_response.height.value();
                Ok::<_, tendermint_rpc::Error>(tx_response.tx_result.events.into_iter().filter_map(
                    move |e| {
                        let event_type = EurekaEvent::try_from(e).ok()?;
                        Some(EurekaEventWithHeight {
                            event: event_type,
                            height,
                        })
                    },
                ))
            }))
            .await?
            .into_iter()
            .flatten()
            .collect(),
        )
    }

    #[instrument(skip(self), fields(block_range = format!("{}..={}", start_height, end_height)), err(Debug))]
    async fn fetch_events(
        &self,
        start_height: u64,
        end_height: u64,
    ) -> Result<Vec<EurekaEventWithHeight>> {
        Ok(
            future::try_join_all((start_height..=end_height).map(|h| async move {
                let height: Height = h.try_into()?;
                let resp = self.client().block_results(height).await?;
                Ok::<_, anyhow::Error>(
                    resp.txs_results
                        .unwrap_or_default()
                        .into_iter()
                        .flat_map(|tx| tx.events)
                        .chain(resp.begin_block_events.unwrap_or_default())
                        .chain(resp.end_block_events.unwrap_or_default())
                        .chain(resp.finalize_block_events)
                        .filter_map(move |e| {
                            let event_type = EurekaEvent::try_from(e).ok()?;
                            Some(EurekaEventWithHeight {
                                event: event_type,
                                height: h,
                            })
                        }),
                )
            }))
            .await?
            .into_iter()
            .flatten()
            .collect(),
        )
    }
}
