//! Transport-independent proving operations shared by the host and Proof API.
use crate::{
    artifact::{MembershipBatchArtifact, ProofArtifact},
    identity::{ApprovedPrograms, ProgramIdentity},
    prepared::{identity_from_directory, validate_elf_identity, validate_vkey_identity},
    prover::{prove_groth16, serialized_witness_metrics},
    witness::{
        build_membership_batch_witness_from_source, build_non_membership_witness_from_source,
        build_update_witness, MembershipIntentInput, StateProofInputs, WitnessRpc,
    },
    ToolError,
};
use besu_public_values::{decode_membership, decode_non_membership, decode_update};
use serde::Serialize;
use std::{path::Path, sync::Arc};

#[derive(Clone)]
struct Program {
    name: &'static str,
    elf: Arc<[u8]>,
    identity: ProgramIdentity,
}

impl Program {
    fn load(name: &'static str, path: &Path) -> Result<Self, ToolError> {
        let elf = std::fs::read(path)?;
        let identity = identity_from_directory(name, path.parent().unwrap_or(Path::new(".")))?;
        validate_elf_identity(name, &elf, &identity)?;
        Ok(Self {
            name,
            elf: Arc::from(elf),
            identity,
        })
    }
}

#[derive(Clone)]
struct Programs {
    update: Program,
    membership: Program,
    non_membership: Program,
}

impl Programs {
    fn load(directory: &Path) -> Result<Self, ToolError> {
        // Resolve the published set once; a rebuild must not mix generations during startup.
        let current = directory.join("current");
        let directory = if current.is_dir() {
            std::fs::canonicalize(current)?
        } else {
            directory.to_path_buf()
        };
        Ok(Self {
            update: Program::load("update-client", &directory.join("update-client.elf"))?,
            membership: Program::load("membership", &directory.join("membership.elf"))?,
            non_membership: Program::load("non-membership", &directory.join("non-membership.elf"))?,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WitnessMetadata {
    pub serialized_bytes: usize,
    pub sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProofResponse {
    pub proof: ProofArtifact,
    pub witness: WitnessMetadata,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MembershipProofResponse {
    pub proof: MembershipBatchArtifact,
    pub witness: WitnessMetadata,
}

/// An immutable snapshot of all three programs. Admission and cancellation belong to the caller.
#[derive(Clone)]
pub struct ProvingService {
    programs: Programs,
}

impl ProvingService {
    pub fn load(directory: &Path) -> Result<Self, ToolError> {
        Ok(Self {
            programs: Programs::load(directory)?,
        })
    }

    /// Load one artifact generation and bind it to operator-approved configuration.
    pub fn load_approved(directory: &Path, approved: &ApprovedPrograms) -> Result<Self, ToolError> {
        approved.validate()?;
        let service = Self::load(directory)?;
        for ((name, expected), program) in approved.entries().into_iter().zip([
            &service.programs.update,
            &service.programs.membership,
            &service.programs.non_membership,
        ]) {
            validate_elf_identity(name, program.elf.as_ref(), expected)?;
            validate_vkey_identity(name, &program.identity.program_v_key, expected)?;
        }
        Ok(service)
    }

    pub async fn update(
        &self,
        source: &impl WitnessRpc,
        destination: &impl WitnessRpc,
        source_chain_id: u64,
        destination_light_client: [u8; 20],
        trusted_height: u64,
        target_height: u64,
    ) -> Result<ProofResponse, ToolError> {
        let witness = build_update_witness(
            source,
            destination,
            source_chain_id,
            destination_light_client,
            trusted_height,
            target_height,
        )
        .await?;
        let program = &self.programs.update;
        let output = prove_groth16(program.elf.as_ref(), &witness).await?;
        decode_update(&output.public_values).map_err(|e| ToolError::Sp1(e.to_string()))?;
        proof_response(program, &witness, &output)
    }

    pub async fn membership(
        &self,
        source: &impl WitnessRpc,
        inputs: StateProofInputs,
        intents: Vec<MembershipIntentInput>,
    ) -> Result<MembershipProofResponse, ToolError> {
        let witness = build_membership_batch_witness_from_source(source, inputs, intents).await?;
        let program = &self.programs.membership;
        let output = prove_groth16(program.elf.as_ref(), &witness).await?;
        decode_membership(&output.public_values).map_err(|e| ToolError::Sp1(e.to_string()))?;
        membership_response(program, &witness, &output)
    }

    pub async fn non_membership(
        &self,
        source: &impl WitnessRpc,
        inputs: StateProofInputs,
    ) -> Result<ProofResponse, ToolError> {
        let witness = build_non_membership_witness_from_source(source, inputs).await?;
        let program = &self.programs.non_membership;
        let output = prove_groth16(program.elf.as_ref(), &witness).await?;
        decode_non_membership(&output.public_values).map_err(|e| ToolError::Sp1(e.to_string()))?;
        proof_response(program, &witness, &output)
    }
}

fn membership_response(
    program: &Program,
    witness: &besu_zk_types::MembershipWitness,
    output: &crate::prover::ProofOutput,
) -> Result<MembershipProofResponse, ToolError> {
    validate_vkey_identity(program.name, &output.program_v_key, &program.identity)?;
    let metrics = serialized_witness_metrics(witness);
    Ok(MembershipProofResponse {
        proof: MembershipBatchArtifact::new(
            program.elf.as_ref(),
            &output.program_v_key,
            witness,
            &output.public_values,
            &output.proof_bytes,
        )?,
        witness: WitnessMetadata {
            serialized_bytes: metrics.byte_len,
            sha256: format!("0x{}", hex::encode(metrics.sha256)),
        },
    })
}

fn proof_response<W: Serialize>(
    program: &Program,
    witness: &W,
    output: &crate::prover::ProofOutput,
) -> Result<ProofResponse, ToolError> {
    validate_vkey_identity(program.name, &output.program_v_key, &program.identity)?;
    let metrics = serialized_witness_metrics(witness);
    Ok(ProofResponse {
        proof: ProofArtifact::new(
            program.name,
            program.elf.as_ref(),
            &output.program_v_key,
            &output.public_values,
            &output.proof_bytes,
        )?,
        witness: WitnessMetadata {
            serialized_bytes: metrics.byte_len,
            sha256: format!("0x{}", hex::encode(metrics.sha256)),
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    #[test]
    fn membership_response_contains_verified_branches_and_one_full_envelope(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let witness = crate::golden::membership_batch_witness(3)?;
        let values = ibc_storage::verify_membership(&witness)?;
        let output = crate::prover::ProofOutput {
            public_values: besu_public_values::encode_membership(&values),
            proof_bytes: vec![1, 2, 3],
            program_v_key: format!("0x{}", "11".repeat(32)),
        };
        let program = Program {
            name: "membership",
            elf: Arc::from([1u8, 2, 3].as_slice()),
            identity: ProgramIdentity {
                elf_sha256: format!("0x{}", hex::encode(besu_zk_types::sha256(&[1, 2, 3]))),
                program_v_key: output.program_v_key.clone(),
            },
        };
        let response = membership_response(&program, &witness, &output)
            .map_err(|error| format!("{error:?}"))?;
        let json = serde_json::to_value(&response)?;
        assert_eq!(json["proof"]["schemaVersion"], 2);
        assert_eq!(json["proof"]["entryCount"], 3);
        assert!(json["proof"].get("proofEnvelope").is_none());
        for (index, entry) in response.proof.entries.iter().enumerate() {
            let envelope = besu_public_values::decode_membership_envelope(&hex::decode(
                entry.proof_envelope.trim_start_matches("0x"),
            )?)?;
            let intent = &witness.entries[index];
            let leaf = batch_merkle::membership_leaf(
                ibc_storage::path_digest(&intent.path),
                intent.expected_commitment,
            );
            batch_merkle::verify(values.batch_root, leaf, index, 3, &envelope.merkle_proof)?;
            if index == 0 {
                assert_eq!(envelope.mode, besu_zk_types::MEMBERSHIP_ENVELOPE_FULL);
                assert_eq!(envelope.public_values, output.public_values);
                assert_eq!(envelope.proof_bytes, output.proof_bytes);
            } else {
                assert_eq!(envelope.mode, besu_zk_types::MEMBERSHIP_ENVELOPE_CACHED);
                assert!(envelope.public_values.is_empty());
                assert!(envelope.proof_bytes.is_empty());
            }
        }
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn loaded_programs_keep_their_generated_identity_snapshot(
    ) -> Result<(), Box<dyn std::error::Error>> {
        use std::{collections::BTreeMap, os::unix::fs::symlink, time::SystemTime};

        struct Fixture(PathBuf);
        impl Drop for Fixture {
            fn drop(&mut self) {
                let _ = std::fs::remove_dir_all(&self.0);
            }
        }
        let directory = Fixture(std::env::temp_dir().join(format!(
            "besu-programs-{}-{}",
            std::process::id(),
            SystemTime::now().duration_since(SystemTime::UNIX_EPOCH)?.as_nanos()
        )));
        std::fs::create_dir_all(&directory.0)?;
        for (generation, key) in [("one", "1"), ("two", "2")] {
            let path = directory.0.join(generation);
            std::fs::create_dir(&path)?;
            let mut identities = BTreeMap::new();
            for program in ["update-client", "membership", "non-membership"] {
                std::fs::write(path.join(format!("{program}.elf")), generation)?;
                identities.insert(
                    program,
                    ProgramIdentity {
                        elf_sha256: format!(
                            "0x{}",
                            hex::encode(besu_zk_types::sha256(generation.as_bytes()))
                        ),
                        program_v_key: format!("0x{}", key.repeat(64)),
                    },
                );
            }
            std::fs::write(path.join("vkeys.json"), serde_json::to_vec(&identities)?)?;
        }
        symlink("one", directory.0.join("current"))?;
        let first = Programs::load(&directory.0)?;
        symlink("two", directory.0.join("next"))?;
        std::fs::rename(directory.0.join("next"), directory.0.join("current"))?;
        let second = Programs::load(&directory.0)?;
        assert_eq!(first.membership.elf.as_ref(), b"one");
        assert_eq!(second.membership.elf.as_ref(), b"two");
        assert!(validate_vkey_identity(
            "membership",
            &first.membership.identity.program_v_key,
            &first.membership.identity
        )
        .is_ok());
        assert!(validate_vkey_identity(
            "membership",
            &first.membership.identity.program_v_key,
            &second.membership.identity
        )
        .is_err());
        std::fs::write(directory.0.join("two/membership.elf"), "incorrect ELF")?;
        assert!(Programs::load(&directory.0).is_err());
        std::fs::remove_file(directory.0.join("two/vkeys.json"))?;
        assert!(Programs::load(&directory.0).is_err());
        Ok(())
    }
}
