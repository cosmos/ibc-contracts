use besu_zk_types::{
    is_zero_address, MembershipPublicValues, MembershipWitness, NonMembershipPublicValues,
    NonMembershipWitness, MEMBERSHIP_SCHEMA_VERSION, SCHEMA_VERSION,
};
use evm_mpt::{
    verify_account_storage_root, verify_storage_membership_expected, verify_storage_non_membership,
};

use crate::{
    logical_storage_slot,
    path::{parse_membership_path, parse_receipt_path, terminal_path_digest},
    path_digest, IbcStorageError,
};

/// Verifies the complete Router account and commitment-slot inclusion statement.
pub fn verify_membership(
    witness: &MembershipWitness,
) -> Result<MembershipPublicValues, IbcStorageError> {
    validate_membership_common(witness)?;
    let storage_root = verify_account_storage_root(
        witness.state_root,
        witness.source_router,
        &witness.account_proof,
    )?;
    let mut leaves = alloc::vec::Vec::with_capacity(witness.entries.len());
    let mut previous = None;
    for entry in &witness.entries {
        let terminal = parse_membership_path(&entry.path)?;
        let digest = path_digest(&entry.path);
        if let Some(previous_digest) = previous {
            if digest == previous_digest {
                return Err(IbcStorageError::DuplicatePathDigest);
            }
            if digest < previous_digest {
                return Err(IbcStorageError::NonCanonicalBatchOrder);
            }
        }
        previous = Some(digest);
        let slot = logical_storage_slot(terminal.raw);
        verify_storage_membership_expected(
            storage_root,
            slot,
            &entry.storage_proof,
            &entry.expected_commitment,
        )
        .map_err(|error| match error {
            evm_mpt::MptError::StorageAbsent => IbcStorageError::MissingCommitment,
            evm_mpt::MptError::StorageValueMismatch => IbcStorageError::CommitmentMismatch,
            other => IbcStorageError::Mpt(other),
        })?;
        leaves.push(batch_merkle::membership_leaf(
            digest,
            entry.expected_commitment,
        ));
    }
    let entry_count =
        u32::try_from(witness.entries.len()).map_err(|_| IbcStorageError::EntryCountOverflow)?;
    let batch_root = batch_merkle::root(&leaves)?;

    Ok(MembershipPublicValues {
        schema_version: witness.schema_version,
        proof_height: witness.proof_height,
        state_root: witness.state_root,
        source_router: witness.source_router,
        batch_root,
        entry_count,
    })
}

fn validate_membership_common(witness: &MembershipWitness) -> Result<(), IbcStorageError> {
    if witness.schema_version != MEMBERSHIP_SCHEMA_VERSION {
        return Err(IbcStorageError::UnsupportedSchemaVersion(
            witness.schema_version,
        ));
    }
    batch_merkle::validate_entry_count(witness.entries.len())?;
    if witness.proof_height == 0 {
        return Err(IbcStorageError::ZeroProofHeight);
    }
    if is_zero_address(&witness.source_router) {
        return Err(IbcStorageError::ZeroSourceRouter);
    }
    Ok(())
}

/// Verifies the complete Router account inclusion and receipt-slot exclusion statement.
pub fn verify_non_membership(
    witness: &NonMembershipWitness,
) -> Result<NonMembershipPublicValues, IbcStorageError> {
    if witness.schema_version != SCHEMA_VERSION {
        return Err(IbcStorageError::UnsupportedSchemaVersion(
            witness.schema_version,
        ));
    }
    if witness.proof_height == 0 {
        return Err(IbcStorageError::ZeroProofHeight);
    }
    if is_zero_address(&witness.source_router) {
        return Err(IbcStorageError::ZeroSourceRouter);
    }
    let terminal = parse_receipt_path(&witness.path)?;
    let slot = logical_storage_slot(terminal.raw);
    let storage_root = verify_account_storage_root(
        witness.state_root,
        witness.source_router,
        &witness.proof.account_proof,
    )?;
    verify_storage_non_membership(storage_root, slot, &witness.proof.storage_proof)?;

    Ok(NonMembershipPublicValues {
        schema_version: witness.schema_version,
        proof_height: witness.proof_height,
        state_root: witness.state_root,
        source_router: witness.source_router,
        path_digest: terminal_path_digest(terminal.raw)?,
    })
}
