// SPDX-License-Identifier: Apache-2.0

use anchor_lang::prelude::*;

use crate::error::ErrorCode;

use solana_secp256k1_recover::{SECP256K1_PUBLIC_KEY_LENGTH, SECP256K1_SIGNATURE_LENGTH};
use solana_sha256_hasher::hash as sha256;

const SIGNATURE_LEN: usize = SECP256K1_SIGNATURE_LENGTH + 1;
const ETH_RECOVERY_ID_OFFSET: u8 = 27;
pub const ETH_ADDRESS_LEN: usize = 20;

pub type MessageHash = [u8; 32];

/// Length of the domain-separated signing preimage: 1-byte type tag + 32-byte SHA-256 hash.
pub(crate) const DOMAIN_SEPARATED_PREIMAGE_LEN: usize = 1 + 32;

/// Distinguishes attestation types to prevent cross-protocol signature replay.
#[derive(Clone, Copy)]
#[repr(u8)]
pub enum AttestationType {
    State = 0x01,
    Packet = 0x02,
}

pub fn sha256_digest(data: &[u8]) -> MessageHash {
    sha256(data).to_bytes()
}

/// Compute the prehash for signature verification with domain separation:
/// `sha256(type_tag || sha256(data))`
pub fn tagged_signing_input(data: &[u8], attestation_type: AttestationType) -> MessageHash {
    let inner_hash = sha256(data).to_bytes();
    let mut tagged = Vec::with_capacity(DOMAIN_SEPARATED_PREIMAGE_LEN);
    tagged.push(attestation_type as u8);
    tagged.extend_from_slice(&inner_hash);
    sha256(&tagged).to_bytes()
}

struct ParsedSignature {
    sig_bytes: [u8; SECP256K1_SIGNATURE_LENGTH],
    recovery_id: u8,
}

/// Parse a 65-byte Ethereum signature (r || s || v) into its components,
/// normalizing the recovery id from Ethereum-style (27/28) to canonical (0/1).
fn parse_eth_signature(signature: &[u8]) -> Result<ParsedSignature> {
    // Expect exactly 65 bytes: 32 (r) + 32 (s) + 1 (v)
    if signature.len() != SIGNATURE_LEN {
        return Err(error!(ErrorCode::InvalidSignature));
    }

    // Extract the 64-byte (r, s) component
    let sig_bytes = signature[..SECP256K1_SIGNATURE_LENGTH].try_into().unwrap();

    // Normalize Ethereum-style v (27/28) to canonical recovery id (0/1)
    let raw_v = signature[SECP256K1_SIGNATURE_LENGTH];
    let recovery_id = raw_v.saturating_sub(if raw_v >= ETH_RECOVERY_ID_OFFSET {
        ETH_RECOVERY_ID_OFFSET
    } else {
        0
    });

    Ok(ParsedSignature {
        sig_bytes,
        recovery_id,
    })
}

/// Recover Ethereum address from a precomputed message hash and a 65-byte secp256k1 signature.
#[cfg(target_os = "solana")]
pub fn recover_eth_address(
    message_hash: &MessageHash,
    signature: &[u8],
) -> Result<[u8; ETH_ADDRESS_LEN]> {
    let parsed_eth_signature = parse_eth_signature(signature)?;

    let pubkey = solana_secp256k1_recover::secp256k1_recover(
        message_hash,
        parsed_eth_signature.recovery_id,
        &parsed_eth_signature.sig_bytes,
    )
    .map_err(|_| error!(ErrorCode::InvalidSignature))?;

    Ok(pubkey_to_eth_address(&pubkey.0))
}

/// Recover Ethereum address from a precomputed message hash and a 65-byte secp256k1 signature (test impl).
#[cfg(all(not(target_os = "solana"), test))]
pub fn recover_eth_address(
    message_hash: &MessageHash,
    signature: &[u8],
) -> Result<[u8; ETH_ADDRESS_LEN]> {
    use alloy_primitives::Signature;

    let parsed_eth_signature = parse_eth_signature(signature)?;

    if parsed_eth_signature.recovery_id > 1 {
        return Err(error!(ErrorCode::InvalidSignature));
    }

    let sig = Signature::new(
        alloy_primitives::U256::from_be_slice(
            &parsed_eth_signature.sig_bytes[..SECP256K1_SIGNATURE_LENGTH / 2],
        ),
        alloy_primitives::U256::from_be_slice(
            &parsed_eth_signature.sig_bytes[SECP256K1_SIGNATURE_LENGTH / 2..],
        ),
        parsed_eth_signature.recovery_id != 0,
    );

    let address = sig
        .recover_address_from_prehash(&alloy_primitives::B256::from(*message_hash))
        .map_err(|_| error!(ErrorCode::InvalidSignature))?;

    Ok(address.0 .0)
}

#[cfg(all(not(target_os = "solana"), not(test)))]
pub fn recover_eth_address(
    _message_hash: &MessageHash,
    signature: &[u8],
) -> Result<[u8; ETH_ADDRESS_LEN]> {
    if signature.len() != SIGNATURE_LEN {
        return Err(error!(ErrorCode::InvalidSignature));
    }
    Err(error!(ErrorCode::InvalidSignature))
}

fn pubkey_to_eth_address(pubkey: &[u8; SECP256K1_PUBLIC_KEY_LENGTH]) -> [u8; ETH_ADDRESS_LEN] {
    let solana_keccak_hasher::Hash(hash) = solana_keccak_hasher::hash(pubkey);
    hash[hash.len() - ETH_ADDRESS_LEN..].try_into().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;

    #[rstest]
    #[case::short(vec![0u8; 64])]
    #[case::long(vec![0u8; 66])]
    #[case::empty(vec![])]
    fn test_recover_eth_address_invalid_signature_length(#[case] sig: Vec<u8>) {
        let hash = sha256_digest(b"test message");
        assert!(recover_eth_address(&hash, &sig).is_err());
    }

    #[test]
    fn test_recover_eth_address_recovery_id_normalization() {
        use crate::test_helpers::signing::TestAttestor;

        let attestor = TestAttestor::new(1);
        let message = b"test message for recovery id normalization";
        let hash = tagged_signing_input(message, AttestationType::State);
        let sig = attestor.sign(message, AttestationType::State);

        // Original signature uses Ethereum-style v (27 or 28)
        let original_v = sig[64];
        assert!(original_v == 27 || original_v == 28);

        // Recover with original Ethereum-style v
        let addr_original = recover_eth_address(&hash, &sig).unwrap();

        // Recover with canonical v (0 or 1)
        let mut sig_canonical = sig;
        sig_canonical[64] = original_v - 27;
        let addr_canonical = recover_eth_address(&hash, &sig_canonical).unwrap();

        // Both should recover the same address
        assert_eq!(addr_original, addr_canonical);
        assert_eq!(addr_original, attestor.eth_address);
    }

    #[test]
    fn test_recover_eth_address_invalid_recovery_id() {
        use crate::test_helpers::signing::TestAttestor;

        let attestor = TestAttestor::new(1);
        let message = b"test message for invalid recovery id";
        let hash = tagged_signing_input(message, AttestationType::State);
        let mut sig = attestor.sign(message, AttestationType::State);

        // v=29 normalizes to 2, which is invalid for secp256k1
        sig[64] = 29;
        assert!(recover_eth_address(&hash, &sig).is_err());
    }
}
