use besu_zk_types::{keccak256, IBCSTORE_STORAGE_SLOT};
use ibc_storage::{logical_storage_slot, path_digest, terminal_path, PathKind};

#[test]
fn slot_and_digest_bind_every_path_byte() {
    let packet = terminal_path(b"client-0", PathKind::Packet, 1);
    let ack = terminal_path(b"client-0", PathKind::Acknowledgement, 1);
    assert_ne!(
        logical_storage_slot(&packet[0]),
        logical_storage_slot(&ack[0])
    );
    assert_ne!(path_digest(&packet), path_digest(&ack));
    assert_eq!(
        hex::encode(logical_storage_slot(&packet[0])),
        "79a5307f4993e7eb3894d3aafa592f86bb1b605ca1a506e595f1874343b9a987"
    );
    assert_eq!(
        hex::encode(path_digest(&packet)),
        "96f56efbd67677c590346e37da4be678ec7e2dde8597350558f7ff406e930598"
    );
}

#[test]
fn slot_binds_sequence_namespace_and_exact_mapping_preimage() {
    let packet_one = terminal_path(b"client-0", PathKind::Packet, 1);
    let packet_two = terminal_path(b"client-0", PathKind::Packet, 2);
    assert_ne!(
        logical_storage_slot(&packet_one[0]),
        logical_storage_slot(&packet_two[0])
    );

    let hashed_path = keccak256(&packet_one[0]);
    let mut canonical = [0_u8; 64];
    canonical[..32].copy_from_slice(&hashed_path);
    canonical[32..].copy_from_slice(&IBCSTORE_STORAGE_SLOT);
    assert_eq!(keccak256(&canonical), logical_storage_slot(&packet_one[0]));

    let mut changed_namespace = canonical;
    changed_namespace[63] ^= 1;
    assert_ne!(keccak256(&changed_namespace), keccak256(&canonical));

    let mut shifted_word = [0_u8; 64];
    shifted_word[1..33].copy_from_slice(&hashed_path);
    shifted_word[33..].copy_from_slice(&IBCSTORE_STORAGE_SLOT[..31]);
    assert_ne!(keccak256(&shifted_word), keccak256(&canonical));
}
