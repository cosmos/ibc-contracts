// SPDX-License-Identifier: Apache-2.0

use crate::{state::*, ICS26_ROUTER_ID};
use anchor_lang::prelude::*;

/// Accounts for handling a packet timeout callback.
///
/// The account layout must match the CPI interface defined in
/// `solana_ibc_types::ibc_app::OnTimeoutPacket`: `[app_state, instructions_sysvar, payer, system_program]`.
#[derive(Accounts)]
pub struct OnTimeoutPacket<'info> {
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

pub fn on_timeout_packet(ctx: Context<OnTimeoutPacket>, msg: OnTimeoutPacketMsg) -> Result<()> {
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

    // Increment packet timed out counter
    app_state.packets_timed_out = app_state.packets_timed_out.saturating_add(1);

    // Emit event
    emit!(PacketTimedOut {
        source_client: msg.source_client.clone(),
        dest_client: msg.dest_client.clone(),
        sequence: msg.sequence,
    });

    msg!(
        "Test IBC App: Timed out packet from {} to {} (seq: {}), total timed out: {}",
        msg.source_client,
        msg.dest_client,
        msg.sequence,
        app_state.packets_timed_out
    );

    Ok(())
}
