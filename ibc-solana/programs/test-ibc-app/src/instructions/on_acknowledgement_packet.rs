// SPDX-License-Identifier: Apache-2.0

use crate::{state::*, ICS26_ROUTER_ID};
use anchor_lang::prelude::*;

/// Accounts for handling a packet acknowledgement callback.
#[derive(Accounts)]
#[instruction(msg: OnAcknowledgementPacketMsg)]
pub struct OnAcknowledgementPacket<'info> {
    /// App state PDA that tracks packet counters.
    #[account(
        init_if_needed,
        payer = payer,
        space = 8 + TestIbcAppState::INIT_SPACE,
        seeds = [IBCAppState::SEED],
        bump
    )]
    pub app_state: Account<'info, TestIbcAppState>,

    /// Instructions sysvar for CPI validation
    /// CHECK: Validated via address constraint
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instruction_sysvar: AccountInfo<'info>,

    /// Payer for account creation if needed
    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn on_acknowledgement_packet(
    ctx: Context<OnAcknowledgementPacket>,
    msg: OnAcknowledgementPacketMsg,
) -> Result<()> {
    // Validate CPI caller using shared validation function
    solana_ibc_types::validate_cpi_caller(
        &ctx.accounts.instruction_sysvar,
        &ICS26_ROUTER_ID,
        &crate::ID,
    )?;

    let app_state = &mut ctx.accounts.app_state;

    // Initialize authority if this is the first time (account was just created)
    if app_state.authority == Pubkey::default() {
        app_state.authority = ctx.accounts.payer.key();
    }

    // Increment packet acknowledged counter
    app_state.packets_acknowledged = app_state.packets_acknowledged.saturating_add(1);

    // Emit event
    emit!(PacketAcknowledged {
        source_client: msg.source_client.clone(),
        dest_client: msg.dest_client.clone(),
        sequence: msg.sequence,
        acknowledgement: msg.acknowledgement.clone(),
    });

    msg!(
        "Test IBC App: Acknowledged packet from {} to {} (seq: {}), ack: {:?}, total acknowledged: {}",
        msg.source_client,
        msg.dest_client,
        msg.sequence,
        String::from_utf8_lossy(&msg.acknowledgement),
        app_state.packets_acknowledged
    );

    Ok(())
}
