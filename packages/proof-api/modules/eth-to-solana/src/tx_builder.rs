// SPDX-License-Identifier: Apache-2.0

//! Transaction building for Eth-to-Solana relay.
//!
//! Supports attestation-based light client mode.

mod packets;

use std::sync::Arc;

use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;

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

    /// Create a client on Solana (stub for attestation mode).
    ///
    /// # Errors
    ///
    /// Always returns an error because this is not yet implemented.
    #[allow(clippy::unused_async, clippy::unused_async_trait_impl)]
    pub async fn create_client(
        &self,
        _parameters: &std::collections::HashMap<String, String>,
    ) -> Result<Vec<u8>> {
        anyhow::bail!("create_client not yet implemented for eth-to-solana")
    }
}
