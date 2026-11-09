use alloc::{vec, vec::Vec};

use alloy_primitives::Bytes;
use alloy_sol_types::SolValue;
use besu_zk_types::{
    keccak256, ACKNOWLEDGEMENT_PATH_TYPE, IBC_PATH_ELEMENTS, PACKET_PATH_TYPE, RECEIPT_PATH_TYPE,
};

use crate::IbcStorageError;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PathKind {
    Packet,
    Receipt,
    Acknowledgement,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TerminalPath<'a> {
    pub client_id: &'a [u8],
    pub kind: PathKind,
    pub sequence: u64,
    pub raw: &'a [u8],
}

pub fn parse_membership_path(path: &[Vec<u8>]) -> Result<TerminalPath<'_>, IbcStorageError> {
    let parsed = parse(path)?;
    match parsed.kind {
        PathKind::Packet | PathKind::Acknowledgement => Ok(parsed),
        PathKind::Receipt => Err(IbcStorageError::WrongPathType(RECEIPT_PATH_TYPE)),
    }
}

pub fn parse_receipt_path(path: &[Vec<u8>]) -> Result<TerminalPath<'_>, IbcStorageError> {
    let parsed = parse(path)?;
    if parsed.kind == PathKind::Receipt {
        Ok(parsed)
    } else {
        let value = match parsed.kind {
            PathKind::Packet => PACKET_PATH_TYPE,
            PathKind::Receipt => RECEIPT_PATH_TYPE,
            PathKind::Acknowledgement => ACKNOWLEDGEMENT_PATH_TYPE,
        };
        Err(IbcStorageError::WrongPathType(value))
    }
}

fn parse(path: &[Vec<u8>]) -> Result<TerminalPath<'_>, IbcStorageError> {
    if path.len() != IBC_PATH_ELEMENTS {
        return Err(IbcStorageError::WrongPathElementCount(path.len()));
    }
    let raw = path
        .first()
        .ok_or(IbcStorageError::WrongPathElementCount(0))?;
    if raw.len() < 10 {
        return Err(IbcStorageError::TerminalPathTooShort);
    }
    let type_index = raw
        .len()
        .checked_sub(9)
        .ok_or(IbcStorageError::TerminalPathTooShort)?;
    let (client_id, suffix) = raw
        .split_at_checked(type_index)
        .ok_or(IbcStorageError::TerminalPathTooShort)?;
    if !valid_client_id(client_id) {
        return Err(IbcStorageError::InvalidClientIdentifier);
    }
    let (path_type, sequence) = suffix
        .split_first()
        .ok_or(IbcStorageError::TerminalPathTooShort)?;
    let path_type = *path_type;
    let kind = match path_type {
        PACKET_PATH_TYPE => PathKind::Packet,
        RECEIPT_PATH_TYPE => PathKind::Receipt,
        ACKNOWLEDGEMENT_PATH_TYPE => PathKind::Acknowledgement,
        _ => return Err(IbcStorageError::WrongPathType(path_type)),
    };
    let sequence_bytes: [u8; 8] = sequence
        .try_into()
        .map_err(|_| IbcStorageError::TerminalPathTooShort)?;
    Ok(TerminalPath {
        client_id,
        kind,
        sequence: u64::from_be_bytes(sequence_bytes),
        raw,
    })
}

fn valid_client_id(identifier: &[u8]) -> bool {
    if let Some(sequence) = identifier.strip_prefix(b"client-") {
        return !sequence.is_empty()
            && sequence.iter().all(u8::is_ascii_digit)
            && (sequence.len() == 1 || sequence.first() != Some(&b'0'));
    }
    if !(4..=128).contains(&identifier.len())
        || identifier.starts_with(b"channel-")
        || identifier.starts_with(b"client-")
    {
        return false;
    }
    identifier.iter().all(|byte| {
        byte.is_ascii_alphanumeric()
            || matches!(
                *byte,
                b'.' | b'_' | b'+' | b'-' | b'#' | b'[' | b']' | b'<' | b'>'
            )
    })
}

#[must_use]
pub fn path_digest(path: &[Vec<u8>]) -> [u8; 32] {
    let value = path.iter().cloned().map(Bytes::from).collect::<Vec<_>>();
    keccak256(&value.abi_encode())
}

pub(crate) fn terminal_path_digest(raw: &[u8]) -> Result<[u8; 32], IbcStorageError> {
    let padded_length = raw
        .len()
        .checked_add(31)
        .map(|length| length / 32 * 32)
        .ok_or(IbcStorageError::PathEncodingOverflow)?;
    let capacity = 128_usize
        .checked_add(padded_length)
        .ok_or(IbcStorageError::PathEncodingOverflow)?;
    let mut encoded = Vec::with_capacity(capacity);
    append_abi_word(&mut encoded, 32);
    append_abi_word(&mut encoded, 1);
    append_abi_word(&mut encoded, 32);
    append_abi_word(&mut encoded, raw.len());
    encoded.extend_from_slice(raw);
    encoded.resize(capacity, 0);
    Ok(keccak256(&encoded))
}

fn append_abi_word(encoded: &mut Vec<u8>, value: usize) {
    let mut word = [0_u8; 32];
    let value_bytes = value.to_be_bytes();
    let start = word.len() - value_bytes.len();
    word[start..].copy_from_slice(&value_bytes);
    encoded.extend_from_slice(&word);
}

#[must_use]
pub fn terminal_path(client_id: &[u8], kind: PathKind, sequence: u64) -> Vec<Vec<u8>> {
    let path_type = match kind {
        PathKind::Packet => PACKET_PATH_TYPE,
        PathKind::Receipt => RECEIPT_PATH_TYPE,
        PathKind::Acknowledgement => ACKNOWLEDGEMENT_PATH_TYPE,
    };
    let mut raw = Vec::with_capacity(client_id.len().saturating_add(9));
    raw.extend_from_slice(client_id);
    raw.push(path_type);
    raw.extend_from_slice(&sequence.to_be_bytes());
    vec![raw]
}

#[cfg(test)]
mod tests {
    use super::{path_digest, terminal_path_digest};
    use crate::IbcStorageError;

    #[test]
    fn optimized_single_path_abi_matches_canonical_encoder_at_word_boundaries(
    ) -> Result<(), IbcStorageError> {
        for length in [10, 31, 32, 33, 63, 64, 65, 137] {
            let raw = (0..length)
                .map(|index| u8::try_from(index % 251).unwrap_or(0))
                .collect::<Vec<_>>();
            assert_eq!(terminal_path_digest(&raw)?, path_digest(&[raw]));
        }
        Ok(())
    }
}
