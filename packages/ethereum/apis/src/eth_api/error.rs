// SPDX-License-Identifier: Apache-2.0

//! This module defines errors for `EthApiClient`.

use alloy::{primitives::B256, transports::TransportError};

#[derive(Debug, thiserror::Error)]
#[allow(missing_docs, clippy::module_name_repetitions)]
pub enum EthClientError {
    #[error("provider error: {0}")]
    ProviderError(#[from] TransportError),

    #[error("parse error trying to parse {0}, {1}")]
    ParseError(String, String),

    #[error("block not found for block number {0}")]
    BlockNotFound(u64),

    #[error("block not found for block hash {0}")]
    BlockHashNotFound(B256),

    #[error("encoded block header hashes to {found} but expected {expected}")]
    BlockHeaderHashMismatch { expected: B256, found: B256 },
}
