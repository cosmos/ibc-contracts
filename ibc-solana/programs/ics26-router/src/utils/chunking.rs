// SPDX-License-Identifier: Apache-2.0

use crate::errors::RouterError;
use crate::state::{Delivery, MsgPayload, PayloadChunk, ProofChunk};
use anchor_lang::prelude::*;

/// Parameters for assembling single payload chunks
pub struct AssemblePayloadParams<'a, 'b, 'c> {
    pub remaining_accounts: &'a [AccountInfo<'b>],
    pub relayer: &'c AccountInfo<'b>,
    pub submitter: Pubkey,
    pub client_id: &'a str,
    pub sequence: u64,
    pub payload_index: u8,
    pub total_chunks: u8,
    pub start_index: usize,
}

/// Parameters for assembling proof chunks
pub struct AssembleProofParams<'a, 'b, 'c> {
    pub remaining_accounts: &'a [AccountInfo<'b>],
    pub relayer: &'c AccountInfo<'b>,
    pub submitter: Pubkey,
    pub client_id: &'a str,
    pub sequence: u64,
    pub delivery: &'a Delivery,
    pub start_index: usize,
}

/// Parameters for reconstructing a packet
pub struct ReconstructPacketParams<'a, 'b, 'c> {
    pub msg_packet: &'a solana_ibc_types::MsgPacket,
    pub remaining_accounts: &'a [AccountInfo<'b>],
    pub relayer: &'c AccountInfo<'b>,
    pub submitter: Pubkey,
    pub client_id: &'a str,
}

/// Parameters for cleaning up payload chunks
pub struct CleanupPayloadChunksParams<'a, 'b> {
    pub chunk_accounts: &'a [AccountInfo<'b>],
    pub submitter: Pubkey,
    pub client_id: &'a str,
    pub sequence: u64,
    pub payload_index: u8,
    pub total_chunks: u8,
}

/// Parameters for cleaning up proof chunks
pub struct CleanupProofChunksParams<'a, 'b> {
    pub chunk_accounts: &'a [AccountInfo<'b>],
    pub submitter: Pubkey,
    pub client_id: &'a str,
    pub sequence: u64,
}

pub fn assemble_multiple_payloads<'b>(
    remaining_accounts: &[AccountInfo<'b>],
    relayer: &AccountInfo<'b>,
    submitter: Pubkey,
    client_id: &str,
    sequence: u64,
    payloads: &[MsgPayload],
) -> Result<Vec<Vec<u8>>> {
    let mut all_payloads = Vec::new();
    let mut account_offset = 0;

    for (payload_index, payload) in payloads.iter().enumerate() {
        let total_chunks = match &payload.data {
            Delivery::Chunked { total_chunks } => *total_chunks,
            Delivery::Inline { .. } => {
                return Err(RouterError::MixedDeliveryModes.into());
            }
        };

        let payload_data = assemble_single_payload_chunks(AssemblePayloadParams {
            remaining_accounts,
            relayer,
            submitter,
            client_id,
            sequence,
            payload_index: payload_index as u8,
            total_chunks,
            start_index: account_offset,
        })?;

        all_payloads.push(payload_data);
        account_offset += total_chunks as usize;
    }

    Ok(all_payloads)
}

pub fn assemble_single_payload_chunks(params: AssemblePayloadParams) -> Result<Vec<u8>> {
    let mut payload_data = Vec::new();
    let mut accounts_processed = 0;

    // Collect and validate chunks
    for i in 0..params.total_chunks {
        let account_index = params.start_index + accounts_processed;
        require!(
            account_index < params.remaining_accounts.len(),
            RouterError::InvalidChunkCount
        );

        let chunk_account = &params.remaining_accounts[account_index];

        let expected_seeds = &[
            PayloadChunk::SEED,
            params.submitter.as_ref(),
            params.client_id.as_bytes(),
            &params.sequence.to_le_bytes(),
            &[params.payload_index],
            &[i],
        ];
        let (expected_pda, _) = Pubkey::find_program_address(expected_seeds, &crate::ID);

        require_keys_eq!(
            chunk_account.key(),
            expected_pda,
            RouterError::InvalidChunkAccount
        );

        require_keys_eq!(
            *chunk_account.owner,
            crate::ID,
            RouterError::InvalidAccountOwner
        );

        let chunk_data = chunk_account.try_borrow_data()?;
        let chunk: PayloadChunk = PayloadChunk::try_deserialize(&mut &chunk_data[..])?;

        require_eq!(&chunk.client_id, params.client_id);
        require_eq!(chunk.sequence, params.sequence);
        require_eq!(chunk.payload_index, params.payload_index);
        require_eq!(chunk.chunk_index, i);

        payload_data.extend_from_slice(&chunk.chunk_data);
        accounts_processed += 1;
    }

    // Clean up chunks and return rent
    cleanup_payload_chunks(CleanupPayloadChunksParams {
        chunk_accounts: &params.remaining_accounts
            [params.start_index..params.start_index + accounts_processed],
        submitter: params.submitter,
        client_id: params.client_id,
        sequence: params.sequence,
        payload_index: params.payload_index,
        total_chunks: params.total_chunks,
    })?;

    Ok(payload_data)
}

/// Validates that all payloads use the same delivery mode (all inline or all chunked)
/// and returns an error if mixed modes are detected.
fn validate_uniform_delivery(payloads: &[MsgPayload]) -> Result<()> {
    require!(
        payloads.windows(2).all(|w| {
            matches!(
                (&w[0].data, &w[1].data),
                (Delivery::Inline { .. }, Delivery::Inline { .. })
                    | (Delivery::Chunked { .. }, Delivery::Chunked { .. })
            )
        }),
        RouterError::MixedDeliveryModes
    );
    Ok(())
}

/// Returns the total number of payload chunk accounts needed for the given payloads.
pub fn total_payload_chunks(payloads: &[MsgPayload]) -> Result<usize> {
    validate_uniform_delivery(payloads)?;

    Ok(payloads
        .iter()
        .map(|p| p.data.total_chunks() as usize)
        .sum())
}

/// Filter out chunk accounts from `remaining_accounts` before passing to IBC app CPI
///
/// Chunk accounts (payload chunks and proof chunks) are router implementation details
/// and should not be visible to IBC applications. This function returns only the accounts
/// that come after all chunk accounts.
///
/// # Arguments
/// * `remaining_accounts` - All remaining accounts from the instruction
/// * `total_payload_chunks` - Number of payload chunk accounts (already calculated by caller)
/// * `proof_total_chunks` - Number of proof chunk accounts
///
/// # Returns
/// Slice of `remaining_accounts` with chunk accounts filtered out. Returns empty slice
/// if all accounts are chunks (which is valid when IBC app needs no extra accounts).
pub fn filter_app_remaining_accounts<'a, 'b>(
    remaining_accounts: &'a [AccountInfo<'b>],
    total_payload_chunks: usize,
    proof_total_chunks: u8,
) -> &'a [AccountInfo<'b>] {
    // Calculate total chunk accounts that need to be filtered out
    // Chunk accounts are at the beginning of remaining_accounts:
    // - First: payload chunk accounts (total_payload_chunks)
    // - Then: proof chunk accounts (proof_total_chunks)
    // - After chunks: IBC app-specific accounts
    let total_chunk_accounts = total_payload_chunks + proof_total_chunks as usize;

    // Return accounts after chunks, or empty slice if all accounts are chunks
    if total_chunk_accounts < remaining_accounts.len() {
        &remaining_accounts[total_chunk_accounts..]
    } else {
        // All remaining_accounts are chunks (valid case - app needs no extra accounts)
        // Note: If total_chunk_accounts > remaining_accounts.len(), validation would
        // have already failed in assemble_*_chunks() with InvalidChunkCount error
        &[]
    }
}

/// Assemble proof data from either inline delivery or chunked accounts
pub fn assemble_proof_chunks(params: AssembleProofParams) -> Result<Vec<u8>> {
    let total_chunks = match params.delivery {
        Delivery::Inline { data } => return Ok(data.clone()),
        Delivery::Chunked { total_chunks } => *total_chunks,
    };

    let mut proof_data = Vec::new();
    let mut accounts_processed = 0;

    // Collect and validate chunks
    for i in 0..total_chunks {
        let account_index = params.start_index + accounts_processed;
        require!(
            account_index < params.remaining_accounts.len(),
            RouterError::InvalidChunkCount
        );

        let chunk_account = &params.remaining_accounts[account_index];

        let expected_seeds = &[
            ProofChunk::SEED,
            params.submitter.as_ref(),
            params.client_id.as_bytes(),
            &params.sequence.to_le_bytes(),
            &[i],
        ];
        let (expected_pda, _) = Pubkey::find_program_address(expected_seeds, &crate::ID);

        require_keys_eq!(
            chunk_account.key(),
            expected_pda,
            RouterError::InvalidChunkAccount
        );

        require_keys_eq!(
            *chunk_account.owner,
            crate::ID,
            RouterError::InvalidAccountOwner
        );

        let chunk_data = chunk_account.try_borrow_data()?;
        let chunk: ProofChunk = ProofChunk::try_deserialize(&mut &chunk_data[..])?;

        require_eq!(&chunk.client_id, params.client_id);
        require_eq!(chunk.sequence, params.sequence);
        require_eq!(chunk.chunk_index, i);

        proof_data.extend_from_slice(&chunk.chunk_data);
        accounts_processed += 1;
    }

    cleanup_proof_chunks(CleanupProofChunksParams {
        chunk_accounts: &params.remaining_accounts
            [params.start_index..params.start_index + accounts_processed],
        submitter: params.submitter,
        client_id: params.client_id,
        sequence: params.sequence,
    })?;

    Ok(proof_data)
}

/// Clean up payload chunks by zeroing data (lamports remain for later reclaim via `cleanup_chunks`)
fn cleanup_payload_chunks(params: CleanupPayloadChunksParams) -> Result<()> {
    require_eq!(
        params.total_chunks,
        u8::try_from(params.chunk_accounts.len()).map_err(|_| RouterError::InvalidChunkCount)?,
        RouterError::InvalidChunkCount
    );

    for (i, chunk_account) in params.chunk_accounts.iter().enumerate() {
        // Double-check PDA (paranoid check)
        let expected_seeds = &[
            PayloadChunk::SEED,
            params.submitter.as_ref(),
            params.client_id.as_bytes(),
            &params.sequence.to_le_bytes(),
            &[params.payload_index],
            &[i as u8],
        ];
        let (expected_pda, _) = Pubkey::find_program_address(expected_seeds, &crate::ID);

        require_keys_eq!(
            chunk_account.key(),
            expected_pda,
            RouterError::InvalidChunkAccount
        );

        require_keys_eq!(
            *chunk_account.owner,
            crate::ID,
            RouterError::InvalidAccountOwner
        );

        // Clear the chunk data to prevent replay
        // Note: Lamports are NOT transferred here to avoid UnbalancedInstruction errors.
        // Users must call cleanup_chunks separately to reclaim rent.
        let mut data = chunk_account.try_borrow_mut_data()?;
        data.fill(0);
    }
    Ok(())
}

/// Clean up proof chunks by zeroing data (lamports remain for later reclaim via `cleanup_chunks`)
fn cleanup_proof_chunks(params: CleanupProofChunksParams) -> Result<()> {
    for (i, chunk_account) in params.chunk_accounts.iter().enumerate() {
        // Double-check PDA (paranoid check)
        let expected_seeds = &[
            ProofChunk::SEED,
            params.submitter.as_ref(),
            params.client_id.as_bytes(),
            &params.sequence.to_le_bytes(),
            &[i as u8],
        ];
        let (expected_pda, _) = Pubkey::find_program_address(expected_seeds, &crate::ID);
        require_keys_eq!(
            chunk_account.key(),
            expected_pda,
            RouterError::InvalidChunkAccount
        );

        require_keys_eq!(
            *chunk_account.owner,
            crate::ID,
            RouterError::InvalidAccountOwner
        );

        // Clear the chunk data to prevent replay
        // Note: Lamports are NOT transferred here to avoid UnbalancedInstruction errors.
        // Users must call cleanup_chunks separately to reclaim rent.
        let mut data = chunk_account.try_borrow_mut_data()?;
        data.fill(0);
    }
    Ok(())
}

/// Reconstruct a complete packet from either inline mode or chunked mode
///
/// This function provides a unified interface for packet reconstruction, handling both:
/// - **Inline mode**: All payloads use `Delivery::Inline` — data is extracted directly
/// - **Chunked mode**: All payloads use `Delivery::Chunked` — data is assembled from chunk accounts
///
/// All payloads must use the same delivery variant (no mixed modes).
///
/// # Returns
/// * `Ok(solana_ibc_types::Packet)` - Reconstructed packet with payloads
/// * `Err` - If validation fails, mixed delivery modes, or chunks cannot be assembled
pub fn validate_and_reconstruct_packet(
    params: ReconstructPacketParams,
) -> Result<solana_ibc_types::Packet> {
    let msg_payloads = &params.msg_packet.payloads;
    require!(!msg_payloads.is_empty(), RouterError::InvalidPayloadCount);

    validate_uniform_delivery(msg_payloads)?;

    // Safe to check only first: validate_uniform_delivery guarantees all payloads use the same mode
    let has_inline_payloads = matches!(&msg_payloads[0].data, Delivery::Inline { .. });

    let payloads = if has_inline_payloads {
        // Inline mode: extract data directly from each MsgPayload
        msg_payloads
            .iter()
            .map(|payload| {
                let data = match &payload.data {
                    Delivery::Inline { data } => data.clone(),
                    Delivery::Chunked { .. } => unreachable!(),
                };
                solana_ibc_types::Payload {
                    source_port: payload.source_port.clone(),
                    dest_port: payload.dest_port.clone(),
                    version: payload.version.clone(),
                    encoding: payload.encoding.clone(),
                    value: data,
                }
            })
            .collect()
    } else {
        // Chunked mode: assemble from chunk accounts
        let payload_data_vec = assemble_multiple_payloads(
            params.remaining_accounts,
            params.relayer,
            params.submitter,
            params.client_id,
            params.msg_packet.sequence,
            msg_payloads,
        )?;

        let mut assembled_payloads = Vec::new();
        for (i, metadata) in msg_payloads.iter().enumerate() {
            let payload = solana_ibc_types::Payload {
                source_port: metadata.source_port.clone(),
                dest_port: metadata.dest_port.clone(),
                version: metadata.version.clone(),
                encoding: metadata.encoding.clone(),
                value: payload_data_vec[i].clone(),
            };
            assembled_payloads.push(payload);
        }
        assembled_payloads
    };

    Ok(solana_ibc_types::Packet {
        sequence: params.msg_packet.sequence,
        source_client: params.msg_packet.source_client.clone(),
        dest_client: params.msg_packet.dest_client.clone(),
        timeout_timestamp: params.msg_packet.timeout_timestamp,
        payloads,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_total_payload_chunks_rejects_mixed_delivery() {
        let payloads = vec![
            MsgPayload {
                source_port: "port".to_string(),
                dest_port: "port".to_string(),
                version: "1".to_string(),
                encoding: "json".to_string(),
                data: Delivery::Inline {
                    data: b"data".to_vec(),
                },
            },
            MsgPayload {
                source_port: "port".to_string(),
                dest_port: "port".to_string(),
                version: "1".to_string(),
                encoding: "json".to_string(),
                data: Delivery::Chunked { total_chunks: 2 },
            },
        ];

        let result = total_payload_chunks(&payloads);
        assert!(result.is_err());
    }

    #[test]
    fn test_total_payload_chunks_all_inline() {
        let payloads = vec![MsgPayload {
            source_port: "port".to_string(),
            dest_port: "port".to_string(),
            version: "1".to_string(),
            encoding: "json".to_string(),
            data: Delivery::Inline {
                data: b"data".to_vec(),
            },
        }];

        assert_eq!(total_payload_chunks(&payloads).unwrap(), 0);
    }

    #[test]
    fn test_total_payload_chunks_all_chunked() {
        let payloads = vec![
            MsgPayload {
                source_port: "port".to_string(),
                dest_port: "port".to_string(),
                version: "1".to_string(),
                encoding: "json".to_string(),
                data: Delivery::Chunked { total_chunks: 2 },
            },
            MsgPayload {
                source_port: "port".to_string(),
                dest_port: "port".to_string(),
                version: "1".to_string(),
                encoding: "json".to_string(),
                data: Delivery::Chunked { total_chunks: 3 },
            },
        ];

        assert_eq!(total_payload_chunks(&payloads).unwrap(), 5);
    }

    #[test]
    fn test_validate_and_reconstruct_packet_inline_mode_success() {
        let packet = solana_ibc_types::MsgPacket {
            sequence: 1,
            source_client: "source-client".to_string(),
            dest_client: "dest-client".to_string(),
            timeout_timestamp: 1000,
            payloads: vec![MsgPayload {
                source_port: "transfer".to_string(),
                dest_port: "transfer".to_string(),
                version: "ics20-1".to_string(),
                encoding: "json".to_string(),
                data: Delivery::Inline {
                    data: b"test data".to_vec(),
                },
            }],
        };

        let relayer = Pubkey::new_unique();
        let mut lamports = 0u64;
        let mut data = [];
        let relayer_account = AccountInfo::new(
            &relayer,
            false,
            false,
            &mut lamports,
            &mut data,
            &crate::ID,
            false,
            0,
        );

        let params = ReconstructPacketParams {
            msg_packet: &packet,
            remaining_accounts: &[],
            relayer: &relayer_account,
            submitter: relayer,
            client_id: "client-0",
        };

        let result = validate_and_reconstruct_packet(params).unwrap();

        assert_eq!(result.sequence, 1);
        assert_eq!(result.payloads.len(), 1);
        assert_eq!(result.payloads[0].value, b"test data");
        assert_eq!(result.payloads[0].source_port, "transfer");
    }

    #[test]
    fn test_validate_and_reconstruct_packet_mixed_delivery_rejected() {
        let packet = solana_ibc_types::MsgPacket {
            sequence: 1,
            source_client: "client-0".to_string(),
            dest_client: "client-1".to_string(),
            timeout_timestamp: 1000,
            payloads: vec![
                MsgPayload {
                    source_port: "transfer".to_string(),
                    dest_port: "transfer".to_string(),
                    version: "ics20-1".to_string(),
                    encoding: "json".to_string(),
                    data: Delivery::Inline {
                        data: b"test data".to_vec(),
                    },
                },
                MsgPayload {
                    source_port: "transfer".to_string(),
                    dest_port: "transfer".to_string(),
                    version: "ics20-1".to_string(),
                    encoding: "json".to_string(),
                    data: Delivery::Chunked { total_chunks: 2 },
                },
            ],
        };

        let relayer = Pubkey::new_unique();
        let mut lamports = 0u64;
        let mut data = [];
        let relayer_account = AccountInfo::new(
            &relayer,
            false,
            false,
            &mut lamports,
            &mut data,
            &crate::ID,
            false,
            0,
        );

        let params = ReconstructPacketParams {
            msg_packet: &packet,
            remaining_accounts: &[],
            relayer: &relayer_account,
            submitter: relayer,
            client_id: "client-0",
        };

        let result = validate_and_reconstruct_packet(params);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(
            error.to_string().contains("MixedDeliveryModes"),
            "Expected MixedDeliveryModes error, got: {error}",
        );
    }

    #[test]
    fn test_validate_and_reconstruct_packet_empty_payloads() {
        let packet = solana_ibc_types::MsgPacket {
            sequence: 1,
            source_client: "client-0".to_string(),
            dest_client: "client-1".to_string(),
            timeout_timestamp: 1000,
            payloads: vec![],
        };

        let relayer = Pubkey::new_unique();
        let mut lamports = 0u64;
        let mut data = [];
        let relayer_account = AccountInfo::new(
            &relayer,
            false,
            false,
            &mut lamports,
            &mut data,
            &crate::ID,
            false,
            0,
        );

        let params = ReconstructPacketParams {
            msg_packet: &packet,
            remaining_accounts: &[],
            relayer: &relayer_account,
            submitter: relayer,
            client_id: "client-0",
        };

        let result = validate_and_reconstruct_packet(params);
        assert!(result.is_err());
        let error = result.unwrap_err();
        assert!(
            error.to_string().contains("InvalidPayloadCount"),
            "Expected InvalidPayloadCount error, got: {error}",
        );
    }

    // Helper function to create mock AccountInfo for testing
    fn create_mock_account_info<'a>(
        key: &'a Pubkey,
        lamports: &'a mut u64,
        data: &'a mut [u8],
        owner: &'a Pubkey,
    ) -> AccountInfo<'a> {
        AccountInfo::new(key, false, false, lamports, data, owner, false, 0)
    }

    #[test]
    fn test_filter_app_remaining_accounts_with_payload_and_proof_chunks() {
        // Create 3 payload chunks + 2 proof chunks + 4 app accounts (9 total)
        let keys = [
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
        ];
        let mut lamports0 = 0u64;
        let mut lamports1 = 0u64;
        let mut lamports2 = 0u64;
        let mut lamports3 = 0u64;
        let mut lamports4 = 0u64;
        let mut lamports5 = 0u64;
        let mut lamports6 = 0u64;
        let mut lamports7 = 0u64;
        let mut lamports8 = 0u64;
        let mut data0 = vec![];
        let mut data1 = vec![];
        let mut data2 = vec![];
        let mut data3 = vec![];
        let mut data4 = vec![];
        let mut data5 = vec![];
        let mut data6 = vec![];
        let mut data7 = vec![];
        let mut data8 = vec![];
        let owner = Pubkey::new_unique();

        let accounts = [
            create_mock_account_info(&keys[0], &mut lamports0, &mut data0, &owner),
            create_mock_account_info(&keys[1], &mut lamports1, &mut data1, &owner),
            create_mock_account_info(&keys[2], &mut lamports2, &mut data2, &owner),
            create_mock_account_info(&keys[3], &mut lamports3, &mut data3, &owner),
            create_mock_account_info(&keys[4], &mut lamports4, &mut data4, &owner),
            create_mock_account_info(&keys[5], &mut lamports5, &mut data5, &owner),
            create_mock_account_info(&keys[6], &mut lamports6, &mut data6, &owner),
            create_mock_account_info(&keys[7], &mut lamports7, &mut data7, &owner),
            create_mock_account_info(&keys[8], &mut lamports8, &mut data8, &owner),
        ];

        // Filter: 3 payload chunks + 2 proof chunks = 5 chunks to skip
        let result = filter_app_remaining_accounts(&accounts, 3, 2);

        // Should return last 4 accounts (app accounts after chunks)
        assert_eq!(result.len(), 4);
        assert_eq!(result[0].key, &keys[5]);
        assert_eq!(result[1].key, &keys[6]);
        assert_eq!(result[2].key, &keys[7]);
        assert_eq!(result[3].key, &keys[8]);
    }

    #[test]
    fn test_filter_app_remaining_accounts_only_payload_chunks() {
        // Create 2 payload chunks + 0 proof chunks + 3 app accounts (5 total)
        let keys = [
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
        ];
        let mut lamports0 = 0u64;
        let mut lamports1 = 0u64;
        let mut lamports2 = 0u64;
        let mut lamports3 = 0u64;
        let mut lamports4 = 0u64;
        let mut data0 = vec![];
        let mut data1 = vec![];
        let mut data2 = vec![];
        let mut data3 = vec![];
        let mut data4 = vec![];
        let owner = Pubkey::new_unique();

        let accounts = [
            create_mock_account_info(&keys[0], &mut lamports0, &mut data0, &owner),
            create_mock_account_info(&keys[1], &mut lamports1, &mut data1, &owner),
            create_mock_account_info(&keys[2], &mut lamports2, &mut data2, &owner),
            create_mock_account_info(&keys[3], &mut lamports3, &mut data3, &owner),
            create_mock_account_info(&keys[4], &mut lamports4, &mut data4, &owner),
        ];

        // Filter: 2 payload chunks + 0 proof chunks
        let result = filter_app_remaining_accounts(&accounts, 2, 0);

        // Should return last 3 accounts
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].key, &keys[2]);
        assert_eq!(result[1].key, &keys[3]);
        assert_eq!(result[2].key, &keys[4]);
    }

    #[test]
    fn test_filter_app_remaining_accounts_only_proof_chunks() {
        // Create 0 payload chunks + 3 proof chunks + 2 app accounts (5 total)
        let keys = [
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
        ];
        let mut lamports0 = 0u64;
        let mut lamports1 = 0u64;
        let mut lamports2 = 0u64;
        let mut lamports3 = 0u64;
        let mut lamports4 = 0u64;
        let mut data0 = vec![];
        let mut data1 = vec![];
        let mut data2 = vec![];
        let mut data3 = vec![];
        let mut data4 = vec![];
        let owner = Pubkey::new_unique();

        let accounts = [
            create_mock_account_info(&keys[0], &mut lamports0, &mut data0, &owner),
            create_mock_account_info(&keys[1], &mut lamports1, &mut data1, &owner),
            create_mock_account_info(&keys[2], &mut lamports2, &mut data2, &owner),
            create_mock_account_info(&keys[3], &mut lamports3, &mut data3, &owner),
            create_mock_account_info(&keys[4], &mut lamports4, &mut data4, &owner),
        ];

        // Filter: 0 payload chunks + 3 proof chunks
        let result = filter_app_remaining_accounts(&accounts, 0, 3);

        // Should return last 2 accounts
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].key, &keys[3]);
        assert_eq!(result[1].key, &keys[4]);
    }

    #[test]
    fn test_filter_app_remaining_accounts_no_chunks() {
        // Create 5 app accounts only (no chunks)
        let keys = [
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
        ];
        let mut lamports0 = 0u64;
        let mut lamports1 = 0u64;
        let mut lamports2 = 0u64;
        let mut lamports3 = 0u64;
        let mut lamports4 = 0u64;
        let mut data0 = vec![];
        let mut data1 = vec![];
        let mut data2 = vec![];
        let mut data3 = vec![];
        let mut data4 = vec![];
        let owner = Pubkey::new_unique();

        let accounts = [
            create_mock_account_info(&keys[0], &mut lamports0, &mut data0, &owner),
            create_mock_account_info(&keys[1], &mut lamports1, &mut data1, &owner),
            create_mock_account_info(&keys[2], &mut lamports2, &mut data2, &owner),
            create_mock_account_info(&keys[3], &mut lamports3, &mut data3, &owner),
            create_mock_account_info(&keys[4], &mut lamports4, &mut data4, &owner),
        ];

        // Filter: 0 payload chunks + 0 proof chunks
        let result = filter_app_remaining_accounts(&accounts, 0, 0);

        // Should return all 5 accounts unchanged
        assert_eq!(result.len(), 5);
        for (i, account) in result.iter().enumerate() {
            assert_eq!(account.key, &keys[i]);
        }
    }

    #[test]
    fn test_filter_app_remaining_accounts_all_chunks_no_app_accounts() {
        // Create 2 payload chunks + 1 proof chunk (3 total, no app accounts)
        let keys = [
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
        ];
        let mut lamports0 = 0u64;
        let mut lamports1 = 0u64;
        let mut lamports2 = 0u64;
        let mut data0 = vec![];
        let mut data1 = vec![];
        let mut data2 = vec![];
        let owner = Pubkey::new_unique();

        let accounts = [
            create_mock_account_info(&keys[0], &mut lamports0, &mut data0, &owner),
            create_mock_account_info(&keys[1], &mut lamports1, &mut data1, &owner),
            create_mock_account_info(&keys[2], &mut lamports2, &mut data2, &owner),
        ];

        // Filter: 2 payload chunks + 1 proof chunk = all 3 accounts
        let result = filter_app_remaining_accounts(&accounts, 2, 1);

        // Should return empty slice (no app accounts after chunks)
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_filter_app_remaining_accounts_more_chunks_than_accounts() {
        // Create 3 accounts total
        let keys = [
            Pubkey::new_unique(),
            Pubkey::new_unique(),
            Pubkey::new_unique(),
        ];
        let mut lamports0 = 0u64;
        let mut lamports1 = 0u64;
        let mut lamports2 = 0u64;
        let mut data0 = vec![];
        let mut data1 = vec![];
        let mut data2 = vec![];
        let owner = Pubkey::new_unique();

        let accounts = [
            create_mock_account_info(&keys[0], &mut lamports0, &mut data0, &owner),
            create_mock_account_info(&keys[1], &mut lamports1, &mut data1, &owner),
            create_mock_account_info(&keys[2], &mut lamports2, &mut data2, &owner),
        ];

        // Filter: 2 payload chunks + 2 proof chunks = 4 chunks expected (more than available)
        let result = filter_app_remaining_accounts(&accounts, 2, 2);

        // Should return empty slice (defensive behavior when chunks >= total accounts)
        assert_eq!(result.len(), 0);
    }
}
