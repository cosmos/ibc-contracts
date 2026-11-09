use std::{collections::BTreeMap, fs};

use proof_api::config::Config;
use prover_script::identity::{ApprovedPrograms, ProgramIdentity};
use serde_json::{json, Value};

fn identity(elf: &[u8], key: u8) -> ProgramIdentity {
    ProgramIdentity {
        elf_sha256: format!("0x{}", hex::encode(besu_zk_types::sha256(elf))),
        program_v_key: format!("0x{}", hex::encode([key; 32])),
    }
}

fn fixture() -> Result<(tempfile::TempDir, Value), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    fs::create_dir(directory.path().join("artifacts"))?;
    let mut programs = BTreeMap::new();
    for (program, key) in [
        ("update-client", 1),
        ("membership", 2),
        ("non-membership", 3),
    ] {
        let elf = program.as_bytes();
        fs::write(
            directory
                .path()
                .join("artifacts")
                .join(format!("{program}.elf")),
            elf,
        )?;
        programs.insert(program, identity(elf, key));
    }
    fs::write(
        directory.path().join("artifacts/vkeys.json"),
        serde_json::to_vec(&programs)?,
    )?;
    let value = json!({"schemaVersion": 1, "prover": {"artifactsDir": "artifacts", "approvedPrograms": programs}});
    Ok((directory, value))
}

#[test]
fn yaml_approval_binds_elf_metadata_and_vkeys() -> Result<(), Box<dyn std::error::Error>> {
    let (directory, mut value) = fixture()?;
    let path = directory.path().join("proof-api.yaml");
    fs::write(&path, serde_yaml_ng::to_string(&value)?)?;
    let original = Config::load(&path)?;
    assert_eq!(
        original.prover.artifacts_dir,
        directory.path().join("artifacts")
    );
    original.load_proving_service()?;
    original
        .prover
        .approved_programs
        .validate_deployed_vkeys([1; 32], [2; 32], [3; 32])?;
    assert!(original
        .prover
        .approved_programs
        .validate_deployed_vkeys([9; 32], [2; 32], [3; 32])
        .is_err());

    // Rebuilding ELF plus generated metadata must not silently change approval.
    let replacement = identity(b"new-program", 4);
    fs::write(
        directory.path().join("artifacts/update-client.elf"),
        b"new-program",
    )?;
    let mut metadata: Value =
        serde_json::from_slice(&fs::read(directory.path().join("artifacts/vkeys.json"))?)?;
    metadata["update-client"] = serde_json::to_value(&replacement)?;
    fs::write(
        directory.path().join("artifacts/vkeys.json"),
        serde_json::to_vec(&metadata)?,
    )?;
    assert!(original.load_proving_service().is_err());

    // Explicitly approving the new identity in YAML accepts a new release without code changes.
    value["prover"]["approvedPrograms"]["update-client"] = serde_json::to_value(&replacement)?;
    fs::write(&path, serde_yaml_ng::to_string(&value)?)?;
    Config::load(&path)?.load_proving_service()?;
    assert!(original.load_proving_service().is_err());

    metadata["update-client"]["programVKey"] = json!(format!("0x{}", "55".repeat(32)));
    fs::write(
        directory.path().join("artifacts/vkeys.json"),
        serde_json::to_vec(&metadata)?,
    )?;
    assert!(Config::load(&path)?.load_proving_service().is_err());
    Ok(())
}

#[test]
fn rejects_missing_unknown_duplicate_and_malformed_identity_settings(
) -> Result<(), Box<dyn std::error::Error>> {
    let (_, original) = fixture()?;
    for role in ["update-client", "membership", "non-membership"] {
        let mut missing = original.clone();
        missing["prover"]["approvedPrograms"]
            .as_object_mut()
            .ok_or("expected object")?
            .remove(role);
        assert!(serde_yaml_ng::from_str::<Config>(&serde_yaml_ng::to_string(&missing)?).is_err());
        for field in ["elfSha256", "programVKey"] {
            for invalid in [
                String::new(),
                "0x12".to_owned(),
                "11".repeat(32),
                format!("0x{}", "00".repeat(32)),
                format!("0x{}", "gg".repeat(32)),
            ] {
                let mut bad = original.clone();
                bad["prover"]["approvedPrograms"][role][field] = json!(invalid);
                let config: Config = serde_yaml_ng::from_str(&serde_yaml_ng::to_string(&bad)?)?;
                assert!(config.validate().is_err(), "accepted {role}.{field}");
            }
        }
    }
    let mut unknown = original.clone();
    unknown["prover"]["approvedPrograms"]["membership"]["typo"] = json!(true);
    assert!(serde_yaml_ng::from_str::<Config>(&serde_yaml_ng::to_string(&unknown)?).is_err());
    let yaml = format!(
        "{}\nschemaVersion: 1\n",
        serde_yaml_ng::to_string(&original)?
    );
    assert!(serde_yaml_ng::from_str::<Config>(&yaml).is_err());
    Ok(())
}

#[test]
fn schema_size_empty_path_and_case_handling() -> Result<(), Box<dyn std::error::Error>> {
    let (directory, original) = fixture()?;
    let path = directory.path().join("config.yaml");
    for (field, replacement) in [
        ("schemaVersion", json!(2)),
        ("prover.artifactsDir", json!("")),
    ] {
        let mut bad = original.clone();
        if field == "schemaVersion" {
            bad[field] = replacement;
        } else {
            bad["prover"]["artifactsDir"] = replacement;
        }
        fs::write(&path, serde_yaml_ng::to_string(&bad)?)?;
        assert!(Config::load(&path).is_err());
    }
    fs::write(&path, vec![b' '; 65_537])?;
    assert!(Config::load(&path).is_err());
    let mut approved: ApprovedPrograms =
        serde_json::from_value(original["prover"]["approvedPrograms"].clone())?;
    approved.update.elf_sha256 = format!("0x{}", approved.update.elf_sha256[2..].to_uppercase());
    prover_script::proving_service::ProvingService::load_approved(
        &directory.path().join("artifacts"),
        &approved,
    )?;
    Ok(())
}

#[test]
fn cli_loads_yaml_and_rejects_unapproved_artifacts() -> Result<(), Box<dyn std::error::Error>> {
    let (directory, value) = fixture()?;
    let path = directory.path().join("proof-api.yaml");
    fs::write(&path, serde_yaml_ng::to_string(&value)?)?;
    for command in ["validate-config", "validate-artifacts"] {
        let output = std::process::Command::new(env!("CARGO_BIN_EXE_proof-api"))
            .args([command, "--config"])
            .arg(&path)
            .output()?;
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    fs::write(
        directory.path().join("artifacts/membership.elf"),
        b"tampered",
    )?;
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_proof-api"))
        .args(["validate-artifacts", "--config"])
        .arg(&path)
        .output()?;
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("membership ELF SHA-256"));
    Ok(())
}

#[test]
fn legacy_duration_names_and_runtime_validation() -> Result<(), Box<dyn std::error::Error>> {
    let (_, mut value) = fixture()?;
    value["server"] = json!({"listen":"0.0.0.0:8085"});
    value["client"] = json!({"chainId":"41002","clientId":"cli-41001-41002"});
    value["rpc"] = json!({"source":{"proxyUrl":"http://besu-a:8545"},"verifier":{"proxyUrl":"http://besu-b:8745"},
        "callTimeout":"30s","retryBackoff":"100ms","maxAttempts":2,"maxResponseBytes":16_777_216});
    value["contracts"] = json!({"verifierRouter":format!("0x{}","11".repeat(20))});
    value["scheduler"] = json!({"maxBatchWait":"3s","updateJobDeduplication":true});
    let config: Config = serde_yaml_ng::from_str(&serde_yaml_ng::to_string(&value)?)?;
    config.validate_runtime()?;
    assert_eq!(
        config.rpc.as_ref().ok_or("missing RPC")?.call_timeout_ms,
        30_000
    );
    assert_eq!(config.scheduler.max_batch_wait_ms, 3000);
    for (wait, expected_ms) in [
        (json!("5001ms"), 5001),
        (json!("30s"), 30_000),
        (json!("1h"), 3_600_000),
        (json!(u64::MAX), u64::MAX),
    ] {
        value["scheduler"]["maxBatchWait"] = wait;
        let config: Config = serde_yaml_ng::from_str(&serde_yaml_ng::to_string(&value)?)?;
        config.validate_runtime()?;
        assert_eq!(config.scheduler.max_batch_wait_ms, expected_ms);
    }
    value["scheduler"]["maxBatchWait"] = json!("1999ms");
    assert!(
        serde_yaml_ng::from_str::<Config>(&serde_yaml_ng::to_string(&value)?)?
            .validate_runtime()
            .is_err()
    );
    value["scheduler"]["maxBatchWait"] = json!("2s");
    serde_yaml_ng::from_str::<Config>(&serde_yaml_ng::to_string(&value)?)?.validate_runtime()?;
    value["scheduler"]["updateJobDeduplication"] = json!(false);
    assert!(
        serde_yaml_ng::from_str::<Config>(&serde_yaml_ng::to_string(&value)?)?
            .validate()
            .is_err()
    );
    value["scheduler"]["updateJobDeduplication"] = json!(true);
    value["contracts"]["verifierRouter"] = json!(format!("0x{}", "00".repeat(20)));
    assert!(
        serde_yaml_ng::from_str::<Config>(&serde_yaml_ng::to_string(&value)?)?
            .validate()
            .is_err()
    );
    Ok(())
}
