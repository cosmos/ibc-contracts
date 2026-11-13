use std::env::{self, VarError};

use serde::Serialize;
use sp1_sdk::{
    network::signer::NetworkSigner, Elf, HashableKey, ProveRequest, Prover, ProverClient,
    ProvingKey, SP1Stdin,
};

use crate::ToolError;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExecutionOutput {
    pub public_values: Vec<u8>,
    pub instruction_count: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProofOutput {
    pub public_values: Vec<u8>,
    pub proof_bytes: Vec<u8>,
    pub program_v_key: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SerializedWitnessMetrics {
    pub byte_len: usize,
    pub sha256: [u8; 32],
}

pub fn serialized_witness_metrics<W: Serialize>(witness: &W) -> SerializedWitnessMetrics {
    let mut stdin = SP1Stdin::new();
    stdin.write(witness);
    let bytes = stdin.buffer.concat();
    SerializedWitnessMetrics {
        byte_len: bytes.len(),
        sha256: besu_zk_types::sha256(&bytes),
    }
}

fn optional_unicode_environment(name: &'static str) -> Result<Option<String>, ToolError> {
    match env::var(name) {
        Ok(value) => Ok(Some(value)),
        Err(VarError::NotPresent) => Ok(None),
        Err(VarError::NotUnicode(_)) => Err(ToolError::NonUnicodeEnvironment(name)),
    }
}

fn validate_prover_configuration(
    prover: Option<&str>,
    network_private_key: Option<&str>,
    network_rpc_url: Option<&str>,
) -> Result<(), ToolError> {
    match prover.unwrap_or("cpu") {
        "cpu" | "light" | "mock" => Ok(()),
        "network" => {
            let private_key = network_private_key
                .filter(|value| !value.is_empty())
                .ok_or(ToolError::MissingNetworkPrivateKey)?;
            NetworkSigner::local(private_key).map_err(|_| ToolError::InvalidNetworkPrivateKey)?;

            if let Some(rpc_url) = network_rpc_url {
                let parsed =
                    reqwest::Url::parse(rpc_url).map_err(|_| ToolError::InvalidNetworkRpcUrl)?;
                if !matches!(parsed.scheme(), "http" | "https") || parsed.host_str().is_none() {
                    return Err(ToolError::InvalidNetworkRpcUrl);
                }
            }
            Ok(())
        }
        unsupported => Err(ToolError::UnsupportedSp1Prover(unsupported.to_owned())),
    }
}

pub fn validate_prover_environment() -> Result<(), ToolError> {
    let prover = optional_unicode_environment("SP1_PROVER")?;
    if prover.as_deref() != Some("network") {
        return validate_prover_configuration(prover.as_deref(), None, None);
    }

    let network_private_key = optional_unicode_environment("NETWORK_PRIVATE_KEY")?;
    let network_rpc_url = optional_unicode_environment("NETWORK_RPC_URL")?;
    validate_prover_configuration(
        prover.as_deref(),
        network_private_key.as_deref(),
        network_rpc_url.as_deref(),
    )
}

pub async fn execute<W: Serialize>(elf: &[u8], witness: &W) -> Result<ExecutionOutput, ToolError> {
    let mut stdin = SP1Stdin::new();
    stdin.write(witness);
    validate_prover_environment()?;
    let client = ProverClient::from_env().await;
    let (public_values, report) = client
        .execute(Elf::from(elf), stdin)
        .await
        .map_err(|error| ToolError::Sp1(error.to_string()))?;
    if report.exit_code != 0 {
        return Err(ToolError::Sp1(format!(
            "guest exited with status {}",
            report.exit_code
        )));
    }
    Ok(ExecutionOutput {
        public_values: public_values.to_vec(),
        instruction_count: report.total_instruction_count(),
    })
}

pub async fn prove_groth16<W: Serialize>(
    elf: &[u8],
    witness: &W,
) -> Result<ProofOutput, ToolError> {
    let mut stdin = SP1Stdin::new();
    stdin.write(witness);
    validate_prover_environment()?;
    let client = ProverClient::from_env().await;
    let proving_key = client
        .setup(Elf::from(elf))
        .await
        .map_err(|error| ToolError::Sp1(error.to_string()))?;
    let proof = match client.prove(&proving_key, stdin).groth16().await {
        Ok(proof) => proof,
        Err(error) => {
            crate::logging::log(serde_json::json!({
                "component": "sp1-prover", "event": "proving_failed",
                "elf_sha256": format!("0x{}", hex::encode(besu_zk_types::sha256(elf))),
                "error": error.to_string(),
            }));
            if optional_unicode_environment("SP1_PROVER")?.as_deref() == Some("network") {
                match execute(elf, witness).await {
                    Ok(output) => crate::logging::log(serde_json::json!({
                        "component": "sp1-prover", "event": "diagnostic_execution", "result": "passed",
                        "instructions": output.instruction_count, "public_values_bytes": output.public_values.len(),
                    })),
                    Err(diagnostic_error) => crate::logging::log(serde_json::json!({
                        "component": "sp1-prover", "event": "diagnostic_execution", "result": "failed",
                        "error": diagnostic_error.to_string(),
                    })),
                }
            }
            return Err(ToolError::Sp1(error.to_string()));
        }
    };
    client
        .verify(&proof, proving_key.verifying_key(), None)
        .map_err(|error| ToolError::Sp1(error.to_string()))?;
    Ok(ProofOutput {
        public_values: proof.public_values.to_vec(),
        proof_bytes: proof.bytes(),
        program_v_key: proving_key.verifying_key().bytes32(),
    })
}

#[cfg(test)]
mod tests {
    use serde::Serialize;

    use super::{serialized_witness_metrics, validate_prover_configuration};
    use crate::ToolError;

    #[derive(Serialize)]
    struct Fixture {
        height: u64,
        payload: Vec<u8>,
    }

    #[test]
    fn serialized_witness_measurement_is_deterministic_and_payload_sensitive() {
        let first = Fixture {
            height: 7,
            payload: vec![1, 2, 3],
        };
        let same = Fixture {
            height: 7,
            payload: vec![1, 2, 3],
        };
        let larger = Fixture {
            height: 7,
            payload: vec![1, 2, 3, 4],
        };

        assert_eq!(
            serialized_witness_metrics(&first),
            serialized_witness_metrics(&same)
        );
        let first_metrics = serialized_witness_metrics(&first);
        let larger_metrics = serialized_witness_metrics(&larger);
        assert!(larger_metrics.byte_len > first_metrics.byte_len);
        assert_ne!(larger_metrics.sha256, first_metrics.sha256);
    }

    #[test]
    fn local_prover_modes_do_not_require_network_credentials() {
        for prover in [None, Some("cpu"), Some("light"), Some("mock")] {
            assert!(validate_prover_configuration(prover, None, None).is_ok());
        }
    }

    #[test]
    fn unsupported_prover_mode_is_rejected() {
        assert!(matches!(
            validate_prover_configuration(Some("cuda"), None, None),
            Err(ToolError::UnsupportedSp1Prover(prover)) if prover == "cuda"
        ));
    }

    #[test]
    fn network_prover_requires_a_valid_private_key() {
        assert!(matches!(
            validate_prover_configuration(Some("network"), None, None),
            Err(ToolError::MissingNetworkPrivateKey)
        ));
        assert!(matches!(
            validate_prover_configuration(Some("network"), Some(""), None),
            Err(ToolError::MissingNetworkPrivateKey)
        ));
        assert!(matches!(
            validate_prover_configuration(Some("network"), Some("not-a-private-key"), None),
            Err(ToolError::InvalidNetworkPrivateKey)
        ));
    }

    #[test]
    fn network_prover_accepts_default_or_valid_custom_rpc_url() {
        let private_key = "0x0000000000000000000000000000000000000000000000000000000000000001";
        assert!(validate_prover_configuration(Some("network"), Some(private_key), None).is_ok());
        assert!(validate_prover_configuration(
            Some("network"),
            Some(private_key),
            Some("https://sp1-cluster.example.test"),
        )
        .is_ok());
    }

    #[test]
    fn network_prover_rejects_invalid_custom_rpc_url() {
        let private_key = "0x0000000000000000000000000000000000000000000000000000000000000001";
        for rpc_url in ["", "sp1-cluster", "ftp://sp1-cluster.example.test"] {
            assert!(matches!(
                validate_prover_configuration(Some("network"), Some(private_key), Some(rpc_url),),
                Err(ToolError::InvalidNetworkRpcUrl)
            ));
        }
    }
}
