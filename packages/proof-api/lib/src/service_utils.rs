// SPDX-License-Identifier: Apache-2.0

//! Common service patterns and utilities for proof API modules
//! This module provides shared functionality for `ProofApiService` implementations

use solana_sdk::signature::Signature;
use tendermint::Hash;
use tonic::Status;

/// Convert `anyhow::Error` to `tonic::Status`
#[must_use]
pub fn to_tonic_status(err: anyhow::Error) -> Status {
    Status::from_error(err.into())
}

/// Parse Cosmos transaction hashes from request
///
/// # Errors
///
/// Returns a `Status` error if any transaction ID cannot be parsed as a valid hash
#[inline]
#[allow(clippy::result_large_err)]
pub fn parse_cosmos_tx_hashes(tx_ids: Vec<Vec<u8>>) -> Result<Vec<Hash>, Status> {
    tx_ids
        .into_iter()
        .map(Hash::try_from)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| Status::from_error(e.into()))
}

/// Parse Ethereum transaction hashes from request
///
/// # Errors
///
/// Returns a `Status` error if any transaction ID is not exactly 32 bytes
#[inline]
#[allow(clippy::result_large_err)]
pub fn parse_eth_tx_hashes(tx_ids: Vec<Vec<u8>>) -> Result<Vec<[u8; 32]>, Status> {
    tx_ids
        .into_iter()
        .map(|tx_id| {
            tx_id
                .try_into()
                .map_err(|tx| format!("invalid tx hash: {tx:?}"))
        })
        .collect::<Result<Vec<[u8; 32]>, _>>()
        .map_err(|e| Status::from_error(e.into()))
}

/// Parse Solana transaction signatures (hashes) from request
///
/// # Errors
///
/// Returns a `Status` error if any transaction ID is not Solana Signature
#[inline]
#[allow(clippy::result_large_err)]
pub fn parse_solana_tx_hashes(tx_ids: Vec<Vec<u8>>) -> Result<Vec<Signature>, Status> {
    tx_ids
        .into_iter()
        .map(|tx_id| {
            let sig_str =
                String::from_utf8(tx_id).map_err(|e| format!("Invalid signature : {e}"))?;

            sig_str
                .parse::<Signature>()
                .map_err(|e| format!("Invalid signature: {e}"))
        })
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| Status::from_error(e.into()))
}
