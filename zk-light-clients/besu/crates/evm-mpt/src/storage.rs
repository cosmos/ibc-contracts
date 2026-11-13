use alloc::vec::Vec;

use besu_zk_types::{keccak256, ProofResult};

use crate::{
    inclusion::require_present,
    nibble::secure_key_nibbles,
    proof::{verify_trie, TrieResult},
    rlp::{decode_exact, validate_integer},
    MptError,
};

pub fn verify_storage_membership(
    storage_root: [u8; 32],
    logical_slot: [u8; 32],
    proof: &[Vec<u8>],
) -> Result<[u8; 32], MptError> {
    require_present(&verify_storage(storage_root, logical_slot, proof)?)
}

/// Authenticates an exact storage word and compares its canonical integer
/// bytes directly, avoiding an intermediate padded word in membership guests.
pub fn verify_storage_membership_expected(
    storage_root: [u8; 32],
    logical_slot: [u8; 32],
    proof: &[Vec<u8>],
    expected: &[u8; 32],
) -> Result<(), MptError> {
    let Some(actual) = authenticated_storage_value(storage_root, logical_slot, proof)? else {
        return Err(MptError::StorageAbsent);
    };
    let first_non_zero = expected
        .iter()
        .position(|byte| *byte != 0)
        .unwrap_or(expected.len());
    if actual == &expected[first_non_zero..] {
        Ok(())
    } else {
        Err(MptError::StorageValueMismatch)
    }
}

pub fn verify_storage(
    storage_root: [u8; 32],
    logical_slot: [u8; 32],
    proof: &[Vec<u8>],
) -> Result<ProofResult, MptError> {
    match authenticated_storage_value(storage_root, logical_slot, proof)? {
        None => Ok(ProofResult::Absent),
        Some(bytes) => {
            let mut word = [0_u8; 32];
            let start = 32_usize
                .checked_sub(bytes.len())
                .ok_or(MptError::StorageValueOverflow)?;
            word[start..].copy_from_slice(bytes);
            Ok(ProofResult::Present(word))
        }
    }
}

fn authenticated_storage_value(
    storage_root: [u8; 32],
    logical_slot: [u8; 32],
    proof: &[Vec<u8>],
) -> Result<Option<&[u8]>, MptError> {
    let path = secure_key_nibbles(keccak256(&logical_slot));
    match verify_trie(storage_root, &path, proof)? {
        TrieResult::Absent => Ok(None),
        TrieResult::Present(encoded_value) => {
            let bytes = decode_exact(encoded_value)?.as_bytes()?;
            validate_integer(bytes)?;
            if bytes.len() > 32 {
                return Err(MptError::StorageValueOverflow);
            }
            Ok(Some(bytes))
        }
    }
}

pub fn verify_storage_non_membership(
    storage_root: [u8; 32],
    logical_slot: [u8; 32],
    proof: &[Vec<u8>],
) -> Result<(), MptError> {
    match authenticated_storage_value(storage_root, logical_slot, proof)? {
        None => Ok(()),
        Some(_) => Err(MptError::StoragePresent),
    }
}
