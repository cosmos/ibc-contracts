pub const SCHEMA_VERSION: u32 = 1;
pub const MEMBERSHIP_SCHEMA_VERSION: u32 = 2;
pub const MAX_BATCH_MESSAGES: usize = 32;
pub const MEMBERSHIP_ENVELOPE_FULL: u8 = 1;
pub const MEMBERSHIP_ENVELOPE_CACHED: u8 = 2;
pub const QBFT_VANITY_BYTES: usize = 32;
pub const QBFT_COMMIT_SEAL_BYTES: usize = 65;
pub const IBC_PATH_ELEMENTS: usize = 1;
pub const PACKET_PATH_TYPE: u8 = 0x01;
pub const RECEIPT_PATH_TYPE: u8 = 0x02;
pub const ACKNOWLEDGEMENT_PATH_TYPE: u8 = 0x03;

pub const VALIDATOR_SET_DOMAIN: &[u8] = b"BESU_QBFT_VALIDATOR_SET_V1";

pub const IBCSTORE_STORAGE_SLOT: [u8; 32] = [
    0x12, 0x60, 0x94, 0x44, 0x89, 0x27, 0x29, 0x88, 0xd9, 0xdf, 0x28, 0x51, 0x49, 0xb5, 0xaa, 0x1b,
    0x0f, 0x48, 0xf2, 0x13, 0x6d, 0x6f, 0x41, 0x61, 0x59, 0xf8, 0x40, 0xa3, 0xe0, 0x74, 0x76, 0x00,
];
