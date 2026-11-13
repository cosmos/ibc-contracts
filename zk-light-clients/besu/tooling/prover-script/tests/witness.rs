use besu_zk_types::{MembershipWitness, NonMembershipWitness, UpdateWitness};
use prover_script::golden::{membership_witness, non_membership_witness, update_witness};
use serde::Serialize;
use sp1_sdk::SP1Stdin;

fn stdin_bytes<T: Serialize>(witness: &T) -> Vec<Vec<u8>> {
    let mut stdin = SP1Stdin::new();
    stdin.write(witness);
    stdin.buffer
}

#[test]
fn normalized_witnesses_are_structurally_deterministic() -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(update_witness()?, update_witness()?);
    assert_eq!(membership_witness()?, membership_witness()?);
    assert_eq!(non_membership_witness(), non_membership_witness());
    Ok(())
}

#[test]
fn canonical_sp1_stdin_is_byte_identical_for_identical_witnesses(
) -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(
        stdin_bytes(&update_witness()?),
        stdin_bytes(&update_witness()?)
    );
    assert_eq!(
        stdin_bytes(&membership_witness()?),
        stdin_bytes(&membership_witness()?)
    );
    assert_eq!(
        stdin_bytes(&non_membership_witness()),
        stdin_bytes(&non_membership_witness())
    );
    Ok(())
}

#[test]
fn canonical_sp1_stdin_roundtrips_nested_witness_vectors() -> Result<(), Box<dyn std::error::Error>>
{
    let update = update_witness()?;
    let membership = membership_witness()?;
    let non_membership = non_membership_witness();

    let mut update_stdin = SP1Stdin::new();
    update_stdin.write(&update);
    assert_eq!(update_stdin.read::<UpdateWitness>(), update);

    let mut membership_stdin = SP1Stdin::new();
    membership_stdin.write(&membership);
    assert_eq!(membership_stdin.read::<MembershipWitness>(), membership);

    let mut non_membership_stdin = SP1Stdin::new();
    non_membership_stdin.write(&non_membership);
    assert_eq!(
        non_membership_stdin.read::<NonMembershipWitness>(),
        non_membership
    );
    Ok(())
}
