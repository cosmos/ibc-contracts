use ibc_storage::{
    parse_membership_path, parse_receipt_path, terminal_path, IbcStorageError, PathKind,
};

#[test]
fn parses_upstream_packet_receipt_and_ack_paths() {
    for sequence in [0, 1, 1_u64 << 32, u64::MAX] {
        let packet = terminal_path(b"client-0", PathKind::Packet, sequence);
        assert_eq!(
            parse_membership_path(&packet).map(|path| path.sequence),
            Ok(sequence)
        );
        let receipt = terminal_path(b"client-0", PathKind::Receipt, sequence);
        assert_eq!(
            parse_receipt_path(&receipt).map(|path| path.sequence),
            Ok(sequence)
        );
        let ack = terminal_path(b"custom-id", PathKind::Acknowledgement, sequence);
        assert_eq!(
            parse_membership_path(&ack).map(|path| path.sequence),
            Ok(sequence)
        );
    }
}

#[test]
fn path_shape_type_and_identifier_fail_closed() {
    assert_eq!(
        parse_membership_path(&[]),
        Err(IbcStorageError::WrongPathElementCount(0))
    );
    assert_eq!(
        parse_membership_path(&[vec![1; 10], vec![2; 10]]),
        Err(IbcStorageError::WrongPathElementCount(2))
    );
    assert_eq!(
        parse_membership_path(&[b"x\x01\0\0\0\0\0\0\0\x01".to_vec()]),
        Err(IbcStorageError::InvalidClientIdentifier)
    );
    assert!(parse_membership_path(&terminal_path(b"client-0", PathKind::Receipt, 1)).is_err());
    assert!(parse_receipt_path(&terminal_path(b"client-0", PathKind::Packet, 1)).is_err());
    assert!(parse_membership_path(&[Vec::new()]).is_err());
    assert!(parse_membership_path(&[b"client-0\x01\0\0\0\0\0\0\0".to_vec()]).is_err());
    assert!(parse_membership_path(&[b"client-01\x01\0\0\0\0\0\0\0\x01".to_vec()]).is_err());
    assert!(parse_membership_path(&[b"channel-0\x01\0\0\0\0\0\0\0\x01".to_vec()]).is_err());
    assert!(parse_membership_path(&[b"client-0\x04\0\0\0\0\0\0\0\x01".to_vec()]).is_err());
}
