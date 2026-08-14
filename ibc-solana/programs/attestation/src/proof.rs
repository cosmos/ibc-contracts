// SPDX-License-Identifier: Apache-2.0

use anchor_lang::prelude::*;
use borsh::BorshDeserialize;

use crate::error::ErrorCode;
use crate::types::MembershipProof;

const MAX_PROOF_SIZE: usize = 64 * 1024; // 64 KB

/// Deserialize membership proof with size validation.
pub fn deserialize_membership_proof(proof_bytes: &[u8]) -> Result<MembershipProof> {
    if proof_bytes.len() > MAX_PROOF_SIZE {
        msg!(
            "Proof size {} exceeds maximum {}",
            proof_bytes.len(),
            MAX_PROOF_SIZE
        );
        return Err(error!(ErrorCode::InvalidProof));
    }

    MembershipProof::try_from_slice(proof_bytes).map_err(|e| {
        msg!("Failed to deserialize membership proof: {}", e);
        error!(ErrorCode::InvalidProof)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deserialize_membership_proof_valid() {
        let proof = MembershipProof {
            attestation_data: vec![1, 2, 3],
            signatures: vec![vec![4, 5, 6]],
        };
        let borsh_bytes = borsh::to_vec(&proof).unwrap();

        let result = deserialize_membership_proof(&borsh_bytes).unwrap();
        assert_eq!(result.attestation_data, vec![1, 2, 3]);
        assert_eq!(result.signatures, vec![vec![4, 5, 6]]);
    }

    #[test]
    fn test_deserialize_membership_proof_empty_fields() {
        let proof = MembershipProof {
            attestation_data: vec![],
            signatures: vec![],
        };
        let borsh_bytes = borsh::to_vec(&proof).unwrap();

        let result = deserialize_membership_proof(&borsh_bytes).unwrap();
        assert!(result.attestation_data.is_empty());
        assert!(result.signatures.is_empty());
    }

    #[rstest::rstest]
    #[case::invalid_bytes(b"not valid borsh data".to_vec())]
    #[case::truncated({
        let proof = MembershipProof {
            attestation_data: vec![1, 2, 3],
            signatures: vec![vec![4, 5, 6]],
        };
        let mut bytes = borsh::to_vec(&proof).unwrap();
        bytes.truncate(5);
        bytes
    })]
    #[case::empty_bytes(vec![])]
    fn test_deserialize_membership_proof_invalid(#[case] data: Vec<u8>) {
        assert!(deserialize_membership_proof(&data).is_err());
    }

    #[test]
    fn test_deserialize_membership_proof_multiple_signatures() {
        let proof = MembershipProof {
            attestation_data: vec![1, 2, 3, 4, 5],
            signatures: vec![vec![10; 65], vec![20; 65], vec![30; 65]],
        };
        let borsh_bytes = borsh::to_vec(&proof).unwrap();

        let result = deserialize_membership_proof(&borsh_bytes).unwrap();
        assert_eq!(result.attestation_data, vec![1, 2, 3, 4, 5]);
        assert_eq!(result.signatures.len(), 3);
    }

    #[test]
    fn test_deserialize_membership_proof_large_attestation_data() {
        let proof = MembershipProof {
            attestation_data: vec![0xAB; 1024],
            signatures: vec![vec![0xCD; 65]],
        };
        let borsh_bytes = borsh::to_vec(&proof).unwrap();

        let result = deserialize_membership_proof(&borsh_bytes).unwrap();
        assert_eq!(result.attestation_data.len(), 1024);
    }

    #[test]
    fn test_deserialize_membership_proof_exceeds_max_size() {
        let oversized_bytes = vec![0u8; MAX_PROOF_SIZE + 1];
        let result = deserialize_membership_proof(&oversized_bytes);
        assert!(result.is_err());
    }

    #[test]
    fn test_deserialize_membership_proof_at_max_size() {
        let proof = MembershipProof {
            attestation_data: vec![0xAB; MAX_PROOF_SIZE - 100],
            signatures: vec![],
        };
        let borsh_bytes = borsh::to_vec(&proof).unwrap();

        assert!(borsh_bytes.len() <= MAX_PROOF_SIZE);
        let result = deserialize_membership_proof(&borsh_bytes);
        assert!(result.is_ok());
    }
}
