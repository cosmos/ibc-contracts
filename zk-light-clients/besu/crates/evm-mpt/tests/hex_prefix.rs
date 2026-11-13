use evm_mpt::{bytes_to_nibbles, decode_compact_path, MptError};

#[test]
fn decodes_leaf_and_extension_parities() -> Result<(), MptError> {
    let odd_leaf = decode_compact_path(&[0x31, 0x23])?;
    assert!(odd_leaf.is_leaf);
    assert_eq!(odd_leaf.nibbles, [1, 2, 3]);

    let even_leaf = decode_compact_path(&[0x20, 0x12])?;
    assert!(even_leaf.is_leaf);
    assert_eq!(even_leaf.nibbles, [1, 2]);

    let odd_extension = decode_compact_path(&[0x11, 0x23])?;
    assert!(!odd_extension.is_leaf);
    assert_eq!(odd_extension.nibbles, [1, 2, 3]);

    let even_extension = decode_compact_path(&[0x00, 0x12])?;
    assert!(!even_extension.is_leaf);
    assert_eq!(even_extension.nibbles, [1, 2]);
    assert_eq!(bytes_to_nibbles(&[0xab]), [0x0a, 0x0b]);
    Ok(())
}

#[test]
fn rejects_bad_flags_padding_empty_extension_and_long_paths() {
    assert_eq!(
        decode_compact_path(&[0x40]),
        Err(MptError::InvalidCompactPath)
    );
    assert_eq!(
        decode_compact_path(&[0x01]),
        Err(MptError::InvalidCompactPath)
    );
    assert_eq!(
        decode_compact_path(&[0x00]),
        Err(MptError::EmptyExtensionPath)
    );
    assert_eq!(
        decode_compact_path(&[0x20; 34]),
        Err(MptError::CompactPathTooLong)
    );
}
