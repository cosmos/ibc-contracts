// SPDX-License-Identifier: Apache-2.0

//! Transaction building utilities for Solana.

use anchor_lang::prelude::*;
use anyhow::{Context, Result};
use solana_sdk::{commitment_config::CommitmentConfig, instruction::Instruction, pubkey::Pubkey};

use crate::constants::ANCHOR_DISCRIMINATOR_SIZE;
use proof_api_lib::utils::solana_v0_tx;
use solana_ibc_sdk::attestation::{
    accounts::{
        ClientState as AttestationClientState,
        ConsensusStateStore as AttestationConsensusStateStore,
    },
    instructions as attestation_instructions,
};
use solana_ibc_sdk::ics07_tendermint::{
    accounts::ClientState, instructions as ics07_tendermint_instructions,
};
use solana_ibc_sdk::ics26_router::{
    accounts::{IBCApp, RouterState},
    instructions as router_instructions,
    types::ClientAccount,
};

impl super::TxBuilder {
    /// Fetches the ICS07 Tendermint program's upgrade authority from its `ProgramData` account.
    pub(crate) fn fetch_ics07_upgrade_authority(&self) -> Result<Pubkey> {
        let solana_ics07_program_id: Pubkey = solana_ibc_constants::ICS07_TENDERMINT_ID
            .parse()
            .expect("Invalid ICS07_TENDERMINT_ID constant");

        let (program_data_pda, _) = Pubkey::find_program_address(
            &[solana_ics07_program_id.as_ref()],
            &solana_sdk::bpf_loader_upgradeable::id(),
        );

        let account = self
            .target_solana_client
            .get_account_with_commitment(&program_data_pda, CommitmentConfig::confirmed())
            .map_err(|e| anyhow::anyhow!("Failed to fetch ICS07 ProgramData account: {e}"))?
            .value
            .ok_or_else(|| anyhow::anyhow!("ICS07 ProgramData account not found"))?;

        let loader_state: solana_sdk::bpf_loader_upgradeable::UpgradeableLoaderState =
            bincode::deserialize(&account.data)
                .map_err(|e| anyhow::anyhow!("Failed to deserialize ProgramData: {e}"))?;

        match loader_state {
            solana_sdk::bpf_loader_upgradeable::UpgradeableLoaderState::ProgramData {
                upgrade_authority_address: Some(authority),
                ..
            } => Ok(authority),
            _ => Err(anyhow::anyhow!(
                "ICS07 program has no upgrade authority (immutable)"
            )),
        }
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
            return Err(anyhow::anyhow!(
                "Account data too short for RouterState account"
            ));
        }

        let mut data = &account.data[ANCHOR_DISCRIMINATOR_SIZE..];
        let router_state = RouterState::deserialize(&mut data)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize RouterState account: {e}"))?;

        Ok(router_state.am_state.access_manager)
    }

    /// Resolve the IBC app program ID for a given port
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
            return Err(anyhow::anyhow!("Account data too short for IBCApp account"));
        }

        let mut data = &account.data[ANCHOR_DISCRIMINATOR_SIZE..];
        let ibc_app = IBCApp::deserialize(&mut data)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize IBCApp account: {e}"))?;

        tracing::debug!("Resolved port '{}' → {}", port_id, ibc_app.app_program_id);

        Ok(ibc_app.app_program_id)
    }

    /// Resolve the light client program id.
    pub(crate) fn resolve_client_program_id(&self, client_id: &str) -> Result<Pubkey> {
        let (client_account, _) =
            router_instructions::SendPacket::client_pda(client_id, &self.solana_ics26_program_id);

        let account = self
            .target_solana_client
            .get_account_with_commitment(&client_account, CommitmentConfig::confirmed())
            .map_err(|e| {
                anyhow::anyhow!("Failed to fetch Client account for client '{client_id}': {e}")
            })?
            .value
            .ok_or_else(|| anyhow::anyhow!("Client account not found for client '{client_id}'"))?;

        if account.data.len() < ANCHOR_DISCRIMINATOR_SIZE {
            return Err(anyhow::anyhow!("Account data too short for Client account"));
        }

        // Deserialize Client account using borsh (skip discriminator)
        let mut data = &account.data[ANCHOR_DISCRIMINATOR_SIZE..];
        let client = ClientAccount::deserialize(&mut data)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize Client account: {e}"))?;

        tracing::debug!(
            "Resolved client '{}' → {}",
            client_id,
            client.client_program_id
        );

        Ok(client.client_program_id)
    }

    /// Get chain ID from Cosmos
    pub(crate) async fn chain_id(&self) -> Result<String> {
        use tendermint_rpc::Client as _;
        Ok(self
            .src_tm_client
            .latest_block()
            .await?
            .block
            .header
            .chain_id
            .into())
    }

    /// Fetch Cosmos client state from the light client on Solana.
    pub(crate) fn cosmos_client_state(
        &self,
        solana_ics07_program_id: Pubkey,
    ) -> Result<ClientState> {
        let (client_state_pda, _) =
            ics07_tendermint_instructions::Initialize::client_state_account_pda(
                &solana_ics07_program_id,
            );

        let account = self
            .target_solana_client
            .get_account_with_commitment(&client_state_pda, CommitmentConfig::confirmed())
            .context("Failed to fetch client state account")?
            .value
            .ok_or_else(|| anyhow::anyhow!("Client state account not found"))?;

        let mut data = &account.data[ANCHOR_DISCRIMINATOR_SIZE..];
        let client_state =
            ClientState::deserialize(&mut data).context("Failed to deserialize client state")?;

        Ok(client_state)
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

    /// Fetch the attestation consensus state timestamp at a given height (in seconds).
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
        let consensus_state = AttestationConsensusStateStore::deserialize(&mut data)
            .context("Failed to deserialize attestation consensus state")?;

        // Attestation consensus state stores timestamp in seconds (not nanoseconds)
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

    /// Check if an account exists on-chain
    /// Used to filter out signing-only PDAs from ALT (they don't exist as accounts)
    pub(crate) fn account_exists(&self, pubkey: &Pubkey) -> bool {
        self.target_solana_client
            .get_account_with_commitment(pubkey, CommitmentConfig::confirmed())
            .is_ok_and(|response| response.value.is_some())
    }
}

impl proof_api_lib::utils::solana_attested::SolanaAttestationTxBuilder for super::TxBuilder {
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
