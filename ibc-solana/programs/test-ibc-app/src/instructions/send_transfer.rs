// SPDX-License-Identifier: Apache-2.0

use crate::{errors::TestIbcAppError, state::*};
use anchor_lang::prelude::*;
use ibc_proto::ibc::applications::transfer::v2::FungibleTokenPacketData;
use ics26_router::cpi as router_cpi;
use ics26_router::program::Ics26Router;
use ics26_router::{
    cpi::accounts::SendPacket as RouterSendPacket,
    state::{Client, IBCApp, MsgSendPacket, RouterState},
};
use prost::Message;
use solana_ibc_types::Payload;

/// Message for sending a transfer via IBC
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct SendTransferMsg {
    /// Token denomination (e.g., "sol", "usdc", etc.)
    pub denom: String,
    /// Amount to transfer as string (for compatibility with ICS20)
    pub amount: String,
    /// Receiver address on the destination chain
    pub receiver: String,
    /// Source client ID for the destination chain
    pub source_client: String,
    /// Destination port (usually "transfer")
    pub dest_port: String,
    /// Timeout timestamp (Unix timestamp in seconds)
    pub timeout_timestamp: u64,
    /// Optional memo field
    pub memo: String,
    /// Caller-chosen packet sequence number
    pub sequence: u64,
}

/// Accounts for sending an ICS-20 transfer via the router.
#[derive(Accounts)]
#[instruction(msg: SendTransferMsg)]
pub struct SendTransfer<'info> {
    /// App state PDA, also used as the `app_signer` for router CPI.
    #[account(
        mut,
        seeds = [IBCAppState::SEED],
        bump
    )]
    pub app_state: Account<'info, TestIbcAppState>,

    /// User sending the transfer
    #[account(mut)]
    pub user: Signer<'info>,

    /// Escrow account to hold SOL during transfer
    /// CHECK: PDA derived from `client_id`, will be validated
    #[account(
        mut,
        seeds = [TestIbcAppState::ESCROW_SEED, msg.source_client.as_bytes()],
        bump
    )]
    pub escrow_account: AccountInfo<'info>,

    /// Optional escrow state to track transfers (created if needed)
    #[account(
        init_if_needed,
        payer = user,
        space = 8 + EscrowState::INIT_SPACE,
        seeds = [EscrowState::SEED, msg.source_client.as_bytes()],
        bump
    )]
    pub escrow_state: Account<'info, EscrowState>,

    /// Router global state (read-only).
    #[account(
        seeds = [RouterState::SEED],
        bump,
        seeds::program = router_program
    )]
    pub router_state: Account<'info, RouterState>,

    /// Router port-to-app binding for the transfer port.
    #[account(
        seeds = [IBCApp::SEED, TRANSFER_PORT.as_bytes()],
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

pub fn send_transfer(ctx: Context<SendTransfer>, msg: SendTransferMsg) -> Result<()> {
    // Get clock directly via syscall
    let clock = Clock::get()?;

    // No need to validate router_caller since it's a PDA derived by Anchor

    // Validate timeout - for this demo we'll just use a simple check
    let current_timestamp =
        u64::try_from(clock.unix_timestamp).map_err(|_| TestIbcAppError::ArithmeticOverflow)?;
    if msg.timeout_timestamp <= current_timestamp {
        return Err(error!(TestIbcAppError::InvalidPacketData));
    }

    // Parse amount for SOL transfer
    let amount_lamports = msg
        .amount
        .parse::<u64>()
        .map_err(|_| error!(TestIbcAppError::InvalidPacketData))?;

    // Validate user has enough SOL
    require!(
        ctx.accounts.user.lamports() >= amount_lamports,
        TestIbcAppError::InvalidPacketData
    );

    // Transfer SOL from user to escrow account via System Program CPI
    let transfer_instruction = anchor_lang::solana_program::system_instruction::transfer(
        &ctx.accounts.user.key(),
        &ctx.accounts.escrow_account.key(),
        amount_lamports,
    );

    anchor_lang::solana_program::program::invoke(
        &transfer_instruction,
        &[
            ctx.accounts.user.to_account_info(),
            ctx.accounts.escrow_account.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
    )?;

    // Update escrow state tracking
    let escrow_state = &mut ctx.accounts.escrow_state;
    if escrow_state.client_id.is_empty() {
        escrow_state.client_id.clone_from(&msg.source_client);
        escrow_state.authority = ctx.accounts.user.key();
    } else {
        // Validate existing escrow state matches current transaction
        require!(
            escrow_state.client_id == msg.source_client,
            TestIbcAppError::InvalidPacketData
        );
        require!(
            escrow_state.authority == ctx.accounts.user.key(),
            TestIbcAppError::InvalidPacketData
        );
    }
    escrow_state.total_escrowed = escrow_state.total_escrowed.saturating_add(amount_lamports);
    escrow_state.active_transfers = escrow_state.active_transfers.saturating_add(1);

    msg!(
        "Transferred {} lamports to escrow for transfer (total escrowed: {})",
        amount_lamports,
        escrow_state.total_escrowed
    );

    // Create ICS20-compatible packet data using proper protobuf encoding
    let fungible_token_data = FungibleTokenPacketData {
        denom: msg.denom.clone(),
        amount: msg.amount.clone(),
        sender: ctx.accounts.user.key().to_string(),
        receiver: msg.receiver.clone(),
        memo: msg.memo.clone(),
    };

    // Serialize to protobuf bytes
    let packet_data = fungible_token_data.encode_to_vec();

    // Create payload for router
    let payload = Payload {
        source_port: "transfer".to_string(),
        dest_port: msg.dest_port,
        version: "ics20-1".to_string(),
        encoding: "application/x-protobuf".to_string(),
        value: packet_data,
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
    emit!(TransferSent {
        sequence,
        source_client: msg.source_client.clone(),
        denom: msg.denom.clone(),
        amount: msg.amount.clone(),
        sender: ctx.accounts.user.key().to_string(),
        receiver: msg.receiver.clone(),
    });

    msg!(
        "Test app sent transfer: {} {} from {} to {} (seq: {})",
        msg.amount,
        msg.denom,
        ctx.accounts.user.key(),
        msg.receiver,
        sequence
    );

    Ok(())
}
