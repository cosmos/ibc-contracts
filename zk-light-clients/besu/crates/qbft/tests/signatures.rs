use std::{error::Error, io};

use besu_qbft::{recover_address, validate_seal, QbftError};
use besu_zk_types::keccak256;
use k256::ecdsa::{Signature, SigningKey};

const SECP256K1_ORDER: [u8; 32] = [
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe,
    0xba, 0xae, 0xdc, 0xe6, 0xaf, 0x48, 0xa0, 0x3b, 0xbf, 0xd2, 0x5e, 0x8c, 0xd0, 0x36, 0x41, 0x41,
];

const SECP256K1_ORDER_MINUS_ONE: [u8; 32] = [
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe,
    0xba, 0xae, 0xdc, 0xe6, 0xaf, 0x48, 0xa0, 0x3b, 0xbf, 0xd2, 0x5e, 0x8c, 0xd0, 0x36, 0x41, 0x40,
];

#[test]
fn besu_recovery_ids_are_exactly_zero_or_one() {
    let mut seal = [0_u8; 65];
    seal[31] = 1;
    seal[63] = 1;
    seal[64] = 0;
    assert_eq!(validate_seal(&seal), Ok(()));
    seal[64] = 1;
    assert_eq!(validate_seal(&seal), Ok(()));
    seal[64] = 27;
    assert_eq!(validate_seal(&seal), Err(QbftError::InvalidRecoveryId(27)));
    seal[64] = 28;
    assert_eq!(validate_seal(&seal), Err(QbftError::InvalidRecoveryId(28)));
}

#[test]
fn signature_scalars_and_length_are_checked() {
    assert_eq!(
        validate_seal(&[0; 64]),
        Err(QbftError::InvalidSealLength(64))
    );
    let mut seal = [0_u8; 65];
    seal[63] = 1;
    assert_eq!(validate_seal(&seal), Err(QbftError::InvalidSignatureScalar));

    seal[31] = 1;
    seal[32..64].fill(0);
    assert_eq!(validate_seal(&seal), Err(QbftError::InvalidSignatureScalar));

    seal[..32].copy_from_slice(&SECP256K1_ORDER);
    seal[63] = 1;
    assert_eq!(validate_seal(&seal), Err(QbftError::InvalidSignatureScalar));
    seal[..32].fill(0);
    seal[31] = 1;
    seal[32..64].copy_from_slice(&SECP256K1_ORDER);
    assert_eq!(validate_seal(&seal), Err(QbftError::InvalidSignatureScalar));

    seal[..32].copy_from_slice(&SECP256K1_ORDER_MINUS_ONE);
    seal[32..64].copy_from_slice(&SECP256K1_ORDER_MINUS_ONE);
    assert_eq!(validate_seal(&seal), Ok(()));
}

#[test]
fn recovery_verifies_signature_digest_and_ethereum_address_binding() -> Result<(), Box<dyn Error>> {
    let key = SigningKey::from_slice(&[0x42; 32])
        .map_err(|_| io::Error::other("invalid deterministic signing key"))?;
    let encoded = key.verifying_key().to_encoded_point(false);
    let coordinates = encoded
        .as_bytes()
        .get(1..)
        .ok_or_else(|| io::Error::other("missing public-key coordinates"))?;
    let expected: [u8; 20] = keccak256(coordinates)
        .get(12..)
        .ok_or_else(|| io::Error::other("invalid address hash"))?
        .try_into()?;
    let digest = keccak256(b"canonical QBFT header digest fixture");
    let (signature, recovery_id) = key
        .sign_prehash_recoverable(&digest)
        .map_err(|_| io::Error::other("test signing failed"))?;
    let mut seal = signature.to_bytes().to_vec();
    seal.push(recovery_id.to_byte());

    assert_eq!(recover_address(digest, &seal), Ok(expected));

    let mut mutated_r = seal.clone();
    mutated_r[0] ^= 1;
    assert_ne!(recover_address(digest, &mutated_r), Ok(expected));
    let mut mutated_s = seal.clone();
    mutated_s[32] ^= 1;
    assert_ne!(recover_address(digest, &mutated_s), Ok(expected));
    let mut mutated_digest = digest;
    mutated_digest[0] ^= 1;
    assert_ne!(recover_address(mutated_digest, &seal), Ok(expected));

    let other_key = SigningKey::from_slice(&[0x43; 32])
        .map_err(|_| io::Error::other("invalid alternate signing key"))?;
    let (other_signature, other_recovery_id) = other_key
        .sign_prehash_recoverable(&digest)
        .map_err(|_| io::Error::other("alternate test signing failed"))?;
    let mut other_seal = other_signature.to_bytes().to_vec();
    other_seal.push(other_recovery_id.to_byte());
    assert_ne!(recover_address(digest, &other_seal), Ok(expected));

    let other_digest = keccak256(b"another header");
    let (other_header_signature, other_header_recovery_id) = key
        .sign_prehash_recoverable(&other_digest)
        .map_err(|_| io::Error::other("other-header test signing failed"))?;
    let mut other_header_seal = other_header_signature.to_bytes().to_vec();
    other_header_seal.push(other_header_recovery_id.to_byte());
    assert_ne!(recover_address(digest, &other_header_seal), Ok(expected));
    Ok(())
}

#[test]
fn high_s_and_normalized_low_s_forms_recover_the_same_address() -> Result<(), Box<dyn Error>> {
    let key = SigningKey::from_slice(&[0x44; 32])
        .map_err(|_| io::Error::other("invalid deterministic signing key"))?;
    let encoded = key.verifying_key().to_encoded_point(false);
    let coordinates = encoded
        .as_bytes()
        .get(1..)
        .ok_or_else(|| io::Error::other("missing public-key coordinates"))?;
    let expected: [u8; 20] = keccak256(coordinates)
        .get(12..)
        .ok_or_else(|| io::Error::other("invalid address hash"))?
        .try_into()?;

    let digest = keccak256(b"QBFT high-S normalization regression fixture");
    let (low_signature, low_recovery_id) = key
        .sign_prehash_recoverable(&digest)
        .map_err(|_| io::Error::other("test signing failed"))?;
    let mut low_seal = low_signature.to_bytes().to_vec();
    low_seal.push(low_recovery_id.to_byte());

    // (r, n - s) is the equivalent high-S form. Its recovery parity must be
    // flipped so that it identifies the same signer as the canonical low-S form.
    let high_s = -*low_signature.s();
    let high_signature =
        Signature::from_scalars(low_signature.r().to_bytes(), high_s.to_bytes())
            .map_err(|_| io::Error::other("high-S test signature construction failed"))?;
    let mut high_seal = high_signature.to_bytes().to_vec();
    high_seal.push(low_recovery_id.to_byte() ^ 1);

    assert_eq!(recover_address(digest, &low_seal), Ok(expected));
    assert_eq!(recover_address(digest, &high_seal), Ok(expected));
    assert_eq!(
        recover_address(digest, &low_seal),
        recover_address(digest, &high_seal)
    );
    Ok(())
}

#[test]
fn recovery_matches_signing_keys_across_deterministic_vectors() -> Result<(), Box<dyn Error>> {
    for key_byte in 1_u8..=32 {
        let key = SigningKey::from_slice(&[key_byte; 32])
            .map_err(|_| io::Error::other("invalid deterministic signing key"))?;
        let encoded = key.verifying_key().to_encoded_point(false);
        let coordinates = encoded
            .as_bytes()
            .get(1..)
            .ok_or_else(|| io::Error::other("missing public-key coordinates"))?;
        let expected: [u8; 20] = keccak256(coordinates)
            .get(12..)
            .ok_or_else(|| io::Error::other("invalid address hash"))?
            .try_into()?;

        let mut preimage = [0_u8; 64];
        preimage[..32].fill(key_byte);
        preimage[32..].fill(key_byte.wrapping_mul(17));
        let digest = keccak256(&preimage);
        let (signature, recovery_id) = key
            .sign_prehash_recoverable(&digest)
            .map_err(|_| io::Error::other("test signing failed"))?;
        let mut seal = signature.to_bytes().to_vec();
        seal.push(recovery_id.to_byte());

        assert_eq!(recover_address(digest, &seal), Ok(expected));

        let recovery_byte = seal
            .last_mut()
            .ok_or_else(|| io::Error::other("missing recovery byte"))?;
        *recovery_byte ^= 1;
        assert_ne!(recover_address(digest, &seal), Ok(expected));
    }
    Ok(())
}
