use alloc::vec::Vec;

use besu_zk_types::keccak256;

use crate::{
    node::{decode_node, Node},
    rlp::{decode_exact, RlpItem, RlpKind},
    MptError,
};

pub(crate) const EMPTY_TRIE_ROOT: [u8; 32] = [
    0x56, 0xe8, 0x1f, 0x17, 0x1b, 0xcc, 0x55, 0xa6, 0xff, 0x83, 0x45, 0xe6, 0x92, 0xc0, 0xf8, 0x6e,
    0x5b, 0x48, 0xe0, 0x1b, 0x99, 0x6c, 0xad, 0xc0, 0x01, 0x62, 0x2f, 0xb5, 0xe3, 0x63, 0xb4, 0x21,
];

pub(crate) enum TrieResult<'a> {
    Present(&'a [u8]),
    Absent,
}

enum Reference<'a> {
    Hash([u8; 32]),
    Inline(&'a [u8]),
}

struct ProofNodeIndex {
    hash: [u8; 32],
    index: usize,
    used: bool,
}

pub(crate) fn verify_trie<'a>(
    root: [u8; 32],
    path: &[u8],
    proof: &'a [Vec<u8>],
) -> Result<TrieResult<'a>, MptError> {
    if root == EMPTY_TRIE_ROOT {
        return if proof.is_empty() {
            Ok(TrieResult::Absent)
        } else {
            Err(MptError::UnusedProofNode)
        };
    }
    if path.len() != 64 {
        return Err(MptError::PathOverflow);
    }

    // EIP-1186 proofs are short. A contiguous index is cheaper in the zkVM
    // than a tree map and preserves support for any supplied node order.
    let mut nodes = Vec::with_capacity(proof.len());
    for (index, encoded) in proof.iter().enumerate() {
        let hash = keccak256(encoded);
        if nodes.iter().any(|node: &ProofNodeIndex| node.hash == hash) {
            return Err(MptError::DuplicateNode);
        }
        nodes.push(ProofNodeIndex {
            hash,
            index,
            used: false,
        });
    }
    if !nodes.iter().any(|node| node.hash == root) {
        return Err(MptError::RootMismatch);
    }

    let mut reference = Reference::Hash(root);
    let mut offset = 0_usize;
    let outcome = loop {
        let encoded = resolve(&reference, proof, &mut nodes)?;
        let item = decode_exact(encoded)?;
        let branch_field = if offset == path.len() {
            16
        } else {
            let nibble = usize::from(*path.get(offset).ok_or(MptError::PathOverflow)?);
            if nibble > 15 {
                return Err(MptError::PathOverflow);
            }
            nibble
        };
        let node = decode_node(item, branch_field)?;
        match node {
            Node::Branch(field) => {
                if offset == path.len() {
                    let value = field.as_bytes()?;
                    break if value.is_empty() {
                        TrieResult::Absent
                    } else {
                        TrieResult::Present(value)
                    };
                }
                offset = offset.checked_add(1).ok_or(MptError::PathOverflow)?;
                if field.kind() == RlpKind::Bytes && field.as_bytes()?.is_empty() {
                    break TrieResult::Absent;
                }
                reference = child_reference(field)?;
            }
            Node::Extension {
                path: compact,
                child,
            } => {
                let end = offset
                    .checked_add(compact.len())
                    .ok_or(MptError::PathOverflow)?;
                let remaining = path.get(offset..end).ok_or(MptError::PathOverflow)?;
                if !compact.matches(remaining) {
                    break TrieResult::Absent;
                }
                offset = end;
                reference = child_reference(child)?;
            }
            Node::Leaf {
                path: compact,
                value,
            } => {
                let remaining = path.get(offset..).ok_or(MptError::PathOverflow)?;
                break if compact.matches(remaining) {
                    TrieResult::Present(value)
                } else {
                    TrieResult::Absent
                };
            }
        }
    };

    if nodes.iter().all(|node| node.used) {
        Ok(outcome)
    } else {
        Err(MptError::UnusedProofNode)
    }
}

fn resolve<'a>(
    reference: &Reference<'a>,
    proof: &'a [Vec<u8>],
    nodes: &mut [ProofNodeIndex],
) -> Result<&'a [u8], MptError> {
    match reference {
        Reference::Inline(encoded) => Ok(encoded),
        Reference::Hash(hash) => {
            let node = nodes
                .iter_mut()
                .find(|node| node.hash == *hash)
                .ok_or(MptError::MissingNode)?;
            node.used = true;
            proof
                .get(node.index)
                .map(Vec::as_slice)
                .ok_or(MptError::MissingNode)
        }
    }
}

fn child_reference(item: RlpItem<'_>) -> Result<Reference<'_>, MptError> {
    match item.kind() {
        RlpKind::List => {
            if item.encoded().len() >= 32 {
                Err(MptError::InvalidChildReference)
            } else {
                Ok(Reference::Inline(item.encoded()))
            }
        }
        RlpKind::Bytes => {
            let bytes = item.as_bytes()?;
            if bytes.len() == 32 {
                let hash = bytes
                    .try_into()
                    .map_err(|_| MptError::InvalidChildReference)?;
                Ok(Reference::Hash(hash))
            } else {
                Err(MptError::InvalidChildReference)
            }
        }
    }
}
