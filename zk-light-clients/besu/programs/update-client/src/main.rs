#![no_main]

use besu_public_values::encode_update;
use besu_qbft::verify_update;
use besu_zk_types::UpdateWitness;

sp1_zkvm::entrypoint!(main);

pub fn main() {
    let witness = sp1_zkvm::io::read::<UpdateWitness>();
    let Ok(public_values) = verify_update(&witness) else {
        sp1_zkvm::syscalls::syscall_halt(1)
    };
    sp1_zkvm::io::commit_slice(&encode_update(&public_values));
}
