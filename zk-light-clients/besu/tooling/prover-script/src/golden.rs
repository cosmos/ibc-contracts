use std::{collections::BTreeMap, error::Error, io};

use besu_qbft::{
    committed_seal_digest, encode_extra_data, target_quorum, validator_set_hash, QbftExtraData,
};
use besu_zk_types::{
    keccak256, Address, Eip1186ProofWitness, MembershipEntryWitness, MembershipWitness,
    NonMembershipWitness, QbftHeaderInput, UpdateWitness, MEMBERSHIP_SCHEMA_VERSION, SCHEMA_VERSION,
};
use evm_mpt::rlp::{encode_bytes, encode_list, encode_u256};
use ibc_storage::{logical_storage_slot, terminal_path, PathKind};
use k256::ecdsa::SigningKey;

fn address(key: &SigningKey) -> Result<Address, io::Error> {
    let encoded = key.verifying_key().to_encoded_point(false);
    let coordinates = encoded
        .as_bytes()
        .get(1..)
        .ok_or_else(|| io::Error::other("missing public-key coordinates"))?;
    keccak256(coordinates)
        .get(12..)
        .ok_or_else(|| io::Error::other("invalid address hash"))?
        .try_into()
        .map_err(|_| io::Error::other("invalid address length"))
}

pub fn update_witness() -> Result<UpdateWitness, Box<dyn Error>> {
    update_witness_with_validator_count(4)
}

pub fn update_witness_with_validator_count(
    validator_count: u8,
) -> Result<UpdateWitness, Box<dyn Error>> {
    if validator_count == 0 {
        return Err(io::Error::other("validator count must be non-zero").into());
    }
    let mut keys = Vec::new();
    for byte in 1_u8..=validator_count {
        let key = SigningKey::from_slice(&[byte; 32])
            .map_err(|_| io::Error::other("invalid deterministic key"))?;
        keys.push((address(&key)?, key));
    }
    keys.sort_by_key(|(address, _)| *address);
    let validators = keys.iter().map(|(address, _)| *address).collect::<Vec<_>>();
    let mut header = QbftHeaderInput {
        parent_hash: [1; 32],
        ommers_hash: [2; 32],
        beneficiary: [3; 20],
        state_root: [4; 32],
        transactions_root: [5; 32],
        receipts_root: [6; 32],
        logs_bloom: [7; 256],
        difficulty: [0; 32],
        number: 11,
        gas_limit: 30_000_000,
        gas_used: 1,
        timestamp: 1_700_000_011,
        extra_data: Vec::new(),
        mix_hash: [8; 32],
        nonce: [0; 8],
        base_fee_per_gas: None,
        withdrawals_root: None,
        blob_gas_used: None,
        excess_blob_gas: None,
        parent_beacon_block_root: None,
        requests_hash: None,
        block_access_list_hash: None,
    };
    let mut extra = QbftExtraData {
        vanity: [9; 32],
        validators: validators.clone(),
        vote: None,
        round: 2,
        committed_seals: Vec::new(),
    };
    header.extra_data = encode_extra_data(&extra, true, true);
    let digest = committed_seal_digest(&header)?;
    for (_, key) in keys.iter().take(target_quorum(keys.len())?) {
        let (signature, recovery_id) = key
            .sign_prehash_recoverable(&digest)
            .map_err(|_| io::Error::other("test signing failed"))?;
        let mut seal = signature.to_bytes().to_vec();
        seal.push(recovery_id.to_byte());
        extra.committed_seals.push(seal);
    }
    header.extra_data = encode_extra_data(&extra, true, true);
    Ok(UpdateWitness {
        schema_version: SCHEMA_VERSION,
        trusted_height: 10,
        trusted_validator_set_hash: validator_set_hash(&validators)?,
        trusted_validators: validators,
        target_header: header,
    })
}

fn leaf_for_secure_key(key: [u8; 32], value: &[u8]) -> Vec<u8> {
    leaf_for_nibbles(&bytes_to_nibbles(&key), value)
}

fn leaf_for_nibbles(nibbles: &[u8], value: &[u8]) -> Vec<u8> {
    let mut compact_path = Vec::with_capacity(1 + nibbles.len().div_ceil(2));
    let odd = nibbles.len() % 2 == 1;
    let mut index = 0;
    if odd {
        compact_path.push(0x30 | nibbles[0]);
        index = 1;
    } else {
        compact_path.push(0x20);
    }
    while index < nibbles.len() {
        compact_path.push((nibbles[index] << 4) | nibbles[index + 1]);
        index += 2;
    }
    encode_list(&[encode_bytes(&compact_path), encode_bytes(value)])
}

fn bytes_to_nibbles(bytes: &[u8]) -> Vec<u8> {
    let mut nibbles = Vec::with_capacity(bytes.len() * 2);
    for byte in bytes {
        nibbles.push(byte >> 4);
        nibbles.push(byte & 0x0f);
    }
    nibbles
}

fn branch(target_nibble: u8, target: Option<&[u8]>, sibling: &[u8]) -> Vec<u8> {
    let sibling_nibble = (target_nibble + 1) % 16;
    let mut items = Vec::with_capacity(17);
    for nibble in 0_u8..16 {
        let child = if nibble == target_nibble {
            target
        } else if nibble == sibling_nibble {
            Some(sibling)
        } else {
            None
        };
        items.push(child.map_or_else(|| encode_bytes(&[]), |node| encode_bytes(&keccak256(node))));
    }
    items.push(encode_bytes(&[]));
    encode_list(&items)
}

fn sibling_leaf(remaining_nibbles: usize, marker: u8) -> Vec<u8> {
    let nibbles = vec![marker & 0x0f; remaining_nibbles];
    leaf_for_nibbles(&nibbles, &encode_u256(&[marker; 32]))
}

fn storage_membership_proof(
    slot: [u8; 32],
    value: [u8; 32],
    branch_depth: usize,
) -> Result<([u8; 32], Vec<Vec<u8>>), io::Error> {
    let key = bytes_to_nibbles(&keccak256(&slot));
    if branch_depth >= key.len() {
        return Err(io::Error::other("branch depth exceeds secure key"));
    }
    let leaf = leaf_for_nibbles(&key[branch_depth..], &encode_u256(&value));
    let mut current = leaf.clone();
    let mut branches = Vec::with_capacity(branch_depth);
    for depth in (0..branch_depth).rev() {
        let marker = u8::try_from(depth + 1)
            .map_err(|_| io::Error::other("branch depth marker overflow"))?;
        let sibling = sibling_leaf(key.len() - depth - 1, marker);
        current = branch(key[depth], Some(&current), &sibling);
        branches.push(current.clone());
    }
    branches.reverse();
    branches.push(leaf);
    Ok((keccak256(&current), branches))
}

fn storage_exclusion_proof(
    slot: [u8; 32],
    branch_depth: usize,
) -> Result<([u8; 32], Vec<Vec<u8>>), io::Error> {
    let key = bytes_to_nibbles(&keccak256(&slot));
    if branch_depth == 0 || branch_depth > key.len() {
        return Err(io::Error::other("exclusion branch depth is invalid"));
    }
    let last = branch_depth - 1;
    let sibling = sibling_leaf(key.len() - last - 1, 0x0f);
    let mut current = branch(key[last], None, &sibling);
    let mut branches = vec![current.clone()];
    for depth in (0..last).rev() {
        let marker = u8::try_from(depth + 1)
            .map_err(|_| io::Error::other("branch depth marker overflow"))?;
        let sibling = sibling_leaf(key.len() - depth - 1, marker);
        current = branch(key[depth], Some(&current), &sibling);
        branches.push(current.clone());
    }
    branches.reverse();
    Ok((keccak256(&current), branches))
}

fn account_proof(router: [u8; 20], storage_root: [u8; 32]) -> ([u8; 32], Vec<Vec<u8>>) {
    let account = encode_list(&[
        encode_bytes(&[]),
        encode_bytes(&[]),
        encode_bytes(&storage_root),
        encode_bytes(&keccak256(&[])),
    ]);
    let leaf = leaf_for_secure_key(keccak256(&router), &account);
    (keccak256(&leaf), vec![leaf])
}

struct BuiltTrie {
    encoded: Vec<u8>,
    proofs: BTreeMap<usize, Vec<Vec<u8>>>,
}

fn compact_path(nibbles: &[u8], leaf: bool) -> Vec<u8> {
    let odd = nibbles.len() % 2 == 1;
    let flag = if leaf { 2_u8 } else { 0_u8 };
    let mut encoded = Vec::with_capacity(1 + nibbles.len().div_ceil(2));
    let mut index = 0;
    if odd {
        encoded.push(((flag + 1) << 4) | nibbles[0]);
        index = 1;
    } else {
        encoded.push(flag << 4);
    }
    while index < nibbles.len() {
        encoded.push((nibbles[index] << 4) | nibbles[index + 1]);
        index += 2;
    }
    encoded
}

fn child_reference(encoded: &[u8]) -> Vec<u8> {
    if encoded.len() < 32 { encoded.to_vec() } else { encode_bytes(&keccak256(encoded)) }
}

fn build_storage_trie(
    entries: &[(usize, Vec<u8>, [u8; 32])],
    depth: usize,
) -> Result<BuiltTrie, io::Error> {
    if entries.is_empty() { return Err(io::Error::other("cannot build empty trie")); }
    if entries.len() == 1 {
        let (index, key, value) = &entries[0];
        let remaining = key.get(depth..).ok_or_else(|| io::Error::other("trie depth overflow"))?;
        let encoded = encode_list(&[
            encode_bytes(&compact_path(remaining, true)),
            encode_bytes(&encode_u256(value)),
        ]);
        return Ok(BuiltTrie { encoded: encoded.clone(), proofs: BTreeMap::from([(*index, vec![encoded])]) });
    }

    let first = entries[0].1.get(depth..).ok_or_else(|| io::Error::other("trie depth overflow"))?;
    let common = first.iter().enumerate().take_while(|(offset, nibble)| {
        entries.iter().all(|(_, key, _)| key.get(depth + *offset) == Some(nibble))
    }).count();
    if common > 0 {
        let child = build_storage_trie(entries, depth + common)?;
        let child_inline = child.encoded.len() < 32;
        let encoded = encode_list(&[
            encode_bytes(&compact_path(&first[..common], false)),
            child_reference(&child.encoded),
        ]);
        let proofs = child.proofs.into_iter().map(|(index, mut proof)| {
            if child_inline && !proof.is_empty() { proof.remove(0); }
            proof.insert(0, encoded.clone());
            (index, proof)
        }).collect();
        return Ok(BuiltTrie { encoded, proofs });
    }

    let mut items = vec![encode_bytes(&[]); 17];
    let mut child_results = Vec::new();
    for nibble in 0_u8..16 {
        let group: Vec<_> = entries.iter().filter(|(_, key, _)| key.get(depth) == Some(&nibble)).cloned().collect();
        if group.is_empty() { continue; }
        let child = build_storage_trie(&group, depth + 1)?;
        items[usize::from(nibble)] = child_reference(&child.encoded);
        child_results.push((child.encoded.len() < 32, child));
    }
    let encoded = encode_list(&items);
    let mut proofs = BTreeMap::new();
    for (child_inline, child) in child_results {
        for (index, mut proof) in child.proofs {
            if child_inline && !proof.is_empty() { proof.remove(0); }
            proof.insert(0, encoded.clone());
            proofs.insert(index, proof);
        }
    }
    Ok(BuiltTrie { encoded, proofs })
}

pub fn membership_witness() -> Result<MembershipWitness, io::Error> {
    membership_batch_witness(1)
}

pub fn membership_batch_witness(count: usize) -> Result<MembershipWitness, io::Error> {
    if count == 0 || count > 32 { return Err(io::Error::other("batch size must be 1..=32")); }
    let source_router = [0x11; 20];
    let mut canonical = (0..count).map(|index| {
        let sequence = u64::try_from(index + 1).map_err(|_| io::Error::other("sequence overflow"))?;
        let kind = if index % 2 == 0 { PathKind::Packet } else { PathKind::Acknowledgement };
        let path = terminal_path(b"client-1", kind, sequence);
        let digest = ibc_storage::path_digest(&path);
        let commitment = [u8::try_from(index + 1).map_err(|_| io::Error::other("marker overflow"))?; 32];
        Ok((digest, path, commitment))
    }).collect::<Result<Vec<_>, io::Error>>()?;
    canonical.sort_by_key(|(digest, _, _)| *digest);

    let trie_entries = canonical.iter().enumerate().map(|(index, (_, path, commitment))| {
        let terminal = path.first().ok_or_else(|| io::Error::other("missing terminal path"))?;
        let slot = logical_storage_slot(terminal);
        Ok((index, bytes_to_nibbles(&keccak256(&slot)), *commitment))
    }).collect::<Result<Vec<_>, io::Error>>()?;
    let trie = build_storage_trie(&trie_entries, 0)?;
    let storage_root = keccak256(&trie.encoded);
    let (state_root, account_proof) = account_proof(source_router, storage_root);
    let entries = canonical.into_iter().enumerate().map(|(index, (_, path, expected_commitment))| {
        let storage_proof = trie.proofs.get(&index).cloned()
            .ok_or_else(|| io::Error::other("missing generated storage proof"))?;
        Ok(MembershipEntryWitness { path, expected_commitment, storage_proof })
    }).collect::<Result<Vec<_>, io::Error>>()?;
    Ok(MembershipWitness {
        schema_version: MEMBERSHIP_SCHEMA_VERSION,
        proof_height: 12,
        state_root,
        source_router,
        account_proof,
        entries,
    })
}

pub fn membership_witness_with_proof_depth(
    branch_depth: usize,
) -> Result<MembershipWitness, io::Error> {
    let source_router = [0x11; 20];
    let path = terminal_path(b"client-1", PathKind::Acknowledgement, 7);
    let terminal = path
        .first()
        .ok_or_else(|| io::Error::other("missing terminal path"))?;
    let slot = logical_storage_slot(terminal);
    let commitment = [0x44; 32];
    let (storage_root, storage_proof) = storage_membership_proof(slot, commitment, branch_depth)?;
    let (state_root, account_proof) = account_proof(source_router, storage_root);
    Ok(MembershipWitness {
        schema_version: MEMBERSHIP_SCHEMA_VERSION,
        proof_height: 12,
        state_root,
        source_router,
        account_proof,
        entries: vec![MembershipEntryWitness {
            path,
            expected_commitment: commitment,
            storage_proof,
        }],
    })
}

#[must_use]
pub fn non_membership_witness() -> NonMembershipWitness {
    let source_router = [0x22; 20];
    let empty_storage_root = keccak256(&encode_bytes(&[]));
    let (state_root, account_proof) = account_proof(source_router, empty_storage_root);
    NonMembershipWitness {
        schema_version: SCHEMA_VERSION,
        proof_height: 13,
        state_root,
        source_router,
        path: terminal_path(b"client-2", PathKind::Receipt, 8),
        proof: Eip1186ProofWitness {
            account_proof,
            storage_proof: Vec::new(),
        },
    }
}

pub fn non_membership_witness_with_proof_depth(
    branch_depth: usize,
) -> Result<NonMembershipWitness, io::Error> {
    if branch_depth == 0 {
        return Ok(non_membership_witness());
    }
    let source_router = [0x22; 20];
    let path = terminal_path(b"client-2", PathKind::Receipt, 8);
    let terminal = path
        .first()
        .ok_or_else(|| io::Error::other("missing terminal path"))?;
    let slot = logical_storage_slot(terminal);
    let (storage_root, storage_proof) = storage_exclusion_proof(slot, branch_depth)?;
    let (state_root, account_proof) = account_proof(source_router, storage_root);
    Ok(NonMembershipWitness {
        schema_version: SCHEMA_VERSION,
        proof_height: 13,
        state_root,
        source_router,
        path,
        proof: Eip1186ProofWitness {
            account_proof,
            storage_proof,
        },
    })
}

#[cfg(test)]
mod tests {
    use besu_qbft::verify_update;

    use ibc_storage::{verify_membership, verify_non_membership};

    use super::{
        membership_batch_witness, membership_witness_with_proof_depth,
        non_membership_witness_with_proof_depth, update_witness_with_validator_count,
    };

    #[test]
    fn update_benchmark_fixtures_are_valid_and_distinct() -> Result<(), Box<dyn std::error::Error>>
    {
        let small = update_witness_with_validator_count(1)?;
        let four = update_witness_with_validator_count(4)?;
        let larger = update_witness_with_validator_count(16)?;

        verify_update(&small)?;
        verify_update(&four)?;
        verify_update(&larger)?;
        assert_eq!(small.trusted_validators.len(), 1);
        assert_eq!(four.trusted_validators.len(), 4);
        assert_eq!(larger.trusted_validators.len(), 16);
        assert!(small.target_header.extra_data.len() < four.target_header.extra_data.len());
        assert!(four.target_header.extra_data.len() < larger.target_header.extra_data.len());
        assert!(update_witness_with_validator_count(0).is_err());
        Ok(())
    }

    #[test]
    fn deeper_mpt_benchmark_fixtures_are_valid_and_distinct(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let small_membership = membership_witness_with_proof_depth(0)?;
        let larger_membership = membership_witness_with_proof_depth(12)?;
        let small_non_membership = non_membership_witness_with_proof_depth(0)?;
        let larger_non_membership = non_membership_witness_with_proof_depth(12)?;

        verify_membership(&small_membership)?;
        verify_membership(&larger_membership)?;
        verify_non_membership(&small_non_membership)?;
        verify_non_membership(&larger_non_membership)?;
        assert!(
            small_membership.entries[0].storage_proof.len()
                < larger_membership.entries[0].storage_proof.len()
        );
        assert!(
            small_non_membership.proof.storage_proof.len()
                < larger_non_membership.proof.storage_proof.len()
        );
        assert!(membership_witness_with_proof_depth(64).is_err());
        assert!(non_membership_witness_with_proof_depth(65).is_err());
        Ok(())
    }

    #[test]
    fn membership_batch_fixtures_cover_release_matrix() -> Result<(), Box<dyn std::error::Error>> {
        for count in [1_usize, 5, 10, 20, 32] {
            let witness = membership_batch_witness(count)?;
            let values = verify_membership(&witness)?;
            assert_eq!(usize::try_from(values.entry_count)?, count);
            assert_eq!(values.schema_version, besu_zk_types::MEMBERSHIP_SCHEMA_VERSION);
        }
        assert!(membership_batch_witness(0).is_err());
        assert!(membership_batch_witness(33).is_err());
        Ok(())
    }
}
