use besu_qbft::{validate_validator_set, validator_set_hash, QbftError};

#[test]
fn validator_set_is_strictly_canonical_and_committed_in_order() {
    let one = [[1_u8; 20]];
    let many = [[1_u8; 20], [2_u8; 20], [3_u8; 20]];
    assert!(validate_validator_set(&one).is_ok());
    assert!(validate_validator_set(&many).is_ok());
    assert_ne!(validator_set_hash(&one), validator_set_hash(&many));
    assert_eq!(
        validate_validator_set(&[[2; 20], [1; 20]]),
        Err(QbftError::UnsortedValidators)
    );
    assert_eq!(
        validate_validator_set(&[[1; 20], [1; 20]]),
        Err(QbftError::DuplicateValidator)
    );
    assert_eq!(
        validate_validator_set(&[[0; 20]]),
        Err(QbftError::ZeroValidator)
    );
    assert_eq!(
        validate_validator_set(&[]),
        Err(QbftError::EmptyValidatorSet)
    );
    assert_eq!(
        validator_set_hash(&one).map(hex::encode),
        Ok("e9541b8c2b9e787532e847022ccd1f0aee2394cd5d2b64904ba886948fa932ca".to_owned())
    );
    assert_ne!(
        validator_set_hash(&[[1; 20], [2; 20]]),
        validator_set_hash(&[[1; 20], [3; 20]])
    );
}
