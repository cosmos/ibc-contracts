use alloc::vec::Vec;

#[must_use]
pub fn bytes_to_nibbles(bytes: &[u8]) -> Vec<u8> {
    let mut nibbles = Vec::with_capacity(bytes.len().saturating_mul(2));
    for byte in bytes {
        nibbles.push(byte >> 4);
        nibbles.push(byte & 0x0f);
    }
    nibbles
}

#[must_use]
pub(crate) fn secure_key_nibbles(bytes: [u8; 32]) -> [u8; 64] {
    let mut nibbles = [0_u8; 64];
    for (index, byte) in bytes.iter().enumerate() {
        nibbles[index * 2] = byte >> 4;
        nibbles[index * 2 + 1] = byte & 0x0f;
    }
    nibbles
}
