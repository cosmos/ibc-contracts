// SPDX-License-Identifier: Apache-2.0

//! Packet instruction builders for recv, ack, and timeout packets.

use anchor_lang::prelude::*;
use anyhow::Result;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    instruction::{AccountMeta, Instruction},
    pubkey::Pubkey,
};

use crate::gmp;
use solana_ibc_sdk::access_manager::instructions as access_manager_instructions;
use solana_ibc_sdk::ics07_tendermint::instructions as ics07_tendermint_instructions;
use solana_ibc_sdk::ics26_router::{
    accounts::IBCApp,
    instructions::{
        AckPacket, AckPacketAccounts, RecvPacket, RecvPacketAccounts, SendPacket, TimeoutPacket,
        TimeoutPacketAccounts,
    },
    types::{Delivery, MsgAckPacket, MsgPayload, MsgRecvPacket, MsgTimeoutPacket},
};

use crate::constants::ANCHOR_DISCRIMINATOR_SIZE;

/// Derives client state and consensus state PDAs which always use the same seeds across
/// different light client implementations
fn derive_light_client_pdas(height: u64, light_client_program_id: Pubkey) -> (Pubkey, Pubkey) {
    let (cs, _) = ics07_tendermint_instructions::Initialize::client_state_account_pda(
        &light_client_program_id,
    );
    let (cons, _) = ics07_tendermint_instructions::VerifyMembership::consensus_state_at_height_pda(
        height,
        &light_client_program_id,
    );
    (cs, cons)
}

/// Extracted payload info for recv packet processing.
pub(super) struct RecvPayloadInfo<'a> {
    pub dest_port: &'a str,
    pub encoding: &'a str,
    pub value: &'a [u8],
}

/// Extract `source_port` from packet payloads.
fn extract_source_port<'a>(payloads: &'a [MsgPayload], context: &str) -> Result<&'a str> {
    let [payload] = payloads else {
        anyhow::bail!(
            "Expected exactly one {context} packet payload element, got {}",
            payloads.len()
        );
    };
    Ok(&payload.source_port)
}

/// Extract payload info from packet payloads, using `payload_data` for chunked deliveries.
pub(super) fn extract_recv_payload_info<'a>(
    msg: &'a MsgRecvPacket,
    payload_data: &'a [Vec<u8>],
) -> Result<RecvPayloadInfo<'a>> {
    let [payload] = msg.packet.payloads.as_slice() else {
        anyhow::bail!("Expected exactly one recv packet payload element");
    };
    match &payload.data {
        Delivery::Inline { data } => Ok(RecvPayloadInfo {
            dest_port: &payload.dest_port,
            encoding: &payload.encoding,
            value: data,
        }),
        Delivery::Chunked { .. } => {
            let value = payload_data
                .first()
                .ok_or_else(|| anyhow::anyhow!("Missing payload data"))?
                .as_slice();
            Ok(RecvPayloadInfo {
                dest_port: &payload.dest_port,
                encoding: &payload.encoding,
                value,
            })
        }
    }
}

impl super::TxBuilder {
    pub(crate) fn build_recv_packet_instruction(
        &self,
        msg: &MsgRecvPacket,
        chunk_accounts: Vec<Pubkey>,
        payload_data: &[Vec<u8>],
    ) -> Result<Instruction> {
        let payload_info = extract_recv_payload_info(msg, payload_data)?;

        let light_client_program_id = self.resolve_client_program_id(&msg.packet.dest_client)?;
        let (client_state, consensus_state) =
            derive_light_client_pdas(msg.proof.height, light_client_program_id);

        let ibc_app_program_id = self.resolve_port_program_id(payload_info.dest_port)?;
        let (ibc_app_state, _) = solana_ibc_sdk::pda::ibc_app::app_state_pda(&ibc_app_program_id);
        let access_manager_program_id = self.resolve_access_manager_program_id()?;
        let (access_manager, _) =
            access_manager_instructions::Initialize::access_manager_pda(&access_manager_program_id);

        let gmp_accounts = gmp::extract_gmp_accounts(
            payload_info.dest_port,
            payload_info.encoding,
            payload_info.value,
            &msg.packet.dest_client,
            ibc_app_program_id,
        )?;

        Ok(RecvPacket::builder(&self.solana_ics26_program_id)
            .accounts(RecvPacketAccounts {
                access_manager,
                ibc_app_program: ibc_app_program_id,
                ibc_app_state,
                relayer: self.fee_payer,
                light_client_program: light_client_program_id,
                client_state,
                consensus_state,
                dest_port: payload_info.dest_port.as_bytes(),
                dest_client: &msg.packet.dest_client,
                sequence: msg.packet.sequence,
            })
            .args(msg)
            .remaining_accounts(
                chunk_accounts
                    .into_iter()
                    .map(|a| AccountMeta::new(a, false))
                    .chain(gmp_accounts),
            )
            .build())
    }

    pub(crate) fn build_ack_packet_instruction(
        &self,
        msg: &MsgAckPacket,
        chunk_accounts: Vec<Pubkey>,
    ) -> Result<Instruction> {
        let source_port = extract_source_port(&msg.packet.payloads, "ack")?;

        let (ibc_app_pda, _) = SendPacket::ibc_app_pda(source_port, &self.solana_ics26_program_id);

        let ibc_app_account = self
            .target_solana_client
            .get_account_with_commitment(&ibc_app_pda, CommitmentConfig::confirmed())
            .map_err(|e| anyhow::anyhow!("Failed to get IBC app account: {e}"))?
            .value
            .ok_or_else(|| anyhow::anyhow!("IBC app account not found"))?;

        if ibc_app_account.data.len() < ANCHOR_DISCRIMINATOR_SIZE {
            anyhow::bail!("Account data too short for IBCApp account");
        }

        let mut account_data = &ibc_app_account.data[ANCHOR_DISCRIMINATOR_SIZE..];
        let ibc_app = IBCApp::deserialize(&mut account_data)
            .map_err(|e| anyhow::anyhow!("Failed to deserialize IBCApp account: {e}"))?;
        let ibc_app_program = ibc_app.app_program_id;

        let (app_state, _) = solana_ibc_sdk::pda::ibc_app::app_state_pda(&ibc_app_program);

        let light_client_program_id = self.resolve_client_program_id(&msg.packet.source_client)?;
        let (client_state, consensus_state) =
            derive_light_client_pdas(msg.proof.height, light_client_program_id);

        let access_manager_program_id = self.resolve_access_manager_program_id()?;
        let (access_manager, _) =
            access_manager_instructions::Initialize::access_manager_pda(&access_manager_program_id);

        // GMP result PDA for GMP packets (initialized by on_ack_packet)
        // IFT finalize_transfer is handled as a separate transaction after ack completes
        let gmp_result = gmp::find_gmp_result_pda(
            source_port,
            &msg.packet.source_client,
            msg.packet.sequence,
            ibc_app_program,
        )
        .map(|pda| AccountMeta::new(pda, false));

        Ok(AckPacket::builder(&self.solana_ics26_program_id)
            .accounts(AckPacketAccounts {
                access_manager,
                ibc_app_program,
                ibc_app_state: app_state,
                relayer: self.fee_payer,
                light_client_program: light_client_program_id,
                client_state,
                consensus_state,
                source_port: source_port.as_bytes(),
                source_client: &msg.packet.source_client,
                sequence: msg.packet.sequence,
            })
            .args(msg)
            .remaining_accounts(
                chunk_accounts
                    .into_iter()
                    .map(|a| AccountMeta::new(a, false))
                    .chain(gmp_result),
            )
            .build())
    }

    pub(crate) fn build_timeout_packet_instruction(
        &self,
        msg: &MsgTimeoutPacket,
        chunk_accounts: Vec<Pubkey>,
    ) -> Result<Instruction> {
        let source_port = extract_source_port(&msg.packet.payloads, "timeout")?;

        let ibc_app_program_id = self.resolve_port_program_id(source_port)?;
        let (ibc_app_state, _) = solana_ibc_sdk::pda::ibc_app::app_state_pda(&ibc_app_program_id);

        let light_client_program_id = self.resolve_client_program_id(&msg.packet.source_client)?;
        let (client_state, consensus_state) =
            derive_light_client_pdas(msg.proof.height, light_client_program_id);

        let access_manager_program_id = self.resolve_access_manager_program_id()?;
        let (access_manager, _) =
            access_manager_instructions::Initialize::access_manager_pda(&access_manager_program_id);

        // GMP result PDA for GMP packets (initialized by on_timeout_packet)
        // IFT finalize_transfer is handled as a separate transaction after timeout completes
        let gmp_result = gmp::find_gmp_result_pda(
            source_port,
            &msg.packet.source_client,
            msg.packet.sequence,
            ibc_app_program_id,
        )
        .map(|pda| AccountMeta::new(pda, false));

        Ok(TimeoutPacket::builder(&self.solana_ics26_program_id)
            .accounts(TimeoutPacketAccounts {
                access_manager,
                ibc_app_program: ibc_app_program_id,
                ibc_app_state,
                relayer: self.fee_payer,
                light_client_program: light_client_program_id,
                client_state,
                consensus_state,
                source_port: source_port.as_bytes(),
                source_client: &msg.packet.source_client,
                sequence: msg.packet.sequence,
            })
            .args(msg)
            .remaining_accounts(
                chunk_accounts
                    .into_iter()
                    .map(|a| AccountMeta::new(a, false))
                    .chain(gmp_result),
            )
            .build())
    }
}
