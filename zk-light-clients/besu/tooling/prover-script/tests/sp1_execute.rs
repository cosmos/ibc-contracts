use std::{error::Error, path::PathBuf};

use besu_public_values::{encode_membership, encode_non_membership, encode_update};
use besu_qbft::verify_update;
use besu_zk_types::{MembershipWitness, NonMembershipWitness, UpdateWitness, MEMBERSHIP_SCHEMA_VERSION, SCHEMA_VERSION};
use ibc_storage::{verify_membership, verify_non_membership};
use prover_script::{
    golden::{membership_witness, non_membership_witness, update_witness},
    prover::execute,
};

fn elf(name: &str) -> Result<Vec<u8>, std::io::Error> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join("artifacts")
        .join(name);
    std::fs::read(path)
}

#[tokio::test]
async fn all_three_guests_execute_canonical_shared_verifiers() -> Result<(), Box<dyn Error>> {
    let update = update_witness()?;
    let native_update = encode_update(&verify_update(&update)?);
    let zk_update = execute(&elf("update-client.elf")?, &update).await?;
    assert_eq!(zk_update.public_values, native_update);

    let membership = membership_witness()?;
    let native_membership = encode_membership(&verify_membership(&membership)?);
    let zk_membership = execute(&elf("membership.elf")?, &membership).await?;
    assert_eq!(zk_membership.public_values, native_membership);

    let non_membership = non_membership_witness();
    let native_non_membership = encode_non_membership(&verify_non_membership(&non_membership)?);
    let zk_non_membership = execute(&elf("non-membership.elf")?, &non_membership).await?;
    assert_eq!(zk_non_membership.public_values, native_non_membership);

    Ok(())
}

#[tokio::test]
async fn update_guest_rejects_each_major_native_rejection_class() -> Result<(), Box<dyn Error>> {
    let program = elf("update-client.elf")?;
    let valid = update_witness()?;
    let mut cases: Vec<UpdateWitness> = Vec::new();

    let mut invalid = valid.clone();
    invalid.schema_version = MEMBERSHIP_SCHEMA_VERSION + 1;
    cases.push(invalid);
    invalid = valid.clone();
    invalid.trusted_validator_set_hash[0] ^= 1;
    cases.push(invalid);
    invalid = valid.clone();
    invalid.target_header.number = invalid.trusted_height;
    cases.push(invalid);
    invalid = valid.clone();
    invalid.target_header.extra_data = vec![0xc0];
    cases.push(invalid);
    invalid = valid.clone();
    invalid.target_header.base_fee_per_gas = None;
    invalid.target_header.withdrawals_root = Some([1; 32]);
    cases.push(invalid);
    invalid = valid;
    let mut extra = besu_qbft::parse_extra_data(&invalid.target_header.extra_data)?;
    extra.committed_seals.clear();
    invalid.target_header.extra_data = besu_qbft::encode_extra_data(&extra, true, true);
    cases.push(invalid);

    for case in cases {
        assert!(verify_update(&case).is_err());
        assert!(execute(&program, &case).await.is_err());
    }
    Ok(())
}

#[tokio::test]
async fn membership_guest_rejects_each_major_native_rejection_class() -> Result<(), Box<dyn Error>>
{
    let program = elf("membership.elf")?;
    let valid = membership_witness()?;
    let mut cases: Vec<MembershipWitness> = Vec::new();

    let mut invalid = valid.clone();
    invalid.schema_version = SCHEMA_VERSION + 1;
    cases.push(invalid);
    invalid = valid.clone();
    invalid.entries[0].path.clear();
    cases.push(invalid);
    invalid = valid.clone();
    invalid.state_root[0] ^= 1;
    cases.push(invalid);
    invalid = valid.clone();
    invalid.entries[0].expected_commitment[0] ^= 1;
    cases.push(invalid);
    invalid = valid;
    invalid.entries[0].storage_proof.clear();
    cases.push(invalid);

    for case in cases {
        assert!(verify_membership(&case).is_err());
        assert!(execute(&program, &case).await.is_err());
    }
    Ok(())
}

#[tokio::test]
async fn non_membership_guest_rejects_each_major_native_rejection_class(
) -> Result<(), Box<dyn Error>> {
    let program = elf("non-membership.elf")?;
    let valid = non_membership_witness();
    let mut cases: Vec<NonMembershipWitness> = Vec::new();

    let mut invalid = valid.clone();
    invalid.schema_version = SCHEMA_VERSION + 1;
    cases.push(invalid);
    invalid = valid.clone();
    invalid.path.clear();
    cases.push(invalid);
    invalid = valid.clone();
    invalid.state_root[0] ^= 1;
    cases.push(invalid);
    invalid = valid;
    invalid.proof.account_proof.clear();
    cases.push(invalid);

    for case in cases {
        assert!(verify_non_membership(&case).is_err());
        assert!(execute(&program, &case).await.is_err());
    }
    Ok(())
}
