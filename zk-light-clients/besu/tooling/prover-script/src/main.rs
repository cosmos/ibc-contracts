use std::{path::PathBuf, time::Duration};

use besu_public_values::{decode_membership, decode_non_membership, decode_update};
use clap::{Args, Parser, Subcommand};
use prover_script::{
    artifact::{MembershipBatchArtifact, ProofArtifact},
    prepared::{
        load_membership_witness, load_non_membership_witness, load_update_witness, validate_elf,
        validate_vkey,
    },
    prover::{execute, prove_groth16, serialized_witness_metrics, SerializedWitnessMetrics},
    rpc::{fixed_hex, variable_hex, RpcClient},
    witness::{
        build_membership_witness, build_non_membership_witness, build_update_witness,
        StateProofInputs,
    },
    ToolError,
};

#[derive(Parser)]
#[command(
    name = "besu-zk-prover",
    about = "Local Besu QBFT SP1 witness and proof tool"
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    ExecuteUpdate(UpdateArgs),
    ProveUpdate(ProveUpdateArgs),
    ProveUpdateWitness(PreparedWitnessArgs),
    ExecuteMembership(MembershipArgs),
    ProveMembership(ProveMembershipArgs),
    ProveMembershipWitness(PreparedWitnessArgs),
    ExecuteNonMembership(NonMembershipArgs),
    ProveNonMembership(ProveNonMembershipArgs),
    ProveNonMembershipWitness(PreparedWitnessArgs),
}

#[derive(Clone, Args)]
struct RpcArgs {
    #[arg(long, env = "SOURCE_RPC_URL")]
    source_rpc: String,
    #[arg(long, env = "DESTINATION_RPC_URL")]
    destination_rpc: String,
    #[arg(long, env = "EXPECTED_SOURCE_CHAIN_ID")]
    expected_source_chain_id: u64,
    #[arg(long, default_value_t = 30)]
    rpc_timeout_seconds: u64,
    #[arg(long, env = "SOURCE_VALIDATOR_MODE")]
    source_validator_mode: String,
    #[arg(long, env = "SP1_PROOF_MODE")]
    proof_mode: String,
    #[arg(long, env = "TRUSTING_PERIOD_SECONDS")]
    trusting_period_seconds: u64,
    #[arg(long, env = "MAX_CLOCK_DRIFT_SECONDS")]
    max_clock_drift_seconds: u64,
}

#[derive(Clone, Args)]
struct ProgramArgs {
    #[arg(long)]
    elf: PathBuf,
}

#[derive(Clone, Args)]
struct PreparedWitnessArgs {
    #[command(flatten)]
    program: ProgramArgs,
    #[arg(long, value_name = "PATH")]
    witness: PathBuf,
    #[arg(long, value_name = "PATH")]
    output: PathBuf,
}

#[derive(Clone, Args)]
struct UpdateArgs {
    #[command(flatten)]
    rpc: RpcArgs,
    #[command(flatten)]
    program: ProgramArgs,
    #[arg(long, env = "DESTINATION_LIGHT_CLIENT")]
    destination_light_client: String,
    #[arg(long)]
    trusted_height: u64,
    #[arg(long)]
    target_height: u64,
    #[arg(long, value_name = "PATH")]
    witness_output: Option<PathBuf>,
}

#[derive(Clone, Args)]
struct ProveUpdateArgs {
    #[command(flatten)]
    update: UpdateArgs,
    #[arg(long)]
    output: PathBuf,
}

#[derive(Clone, Args)]
struct StateProofArgs {
    #[command(flatten)]
    rpc: RpcArgs,
    #[command(flatten)]
    program: ProgramArgs,
    #[arg(long, env = "SOURCE_IBC_ROUTER")]
    source_router: String,
    #[arg(long, env = "DESTINATION_LIGHT_CLIENT")]
    destination_light_client: String,
    #[arg(long)]
    proof_height: u64,
    #[arg(long, value_name = "0xHEX")]
    path: String,
    #[arg(long, value_name = "PATH")]
    witness_output: Option<PathBuf>,
}

#[derive(Clone, Args)]
struct MembershipArgs {
    #[command(flatten)]
    state: StateProofArgs,
    #[arg(long, value_name = "0xBYTES32")]
    expected_commitment: String,
}

#[derive(Clone, Args)]
struct ProveMembershipArgs {
    #[command(flatten)]
    membership: MembershipArgs,
    #[arg(long)]
    output: PathBuf,
}

#[derive(Clone, Args)]
struct NonMembershipArgs {
    #[command(flatten)]
    state: StateProofArgs,
}

#[derive(Clone, Args)]
struct ProveNonMembershipArgs {
    #[command(flatten)]
    non_membership: NonMembershipArgs,
    #[arg(long)]
    output: PathBuf,
}

#[tokio::main]
async fn main() {
    if let Err(error) = run(Cli::parse()).await {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> Result<(), ToolError> {
    match cli.command {
        Command::ExecuteUpdate(args) => execute_update(&args).await,
        Command::ProveUpdate(args) => prove_update(&args).await,
        Command::ProveUpdateWitness(args) => prove_update_witness(&args).await,
        Command::ExecuteMembership(args) => execute_membership(&args).await,
        Command::ProveMembership(args) => prove_membership(&args).await,
        Command::ProveMembershipWitness(args) => prove_membership_witness(&args).await,
        Command::ExecuteNonMembership(args) => execute_non_membership(&args).await,
        Command::ProveNonMembership(args) => prove_non_membership(&args).await,
        Command::ProveNonMembershipWitness(args) => prove_non_membership_witness(&args).await,
    }
}

fn clients(args: &RpcArgs) -> Result<(RpcClient, RpcClient), ToolError> {
    validate_configuration(args)?;
    let timeout = Duration::from_secs(args.rpc_timeout_seconds);
    Ok((
        RpcClient::new(&args.source_rpc, timeout)?,
        RpcClient::new(&args.destination_rpc, timeout)?,
    ))
}

fn validate_configuration(args: &RpcArgs) -> Result<(), ToolError> {
    if args.source_validator_mode != "header-qbft" {
        return Err(ToolError::UnsupportedValidatorMode);
    }
    if args.proof_mode != "groth16" {
        return Err(ToolError::UnsupportedProofMode);
    }
    // Both policy values are parsed directly as u64 by clap. They are intentionally not part of
    // any witness: Solidity applies these destination-chain time policies.
    let _ = (args.trusting_period_seconds, args.max_clock_drift_seconds);
    Ok(())
}

async fn update_witness(args: &UpdateArgs) -> Result<besu_zk_types::UpdateWitness, ToolError> {
    let (source, destination) = clients(&args.rpc)?;
    build_update_witness(
        &source,
        &destination,
        args.rpc.expected_source_chain_id,
        fixed_hex(&args.destination_light_client, "destination light client")?,
        args.trusted_height,
        args.target_height,
    )
    .await
}

fn state_proof_inputs(args: &StateProofArgs) -> Result<StateProofInputs, ToolError> {
    Ok(StateProofInputs {
        expected_source_chain_id: args.rpc.expected_source_chain_id,
        source_router: fixed_hex(&args.source_router, "source Router")?,
        destination_light_client: fixed_hex(
            &args.destination_light_client,
            "destination light client",
        )?,
        proof_height: args.proof_height,
        path: vec![variable_hex(&args.path, "path")?],
    })
}

async fn membership_witness(
    args: &MembershipArgs,
) -> Result<besu_zk_types::MembershipWitness, ToolError> {
    let (source, destination) = clients(&args.state.rpc)?;
    build_membership_witness(
        &source,
        &destination,
        state_proof_inputs(&args.state)?,
        fixed_hex(&args.expected_commitment, "expected commitment")?,
    )
    .await
}

async fn non_membership_witness(
    args: &NonMembershipArgs,
) -> Result<besu_zk_types::NonMembershipWitness, ToolError> {
    let (source, destination) = clients(&args.state.rpc)?;
    build_non_membership_witness(&source, &destination, state_proof_inputs(&args.state)?).await
}

async fn execute_update(args: &UpdateArgs) -> Result<(), ToolError> {
    let witness = update_witness(args).await?;
    write_witness_if_requested(&witness, args.witness_output.as_deref())?;
    let metrics = serialized_witness_metrics(&witness);
    let elf = std::fs::read(&args.program.elf)?;
    let output = execute(&elf, &witness).await?;
    decode_update(&output.public_values).map_err(|error| ToolError::Sp1(error.to_string()))?;
    print_execution(&output, &metrics);
    Ok(())
}

async fn prove_update(args: &ProveUpdateArgs) -> Result<(), ToolError> {
    let witness = update_witness(&args.update).await?;
    write_witness_if_requested(&witness, args.update.witness_output.as_deref())?;
    let elf = std::fs::read(&args.update.program.elf)?;
    let output = prove_groth16(&elf, &witness).await?;
    decode_update(&output.public_values).map_err(|error| ToolError::Sp1(error.to_string()))?;
    write_artifact("update-client", &elf, &output, &args.output)
}

async fn prove_update_witness(args: &PreparedWitnessArgs) -> Result<(), ToolError> {
    let witness = load_update_witness(&args.witness)?;
    let elf = std::fs::read(&args.program.elf)?;
    validate_elf("update-client", &elf)?;
    let output = prove_groth16(&elf, &witness).await?;
    decode_update(&output.public_values).map_err(|error| ToolError::Sp1(error.to_string()))?;
    validate_vkey("update-client", &output.program_v_key)?;
    write_artifact("update-client", &elf, &output, &args.output)
}

async fn execute_membership(args: &MembershipArgs) -> Result<(), ToolError> {
    let witness = membership_witness(args).await?;
    write_witness_if_requested(&witness, args.state.witness_output.as_deref())?;
    let metrics = serialized_witness_metrics(&witness);
    let elf = std::fs::read(&args.state.program.elf)?;
    let output = execute(&elf, &witness).await?;
    decode_membership(&output.public_values).map_err(|error| ToolError::Sp1(error.to_string()))?;
    print_execution(&output, &metrics);
    Ok(())
}

async fn prove_membership(args: &ProveMembershipArgs) -> Result<(), ToolError> {
    let witness = membership_witness(&args.membership).await?;
    write_witness_if_requested(&witness, args.membership.state.witness_output.as_deref())?;
    let elf = std::fs::read(&args.membership.state.program.elf)?;
    let output = prove_groth16(&elf, &witness).await?;
    decode_membership(&output.public_values).map_err(|error| ToolError::Sp1(error.to_string()))?;
    write_membership_artifact(&elf, &witness, &output, &args.output)
}

async fn prove_membership_witness(args: &PreparedWitnessArgs) -> Result<(), ToolError> {
    let witness = load_membership_witness(&args.witness)?;
    let elf = std::fs::read(&args.program.elf)?;
    validate_elf("membership", &elf)?;
    let output = prove_groth16(&elf, &witness).await?;
    decode_membership(&output.public_values).map_err(|error| ToolError::Sp1(error.to_string()))?;
    validate_vkey("membership", &output.program_v_key)?;
    write_membership_artifact(&elf, &witness, &output, &args.output)
}

async fn execute_non_membership(args: &NonMembershipArgs) -> Result<(), ToolError> {
    let witness = non_membership_witness(args).await?;
    write_witness_if_requested(&witness, args.state.witness_output.as_deref())?;
    let metrics = serialized_witness_metrics(&witness);
    let elf = std::fs::read(&args.state.program.elf)?;
    let output = execute(&elf, &witness).await?;
    decode_non_membership(&output.public_values)
        .map_err(|error| ToolError::Sp1(error.to_string()))?;
    print_execution(&output, &metrics);
    Ok(())
}

async fn prove_non_membership(args: &ProveNonMembershipArgs) -> Result<(), ToolError> {
    let witness = non_membership_witness(&args.non_membership).await?;
    write_witness_if_requested(
        &witness,
        args.non_membership.state.witness_output.as_deref(),
    )?;
    let elf = std::fs::read(&args.non_membership.state.program.elf)?;
    let output = prove_groth16(&elf, &witness).await?;
    decode_non_membership(&output.public_values)
        .map_err(|error| ToolError::Sp1(error.to_string()))?;
    write_artifact("non-membership", &elf, &output, &args.output)
}

async fn prove_non_membership_witness(args: &PreparedWitnessArgs) -> Result<(), ToolError> {
    let witness = load_non_membership_witness(&args.witness)?;
    let elf = std::fs::read(&args.program.elf)?;
    validate_elf("non-membership", &elf)?;
    let output = prove_groth16(&elf, &witness).await?;
    decode_non_membership(&output.public_values)
        .map_err(|error| ToolError::Sp1(error.to_string()))?;
    validate_vkey("non-membership", &output.program_v_key)?;
    write_artifact("non-membership", &elf, &output, &args.output)
}

fn write_artifact(
    program: &str,
    elf: &[u8],
    output: &prover_script::prover::ProofOutput,
    path: &std::path::Path,
) -> Result<(), ToolError> {
    ProofArtifact::new(
        program,
        elf,
        &output.program_v_key,
        &output.public_values,
        &output.proof_bytes,
    )?
    .write(path)?;
    println!("wrote {} Groth16 proof artifact", path.display());
    Ok(())
}

fn write_membership_artifact(
    elf: &[u8],
    witness: &besu_zk_types::MembershipWitness,
    output: &prover_script::prover::ProofOutput,
    path: &std::path::Path,
) -> Result<(), ToolError> {
    MembershipBatchArtifact::new(
        elf,
        &output.program_v_key,
        witness,
        &output.public_values,
        &output.proof_bytes,
    )?
    .write(path)?;
    println!("wrote {} batched membership relay artifact", path.display());
    Ok(())
}

fn write_witness_if_requested<W: serde::Serialize>(
    witness: &W,
    path: Option<&std::path::Path>,
) -> Result<(), ToolError> {
    let Some(path) = path else {
        return Ok(());
    };
    let encoded = serde_json::to_vec_pretty(witness)
        .map_err(|error| ToolError::InvalidWitness(error.to_string()))?;
    prover_script::json_file::write_atomic(path, &encoded)?;
    println!("wrote {} normalized witness", path.display());
    Ok(())
}

fn print_execution(
    output: &prover_script::prover::ExecutionOutput,
    witness: &SerializedWitnessMetrics,
) {
    println!("publicValues=0x{}", hex::encode(&output.public_values));
    println!("instructions={}", output.instruction_count);
    println!("sp1Cycles={}", output.instruction_count);
    println!("serializedWitnessBytes={}", witness.byte_len);
    println!("witnessSha256=0x{}", hex::encode(witness.sha256));
}

#[cfg(test)]
mod tests {
    use super::{Cli, Command};
    use clap::Parser;
    use std::io;

    fn update_args() -> Vec<&'static str> {
        vec![
            "besu-zk-prover",
            "execute-update",
            "--source-rpc",
            "http://source.invalid",
            "--destination-rpc",
            "http://destination.invalid",
            "--expected-source-chain-id",
            "1",
            "--source-validator-mode",
            "header-qbft",
            "--proof-mode",
            "groth16",
            "--trusting-period-seconds",
            "100",
            "--max-clock-drift-seconds",
            "10",
            "--elf",
            "update-client.elf",
            "--destination-light-client",
            "0x1111111111111111111111111111111111111111",
            "--trusted-height",
            "7",
            "--target-height",
            "9",
        ]
    }

    #[test]
    fn parses_complete_runtime_configuration() -> Result<(), Box<dyn std::error::Error>> {
        let cli = Cli::try_parse_from(update_args())?;
        assert!(matches!(cli.command, Command::ExecuteUpdate(_)));
        Ok(())
    }

    #[test]
    fn parses_optional_witness_output() -> Result<(), Box<dyn std::error::Error>> {
        let mut args = update_args();
        args.extend(["--witness-output", "/tmp/besu-zk-witness.json"]);
        let cli = Cli::try_parse_from(args)?;
        let Command::ExecuteUpdate(update) = cli.command else {
            return Err(io::Error::other("wrong command").into());
        };
        assert_eq!(
            update.witness_output,
            Some(std::path::PathBuf::from("/tmp/besu-zk-witness.json"))
        );
        Ok(())
    }

    #[test]
    fn witness_output_is_complete_and_atomically_renamed() -> Result<(), Box<dyn std::error::Error>>
    {
        let path = std::env::temp_dir().join(format!(
            "besu-zk-normalized-witness-test-{}.json",
            std::process::id()
        ));
        let temporary = path.with_extension(format!("tmp-{}", std::process::id()));
        let witness = serde_json::json!({"height": 7, "nodes": ["0x01"]});

        super::write_witness_if_requested(&witness, Some(&path))?;
        let stored: serde_json::Value = serde_json::from_slice(&std::fs::read(&path)?)?;
        assert_eq!(stored, witness);
        assert!(!temporary.exists());
        std::fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn rejects_policy_values_outside_u64() -> Result<(), Box<dyn std::error::Error>> {
        let mut args = update_args();
        let index = args
            .iter()
            .position(|value| *value == "100")
            .ok_or_else(|| io::Error::other("missing trusting-period fixture"))?;
        args[index] = "18446744073709551616";
        assert!(Cli::try_parse_from(args).is_err());
        Ok(())
    }

    #[test]
    fn configuration_modes_fail_closed_before_rpc_use() -> Result<(), Box<dyn std::error::Error>> {
        let mut args = update_args();
        let validator = args
            .iter()
            .position(|value| *value == "header-qbft")
            .ok_or_else(|| io::Error::other("missing validator-mode fixture"))?;
        args[validator] = "contract";
        let cli = Cli::try_parse_from(args)?;
        let Command::ExecuteUpdate(update) = cli.command else {
            return Err(io::Error::other("wrong command").into());
        };
        assert!(super::clients(&update.rpc).is_err());

        let mut args = update_args();
        let proof = args
            .iter()
            .position(|value| *value == "groth16")
            .ok_or_else(|| io::Error::other("missing proof-mode fixture"))?;
        args[proof] = "plonk";
        let cli = Cli::try_parse_from(args)?;
        let Command::ExecuteUpdate(update) = cli.command else {
            return Err(io::Error::other("wrong command").into());
        };
        assert!(super::clients(&update.rpc).is_err());
        Ok(())
    }
}
