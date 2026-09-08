// SPDX-License-Identifier: Apache-2.0

//! ABI encoding and decoding for GMP Solana payload using `alloy-sol-types`.

use alloy_sol_types::SolValue;
use anchor_lang::prelude::*;

use crate::errors::GMPError;
pub use crate::sol_types::ISolanaGMPMsgs::GMPSolanaPayload as GmpSolanaPayloadAbi;
use solana_ibc_proto::{RawGmpSolanaPayload, RawSolanaAccountMeta};

/// Size of a packed account entry: pubkey(32) + `is_signer`(1) + `is_writable`(1)
const PACKED_ACCOUNT_SIZE: usize = 34;

const fn parse_bool_byte(byte: u8) -> std::result::Result<bool, GMPError> {
    match byte {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(GMPError::InvalidAbiEncoding),
    }
}

impl TryFrom<GmpSolanaPayloadAbi> for RawGmpSolanaPayload {
    type Error = GMPError;

    fn try_from(abi: GmpSolanaPayloadAbi) -> std::result::Result<Self, Self::Error> {
        let chunks = abi.packedAccounts.chunks_exact(PACKED_ACCOUNT_SIZE);
        if !chunks.remainder().is_empty() {
            return Err(GMPError::InvalidAbiEncoding);
        }
        let accounts = chunks
            .map(|chunk| {
                Ok(RawSolanaAccountMeta {
                    pubkey: chunk[..32].to_vec(),
                    is_signer: parse_bool_byte(chunk[32])?,
                    is_writable: parse_bool_byte(chunk[33])?,
                })
            })
            .collect::<std::result::Result<Vec<_>, GMPError>>()?;
        Ok(Self {
            accounts,
            data: abi.instructionData.into(),
            prefund_lamports: abi.prefundLamports,
        })
    }
}

/// Decode ABI-encoded bytes into a raw (unvalidated) [`RawGmpSolanaPayload`].
///
/// Performs structural decoding only (ABI envelope + packed-account splitting).
/// The caller is responsible for converting to the validated [`GmpSolanaPayload`]
/// via `TryFrom`.
pub fn decode_abi_to_raw(data: &[u8]) -> Result<RawGmpSolanaPayload> {
    let decoded =
        GmpSolanaPayloadAbi::abi_decode(data).map_err(|_| error!(GMPError::InvalidAbiEncoding))?;
    decoded.try_into().map_err(|e: GMPError| error!(e))
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_ibc_proto::MAX_GMP_SOLANA_PAYLOAD_ACCOUNTS;

    fn assert_anchor_error(result: Result<RawGmpSolanaPayload>, expected: GMPError) {
        let err = result.unwrap_err();
        match err {
            anchor_lang::error::Error::AnchorError(anchor_err) => {
                assert_eq!(
                    anchor_err.error_code_number,
                    u32::from(expected),
                    "expected {expected:?}, got {}",
                    anchor_err.error_name
                );
            }
            other @ anchor_lang::error::Error::ProgramError(_) => {
                panic!("expected AnchorError, got {other:?}")
            }
        }
    }

    #[test]
    fn test_decode_abi_to_raw_roundtrip() {
        let pubkey1 = Pubkey::new_unique();
        let pubkey2 = Pubkey::new_unique();

        let mut packed = Vec::new();
        packed.extend_from_slice(&pubkey1.to_bytes());
        packed.push(1); // is_signer
        packed.push(0); // is_writable
        packed.extend_from_slice(&pubkey2.to_bytes());
        packed.push(0); // is_signer
        packed.push(1); // is_writable

        let instr_data = vec![0xAA, 0xBB, 0xCC, 0xDD];

        let encoded = GmpSolanaPayloadAbi {
            packedAccounts: packed.into(),
            instructionData: instr_data.clone().into(),
            prefundLamports: 8,
        }
        .abi_encode();

        let raw = decode_abi_to_raw(&encoded).unwrap();

        assert_eq!(raw.accounts.len(), 2);
        assert_eq!(raw.accounts[0].pubkey, pubkey1.to_bytes());
        assert!(raw.accounts[0].is_signer);
        assert!(!raw.accounts[0].is_writable);
        assert_eq!(raw.accounts[1].pubkey, pubkey2.to_bytes());
        assert!(!raw.accounts[1].is_signer);
        assert!(raw.accounts[1].is_writable);
        assert_eq!(raw.data, instr_data);
        assert_eq!(raw.prefund_lamports, 8);
    }

    #[test]
    fn test_decode_abi_to_raw_empty_accounts() {
        let instr_data = vec![1, 2, 3];
        let encoded = GmpSolanaPayloadAbi {
            packedAccounts: Vec::new().into(),
            instructionData: instr_data.clone().into(),
            prefundLamports: 0,
        }
        .abi_encode();

        let raw = decode_abi_to_raw(&encoded).unwrap();

        assert!(raw.accounts.is_empty());
        assert_eq!(raw.data, instr_data);
        assert_eq!(raw.prefund_lamports, 0);
    }

    #[test]
    fn test_decode_abi_to_raw_too_short() {
        let data = vec![0u8; 95]; // less than 3 words
        assert!(decode_abi_to_raw(&data).is_err());
    }

    #[test]
    fn test_decode_abi_to_raw_misaligned_accounts() {
        let bad_packed = vec![0u8; 35]; // not a multiple of 34
        let encoded = GmpSolanaPayloadAbi {
            packedAccounts: bad_packed.into(),
            instructionData: vec![1].into(),
            prefundLamports: 0,
        }
        .abi_encode();

        assert_anchor_error(decode_abi_to_raw(&encoded), GMPError::InvalidAbiEncoding);
    }

    #[test]
    fn test_decode_abi_to_raw_empty_instruction_data() {
        let encoded = GmpSolanaPayloadAbi {
            packedAccounts: Vec::new().into(),
            instructionData: Vec::new().into(),
            prefundLamports: 0,
        }
        .abi_encode();

        // Raw decoding succeeds — empty data is rejected during validation
        let raw = decode_abi_to_raw(&encoded).unwrap();
        assert!(raw.data.is_empty());
    }

    #[test]
    fn test_decode_abi_to_raw_invalid_is_signer_byte() {
        let pubkey = Pubkey::new_unique();
        let mut packed = Vec::new();
        packed.extend_from_slice(&pubkey.to_bytes());
        packed.push(2); // invalid is_signer
        packed.push(0);

        let encoded = GmpSolanaPayloadAbi {
            packedAccounts: packed.into(),
            instructionData: vec![1].into(),
            prefundLamports: 0,
        }
        .abi_encode();

        assert_anchor_error(decode_abi_to_raw(&encoded), GMPError::InvalidAbiEncoding);
    }

    #[test]
    fn test_decode_abi_to_raw_invalid_is_writable_byte() {
        let pubkey = Pubkey::new_unique();
        let mut packed = Vec::new();
        packed.extend_from_slice(&pubkey.to_bytes());
        packed.push(0);
        packed.push(3); // invalid is_writable

        let encoded = GmpSolanaPayloadAbi {
            packedAccounts: packed.into(),
            instructionData: vec![1].into(),
            prefundLamports: 0,
        }
        .abi_encode();

        assert_anchor_error(decode_abi_to_raw(&encoded), GMPError::InvalidAbiEncoding);
    }

    #[test]
    fn test_decode_abi_to_raw_many_accounts() {
        // Raw decoding succeeds even with MAX+1 accounts — count is checked during validation
        let packed: Vec<u8> = (0..=MAX_GMP_SOLANA_PAYLOAD_ACCOUNTS)
            .flat_map(|_| {
                let mut entry = Pubkey::new_unique().to_bytes().to_vec();
                entry.push(0); // is_signer
                entry.push(0); // is_writable
                entry
            })
            .collect();

        let encoded = GmpSolanaPayloadAbi {
            packedAccounts: packed.into(),
            instructionData: vec![1].into(),
            prefundLamports: 0,
        }
        .abi_encode();

        let raw = decode_abi_to_raw(&encoded).unwrap();
        assert_eq!(raw.accounts.len(), MAX_GMP_SOLANA_PAYLOAD_ACCOUNTS + 1);
    }

    #[test]
    fn test_decode_abi_to_raw_max_accounts() {
        let packed: Vec<u8> = (0..MAX_GMP_SOLANA_PAYLOAD_ACCOUNTS)
            .flat_map(|_| {
                let mut entry = Pubkey::new_unique().to_bytes().to_vec();
                entry.push(0);
                entry.push(1);
                entry
            })
            .collect();

        let encoded = GmpSolanaPayloadAbi {
            packedAccounts: packed.into(),
            instructionData: vec![1].into(),
            prefundLamports: 0,
        }
        .abi_encode();

        let raw = decode_abi_to_raw(&encoded).unwrap();
        assert_eq!(raw.accounts.len(), MAX_GMP_SOLANA_PAYLOAD_ACCOUNTS);
    }
}
