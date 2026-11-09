#![forbid(unsafe_code)]

extern crate alloc;

use alloc::vec::Vec;
use besu_zk_types::{keccak256, MAX_BATCH_MESSAGES};
use core::fmt;

pub const LEAF_DOMAIN: u8 = 0x00;
pub const NODE_DOMAIN: u8 = 0x01;
pub const EMPTY_LEAF_DOMAIN: u8 = 0x02;

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BatchMerkleError {
    EmptyBatch,
    BatchTooLarge(usize),
    InvalidLeafIndex { index: usize, count: usize },
    InvalidProofLength { expected: usize, actual: usize },
    RootMismatch,
}

impl fmt::Display for BatchMerkleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyBatch => f.write_str("membership batch is empty"),
            Self::BatchTooLarge(count) => write!(f, "membership batch has {count} entries"),
            Self::InvalidLeafIndex { index, count } => {
                write!(f, "leaf index {index} is outside entry count {count}")
            }
            Self::InvalidProofLength { expected, actual } => {
                write!(f, "expected Merkle proof length {expected}, got {actual}")
            }
            Self::RootMismatch => f.write_str("membership batch Merkle root mismatch"),
        }
    }
}

impl core::error::Error for BatchMerkleError {}

#[must_use]
pub fn membership_leaf(path_digest: [u8; 32], expected_commitment: [u8; 32]) -> [u8; 32] {
    let mut bytes = [0_u8; 65];
    bytes[0] = LEAF_DOMAIN;
    bytes[1..33].copy_from_slice(&path_digest);
    bytes[33..].copy_from_slice(&expected_commitment);
    keccak256(&bytes)
}

#[must_use]
pub fn parent(left: [u8; 32], right: [u8; 32]) -> [u8; 32] {
    let mut bytes = [0_u8; 65];
    bytes[0] = NODE_DOMAIN;
    bytes[1..33].copy_from_slice(&left);
    bytes[33..].copy_from_slice(&right);
    keccak256(&bytes)
}

#[must_use]
pub fn empty_batch_leaf() -> [u8; 32] {
    keccak256(&[EMPTY_LEAF_DOMAIN])
}

pub fn root(leaves: &[[u8; 32]]) -> Result<[u8; 32], BatchMerkleError> {
    let mut level = padded(leaves)?;
    while level.len() > 1 {
        level = next_level(&level);
    }
    level.first().copied().ok_or(BatchMerkleError::EmptyBatch)
}

pub fn branch(leaves: &[[u8; 32]], index: usize) -> Result<Vec<[u8; 32]>, BatchMerkleError> {
    if index >= leaves.len() {
        return Err(BatchMerkleError::InvalidLeafIndex {
            index,
            count: leaves.len(),
        });
    }
    let mut level = padded(leaves)?;
    let mut current = index;
    let mut proof = Vec::new();
    while level.len() > 1 {
        proof.push(level[current ^ 1]);
        current /= 2;
        level = next_level(&level);
    }
    Ok(proof)
}

pub fn verify(
    expected_root: [u8; 32],
    leaf: [u8; 32],
    index: usize,
    entry_count: usize,
    proof: &[[u8; 32]],
) -> Result<(), BatchMerkleError> {
    validate_entry_count(entry_count)?;
    if index >= entry_count {
        return Err(BatchMerkleError::InvalidLeafIndex {
            index,
            count: entry_count,
        });
    }
    let expected_len = entry_count.next_power_of_two().trailing_zeros() as usize;
    if proof.len() != expected_len {
        return Err(BatchMerkleError::InvalidProofLength {
            expected: expected_len,
            actual: proof.len(),
        });
    }
    let mut current = leaf;
    let mut position = index;
    for sibling in proof {
        current = if position & 1 == 0 {
            parent(current, *sibling)
        } else {
            parent(*sibling, current)
        };
        position /= 2;
    }
    if current == expected_root {
        Ok(())
    } else {
        Err(BatchMerkleError::RootMismatch)
    }
}

// Callers start with power-of-two padding and stop before reducing a singleton.
fn next_level(level: &[[u8; 32]]) -> Vec<[u8; 32]> {
    let mut next = Vec::with_capacity(level.len() / 2);
    for pair in level.chunks_exact(2) {
        next.push(parent(pair[0], pair[1]));
    }
    next
}

fn padded(leaves: &[[u8; 32]]) -> Result<Vec<[u8; 32]>, BatchMerkleError> {
    validate_entry_count(leaves.len())?;
    let padded_len = leaves.len().next_power_of_two();
    let mut level = Vec::with_capacity(padded_len);
    level.extend_from_slice(leaves);
    level.resize(padded_len, empty_batch_leaf());
    Ok(level)
}

/// Checks the entry count shared by membership verification and Merkle operations.
pub fn validate_entry_count(count: usize) -> Result<(), BatchMerkleError> {
    if count == 0 {
        Err(BatchMerkleError::EmptyBatch)
    } else if count > MAX_BATCH_MESSAGES {
        Err(BatchMerkleError::BatchTooLarge(count))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn positional_roots_and_branches_cover_required_sizes() -> Result<(), Box<dyn std::error::Error>>
    {
        for count in 1..=MAX_BATCH_MESSAGES {
            let leaves = (0..count)
                .map(|index| {
                    let marker = u8::try_from(index)?;
                    Ok(membership_leaf([marker; 32], [0x80 | marker; 32]))
                })
                .collect::<Result<Vec<_>, core::num::TryFromIntError>>()?;
            let batch_root = root(&leaves)?;
            for (index, leaf) in leaves.iter().enumerate() {
                let proof = branch(&leaves, index)?;
                verify(batch_root, *leaf, index, count, &proof)?;
                assert!(
                    verify(batch_root, *leaf, (index + 1) % count, count, &proof).is_err()
                        || count == 1
                );
            }
        }
        Ok(())
    }

    #[test]
    fn rejects_empty_oversized_and_mutated_inputs() -> Result<(), BatchMerkleError> {
        assert_eq!(root(&[]), Err(BatchMerkleError::EmptyBatch));
        assert!(root(&[[0_u8; 32]; 33]).is_err());
        let leaves = [[1_u8; 32], [2_u8; 32], [3_u8; 32]];
        let batch_root = root(&leaves)?;
        let mut proof = branch(&leaves, 1)?;
        proof[0][0] ^= 1;
        assert!(verify(batch_root, leaves[1], 1, 3, &proof).is_err());
        Ok(())
    }
}
