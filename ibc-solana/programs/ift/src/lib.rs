// SPDX-License-Identifier: Apache-2.0

use anchor_lang::prelude::*;

pub mod constants;
pub mod errors;
pub mod events;
pub mod evm_selectors;
pub mod gmp_cpi;
pub mod helpers;
pub mod instructions;
pub mod state;

#[cfg(test)]
pub mod test_utils;

use instructions::*;
use state::{
    AdminMintMsg, CreateTokenParams, IFTMintMsg, IFTTransferMsg, RegisterIFTBridgeMsg,
    SetMintRateLimitMsg, SetPausedMsg,
};

declare_id!("DQU7WYvJTdpbLSzpLjHtCRF7wiaWe7thXwboafEN4kcy");

#[program]
pub mod ift {
    use super::*;

    /// Initialize global IFT state (once-only)
    pub fn initialize(ctx: Context<Initialize>, admin: Pubkey) -> Result<()> {
        instructions::initialize(ctx, admin)
    }

    /// Create and initialize a new SPL token mint for IFT.
    /// Pass `CreateTokenParams::SplToken` for legacy or `CreateTokenParams::Token2022`
    /// for a mint with `MetadataPointer` and on-chain metadata.
    pub fn create_and_initialize_spl_token(
        ctx: Context<CreateAndInitializeSplToken>,
        params: CreateTokenParams,
    ) -> Result<()> {
        instructions::create_and_initialize_spl_token(ctx, params)
    }

    /// Initialize IFT for an existing SPL token by transferring mint authority
    pub fn initialize_existing_token(ctx: Context<InitializeExistingToken>) -> Result<()> {
        instructions::initialize_existing_token(ctx)
    }

    /// Register an IFT bridge to a counterparty chain
    pub fn register_ift_bridge(
        ctx: Context<RegisterIFTBridge>,
        msg: RegisterIFTBridgeMsg,
    ) -> Result<()> {
        instructions::register_ift_bridge(ctx, msg)
    }

    /// Remove an IFT bridge
    pub fn remove_ift_bridge(ctx: Context<RemoveIFTBridge>, client_id: String) -> Result<()> {
        instructions::remove_ift_bridge(ctx, client_id)
    }

    /// Initiate an IFT transfer to another chain
    /// Burns tokens and sends GMP call to mint on destination
    pub fn ift_transfer(ctx: Context<IFTTransfer>, msg: IFTTransferMsg) -> Result<u64> {
        instructions::ift_transfer(ctx, msg)
    }

    /// Mint IFT tokens (called by GMP when receiving cross-chain transfer).
    pub fn ift_mint(ctx: Context<IFTMint>, msg: IFTMintMsg) -> Result<()> {
        instructions::ift_mint(ctx, msg)
    }

    /// Finalize a pending transfer after GMP result is recorded and proved ack/timeout.
    pub fn finalize_transfer(
        ctx: Context<FinalizeTransfer>,
        client_id: String,
        sequence: u64,
    ) -> Result<()> {
        instructions::finalize_transfer(ctx, client_id, sequence)
    }

    /// Propose a new admin (current admin only). The proposed admin must call
    /// `accept_admin` to finalize the transfer.
    pub fn propose_admin(ctx: Context<ProposeAdmin>, new_admin: Pubkey) -> Result<()> {
        instructions::propose_admin(ctx, new_admin)
    }

    /// Accept a pending admin proposal. Must be signed by the proposed admin.
    pub fn accept_admin(ctx: Context<AcceptAdmin>) -> Result<()> {
        instructions::accept_admin(ctx)
    }

    /// Cancel a pending admin proposal. Must be signed by the current admin.
    pub fn cancel_admin_proposal(ctx: Context<CancelAdminProposal>) -> Result<()> {
        instructions::cancel_admin_proposal(ctx)
    }

    /// Revoke mint authority from IFT and transfer it to a new authority.
    pub fn revoke_mint_authority(ctx: Context<RevokeMintAuthority>) -> Result<()> {
        instructions::revoke_mint_authority(ctx)
    }

    /// Set the daily mint rate limit (admin only)
    pub fn set_mint_rate_limit(
        ctx: Context<SetMintRateLimit>,
        msg: SetMintRateLimitMsg,
    ) -> Result<()> {
        instructions::set_mint_rate_limit(ctx, msg)
    }

    /// Pause or unpause an IFT token (admin only)
    pub fn set_paused(ctx: Context<SetPaused>, msg: SetPausedMsg) -> Result<()> {
        instructions::set_paused(ctx, msg)
    }

    /// Mint tokens to any account (admin only)
    pub fn admin_mint(ctx: Context<AdminMint>, msg: AdminMintMsg) -> Result<()> {
        instructions::admin_mint(ctx, msg)
    }
}
