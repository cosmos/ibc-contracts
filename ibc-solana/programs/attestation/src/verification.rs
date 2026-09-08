// SPDX-License-Identifier: Apache-2.0

use anchor_lang::prelude::*;

use crate::crypto::{recover_eth_address, tagged_signing_input, AttestationType};
use crate::error::ErrorCode;
use crate::types::ClientState;
use crate::ETH_ADDRESS_LEN;

/// Verify attestation signatures against the trusted attestor set.
pub fn verify_attestation(
    client_state: &ClientState,
    attestation_data: &[u8],
    raw_signatures: &[Vec<u8>],
    attestation_type: AttestationType,
) -> Result<()> {
    require!(!raw_signatures.is_empty(), ErrorCode::EmptySignatures);
    require!(
        raw_signatures.len() >= client_state.min_required_sigs as usize,
        ErrorCode::ThresholdNotMet
    );

    let message_hash = tagged_signing_input(attestation_data, attestation_type);

    // Recover addresses and check for duplicates + trust in single pass
    let mut recovered_addresses: Vec<[u8; ETH_ADDRESS_LEN]> =
        Vec::with_capacity(raw_signatures.len());

    for raw_sig in raw_signatures {
        let recovered_address = recover_eth_address(&message_hash, raw_sig)?;

        if recovered_addresses.contains(&recovered_address) {
            return Err(error!(ErrorCode::DuplicateSigner));
        }

        if !client_state.attestor_addresses.contains(&recovered_address) {
            return Err(error!(ErrorCode::UnknownSigner));
        }

        recovered_addresses.push(recovered_address);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::AttestationType;
    use crate::test_helpers::signing::TestAttestor;
    use crate::types::AccountVersion;
    use rstest::rstest;

    fn create_test_client_state(
        attestor_addresses: Vec<[u8; ETH_ADDRESS_LEN]>,
        min_required_sigs: u8,
    ) -> ClientState {
        ClientState {
            version: AccountVersion::V1,
            attestor_addresses,
            min_required_sigs,
            latest_height: 100,
            is_frozen: false,
        }
    }

    fn create_test_signature(index: u8) -> Vec<u8> {
        let mut sig = vec![index; 64];
        sig.push(27); // recovery_id
        sig
    }

    #[rstest]
    #[case::no_signatures(vec![[1u8; 20]], 1, vec![])]
    #[case::too_few_signatures(vec![[1u8; 20], [2u8; 20]], 2, vec![create_test_signature(1)])]
    #[case::duplicate_signers(vec![[1u8; 20], [2u8; 20]], 2, vec![create_test_signature(1), create_test_signature(1)])]
    #[case::min_sigs_zero_with_no_sigs(vec![[1u8; 20]], 0, vec![])]
    #[case::different_signatures(vec![[1u8; 20], [2u8; 20]], 2, vec![create_test_signature(1), create_test_signature(2)])]
    #[case::exact_required_sigs(vec![[1u8; 20], [2u8; 20], [3u8; 20]], 2, vec![create_test_signature(1), create_test_signature(2)])]
    #[case::more_than_required_sigs(vec![[1u8; 20], [2u8; 20], [3u8; 20]], 2, vec![create_test_signature(1), create_test_signature(2), create_test_signature(3)])]
    fn test_verify_attestation_stub_signature_errors(
        #[case] addrs: Vec<[u8; ETH_ADDRESS_LEN]>,
        #[case] min_sigs: u8,
        #[case] signatures: Vec<Vec<u8>>,
    ) {
        let client_state = create_test_client_state(addrs, min_sigs);
        let result = verify_attestation(
            &client_state,
            b"test data",
            &signatures,
            AttestationType::State,
        );
        assert!(result.is_err());
    }

    // Tests below use TestAttestor which creates real ECDSA signatures.
    // Signature recovery uses k256 crate in native builds (see crypto.rs).

    #[rstest]
    #[case::duplicate_signer_same_key(1, &[1], 2, &[1, 1])]
    #[case::unknown_signer(2, &[1], 1, &[2])]
    #[case::mixed_trusted_and_unknown(2, &[1], 2, &[1, 2])]
    fn test_verify_attestation_real_signature_errors(
        #[case] num_attestors: u8,
        #[case] trusted_seeds: &[u8],
        #[case] min_sigs: u8,
        #[case] signer_seeds: &[u8],
    ) {
        let attestors: Vec<_> = (1..=num_attestors).map(TestAttestor::new).collect();
        let trusted_addrs: Vec<_> = trusted_seeds
            .iter()
            .map(|&s| attestors[(s - 1) as usize].eth_address)
            .collect();
        let client_state = create_test_client_state(trusted_addrs, min_sigs);
        let attestation_data = b"test data";

        let signatures: Vec<_> = signer_seeds
            .iter()
            .map(|&s| attestors[(s - 1) as usize].sign(attestation_data, AttestationType::State))
            .collect();

        let result = verify_attestation(
            &client_state,
            attestation_data,
            &signatures,
            AttestationType::State,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_attestation_happy_path() {
        let attestor1 = TestAttestor::new(1);
        let attestor2 = TestAttestor::new(2);
        let client_state =
            create_test_client_state(vec![attestor1.eth_address, attestor2.eth_address], 2);
        let attestation_data = b"test data";

        let sig1 = attestor1.sign(attestation_data, AttestationType::State);
        let sig2 = attestor2.sign(attestation_data, AttestationType::State);

        let result = verify_attestation(
            &client_state,
            attestation_data,
            &[sig1, sig2],
            AttestationType::State,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_attestation_single_signer_happy_path() {
        let attestor = TestAttestor::new(1);
        let client_state = create_test_client_state(vec![attestor.eth_address], 1);
        let attestation_data = b"test data";

        let sig = attestor.sign(attestation_data, AttestationType::State);

        let result = verify_attestation(
            &client_state,
            attestation_data,
            &[sig],
            AttestationType::State,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_attestation_three_of_five_quorum() {
        let attestors: Vec<_> = (1..=5).map(TestAttestor::new).collect();
        let addresses: Vec<_> = attestors.iter().map(|a| a.eth_address).collect();
        let client_state = create_test_client_state(addresses, 3);
        let attestation_data = b"test data";

        // Sign with only 3 of 5 attestors
        let signatures: Vec<_> = attestors[0..3]
            .iter()
            .map(|a| a.sign(attestation_data, AttestationType::State))
            .collect();

        let result = verify_attestation(
            &client_state,
            attestation_data,
            &signatures,
            AttestationType::State,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn test_verify_attestation_rejects_state_sig_as_packet() {
        let attestor = TestAttestor::new(1);
        let client_state = create_test_client_state(vec![attestor.eth_address], 1);
        let attestation_data = b"test data";

        // Sign as State
        let sig = attestor.sign(attestation_data, AttestationType::State);

        // Verify as Packet — must fail (cross-domain replay)
        let result = verify_attestation(
            &client_state,
            attestation_data,
            &[sig],
            AttestationType::Packet,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_verify_attestation_rejects_packet_sig_as_state() {
        let attestor = TestAttestor::new(1);
        let client_state = create_test_client_state(vec![attestor.eth_address], 1);
        let attestation_data = b"test data";

        // Sign as Packet
        let sig = attestor.sign(attestation_data, AttestationType::Packet);

        // Verify as State — must fail
        let result = verify_attestation(
            &client_state,
            attestation_data,
            &[sig],
            AttestationType::State,
        );
        assert!(result.is_err());
    }
}
