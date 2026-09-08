// SPDX-License-Identifier: Apache-2.0

//! ECDSA attestor for signing attestation proofs in integration tests.
//!
//! Replicates the `TestAttestor` from `attestation::test_helpers::signing`
//! (which is `#[cfg(test)]`-gated and inaccessible from this crate).
//!
//! Attestors are off-chain signers (Ethereum ECDSA keys) that do not hold
//! a Solana keypair, so they do not implement the [`Actor`](crate::Actor)
//! trait and live outside the `actors/` directory.

use alloy_signer::SignerSync;
use alloy_signer_local::PrivateKeySigner;
use attestation::crypto::AttestationType;
use sha2::{Digest, Sha256};

const ETH_ADDRESS_LEN: usize = 20;
const DOMAIN_SEPARATED_PREIMAGE_LEN: usize = 1 + 32;

/// Off-chain ECDSA signer that produces attestation signatures consumed by
/// the relayer. Has no Solana keypair and does not submit transactions.
pub struct Attestor {
    signer: PrivateKeySigner,
    pub eth_address: [u8; ETH_ADDRESS_LEN],
}

impl Attestor {
    /// Create an attestor with a random ECDSA key.
    pub fn random() -> Self {
        let signer = PrivateKeySigner::random();
        let eth_address = signer.address().0 .0;

        Self {
            signer,
            eth_address,
        }
    }

    /// Sign attestation data with domain separation and return a 65-byte signature.
    ///
    /// Computes `sha256(type_tag || sha256(data))` then signs the result.
    pub fn sign(&self, data: &[u8], attestation_type: AttestationType) -> Vec<u8> {
        let inner_hash: [u8; 32] = Sha256::digest(data).into();
        let mut tagged = Vec::with_capacity(DOMAIN_SEPARATED_PREIMAGE_LEN);
        tagged.push(attestation_type as u8);
        tagged.extend_from_slice(&inner_hash);
        let message_hash: [u8; 32] = Sha256::digest(&tagged).into();

        let sig = self
            .signer
            .sign_hash_sync(&message_hash.into())
            .expect("signing should succeed");

        let mut result = Vec::with_capacity(65);
        result.extend_from_slice(&sig.r().to_be_bytes::<32>());
        result.extend_from_slice(&sig.s().to_be_bytes::<32>());
        result.push(u8::from(sig.v()) + 27);
        result
    }
}

/// A set of attestors with random keys.
///
/// Each call to [`Attestors::new`] produces a unique set, so tests that need
/// independent attestor sets per chain can simply call `new` twice.
pub struct Attestors(Vec<Attestor>);

impl Attestors {
    /// Create `count` attestors with random ECDSA keys.
    pub fn new(count: usize) -> Self {
        Self((0..count).map(|_| Attestor::random()).collect())
    }

    pub fn as_slice(&self) -> &[Attestor] {
        &self.0
    }

    pub fn eth_addresses(&self) -> Vec<[u8; ETH_ADDRESS_LEN]> {
        self.0.iter().map(|a| a.eth_address).collect()
    }
}
