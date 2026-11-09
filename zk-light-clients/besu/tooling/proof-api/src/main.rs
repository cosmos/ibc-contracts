use clap::{Parser, Subcommand};
use proof_api::config::Config;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "proof-api", about = "Unified Besu Proof API")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run the complete Proof API with in-process witness construction and proving.
    Serve {
        #[arg(long)]
        config: PathBuf,
    },
    /// Validate loaded artifacts and the deployed chain integration without generating proofs.
    ValidateIntegration {
        #[arg(long)]
        config: PathBuf,
    },
    Version,
    /// Check local readiness (used by Docker without an additional HTTP utility).
    Healthcheck {
        #[arg(long, default_value = "127.0.0.1:8085")]
        address: std::net::SocketAddr,
    },

    /// Validate the YAML schema and configured program identities without accessing RPCs.
    ValidateConfig {
        #[arg(long)]
        config: PathBuf,
    },
    /// Check that the loaded ELF generation and verification keys match the configured approval.
    ValidateArtifacts {
        #[arg(long)]
        config: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();
    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(runtime) => runtime,
        Err(error) => {
            prover_script::logging::log(
                serde_json::json!({"component":"proof-api", "event":"startup_failed", "error":error.to_string()}),
            );
            std::process::exit(1);
        }
    };
    let result = runtime.block_on(run(cli));
    runtime.shutdown_timeout(std::time::Duration::from_secs(5));
    if let Err(error) = result {
        prover_script::logging::log(
            serde_json::json!({"component":"proof-api", "event":"startup_failed", "error":error.to_string()}),
        );
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    match cli.command {
        Command::Serve { config } => proof_api::runtime::serve(Config::load(&config)?).await?,
        Command::ValidateIntegration { config } => {
            let config = Config::load(&config)?;
            config.validate_runtime()?;
            config.load_proving_service()?;
            let rpc = config.rpc.as_ref().ok_or("rpc is required")?;
            let (source, verifier) = rpc.clients()?;
            tokio::time::timeout(
                std::time::Duration::from_secs(60),
                proof_api::integration::validate(
                    &source,
                    &verifier,
                    rpc,
                    config.client.as_ref().ok_or("client is required")?,
                    config.contracts.as_ref().ok_or("contracts are required")?,
                    &config.prover.approved_programs,
                ),
            )
            .await??;
            println!("contract integration valid");
        }
        Command::Healthcheck { address } => {
            reqwest::Client::new()
                .get(format!("http://{address}/readyz"))
                .timeout(std::time::Duration::from_secs(3))
                .send()
                .await?
                .error_for_status()?;
        }
        Command::Version => println!("proof-api {}", env!("CARGO_PKG_VERSION")),
        Command::ValidateConfig { config } => {
            Config::load(&config)?;
            println!("configuration valid");
        }
        Command::ValidateArtifacts { config } => {
            Config::load(&config)?.load_proving_service()?;
            println!("artifacts match configured approval");
        }
    }
    Ok(())
}
