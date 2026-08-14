// SPDX-License-Identifier: Apache-2.0

//! GMP (General Message Passing) account extraction utilities.
//!
//! Handles both protobuf-encoded (Cosmos-originated) and ABI-encoded
//! (Ethereum-originated) GMP payloads. Decodes packet data and builds
//! the remaining accounts needed for Solana `recv_packet` transactions.

use std::str::FromStr;

use alloy::sol_types::SolValue;
use anyhow::Result;
use solana_sdk::{instruction::AccountMeta, pubkey::Pubkey};

use solana_ibc_proto::{GmpPacketData, GmpSolanaPayload, Protobuf};

/// GMP (General Message Passing) port identifier.
pub const GMP_PORT_ID: &str = "gmpport";

/// Protobuf encoding type for GMP packets.
pub const PROTOBUF_ENCODING: &str = "application/x-protobuf";

/// ABI encoding identifier used by Ethereum ICS27 GMP.
pub const ABI_ENCODING: &str = "application/x-solidity-abi";

/// Maximum lamports the proof API will pre-fund per GMP packet (~0.05 SOL).
/// Caps the sender-specified `prefund_lamports` to prevent griefing.
pub const MAX_PREFUND_LAMPORTS: u64 = 50_000_000;

/// Packed account entry size: pubkey(32) + `is_signer`(1) + `is_writable`(1)
const PACKED_ACCOUNT_SIZE: usize = 34;

alloy::sol!("../../../ibc-solidity/contracts/msgs/IICS27GMPMsgs.sol");
alloy::sol!("../../../ibc-solidity/contracts/msgs/ISolanaGMPMsgs.sol");

/// ABI type alias for [`IICS27GMPMsgs::GMPPacketData`].
pub type AbiGmpPacketData = IICS27GMPMsgs::GMPPacketData;
/// ABI type alias for [`AbiGmpSolanaPayload`].
pub type AbiGmpSolanaPayload = ISolanaGMPMsgs::GMPSolanaPayload;

/// Extract GMP remaining accounts from a packet payload.
///
/// Dispatches to protobuf or ABI decoding based on `encoding`.
/// Returns an empty vec for non-GMP payloads.
///
/// # Errors
///
/// Returns an error if the payload cannot be decoded or if the receiver
/// pubkey / client ID is invalid.
pub fn extract_gmp_accounts(
    dest_port: &str,
    encoding: &str,
    payload_value: &[u8],
    dest_client: &str,
    gmp_program_id: Pubkey,
) -> Result<Vec<AccountMeta>> {
    if dest_port != GMP_PORT_ID {
        return Ok(Vec::new());
    }

    let Some(decoded) = decode_gmp_payload(encoding, payload_value)? else {
        return Ok(Vec::new());
    };

    build_gmp_account_list(
        decoded.sender,
        &decoded.receiver,
        decoded.salt,
        dest_client,
        gmp_program_id,
        decoded.execution_accounts.into_iter(),
    )
}

/// Find the GMP call result PDA for a given packet.
///
/// Returns `Some(pda)` if the packet is from the GMP port, where the PDA
/// stores the acknowledgement or timeout result of a GMP call.
///
/// Returns `None` for non-GMP ports or invalid sequence (0).
#[must_use]
pub fn find_gmp_result_pda(
    source_port: &str,
    source_client: &str,
    sequence: u64,
    gmp_program_id: Pubkey,
) -> Option<Pubkey> {
    if source_port != GMP_PORT_ID || sequence == 0 {
        return None;
    }

    let (pda, _bump) =
        solana_ibc_gmp_types::GMPCallResult::pda(source_client, sequence, &gmp_program_id);
    Some(pda)
}

/// Extract GMP PDA and `prefund_lamports` from a packet payload.
///
/// Handles both protobuf-encoded (Cosmos-originated) and ABI-encoded
/// (Ethereum-originated) payloads. Returns `None` for non-GMP packets or
/// when `prefund_lamports` is zero.
///
/// The caller uses the returned value to build a `system_program::transfer`
/// instruction before `recv_packet`.
///
/// # Errors
///
/// Returns an error if the payload cannot be decoded or if the client ID
/// or sender fields are invalid.
pub fn extract_gmp_prefund_lamports(
    dest_port: &str,
    encoding: &str,
    payload_value: &[u8],
    dest_client: &str,
    gmp_program_id: Pubkey,
) -> Result<Option<(Pubkey, u64)>> {
    if dest_port != GMP_PORT_ID {
        return Ok(None);
    }

    let Some(decoded) = decode_gmp_payload(encoding, payload_value)? else {
        return Ok(None);
    };

    if decoded.prefund_lamports == 0 {
        return Ok(None);
    }

    let client_id = solana_ibc_gmp_types::ClientId::new(dest_client)
        .map_err(|e| anyhow::anyhow!("Invalid client ID: {e:?}"))?;
    let gmp_account = solana_ibc_gmp_types::GMPAccount::new(
        client_id,
        decoded.sender,
        decoded.salt,
        &gmp_program_id,
    );
    let (gmp_pda, _) = gmp_account.pda();

    Ok(Some((gmp_pda, decoded.prefund_lamports)))
}

// ---------------------------------------------------------------------------
// Shared decoding
// ---------------------------------------------------------------------------

/// Decoded GMP payload fields shared by account extraction and prefund logic.
struct DecodedGmpPayload {
    sender: solana_ibc_proto::Sender,
    receiver: String,
    salt: solana_ibc_proto::Salt,
    prefund_lamports: u64,
    execution_accounts: Vec<AccountMeta>,
}

/// Decode a GMP payload using either protobuf or ABI encoding.
///
/// Returns `None` when the encoding is unrecognized or protobuf decoding
/// fails (treated as a non-GMP packet).
fn decode_gmp_payload(encoding: &str, payload_value: &[u8]) -> Result<Option<DecodedGmpPayload>> {
    let is_protobuf = encoding.is_empty() || encoding == PROTOBUF_ENCODING;
    let is_abi = encoding == ABI_ENCODING;

    if is_protobuf {
        decode_protobuf_gmp_payload(payload_value)
    } else if is_abi {
        decode_abi_gmp_payload(payload_value)
    } else {
        tracing::debug!("Skipping GMP payload with unknown encoding: '{encoding}'");
        Ok(None)
    }
}

fn decode_protobuf_gmp_payload(payload_value: &[u8]) -> Result<Option<DecodedGmpPayload>> {
    let packet = match GmpPacketData::decode_vec(payload_value) {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!("Failed to decode protobuf GMP packet: {e:?}");
            return Ok(None);
        }
    };

    let solana_payload = GmpSolanaPayload::decode_vec(&packet.payload)
        .map_err(|e| anyhow::anyhow!("Failed to decode GMP Solana payload: {e}"))?;

    // is_signer is always false at the transaction level.
    // PDAs sign via invoke_signed during CPI.
    let execution_accounts = solana_payload
        .accounts
        .iter()
        .map(|a| AccountMeta {
            pubkey: a.pubkey,
            is_signer: false,
            is_writable: a.is_writable,
        })
        .collect();

    Ok(Some(DecodedGmpPayload {
        sender: packet.sender,
        receiver: packet.receiver.to_string(),
        salt: packet.salt,
        prefund_lamports: solana_payload.prefund_lamports,
        execution_accounts,
    }))
}

fn decode_abi_gmp_payload(payload_value: &[u8]) -> Result<Option<DecodedGmpPayload>> {
    let abi_gmp: AbiGmpPacketData = SolValue::abi_decode(payload_value)
        .map_err(|e| anyhow::anyhow!("Failed to ABI decode GMPPacketData: {e}"))?;

    let abi_solana: AbiGmpSolanaPayload = SolValue::abi_decode(&abi_gmp.payload)
        .map_err(|e| anyhow::anyhow!("Failed to ABI decode GmpSolanaPayload: {e}"))?;

    let packed = &abi_solana.packedAccounts;
    if !packed.len().is_multiple_of(PACKED_ACCOUNT_SIZE) {
        anyhow::bail!(
            "Packed accounts length {} is not a multiple of {}",
            packed.len(),
            PACKED_ACCOUNT_SIZE
        );
    }

    let sender: solana_ibc_proto::Sender = abi_gmp
        .sender
        .try_into()
        .map_err(|_| anyhow::anyhow!("Invalid sender"))?;
    let salt: solana_ibc_proto::Salt = abi_gmp
        .salt
        .to_vec()
        .try_into()
        .map_err(|_| anyhow::anyhow!("Invalid salt"))?;

    let execution_accounts = packed
        .chunks_exact(PACKED_ACCOUNT_SIZE)
        .map(|chunk| {
            let pubkey_bytes: [u8; 32] = chunk[..32]
                .try_into()
                .expect("chunk is exactly PACKED_ACCOUNT_SIZE");
            AccountMeta {
                pubkey: Pubkey::from(pubkey_bytes),
                is_signer: false,
                is_writable: chunk[33] != 0,
            }
        })
        .collect();

    Ok(Some(DecodedGmpPayload {
        sender,
        receiver: abi_gmp.receiver,
        salt,
        prefund_lamports: abi_solana.prefundLamports,
        execution_accounts,
    }))
}

// ---------------------------------------------------------------------------
// Shared account building
// ---------------------------------------------------------------------------

/// Derive the GMP PDA and build the base account list
/// `[gmp_pda, target_program, ...execution_accounts]`.
fn build_gmp_account_list(
    sender: solana_ibc_proto::Sender,
    receiver: &str,
    salt: solana_ibc_proto::Salt,
    dest_client: &str,
    gmp_program_id: Pubkey,
    execution_accounts: impl Iterator<Item = AccountMeta>,
) -> Result<Vec<AccountMeta>> {
    let target_program = Pubkey::from_str(receiver)
        .map_err(|e| anyhow::anyhow!("Invalid target program pubkey: {e}"))?;

    let client_id = solana_ibc_gmp_types::ClientId::new(dest_client)
        .map_err(|e| anyhow::anyhow!("Invalid client ID: {e:?}"))?;

    tracing::info!(%sender, dest_client, "Building GMP account list");

    let gmp_account =
        solana_ibc_gmp_types::GMPAccount::new(client_id, sender, salt, &gmp_program_id);
    let (gmp_pda, _) = gmp_account.pda();

    let mut accounts = vec![
        AccountMeta {
            pubkey: gmp_pda,
            is_signer: false,
            is_writable: false,
        },
        AccountMeta {
            pubkey: target_program,
            is_signer: false,
            is_writable: false,
        },
    ];

    accounts.extend(execution_accounts);

    tracing::debug!(
        num_accounts = accounts.len(),
        gmp_pda = %gmp_pda,
        target = %target_program,
        "GMP account list built"
    );

    Ok(accounts)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn find_gmp_result_pda_returns_none_for_non_gmp_port() {
        let program_id = Pubkey::new_unique();
        assert!(find_gmp_result_pda("transfer", "client-0", 1, program_id).is_none());
    }

    #[test]
    fn find_gmp_result_pda_returns_none_for_zero_sequence() {
        let program_id = Pubkey::new_unique();
        assert!(find_gmp_result_pda(GMP_PORT_ID, "client-0", 0, program_id).is_none());
    }

    #[test]
    fn find_gmp_result_pda_returns_pda_for_valid_gmp_packet() {
        let program_id = Pubkey::new_unique();
        let result = find_gmp_result_pda(GMP_PORT_ID, "client-0", 1, program_id);
        assert!(result.is_some());

        let result2 = find_gmp_result_pda(GMP_PORT_ID, "client-0", 1, program_id);
        assert_eq!(result, result2);
    }
}
