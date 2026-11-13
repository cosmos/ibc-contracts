use std::path::Path;

use serde::Serialize;

use crate::ToolError;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProofArtifact {
    pub schema_version: u32,
    pub program: String,
    pub elf_sha256: String,
    pub program_v_key: String,
    pub public_values: String,
    pub proof_bytes: String,
    pub proof_envelope: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MembershipArtifactEntry {
    pub leaf_index: u32,
    pub path_digest: String,
    pub expected_commitment: String,
    pub merkle_proof: Vec<String>,
    pub proof_envelope: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MembershipBatchArtifact {
    pub schema_version: u32,
    pub program: String,
    pub proof_height: u64,
    pub batch_root: String,
    pub entry_count: u32,
    pub elf_sha256: String,
    pub program_v_key: String,
    pub public_values: String,
    pub proof_bytes: String,
    pub entries: Vec<MembershipArtifactEntry>,
}

impl MembershipBatchArtifact {
    pub fn new(
        elf: &[u8],
        program_v_key: &str,
        witness: &besu_zk_types::MembershipWitness,
        public_values: &[u8],
        proof_bytes: &[u8],
    ) -> Result<Self, ToolError> {
        if elf.is_empty() || public_values.is_empty() || proof_bytes.is_empty() {
            return Err(ToolError::InvalidArtifact(
                "empty membership proof component",
            ));
        }
        let values = besu_public_values::decode_membership(public_values)
            .map_err(|_| ToolError::InvalidArtifact("invalid membership public values"))?;
        let mut leaves = Vec::with_capacity(witness.entries.len());
        let mut digests = Vec::with_capacity(witness.entries.len());
        for entry in &witness.entries {
            let digest = ibc_storage::path_digest(&entry.path);
            digests.push(digest);
            leaves.push(batch_merkle::membership_leaf(
                digest,
                entry.expected_commitment,
            ));
        }
        let batch_root = batch_merkle::root(&leaves)
            .map_err(|_| ToolError::InvalidArtifact("invalid membership batch"))?;
        if values.batch_root != batch_root
            || usize::try_from(values.entry_count).ok() != Some(leaves.len())
        {
            return Err(ToolError::InvalidArtifact(
                "membership batch/public-values mismatch",
            ));
        }
        let mut entries = Vec::with_capacity(leaves.len());
        for (index, ((entry, digest), _leaf)) in witness
            .entries
            .iter()
            .zip(digests)
            .zip(leaves.iter())
            .enumerate()
        {
            let branch = batch_merkle::branch(&leaves, index)
                .map_err(|_| ToolError::InvalidArtifact("cannot build membership branch"))?;
            let mode = if index == 0 {
                besu_zk_types::MEMBERSHIP_ENVELOPE_FULL
            } else {
                besu_zk_types::MEMBERSHIP_ENVELOPE_CACHED
            };
            let envelope = besu_zk_types::MembershipBatchProofEnvelope {
                mode,
                public_values: if index == 0 {
                    public_values.to_vec()
                } else {
                    Vec::new()
                },
                proof_bytes: if index == 0 {
                    proof_bytes.to_vec()
                } else {
                    Vec::new()
                },
                batch_root,
                entry_count: values.entry_count,
                leaf_index: u32::try_from(index)
                    .map_err(|_| ToolError::InvalidArtifact("membership leaf index overflow"))?,
                merkle_proof: branch.clone(),
            };
            entries.push(MembershipArtifactEntry {
                leaf_index: envelope.leaf_index,
                path_digest: format!("0x{}", hex::encode(digest)),
                expected_commitment: format!("0x{}", hex::encode(entry.expected_commitment)),
                merkle_proof: branch
                    .iter()
                    .map(|hash| format!("0x{}", hex::encode(hash)))
                    .collect(),
                proof_envelope: format!(
                    "0x{}",
                    hex::encode(besu_public_values::encode_membership_envelope(&envelope))
                ),
            });
        }
        Ok(Self {
            schema_version: besu_zk_types::MEMBERSHIP_SCHEMA_VERSION,
            program: "membership".to_owned(),
            proof_height: values.proof_height,
            batch_root: format!("0x{}", hex::encode(batch_root)),
            entry_count: values.entry_count,
            elf_sha256: format!("0x{}", hex::encode(besu_zk_types::sha256(elf))),
            program_v_key: program_v_key.to_owned(),
            public_values: format!("0x{}", hex::encode(public_values)),
            proof_bytes: format!("0x{}", hex::encode(proof_bytes)),
            entries,
        })
    }

    pub fn write(&self, path: &Path) -> Result<(), ToolError> {
        let encoded = serde_json::to_vec_pretty(self).map_err(ToolError::ArtifactSerialization)?;
        crate::json_file::write_atomic(path, &encoded)?;
        Ok(())
    }
}

impl ProofArtifact {
    pub fn new(
        program: &str,
        elf: &[u8],
        program_v_key: &str,
        public_values: &[u8],
        proof_bytes: &[u8],
    ) -> Result<Self, ToolError> {
        if !matches!(program, "update-client" | "membership" | "non-membership") {
            return Err(ToolError::InvalidArtifact("unknown program identity"));
        }
        if elf.is_empty() {
            return Err(ToolError::InvalidArtifact("empty program ELF"));
        }
        if public_values.is_empty() {
            return Err(ToolError::InvalidArtifact("empty public values"));
        }
        if proof_bytes.is_empty() {
            return Err(ToolError::InvalidArtifact("empty proof bytes"));
        }
        let vkey_payload = program_v_key
            .strip_prefix("0x")
            .ok_or(ToolError::InvalidArtifact(
                "program VKey is not 0x-prefixed",
            ))?;
        if vkey_payload.len() != 64 || !vkey_payload.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(ToolError::InvalidArtifact(
                "program VKey is not exactly bytes32",
            ));
        }
        let proof_envelope =
            besu_public_values::encode_proof_envelope(&besu_zk_types::SP1ProofEnvelope {
                public_values: public_values.to_vec(),
                proof_bytes: proof_bytes.to_vec(),
            });
        Ok(Self {
            schema_version: besu_zk_types::SCHEMA_VERSION,
            program: program.to_owned(),
            elf_sha256: format!("0x{}", hex::encode(besu_zk_types::sha256(elf))),
            program_v_key: program_v_key.to_owned(),
            public_values: format!("0x{}", hex::encode(public_values)),
            proof_bytes: format!("0x{}", hex::encode(proof_bytes)),
            proof_envelope: format!("0x{}", hex::encode(proof_envelope)),
        })
    }

    pub fn write(&self, path: &Path) -> Result<(), ToolError> {
        let encoded = serde_json::to_vec_pretty(self).map_err(ToolError::ArtifactSerialization)?;
        crate::json_file::write_atomic(path, &encoded)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use besu_public_values::decode_proof_envelope;

    const VKEY: &str = "0x1111111111111111111111111111111111111111111111111111111111111111";

    #[test]
    fn artifact_is_canonical_complete_and_atomically_written(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let artifact = ProofArtifact::new("membership", &[1, 2], VKEY, &[3, 4], &[5, 6])?;
        assert_eq!(artifact.schema_version, besu_zk_types::SCHEMA_VERSION);
        assert_eq!(artifact.program, "membership");
        assert_eq!(artifact.elf_sha256.len(), 66);
        let envelope = decode_proof_envelope(&hex::decode(
            artifact
                .proof_envelope
                .strip_prefix("0x")
                .ok_or("artifact envelope lost its prefix")?,
        )?)?;
        assert_eq!(envelope.public_values, [3, 4]);
        assert_eq!(envelope.proof_bytes, [5, 6]);

        let path = std::env::temp_dir().join(format!(
            "besu-zk-proof-artifact-test-{}.json",
            std::process::id()
        ));
        artifact.write(&path)?;
        let stored: serde_json::Value = serde_json::from_slice(&std::fs::read(&path)?)?;
        std::fs::remove_file(&path)?;
        assert_eq!(stored["program"], "membership");
        Ok(())
    }

    #[test]
    fn artifact_rejects_unknown_program_empty_fields_and_malformed_vkey() {
        assert!(ProofArtifact::new("fourth", &[1], VKEY, &[2], &[3]).is_err());
        assert!(ProofArtifact::new("membership", &[], VKEY, &[2], &[3]).is_err());
        assert!(ProofArtifact::new("membership", &[1], VKEY, &[], &[3]).is_err());
        assert!(ProofArtifact::new("membership", &[1], VKEY, &[2], &[]).is_err());
        assert!(ProofArtifact::new("membership", &[1], "0x01", &[2], &[3]).is_err());
    }
}
