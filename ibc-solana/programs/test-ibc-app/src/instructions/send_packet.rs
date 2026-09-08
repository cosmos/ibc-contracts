// SPDX-License-Identifier: Apache-2.0

use crate::{errors::TestIbcAppError, state::*};
use anchor_lang::prelude::*;
use ics26_router::cpi as router_cpi;
use ics26_router::program::Ics26Router;
use ics26_router::{
    cpi::accounts::SendPacket as RouterSendPacket,
    state::{Client, IBCApp, MsgSendPacket, RouterState},
};
use solana_ibc_types::Payload;

/// Message for sending an arbitrary packet via IBC
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct SendPacketMsg {
    /// Source client ID for the destination chain
    pub source_client: String,
    /// Source port (e.g., "transfer", "oracle", etc.)
    pub source_port: String,
    /// Destination port
    pub dest_port: String,
    /// Version string for the packet
    pub version: String,
    /// Encoding format (e.g., "json", "protobuf")
    pub encoding: String,
    /// Arbitrary packet data
    pub packet_data: Vec<u8>,
    /// Timeout timestamp (Unix timestamp in seconds)
    pub timeout_timestamp: u64,
    /// Caller-chosen packet sequence number
    pub sequence: u64,
}

/// Accounts for sending an arbitrary IBC packet via the router.
#[derive(Accounts)]
#[instruction(msg: SendPacketMsg)]
pub struct SendPacket<'info> {
    /// App state PDA, also used as the `app_signer` for router CPI.
    #[account(
        mut,
        seeds = [IBCAppState::SEED],
        bump
    )]
    pub app_state: Account<'info, TestIbcAppState>,

    /// User sending the packet
    #[account(mut)]
    pub user: Signer<'info>,

    /// Router global state (read-only).
    #[account(
        seeds = [RouterState::SEED],
        bump,
        seeds::program = router_program
    )]
    pub router_state: Account<'info, RouterState>,

    /// Router port-to-app binding for the source port.
    #[account(
        seeds = [IBCApp::SEED, msg.source_port.as_bytes()],
        bump,
        seeds::program = router_program
    )]
    pub ibc_app: Account<'info, IBCApp>,

    /// Will be created by the router
    /// CHECK: PDA will be validated by router program
    #[account(mut)]
    pub packet_commitment: AccountInfo<'info>,

    /// Client registration entry for the source client.
    #[account(
        seeds = [Client::SEED, msg.source_client.as_bytes()],
        bump,
        seeds::program = router_program
    )]
    pub client: Account<'info, Client>,

    /// CHECK: Light client program, forwarded to router for status check
    pub light_client_program: AccountInfo<'info>,

    /// CHECK: Client state account, forwarded to router for status check
    pub client_state: AccountInfo<'info>,

    /// CHECK: Consensus state account, forwarded to router for expiry check
    pub consensus_state: AccountInfo<'info>,

    /// ICS26 router program for CPI.
    pub router_program: Program<'info, Ics26Router>,

    pub system_program: Program<'info, System>,
}

pub fn send_packet(ctx: Context<SendPacket>, msg: SendPacketMsg) -> Result<()> {
    // Get clock directly via syscall
    let clock = Clock::get()?;

    // Validate timeout
    let current_timestamp =
        u64::try_from(clock.unix_timestamp).map_err(|_| TestIbcAppError::ArithmeticOverflow)?;
    if msg.timeout_timestamp <= current_timestamp {
        return Err(error!(TestIbcAppError::InvalidPacketData));
    }

    let payload = Payload {
        source_port: msg.source_port.clone(),
        dest_port: msg.dest_port.clone(),
        version: msg.version,
        encoding: msg.encoding,
        value: msg.packet_data.clone(),
    };

    // Call router via CPI to send packet
    let router_msg = MsgSendPacket {
        source_client: msg.source_client.clone(),
        sequence: msg.sequence,
        timeout_timestamp: msg.timeout_timestamp,
        payload,
    };

    let cpi_accounts = RouterSendPacket {
        router_state: ctx.accounts.router_state.to_account_info(),
        ibc_app: ctx.accounts.ibc_app.to_account_info(),
        packet_commitment: ctx.accounts.packet_commitment.to_account_info(),
        app_signer: ctx.accounts.app_state.to_account_info(),
        payer: ctx.accounts.user.to_account_info(),
        system_program: ctx.accounts.system_program.to_account_info(),
        client: ctx.accounts.client.to_account_info(),
        light_client_program: ctx.accounts.light_client_program.to_account_info(),
        client_state: ctx.accounts.client_state.to_account_info(),
        consensus_state: ctx.accounts.consensus_state.to_account_info(),
    };

    // Sign the app_state PDA to prove this app is the immediate caller
    let bump = ctx.bumps.app_state;
    let signer_seeds: &[&[u8]] = &[IBCAppState::SEED, &[bump]];
    let seeds = [signer_seeds];
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.router_program.to_account_info(),
        cpi_accounts,
        &seeds,
    );
    let sequence_result = router_cpi::send_packet(cpi_ctx, router_msg)?;
    let sequence = sequence_result.get();

    // Update app state - track packets sent
    let app_state = &mut ctx.accounts.app_state;
    app_state.packets_sent = app_state.packets_sent.saturating_add(1);

    // Emit event for tracking
    emit!(PacketSent {
        sequence,
        source_client: msg.source_client.clone(),
        source_port: msg.source_port.clone(),
        dest_port: msg.dest_port.clone(),
        data_length: msg.packet_data.len() as u64,
    });

    msg!(
        "Test app sent packet: {} -> {} (seq: {}, {} bytes)",
        msg.source_port,
        msg.dest_port,
        sequence,
        msg.packet_data.len()
    );

    Ok(())
}

#[event]
pub struct PacketSent {
    pub sequence: u64,
    pub source_client: String,
    pub source_port: String,
    pub dest_port: String,
    pub data_length: u64,
}
