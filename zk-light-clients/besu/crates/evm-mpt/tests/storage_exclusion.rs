use besu_zk_types::{keccak256, ProofResult};
use evm_mpt::{
    rlp::{encode_bytes, encode_list},
    verify_storage, verify_storage_non_membership, MptError, RlpError,
};

const EMPTY_TRIE_ROOT: [u8; 32] = [
    0x56, 0xe8, 0x1f, 0x17, 0x1b, 0xcc, 0x55, 0xa6, 0xff, 0x83, 0x45, 0xe6, 0x92, 0xc0, 0xf8, 0x6e,
    0x5b, 0x48, 0xe0, 0x1b, 0x99, 0x6c, 0xad, 0xc0, 0x01, 0x62, 0x2f, 0xb5, 0xe3, 0x63, 0xb4, 0x21,
];

#[test]
fn empty_trie_is_cryptographic_absence() -> Result<(), MptError> {
    assert_eq!(
        verify_storage(EMPTY_TRIE_ROOT, [1; 32], &[])?,
        ProofResult::Absent
    );
    verify_storage_non_membership(EMPTY_TRIE_ROOT, [1; 32], &[])
}

#[test]
fn present_zero_is_not_absent() {
    assert_ne!(ProofResult::Present([0; 32]), ProofResult::Absent);
}

#[test]
fn proves_missing_branch_child_extension_and_leaf_divergence() -> Result<(), MptError> {
    let slot = [0x11; 32];

    let empty_fields = (0..17).map(|_| encode_bytes(&[])).collect::<Vec<_>>();
    let branch = encode_list(&empty_fields);
    verify_storage_non_membership(keccak256(&branch), slot, &[branch])?;

    let key_nibbles = evm_mpt::bytes_to_nibbles(&keccak256(&slot));
    let first = key_nibbles.first().copied().ok_or(MptError::PathOverflow)?;
    let divergent = (first + 1) % 16;
    let extension_path = encode_bytes(&[0x10 | divergent]);
    let extension = encode_list(&[extension_path, encode_bytes(&[0x22; 32])]);
    verify_storage_non_membership(keccak256(&extension), slot, &[extension])?;

    let other_slot = [0x12; 32];
    let mut leaf_path = vec![0x20];
    leaf_path.extend_from_slice(&keccak256(&other_slot));
    let leaf = encode_list(&[encode_bytes(&leaf_path), encode_bytes(&encode_bytes(&[1]))]);
    verify_storage_non_membership(keccak256(&leaf), slot, &[leaf])?;
    Ok(())
}

#[test]
fn present_zero_leaf_is_rejected_as_non_membership() {
    let slot = [0x11; 32];
    let mut leaf_path = vec![0x20];
    leaf_path.extend_from_slice(&keccak256(&slot));
    let leaf = encode_list(&[encode_bytes(&leaf_path), encode_bytes(&encode_bytes(&[]))]);
    let root = keccak256(&leaf);
    assert_eq!(
        verify_storage(root, slot, std::slice::from_ref(&leaf)),
        Ok(ProofResult::Present([0; 32]))
    );
    assert_eq!(
        verify_storage_non_membership(root, slot, &[leaf]),
        Err(MptError::StoragePresent)
    );
}

#[test]
fn present_nonzero_and_malformed_values_never_become_absence() {
    let slot = [0x21; 32];
    let mut leaf_path = vec![0x20];
    leaf_path.extend_from_slice(&keccak256(&slot));

    let present = encode_list(&[encode_bytes(&leaf_path), encode_bytes(&encode_bytes(&[1]))]);
    assert_eq!(
        verify_storage_non_membership(keccak256(&present), slot, std::slice::from_ref(&present)),
        Err(MptError::StoragePresent)
    );

    let leading_zero = encode_list(&[
        encode_bytes(&leaf_path),
        encode_bytes(&encode_bytes(&[0, 1])),
    ]);
    assert_eq!(
        verify_storage_non_membership(
            keccak256(&leading_zero),
            slot,
            std::slice::from_ref(&leading_zero)
        ),
        Err(MptError::Rlp(RlpError::IntegerLeadingZero))
    );

    let overflow = encode_list(&[
        encode_bytes(&leaf_path),
        encode_bytes(&encode_bytes(&[1; 33])),
    ]);
    assert_eq!(
        verify_storage_non_membership(keccak256(&overflow), slot, std::slice::from_ref(&overflow)),
        Err(MptError::StorageValueOverflow)
    );
}

#[test]
fn exclusion_rejects_wrong_root_truncation_and_malformed_compact_path() {
    let slot = [0x33; 32];
    let mut leaf_path = vec![0x20];
    leaf_path.extend_from_slice(&keccak256(&[0x44; 32]));
    let leaf = encode_list(&[encode_bytes(&leaf_path), encode_bytes(&encode_bytes(&[1]))]);
    let root = keccak256(&leaf);
    assert!(verify_storage_non_membership([0; 32], slot, std::slice::from_ref(&leaf)).is_err());

    let mut truncated = leaf.clone();
    truncated.pop();
    assert!(verify_storage_non_membership(root, slot, &[truncated]).is_err());

    let malformed = encode_list(&[encode_bytes(&[0x40]), encode_bytes(&encode_bytes(&[1]))]);
    assert!(verify_storage_non_membership(keccak256(&malformed), slot, &[malformed]).is_err());
}
