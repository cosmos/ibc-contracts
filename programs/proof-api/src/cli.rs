// SPDX-License-Identifier: Apache-2.0

//! Defines the client interface for the proof API server.

use clap::Parser;

/// The command line interface for the proof API.
#[derive(Clone, Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct ProofApiCli {
    /// The subcommand to run.
    #[command(subcommand)]
    pub command: Commands,
}

/// The subcommands for the proof API.
#[derive(Clone, Debug, Parser)]
pub enum Commands {
    /// The subcommand to run the proof API.
    Start(start::Args),
}

/// The arguments for the start subcommand.
pub mod start {
    use super::Parser;

    /// The arguments for the start subcommand.
    #[derive(Clone, Debug, Parser)]
    pub struct Args {
        /// The configuration file for the proof API.
        #[clap(short = 'c', long)]
        pub config: String,
    }
}
