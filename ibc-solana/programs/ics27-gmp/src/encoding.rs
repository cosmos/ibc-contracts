// SPDX-License-Identifier: Apache-2.0

use crate::constants::{ICS27_ENCODING_ABI, ICS27_ENCODING_PROTOBUF};
use crate::errors::GMPError;
use anchor_lang::prelude::*;
use solana_ibc_proto::{
    GmpAcknowledgement, GmpPacketData, ProstMessage, Protobuf, RawGmpPacketData,
};

use crate::sol_types::IICS27GMPMsgs::GMPAcknowledgement as GmpAcknowledgementAbi;
pub use crate::sol_types::IICS27GMPMsgs::GMPPacketData as GmpPacketDataAbi;

impl From<GmpPacketData> for GmpPacketDataAbi {
    fn from(data: GmpPacketData) -> Self {
        Self {
            sender: data.sender.into_string(),
            receiver: data.receiver.into_string(),
            salt: data.salt.into_vec().into(),
            payload: data.payload.into_inner().into(),
            memo: data.memo.into_string(),
        }
    }
}

impl From<GmpAcknowledgement> for GmpAcknowledgementAbi {
    fn from(ack: GmpAcknowledgement) -> Self {
        Self {
            result: ack.result.into(),
        }
    }
}

impl From<GmpPacketDataAbi> for RawGmpPacketData {
    fn from(abi: GmpPacketDataAbi) -> Self {
        Self {
            sender: abi.sender,
            receiver: abi.receiver,
            salt: abi.salt.into(),
            payload: abi.payload.into(),
            memo: abi.memo,
        }
    }
}

/// Encodes a validated [`GmpPacketData`] into its wire format.
///
/// # Errors
///
/// Returns [`GMPError::InvalidEncoding`] if `encoding` is not one of:
/// - `"application/x-solidity-abi"` — Solidity ABI encoding
/// - `"application/x-protobuf"` — Protobuf encoding
pub fn encode_gmp_packet(data: GmpPacketData, encoding: &str) -> Result<Vec<u8>> {
    match encoding {
        ICS27_ENCODING_ABI => {
            use alloy_sol_types::SolValue;
            Ok(GmpPacketDataAbi::from(data).abi_encode())
        }
        ICS27_ENCODING_PROTOBUF => Ok(data.encode_vec()),
        _ => Err(GMPError::InvalidEncoding.into()),
    }
}

/// Decodes raw bytes into a [`RawGmpPacketData`] (unvalidated fields).
///
/// Use [`GmpPacketData::try_from`] on the result to validate field contents.
///
/// # Errors
///
/// - [`GMPError::InvalidEncoding`] if `encoding` is not one of:
///   - `"application/x-solidity-abi"` — Solidity ABI encoding
///   - `"application/x-protobuf"` — Protobuf encoding
/// - [`GMPError::InvalidPacketData`] if `bytes` cannot be decoded in the given format.
pub fn decode_gmp_packet(bytes: &[u8], encoding: &str) -> Result<RawGmpPacketData> {
    match encoding {
        ICS27_ENCODING_ABI => {
            use alloy_sol_types::SolValue;
            let abi =
                GmpPacketDataAbi::abi_decode(bytes).map_err(|_| GMPError::InvalidPacketData)?;
            Ok(RawGmpPacketData::from(abi))
        }
        ICS27_ENCODING_PROTOBUF => {
            RawGmpPacketData::decode(bytes).map_err(|_| GMPError::InvalidPacketData.into())
        }
        _ => Err(GMPError::InvalidEncoding.into()),
    }
}

/// Encodes an acknowledgement **result** (not a full ack) into its wire format.
///
/// `result` is the raw success payload, not a pre-built [`GmpAcknowledgement`].
/// For protobuf encoding with an empty `result`, a sentinel byte (`0x00`) is
/// used to distinguish success-with-no-data from a missing field.
///
/// # Errors
///
/// Returns [`GMPError::InvalidEncoding`] if `encoding` is not one of:
/// - `"application/x-solidity-abi"` — Solidity ABI encoding
/// - `"application/x-protobuf"` — Protobuf encoding
pub fn encode_gmp_ack(result: &[u8], encoding: &str) -> Result<Vec<u8>> {
    match encoding {
        ICS27_ENCODING_ABI => {
            use alloy_sol_types::SolValue;
            Ok(
                GmpAcknowledgementAbi::from(GmpAcknowledgement::success(result.to_vec()))
                    .abi_encode(),
            )
        }
        ICS27_ENCODING_PROTOBUF => {
            let ack = if result.is_empty() {
                GmpAcknowledgement::protobuf_empty_success()
            } else {
                GmpAcknowledgement::success(result.to_vec())
            };
            Ok(ack.encode_to_vec())
        }
        _ => Err(GMPError::InvalidEncoding.into()),
    }
}

/// Decodes raw bytes into a [`GmpAcknowledgement`].
///
/// # Errors
///
/// - [`GMPError::InvalidEncoding`] if `encoding` is not one of:
///   - `"application/x-solidity-abi"` — Solidity ABI encoding
///   - `"application/x-protobuf"` — Protobuf encoding
/// - [`GMPError::InvalidPacketData`] if `bytes` cannot be decoded in the given format.
pub fn decode_gmp_ack(bytes: &[u8], encoding: &str) -> Result<GmpAcknowledgement> {
    match encoding {
        ICS27_ENCODING_ABI => {
            use alloy_sol_types::SolValue;
            let abi = GmpAcknowledgementAbi::abi_decode(bytes)
                .map_err(|_| GMPError::InvalidPacketData)?;
            Ok(GmpAcknowledgement::success(abi.result.into()))
        }
        ICS27_ENCODING_PROTOBUF => {
            GmpAcknowledgement::decode_vec(bytes).map_err(|_| GMPError::InvalidPacketData.into())
        }
        _ => Err(GMPError::InvalidEncoding.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_sol_types::SolValue;

    fn sample_packet_data() -> GmpPacketData {
        let raw = RawGmpPacketData {
            sender: "solana_sender_pubkey".to_string(),
            receiver: "0xabcdef1234567890abcdef1234567890abcdef12".to_string(),
            salt: vec![1, 2, 3],
            payload: vec![4, 5, 6, 7],
            memo: "test memo".to_string(),
        };
        GmpPacketData::try_from(raw).unwrap()
    }

    fn assert_raw_matches_sample(raw: &RawGmpPacketData) {
        assert_eq!(raw.sender, "solana_sender_pubkey");
        assert_eq!(raw.receiver, "0xabcdef1234567890abcdef1234567890abcdef12");
        assert_eq!(raw.salt, [1, 2, 3]);
        assert_eq!(raw.payload, [4, 5, 6, 7]);
        assert_eq!(raw.memo, "test memo");
    }

    #[test]
    fn abi_round_trip() {
        let original = sample_packet_data();
        let encoded = encode_gmp_packet(original, ICS27_ENCODING_ABI).unwrap();
        let raw = decode_gmp_packet(&encoded, ICS27_ENCODING_ABI).unwrap();
        assert_raw_matches_sample(&raw);

        let validated = GmpPacketData::try_from(raw).unwrap();
        assert_eq!(validated.sender.as_ref(), "solana_sender_pubkey");
    }

    #[test]
    fn protobuf_round_trip() {
        let original = sample_packet_data();
        let encoded = encode_gmp_packet(original, ICS27_ENCODING_PROTOBUF).unwrap();
        let raw = decode_gmp_packet(&encoded, ICS27_ENCODING_PROTOBUF).unwrap();
        assert_raw_matches_sample(&raw);

        let validated = GmpPacketData::try_from(raw).unwrap();
        assert_eq!(validated.sender.as_ref(), "solana_sender_pubkey");
    }

    #[test]
    fn invalid_encoding_rejected() {
        let data = sample_packet_data();
        let result = encode_gmp_packet(data, "application/json");
        assert!(result.is_err());
    }

    #[test]
    fn invalid_encoding_decode_rejected() {
        let result = decode_gmp_packet(&[0, 1, 2], "application/json");
        assert!(result.is_err());
    }

    #[test]
    fn abi_encode_matches_solidity_layout() {
        let abi = GmpPacketDataAbi {
            sender: "test".to_string(),
            receiver: "test".to_string(),
            salt: vec![].into(),
            payload: vec![1].into(),
            memo: String::new(),
        };
        let encoded = abi.abi_encode();
        // First 32 bytes should be offset pointer 0x20 for the dynamic struct
        assert_eq!(encoded[31], 0x20);
    }

    #[test]
    fn from_gmp_packet_data_preserves_fields() {
        let original = sample_packet_data();
        let abi: GmpPacketDataAbi = original.into();

        assert_eq!(abi.sender, "solana_sender_pubkey");
        assert_eq!(abi.receiver, "0xabcdef1234567890abcdef1234567890abcdef12");
        assert_eq!(&abi.salt[..], &[1, 2, 3]);
        assert_eq!(&abi.payload[..], &[4, 5, 6, 7]);
        assert_eq!(abi.memo, "test memo");
    }

    #[test]
    fn ack_abi_round_trip() {
        let data = vec![1, 2, 3, 4];
        let encoded = encode_gmp_ack(&data, ICS27_ENCODING_ABI).unwrap();
        assert!(!encoded.is_empty());
        let decoded = decode_gmp_ack(&encoded, ICS27_ENCODING_ABI).unwrap();
        assert_eq!(decoded.result, data);
    }

    #[test]
    fn ack_abi_empty_result() {
        let encoded = encode_gmp_ack(&[], ICS27_ENCODING_ABI).unwrap();
        assert!(!encoded.is_empty());
        let decoded = decode_gmp_ack(&encoded, ICS27_ENCODING_ABI).unwrap();
        assert!(decoded.result.is_empty());
    }

    #[test]
    fn ack_protobuf_round_trip() {
        let data = vec![1, 2, 3, 4];
        let encoded = encode_gmp_ack(&data, ICS27_ENCODING_PROTOBUF).unwrap();
        assert!(!encoded.is_empty());
        let decoded = decode_gmp_ack(&encoded, ICS27_ENCODING_PROTOBUF).unwrap();
        assert_eq!(decoded.result, data);
    }

    #[test]
    fn ack_protobuf_empty_result_uses_sentinel() {
        let encoded = encode_gmp_ack(&[], ICS27_ENCODING_PROTOBUF).unwrap();
        assert!(!encoded.is_empty());
        let decoded = decode_gmp_ack(&encoded, ICS27_ENCODING_PROTOBUF).unwrap();
        assert_eq!(decoded.result, vec![0]);
    }

    #[test]
    fn invalid_encoding_ack_rejected() {
        assert!(encode_gmp_ack(&[1], "application/json").is_err());
    }

    #[test]
    fn abi_to_raw_then_validation_catches_invalid() {
        let abi = GmpPacketDataAbi {
            sender: String::new(), // Empty sender should fail validation
            receiver: "test".to_string(),
            salt: vec![].into(),
            payload: vec![1].into(),
            memo: String::new(),
        };
        let raw = RawGmpPacketData::from(abi);
        let result = GmpPacketData::try_from(raw);
        assert!(result.is_err());
    }
}
