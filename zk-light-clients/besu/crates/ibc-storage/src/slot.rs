use besu_zk_types::{keccak256, IBCSTORE_STORAGE_SLOT};

#[must_use]
pub fn logical_storage_slot(terminal_path: &[u8]) -> [u8; 32] {
    let hashed_path = keccak256(terminal_path);
    let mut mapping_preimage = [0_u8; 64];
    mapping_preimage[..32].copy_from_slice(&hashed_path);
    mapping_preimage[32..].copy_from_slice(&IBCSTORE_STORAGE_SLOT);
    keccak256(&mapping_preimage)
}
