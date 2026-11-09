use std::{collections::BTreeMap, fs::File, io::Read, path::Path};

use besu_zk_types::{MembershipWitness, NonMembershipWitness, UpdateWitness};
use serde::de::DeserializeOwned;

use crate::{identity::ProgramIdentity, ToolError};

const MAX_WITNESS_JSON_BYTES: usize = 16 * 1024 * 1024;

pub fn load_update_witness(path: &Path) -> Result<UpdateWitness, ToolError> {
    let witness = read_witness(path)?;
    besu_qbft::verify_update(&witness)
        .map_err(|error| ToolError::InvalidWitness(error.to_string()))?;
    Ok(witness)
}

pub fn load_membership_witness(path: &Path) -> Result<MembershipWitness, ToolError> {
    let witness = read_witness(path)?;
    ibc_storage::verify_membership(&witness)
        .map_err(|error| ToolError::InvalidWitness(error.to_string()))?;
    Ok(witness)
}

pub fn load_non_membership_witness(path: &Path) -> Result<NonMembershipWitness, ToolError> {
    let witness = read_witness(path)?;
    ibc_storage::verify_non_membership(&witness)
        .map_err(|error| ToolError::InvalidWitness(error.to_string()))?;
    Ok(witness)
}

pub fn validate_elf(program: &str, elf: &[u8]) -> Result<(), ToolError> {
    let identity = program_identity(program)?;
    validate_elf_identity(program, elf, &identity)
}

pub fn validate_elf_identity(
    program: &str,
    elf: &[u8],
    identity: &ProgramIdentity,
) -> Result<(), ToolError> {
    let actual = format!("0x{}", hex::encode(besu_zk_types::sha256(elf)));
    if !actual.eq_ignore_ascii_case(&identity.elf_sha256) {
        return Err(ToolError::InvalidProgramIdentity(format!(
            "{program} ELF SHA-256 is {actual}; expected {}",
            identity.elf_sha256
        )));
    }
    Ok(())
}

pub fn validate_vkey(program: &str, vkey: &str) -> Result<(), ToolError> {
    let identity = program_identity(program)?;
    validate_vkey_identity(program, vkey, &identity)
}

pub fn validate_vkey_identity(
    program: &str,
    vkey: &str,
    identity: &ProgramIdentity,
) -> Result<(), ToolError> {
    if !vkey.eq_ignore_ascii_case(&identity.program_v_key) {
        return Err(ToolError::InvalidProgramIdentity(format!(
            "{program} VKey is {vkey}; expected {}",
            identity.program_v_key
        )));
    }
    Ok(())
}

fn read_witness<W: DeserializeOwned>(path: &Path) -> Result<W, ToolError> {
    let file = File::open(path)?;
    let mut limited = file.take((MAX_WITNESS_JSON_BYTES + 1) as u64);
    let mut encoded = Vec::new();
    limited.read_to_end(&mut encoded)?;
    ensure_within_limit(encoded.len())?;
    serde_json::from_slice(&encoded).map_err(|error| ToolError::InvalidWitness(error.to_string()))
}

fn ensure_within_limit(bytes_read: usize) -> Result<(), ToolError> {
    if bytes_read > MAX_WITNESS_JSON_BYTES {
        return Err(ToolError::WitnessInputTooLarge {
            max_bytes: MAX_WITNESS_JSON_BYTES,
        });
    }
    Ok(())
}

fn program_identity(program: &str) -> Result<ProgramIdentity, ToolError> {
    let directory = std::env::var_os("BESU_PROVER_ARTIFACTS_DIR").map_or_else(
        || std::path::PathBuf::from("artifacts"),
        std::path::PathBuf::from,
    );
    identity_from_directory(program, &directory)
}

pub fn identity_from_directory(
    program: &str,
    directory: &Path,
) -> Result<ProgramIdentity, ToolError> {
    let encoded = std::fs::read(directory.join("vkeys.json"))?;
    let identities: BTreeMap<String, ProgramIdentity> = serde_json::from_slice(&encoded)
        .map_err(|error| ToolError::InvalidProgramIdentity(error.to_string()))?;
    identities
        .get(program)
        .cloned()
        .ok_or_else(|| ToolError::InvalidProgramIdentity(format!("unknown program {program}")))
}
