// SPDX-License-Identifier: Apache-2.0

//! This module defines [`TxBuilder`] which is responsible for building transactions to be sent to
//! Solana from events received from a Cosmos SDK chain.

mod chunking;
mod client;
mod packets;
mod transaction;

use std::{collections::HashMap, sync::Arc, time::Duration};

use anchor_lang::prelude::*;
use anyhow::{Context, Result};
use ibc_client_tendermint::types::Header as TmHeader;
use ibc_proto_eureka::ibc::core::{
    channel::v2::{MsgAcknowledgement, MsgRecvPacket},
    client::v1::Height,
};
use ibc_proto_eureka::ibc::lightclients::tendermint::v1::Fraction;
use proof_api_lib::utils::solana::convert_client_state_to_sol;
use proof_api_lib::{
    aggregator::{Aggregator, Config as AggregatorConfig},
    events::{
        solana::solana_timeout_packet_to_tm_timeout, EurekaEventWithHeight, SolanaEurekaEvent,
        SolanaEurekaEventWithHeight,
    },
    utils::{
        cosmos::{
            self as cosmos_utils, tm_create_client_params, tm_update_client_params,
            TmCreateClientParams, TmUpdateClientParams,
        },
        solana::{
            self as solana_utils, convert_consensus_state, ibc_to_solana_ack_packet,
            ibc_to_solana_recv_packet, target_events_to_timeout_msgs,
        },
        solana_attested, wait_for_condition,
    },
};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey};

use crate::constants::ANCHOR_DISCRIMINATOR_SIZE;
use proof_api_core::api::{self, SolanaPacketTxs};

use solana_ibc_sdk::ics07_tendermint::{
    instructions::{self as ics07_tendermint_instructions, AssembleAndUpdateClient},
    types::ConsensusState,
};
use solana_ibc_sdk::ics26_router::types::Delivery;
use solana_ibc_sdk::pda::ics07_tendermint::{header_chunk_pda, sig_verify_pda};

/// Parameters for relaying events between Cosmos and Solana
pub struct RelayParams {
    /// Events from the source chain (Cosmos)
    pub src_events: Vec<EurekaEventWithHeight>,
    /// Events from the destination chain (Solana)
    pub dest_events: Vec<SolanaEurekaEventWithHeight>,
    /// Client ID on the source chain
    pub src_client_id: String,
    /// Client ID on the destination chain
    pub dst_client_id: String,
    /// Packet sequences from the source chain
    pub src_packet_seqs: Vec<u64>,
    /// Packet sequences from the destination chain
    pub dst_packet_seqs: Vec<u64>,
    /// For timeout packets, the current height from the source chain where
    /// non-membership needs to be proven. Required when processing timeouts.
    pub timeout_relay_height: Option<u64>,
}

/// Maximum accounts that fit in a Solana transaction without ALT
const MAX_ACCOUNTS_WITHOUT_ALT: usize = 20;

/// Nanoseconds per second for timestamp conversion
const NANOS_PER_SECOND: u64 = 1_000_000_000;

/// The `TxBuilder` produces Solana transactions based on events from Cosmos SDK.
///
/// This builder handles ICS07 Tendermint light client transactions.
/// For attestation mode, use `AttestedTxBuilder` which wraps this builder.
pub struct TxBuilder {
    /// The HTTP client for Cosmos chain.
    pub src_tm_client: tendermint_rpc::HttpClient,
    /// The target Rpc Client for Solana.
    pub target_solana_client: Arc<RpcClient>,
    /// The Solana ICS26 router program ID.
    pub solana_ics26_program_id: Pubkey,
    /// The fee payer address for transactions.
    pub fee_payer: Pubkey,
    /// Address Lookup Table address for reducing transaction size (optional).
    pub alt_address: Option<Pubkey>,
    /// Signature threshold for skipping pre-verification.
    /// None = always use pre-verification, Some(n) = skip when signatures ≤ n.
    pub skip_pre_verify_threshold: Option<usize>,
    /// Whitelisted IFT program IDs. Only IFT instructions targeting these
    /// programs are relayed (defense-in-depth against untrusted GMP sender
    /// fields).
    pub ift_program_ids: Vec<Pubkey>,
}

impl TxBuilder {
    /// Creates a new `TxBuilder`.
    ///
    /// # Errors
    /// This function cannot currently fail but returns `Result` for API consistency.
    pub const fn new(
        src_tm_client: tendermint_rpc::HttpClient,
        target_solana_client: Arc<RpcClient>,
        solana_ics26_program_id: Pubkey,
        fee_payer: Pubkey,
        alt_address: Option<Pubkey>,
        skip_pre_verify_threshold: Option<usize>,
        ift_program_ids: Vec<Pubkey>,
    ) -> Result<Self> {
        Ok(Self {
            src_tm_client,
            target_solana_client,
            solana_ics26_program_id,
            fee_payer,
            alt_address,
            skip_pre_verify_threshold,
            ift_program_ids,
        })
    }

    /// Create a new ICS07 Tendermint client on Solana
    ///
    /// # Errors
    /// Returns an error if parameters are invalid or Solana/Tendermint calls fail.
    pub async fn create_client(&self, parameters: &HashMap<String, String>) -> Result<Vec<u8>> {
        let trust_level = parameters
            .get("trust_level")
            .map(|s| Self::parse_trust_level(s))
            .transpose()?;

        let TmCreateClientParams {
            latest_height,
            client_state: tm_client_state,
            consensus_state: tm_consensus_state,
        } = tm_create_client_params(&self.src_tm_client, trust_level).await?;

        let access_manager_program_id = self.resolve_access_manager_program_id()?;
        let client_state = convert_client_state_to_sol(tm_client_state)?;
        let consensus_state = convert_consensus_state(&tm_consensus_state)?;

        let authority = self
            .fetch_ics07_upgrade_authority()
            .context("Failed to fetch ICS07 upgrade authority")?;

        let instruction = self.build_create_client_instruction(
            latest_height,
            &client_state,
            &consensus_state,
            access_manager_program_id,
            authority,
        );

        self.create_tx_bytes(&[instruction])
    }

    /// Build chunked update client transactions to latest tendermint height
    ///
    /// # Errors
    /// Returns an error if Solana/Tendermint calls fail or transaction building fails.
    #[allow(clippy::too_many_lines)]
    pub async fn update_client(&self, dst_client_id: &str) -> Result<api::SolanaUpdateClient> {
        const ALT_EXTEND_BATCH_SIZE: usize = 20;

        let solana_ics07_program_id = self.resolve_client_program_id(dst_client_id)?;
        let chain_id = self.chain_id().await?;
        let client_state = self.cosmos_client_state(solana_ics07_program_id)?;

        let TmUpdateClientParams {
            target_height,
            trusted_height,
            proposed_header,
        } = tm_update_client_params(
            client_state.latest_height.revision_height,
            &self.src_tm_client,
            None,
        )
        .await?;

        tracing::debug!(
            "Building update client: {} → height {}",
            dst_client_id,
            target_height
        );

        let header = TmHeader::try_from(proposed_header)
            .context("Failed to convert protobuf Header to ibc-rs Header")?;

        let signature_data = Self::select_minimal_signatures(
            &Self::extract_signature_data_from_header(&header, &chain_id)?,
            &header,
            client_state.trust_level_numerator,
            client_state.trust_level_denominator,
        )?;

        let borsh_header = crate::borsh_conversions::header_to_borsh(header);
        let header_bytes = borsh_header.try_to_vec()?;
        let chunks = Self::split_into_chunks(&header_bytes);
        let total_chunks = u8::try_from(chunks.len())
            .map_err(|_| anyhow::anyhow!("Too many chunks: {} should fit u8", chunks.len()))?;

        // Check if we can use the optimized path (skip pre-verify and ALT)
        // Accounts = static accounts + chunk PDAs (no signature PDAs in optimized path)
        let optimized_accounts = AssembleAndUpdateClient::COUNT + total_chunks as usize;
        let can_skip_alt = optimized_accounts <= MAX_ACCOUNTS_WITHOUT_ALT;

        let use_optimized_path = self
            .skip_pre_verify_threshold
            .is_some_and(|threshold| signature_data.len() <= threshold && can_skip_alt);

        if use_optimized_path {
            tracing::debug!(
                "Optimized path: {} sigs, {} accounts (no ALT)",
                signature_data.len(),
                optimized_accounts
            );

            let chunk_txs =
                self.build_chunk_transactions(&chunks, target_height, solana_ics07_program_id)?;

            let assembly_tx = self.build_assemble_and_update_client_tx(
                target_height,
                trusted_height,
                total_chunks,
                &[],
                None,
                solana_ics07_program_id,
            )?;

            let cleanup_tx =
                self.build_cleanup_tx(target_height, total_chunks, &[], solana_ics07_program_id)?;

            return Ok(api::SolanaUpdateClient {
                chunk_txs,
                alt_create_tx: vec![],
                alt_extend_txs: vec![],
                assembly_tx,
                target_height,
                cleanup_tx,
            });
        }

        tracing::debug!(
            "Full path: {} sigs, using pre-verify + ALT",
            signature_data.len()
        );

        let mut prep_txs: Vec<Vec<u8>> = signature_data
            .iter()
            .map(|sig_data| {
                self.build_pre_verify_signature_transaction(sig_data, solana_ics07_program_id)
            })
            .collect::<Result<Vec<_>>>()?;

        prep_txs.extend(self.build_chunk_transactions(
            &chunks,
            target_height,
            solana_ics07_program_id,
        )?);

        let slot = self
            .target_solana_client
            .get_slot_with_commitment(CommitmentConfig::processed())?;

        let alt_create_tx = self.build_create_alt_tx(slot)?;

        let (client_state_pda, _) =
            ics07_tendermint_instructions::Initialize::client_state_account_pda(
                &solana_ics07_program_id,
            );
        let (trusted_consensus_state, _) =
            ics07_tendermint_instructions::VerifyMembership::consensus_state_at_height_pda(
                trusted_height,
                &solana_ics07_program_id,
            );
        let (new_consensus_state, _) =
            ics07_tendermint_instructions::VerifyMembership::consensus_state_at_height_pda(
                target_height,
                &solana_ics07_program_id,
            );

        let mut alt_accounts = vec![
            client_state_pda,
            trusted_consensus_state,
            new_consensus_state,
            self.fee_payer,
            solana_sdk::system_program::id(),
        ];

        alt_accounts.extend((0..total_chunks).map(|chunk_index| {
            header_chunk_pda(
                &self.fee_payer,
                target_height,
                chunk_index,
                &solana_ics07_program_id,
            )
            .0
        }));

        alt_accounts.extend(
            signature_data.iter().map(|sig_data| {
                sig_verify_pda(&sig_data.signature_hash, &solana_ics07_program_id).0
            }),
        );

        let alt_extend_txs: Vec<Vec<u8>> = alt_accounts
            .chunks(ALT_EXTEND_BATCH_SIZE)
            .map(|batch| self.build_extend_alt_tx(slot, batch.to_vec()))
            .collect::<Result<Vec<_>>>()?;

        let assembly_tx = self.build_assemble_and_update_client_tx(
            target_height,
            trusted_height,
            total_chunks,
            &signature_data,
            Some((slot, alt_accounts)),
            solana_ics07_program_id,
        )?;

        let cleanup_tx = self.build_cleanup_tx(
            target_height,
            total_chunks,
            &signature_data,
            solana_ics07_program_id,
        )?;

        Ok(api::SolanaUpdateClient {
            chunk_txs: prep_txs,
            alt_create_tx,
            alt_extend_txs,
            assembly_tx,
            target_height,
            cleanup_tx,
        })
    }

    /// Build relay transaction from Cosmos events to Solana
    ///
    /// # Errors
    /// Returns an error if proof generation or transaction building fails.
    pub async fn relay_events_chunked(&self, params: RelayParams) -> Result<Vec<SolanaPacketTxs>> {
        self.relay_events_chunked_internal(params, None).await
    }

    /// Internal relay implementation with optional proof height override
    async fn relay_events_chunked_internal(
        &self,
        params: RelayParams,
        proof_height_override: Option<u64>,
    ) -> Result<Vec<SolanaPacketTxs>> {
        let RelayParams {
            src_events,
            dest_events,
            src_client_id,
            dst_client_id,
            src_packet_seqs,
            dst_packet_seqs,
            timeout_relay_height: _,
        } = params;

        let solana_ics07_program_id = self.resolve_client_program_id(&dst_client_id)?;
        let client_state = self.cosmos_client_state(solana_ics07_program_id)?;

        let proof_height = proof_height_override.map_or_else(
            || {
                Self::validate_height_and_get_proof_params(
                    &src_events,
                    client_state.latest_height.revision_height,
                    client_state.latest_height.revision_number,
                )
            },
            |h| {
                Ok(ibc_proto_eureka::ibc::core::client::v1::Height {
                    revision_number: client_state.latest_height.revision_number,
                    revision_height: h,
                })
            },
        )?;

        let now_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();

        let slot = self
            .target_solana_client
            .get_slot_with_commitment(CommitmentConfig::finalized())
            .map_err(|e| anyhow::anyhow!("Failed to get Solana slot: {e}"))?;

        let timeout_msgs = target_events_to_timeout_msgs(
            dest_events,
            &src_client_id,
            &dst_client_id,
            &dst_packet_seqs,
            slot,
            now_secs,
        );

        let (mut recv_msgs, mut ack_msgs) = cosmos_utils::src_events_to_recv_and_ack_msgs(
            src_events,
            &src_client_id,
            &dst_client_id,
            &src_packet_seqs,
            &dst_packet_seqs,
            "",
            now_secs,
        );

        let mut timeout_msgs_tm: Vec<_> = timeout_msgs
            .iter()
            .map(|t| solana_timeout_packet_to_tm_timeout(t.msg.clone(), String::new()))
            .collect::<Result<Vec<_>, _>>()?;

        cosmos_utils::inject_tendermint_proofs(
            &mut recv_msgs,
            &mut ack_msgs,
            &mut timeout_msgs_tm,
            &self.src_tm_client,
            &proof_height,
        )
        .await?;

        let mut timeout_msgs_with_chunks = timeout_msgs;
        for (timeout_with_chunks, tm_msg) in
            timeout_msgs_with_chunks.iter_mut().zip(&timeout_msgs_tm)
        {
            Self::update_timeout_proof_chunks(timeout_with_chunks, tm_msg);
        }

        let mut packet_txs = Vec::new();

        for recv_msg in recv_msgs {
            let r = ibc_to_solana_recv_packet(recv_msg)?;
            packet_txs.push(self.build_recv_packet_chunked(
                &r.msg,
                &r.payload_chunks,
                &r.proof_chunks,
            )?);
        }

        for ack_msg in ack_msgs {
            let a = ibc_to_solana_ack_packet(ack_msg)?;
            packet_txs.push(self.build_ack_packet_chunked(
                &a.msg,
                &a.payload_chunks,
                &a.proof_chunks,
            )?);
        }

        for t in timeout_msgs_with_chunks {
            packet_txs.push(self.build_timeout_packet_chunked(
                &t.msg,
                &t.payload_chunks,
                &t.proof_chunks,
            )?);
        }

        Ok(packet_txs)
    }

    /// Build relay transactions with optional update client (auto-generated when needed)
    ///
    /// # Errors
    /// Returns an error if update client generation or relay building fails.
    pub async fn relay_events_with_update(
        &self,
        params: RelayParams,
    ) -> Result<(Vec<SolanaPacketTxs>, Option<api::SolanaUpdateClient>)> {
        let solana_ics07_program_id = self.resolve_client_program_id(&params.dst_client_id)?;
        let client_state = self.cosmos_client_state(solana_ics07_program_id)?;
        let current_height = client_state.latest_height.revision_height;

        let max_timeout_ts = Self::max_timeout_timestamp(&params.dest_events);
        let consensus_ts = max_timeout_ts.and_then(|_| {
            self.get_consensus_state_timestamp_secs(current_height, solana_ics07_program_id)
                .ok()
        });

        // Build update_client transaction if needed
        let update_client = match Self::needs_update_client(
            &params.src_events,
            current_height,
            consensus_ts,
            max_timeout_ts,
        ) {
            Some(_) => Some(self.update_client(&params.dst_client_id).await?),
            Option::None => None,
        };

        let proof_height = update_client
            .as_ref()
            .map_or(current_height, |u| u.target_height);

        let packets = self
            .relay_events_chunked_internal(params, Some(proof_height))
            .await?;

        Ok((packets, update_client))
    }

    // Relay helper functions
    fn max_timeout_timestamp(dest_events: &[SolanaEurekaEventWithHeight]) -> Option<u64> {
        dest_events
            .iter()
            .filter_map(|e| match &e.event {
                SolanaEurekaEvent::SendPacket(event) => Some(event.timeout_timestamp),
                SolanaEurekaEvent::WriteAcknowledgement(_) => None,
            })
            .max()
    }

    fn get_consensus_state_timestamp_secs(
        &self,
        height: u64,
        solana_ics07_program_id: Pubkey,
    ) -> Result<u64> {
        let (consensus_state_pda, _) =
            ics07_tendermint_instructions::VerifyMembership::consensus_state_at_height_pda(
                height,
                &solana_ics07_program_id,
            );

        let account = self
            .target_solana_client
            .get_account_with_commitment(&consensus_state_pda, CommitmentConfig::confirmed())
            .context("Failed to fetch consensus state account")?
            .value
            .ok_or_else(|| anyhow::anyhow!("Consensus state account not found"))?;

        let mut data = &account.data[ANCHOR_DISCRIMINATOR_SIZE..];
        let consensus_state = ConsensusState::deserialize(&mut data)
            .context("Failed to deserialize consensus state")?;

        Ok(consensus_state.timestamp / NANOS_PER_SECOND)
    }

    fn needs_update_client(
        src_events: &[EurekaEventWithHeight],
        current_height: u64,
        current_consensus_timestamp_secs: Option<u64>,
        max_timeout_ts: Option<u64>,
    ) -> Option<u64> {
        let max_event_height = src_events
            .iter()
            .map(|e| e.height)
            .max()
            .unwrap_or_else(|| current_height.saturating_sub(1));

        let required_height = max_event_height + 1;

        // Need update if height is insufficient for proofs
        let needs_height_update = current_height < required_height;

        // Need update if timestamp is insufficient for timeouts
        let needs_timestamp_update = matches!(
            (current_consensus_timestamp_secs, max_timeout_ts),
            (Some(consensus_ts), Some(timeout_ts)) if consensus_ts < timeout_ts
        );

        if needs_timestamp_update {
            tracing::debug!("Client update needed for timeout");
        }

        if needs_height_update || needs_timestamp_update {
            Some(required_height.max(current_height + 1))
        } else {
            None
        }
    }

    fn validate_height_and_get_proof_params(
        src_events: &[EurekaEventWithHeight],
        solana_latest_height: u64,
        solana_revision_number: u64,
    ) -> Result<ibc_proto_eureka::ibc::core::client::v1::Height> {
        let max_event_height = src_events
            .iter()
            .map(|e| e.height)
            .max()
            .unwrap_or_else(|| solana_latest_height.saturating_sub(1));

        let required_height = max_event_height + 1;

        if solana_latest_height < required_height {
            anyhow::bail!(
                "Solana client is at height {solana_latest_height} but need height {required_height} to prove events at height {max_event_height}. Update Solana client to at least height {required_height} first!",
            );
        }

        Ok(ibc_proto_eureka::ibc::core::client::v1::Height {
            revision_number: solana_revision_number,
            revision_height: solana_latest_height,
        })
    }

    #[allow(clippy::cast_possible_truncation)]
    fn update_timeout_proof_chunks(
        timeout_with_chunks: &mut proof_api_lib::utils::solana::TimeoutPacketWithChunks,
        tm_msg: &ibc_proto_eureka::ibc::core::channel::v2::MsgTimeout,
    ) {
        use solana_ibc_constants::CHUNK_DATA_SIZE;

        let proof_bytes = &tm_msg.proof_unreceived;
        let proof_total_chunks = proof_bytes.len().div_ceil(CHUNK_DATA_SIZE) as u8;

        timeout_with_chunks.msg.proof.height = tm_msg
            .proof_height
            .as_ref()
            .map_or(0, |h| h.revision_height);
        timeout_with_chunks.msg.proof.data = Delivery::Chunked {
            total_chunks: proof_total_chunks,
        };
        timeout_with_chunks.proof_chunks.clone_from(proof_bytes);
    }

    fn parse_trust_level(trust_level_str: &str) -> Result<Fraction> {
        let parts: Vec<&str> = trust_level_str.split('/').collect();
        let [num_str, denom_str] = parts.as_slice() else {
            anyhow::bail!(
                "Invalid trust level format: expected 'numerator/denominator', got '{trust_level_str}'"
            );
        };

        let numerator = num_str
            .parse::<u64>()
            .map_err(|_| anyhow::anyhow!("Invalid trust level numerator: {num_str}"))?;
        let denominator = denom_str
            .parse::<u64>()
            .map_err(|_| anyhow::anyhow!("Invalid trust level denominator: {denom_str}"))?;

        if numerator == 0 || denominator == 0 {
            anyhow::bail!("Trust level numerator and denominator must be greater than 0");
        }
        if numerator >= denominator {
            anyhow::bail!("Trust level numerator must be less than denominator");
        }

        Ok(Fraction {
            numerator,
            denominator,
        })
    }
}

/// Transaction builder for attested relay from Cosmos to Solana.
///
/// Uses attestor signatures for state verification instead of Tendermint proofs.
pub struct AttestedTxBuilder {
    aggregator: Aggregator,
    tx_builder: TxBuilder,
}

impl AttestedTxBuilder {
    /// Create a new [`AttestedTxBuilder`] instance.
    ///
    /// # Errors
    /// Returns an error if the aggregator cannot be created from the config.
    pub async fn new(aggregator_config: AggregatorConfig, tx_builder: TxBuilder) -> Result<Self> {
        let aggregator = Aggregator::from_config(aggregator_config).await?;
        Ok(Self {
            aggregator,
            tx_builder,
        })
    }

    /// Get the inner `TxBuilder` reference.
    #[must_use]
    pub const fn tx_builder(&self) -> &TxBuilder {
        &self.tx_builder
    }

    /// Relay events from Cosmos to Solana using attestations.
    ///
    /// Returns packet transactions and an `update_client` transaction to create the consensus state.
    ///
    /// # Errors
    /// Returns an error if attestation retrieval or transaction building fails.
    #[allow(clippy::too_many_lines)]
    pub async fn relay_events(
        &self,
        params: RelayParams,
    ) -> Result<(Vec<SolanaPacketTxs>, Option<api::SolanaUpdateClient>)> {
        tracing::info!(
            "Building attested relay transaction for {} source events, {} target events",
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
        let max_timeout_ts = TxBuilder::max_timeout_timestamp(&params.dest_events);

        // Build timeout messages from destination events
        let mut timeout_msgs = solana_utils::target_events_to_timeout_msgs(
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

        // Get current attestation client state to check if update is needed
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

        // Check if height update is needed for proofs
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

        if needs_timestamp_update {
            tracing::info!(
                "Client update needed for timeout: consensus_ts={:?} < timeout_ts={:?}",
                consensus_ts,
                max_timeout_ts
            );
        }

        let needs_update = needs_height_update || needs_timestamp_update;

        // For timeouts, use the source chain height (where non-membership is proven).
        // This ensures the consensus state timestamp is past the packet's timeout.
        let proof_height = if needs_update {
            let min_height = required_height.max(current_height.saturating_add(1));
            let target = params
                .timeout_relay_height
                .map_or(min_height, |h| h.max(min_height));

            tracing::info!(
                "Attestation client at height {}, waiting for aggregator to finalize height {} \
                 (needs_height: {}, needs_timestamp: {})",
                current_height,
                target,
                needs_height_update,
                needs_timestamp_update
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
        tracing::info!("Attestation client requires {} signatures", min_sigs);

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
            tracing::info!(
                "Client already at sufficient height {}, skipping update",
                current_height
            );
            None
        };

        let packets = self.build_packet_transactions(recv_msgs, ack_msgs, timeout_msgs)?;

        Ok((packets, update_client))
    }

    /// Update the attestation light client to the latest height.
    ///
    /// # Errors
    /// Returns an error if fetching attestations or building transactions fails.
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
            let recv_with_chunks = solana_utils::ibc_to_solana_recv_packet(msg)?;
            let packet_txs = self.tx_builder.build_recv_packet_chunked(
                &recv_with_chunks.msg,
                &recv_with_chunks.payload_chunks,
                &recv_with_chunks.proof_chunks,
            )?;
            results.push(packet_txs);
        }

        for msg in ack_msgs {
            let ack_with_chunks = solana_utils::ibc_to_solana_ack_packet(msg)?;
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
