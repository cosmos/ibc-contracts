// SPDX-License-Identifier: Apache-2.0

//! Shared protobuf definitions for Solana IBC
//!
//! This crate provides protobuf-generated types used across Solana IBC programs and relayer.
//! By centralizing proto generation, we ensure type consistency and enable shared validation logic.

use anchor_lang::prelude::*;

// Re-export Protobuf trait and Error type for users
pub use ibc_proto::{Error as ProtobufError, Protobuf};

// Re-export prost Message trait for raw protobuf decoding
pub use prost::Message as ProstMessage;

// Re-export constrained types
pub use ibc_eureka_constrained_types::{
    ConstrainedBytes, ConstrainedError, ConstrainedString, ConstrainedVec, NonEmpty,
};

mod errors;
pub use errors::{GMPPacketError, GmpValidationError};

// Generated protobuf modules
mod ibc_applications_gmp_v1 {
    #![allow(clippy::all, clippy::doc_markdown, clippy::use_self)]
    include!(concat!(env!("OUT_DIR"), "/ibc.applications.gmp.v1.rs"));
}

pub mod solana {
    #![allow(clippy::all, clippy::doc_markdown, clippy::use_self)]
    include!(concat!(env!("OUT_DIR"), "/solana.rs"));
}

pub use ibc_applications_gmp_v1::{
    Acknowledgement as GmpAcknowledgement, GmpPacketData as RawGmpPacketData,
};
pub use solana::{
    GmpSolanaPayload as RawGmpSolanaPayload, SolanaAccountMeta as RawSolanaAccountMeta,
};

/// Maximum client ID length — capped at Solana's `MAX_SEED_LEN` (32 bytes per seed element).
pub const MAX_CLIENT_ID_LENGTH: usize = 32;
/// Maximum sender address length (128 bytes)
pub const MAX_SENDER_LENGTH: usize = 128;
/// Maximum receiver address length (128 bytes)
pub const MAX_RECEIVER_LENGTH: usize = 128;
/// Maximum salt length (32 bytes)
pub const MAX_SALT_LENGTH: usize = 32;
/// Maximum memo length (256 bytes)
pub const MAX_MEMO_LENGTH: usize = 256;

pub type ClientId = ConstrainedString<1, MAX_CLIENT_ID_LENGTH>;
pub type Salt = ConstrainedBytes<0, MAX_SALT_LENGTH>;
pub type Sender = ConstrainedString<1, MAX_SENDER_LENGTH>;
/// Receiver can be empty to support native Cosmos module calls
/// where the receiver is implicitly determined by the GMP payload routing
pub type Receiver = ConstrainedString<0, MAX_RECEIVER_LENGTH>;
pub type Memo = ConstrainedString<0, MAX_MEMO_LENGTH>;
pub type Payload = NonEmpty<Vec<u8>>;

/// Domain type for GMP packet data with validated and constrained fields.
///
/// All fields are validated and constrained at the type level.
/// The payload is validated to be non-empty (no maximum length constraint).
#[derive(Debug, Clone)]
pub struct GmpPacketData {
    pub sender: Sender,
    pub receiver: Receiver,
    pub salt: Salt,
    pub payload: Payload,
    pub memo: Memo,
}

impl Protobuf<RawGmpPacketData> for GmpPacketData {}

impl TryFrom<RawGmpPacketData> for GmpPacketData {
    type Error = GMPPacketError;

    fn try_from(raw: RawGmpPacketData) -> core::result::Result<Self, Self::Error> {
        let sender = raw
            .sender
            .try_into()
            .map_err(|_| GMPPacketError::InvalidSender)?;

        let receiver = raw
            .receiver
            .try_into()
            .map_err(|_| GMPPacketError::InvalidReceiver)?;

        let salt = raw
            .salt
            .try_into()
            .map_err(|_| GMPPacketError::InvalidSalt)?;

        let payload = raw
            .payload
            .try_into()
            .map_err(|_| GMPPacketError::EmptyPayload)?;

        let memo = raw
            .memo
            .try_into()
            .map_err(|_| GMPPacketError::MemoTooLong)?;

        Ok(Self {
            sender,
            receiver,
            salt,
            payload,
            memo,
        })
    }
}

impl From<GmpPacketData> for RawGmpPacketData {
    fn from(packet: GmpPacketData) -> Self {
        Self {
            sender: packet.sender.into_string(),
            receiver: packet.receiver.into_string(),
            salt: packet.salt.into_vec(),
            payload: packet.payload.into_inner(),
            memo: packet.memo.into_string(),
        }
    }
}

/// Domain type for Solana account metadata with validated Pubkey
#[derive(Debug, Clone)]
pub struct SolanaAccountMeta {
    pub pubkey: Pubkey,
    pub is_signer: bool,
    pub is_writable: bool,
}

impl From<&SolanaAccountMeta> for anchor_lang::prelude::AccountMeta {
    fn from(meta: &SolanaAccountMeta) -> Self {
        Self {
            pubkey: meta.pubkey,
            is_signer: meta.is_signer,
            is_writable: meta.is_writable,
        }
    }
}

/// Maximum number of accounts allowed in a [`GmpSolanaPayload`]
pub const MAX_GMP_SOLANA_PAYLOAD_ACCOUNTS: usize = 32;

/// Domain type for GMP Solana payload with type-safe fields
#[derive(Debug, Clone)]
pub struct GmpSolanaPayload {
    pub data: Vec<u8>,
    pub accounts: Vec<SolanaAccountMeta>,
    pub prefund_lamports: u64,
}

impl GmpSolanaPayload {
    /// Convert accounts to Anchor `AccountMeta` format
    pub fn to_account_metas(&self) -> Vec<anchor_lang::prelude::AccountMeta> {
        self.accounts.iter().map(Into::into).collect()
    }
}

impl From<GmpSolanaPayload> for RawGmpSolanaPayload {
    fn from(payload: GmpSolanaPayload) -> Self {
        Self {
            data: payload.data,
            accounts: payload
                .accounts
                .into_iter()
                .map(|acc| RawSolanaAccountMeta {
                    pubkey: acc.pubkey.to_bytes().to_vec(),
                    is_signer: acc.is_signer,
                    is_writable: acc.is_writable,
                })
                .collect(),
            prefund_lamports: payload.prefund_lamports,
        }
    }
}

impl Protobuf<RawGmpSolanaPayload> for GmpSolanaPayload {}

impl TryFrom<RawGmpSolanaPayload> for GmpSolanaPayload {
    type Error = GmpValidationError;

    fn try_from(raw: RawGmpSolanaPayload) -> core::result::Result<Self, Self::Error> {
        // Validate data
        if raw.data.is_empty() {
            return Err(GmpValidationError::EmptyPayload);
        }

        if raw.accounts.len() > MAX_GMP_SOLANA_PAYLOAD_ACCOUNTS {
            return Err(GmpValidationError::TooManyAccounts);
        }

        let mut accounts = Vec::with_capacity(raw.accounts.len());
        for account in raw.accounts {
            let pubkey = Pubkey::try_from(&account.pubkey[..])
                .map_err(|_| GmpValidationError::InvalidAccountKey)?;
            accounts.push(SolanaAccountMeta {
                pubkey,
                is_signer: account.is_signer,
                is_writable: account.is_writable,
            });
        }

        Ok(Self {
            data: raw.data,
            accounts,
            prefund_lamports: raw.prefund_lamports,
        })
    }
}

// GmpAcknowledgement is a simple protobuf type that doesn't need validation
impl Protobuf<Self> for GmpAcknowledgement {}

impl GmpAcknowledgement {
    /// Proto3 omits empty bytes fields, so a bare `vec![]` would encode to
    /// zero bytes and be rejected by the router as an empty acknowledgement.
    /// Only needed for protobuf encoding — ABI handles empty bytes natively.
    const PROTOBUF_EMPTY_SENTINEL: &[u8] = &[0];

    pub const fn success(result: Vec<u8>) -> Self {
        Self { result }
    }

    /// Protobuf-only: use sentinel byte to avoid proto3 dropping empty result.
    pub fn protobuf_empty_success() -> Self {
        Self {
            result: Self::PROTOBUF_EMPTY_SENTINEL.to_vec(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_account() -> RawSolanaAccountMeta {
        RawSolanaAccountMeta {
            pubkey: vec![0u8; 32],
            is_signer: false,
            is_writable: false,
        }
    }

    #[test]
    fn try_from_rejects_too_many_accounts() {
        let raw = RawGmpSolanaPayload {
            data: vec![1],
            accounts: vec![valid_account(); MAX_GMP_SOLANA_PAYLOAD_ACCOUNTS + 1],
            prefund_lamports: 0,
        };
        assert_eq!(
            GmpSolanaPayload::try_from(raw).unwrap_err(),
            GmpValidationError::TooManyAccounts,
        );
    }

    #[test]
    fn try_from_accepts_max_accounts() {
        let raw = RawGmpSolanaPayload {
            data: vec![1],
            accounts: vec![valid_account(); MAX_GMP_SOLANA_PAYLOAD_ACCOUNTS],
            prefund_lamports: 0,
        };
        assert!(GmpSolanaPayload::try_from(raw).is_ok());
    }

    #[test]
    fn gmp_ack_encode_decode_round_trip() {
        // target returned data
        let data = 42u64.to_le_bytes().to_vec();
        let ack = GmpAcknowledgement::success(data.clone());
        let encoded = ack.encode_to_vec();
        assert!(!encoded.is_empty());
        let decoded = GmpAcknowledgement::decode_vec(&encoded).unwrap();
        assert_eq!(decoded.result, data);

        // target returned nothing (like SPL Token) — protobuf needs sentinel
        let ack = GmpAcknowledgement::protobuf_empty_success();
        let encoded = ack.encode_to_vec();
        assert!(!encoded.is_empty());
        let decoded = GmpAcknowledgement::decode_vec(&encoded).unwrap();
        assert_eq!(decoded.result, GmpAcknowledgement::PROTOBUF_EMPTY_SENTINEL);

        // without the sentinel, proto3 drops the field and we get zero bytes
        let broken = GmpAcknowledgement { result: Vec::new() };
        assert!(broken.encode_to_vec().is_empty());
    }
}
