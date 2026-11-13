use besu_zk_types::{keccak256, ProofResult};
use evm_mpt::{
    rlp::{encode_bytes, encode_list, encode_u256},
    verify_storage, verify_storage_membership, verify_storage_membership_expected, MptError,
};

fn storage_leaf(slot: [u8; 32], value: [u8; 32]) -> Vec<u8> {
    let mut compact = Vec::with_capacity(33);
    compact.push(0x20);
    compact.extend_from_slice(&keccak256(&slot));
    encode_list(&[encode_bytes(&compact), encode_bytes(&encode_u256(&value))])
}

fn compact_path(nibbles: &[u8], leaf: bool) -> Vec<u8> {
    let odd = nibbles.len() % 2 == 1;
    let mut encoded = Vec::with_capacity(nibbles.len().div_ceil(2) + 1);
    let flag = if leaf { 2_u8 } else { 0 };
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

fn leaf_for_nibbles(nibbles: &[u8], value: [u8; 32]) -> Vec<u8> {
    encode_list(&[
        encode_bytes(&compact_path(nibbles, true)),
        encode_bytes(&encode_u256(&value)),
    ])
}

fn child_item(child: &[u8]) -> Vec<u8> {
    if child.len() < 32 {
        child.to_vec()
    } else {
        encode_bytes(&keccak256(child))
    }
}

fn branch_at(nibble: u8, child: &[u8]) -> Vec<u8> {
    let mut fields = (0..17).map(|_| encode_bytes(&[])).collect::<Vec<_>>();
    fields[usize::from(nibble)] = child_item(child);
    encode_list(&fields)
}

fn extension(path: &[u8], child: &[u8]) -> Vec<u8> {
    encode_list(&[encode_bytes(&compact_path(path, false)), child_item(child)])
}

#[test]
fn authenticates_exact_slot_and_value() -> Result<(), MptError> {
    let slot = [0x11; 32];
    let expected = [0x22; 32];
    let leaf = storage_leaf(slot, expected);
    let root = keccak256(&leaf);
    assert_eq!(
        verify_storage(root, slot, std::slice::from_ref(&leaf))?,
        ProofResult::Present(expected)
    );
    assert_eq!(
        verify_storage_membership(root, slot, std::slice::from_ref(&leaf))?,
        expected
    );
    verify_storage_membership_expected(root, slot, std::slice::from_ref(&leaf), &expected)?;
    Ok(())
}

#[test]
fn direct_expected_word_binding_handles_padding_zero_and_mismatch() -> Result<(), MptError> {
    let slot = [0x21; 32];
    for expected in [
        [0_u8; 32],
        {
            let mut value = [0_u8; 32];
            value[31] = 1;
            value
        },
        [0xff; 32],
    ] {
        let leaf = storage_leaf(slot, expected);
        let root = keccak256(&leaf);
        verify_storage_membership_expected(root, slot, std::slice::from_ref(&leaf), &expected)?;

        let mut wrong = expected;
        wrong[31] ^= 1;
        assert_eq!(
            verify_storage_membership_expected(root, slot, std::slice::from_ref(&leaf), &wrong),
            Err(MptError::StorageValueMismatch)
        );
    }
    Ok(())
}

#[test]
fn rejects_wrong_root_slot_mutation_truncation_and_extra_nodes() {
    let slot = [0x11; 32];
    let leaf = storage_leaf(slot, [0x22; 32]);
    let root = keccak256(&leaf);
    assert!(verify_storage([0; 32], slot, std::slice::from_ref(&leaf)).is_err());
    assert_eq!(
        verify_storage(root, [0x12; 32], std::slice::from_ref(&leaf)),
        Ok(ProofResult::Absent)
    );
    assert_eq!(
        verify_storage_membership_expected(
            root,
            [0x12; 32],
            std::slice::from_ref(&leaf),
            &[0x22; 32]
        ),
        Err(MptError::StorageAbsent)
    );
    assert!(verify_storage(root, slot, &[]).is_err());
    assert!(verify_storage(root, slot, &[leaf.clone(), vec![0xc0]]).is_err());
    let mut mutated = leaf;
    mutated.pop();
    assert!(verify_storage(root, slot, &[mutated]).is_err());
}

#[test]
fn authenticates_hashed_branch_to_leaf_topology() -> Result<(), MptError> {
    let slot = [0x31; 32];
    let value = [0x41; 32];
    let path = evm_mpt::bytes_to_nibbles(&keccak256(&slot));
    let leaf = leaf_for_nibbles(&path[1..], value);
    let root_node = branch_at(path[0], &leaf);
    assert_eq!(
        verify_storage(keccak256(&root_node), slot, &[root_node, leaf])?,
        ProofResult::Present(value)
    );
    Ok(())
}

#[test]
fn authenticates_extension_branch_and_inline_leaf_topology() -> Result<(), MptError> {
    let slot = [0x51; 32];
    let mut value = [0_u8; 32];
    value[31] = 1;
    let path = evm_mpt::bytes_to_nibbles(&keccak256(&slot));
    let inline_leaf = leaf_for_nibbles(&path[63..], value);
    assert!(inline_leaf.len() < 32);
    let branch = branch_at(path[62], &inline_leaf);
    let root_node = extension(&path[..62], &branch);
    assert_eq!(
        verify_storage(keccak256(&root_node), slot, &[root_node])?,
        ProofResult::Present(value)
    );
    Ok(())
}
