use besu_zk_types::keccak256;
use evm_mpt::{
    rlp::{encode_bytes, encode_list},
    verify_account, verify_account_storage_root, MptError,
};

fn leaf_for_secure_key(key: [u8; 32], value: &[u8]) -> Vec<u8> {
    let mut compact = Vec::with_capacity(33);
    compact.push(0x20);
    compact.extend_from_slice(&key);
    encode_list(&[encode_bytes(&compact), encode_bytes(value)])
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

fn child_hash(child: &[u8]) -> Vec<u8> {
    encode_bytes(&keccak256(child))
}

fn branch(nibble: u8, child: &[u8]) -> Vec<u8> {
    let mut fields = (0..17).map(|_| encode_bytes(&[])).collect::<Vec<_>>();
    fields[usize::from(nibble)] = child_hash(child);
    encode_list(&fields)
}

fn extension(path: &[u8], child: &[u8]) -> Vec<u8> {
    encode_list(&[encode_bytes(&compact_path(path, false)), child_hash(child)])
}

fn account_value(storage_root: [u8; 32], code_hash: [u8; 32]) -> Vec<u8> {
    encode_list(&[
        encode_bytes(&[1]),
        encode_bytes(&[2]),
        encode_bytes(&storage_root),
        encode_bytes(&code_hash),
    ])
}

#[test]
fn authenticates_and_decodes_exact_account_fields() -> Result<(), MptError> {
    let address = [0x11; 20];
    let storage_root = [0x22; 32];
    let code_hash = [0x33; 32];
    let account = encode_list(&[
        encode_bytes(&[1]),
        encode_bytes(&[2]),
        encode_bytes(&storage_root),
        encode_bytes(&code_hash),
    ]);
    let leaf = leaf_for_secure_key(keccak256(&address), &account);
    let root = keccak256(&leaf);
    let decoded = verify_account(root, address, std::slice::from_ref(&leaf))?;
    assert_eq!(decoded.nonce[31], 1);
    assert_eq!(decoded.balance[31], 2);
    assert_eq!(decoded.storage_root, storage_root);
    assert_eq!(decoded.code_hash, code_hash);
    assert_eq!(
        verify_account_storage_root(root, address, std::slice::from_ref(&leaf))?,
        storage_root
    );
    Ok(())
}

#[test]
fn storage_root_only_authentication_still_validates_every_account_field() {
    let address = [0x21; 20];
    for malformed in [
        encode_list(&[
            encode_bytes(&[0]),
            encode_bytes(&[]),
            encode_bytes(&[0x22; 32]),
            encode_bytes(&[0x33; 32]),
        ]),
        encode_list(&[
            encode_bytes(&[]),
            encode_bytes(&[0, 1]),
            encode_bytes(&[0x22; 32]),
            encode_bytes(&[0x33; 32]),
        ]),
        encode_list(&[
            encode_bytes(&[1; 33]),
            encode_bytes(&[]),
            encode_bytes(&[0x22; 32]),
            encode_bytes(&[0x33; 32]),
        ]),
        encode_list(&[
            encode_bytes(&[]),
            encode_bytes(&[1; 33]),
            encode_bytes(&[0x22; 32]),
            encode_bytes(&[0x33; 32]),
        ]),
        encode_list(&[
            encode_bytes(&[]),
            encode_bytes(&[]),
            encode_bytes(&[0x22; 31]),
            encode_bytes(&[0x33; 32]),
        ]),
        encode_list(&[
            encode_bytes(&[]),
            encode_bytes(&[]),
            encode_bytes(&[0x22; 32]),
        ]),
    ] {
        let leaf = leaf_for_secure_key(keccak256(&address), &malformed);
        assert!(verify_account(keccak256(&leaf), address, std::slice::from_ref(&leaf)).is_err());
        assert!(verify_account_storage_root(
            keccak256(&leaf),
            address,
            std::slice::from_ref(&leaf)
        )
        .is_err());
    }
}

#[test]
fn rejects_wrong_root_address_mutation_missing_and_extra_nodes() {
    let address = [0x11; 20];
    let account = encode_list(&[
        encode_bytes(&[]),
        encode_bytes(&[]),
        encode_bytes(&[0x22; 32]),
        encode_bytes(&[0x33; 32]),
    ]);
    let leaf = leaf_for_secure_key(keccak256(&address), &account);
    let root = keccak256(&leaf);
    assert!(verify_account([0; 32], address, std::slice::from_ref(&leaf)).is_err());
    assert_eq!(
        verify_account(root, [0x12; 20], std::slice::from_ref(&leaf)),
        Err(MptError::AccountAbsent)
    );
    assert!(verify_account(root, address, &[]).is_err());
    assert!(verify_account(root, address, &[leaf.clone(), vec![0xc0]]).is_err());
    let mut mutated = leaf;
    if let Some(last) = mutated.last_mut() {
        *last ^= 1;
    }
    assert!(verify_account(root, address, &[mutated]).is_err());
}

#[test]
fn authenticates_branch_extension_and_multiple_extension_account_paths() -> Result<(), MptError> {
    let address = [0x61; 20];
    let storage_root = [0x71; 32];
    let code_hash = [0x81; 32];
    let path = evm_mpt::bytes_to_nibbles(&keccak256(&address));
    let value = account_value(storage_root, code_hash);
    let leaf = encode_list(&[
        encode_bytes(&compact_path(&path[5..], true)),
        encode_bytes(&value),
    ]);
    let second_extension = encode_list(&[
        encode_bytes(&compact_path(&path[3..5], false)),
        child_hash(&leaf),
    ]);
    let branch = {
        let mut fields = (0..17).map(|_| encode_bytes(&[])).collect::<Vec<_>>();
        fields[usize::from(path[2])] = child_hash(&second_extension);
        encode_list(&fields)
    };
    let first_extension = encode_list(&[
        encode_bytes(&compact_path(&path[..2], false)),
        child_hash(&branch),
    ]);
    let proof = [first_extension, branch, second_extension, leaf];
    let decoded = verify_account(keccak256(&proof[0]), address, &proof)?;
    assert_eq!(decoded.storage_root, storage_root);
    assert_eq!(decoded.code_hash, code_hash);

    let mut unordered = proof.to_vec();
    unordered.reverse();
    assert_eq!(
        verify_account_storage_root(keccak256(&proof[0]), address, &unordered)?,
        storage_root
    );

    let mut duplicate = proof.to_vec();
    duplicate.push(proof[0].clone());
    assert_eq!(
        verify_account_storage_root(keccak256(&proof[0]), address, &duplicate),
        Err(MptError::DuplicateNode)
    );
    Ok(())
}

#[test]
fn authenticates_each_account_branch_and_extension_topology() -> Result<(), MptError> {
    let address = [0x62; 20];
    let storage_root = [0x72; 32];
    let code_hash = [0x82; 32];
    let path = evm_mpt::bytes_to_nibbles(&keccak256(&address));
    let value = account_value(storage_root, code_hash);

    let leaf = encode_list(&[
        encode_bytes(&compact_path(&path[1..], true)),
        encode_bytes(&value),
    ]);
    let root = branch(path[0], &leaf);
    assert_eq!(
        verify_account(keccak256(&root), address, &[root, leaf])?.storage_root,
        storage_root
    );

    let leaf = encode_list(&[
        encode_bytes(&compact_path(&path[3..], true)),
        encode_bytes(&value),
    ]);
    let middle = extension(&path[1..3], &leaf);
    let root = branch(path[0], &middle);
    assert_eq!(
        verify_account(keccak256(&root), address, &[root, middle, leaf])?.storage_root,
        storage_root
    );

    let leaf = encode_list(&[
        encode_bytes(&compact_path(&path[3..], true)),
        encode_bytes(&value),
    ]);
    let middle = branch(path[2], &leaf);
    let root = extension(&path[..2], &middle);
    assert_eq!(
        verify_account(keccak256(&root), address, &[root, middle, leaf])?.storage_root,
        storage_root
    );
    Ok(())
}
