use besu_public_values::{decode_membership, encode_membership, PublicValuesError};
use besu_zk_types::{MembershipPublicValues, MEMBERSHIP_SCHEMA_VERSION};

#[test]
fn membership_roundtrip_preserves_address_order() {
    let value = MembershipPublicValues {
        schema_version: MEMBERSHIP_SCHEMA_VERSION,
        proof_height: u64::MAX,
        state_root: [0x11; 32],
        source_router: [0x22; 20],
        batch_root: [0x33; 32],
        entry_count: 32,
    };
    let encoded = encode_membership(&value);
    assert_eq!(encoded.len(), 6 * 32);
    assert_eq!(&encoded[28..32], &MEMBERSHIP_SCHEMA_VERSION.to_be_bytes());
    assert_eq!(&encoded[56..64], &u64::MAX.to_be_bytes());
    assert_eq!(&encoded[64..96], &[0x11; 32]);
    assert_eq!(&encoded[96..108], &[0; 12]);
    assert_eq!(&encoded[108..128], &[0x22; 20]);
    assert_eq!(&encoded[128..160], &[0x33; 32]);
    assert_eq!(&encoded[188..192], &32_u32.to_be_bytes());
    assert_eq!(decode_membership(&encoded), Ok(value));
}

#[test]
fn membership_boundaries_and_malformed_encodings_fail_closed() {
    for height in [0, 1, u64::MAX] {
        let value = MembershipPublicValues {
            schema_version: MEMBERSHIP_SCHEMA_VERSION,
            proof_height: height,
            state_root: [0xff; 32],
            source_router: [0xff; 20],
            batch_root: [0xff; 32],
            entry_count: 1,
        };
        assert_eq!(decode_membership(&encode_membership(&value)), Ok(value));
    }

    let mut wrong_schema = MembershipPublicValues {
        schema_version: 1,
        proof_height: 1,
        state_root: [1; 32],
        source_router: [2; 20],
        batch_root: [3; 32],
        entry_count: 4,
    };
    assert_eq!(
        decode_membership(&encode_membership(&wrong_schema)),
        Err(PublicValuesError::UnsupportedSchema(1))
    );
    wrong_schema.schema_version = MEMBERSHIP_SCHEMA_VERSION;
    let encoded = encode_membership(&wrong_schema);
    assert!(decode_membership(&encoded[..encoded.len() - 1]).is_err());
    let mut trailing = encoded;
    trailing.push(0);
    assert!(decode_membership(&trailing).is_err());
}
