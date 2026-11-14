// SPDX-License-Identifier: Apache-2.0

//! ICS07 Tendermint and Solana account queries.

use anyhow::{Context, Result};
use proof_api_lib::utils::solana::deserialize_anchor_account;
use solana_sdk::{commitment_config::CommitmentConfig, pubkey::Pubkey};

use solana_ibc_sdk::ics07_tendermint::{
    accounts::ClientState, instructions as ics07_tendermint_instructions,
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

        deserialize_anchor_account(&account.data, "client state")
    }

    /// Check if an account exists on-chain
    /// Used to filter out signing-only PDAs from ALT (they don't exist as accounts)
    pub(crate) fn account_exists(&self, pubkey: &Pubkey) -> bool {
        self.target_solana_client
            .get_account_with_commitment(pubkey, CommitmentConfig::confirmed())
            .is_ok_and(|response| response.value.is_some())
    }
}
