use prover_script::host::{serve, HostConfig};

#[tokio::main]
async fn main() {
    let config = match HostConfig::from_env() {
        Ok(config) => config,
        Err(error) => {
            eprintln!("error: {error}");
            std::process::exit(1);
        }
    };

    if let Err(error) = serve(config).await {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}
