use crate::syscall_verify_sp1_proof;

/// Verifies the next proof in the proof input stream given a verification key digest and public
/// values digest. If the proof is invalid, the function will panic.
///
/// Enable this function by adding the `verify` feature to both the `sp1-lib` AND `sp1-zkvm` crates.
pub fn verify_sp1_proof(vk_digest: &[u32; 8], pv_digest: &[u8; 32]) {
    #[cfg(not(target_endian = "little"))]
    compile_error!("expected target to be little endian");
    // SAFETY: Arrays are always laid out in the obvious way. Any possible element value is
    // always valid.
    // Although not a safety invariant, note that the guest target is always little-endian,
    // which was just sanity-checked, so this will always have the expected behavior.
    let vk_digest = unsafe { core::mem::transmute::<[u32; 8], [u64; 4]>(*vk_digest) };
    let pv_digest = unsafe { core::mem::transmute::<[u8; 32], [u64; 4]>(*pv_digest) };

    unsafe {
        syscall_verify_sp1_proof(&vk_digest, &pv_digest);
    }
}
