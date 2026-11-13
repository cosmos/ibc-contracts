use std::{collections::BTreeMap, error::Error, path::PathBuf};

use serde::Serialize;
use sha2::{Digest, Sha256};
use sp1_sdk::{Elf, HashableKey, Prover, ProverClient, ProvingKey};

/// Matches the identity format consumed by the prover and deployment tools.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ProgramIdentity {
    elf_sha256: String,
    program_v_key: String,
}

#[tokio::main]
async fn main() {
    if let Err(error) = run().await {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

async fn run() -> Result<(), Box<dyn Error>> {
    let artifacts = std::env::args_os()
        .nth(1)
        .map_or_else(|| PathBuf::from("artifacts"), PathBuf::from);
    // Key derivation is local and must not depend on proving-service credentials
    // or the caller's SP1_PROVER setting.
    let client = ProverClient::builder().cpu().build().await;
    let mut identities = BTreeMap::new();
    for (name, filename) in [
        ("update-client", "update-client.elf"),
        ("membership", "membership.elf"),
        ("non-membership", "non-membership.elf"),
    ] {
        let elf = std::fs::read(artifacts.join(filename))?;
        let proving_key = client.setup(Elf::from(elf.as_slice())).await?;
        identities.insert(
            name,
            ProgramIdentity {
                elf_sha256: format!("0x{}", hex::encode(Sha256::digest(&elf))),
                program_v_key: proving_key.verifying_key().bytes32(),
            },
        );
    }
    println!("{}", serde_json::to_string_pretty(&identities)?);
    Ok(())
}
