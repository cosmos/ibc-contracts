use sha2::{Digest, Sha256};

const KECCAK_RATE_BYTES: usize = 136;

#[must_use]
pub fn keccak256(bytes: &[u8]) -> [u8; 32] {
    let mut state = [0_u64; 25];
    let mut chunks = bytes.chunks_exact(KECCAK_RATE_BYTES);
    for chunk in &mut chunks {
        absorb(&mut state, chunk);
        keccak_permute(&mut state);
    }

    let remainder = chunks.remainder();
    absorb(&mut state, remainder);
    xor_byte(&mut state, remainder.len(), 0x01);
    xor_byte(&mut state, KECCAK_RATE_BYTES - 1, 0x80);
    keccak_permute(&mut state);

    let mut output = [0_u8; 32];
    for (output_chunk, lane) in output.chunks_exact_mut(8).zip(state.iter()) {
        output_chunk.copy_from_slice(&lane.to_le_bytes());
    }
    output
}

fn absorb(state: &mut [u64; 25], block: &[u8]) {
    for (index, byte) in block.iter().enumerate() {
        xor_byte(state, index, *byte);
    }
}

fn xor_byte(state: &mut [u64; 25], index: usize, byte: u8) {
    let lane = index / 8;
    let shift = (index % 8) * 8;
    state[lane] ^= u64::from(byte) << shift;
}

#[cfg(target_os = "zkvm")]
fn keccak_permute(state: &mut [u64; 25]) {
    sp1_zkvm::syscalls::syscall_keccak_permute(core::ptr::from_mut(state));
}

#[cfg(not(target_os = "zkvm"))]
fn keccak_permute(state: &mut [u64; 25]) {
    tiny_keccak::keccakf(state);
}

#[must_use]
pub fn sha256(bytes: &[u8]) -> [u8; 32] {
    Sha256::digest(bytes).into()
}

#[cfg(test)]
mod tests {
    use super::{keccak256, sha256};
    use tiny_keccak::{Hasher, Keccak};

    fn reference_keccak256(bytes: &[u8]) -> [u8; 32] {
        let mut output = [0_u8; 32];
        let mut hasher = Keccak::v256();
        hasher.update(bytes);
        hasher.finalize(&mut output);
        output
    }

    #[test]
    fn public_hash_vectors() {
        assert_eq!(
            hex::encode(keccak256(b"")),
            "c5d2460186f7233c927e7db2dcc703c0e500b653ca82273b7bfad8045d85a470"
        );
        assert_eq!(
            hex::encode(keccak256(b"abc")),
            "4e03657aea45a94fc7d47ba826c8d667c0d1e6e33a64a036ec44f58fa12d6c45"
        );
        assert_eq!(
            hex::encode(sha256(b"")),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            hex::encode(sha256(b"abc")),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_ne!(keccak256(b"abc"), sha256(b"abc"));
    }

    #[test]
    fn accelerated_sponge_matches_reference_across_rate_boundaries() {
        for length in [0, 1, 31, 32, 55, 56, 135, 136, 137, 271, 272, 273, 1024] {
            let input = (0..length)
                .map(|index| u8::try_from(index % 251).unwrap_or(0))
                .collect::<Vec<_>>();
            assert_eq!(keccak256(&input), reference_keccak256(&input));
        }
    }
}
