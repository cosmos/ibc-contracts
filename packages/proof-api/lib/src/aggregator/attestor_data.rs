// SPDX-License-Identifier: Apache-2.0

use super::rpc::{AggregatedAttestation, Attestation};
use anyhow::{ensure as anyhow_ensure, Context, Result};
use std::collections::HashMap;

type State = Vec<u8>;

// 65-byte recoverable ECDSA signature: r (32) || s (32) || v (1)
const SIGNATURE_BYTE_LENGTH: usize = 65;

/// Maps `attested_data` -> list of attestations
///
/// Structure:
/// ```text
/// Attested_data: 0x1234... (12 bytes)
///     [Attestation_A, Attestation_B]
/// Attested_data: 0x9876...
///     [Attestation_C, Attestation_D]
/// ```
#[derive(Debug, Clone, Default)]
pub struct AttestatorData {
    state_attestations: HashMap<State, Vec<Attestation>>,
}

impl AttestatorData {
    #[must_use]
    pub(super) fn new() -> Self {
        Self::default()
    }

    pub(super) fn insert(&mut self, attestation: Attestation) -> Result<()> {
        attestation.validate().context("Invalid attestation")?;

        let attested_data = State::from(attestation.attested_data.as_slice());

        self.state_attestations
            .entry(attested_data)
            .or_default()
            .push(attestation);

        Ok(())
    }

    #[must_use]
    pub(super) fn agg_quorumed_attestations(&self, quorum: usize) -> Option<AggregatedAttestation> {
        self.state_attestations
            .iter()
            .find(|(_, attestations)| attestations.len() >= quorum)
            .map(|(_, attestations)| {
                // Safe to unwrap as quorum lookup ensures non-zero vecs
                let (att, h, ts) = attestations
                    .first()
                    .map(|att| (att.attested_data.clone(), att.height, att.timestamp))
                    .unwrap();
                let sigs: Vec<_> = attestations
                    .iter()
                    .map(|att| att.signature.clone())
                    .collect();

                AggregatedAttestation {
                    height: h,
                    timestamp: ts,
                    attested_data: att,
                    signatures: sigs,
                }
            })
    }
}

impl Attestation {
    fn validate(&self) -> Result<()> {
        // Always enforce 65-byte recoverable signature format
        anyhow_ensure!(
            self.signature.len() == SIGNATURE_BYTE_LENGTH,
            "Invalid signature length: {}",
            self.signature.len()
        );
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_valid_65_byte_signature() -> Vec<u8> {
        let mut sig = vec![0x11; 64]; // r and s components
        sig.push(27); // v component (27 or 28 for Ethereum compatibility)
        sig
    }

    #[test]
    fn accepts_65_byte_signatures() {
        let mut attestator_data = AttestatorData::new();

        let result = attestator_data.insert(Attestation {
            attested_data: vec![1],
            height: 100,
            timestamp: Some(100),
            signature: create_valid_65_byte_signature(),
        });

        assert!(result.is_ok(), "Should accept 65-byte signatures");
    }

    #[test]
    fn rejects_64_byte_signatures() {
        let mut attestator_data = AttestatorData::new();

        let result = attestator_data.insert(Attestation {
            attested_data: vec![1],
            height: 100,
            timestamp: Some(100),
            signature: vec![0x04; 64], // Old 64-byte format
        });

        assert!(result.is_err(), "Should reject 64-byte signatures");
        // The error is wrapped by context, so we just verify it's an error
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Invalid attestation"),
            "Should be an invalid attestation error, got: {err_msg}",
        );
    }

    #[test]
    fn ignores_states_below_quorum() {
        // We have a height 100 but only 1 signature < quorum 2
        let mut attestator_data = AttestatorData::new();

        attestator_data
            .insert(Attestation {
                attested_data: vec![1],
                height: 100,
                timestamp: Some(100),
                signature: create_valid_65_byte_signature(),
            })
            .unwrap();

        let latest = attestator_data.agg_quorumed_attestations(2);
        assert!(latest.is_none(), "Should not return a state below quorum");
    }

    #[test]
    fn state_meeting_quorum() {
        let mut attestator_data = AttestatorData::new();
        let attestation = vec![0xAA];
        let height = 123;

        attestator_data
            .insert(Attestation {
                attested_data: attestation.clone(),
                height,
                timestamp: Some(height),
                signature: create_valid_65_byte_signature(),
            })
            .unwrap();

        attestator_data
            .insert(Attestation {
                attested_data: attestation.clone(),
                height,
                timestamp: Some(height),
                signature: create_valid_65_byte_signature(),
            })
            .unwrap();

        let latest = attestator_data.agg_quorumed_attestations(2);
        assert!(latest.is_some(), "Should return a state meeting quorum");

        let latest = latest.unwrap();
        assert_eq!(latest.height, height);
        assert_eq!(latest.attested_data, attestation);
        assert_eq!(latest.signatures.len(), 2);
    }
}
