// SPDX-License-Identifier: Apache-2.0

//! Attestation-based transaction builder for Eth-to-Solana relay.

use std::time::Duration;

use alloy::primitives::Address;
use anyhow::{Context, Result};
use ibc_proto_eureka::ibc::core::{
    channel::v2::{MsgAcknowledgement, MsgRecvPacket},
    client::v1::Height,
};
use proof_api_lib::{
    aggregator::{Aggregator, Config as AggregatorConfig},
    events::{solana::SolanaEurekaEvent, SolanaEurekaEventWithHeight},
    utils::{
        cosmos as cosmos_utils,
        solana::{
            self as solana_utils, ibc_to_solana_ack_packet, ibc_to_solana_recv_packet,
            target_events_to_timeout_msgs,
        },
        solana_attested, wait_for_condition,
    },
};
use solana_sdk::commitment_config::CommitmentConfig;

use proof_api_core::api::{self, SolanaPacketTxs};

use super::{RelayParams, SolanaTxBuilder};

/// Transaction builder using attestation proofs for Eth-to-Solana relay.
pub struct AttestedTxBuilder {
    aggregator: Aggregator,
    tx_builder: SolanaTxBuilder,
    ics26_eth_address: Address,
}

impl AttestedTxBuilder {
    /// Create a new [`AttestedTxBuilder`] instance.
    ///
    /// # Errors
    ///
    /// Returns an error if the aggregator cannot be created from the config.
    pub async fn new(
        aggregator_config: AggregatorConfig,
        tx_builder: SolanaTxBuilder,
        ics26_eth_address: Address,
    ) -> Result<Self> {
        let aggregator = Aggregator::from_config(aggregator_config).await?;
        Ok(Self {
            aggregator,
            tx_builder,
            ics26_eth_address,
        })
    }

    /// Get the inner `SolanaTxBuilder` reference.
    #[must_use]
    pub const fn tx_builder(&self) -> &SolanaTxBuilder {
        &self.tx_builder
    }

    /// Get the ICS26 Ethereum address.
    #[must_use]
    pub const fn ics26_eth_address(&self) -> &Address {
        &self.ics26_eth_address
    }

    /// Relay events from Ethereum to Solana using attestations.
    ///
    /// # Errors
    ///
    /// Returns an error if building relay transactions fails.
    #[allow(clippy::too_many_lines)]
    pub async fn relay_events(
        &self,
        params: RelayParams,
    ) -> Result<(Vec<SolanaPacketTxs>, Option<api::SolanaUpdateClient>)> {
        tracing::info!(
            "Building attested relay: {} src events, {} target events",
            params.src_events.len(),
            params.dest_events.len()
        );

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();

        let (mut recv_msgs, mut ack_msgs) = cosmos_utils::src_events_to_recv_and_ack_msgs(
            params.src_events.clone(),
            &params.src_client_id,
            &params.dst_client_id,
            &params.src_packet_seqs,
            &params.dst_packet_seqs,
            "",
            now,
        );

        // Get Solana slot for timeout packet handling
        let slot = self
            .tx_builder
            .target_solana_client
            .get_slot_with_commitment(CommitmentConfig::finalized())
            .map_err(|e| anyhow::anyhow!("Failed to get Solana slot: {e}"))?;

        // Extract max timeout timestamp before consuming dest_events
        let max_timeout_ts = max_timeout_timestamp(&params.dest_events);

        // Build timeout messages from destination events
        let mut timeout_msgs = target_events_to_timeout_msgs(
            params.dest_events,
            &params.src_client_id,
            &params.dst_client_id,
            &params.dst_packet_seqs,
            slot,
            now,
        );

        if recv_msgs.is_empty() && ack_msgs.is_empty() && timeout_msgs.is_empty() {
            tracing::info!("No packets to relay");
            return Ok((vec![], None));
        }

        // Resolve light client program ID for the destination client
        let light_client_program_id = self
            .tx_builder
            .resolve_client_program_id(&params.dst_client_id)?;

        // Get current attestation client state
        let client_state = self
            .tx_builder
            .attestation_client_state(light_client_program_id)?;
        let current_height = client_state.latest_height;

        let max_height = params
            .src_events
            .iter()
            .map(|e| e.height)
            .max()
            .unwrap_or(0);
        let required_height = max_height.saturating_add(1);

        let needs_height_update = current_height < required_height;

        // Check if timestamp update is needed for timeouts
        let consensus_ts = max_timeout_ts.and_then(|_| {
            self.tx_builder
                .attestation_consensus_state_timestamp_secs(current_height, light_client_program_id)
                .ok()
        });

        let needs_timestamp_update = matches!(
            (consensus_ts, max_timeout_ts),
            (Some(cs_ts), Some(timeout_ts)) if cs_ts < timeout_ts
        );

        let needs_update = needs_height_update || needs_timestamp_update;

        // For timeouts, use the source chain height (where non-membership is proven).
        // This ensures the consensus state timestamp is past the packet's timeout.
        let proof_height = if needs_update {
            let min_height = required_height.max(current_height.saturating_add(1));
            let target = params
                .timeout_relay_height
                .map_or(min_height, |h| h.max(min_height));

            tracing::info!(
                "Attestation client at height {}, waiting for aggregator to finalize height {}",
                current_height,
                target
            );

            wait_for_condition(Duration::from_mins(15), Duration::from_secs(1), || async {
                let finalized_height = self.aggregator.get_latest_height().await?;
                Ok(finalized_height >= target)
            })
            .await
            .context("Timeout waiting for aggregator to finalize height")?;

            tracing::info!("Aggregator has finalized height {}", target);
            target
        } else {
            current_height
        };

        let min_sigs = client_state.min_required_sigs as usize;

        let target_height = Height {
            revision_number: 0,
            revision_height: proof_height,
        };

        solana_attested::inject_solana_attestor_proofs(
            &self.aggregator,
            &mut recv_msgs,
            &mut ack_msgs,
            &mut timeout_msgs,
            &target_height,
            min_sigs,
        )
        .await?;

        // Build update_client transaction only if needed
        let update_client = if needs_update {
            let result = solana_attested::build_attestation_update_client_tx(
                &self.aggregator,
                &self.tx_builder,
                &params.dst_client_id,
                proof_height,
            )
            .await?;
            Some(api::SolanaUpdateClient {
                chunk_txs: vec![],
                alt_create_tx: vec![],
                alt_extend_txs: vec![],
                assembly_tx: result.assembly_tx,
                target_height: result.target_height,
                cleanup_tx: vec![],
            })
        } else {
            None
        };

        let packets = self.build_packet_transactions(recv_msgs, ack_msgs, timeout_msgs)?;

        Ok((packets, update_client))
    }

    /// Update the attestation light client to the latest height.
    ///
    /// # Errors
    ///
    /// Returns an error if the attestation update transaction cannot be built.
    pub async fn update_client(&self, dst_client_id: &str) -> Result<api::SolanaUpdateClient> {
        let result = solana_attested::update_attestation_client_tx(
            &self.aggregator,
            &self.tx_builder,
            dst_client_id,
        )
        .await?;
        Ok(api::SolanaUpdateClient {
            chunk_txs: vec![],
            alt_create_tx: vec![],
            alt_extend_txs: vec![],
            assembly_tx: result.assembly_tx,
            target_height: result.target_height,
            cleanup_tx: vec![],
        })
    }

    fn build_packet_transactions(
        &self,
        recv_msgs: Vec<MsgRecvPacket>,
        ack_msgs: Vec<MsgAcknowledgement>,
        timeout_msgs: Vec<solana_utils::TimeoutPacketWithChunks>,
    ) -> Result<Vec<SolanaPacketTxs>> {
        let mut results = Vec::new();

        for msg in recv_msgs {
            let recv_with_chunks = ibc_to_solana_recv_packet(msg)?;
            let packet_txs = self.tx_builder.build_recv_packet_chunked(
                &recv_with_chunks.msg,
                &recv_with_chunks.payload_chunks,
                &recv_with_chunks.proof_chunks,
            )?;

            results.push(packet_txs);
        }

        for msg in ack_msgs {
            let ack_with_chunks = ibc_to_solana_ack_packet(msg)?;
            let packet_txs = self.tx_builder.build_ack_packet_chunked(
                &ack_with_chunks.msg,
                &ack_with_chunks.payload_chunks,
                &ack_with_chunks.proof_chunks,
            )?;
            results.push(packet_txs);
        }

        for timeout in timeout_msgs {
            tracing::info!(
                "Building timeout packet for sequence {}",
                timeout.msg.packet.sequence
            );
            let packet_txs = self.tx_builder.build_timeout_packet_chunked(
                &timeout.msg,
                &timeout.payload_chunks,
                &timeout.proof_chunks,
            )?;
            results.push(packet_txs);
        }

        Ok(results)
    }
}

/// Extract maximum timeout timestamp from Solana destination events.
///
/// Looks at `SendPacket` events to find the maximum timeout timestamp,
/// which indicates the latest timeout we may need to prove against.
fn max_timeout_timestamp(events: &[SolanaEurekaEventWithHeight]) -> Option<u64> {
    events
        .iter()
        .filter_map(|e| match &e.event {
            SolanaEurekaEvent::SendPacket(send) => Some(send.timeout_timestamp),
            SolanaEurekaEvent::WriteAcknowledgement(_) => None,
        })
        .max()
}
