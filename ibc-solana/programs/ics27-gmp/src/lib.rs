// SPDX-License-Identifier: Apache-2.0

use anchor_lang::prelude::*;
use solana_ibc_macros::ibc_app;

pub mod abi;
pub mod constants;
pub mod encoding;
pub mod errors;
pub mod events;
pub mod gmp_solana_payload;
pub mod instructions;
pub mod proto;
pub mod router_cpi;
mod sol_types;
pub mod state;

#[cfg(test)]
pub mod test_utils;

use instructions::*;
use state::SendCallMsg;

declare_id!("3W3h4WSE8J9vFzVN8TGFGc9Uchbry3M4MBz4icdSWcFi");

#[cfg(test)]
pub fn get_gmp_program_path() -> &'static str {
    use std::sync::OnceLock;
    static PATH: OnceLock<String> = OnceLock::new();

    PATH.get_or_init(|| {
        std::env::var("GMP_PROGRAM_PATH")
            .unwrap_or_else(|_| "../../target/deploy/ics27_gmp".to_string())
    })
}

#[ibc_app]
pub mod ics27_gmp {
    use super::*;

    /// Initialize the ICS27 GMP application
    pub fn initialize(ctx: Context<Initialize>, access_manager: Pubkey) -> Result<()> {
        instructions::initialize(ctx, access_manager)
    }

    /// Send a GMP call packet (direct wallet call only)
    pub fn send_call(ctx: Context<SendCall>, msg: SendCallMsg) -> Result<u64> {
        instructions::send_call(ctx, msg)
    }

    /// Send a GMP call packet via CPI (program callers only)
    pub fn send_call_cpi(ctx: Context<SendCallCpi>, msg: SendCallMsg) -> Result<u64> {
        instructions::send_call_cpi(ctx, msg)
    }

    /// IBC packet receive handler (called by router via CPI)
    pub fn on_recv_packet<'info>(
        ctx: Context<'_, '_, 'info, 'info, OnRecvPacket<'info>>,
        msg: solana_ibc_types::OnRecvPacketMsg,
    ) -> Result<Vec<u8>> {
        instructions::on_recv_packet(ctx, msg)
    }

    /// IBC acknowledgement handler (called by router via CPI)
    pub fn on_acknowledgement_packet<'info>(
        ctx: Context<'_, '_, 'info, 'info, OnAckPacket<'info>>,
        msg: solana_ibc_types::OnAcknowledgementPacketMsg,
    ) -> Result<()> {
        instructions::on_acknowledgement_packet(ctx, msg)
    }

    /// IBC timeout handler (called by router via CPI)
    pub fn on_timeout_packet<'info>(
        ctx: Context<'_, '_, 'info, 'info, OnTimeoutPacket<'info>>,
        msg: solana_ibc_types::OnTimeoutPacketMsg,
    ) -> Result<()> {
        instructions::on_timeout_packet(ctx, msg)
    }

    /// Pause the entire GMP app (admin only)
    pub fn pause_app(ctx: Context<PauseApp>) -> Result<()> {
        instructions::pause_app(ctx)
    }

    /// Unpause the entire GMP app (admin only)
    pub fn unpause_app(ctx: Context<UnpauseApp>) -> Result<()> {
        instructions::unpause_app(ctx)
    }

    /// Propose transferring the access manager to a new program (admin only)
    pub fn propose_access_manager_transfer(
        ctx: Context<ProposeAccessManagerTransfer>,
        new_access_manager: Pubkey,
    ) -> Result<()> {
        instructions::propose_access_manager_transfer(ctx, new_access_manager)
    }

    /// Accept a pending access manager transfer (new AM admin only)
    pub fn accept_access_manager_transfer(ctx: Context<AcceptAccessManagerTransfer>) -> Result<()> {
        instructions::accept_access_manager_transfer(ctx)
    }

    /// Cancel a pending access manager transfer (current AM admin only)
    pub fn cancel_access_manager_transfer(ctx: Context<CancelAccessManagerTransfer>) -> Result<()> {
        instructions::cancel_access_manager_transfer(ctx)
    }
}
