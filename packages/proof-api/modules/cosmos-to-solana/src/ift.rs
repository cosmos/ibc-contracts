// SPDX-License-Identifier: Apache-2.0

//! IFT `finalize_transfer` instruction builder for ack/timeout packets.
//!
//! After GMP processes ack/timeout and creates `GMPCallResultAccount`,
//! the proof API calls IFT's `finalize_transfer` to process refunds.

use std::str::FromStr;
use std::sync::Arc;

use anchor_lang::prelude::*;
use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_ibc_sdk::ift::accounts::PendingTransfer;
use solana_ibc_sdk::ift::instructions::{
    FinalizeTransfer, FinalizeTransferAccounts, FinalizeTransferArgs,
};
use solana_sdk::{instruction::Instruction, pubkey::Pubkey};
use spl_associated_token_account::get_associated_token_address_with_program_id;

use crate::constants::ANCHOR_DISCRIMINATOR_SIZE;
use crate::gmp::{ABI_ENCODING, GMP_PORT_ID, PROTOBUF_ENCODING};

/// Parameters for building IFT `finalize_transfer` instruction
pub struct FinalizeTransferParams<'a> {
    pub source_port: &'a str,
    pub encoding: &'a str,
    pub payload_value: &'a [u8],
    pub source_client: &'a str,
    pub sequence: u64,
    pub solana_client: &'a Arc<RpcClient>,
    pub fee_payer: Pubkey,
}

/// Build IFT `finalize_transfer` instruction if this packet is from IFT.
/// Returns None if the packet is not an IFT transfer or no pending transfer exists.
pub fn build_finalize_transfer_instruction(
    params: &FinalizeTransferParams<'_>,
) -> Option<Instruction> {
    // Only process GMP port packets
    if params.source_port != GMP_PORT_ID
        || !(params.encoding.is_empty()
            || params.encoding == PROTOBUF_ENCODING
            || params.encoding == ABI_ENCODING)
    {
        return None;
    }

    let gmp_packet = crate::gmp::decode_gmp_packet(params.payload_value, params.encoding)?;

    // Parse sender as Pubkey (the IFT program ID).
    // GMP's `send_call_cpi` uses the calling program ID as the sender.
    let ift_program_id = match Pubkey::from_str(&gmp_packet.sender) {
        Ok(pk) => pk,
        Err(e) => {
            tracing::warn!(error = ?e, sender = %gmp_packet.sender, "IFT: GMP sender is not a valid Pubkey");
            return None;
        }
    };

    // Try to find pending transfer for this (client_id, sequence)
    let pending_transfer = match find_pending_transfer(
        params.solana_client,
        ift_program_id,
        params.source_client,
        params.sequence,
    ) {
        Ok(Some(pt)) => pt,
        Ok(None) => {
            tracing::debug!(
                client_id = params.source_client,
                sequence = params.sequence,
                "IFT: No pending transfer found"
            );
            return None;
        }
        Err(e) => {
            tracing::error!(error = ?e, "IFT: Error searching for pending transfer");
            return None;
        }
    };

    tracing::debug!(
        mint = %pending_transfer.mint,
        amount = pending_transfer.amount,
        sequence = params.sequence,
        "IFT: Building finalize_transfer instruction"
    );

    let token_program_id = match params.solana_client.get_account(&pending_transfer.mint) {
        Ok(mint_account) => mint_account.owner,
        Err(e) => {
            tracing::warn!(error = ?e, mint = %pending_transfer.mint, "IFT: Failed to fetch mint account, defaulting to spl_token");
            spl_token::id()
        }
    };

    Some(build_finalize_transfer_ix(
        ift_program_id,
        &pending_transfer,
        params.source_client,
        params.sequence,
        params.fee_payer,
        token_program_id,
    ))
}

fn find_pending_transfer(
    solana_client: &Arc<RpcClient>,
    ift_program_id: Pubkey,
    client_id: &str,
    sequence: u64,
) -> Result<Option<PendingTransfer>> {
    let all_accounts = solana_client
        .get_program_accounts(&ift_program_id)
        .map_err(|e| anyhow::anyhow!("Failed to get program accounts: {e}"))?;

    let accounts: Vec<_> = all_accounts
        .into_iter()
        .filter(|(_, account)| {
            account.data.len() >= ANCHOR_DISCRIMINATOR_SIZE
                && account.data[..ANCHOR_DISCRIMINATOR_SIZE] == PendingTransfer::DISCRIMINATOR
        })
        .collect();

    for (pubkey, account) in accounts {
        let mut data = &account.data[ANCHOR_DISCRIMINATOR_SIZE..];
        let pending = match PendingTransfer::deserialize(&mut data) {
            Ok(p) => p,
            Err(e) => {
                tracing::debug!(error = ?e, account = %pubkey, "IFT: Failed to deserialize account");
                continue;
            }
        };

        if pending.client_id == client_id && pending.sequence == sequence {
            return Ok(Some(pending));
        }
    }

    Ok(None)
}

fn build_finalize_transfer_ix(
    ift_program_id: Pubkey,
    pending_transfer: &PendingTransfer,
    client_id: &str,
    sequence: u64,
    fee_payer: Pubkey,
    token_program_id: Pubkey,
) -> Instruction {
    let mint = pending_transfer.mint;

    let (pending_transfer_pda, _) =
        FinalizeTransfer::pending_transfer_pda(&mint, client_id, sequence, &ift_program_id);
    let (gmp_result_pda, _) = FinalizeTransfer::gmp_result_pda(client_id, sequence);

    let sender_token_account = get_associated_token_address_with_program_id(
        &pending_transfer.sender,
        &mint,
        &token_program_id,
    );

    FinalizeTransfer::builder(&ift_program_id)
        .accounts(FinalizeTransferAccounts {
            pending_transfer: pending_transfer_pda,
            gmp_result: gmp_result_pda,
            mint,
            sender_token_account,
            payer: fee_payer,
            token_program: token_program_id,
        })
        .args(&FinalizeTransferArgs {
            client_id: client_id.to_string(),
            sequence,
        })
        .build()
}
