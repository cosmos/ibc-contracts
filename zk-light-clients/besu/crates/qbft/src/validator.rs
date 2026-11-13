use alloc::vec::Vec;

use besu_zk_types::{is_zero_address, keccak256, Address, VALIDATOR_SET_DOMAIN};

use crate::QbftError;

pub fn validate_validator_set(validators: &[Address]) -> Result<(), QbftError> {
    if validators.is_empty() {
        return Err(QbftError::EmptyValidatorSet);
    }
    u32::try_from(validators.len()).map_err(|_| QbftError::TooManyValidators)?;
    let mut previous: Option<&Address> = None;
    for validator in validators {
        if is_zero_address(validator) {
            return Err(QbftError::ZeroValidator);
        }
        if let Some(earlier) = previous {
            match validator.cmp(earlier) {
                core::cmp::Ordering::Less => return Err(QbftError::UnsortedValidators),
                core::cmp::Ordering::Equal => return Err(QbftError::DuplicateValidator),
                core::cmp::Ordering::Greater => {}
            }
        }
        previous = Some(validator);
    }
    Ok(())
}

pub fn validator_set_hash(validators: &[Address]) -> Result<[u8; 32], QbftError> {
    validate_validator_set(validators)?;
    let count = u32::try_from(validators.len()).map_err(|_| QbftError::TooManyValidators)?;
    let validator_bytes = validators
        .len()
        .checked_mul(20)
        .ok_or(QbftError::ArithmeticOverflow)?;
    let capacity = VALIDATOR_SET_DOMAIN
        .len()
        .checked_add(4)
        .and_then(|length| length.checked_add(validator_bytes))
        .ok_or(QbftError::ArithmeticOverflow)?;
    let mut preimage = Vec::with_capacity(capacity);
    preimage.extend_from_slice(VALIDATOR_SET_DOMAIN);
    preimage.extend_from_slice(&count.to_be_bytes());
    for validator in validators {
        preimage.extend_from_slice(validator);
    }
    Ok(keccak256(&preimage))
}
