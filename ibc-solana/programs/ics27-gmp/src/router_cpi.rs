// SPDX-License-Identifier: Apache-2.0

use crate::state::GMPAppState;
use anchor_lang::prelude::*;
use ics26_router::cpi::accounts::SendPacket;
use solana_ibc_types::MsgSendPacket;

/// Send IBC packet via CPI to the ICS26 router
#[allow(clippy::too_many_arguments)]
pub fn send_packet_cpi<'a>(
    router_program: &AccountInfo<'a>,
    router_state: &AccountInfo<'a>,
    packet_commitment: &AccountInfo<'a>,
    app_state: &AccountInfo<'a>,
    bump: u8,
    payer: &AccountInfo<'a>,
    ibc_app: &AccountInfo<'a>,
    client: &AccountInfo<'a>,
    light_client_program: &AccountInfo<'a>,
    client_state: &AccountInfo<'a>,
    consensus_state: &AccountInfo<'a>,
    system_program: &AccountInfo<'a>,
    msg: MsgSendPacket,
) -> Result<u64> {
    let cpi_accounts = SendPacket {
        router_state: router_state.clone(),
        ibc_app: ibc_app.clone(),
        packet_commitment: packet_commitment.clone(),
        app_signer: app_state.clone(),
        payer: payer.clone(),
        system_program: system_program.clone(),
        client: client.clone(),
        light_client_program: light_client_program.clone(),
        client_state: client_state.clone(),
        consensus_state: consensus_state.clone(),
    };

    let bump_slice = [bump];
    let signer_seeds: &[&[u8]] = &[GMPAppState::SEED, &bump_slice];
    let seeds = [signer_seeds];
    let cpi_ctx = CpiContext::new_with_signer(router_program.clone(), cpi_accounts, &seeds);
    let sequence = ics26_router::cpi::send_packet(cpi_ctx, msg)?;
    Ok(sequence.get())
}
