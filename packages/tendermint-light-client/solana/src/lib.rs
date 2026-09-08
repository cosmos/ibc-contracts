// SPDX-License-Identifier: Apache-2.0

//! Solana-optimized Tendermint light client verifier using precompiled Ed25519Program and brine-ed25519

use ibc_core_commitment_types::proto::ics23::HostFunctionsManager;
use solana_account_info::AccountInfo;
use solana_pubkey::Pubkey;
use solana_sha256_hasher::{hash as sha256, hashv as sha256v};
use tendermint::crypto::signature::Error;
use tendermint::{PublicKey, Signature};
use tendermint_light_client_verifier::{
    errors::VerificationError,
    operations::{commit_validator::ProdCommitValidator, ProvidedVotingPowerCalculator},
    predicates::VerificationPredicates,
    types::ValidatorSet,
    PredicateVerifier,
};

use tendermint::merkle::Hash;

/// Solana-optimized predicates that skip redundant Merkle hashing
///
/// **Performance (100 validators):** Saves ~290k compute units total (~145k per validator set hash)
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SolanaPredicates;

impl VerificationPredicates for SolanaPredicates {
    type Sha256 = SolanaSha256;

    /// No-op: validator set hash already verified by [`Header::validate_basic()`][src].
    ///
    /// Skipping the redundant Merkle hash saves ~145k CU (100 validators).
    ///
    /// [src]: https://github.com/informalsystems/ibc-rs/blob/v0.57.0/ibc-clients/ics07-tendermint/types/src/header.rs#L151-L157
    fn validator_sets_match(
        &self,
        _validators: &ValidatorSet,
        _header_validators_hash: tendermint::Hash,
    ) -> Result<(), VerificationError> {
        Ok(())
    }

    /// No-op: trusted next-validators hash already verified by
    /// [`check_trusted_next_validator_set()`][trusted], and the untrusted state
    /// [passes `next_validators: None`][untrusted] so this predicate is skipped.
    ///
    /// Saves ~145k CU (100 validators).
    ///
    /// [trusted]: https://github.com/informalsystems/ibc-rs/blob/be82d123448a59ccea305c9e918f07aca5ec1a6f/ibc-clients/ics07-tendermint/src/client_state/update_client.rs#L52-L54
    /// [untrusted]: https://github.com/informalsystems/ibc-rs/blob/be82d123448a59ccea305c9e918f07aca5ec1a6f/ibc-clients/ics07-tendermint/src/client_state/update_client.rs#L78-L85
    fn next_validators_match(
        &self,
        _next_validators: &ValidatorSet,
        _header_next_validators_hash: tendermint::Hash,
    ) -> Result<(), VerificationError> {
        Ok(())
    }
}

/// Solana-optimized verifier with pre-verified Ed25519 signatures and optimized Merkle hashing
///
/// **Signature verification:** Pre-verification PDAs (~10k CU/sig via Ed25519Program) with brine-ed25519 fallback (~30k CU/sig)
/// **Merkle hashing:** Skips redundant validator set hash validation (saves ~290k CU for 100 validators)
/// **Real-world costs:**
/// - Noble (20 validators): ~548k CU total (~$0.025-0.033 USD)
/// - Celestia (100 validators): ~2.16M CU total (~$0.11-0.14 USD)
pub type SolanaVerifier<'a> =
    PredicateVerifier<SolanaPredicates, SolanaVotingPowerCalculator<'a>, ProdCommitValidator>;

/// Solana voting power calculator using optimized signature verification
pub type SolanaVotingPowerCalculator<'a> =
    ProvidedVotingPowerCalculator<SolanaSignatureVerifier<'a>>;

/// Solana optimised signature verifier with pre-verification account support
#[derive(Clone, Debug)]
pub struct SolanaSignatureVerifier<'a> {
    /// Pre-verified signature accounts from `remaining_accounts`
    verification_accounts: &'a [AccountInfo<'a>],
    /// Program ID for PDA derivation
    program_id: &'a Pubkey,
}

// SAFETY: Solana programs are single-threaded; `AccountInfo` contains `Rc` which
// prevents auto-impl of `Send`/`Sync`, but there is no concurrent access.
unsafe impl Send for SolanaSignatureVerifier<'_> {}
unsafe impl Sync for SolanaSignatureVerifier<'_> {}

impl<'a> SolanaSignatureVerifier<'a> {
    pub fn new(verification_accounts: &'a [AccountInfo<'a>], program_id: &'a Pubkey) -> Self {
        Self {
            verification_accounts,
            program_id,
        }
    }
}

impl<'a> tendermint::crypto::signature::Verifier for SolanaSignatureVerifier<'a> {
    fn verify(&self, pubkey: PublicKey, msg: &[u8], signature: &Signature) -> Result<(), Error> {
        use solana_ibc_types::ics07::SIGNATURE_VERIFICATION_IS_VALID_OFFSET;

        let PublicKey::Ed25519(pk) = pubkey else {
            return Err(Error::UnsupportedKeyType);
        };

        let sig_hash = sha256v(&[pk.as_bytes(), msg, signature.as_bytes()]).to_bytes();
        let (expected_pda, _) =
            Pubkey::find_program_address(&[b"sig_verify", &sig_hash], self.program_id);

        // Fallback: brine-ed25519 (~30k CU/sig, audited by OtterSec)
        let fallback = || {
            brine_ed25519::sig_verify(pk.as_bytes(), signature.as_bytes(), msg)
                .map_err(|_| Error::VerificationFailed)
        };

        let Some(account) = self
            .verification_accounts
            .iter()
            .find(|a| a.key == &expected_pda)
        else {
            return fallback();
        };

        let Ok(data) = account.try_borrow_data() else {
            return fallback();
        };

        match data.get(SIGNATURE_VERIFICATION_IS_VALID_OFFSET) {
            Some(&1) => Ok(()),
            Some(&0) => Err(Error::VerificationFailed),
            Some(v) => {
                solana_msg::msg!("Unexpected verification value: {}", v);
                fallback()
            }
            None => {
                solana_msg::msg!("Verification account data too short");
                fallback()
            }
        }
    }
}

/// Merkle
#[derive(Default)]
pub struct SolanaSha256(tendermint::merkle::NonIncremental<SolanaSha256Impl>);

/// Solana Sha256
#[derive(Default)]
pub struct SolanaSha256Impl;

impl tendermint::crypto::Sha256 for SolanaSha256Impl {
    fn digest(data: impl AsRef<[u8]>) -> [u8; 32] {
        sha256v(&[data.as_ref()]).to_bytes()
    }
}

impl tendermint::crypto::Sha256 for SolanaSha256 {
    fn digest(data: impl AsRef<[u8]>) -> [u8; 32] {
        SolanaSha256Impl::digest(data)
    }
}
impl tendermint::merkle::MerkleHash for SolanaSha256 {
    fn empty_hash(&mut self) -> Hash {
        self.0.empty_hash()
    }

    fn leaf_hash(&mut self, bytes: &[u8]) -> Hash {
        self.0.leaf_hash(bytes)
    }

    fn inner_hash(&mut self, left: Hash, right: Hash) -> Hash {
        self.0.inner_hash(left, right)
    }
}

/// Solana-optimized host functions for ICS-23 Merkle proof verification
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SolanaHostFunctionsManager;

impl ics23::HostFunctionsProvider for SolanaHostFunctionsManager {
    fn sha2_256(message: &[u8]) -> [u8; 32] {
        sha256(message).to_bytes()
    }

    fn sha2_512(message: &[u8]) -> [u8; 64] {
        HostFunctionsManager::sha2_512(message)
    }

    fn sha2_512_truncated(message: &[u8]) -> [u8; 32] {
        HostFunctionsManager::sha2_512_truncated(message)
    }

    fn keccak_256(message: &[u8]) -> [u8; 32] {
        solana_keccak_hasher::hash(message).to_bytes()
    }

    fn ripemd160(message: &[u8]) -> [u8; 20] {
        HostFunctionsManager::ripemd160(message)
    }

    fn blake2b_512(message: &[u8]) -> [u8; 64] {
        HostFunctionsManager::blake2b_512(message)
    }

    fn blake2s_256(message: &[u8]) -> [u8; 32] {
        HostFunctionsManager::blake2s_256(message)
    }

    fn blake3(message: &[u8]) -> [u8; 32] {
        HostFunctionsManager::blake3(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::rstest;
    use solana_ibc_types::ics07::SIGNATURE_VERIFICATION_IS_VALID_OFFSET;
    use std::cell::RefCell;
    use std::rc::Rc;
    use tendermint::crypto::signature::Verifier;

    fn create_verification_account_data(is_valid: u8) -> Vec<u8> {
        let mut data = vec![0u8; SIGNATURE_VERIFICATION_IS_VALID_OFFSET + 1];
        data[SIGNATURE_VERIFICATION_IS_VALID_OFFSET] = is_valid;
        data
    }

    fn create_account_info<'a>(
        key: &'a Pubkey,
        data: &'a mut [u8],
        lamports: &'a mut u64,
        owner: &'a Pubkey,
    ) -> AccountInfo<'a> {
        AccountInfo {
            key,
            is_signer: false,
            is_writable: false,
            lamports: Rc::new(RefCell::new(lamports)),
            data: Rc::new(RefCell::new(data)),
            owner,
            executable: false,
            rent_epoch: 0,
        }
    }

    fn compute_sig_verify_pda(
        pk: &[u8; 32],
        msg: &[u8],
        sig: &[u8; 64],
        program_id: &Pubkey,
    ) -> Pubkey {
        let sig_hash = sha256v(&[pk, msg, sig]).to_bytes();
        let (pda, _) = Pubkey::find_program_address(&[b"sig_verify", &sig_hash], program_id);
        pda
    }

    #[rstest]
    #[case::valid_signature(1, true)]
    #[case::invalid_signature(0, false)]
    #[case::unexpected_value_falls_back(42, false)]
    fn test_preverify_with_account(#[case] is_valid: u8, #[case] expected_ok: bool) {
        let program_id = Pubkey::new_unique();
        let owner = Pubkey::new_unique();
        let pk_bytes = [1u8; 32];
        let msg = b"test message";
        let sig_bytes = [2u8; 64];
        let pda = compute_sig_verify_pda(&pk_bytes, msg, &sig_bytes, &program_id);

        let mut data = create_verification_account_data(is_valid);
        let mut lamports = 1_000_000u64;
        let accounts = [create_account_info(&pda, &mut data, &mut lamports, &owner)];

        let verifier = SolanaSignatureVerifier::new(&accounts, &program_id);
        let pk = tendermint::PublicKey::from_raw_ed25519(&pk_bytes).unwrap();
        let sig = Signature::try_from(sig_bytes.as_slice()).unwrap();

        assert_eq!(verifier.verify(pk, msg, &sig).is_ok(), expected_ok);
    }

    #[test]
    fn test_preverify_data_too_short_falls_back() {
        let program_id = Pubkey::new_unique();
        let owner = Pubkey::new_unique();
        let pk_bytes = [1u8; 32];
        let msg = b"test message";
        let sig_bytes = [2u8; 64];
        let pda = compute_sig_verify_pda(&pk_bytes, msg, &sig_bytes, &program_id);

        let mut data = vec![0u8; SIGNATURE_VERIFICATION_IS_VALID_OFFSET];
        let mut lamports = 1_000_000u64;
        let accounts = [create_account_info(&pda, &mut data, &mut lamports, &owner)];

        let verifier = SolanaSignatureVerifier::new(&accounts, &program_id);
        let pk = tendermint::PublicKey::from_raw_ed25519(&pk_bytes).unwrap();
        let sig = Signature::try_from(sig_bytes.as_slice()).unwrap();

        assert!(verifier.verify(pk, msg, &sig).is_err());
    }

    #[test]
    fn test_no_preverify_account_falls_back() {
        let program_id = Pubkey::new_unique();
        let pk_bytes = [1u8; 32];
        let msg = b"test message";
        let sig_bytes = [2u8; 64];

        let verifier = SolanaSignatureVerifier::new(&[], &program_id);
        let pk = tendermint::PublicKey::from_raw_ed25519(&pk_bytes).unwrap();
        let sig = Signature::try_from(sig_bytes.as_slice()).unwrap();

        assert!(verifier.verify(pk, msg, &sig).is_err());
    }
}
