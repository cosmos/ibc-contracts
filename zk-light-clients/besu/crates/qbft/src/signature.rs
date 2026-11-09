use besu_zk_types::{is_zero_address, keccak256, Address, QBFT_COMMIT_SEAL_BYTES};
use k256::ecdsa::{RecoveryId, Signature, VerifyingKey};

use crate::QbftError;

/// A structurally valid seal. Only this module can construct or change its fields.
pub(crate) struct ValidatedSeal<'a> {
    bytes: &'a [u8],
    signature: Signature,
    recovery_byte: u8,
}

impl<'a> ValidatedSeal<'a> {
    pub(crate) fn parse(seal: &'a [u8]) -> Result<Self, QbftError> {
        if seal.len() != QBFT_COMMIT_SEAL_BYTES {
            return Err(QbftError::InvalidSealLength(seal.len()));
        }
        // Signature decoding checks that r and s are non-zero and below the
        // curve order. High-S scalars remain valid at this stage.
        let signature =
            Signature::from_slice(&seal[..64]).map_err(|_| QbftError::InvalidSignatureScalar)?;
        let recovery_byte = seal[64];
        if recovery_byte > 1 {
            return Err(QbftError::InvalidRecoveryId(recovery_byte));
        }
        Ok(Self {
            bytes: seal,
            signature,
            recovery_byte,
        })
    }

    pub(crate) fn as_bytes(&self) -> &'a [u8] {
        self.bytes
    }

    pub(crate) fn recover_address(&self, digest: [u8; 32]) -> Result<Address, QbftError> {
        let mut signature = self.signature;
        let mut recovery_byte = self.recovery_byte;
        // k256 deliberately rejects high-S signatures during verification. Besu's seal decoder
        // accepts them. Verify the mathematically equivalent low-S signature with the opposite
        // y-parity recovery bit; the recovered signer is identical and the input bytes remain bound.
        if let Some(normalized) = signature.normalize_s() {
            signature = normalized;
            recovery_byte ^= 1;
        }
        let recovery_id = RecoveryId::from_byte(recovery_byte)
            .ok_or(QbftError::InvalidRecoveryId(recovery_byte))?;
        let verifying_key = VerifyingKey::recover_from_prehash(&digest, &signature, recovery_id)
            .map_err(|_| QbftError::SignatureRecovery)?;
        let encoded = verifying_key.to_encoded_point(false);
        let public_key = encoded.as_bytes();
        let coordinates = public_key.get(1..).ok_or(QbftError::SignatureRecovery)?;
        if coordinates.len() != 64 {
            return Err(QbftError::SignatureRecovery);
        }
        let hash = keccak256(coordinates);
        let address: Address = hash
            .get(12..)
            .ok_or(QbftError::SignatureRecovery)?
            .try_into()
            .map_err(|_| QbftError::SignatureRecovery)?;
        if is_zero_address(&address) {
            return Err(QbftError::ZeroSigner);
        }
        Ok(address)
    }
}

pub fn validate_seal(seal: &[u8]) -> Result<(), QbftError> {
    ValidatedSeal::parse(seal).map(|_| ())
}

pub fn recover_address(digest: [u8; 32], seal: &[u8]) -> Result<Address, QbftError> {
    ValidatedSeal::parse(seal)?.recover_address(digest)
}
