use std::{error::Error, io};

use besu_qbft::{
    committed_seal_digest, encode_extra_data, parse_extra_data, validator_set_hash, verify_update,
    QbftError, QbftExtraData, QbftVote, VoteKind,
};
use besu_zk_types::{keccak256, Address, QbftHeaderInput, UpdateWitness, SCHEMA_VERSION};
use k256::ecdsa::{Signature, SigningKey};

struct Fixture {
    witness: UpdateWitness,
    target_keys: Vec<(Address, SigningKey)>,
    other_keys: Vec<(Address, SigningKey)>,
}

fn address(key: &SigningKey) -> Result<Address, Box<dyn Error>> {
    let encoded = key.verifying_key().to_encoded_point(false);
    let coordinates = encoded
        .as_bytes()
        .get(1..)
        .ok_or_else(|| io::Error::other("missing uncompressed public-key prefix"))?;
    let hash = keccak256(coordinates);
    hash.get(12..)
        .ok_or_else(|| io::Error::other("invalid address hash"))?
        .try_into()
        .map_err(|_| io::Error::other("invalid address length").into())
}

fn keys(
    range: core::ops::RangeInclusive<u8>,
) -> Result<Vec<(Address, SigningKey)>, Box<dyn Error>> {
    let mut result = Vec::new();
    for byte in range {
        let key = SigningKey::from_slice(&[byte; 32])
            .map_err(|_| io::Error::other("invalid deterministic test key"))?;
        result.push((address(&key)?, key));
    }
    result.sort_by_key(|(address, _)| *address);
    Ok(result)
}

fn header() -> QbftHeaderInput {
    QbftHeaderInput {
        parent_hash: [1; 32],
        ommers_hash: [2; 32],
        beneficiary: [3; 20],
        state_root: [4; 32],
        transactions_root: [5; 32],
        receipts_root: [6; 32],
        logs_bloom: [7; 256],
        difficulty: [0; 32],
        number: 101,
        gas_limit: 30_000_000,
        gas_used: 12_345,
        timestamp: 1_700_000_101,
        extra_data: Vec::new(),
        mix_hash: [8; 32],
        nonce: [0; 8],
        base_fee_per_gas: Some([9; 32]),
        withdrawals_root: Some([10; 32]),
        blob_gas_used: Some(11),
        excess_blob_gas: Some(12),
        parent_beacon_block_root: Some([13; 32]),
        requests_hash: Some([14; 32]),
        block_access_list_hash: Some([15; 32]),
    }
}

fn sign(key: &SigningKey, digest: &[u8; 32]) -> Result<Vec<u8>, Box<dyn Error>> {
    let (signature, recovery_id) = key
        .sign_prehash_recoverable(digest)
        .map_err(|_| io::Error::other("test signing failed"))?;
    let mut seal = signature.to_bytes().to_vec();
    seal.push(recovery_id.to_byte());
    Ok(seal)
}

fn high_s_seal(key: &SigningKey, digest: &[u8; 32]) -> Result<Vec<u8>, Box<dyn Error>> {
    let (signature, recovery_id) = key
        .sign_prehash_recoverable(digest)
        .map_err(|_| io::Error::other("test signing failed"))?;
    let high_s = -*signature.s();
    let high_signature = Signature::from_scalars(signature.r().to_bytes(), high_s.to_bytes())
        .map_err(|_| io::Error::other("high-S test signature construction failed"))?;
    let mut seal = high_signature.to_bytes().to_vec();
    seal.push(recovery_id.to_byte() ^ 1);
    Ok(seal)
}

fn build_fixture() -> Result<Fixture, Box<dyn Error>> {
    let target_keys = keys(1..=4)?;
    let other_keys = keys(5..=8)?;
    let validators = target_keys
        .iter()
        .map(|(address, _)| *address)
        .collect::<Vec<_>>();
    let mut target_header = header();
    let mut extra = QbftExtraData {
        vanity: [0x42; 32],
        validators: validators.clone(),
        vote: Some(QbftVote {
            recipient: [0x99; 20],
            kind: VoteKind::Add,
        }),
        round: 7,
        committed_seals: Vec::new(),
    };
    target_header.extra_data = encode_extra_data(&extra, true, true);
    let digest = committed_seal_digest(&target_header)?;
    for (_, key) in target_keys.iter().take(3) {
        extra.committed_seals.push(sign(key, &digest)?);
    }
    target_header.extra_data = encode_extra_data(&extra, true, true);
    Ok(Fixture {
        witness: UpdateWitness {
            schema_version: SCHEMA_VERSION,
            trusted_height: 100,
            trusted_validator_set_hash: validator_set_hash(&validators)?,
            trusted_validators: validators,
            target_header,
        },
        target_keys,
        other_keys,
    })
}

fn witness_with_counts(
    target_count: u8,
    seal_count: usize,
    trusted_validators: Vec<Address>,
) -> Result<UpdateWitness, Box<dyn Error>> {
    let target_keys = keys(1..=target_count)?;
    let validators = target_keys
        .iter()
        .map(|(address, _)| *address)
        .collect::<Vec<_>>();
    let mut target_header = header();
    let mut extra = QbftExtraData {
        vanity: [0x42; 32],
        validators,
        vote: None,
        round: 0,
        committed_seals: Vec::new(),
    };
    target_header.extra_data = encode_extra_data(&extra, true, true);
    let digest = committed_seal_digest(&target_header)?;
    for (_, key) in target_keys.iter().take(seal_count) {
        extra.committed_seals.push(sign(key, &digest)?);
    }
    target_header.extra_data = encode_extra_data(&extra, true, true);
    Ok(UpdateWitness {
        schema_version: SCHEMA_VERSION,
        trusted_height: 100,
        trusted_validator_set_hash: validator_set_hash(&trusted_validators)?,
        trusted_validators,
        target_header,
    })
}

fn trusted_set_with_overlap(
    target_keys: &[(Address, SigningKey)],
    count: usize,
    overlap: usize,
) -> Result<Vec<Address>, Box<dyn Error>> {
    let mut trusted = target_keys
        .iter()
        .take(overlap)
        .map(|(address, _)| *address)
        .collect::<Vec<_>>();
    if trusted.len() < count {
        let needed = count - trusted.len();
        trusted.extend(
            keys(100..=u8::try_from(99 + needed)?)?
                .into_iter()
                .map(|(address, _)| address),
        );
    }
    trusted.sort_unstable();
    Ok(trusted)
}

#[test]
fn verifies_a_signed_update_and_preserves_round_in_digest() -> Result<(), Box<dyn Error>> {
    let fixture = build_fixture()?;
    let public_values = verify_update(&fixture.witness)?;
    assert_eq!(public_values.verified_height, 101);
    assert_eq!(public_values.state_root, [4; 32]);

    let mut changed = fixture.witness.target_header.clone();
    let mut extra = parse_extra_data(&changed.extra_data)?;
    extra.round = 0;
    changed.extra_data = encode_extra_data(&extra, true, true);
    assert_ne!(
        committed_seal_digest(&fixture.witness.target_header)?,
        committed_seal_digest(&changed)?
    );
    Ok(())
}

#[test]
fn rejects_schema_height_time_hash_validator_and_header_structure() -> Result<(), Box<dyn Error>> {
    let fixture = build_fixture()?;
    let valid = fixture.witness;

    let mut witness = valid.clone();
    witness.schema_version = SCHEMA_VERSION + 1;
    assert_eq!(
        verify_update(&witness),
        Err(QbftError::UnsupportedSchema(SCHEMA_VERSION + 1))
    );
    witness = valid.clone();
    witness.trusted_height = 0;
    assert_eq!(verify_update(&witness), Err(QbftError::ZeroTrustedHeight));
    witness = valid.clone();
    witness.trusted_validator_set_hash[0] ^= 1;
    assert_eq!(
        verify_update(&witness),
        Err(QbftError::TrustedValidatorHashMismatch)
    );

    for trusted in [
        vec![],
        vec![[0; 20]],
        vec![[2; 20], [1; 20]],
        vec![[1; 20], [1; 20]],
    ] {
        witness = valid.clone();
        witness.trusted_validators = trusted;
        assert!(verify_update(&witness).is_err());
    }

    witness = valid.clone();
    witness.target_header.number = witness.trusted_height;
    assert_eq!(
        verify_update(&witness),
        Err(QbftError::TargetHeightNotNewer)
    );
    witness = valid.clone();
    witness.target_header.timestamp = 0;
    assert_eq!(verify_update(&witness), Err(QbftError::ZeroTargetTimestamp));
    witness = valid.clone();
    witness.target_header.base_fee_per_gas = None;
    assert_eq!(verify_update(&witness), Err(QbftError::InvalidOptionalTail));
    witness = valid.clone();
    witness.target_header.extra_data = vec![0xc0];
    assert!(verify_update(&witness).is_err());

    let extra = parse_extra_data(&valid.target_header.extra_data)?;
    for validators in [
        vec![],
        vec![[0; 20]],
        vec![[2; 20], [1; 20]],
        vec![[1; 20], [1; 20]],
    ] {
        witness = valid.clone();
        let mut malformed = extra.clone();
        malformed.validators = validators;
        witness.target_header.extra_data = encode_extra_data(&malformed, true, true);
        assert!(verify_update(&witness).is_err());
    }
    Ok(())
}

#[test]
fn high_s_form_is_valid_but_cannot_add_quorum_weight() -> Result<(), Box<dyn Error>> {
    let mut fixture = build_fixture()?;
    let digest = committed_seal_digest(&fixture.witness.target_header)?;
    let mut extra = parse_extra_data(&fixture.witness.target_header.extra_data)?;
    let (_, first_key) = fixture
        .target_keys
        .first()
        .ok_or_else(|| io::Error::other("missing target key"))?;
    let high = high_s_seal(first_key, &digest)?;
    let first = extra
        .committed_seals
        .first_mut()
        .ok_or_else(|| io::Error::other("missing seal"))?;
    *first = high.clone();
    fixture.witness.target_header.extra_data = encode_extra_data(&extra, true, true);
    verify_update(&fixture.witness)?;

    extra.committed_seals.push(high);
    fixture.witness.target_header.extra_data = encode_extra_data(&extra, true, true);
    assert_eq!(
        verify_update(&fixture.witness),
        Err(QbftError::DuplicateSigner)
    );
    Ok(())
}

#[test]
fn rejects_non_validator_duplicate_and_invalid_recovery_id() -> Result<(), Box<dyn Error>> {
    let mut fixture = build_fixture()?;
    let digest = committed_seal_digest(&fixture.witness.target_header)?;
    let mut extra = parse_extra_data(&fixture.witness.target_header.extra_data)?;
    let duplicate = extra
        .committed_seals
        .first()
        .ok_or_else(|| io::Error::other("missing seal"))?
        .clone();
    extra.committed_seals.push(duplicate);
    fixture.witness.target_header.extra_data = encode_extra_data(&extra, true, true);
    assert_eq!(
        verify_update(&fixture.witness),
        Err(QbftError::DuplicateSigner)
    );

    fixture = build_fixture()?;
    extra = parse_extra_data(&fixture.witness.target_header.extra_data)?;
    let (_, outsider) = fixture
        .other_keys
        .first()
        .ok_or_else(|| io::Error::other("missing outsider key"))?;
    extra.committed_seals.push(sign(outsider, &digest)?);
    fixture.witness.target_header.extra_data = encode_extra_data(&extra, true, true);
    assert_eq!(
        verify_update(&fixture.witness),
        Err(QbftError::NonValidatorSigner)
    );

    fixture = build_fixture()?;
    extra = parse_extra_data(&fixture.witness.target_header.extra_data)?;
    let seal = extra
        .committed_seals
        .first_mut()
        .ok_or_else(|| io::Error::other("missing seal"))?;
    let recovery = seal
        .get_mut(64)
        .ok_or_else(|| io::Error::other("missing recovery id"))?;
    *recovery = 27;
    fixture.witness.target_header.extra_data = encode_extra_data(&extra, true, true);
    assert_eq!(
        verify_update(&fixture.witness),
        Err(QbftError::InvalidRecoveryId(27))
    );
    Ok(())
}

#[test]
fn malformed_later_seals_are_rejected_before_signer_recovery() -> Result<(), Box<dyn Error>> {
    let fixture = build_fixture()?;
    let digest = committed_seal_digest(&fixture.witness.target_header)?;
    let mut extra = parse_extra_data(&fixture.witness.target_header.extra_data)?;
    let (_, outsider) = fixture
        .other_keys
        .first()
        .ok_or_else(|| io::Error::other("missing outsider key"))?;
    // If recovery ran while parsing, this first seal would fail as a non-validator.
    extra.committed_seals[0] = sign(outsider, &digest)?;
    let valid_last = extra
        .committed_seals
        .last()
        .ok_or_else(|| io::Error::other("missing seal"))?
        .clone();
    let mut zero_scalar = valid_last.clone();
    zero_scalar[..32].fill(0);
    zero_scalar[64] = 27;
    let mut invalid_recovery = valid_last;
    invalid_recovery[64] = 27;
    for (seal, expected_error) in [
        (vec![0; 64], QbftError::InvalidSealLength(64)),
        (zero_scalar, QbftError::InvalidSignatureScalar),
        (invalid_recovery, QbftError::InvalidRecoveryId(27)),
    ] {
        *extra
            .committed_seals
            .last_mut()
            .ok_or_else(|| io::Error::other("missing seal"))? = seal.clone();
        let mut witness = fixture.witness.clone();
        witness.target_header.extra_data = encode_extra_data(&extra, true, true);
        assert_eq!(besu_qbft::validate_seal(&seal), Err(expected_error.clone()));
        assert_eq!(
            besu_qbft::recover_address(digest, &seal),
            Err(expected_error.clone())
        );
        assert_eq!(
            parse_extra_data(&witness.target_header.extra_data),
            Err(expected_error.clone())
        );
        assert_eq!(verify_update(&witness), Err(expected_error));
    }
    Ok(())
}

#[test]
fn enforces_target_quorum_and_trusted_overlap_independently() -> Result<(), Box<dyn Error>> {
    let mut fixture = build_fixture()?;
    let mut extra = parse_extra_data(&fixture.witness.target_header.extra_data)?;
    extra.committed_seals.pop();
    fixture.witness.target_header.extra_data = encode_extra_data(&extra, true, true);
    assert_eq!(
        verify_update(&fixture.witness),
        Err(QbftError::InsufficientTargetQuorum {
            required: 3,
            actual: 2
        })
    );

    fixture = build_fixture()?;
    let first_target = fixture
        .target_keys
        .first()
        .ok_or_else(|| io::Error::other("missing target key"))?
        .0;
    let mut trusted = fixture
        .other_keys
        .iter()
        .take(3)
        .map(|(address, _)| *address)
        .collect::<Vec<_>>();
    trusted.push(first_target);
    trusted.sort_unstable();
    fixture.witness.trusted_validator_set_hash = validator_set_hash(&trusted)?;
    fixture.witness.trusted_validators = trusted;
    assert_eq!(
        verify_update(&fixture.witness),
        Err(QbftError::InsufficientTrustedOverlap {
            required: 2,
            actual: 1
        })
    );
    Ok(())
}

#[test]
fn target_quorum_threshold_boundaries_cover_every_residue_class() -> Result<(), Box<dyn Error>> {
    for target_count in 1_u8..=9 {
        let target_keys = keys(1..=target_count)?;
        let trusted = target_keys
            .iter()
            .map(|(address, _)| *address)
            .collect::<Vec<_>>();
        let required = besu_qbft::target_quorum(usize::from(target_count))?;
        let below = witness_with_counts(target_count, required - 1, trusted.clone())?;
        assert!(matches!(
            verify_update(&below),
            Err(QbftError::InsufficientTargetQuorum { required: r, actual })
                if r == required && actual == required - 1
        ));
        verify_update(&witness_with_counts(
            target_count,
            required,
            trusted.clone(),
        )?)?;
        if required < usize::from(target_count) {
            verify_update(&witness_with_counts(target_count, required + 1, trusted)?)?;
        }
    }
    Ok(())
}

#[test]
fn trusted_overlap_threshold_boundaries_cover_every_residue_class() -> Result<(), Box<dyn Error>> {
    let target_keys = keys(1..=12)?;
    let target_required = besu_qbft::target_quorum(target_keys.len())?;
    for trusted_count in 1_usize..=9 {
        let required = besu_qbft::trusted_overlap_threshold(trusted_count)?;
        let below = trusted_set_with_overlap(&target_keys, trusted_count, required - 1)?;
        let witness = witness_with_counts(12, target_required, below)?;
        assert!(matches!(
            verify_update(&witness),
            Err(QbftError::InsufficientTrustedOverlap { required: r, actual })
                if r == required && actual == required - 1
        ));

        let exact = trusted_set_with_overlap(&target_keys, trusted_count, required)?;
        verify_update(&witness_with_counts(12, target_required, exact)?)?;
        if required < trusted_count && required < target_required {
            let above = trusted_set_with_overlap(&target_keys, trusted_count, required + 1)?;
            verify_update(&witness_with_counts(12, target_required, above)?)?;
        }
    }
    Ok(())
}
