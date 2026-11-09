use besu_public_values::{decode_non_membership, encode_non_membership, PublicValuesError};
use besu_zk_types::{NonMembershipPublicValues, SCHEMA_VERSION};

#[test]
fn non_membership_roundtrip_is_exact() {
    let value = NonMembershipPublicValues {
        schema_version: SCHEMA_VERSION,
        proof_height: 1,
        state_root: [0x11; 32],
        source_router: [0x22; 20],
        path_digest: [0x33; 32],
    };
    let encoded = encode_non_membership(&value);
    assert_eq!(encoded.len(), 5 * 32);
    assert_eq!(&encoded[28..32], &SCHEMA_VERSION.to_be_bytes());
    assert_eq!(&encoded[56..64], &1_u64.to_be_bytes());
    assert_eq!(&encoded[64..96], &[0x11; 32]);
    assert_eq!(&encoded[96..108], &[0; 12]);
    assert_eq!(&encoded[108..128], &[0x22; 20]);
    assert_eq!(&encoded[128..160], &[0x33; 32]);
    assert_eq!(decode_non_membership(&encoded), Ok(value));
}

#[test]
fn non_membership_boundaries_and_malformed_encodings_fail_closed() {
    for height in [0, 1, u64::MAX] {
        let value = NonMembershipPublicValues {
            schema_version: SCHEMA_VERSION,
            proof_height: height,
            state_root: [0xff; 32],
            source_router: [0xff; 20],
            path_digest: [0; 32],
        };
        assert_eq!(
            decode_non_membership(&encode_non_membership(&value)),
            Ok(value)
        );
    }

    let mut wrong_schema = NonMembershipPublicValues {
        schema_version: 2,
        proof_height: 1,
        state_root: [1; 32],
        source_router: [2; 20],
        path_digest: [3; 32],
    };
    assert_eq!(
        decode_non_membership(&encode_non_membership(&wrong_schema)),
        Err(PublicValuesError::UnsupportedSchema(2))
    );
    wrong_schema.schema_version = SCHEMA_VERSION;
    let encoded = encode_non_membership(&wrong_schema);
    assert!(decode_non_membership(&encoded[..encoded.len() - 1]).is_err());
    let mut trailing = encoded;
    trailing.push(0);
    assert!(decode_non_membership(&trailing).is_err());
}
