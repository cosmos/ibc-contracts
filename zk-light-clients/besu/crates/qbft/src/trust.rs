use alloc::{vec, vec::Vec};

use besu_zk_types::{UpdatePublicValues, UpdateWitness, SCHEMA_VERSION};

use crate::{
    extra_data::parse_validated_extra_data, signing::committed_seal_digest_validated,
    target_quorum, trusted_overlap_threshold, validate_optional_tail, validator_set_hash,
    QbftError,
};

pub fn verify_update(witness: &UpdateWitness) -> Result<UpdatePublicValues, QbftError> {
    if witness.schema_version != SCHEMA_VERSION {
        return Err(QbftError::UnsupportedSchema(witness.schema_version));
    }
    if witness.trusted_height == 0 {
        return Err(QbftError::ZeroTrustedHeight);
    }
    let trusted_hash = validator_set_hash(&witness.trusted_validators)?;
    if trusted_hash != witness.trusted_validator_set_hash {
        return Err(QbftError::TrustedValidatorHashMismatch);
    }
    if witness.target_header.number <= witness.trusted_height {
        return Err(QbftError::TargetHeightNotNewer);
    }
    if witness.target_header.timestamp == 0 {
        return Err(QbftError::ZeroTargetTimestamp);
    }
    validate_optional_tail(&witness.target_header)?;
    let parsed = parse_validated_extra_data(&witness.target_header.extra_data)?;
    let extra = &parsed.signing_fields;
    let digest = committed_seal_digest_validated(&witness.target_header, extra)?;
    let mut seen_validators: Vec<u8> = vec![0; extra.validators.len()];
    let mut signer_count = 0_usize;
    let mut trusted_signers = 0_usize;
    for seal in &parsed.committed_seals {
        let signer = seal.recover_address(digest)?;
        let validator_index = extra
            .validators
            .binary_search(&signer)
            .map_err(|_| QbftError::NonValidatorSigner)?;
        let seen = seen_validators
            .get_mut(validator_index)
            .ok_or(QbftError::ArithmeticOverflow)?;
        if *seen != 0 {
            return Err(QbftError::DuplicateSigner);
        }
        *seen = 1;
        signer_count = signer_count
            .checked_add(1)
            .ok_or(QbftError::ArithmeticOverflow)?;
        if witness.trusted_validators.binary_search(&signer).is_ok() {
            trusted_signers = trusted_signers
                .checked_add(1)
                .ok_or(QbftError::ArithmeticOverflow)?;
        }
    }
    let target_required = target_quorum(extra.validators.len())?;
    if signer_count < target_required {
        return Err(QbftError::InsufficientTargetQuorum {
            required: target_required,
            actual: signer_count,
        });
    }
    let trusted_required = trusted_overlap_threshold(witness.trusted_validators.len())?;
    if trusted_signers < trusted_required {
        return Err(QbftError::InsufficientTrustedOverlap {
            required: trusted_required,
            actual: trusted_signers,
        });
    }
    Ok(UpdatePublicValues {
        schema_version: SCHEMA_VERSION,
        trusted_height: witness.trusted_height,
        trusted_validator_set_hash: witness.trusted_validator_set_hash,
        verified_height: witness.target_header.number,
        state_root: witness.target_header.state_root,
        block_timestamp: witness.target_header.timestamp,
        new_validator_set_hash: validator_set_hash(&extra.validators)?,
    })
}
