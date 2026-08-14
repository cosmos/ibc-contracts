// SPDX-License-Identifier: Apache-2.0

//! Transaction building for Eth-to-Solana relay.
//!
//! Supports attestation-based light client mode.

mod attested;
mod packets;

use std::sync::Arc;

use anchor_lang::prelude::*;
use anyhow::{Context, Result};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{commitment_config::CommitmentConfig, instruction::Instruction, pubkey::Pubkey};

use crate::constants::ANCHOR_DISCRIMINATOR_SIZE;
use proof_api_lib::events::{EurekaEventWithHeight, SolanaEurekaEventWithHeight};
use proof_api_lib::utils::solana_v0_tx;
use solana_ibc_sdk::attestation::{
    accounts::{ClientState as AttestationClientState, ConsensusStateStore},
    instructions as attestation_instructions,
};
use solana_ibc_sdk::ics26_router::{
    accounts::{IBCApp, RouterState},
    instructions as router_instructions,
    types::ClientAccount,
};

pub use attested::AttestedTxBuilder;

/// Parameters for relaying events between Ethereum and Solana.
pub struct RelayParams {
    /// Events from the source chain (Ethereum)
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

/// Solana-side transaction builder.
///
/// Contains only the Solana RPC and program config needed for building
/// transactions, without any source-chain specific dependencies.
pub struct SolanaTxBuilder {
    /// The target Solana RPC client.
    pub target_solana_client: Arc<RpcClient>,
    /// The Solana ICS26 router program ID.
    pub solana_ics26_program_id: Pubkey,
    /// The fee payer address for transactions.
    pub fee_payer: Pubkey,
    /// Address Lookup Table address for reducing transaction size.
    pub alt_address: Option<Pubkey>,
    /// Whitelisted IFT program IDs. Only IFT instructions targeting these
    /// programs are relayed (defense-in-depth against untrusted GMP sender
    /// fields).
    pub ift_program_ids: Vec<Pubkey>,
}

impl SolanaTxBuilder {
    /// Creates a new `SolanaTxBuilder`.
    ///
    /// # Errors
    ///
    /// This function is infallible but returns `Result` for API consistency.
    #[allow(clippy::missing_const_for_fn)]
    pub fn new(
        target_solana_client: Arc<RpcClient>,
        solana_ics26_program_id: Pubkey,
        fee_payer: Pubkey,
        alt_address: Option<Pubkey>,
        ift_program_ids: Vec<Pubkey>,
    ) -> Result<Self> {
        Ok(Self {
            target_solana_client,
            solana_ics26_program_id,
            fee_payer,
            alt_address,
            ift_program_ids,
        })
    }

    /// Resolves the access manager program ID from the router state.
    pub(crate) fn resolve_access_manager_program_id(&self) -> Result<Pubkey> {
        let (router_state_pda, _) =
            router_instructions::Initialize::router_state_pda(&self.solana_ics26_program_id);

        let account = self
            .target_solana_client
            .get_account_with_commitment(&router_state_pda, CommitmentConfig::confirmed())
            .map_err(|e| anyhow::anyhow!("Failed to fetch RouterState account: {e}"))?
            .value
            .ok_or_else(|| anyhow::anyhow!("Router state account not found"))?;

        if account.data.len() < ANCHOR_DISCRIMINATOR_SIZE {
            return Err(anyhow::anyhow!("Account data too short for RouterState"));
        }

        let mut data = &account.data[ANCHOR_DISCRIMINATOR_SIZE..];
        let router_state = RouterState::deserialize(&mut data)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize RouterState: {e}"))?;

        Ok(router_state.am_state.access_manager)
    }

    /// Resolve the IBC app program ID for a given port.
    pub(crate) fn resolve_port_program_id(&self, port_id: &str) -> Result<Pubkey> {
        let (ibc_app_account, _) =
            router_instructions::AddIbcApp::ibc_app_pda(port_id, &self.solana_ics26_program_id);

        let account = self
            .target_solana_client
            .get_account_with_commitment(&ibc_app_account, CommitmentConfig::confirmed())
            .map_err(|e| {
                anyhow::anyhow!("Failed to fetch IBCApp account for port '{port_id}': {e}")
            })?
            .value
            .ok_or_else(|| anyhow::anyhow!("IBCApp account not found for port '{port_id}'"))?;

        if account.data.len() < ANCHOR_DISCRIMINATOR_SIZE {
            return Err(anyhow::anyhow!("Account data too short for IBCApp"));
        }

        let mut data = &account.data[ANCHOR_DISCRIMINATOR_SIZE..];
        let ibc_app = IBCApp::deserialize(&mut data)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize IBCApp: {e}"))?;

        Ok(ibc_app.app_program_id)
    }

    /// Resolve the light client program ID.
    pub(crate) fn resolve_client_program_id(&self, client_id: &str) -> Result<Pubkey> {
        let (client_account, _) =
            router_instructions::SendPacket::client_pda(client_id, &self.solana_ics26_program_id);

        let account = self
            .target_solana_client
            .get_account_with_commitment(&client_account, CommitmentConfig::confirmed())
            .map_err(|e| anyhow::anyhow!("Failed to fetch Client account for '{client_id}': {e}"))?
            .value
            .ok_or_else(|| anyhow::anyhow!("Client account not found for '{client_id}'"))?;

        if account.data.len() < ANCHOR_DISCRIMINATOR_SIZE {
            return Err(anyhow::anyhow!("Account data too short for Client"));
        }

        let mut data = &account.data[ANCHOR_DISCRIMINATOR_SIZE..];
        let client = ClientAccount::deserialize(&mut data)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize Client: {e}"))?;

        Ok(client.client_program_id)
    }

    /// Fetch the attestation light client state from Solana.
    pub(crate) fn attestation_client_state(
        &self,
        light_client_program_id: Pubkey,
    ) -> Result<AttestationClientState> {
        let (client_state_pda, _) =
            attestation_instructions::Initialize::client_state_pda(&light_client_program_id);

        let account = self
            .target_solana_client
            .get_account_with_commitment(&client_state_pda, CommitmentConfig::confirmed())
            .context("Failed to fetch attestation client state account")?
            .value
            .ok_or_else(|| anyhow::anyhow!("Attestation client state account not found"))?;

        let mut data = &account.data[ANCHOR_DISCRIMINATOR_SIZE..];
        let client_state = AttestationClientState::deserialize(&mut data)
            .context("Failed to deserialize attestation client state")?;

        Ok(client_state)
    }

    /// Fetch the attestation consensus state timestamp at a given height (seconds).
    pub(crate) fn attestation_consensus_state_timestamp_secs(
        &self,
        height: u64,
        light_client_program_id: Pubkey,
    ) -> Result<u64> {
        let (consensus_state_pda, _) =
            attestation_instructions::VerifyMembership::consensus_state_at_height_pda(
                height,
                &light_client_program_id,
            );

        let account = self
            .target_solana_client
            .get_account_with_commitment(&consensus_state_pda, CommitmentConfig::confirmed())
            .context("Failed to fetch attestation consensus state account")?
            .value
            .ok_or_else(|| anyhow::anyhow!("Attestation consensus state account not found"))?;

        let mut data = &account.data[ANCHOR_DISCRIMINATOR_SIZE..];
        let consensus_state = ConsensusStateStore::deserialize(&mut data)
            .context("Failed to deserialize attestation consensus state")?;

        Ok(consensus_state.timestamp)
    }

    /// Build a serialized v0 transaction from instructions, resolving the ALT if present.
    pub(crate) fn create_tx_bytes(&self, instructions: &[Instruction]) -> Result<Vec<u8>> {
        solana_v0_tx::create_tx_bytes(
            &self.target_solana_client,
            self.fee_payer,
            self.alt_address,
            instructions,
        )
    }

    /// Build a serialized v0 transaction using an explicit ALT address and its entries.
    pub(crate) fn create_tx_bytes_with_alt(
        &self,
        instructions: &[Instruction],
        alt_address: Pubkey,
        alt_addresses: Vec<Pubkey>,
    ) -> Result<Vec<u8>> {
        solana_v0_tx::create_tx_bytes_with_alt(
            &self.target_solana_client,
            self.fee_payer,
            instructions,
            alt_address,
            alt_addresses,
        )
    }

    /// Build a transaction that creates a new Address Lookup Table.
    pub(crate) fn build_create_alt_tx(&self, slot: u64) -> Result<Vec<u8>> {
        solana_v0_tx::build_create_alt_tx(
            &self.target_solana_client,
            self.fee_payer,
            self.alt_address,
            slot,
        )
    }

    /// Build a transaction that extends an Address Lookup Table with new accounts.
    pub(crate) fn build_extend_alt_tx(&self, slot: u64, accounts: Vec<Pubkey>) -> Result<Vec<u8>> {
        solana_v0_tx::build_extend_alt_tx(
            &self.target_solana_client,
            self.fee_payer,
            self.alt_address,
            slot,
            accounts,
        )
    }

    /// Create a client on Solana (stub for attestation mode).
    ///
    /// # Errors
    ///
    /// Always returns an error because this is not yet implemented.
    #[allow(clippy::unused_async)]
    pub async fn create_client(
        &self,
        _parameters: &std::collections::HashMap<String, String>,
    ) -> Result<Vec<u8>> {
        anyhow::bail!("create_client not yet implemented for eth-to-solana")
    }
}

impl proof_api_lib::utils::solana_attested::SolanaAttestationTxBuilder for SolanaTxBuilder {
    fn resolve_client_program_id(&self, client_id: &str) -> Result<Pubkey> {
        self.resolve_client_program_id(client_id)
    }

    fn attestation_client_state(&self, program_id: Pubkey) -> Result<AttestationClientState> {
        self.attestation_client_state(program_id)
    }

    fn resolve_access_manager_program_id(&self) -> Result<Pubkey> {
        self.resolve_access_manager_program_id()
    }

    fn fee_payer(&self) -> Pubkey {
        self.fee_payer
    }

    fn create_tx_bytes(&self, instructions: &[Instruction]) -> Result<Vec<u8>> {
        solana_v0_tx::create_tx_bytes(
            &self.target_solana_client,
            self.fee_payer,
            self.alt_address,
            instructions,
        )
    }
}
