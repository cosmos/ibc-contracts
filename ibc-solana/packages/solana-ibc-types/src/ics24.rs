// SPDX-License-Identifier: Apache-2.0

//! ICS24 commitment path and hash utilities

use crate::router::Packet;
use crate::Payload;
use anchor_lang::prelude::*;
use solana_ibc_constants::IBC_VERSION;
use solana_keccak_hasher::hash as keccak256;
use solana_sha256_hasher::{hash as sha256, hashv as sha256v};
use std::mem::size_of;

pub use solana_ibc_constants::UNIVERSAL_ERROR_ACK;

const HASH_OUTPUT_SIZE: usize = 32;

/// Error type for ICS24 operations
#[error_code]
pub enum Ics24Error {
    #[msg("No acknowledgements provided")]
    NoAcknowledgements,
    #[msg("Empty merkle prefix")]
    EmptyMerklePrefix,
    #[msg("Failed to serialize packet")]
    PacketSerializationError,
}

pub type Ics24Result<T> = core::result::Result<T, Ics24Error>;

/// Computes the commitment path for a packet.
/// Path format: `client_id || 0x01 || sequence (big-endian)`
#[must_use]
pub fn packet_commitment_path(client_id: &str, sequence: u64) -> Vec<u8> {
    let mut path = Vec::with_capacity(client_id.len() + size_of::<u8>() + size_of::<u64>());
    path.extend_from_slice(client_id.as_bytes());
    path.push(1u8);
    path.extend_from_slice(&sequence.to_be_bytes());
    path
}

/// Computes the commitment path for an acknowledgement.
/// Path format: `client_id || 0x03 || sequence (big-endian)`
#[must_use]
pub fn packet_acknowledgement_commitment_path(client_id: &str, sequence: u64) -> Vec<u8> {
    let mut path = Vec::with_capacity(client_id.len() + size_of::<u8>() + size_of::<u64>());
    path.extend_from_slice(client_id.as_bytes());
    path.push(3u8);
    path.extend_from_slice(&sequence.to_be_bytes());
    path
}

/// Computes the commitment path for a packet receipt.
/// Path format: `client_id || 0x02 || sequence (big-endian)`
#[must_use]
pub fn packet_receipt_commitment_path(client_id: &str, sequence: u64) -> Vec<u8> {
    let mut path = Vec::with_capacity(client_id.len() + size_of::<u8>() + size_of::<u64>());
    path.extend_from_slice(client_id.as_bytes());
    path.push(2u8);
    path.extend_from_slice(&sequence.to_be_bytes());
    path
}

/// Computes the keccak256 hash of the packet commitment path.
#[must_use]
pub fn packet_commitment_key(client_id: &str, sequence: u64) -> [u8; 32] {
    let path = packet_commitment_path(client_id, sequence);
    keccak256(&path).to_bytes()
}

/// Computes the keccak256 hash of the acknowledgement commitment path.
#[must_use]
pub fn packet_acknowledgement_commitment_key(client_id: &str, sequence: u64) -> [u8; 32] {
    let path = packet_acknowledgement_commitment_path(client_id, sequence);
    keccak256(&path).to_bytes()
}

/// Computes the keccak256 hash of the packet receipt commitment path.
#[must_use]
pub fn packet_receipt_commitment_key(client_id: &str, sequence: u64) -> [u8; 32] {
    let path = packet_receipt_commitment_path(client_id, sequence);
    keccak256(&path).to_bytes()
}

/// Computes the packet commitment hash.
/// Format: `sha256(0x02 || sha256(destClient) || sha256(timeout) || sha256(payloads))`
#[must_use]
pub fn packet_commitment_bytes32(packet: &Packet) -> [u8; 32] {
    let mut app_bytes = Vec::with_capacity(packet.payloads.len() * HASH_OUTPUT_SIZE);

    for payload in &packet.payloads {
        let payload_hash = hash_payload(payload);
        app_bytes.extend_from_slice(&payload_hash);
    }

    let dest_client_hash = sha256(packet.dest_client.as_bytes()).to_bytes();
    let timeout_hash = sha256(&packet.timeout_timestamp.to_be_bytes()).to_bytes();
    let app_hash = sha256(&app_bytes).to_bytes();
    sha256v(&[&[IBC_VERSION], &dest_client_hash, &timeout_hash, &app_hash]).to_bytes()
}

/// Computes the hash of a payload.
fn hash_payload(payload: &Payload) -> [u8; 32] {
    let mut buf = Vec::with_capacity(5 * HASH_OUTPUT_SIZE);
    buf.extend_from_slice(&sha256(payload.source_port.as_bytes()).to_bytes());
    buf.extend_from_slice(&sha256(payload.dest_port.as_bytes()).to_bytes());
    buf.extend_from_slice(&sha256(payload.version.as_bytes()).to_bytes());
    buf.extend_from_slice(&sha256(payload.encoding.as_bytes()).to_bytes());
    buf.extend_from_slice(&sha256(&payload.value).to_bytes());

    sha256(&buf).to_bytes()
}

/// Computes the acknowledgement commitment hash.
/// Format: `sha256(0x02 || sha256(ack1) || sha256(ack2) || ...)`
///
/// # Errors
/// Returns `Ics24Error::NoAcknowledgements` if the acks slice is empty.
pub fn packet_acknowledgement_commitment_bytes32(acks: &[Vec<u8>]) -> Ics24Result<[u8; 32]> {
    if acks.is_empty() {
        return Err(Ics24Error::NoAcknowledgements);
    }

    let mut ack_bytes = Vec::with_capacity(acks.len() * HASH_OUTPUT_SIZE);
    for ack in acks {
        ack_bytes.extend_from_slice(&sha256(ack).to_bytes());
    }

    Ok(sha256v(&[&[IBC_VERSION], &ack_bytes]).to_bytes())
}

/// Computes the packet receipt commitment hash (keccak256 of serialized packet).
///
/// # Errors
/// Returns `Ics24Error::PacketSerializationError` if the packet cannot be serialized.
pub fn packet_receipt_commitment_bytes32(packet: &Packet) -> Ics24Result<[u8; 32]> {
    let packet_bytes = packet
        .try_to_vec()
        .map_err(|_| Ics24Error::PacketSerializationError)?;
    Ok(keccak256(&packet_bytes).to_bytes())
}

/// Appends a path to the last element of a merkle prefix.
///
/// # Errors
/// Returns `Ics24Error::EmptyMerklePrefix` if the merkle prefix is empty.
pub fn prefixed_path(merkle_prefix: &[Vec<u8>], path: &[u8]) -> Ics24Result<Vec<Vec<u8>>> {
    if merkle_prefix.is_empty() {
        return Err(Ics24Error::EmptyMerklePrefix);
    }

    let mut result = merkle_prefix.to_vec();
    let last_idx = result.len() - 1;
    result[last_idx].extend_from_slice(path);

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_commitment_path() {
        let client_id = "test-client";
        let sequence = 42u64;
        let path = packet_commitment_path(client_id, sequence);

        let expected = [client_id.as_bytes(), &[1u8], &sequence.to_be_bytes()].concat();

        assert_eq!(path, expected);
    }

    #[test]
    fn test_packet_acknowledgement_commitment_path() {
        let client_id = "test-client";
        let sequence = 42u64;
        let path = packet_acknowledgement_commitment_path(client_id, sequence);

        let expected = [client_id.as_bytes(), &[3u8], &sequence.to_be_bytes()].concat();

        assert_eq!(path, expected);
    }

    #[test]
    fn test_packet_receipt_commitment_path() {
        let client_id = "test-client";
        let sequence = 42u64;
        let path = packet_receipt_commitment_path(client_id, sequence);

        let expected = [client_id.as_bytes(), &[2u8], &sequence.to_be_bytes()].concat();

        assert_eq!(path, expected);
    }

    #[test]
    fn test_packet_commitment_key() {
        let client_id = "test-client";
        let sequence = 42u64;
        let key = packet_commitment_key(client_id, sequence);

        assert_eq!(key.len(), 32);

        let key2 = packet_commitment_key(client_id, sequence);
        assert_eq!(key, key2);

        let key3 = packet_commitment_key("different-client", sequence);
        assert_ne!(key, key3);
    }

    fn test_packet() -> Packet {
        Packet {
            source_client: "07-tendermint-0".to_string(),
            dest_client: "08-wasm-0".to_string(),
            sequence: 1,
            timeout_timestamp: 1_700_000_000,
            payloads: vec![Payload {
                source_port: "transfer".to_string(),
                dest_port: "transfer".to_string(),
                version: "ics20-1".to_string(),
                encoding: "application/json".to_string(),
                value: b"{\"amount\":\"100\",\"denom\":\"uatom\"}".to_vec(),
            }],
        }
    }

    #[test]
    fn test_packet_commitment_bytes32() {
        let packet = test_packet();

        let commitment = packet_commitment_bytes32(&packet);
        assert_eq!(commitment.len(), 32);

        let commitment2 = packet_commitment_bytes32(&packet);
        assert_eq!(commitment, commitment2);

        let mut packet_different_timeout = packet;
        packet_different_timeout.timeout_timestamp = 1_800_000_000;
        let commitment3 = packet_commitment_bytes32(&packet_different_timeout);
        assert_ne!(commitment, commitment3);
    }

    #[test]
    fn test_packet_commitment_bytes32_multiple_payloads() {
        let transfer_payload = Payload {
            source_port: "transfer".to_string(),
            dest_port: "transfer".to_string(),
            version: "ics20-1".to_string(),
            encoding: "application/json".to_string(),
            value: b"{\"amount\":\"100\"}".to_vec(),
        };
        let gmp_payload = Payload {
            source_port: "gmp".to_string(),
            dest_port: "gmp".to_string(),
            version: "gmp-1".to_string(),
            encoding: "application/octet-stream".to_string(),
            value: b"callback_data".to_vec(),
        };

        let packet = Packet {
            source_client: "07-tendermint-0".to_string(),
            dest_client: "08-wasm-0".to_string(),
            sequence: 1,
            timeout_timestamp: 1_700_000_000,
            payloads: vec![transfer_payload.clone(), gmp_payload.clone()],
        };

        let commitment = packet_commitment_bytes32(&packet);
        assert_eq!(commitment.len(), 32);

        let packet_reversed = Packet {
            payloads: vec![gmp_payload, transfer_payload],
            ..packet
        };
        let commitment_reversed = packet_commitment_bytes32(&packet_reversed);
        assert_ne!(commitment, commitment_reversed);
    }

    #[test]
    fn test_hash_payload() {
        let payload = Payload {
            source_port: "transfer".to_string(),
            dest_port: "transfer".to_string(),
            version: "ics20-1".to_string(),
            encoding: "application/json".to_string(),
            value: b"{\"amount\":\"100\",\"denom\":\"uatom\"}".to_vec(),
        };

        let hash = hash_payload(&payload);
        assert_eq!(hash.len(), 32);

        let hash2 = hash_payload(&payload);
        assert_eq!(hash, hash2);

        let mut payload_different_value = payload;
        payload_different_value.value = b"{\"amount\":\"200\",\"denom\":\"uatom\"}".to_vec();
        let hash3 = hash_payload(&payload_different_value);
        assert_ne!(hash, hash3);
    }

    #[test]
    fn test_packet_acknowledgement_commitment_bytes32() {
        let ack1 = b"success".to_vec();
        let ack2 = b"transfer_complete".to_vec();
        let acks = vec![ack1.clone(), ack2.clone()];

        let commitment = packet_acknowledgement_commitment_bytes32(&acks).unwrap();
        assert_eq!(commitment.len(), 32);

        let commitment2 = packet_acknowledgement_commitment_bytes32(&acks).unwrap();
        assert_eq!(commitment, commitment2);

        let acks_reversed = vec![ack2, ack1];
        let commitment3 = packet_acknowledgement_commitment_bytes32(&acks_reversed).unwrap();
        assert_ne!(commitment, commitment3);
    }

    #[test]
    fn test_packet_acknowledgement_commitment_bytes32_empty() {
        let acks: Vec<Vec<u8>> = vec![];
        let result = packet_acknowledgement_commitment_bytes32(&acks);
        assert!(matches!(result, Err(Ics24Error::NoAcknowledgements)));
    }

    #[test]
    fn test_packet_acknowledgement_commitment_bytes32_single() {
        let acks = vec![b"success".to_vec()];
        let commitment = packet_acknowledgement_commitment_bytes32(&acks).unwrap();
        assert_eq!(commitment.len(), 32);
    }

    #[test]
    fn test_packet_receipt_commitment_bytes32() {
        let packet = test_packet();

        let commitment = packet_receipt_commitment_bytes32(&packet).unwrap();
        assert_eq!(commitment.len(), 32);

        let commitment2 = packet_receipt_commitment_bytes32(&packet).unwrap();
        assert_eq!(commitment, commitment2);

        let mut packet_different_sequence = packet;
        packet_different_sequence.sequence = 2;
        let commitment3 = packet_receipt_commitment_bytes32(&packet_different_sequence).unwrap();
        assert_ne!(commitment, commitment3);
    }

    #[test]
    fn test_prefixed_path() {
        let prefix_part1 = b"ibc".to_vec();
        let prefix_part2 = b"commitments/".to_vec();
        let merkle_prefix = vec![prefix_part1.clone(), prefix_part2];
        let path = b"packet/1";

        let result = prefixed_path(&merkle_prefix, path).unwrap();

        assert_eq!(result[0], prefix_part1);
        assert_eq!(result[1], b"commitments/packet/1");
    }

    #[test]
    fn test_prefixed_path_empty_prefix() {
        let merkle_prefix: Vec<Vec<u8>> = vec![];
        let path = b"packet/1";

        let result = prefixed_path(&merkle_prefix, path);
        assert!(matches!(result, Err(Ics24Error::EmptyMerklePrefix)));
    }

    #[test]
    fn test_prefixed_path_single_prefix() {
        let merkle_prefix = vec![b"ibc/".to_vec()];
        let path = b"commitments/packet/1";

        let result = prefixed_path(&merkle_prefix, path).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], b"ibc/commitments/packet/1");
    }

    #[test]
    fn test_universal_error_ack_is_sha256_of_string() {
        let computed = sha256(b"UNIVERSAL_ERROR_ACKNOWLEDGEMENT").to_bytes();

        assert_eq!(
            UNIVERSAL_ERROR_ACK, computed,
            "UNIVERSAL_ERROR_ACK must be sha256(\"UNIVERSAL_ERROR_ACKNOWLEDGEMENT\")"
        );
    }

    #[test]
    fn test_path_consistency() {
        let client_id = "test-client";
        let sequence = 100u64;

        let commit_path = packet_commitment_path(client_id, sequence);
        let ack_path = packet_acknowledgement_commitment_path(client_id, sequence);
        let receipt_path = packet_receipt_commitment_path(client_id, sequence);

        let expected_commit = [client_id.as_bytes(), &[1u8], &sequence.to_be_bytes()].concat();
        let expected_receipt = [client_id.as_bytes(), &[2u8], &sequence.to_be_bytes()].concat();
        let expected_ack = [client_id.as_bytes(), &[3u8], &sequence.to_be_bytes()].concat();

        assert_eq!(commit_path, expected_commit);
        assert_eq!(receipt_path, expected_receipt);
        assert_eq!(ack_path, expected_ack);
    }
}
