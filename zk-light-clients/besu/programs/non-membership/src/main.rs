#![no_main]

use besu_public_values::encode_non_membership;
use besu_zk_types::NonMembershipWitness;
use ibc_storage::verify_non_membership;

sp1_zkvm::entrypoint!(main);

pub fn main() {
    let witness = sp1_zkvm::io::read::<NonMembershipWitness>();
    let Ok(public_values) = verify_non_membership(&witness) else {
        sp1_zkvm::syscalls::syscall_halt(1)
    };
    sp1_zkvm::io::commit_slice(&encode_non_membership(&public_values));
}
