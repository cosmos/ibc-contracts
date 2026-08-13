// SPDX-License-Identifier: Apache-2.0

//! Decoding `GmpSolanaPayload` from ABI or protobuf encoding.

use anchor_lang::prelude::*;
use solana_ibc_proto::ProstMessage;

use crate::errors::GMPError;

/// Decode raw (unvalidated) `RawGmpSolanaPayload` from either protobuf or ABI encoding.
///
/// Callers should validate the result into `GmpSolanaPayload` via `try_into()`.
pub fn decode_raw(value: &[u8], encoding: &str) -> Result<solana_ibc_proto::RawGmpSolanaPayload> {
    match encoding {
        crate::constants::ICS27_ENCODING_ABI => crate::abi::decode_abi_to_raw(value).map_err(|e| {
            msg!("GMP ABI Solana payload decode failed: {}", e);
            error!(GMPError::InvalidSolanaPayload)
        }),
        crate::constants::ICS27_ENCODING_PROTOBUF => {
            <solana_ibc_proto::RawGmpSolanaPayload as ProstMessage>::decode(value).map_err(|e| {
                msg!("GMP Solana payload decode failed: {}", e);
                error!(GMPError::InvalidSolanaPayload)
            })
        }
        _ => Err(error!(GMPError::InvalidEncoding)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::abi::GmpSolanaPayloadAbi;
    use crate::proto::GmpSolanaPayload;
    use solana_ibc_proto::ProstMessage;

    #[test]
    fn test_decode_protobuf() {
        let pubkey = Pubkey::new_unique();
        let raw = solana_ibc_proto::RawGmpSolanaPayload {
            accounts: vec![solana_ibc_proto::RawSolanaAccountMeta {
                pubkey: pubkey.to_bytes().to_vec(),
                is_signer: true,
                is_writable: false,
            }],
            data: vec![0xAA, 0xBB],
            prefund_lamports: 0,
        };
        let encoded = raw.encode_to_vec();

        let decoded: GmpSolanaPayload =
            decode_raw(&encoded, crate::constants::ICS27_ENCODING_PROTOBUF)
                .expect("protobuf decode_raw should succeed")
                .try_into()
                .expect("protobuf validation should succeed");

        assert_eq!(decoded.accounts.len(), 1);
        assert_eq!(decoded.accounts[0].pubkey, pubkey);
        assert!(decoded.accounts[0].is_signer);
        assert!(!decoded.accounts[0].is_writable);
        assert_eq!(decoded.data, vec![0xAA, 0xBB]);
        assert_eq!(decoded.prefund_lamports, 0);
    }

    #[test]
    fn test_decode_abi() {
        use alloy_sol_types::SolValue;

        let pubkey = Pubkey::new_unique();
        let mut packed = Vec::new();
        packed.extend_from_slice(&pubkey.to_bytes());
        packed.push(1); // is_signer
        packed.push(0); // is_writable

        let encoded = GmpSolanaPayloadAbi {
            packedAccounts: packed.into(),
            instructionData: vec![0xCC, 0xDD].into(),
            prefundLamports: 0,
        }
        .abi_encode();

        let decoded: GmpSolanaPayload = decode_raw(&encoded, crate::constants::ICS27_ENCODING_ABI)
            .expect("ABI decode_raw should succeed")
            .try_into()
            .expect("ABI validation should succeed");

        assert_eq!(decoded.accounts.len(), 1);
        assert_eq!(decoded.accounts[0].pubkey, pubkey);
        assert!(decoded.accounts[0].is_signer);
        assert!(!decoded.accounts[0].is_writable);
        assert_eq!(decoded.data, vec![0xCC, 0xDD]);
        assert_eq!(decoded.prefund_lamports, 0);
    }

    #[test]
    fn test_decode_raw_invalid_encoding() {
        let result = decode_raw(&[1, 2, 3], "application/json");
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_raw_abi_invalid_bytes() {
        let result = decode_raw(&[0xFF; 10], crate::constants::ICS27_ENCODING_ABI);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_protobuf_empty_data_rejected() {
        let raw = solana_ibc_proto::RawGmpSolanaPayload {
            accounts: vec![],
            data: vec![],
            prefund_lamports: 0,
        };
        let encoded = raw.encode_to_vec();

        // decode_raw succeeds, but validation rejects empty data
        let raw_decoded = decode_raw(&encoded, crate::constants::ICS27_ENCODING_PROTOBUF)
            .expect("protobuf decode_raw should succeed");
        let result: std::result::Result<GmpSolanaPayload, _> = raw_decoded.try_into();
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_abi_empty_data_rejected() {
        use alloy_sol_types::SolValue;

        let encoded = GmpSolanaPayloadAbi {
            packedAccounts: Vec::new().into(),
            instructionData: Vec::new().into(),
            prefundLamports: 0,
        }
        .abi_encode();

        // decode_raw succeeds, but validation rejects empty data
        let raw_decoded = decode_raw(&encoded, crate::constants::ICS27_ENCODING_ABI)
            .expect("ABI decode_raw should succeed");
        let result: std::result::Result<GmpSolanaPayload, _> = raw_decoded.try_into();
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_abi_too_many_accounts_rejected() {
        use alloy_sol_types::SolValue;
        use solana_ibc_proto::MAX_GMP_SOLANA_PAYLOAD_ACCOUNTS;

        let packed: Vec<u8> = (0..=MAX_GMP_SOLANA_PAYLOAD_ACCOUNTS)
            .flat_map(|_| {
                let mut entry = Pubkey::new_unique().to_bytes().to_vec();
                entry.push(0);
                entry.push(0);
                entry
            })
            .collect();

        let encoded = GmpSolanaPayloadAbi {
            packedAccounts: packed.into(),
            instructionData: vec![1].into(),
            prefundLamports: 0,
        }
        .abi_encode();

        // decode_raw succeeds, but validation rejects too many accounts
        let raw_decoded = decode_raw(&encoded, crate::constants::ICS27_ENCODING_ABI)
            .expect("ABI decode_raw should succeed");
        let result: std::result::Result<GmpSolanaPayload, _> = raw_decoded.try_into();
        assert!(result.is_err());
    }
}
