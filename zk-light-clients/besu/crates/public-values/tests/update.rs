use besu_public_values::{decode_update, encode_update, PublicValuesError};
use besu_zk_types::{UpdatePublicValues, SCHEMA_VERSION};

fn vector() -> UpdatePublicValues {
    UpdatePublicValues {
        schema_version: SCHEMA_VERSION,
        trusted_height: 7,
        trusted_validator_set_hash: [0x11; 32],
        verified_height: 9,
        state_root: [0x22; 32],
        block_timestamp: 1_700_000_001,
        new_validator_set_hash: [0x33; 32],
    }
}

#[test]
fn update_roundtrip_is_exact() {
    let value = vector();
    let encoded = encode_update(&value);
    assert_eq!(encoded.len(), 7 * 32);
    assert_eq!(decode_update(&encoded), Ok(value));
}

#[test]
fn update_rejects_schema_truncation_and_trailing_bytes() {
    let mut wrong_schema = vector();
    wrong_schema.schema_version = 2;
    assert_eq!(
        decode_update(&encode_update(&wrong_schema)),
        Err(PublicValuesError::UnsupportedSchema(2))
    );

    let encoded = encode_update(&vector());
    assert!(matches!(
        decode_update(&encoded[..encoded.len() - 1]),
        Err(PublicValuesError::MalformedAbi)
    ));
    let mut trailing = encoded;
    trailing.push(0);
    assert!(decode_update(&trailing).is_err());
}

#[test]
fn update_integer_and_fixed_byte_boundaries_roundtrip() {
    for height in [0, 1, u64::MAX] {
        let value = UpdatePublicValues {
            schema_version: SCHEMA_VERSION,
            trusted_height: height,
            trusted_validator_set_hash: [0; 32],
            verified_height: u64::MAX,
            state_root: [0xff; 32],
            block_timestamp: height,
            new_validator_set_hash: [0xff; 32],
        };
        assert_eq!(decode_update(&encode_update(&value)), Ok(value));
    }
}
