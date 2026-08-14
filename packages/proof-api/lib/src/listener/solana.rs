// SPDX-License-Identifier: Apache-2.0

//! Solana chain listener implementation for IBC Eureka.

use anyhow::{Context, Result};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Signature};
use solana_transaction_status::UiTransactionEncoding;

use std::sync::Arc;

use crate::{
    chain::SolanaEureka,
    events::solana::{parse_events_from_logs, SolanaEurekaEventWithHeight},
    listener::ChainListenerService,
};
use futures::stream::{self, StreamExt, TryStreamExt};

/// The `ChainListener` listens for events on the Solana chain.
pub struct ChainListener {
    /// Client
    rpc_client: Arc<RpcClient>,
    /// The Solana ICS26 router program ID.
    ics26_program_id: Pubkey,
}

impl ChainListener {
    /// Create a new [`Self`] instance.
    ///
    /// # Arguments
    /// - `rpc_url` - The Solana RPC endpoint URL
    /// - `ics26_router_program_id` - The ICS26 Router program ID on Solana
    #[must_use]
    pub fn new(rpc_url: String, ics26_router_program_id: Pubkey) -> Self {
        let rpc_client = Arc::new(RpcClient::new_with_commitment(
            rpc_url,
            CommitmentConfig::confirmed(),
        ));

        Self {
            rpc_client,
            ics26_program_id: ics26_router_program_id,
        }
    }

    /// Get the RPC client.
    #[must_use]
    pub const fn client(&self) -> &Arc<RpcClient> {
        &self.rpc_client
    }

    /// Get Router program id
    #[must_use]
    pub const fn ics26_program_id(&self) -> &Pubkey {
        &self.ics26_program_id
    }

    /// Get the current slot number using the client's default commitment.
    ///
    /// # Errors
    /// Returns an error if the slot cannot be fetched.
    pub fn get_slot(&self) -> Result<u64> {
        self.rpc_client
            .get_slot()
            .context("Failed to get current Solana slot")
    }

    /// Get the latest finalized slot number.
    ///
    /// # Errors
    /// Returns an error if the slot cannot be fetched.
    pub fn get_finalized_slot(&self) -> Result<u64> {
        self.rpc_client
            .get_slot_with_commitment(CommitmentConfig::finalized())
            .context("Failed to get finalized Solana slot")
    }

    /// Get the latest finalized slot and its block time.
    ///
    /// # Errors
    /// Returns an error if the slot or block time cannot be fetched.
    pub fn get_finalized_slot_with_time(&self) -> Result<(u64, u64)> {
        let slot = self.get_finalized_slot()?;
        let block_time = self
            .rpc_client
            .get_block_time(slot)
            .context("Failed to get block time for finalized slot")?;
        let block_time = u64::try_from(block_time).context("Finalized block time is negative")?;
        Ok((slot, block_time))
    }

    /// Parse IBC events from Solana transaction logs.
    fn parse_events_from_logs(
        meta: &solana_transaction_status::UiTransactionStatusMeta,
        height: u64,
    ) -> anyhow::Result<Vec<SolanaEurekaEventWithHeight>> {
        let empty_logs = vec![];
        let logs = meta.log_messages.as_ref().unwrap_or(&empty_logs);
        let parsed_events = parse_events_from_logs(logs).context(format!(
            "Failed to parse Solana events from height: {height}",
        ))?;

        Ok(parsed_events
            .into_iter()
            .map(|event| SolanaEurekaEventWithHeight { event, height })
            .collect())
    }
}

#[async_trait::async_trait]
impl ChainListenerService<SolanaEureka> for ChainListener {
    async fn fetch_tx_events(
        &self,
        tx_ids: Vec<Signature>,
    ) -> Result<Vec<SolanaEurekaEventWithHeight>> {
        let mut events = Vec::new();

        for tx in tx_ids {
            let (tx, meta) = self
                .rpc_client
                .get_transaction_with_config(
                    &tx,
                    solana_client::rpc_config::RpcTransactionConfig {
                        encoding: Some(UiTransactionEncoding::Json),
                        commitment: Some(CommitmentConfig::confirmed()),
                        max_supported_transaction_version: Some(0),
                    },
                )
                .map_err(|e| anyhow::anyhow!("Failed to fetch Solana transaction: {e}"))
                .and_then(|tx| {
                    tx.transaction
                        .meta
                        .clone()
                        .ok_or_else(|| anyhow::anyhow!("Transaction metadata not found"))
                        .and_then(|meta| match meta.err {
                            Some(err) => Err(anyhow::anyhow!("Transaction error {err}")),
                            None => Ok((tx, meta)),
                        })
                })?;

            let tx_events = Self::parse_events_from_logs(&meta, tx.slot)?;
            events.extend(tx_events);
        }

        Ok(events)
    }

    async fn fetch_events(
        &self,
        start_height: u64,
        end_height: u64,
    ) -> Result<Vec<SolanaEurekaEventWithHeight>> {
        const CONCURRENT_REQUESTS: usize = 10;

        let events = stream::iter(start_height..=end_height)
            .map(|slot| async move {
                let block = match self.rpc_client.get_block_with_config(
                    slot,
                    solana_client::rpc_config::RpcBlockConfig {
                        encoding: Some(UiTransactionEncoding::Base64),
                        max_supported_transaction_version: Some(0),
                        rewards: Some(false),
                        commitment: Some(CommitmentConfig::confirmed()),
                        transaction_details: Some(
                            solana_transaction_status::TransactionDetails::Full,
                        ),
                    },
                ) {
                    Ok(block) => block,
                    Err(e) => {
                        tracing::debug!("Skipping slot {}: {}", slot, e);
                        return Ok(Vec::new());
                    }
                };

                let mut slot_events = Vec::new();

                if let Some(transactions) = block.transactions {
                    for tx in transactions {
                        let Some(meta) = tx.meta else {
                            continue;
                        };

                        let solana_transaction_status::option_serializer::OptionSerializer::Some(
                            logs,
                        ) = &meta.log_messages
                        else {
                            continue;
                        };

                        let involves_ibc = logs
                            .iter()
                            .any(|log| log.contains(&self.ics26_program_id.to_string()));

                        if involves_ibc {
                            let parsed_events = Self::parse_events_from_logs(&meta, slot)?;
                            slot_events.extend(parsed_events);
                        }
                    }
                }

                Ok::<Vec<SolanaEurekaEventWithHeight>, anyhow::Error>(slot_events)
            })
            .buffer_unordered(CONCURRENT_REQUESTS)
            .try_fold(Vec::new(), |mut acc, events| async move {
                acc.extend(events);
                Ok(acc)
            })
            .await?;

        Ok(events)
    }
}
