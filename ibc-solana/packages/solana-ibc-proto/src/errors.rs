// SPDX-License-Identifier: Apache-2.0

/// Errors for GMP packet validation
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum GMPPacketError {
    /// Failed to decode protobuf
    #[error("Failed to decode GMP packet")]
    DecodeError,
    /// Sender validation failed
    #[error("Invalid sender address")]
    InvalidSender,
    /// Receiver validation failed
    #[error("Invalid receiver address")]
    InvalidReceiver,
    /// Salt validation failed
    #[error("Invalid salt")]
    InvalidSalt,
    /// Payload is empty
    #[error("Empty payload")]
    EmptyPayload,
    /// Payload validation failed
    #[error("Invalid payload")]
    InvalidPayload,
    /// Memo exceeds maximum length
    #[error("Memo too long")]
    MemoTooLong,
}

/// Validation errors for GMP payloads
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum GmpValidationError {
    #[error("Failed to decode GMP payload")]
    DecodeError,
    #[error("Invalid program ID (must be 32 bytes)")]
    InvalidProgramId,
    #[error("Empty payload data")]
    EmptyPayload,
    #[error("Too many accounts (max 32)")]
    TooManyAccounts,
    #[error("Invalid account key (must be 32 bytes)")]
    InvalidAccountKey,
}
