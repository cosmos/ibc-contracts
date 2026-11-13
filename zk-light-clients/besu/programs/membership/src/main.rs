#![no_main]

use besu_public_values::encode_membership;
use besu_zk_types::MembershipWitness;
use ibc_storage::verify_membership;

sp1_zkvm::entrypoint!(main);

pub fn main() {
    let witness = sp1_zkvm::io::read::<MembershipWitness>();
    let Ok(public_values) = verify_membership(&witness) else {
        sp1_zkvm::syscalls::syscall_halt(1)
    };
    sp1_zkvm::io::commit_slice(&encode_membership(&public_values));
}
