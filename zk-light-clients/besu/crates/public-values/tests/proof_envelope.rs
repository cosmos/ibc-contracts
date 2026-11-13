use besu_public_values::{
    decode_membership_envelope, decode_proof_envelope, encode_membership_envelope,
    encode_proof_envelope, PublicValuesError,
};
use besu_zk_types::{
    MembershipBatchProofEnvelope, SP1ProofEnvelope, MEMBERSHIP_ENVELOPE_CACHED,
    MEMBERSHIP_ENVELOPE_FULL,
};

#[test]
fn proof_envelope_roundtrip_is_canonical() {
    let envelope = SP1ProofEnvelope {
        public_values: vec![1, 2, 3],
        proof_bytes: vec![4, 5, 6, 7],
    };
    let encoded = encode_proof_envelope(&envelope);
    assert_eq!(decode_proof_envelope(&encoded), Ok(envelope));

    let mut trailing = encoded;
    trailing.push(0);
    assert_eq!(
        decode_proof_envelope(&trailing),
        Err(PublicValuesError::NonCanonicalAbi)
    );
}

#[test]
fn membership_full_and_cached_envelopes_are_canonical_and_mode_strict() {
    let full = MembershipBatchProofEnvelope {
        mode: MEMBERSHIP_ENVELOPE_FULL,
        public_values: vec![1],
        proof_bytes: vec![2],
        batch_root: [3; 32],
        entry_count: 2,
        leaf_index: 0,
        merkle_proof: vec![[4; 32]],
    };
    assert_eq!(decode_membership_envelope(&encode_membership_envelope(&full)), Ok(full));

    let cached = MembershipBatchProofEnvelope {
        mode: MEMBERSHIP_ENVELOPE_CACHED,
        public_values: vec![],
        proof_bytes: vec![],
        batch_root: [3; 32],
        entry_count: 2,
        leaf_index: 1,
        merkle_proof: vec![[5; 32]],
    };
    assert_eq!(decode_membership_envelope(&encode_membership_envelope(&cached)), Ok(cached));

    let invalid = MembershipBatchProofEnvelope {
        mode: MEMBERSHIP_ENVELOPE_CACHED,
        public_values: vec![1],
        proof_bytes: vec![],
        batch_root: [0; 32],
        entry_count: 1,
        leaf_index: 0,
        merkle_proof: vec![],
    };
    assert_eq!(
        decode_membership_envelope(&encode_membership_envelope(&invalid)),
        Err(PublicValuesError::InvalidMembershipModePayload)
    );
}

#[test]
fn proof_envelope_rejects_empty_components() {
    let missing_public_values = SP1ProofEnvelope {
        public_values: Vec::new(),
        proof_bytes: vec![1],
    };
    assert_eq!(
        decode_proof_envelope(&encode_proof_envelope(&missing_public_values)),
        Err(PublicValuesError::EmptyPublicValues)
    );

    let missing_proof = SP1ProofEnvelope {
        public_values: vec![1],
        proof_bytes: Vec::new(),
    };
    assert_eq!(
        decode_proof_envelope(&encode_proof_envelope(&missing_proof)),
        Err(PublicValuesError::EmptyProof)
    );
}
