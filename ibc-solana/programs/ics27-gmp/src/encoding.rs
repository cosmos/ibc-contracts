// SPDX-License-Identifier: Apache-2.0

use crate::constants::{ICS27_ENCODING_ABI, ICS27_ENCODING_PROTOBUF};
use crate::errors::GMPError;
use anchor_lang::prelude::*;
use solana_ibc_proto::{
    GmpAcknowledgement, GmpPacketData, ProstMessage, Protobuf, RawGmpPacketData,
};

use crate::sol_types::IICS27GMPMsgs::GMPAcknowledgement as GmpAcknowledgementAbi;
pub use crate::sol_types::IICS27GMPMsgs::GMPPacketData as GmpPacketDataAbi;

/// Wire encoding of GMP packet data, named by the IBC payload's `encoding` field.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GmpEncoding {
    /// [`ICS27_ENCODING_ABI`] — Solidity ABI encoding
    Abi,
    /// [`ICS27_ENCODING_PROTOBUF`] — Protobuf encoding
    Protobuf,
}

impl GmpEncoding {
    /// The payload `encoding` string for this encoding.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Abi => ICS27_ENCODING_ABI,
            Self::Protobuf => ICS27_ENCODING_PROTOBUF,
        }
    }
}

impl TryFrom<&str> for GmpEncoding {
    type Error = GMPError;

    /// Returns [`GMPError::InvalidEncoding`] for any string other than
    /// [`ICS27_ENCODING_ABI`] or [`ICS27_ENCODING_PROTOBUF`].
    fn try_from(encoding: &str) -> core::result::Result<Self, Self::Error> {
        match encoding {
            ICS27_ENCODING_ABI => Ok(Self::Abi),
            ICS27_ENCODING_PROTOBUF => Ok(Self::Protobuf),
            _ => Err(GMPError::InvalidEncoding),
        }
    }
}

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
pub fn encode_gmp_packet(data: GmpPacketData, encoding: GmpEncoding) -> Vec<u8> {
    match encoding {
        GmpEncoding::Abi => {
            use alloy_sol_types::SolValue;
            GmpPacketDataAbi::from(data).abi_encode()
        }
        GmpEncoding::Protobuf => data.encode_vec(),
    }
}

/// Decodes raw bytes into a [`RawGmpPacketData`] (unvalidated fields).
///
/// Use [`GmpPacketData::try_from`] on the result to validate field contents.
///
/// # Errors
///
/// Returns [`GMPError::InvalidPacketData`] if `bytes` cannot be decoded in the given format.
pub fn decode_gmp_packet(bytes: &[u8], encoding: GmpEncoding) -> Result<RawGmpPacketData> {
    match encoding {
        GmpEncoding::Abi => {
            use alloy_sol_types::SolValue;
            let abi =
                GmpPacketDataAbi::abi_decode(bytes).map_err(|_| GMPError::InvalidPacketData)?;
            Ok(RawGmpPacketData::from(abi))
        }
        GmpEncoding::Protobuf => {
            RawGmpPacketData::decode(bytes).map_err(|_| GMPError::InvalidPacketData.into())
        }
    }
}

/// Encodes an acknowledgement **result** (not a full ack) into its wire format.
///
/// `result` is the raw success payload, not a pre-built [`GmpAcknowledgement`].
/// For protobuf encoding with an empty `result`, a sentinel byte (`0x00`) is
/// used to distinguish success-with-no-data from a missing field.
pub fn encode_gmp_ack(result: &[u8], encoding: GmpEncoding) -> Vec<u8> {
    match encoding {
        GmpEncoding::Abi => {
            use alloy_sol_types::SolValue;
            GmpAcknowledgementAbi::from(GmpAcknowledgement::success(result.to_vec())).abi_encode()
        }
        GmpEncoding::Protobuf => {
            let ack = if result.is_empty() {
                GmpAcknowledgement::protobuf_empty_success()
            } else {
                GmpAcknowledgement::success(result.to_vec())
            };
            ack.encode_to_vec()
        }
    }
}

/// Decodes raw bytes into a [`GmpAcknowledgement`].
///
/// # Errors
///
/// Returns [`GMPError::InvalidPacketData`] if `bytes` cannot be decoded in the given format.
pub fn decode_gmp_ack(bytes: &[u8], encoding: GmpEncoding) -> Result<GmpAcknowledgement> {
    match encoding {
        GmpEncoding::Abi => {
            use alloy_sol_types::SolValue;
            let abi = GmpAcknowledgementAbi::abi_decode(bytes)
                .map_err(|_| GMPError::InvalidPacketData)?;
            Ok(GmpAcknowledgement::success(abi.result.into()))
        }
        GmpEncoding::Protobuf => {
            GmpAcknowledgement::decode_vec(bytes).map_err(|_| GMPError::InvalidPacketData.into())
        }
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
        let encoded = encode_gmp_packet(original, GmpEncoding::Abi);
        let raw = decode_gmp_packet(&encoded, GmpEncoding::Abi).unwrap();
        assert_raw_matches_sample(&raw);

        let validated = GmpPacketData::try_from(raw).unwrap();
        assert_eq!(validated.sender.as_ref(), "solana_sender_pubkey");
    }

    #[test]
    fn protobuf_round_trip() {
        let original = sample_packet_data();
        let encoded = encode_gmp_packet(original, GmpEncoding::Protobuf);
        let raw = decode_gmp_packet(&encoded, GmpEncoding::Protobuf).unwrap();
        assert_raw_matches_sample(&raw);

        let validated = GmpPacketData::try_from(raw).unwrap();
        assert_eq!(validated.sender.as_ref(), "solana_sender_pubkey");
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
        let encoded = encode_gmp_ack(&data, GmpEncoding::Abi);
        assert!(!encoded.is_empty());
        let decoded = decode_gmp_ack(&encoded, GmpEncoding::Abi).unwrap();
        assert_eq!(decoded.result, data);
    }

    #[test]
    fn ack_abi_empty_result() {
        let encoded = encode_gmp_ack(&[], GmpEncoding::Abi);
        assert!(!encoded.is_empty());
        let decoded = decode_gmp_ack(&encoded, GmpEncoding::Abi).unwrap();
        assert!(decoded.result.is_empty());
    }

    #[test]
    fn ack_protobuf_round_trip() {
        let data = vec![1, 2, 3, 4];
        let encoded = encode_gmp_ack(&data, GmpEncoding::Protobuf);
        assert!(!encoded.is_empty());
        let decoded = decode_gmp_ack(&encoded, GmpEncoding::Protobuf).unwrap();
        assert_eq!(decoded.result, data);
    }

    #[test]
    fn ack_protobuf_empty_result_uses_sentinel() {
        let encoded = encode_gmp_ack(&[], GmpEncoding::Protobuf);
        assert!(!encoded.is_empty());
        let decoded = decode_gmp_ack(&encoded, GmpEncoding::Protobuf).unwrap();
        assert_eq!(decoded.result, vec![0]);
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
