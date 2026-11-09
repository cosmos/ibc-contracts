use crate::QbftError;

pub fn target_quorum(validator_count: usize) -> Result<usize, QbftError> {
    if validator_count == 0 {
        return Err(QbftError::EmptyValidatorSet);
    }
    Ok(validator_count - validator_count / 3)
}

pub fn trusted_overlap_threshold(validator_count: usize) -> Result<usize, QbftError> {
    if validator_count == 0 {
        return Err(QbftError::EmptyValidatorSet);
    }
    validator_count
        .checked_div(3)
        .and_then(|value| value.checked_add(1))
        .ok_or(QbftError::ArithmeticOverflow)
}
