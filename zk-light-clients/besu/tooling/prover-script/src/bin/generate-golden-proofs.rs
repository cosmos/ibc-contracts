use std::{collections::BTreeSet, error::Error, path::PathBuf, time::Instant};

use besu_public_values::{encode_membership, encode_non_membership, encode_update};
use besu_qbft::verify_update;
use besu_zk_types::{MembershipWitness, NonMembershipWitness, UpdateWitness};
use ibc_storage::{verify_membership, verify_non_membership};
use prover_script::{
    artifact::ProofArtifact,
    golden::{
        membership_witness, membership_witness_with_proof_depth, non_membership_witness,
        non_membership_witness_with_proof_depth, update_witness,
        update_witness_with_validator_count,
    },
    prover::{execute, prove_groth16, serialized_witness_metrics, ExecutionOutput, ProofOutput},
};

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn Error>> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    let artifacts = root.join("artifacts");
    let output = std::env::args_os()
        .nth(1)
        .map_or_else(|| artifacts.join("proofs"), PathBuf::from);
    let selected = std::env::args().nth(2).unwrap_or_else(|| "all".to_owned());
    let benchmark_class = std::env::args().nth(3);
    if !["all", "update-client", "membership", "non-membership"].contains(&selected.as_str()) {
        return Err(format!("unknown program selection: {selected}").into());
    }
    if benchmark_class.is_some() && selected == "all" {
        return Err("a benchmark class requires one selected program".into());
    }
    std::fs::create_dir_all(&output)?;
    let mut keys = BTreeSet::new();

    if selected == "all" || selected == "update-client" {
        let update_elf = std::fs::read(artifacts.join("update-client.elf"))?;
        let update = update_fixture(benchmark_class.as_deref())?;
        let update_witness_metrics = serialized_witness_metrics(&update);
        let expected = encode_update(&verify_update(&update)?);
        let update_execution =
            execute_matches_native(&update_elf, &update, &expected, "update-client").await?;
        let started = Instant::now();
        eprintln!("generating update-client Groth16 proof");
        let update_proof = prove_groth16(&update_elf, &update).await?;
        require_public_values(&update_proof, &expected, "update-client")?;
        report(
            "update-client",
            &update_proof,
            update_execution.instruction_count,
            &update_witness_metrics,
            started,
        );
        keys.insert(update_proof.program_v_key.clone());
        write("update-client", &update_elf, &update_proof, &output)?;
    }

    if selected == "all" || selected == "membership" {
        let membership_elf = std::fs::read(artifacts.join("membership.elf"))?;
        let membership = membership_fixture(benchmark_class.as_deref())?;
        let membership_witness_metrics = serialized_witness_metrics(&membership);
        let expected = encode_membership(&verify_membership(&membership)?);
        let membership_execution =
            execute_matches_native(&membership_elf, &membership, &expected, "membership").await?;
        let started = Instant::now();
        eprintln!("generating membership Groth16 proof");
        let membership_proof = prove_groth16(&membership_elf, &membership).await?;
        require_public_values(&membership_proof, &expected, "membership")?;
        report(
            "membership",
            &membership_proof,
            membership_execution.instruction_count,
            &membership_witness_metrics,
            started,
        );
        keys.insert(membership_proof.program_v_key.clone());
        write("membership", &membership_elf, &membership_proof, &output)?;
    }

    if selected == "all" || selected == "non-membership" {
        let non_membership_elf = std::fs::read(artifacts.join("non-membership.elf"))?;
        let non_membership = non_membership_fixture(benchmark_class.as_deref())?;
        let non_membership_witness_metrics = serialized_witness_metrics(&non_membership);
        let expected = encode_non_membership(&verify_non_membership(&non_membership)?);
        let non_membership_execution = execute_matches_native(
            &non_membership_elf,
            &non_membership,
            &expected,
            "non-membership",
        )
        .await?;
        let started = Instant::now();
        eprintln!("generating non-membership Groth16 proof");
        let non_membership_proof = prove_groth16(&non_membership_elf, &non_membership).await?;
        require_public_values(&non_membership_proof, &expected, "non-membership")?;
        report(
            "non-membership",
            &non_membership_proof,
            non_membership_execution.instruction_count,
            &non_membership_witness_metrics,
            started,
        );
        keys.insert(non_membership_proof.program_v_key.clone());
        write(
            "non-membership",
            &non_membership_elf,
            &non_membership_proof,
            &output,
        )?;
    }

    if selected == "all" && keys.len() != 3 {
        return Err("the three canonical ELFs did not produce distinct program VKeys".into());
    }
    Ok(())
}

fn update_fixture(class: Option<&str>) -> Result<UpdateWitness, Box<dyn Error>> {
    match class {
        None => update_witness(),
        Some("small") => update_witness_with_validator_count(1),
        Some("four-validator") => update_witness_with_validator_count(4),
        Some("larger-synthetic") => update_witness_with_validator_count(16),
        Some(other) => Err(format!("unknown benchmark class: {other}").into()),
    }
}

fn membership_fixture(class: Option<&str>) -> Result<MembershipWitness, Box<dyn Error>> {
    match class {
        None | Some("small") | Some("batch-1") => Ok(membership_witness()?),
        Some("batch-5") => Ok(prover_script::golden::membership_batch_witness(5)?),
        Some("batch-10") => Ok(prover_script::golden::membership_batch_witness(10)?),
        Some("batch-20") => Ok(prover_script::golden::membership_batch_witness(20)?),
        Some("batch-32") => Ok(prover_script::golden::membership_batch_witness(32)?),
        Some("larger-synthetic") => Ok(membership_witness_with_proof_depth(12)?),
        Some("four-validator") => {
            Err("four-validator membership witness must be collected from live Besu RPC".into())
        }
        Some(other) => Err(format!("unknown benchmark class: {other}").into()),
    }
}

fn non_membership_fixture(class: Option<&str>) -> Result<NonMembershipWitness, Box<dyn Error>> {
    match class {
        None | Some("small") => Ok(non_membership_witness()),
        Some("larger-synthetic") => Ok(non_membership_witness_with_proof_depth(12)?),
        Some("four-validator") => {
            Err("four-validator non-membership witness must be collected from live Besu RPC".into())
        }
        Some(other) => Err(format!("unknown benchmark class: {other}").into()),
    }
}

fn report(
    program: &str,
    proof: &ProofOutput,
    sp1_cycles: u64,
    witness: &prover_script::prover::SerializedWitnessMetrics,
    started: Instant,
) {
    eprintln!(
        "{program}: sp1Cycles={sp1_cycles}, serializedWitnessBytes={}, witnessSha256=0x{}, \
         proofWallClockSeconds={:.3}, proofBytes={}, publicValuesBytes={}",
        witness.byte_len,
        hex::encode(witness.sha256),
        started.elapsed().as_secs_f64(),
        proof.proof_bytes.len(),
        proof.public_values.len()
    );
}

fn require_bytes(actual: &[u8], expected: &[u8], context: &str) -> Result<(), Box<dyn Error>> {
    if actual != expected {
        return Err(format!("{context} output differs from native output").into());
    }
    Ok(())
}

async fn execute_matches_native<W: serde::Serialize>(
    elf: &[u8],
    witness: &W,
    expected: &[u8],
    program: &str,
) -> Result<ExecutionOutput, Box<dyn Error>> {
    let execution = execute(elf, witness).await?;
    require_bytes(
        &execution.public_values,
        expected,
        &format!("{program} SP1 execution"),
    )?;
    Ok(execution)
}

fn require_public_values(
    proof: &ProofOutput,
    expected: &[u8],
    program: &str,
) -> Result<(), Box<dyn Error>> {
    require_bytes(
        &proof.public_values,
        expected,
        &format!("{program} SP1 proof"),
    )
}

fn write(
    program: &str,
    elf: &[u8],
    proof: &ProofOutput,
    output: &std::path::Path,
) -> Result<(), Box<dyn Error>> {
    ProofArtifact::new(
        program,
        elf,
        &proof.program_v_key,
        &proof.public_values,
        &proof.proof_bytes,
    )?
    .write(&output.join(format!("{program}.json")))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{membership_fixture, non_membership_fixture, update_fixture};

    #[test]
    fn benchmark_class_selection_is_strict_and_uses_distinct_fixtures(
    ) -> Result<(), Box<dyn std::error::Error>> {
        assert_eq!(update_fixture(Some("small"))?.trusted_validators.len(), 1);
        assert_eq!(
            update_fixture(Some("four-validator"))?
                .trusted_validators
                .len(),
            4
        );
        assert_eq!(
            update_fixture(Some("larger-synthetic"))?
                .trusted_validators
                .len(),
            16
        );
        assert_eq!(
            membership_fixture(Some("larger-synthetic"))?
                .entries
                .first()
                .ok_or("empty membership fixture")?
                .storage_proof
                .len(),
            13
        );
        assert_eq!(
            non_membership_fixture(Some("larger-synthetic"))?
                .proof
                .storage_proof
                .len(),
            12
        );
        assert!(membership_fixture(Some("four-validator")).is_err());
        assert!(non_membership_fixture(Some("four-validator")).is_err());
        assert!(update_fixture(Some("unknown")).is_err());
        Ok(())
    }
}
