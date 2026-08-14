// SPDX-License-Identifier: Apache-2.0

//! Contains the command line interface for the application.

use clap::Parser;
use sp1_ics07_tendermint_prover::prover::SupportedZkAlgorithm;
use std::{convert::Infallible, str::FromStr};
use tendermint_light_client_verifier::types::TrustThreshold;

/// The command line interface for the operator.
#[derive(Clone, Parser)]
#[command(author, version, about, long_about = None)]
pub struct OperatorCli {
    /// The subcommand to run.
    #[command(subcommand)]
    pub command: Commands,
}

/// The subcommands for the operator.
#[derive(Clone, Parser)]
pub enum Commands {
    /// The subcommand to produce the fixtures for testing.
    Fixtures(fixtures::Cmd),
    /// Convert Solana misbehaviour from protobuf to Borsh format (stdin → stdout)
    SolanaMisbehaviourToBorsh,
}

/// The trust options for client operations.
#[derive(Clone, Parser)]
pub struct TrustOptions {
    /// Trust level.
    #[clap(
        long,
        default_value = "1/3",
        value_parser = parse_trust_threshold,
        help = "Trust level as a fraction, e.g. '2/3'",
    )]
    pub trust_level: TrustThreshold,

    /// Trusting period. [default: 2/3 of unbonding period]
    #[clap(long)]
    pub trusting_period: Option<u32>,
}

/// The output path for files.
#[derive(Clone)]
pub enum OutputPath {
    /// Write the output to stdout.
    Stdout,
    /// Write the output to a file.
    File(String),
}

/// The cli interface for the fixtures.
pub mod fixtures {
    use super::Parser;

    /// The cli interface for the fixtures.
    #[derive(Clone, Parser)]
    #[command(about = "Generate fixtures for SP1ICS07Tendermint contract")]
    pub struct Cmd {
        /// The subcommand to run.
        #[command(subcommand)]
        pub command: Cmds,
    }

    /// The subcommands for the fixtures.
    #[derive(Clone, Parser)]
    pub enum Cmds {
        /// The subcommand to generate the update client fixtures.
        UpdateClient(UpdateClientCmd),
        /// The subcommand to generate the verify (non)membership fixtures.
        Membership(MembershipCmd),
        /// The subcommand to generate the update client and verify (non)membership fixtures.
        UpdateClientAndMembership(UpdateClientAndMembershipCmd),
        /// The subcommand to generate the misbehaviour fixtures.
        Misbehaviour(MisbehaviourCmd),
    }

    /// The arguments for the `UpdateClient` fixture executable.
    #[derive(Parser, Clone)]
    #[command(about = "Generate the update client fixture")]
    pub struct UpdateClientCmd {
        /// Trusted block.
        #[clap(long)]
        pub trusted_block: u64,

        /// Target block.
        #[clap(long, env)]
        pub target_block: u64,

        /// Fixture path. If not provided, the output will be written to stdout.
        #[clap(long, short = 'o', value_parser = super::parse_output_path, default_value = "-")]
        pub output_path: super::OutputPath,

        /// Trust options
        #[clap(flatten)]
        pub trust_options: super::TrustOptions,

        /// Sp1 configuration
        #[clap(flatten)]
        pub sp1: Sp1Args,

        /// The paths to the ELF files.
        #[clap(flatten)]
        pub elf_paths: super::SP1ELFPaths,
    }

    /// The arguments for the `Membership` fixture executable.
    #[derive(Parser, Clone)]
    #[command(about = "Generate the verify (non)membership fixture")]
    pub struct MembershipCmd {
        /// Generic membership arguments.
        #[clap(flatten)]
        pub membership: MembershipArgs,

        /// Sp1 configuration
        #[clap(flatten)]
        pub sp1: Sp1Args,

        /// The paths to the ELF files.
        #[clap(flatten)]
        pub elf_paths: super::SP1ELFPaths,
    }

    /// The arguments for generic membership proof generation.
    #[derive(Parser, Clone)]
    pub struct MembershipArgs {
        /// Trusted block.
        #[clap(long)]
        pub trusted_block: u64,

        /// Key paths to prove membership.
        #[clap(long, value_delimiter = ',')]
        pub key_paths: Vec<String>,

        /// Fixture path. If not provided, the output will be written to stdout.
        #[clap(long, short = 'o', value_parser = super::parse_output_path, default_value = "-")]
        pub output_path: super::OutputPath,

        /// Trust options
        #[clap(flatten)]
        pub trust_options: super::TrustOptions,

        /// Indicates that the key paths are base64 encoded.
        /// Module store keys seperated by backslash, '\', eg. 'aWJj\a2V5' for 'ibc/key'.
        #[clap(long)]
        pub base64: bool,
    }

    /// The arguments for the `UpdateClientAndMembership` fixture executable.
    #[derive(Parser, Clone)]
    #[command(about = "Generate the update client and membership fixture")]
    pub struct UpdateClientAndMembershipCmd {
        /// Target block.
        #[clap(long, env)]
        pub target_block: u64,

        /// Membership arguments.
        #[clap(flatten)]
        pub membership: MembershipArgs,

        /// Sp1 configuration
        #[clap(flatten)]
        pub sp1: Sp1Args,

        /// The paths to the ELF files.
        #[clap(flatten)]
        pub elf_paths: super::SP1ELFPaths,
    }

    /// The arguments for the `Misbehaviour` fixture executable.
    #[derive(Parser, Clone)]
    #[command(about = "Generate the misbehaviour fixture")]
    pub struct MisbehaviourCmd {
        /// Path to the misbehaviour json file.
        #[clap(long)]
        pub misbehaviour_json_path: String,

        /// Fixture path. If not provided, the output will be written to stdout.
        #[clap(long, short = 'o', value_parser = super::parse_output_path, default_value = "-")]
        pub output_path: super::OutputPath,

        /// Trust options
        #[clap(flatten)]
        pub trust_options: super::TrustOptions,

        /// Sp1 configuration
        #[clap(flatten)]
        pub sp1: Sp1Args,

        /// The paths to the ELF files.
        #[clap(flatten)]
        pub elf_paths: super::SP1ELFPaths,
    }

    /// The arguments for sp1 configuration.
    #[derive(Parser, Clone)]
    pub struct Sp1Args {
        /// The proof type
        /// Supported proof types: groth16, plonk.
        #[clap(long, short = 'p', value_parser = super::parse_proof_type, default_value = "plonk")]
        pub proof_type: super::SupportedZkAlgorithm,

        /// Running with a private cluster or not
        /// If true, the operator will use the private cluster configuration.
        #[clap(long, default_value = "false")]
        pub private_cluster: bool,
    }
}

/// The paths to the ELF files.
#[derive(Parser, Clone)]
pub struct SP1ELFPaths {
    /// Path to the update client ELF file.
    #[clap(
        long,
        default_value = "ibc-solidity/programs/sp1-programs/target/elf-compilation/riscv64im-succinct-zkvm-elf/release/sp1-ics07-tendermint-update-client"
    )]
    pub update_client_path: String,
    /// Path to the membership ELF file.
    #[clap(
        long,
        default_value = "ibc-solidity/programs/sp1-programs/target/elf-compilation/riscv64im-succinct-zkvm-elf/release/sp1-ics07-tendermint-membership"
    )]
    pub membership_path: String,
    /// Path to the update client and membership ELF file.
    #[clap(
        long,
        default_value = "ibc-solidity/programs/sp1-programs/target/elf-compilation/riscv64im-succinct-zkvm-elf/release/sp1-ics07-tendermint-uc-and-membership"
    )]
    pub uc_and_membership_path: String,
    /// Path to the misbehaviour ELF file.
    #[clap(
        long,
        default_value = "ibc-solidity/programs/sp1-programs/target/elf-compilation/riscv64im-succinct-zkvm-elf/release/sp1-ics07-tendermint-misbehaviour"
    )]
    pub misbehaviour_path: String,
}

#[allow(clippy::unnecessary_wraps)]
fn parse_output_path(path: &str) -> Result<OutputPath, Infallible> {
    if path == "-" {
        Ok(OutputPath::Stdout)
    } else {
        Ok(OutputPath::File(path.to_string()))
    }
}

fn parse_trust_threshold(input: &str) -> anyhow::Result<TrustThreshold> {
    let (num_part, denom_part) = input.split_once('/').ok_or_else(|| {
        anyhow::anyhow!("invalid trust threshold fraction: expected format 'numerator/denominator'")
    })?;
    let numerator = num_part
        .trim()
        .parse()
        .map_err(|_| anyhow::anyhow!("invalid numerator for the fraction"))?;
    let denominator = denom_part
        .trim()
        .parse()
        .map_err(|_| anyhow::anyhow!("invalid denominator for the fraction"))?;
    TrustThreshold::new(numerator, denominator)
        .map_err(|e| anyhow::anyhow!("invalid trust threshold: {e}"))
}

fn parse_proof_type(input: &str) -> anyhow::Result<SupportedZkAlgorithm> {
    Ok(SupportedZkAlgorithm::from_str(input)?)
}
