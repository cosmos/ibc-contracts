// SPDX-License-Identifier: Apache-2.0

//! CPI helpers for calling ICS27-GMP program

use anchor_lang::prelude::*;

/// Accounts required for GMP `send_call_cpi` CPI
pub struct SendGmpCallAccounts<'info> {
    pub gmp_program: AccountInfo<'info>,
    pub gmp_app_state: AccountInfo<'info>,
    pub payer: AccountInfo<'info>,
    pub router_program: AccountInfo<'info>,
    pub router_state: AccountInfo<'info>,
    pub packet_commitment: AccountInfo<'info>,
    pub ibc_app: AccountInfo<'info>,
    pub client: AccountInfo<'info>,
    pub light_client_program: AccountInfo<'info>,
    pub client_state: AccountInfo<'info>,
    pub instruction_sysvar: AccountInfo<'info>,
    pub consensus_state: AccountInfo<'info>,
    pub system_program: AccountInfo<'info>,
}

impl<'info> From<SendGmpCallAccounts<'info>> for ics27_gmp::cpi::accounts::SendCallCpi<'info> {
    fn from(accounts: SendGmpCallAccounts<'info>) -> Self {
        Self {
            app_state: accounts.gmp_app_state,
            payer: accounts.payer,
            router_program: accounts.router_program,
            router_state: accounts.router_state,
            packet_commitment: accounts.packet_commitment,
            ibc_app: accounts.ibc_app,
            client: accounts.client,
            light_client_program: accounts.light_client_program,
            client_state: accounts.client_state,
            instruction_sysvar: accounts.instruction_sysvar,
            consensus_state: accounts.consensus_state,
            system_program: accounts.system_program,
        }
    }
}

/// Message parameters for GMP `send_call_cpi`
pub struct SendGmpCallMsg {
    pub source_client: String,
    pub timeout_timestamp: u64,
    pub receiver: String,
    pub payload: Vec<u8>,
    pub encoding: String,
    pub sequence: u64,
}

impl From<SendGmpCallMsg> for ics27_gmp::state::SendCallMsg {
    fn from(msg: SendGmpCallMsg) -> Self {
        Self {
            source_client: msg.source_client,
            timeout_timestamp: msg.timeout_timestamp,
            receiver: msg.receiver,
            salt: vec![], // Empty salt for IFT
            payload: msg.payload,
            memo: String::new(),
            encoding: msg.encoding,
            sequence: msg.sequence,
        }
    }
}

/// Send a GMP call via CPI to the ICS27-GMP program.
///
/// The calling program's ID is automatically extracted by GMP from the
/// instruction sysvar and used as the sender.
pub fn send_gmp_call(accounts: SendGmpCallAccounts, msg: SendGmpCallMsg) -> Result<u64> {
    let gmp_program = accounts.gmp_program.clone();
    let cpi_ctx = CpiContext::new(gmp_program, accounts.into());
    let sequence = ics27_gmp::cpi::send_call_cpi(cpi_ctx, msg.into())
        .map_err(|_| error!(crate::errors::IFTError::GmpCallFailed))?;
    Ok(sequence.get())
}
