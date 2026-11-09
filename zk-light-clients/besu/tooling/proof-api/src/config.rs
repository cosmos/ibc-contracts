use std::{
    fs::File,
    io::Read,
    path::{Path, PathBuf},
};

use prover_script::rpc::{RpcClient, RpcSettings};
use prover_script::{identity::ApprovedPrograms, proving_service::ProvingService, ToolError};
use serde::Deserialize;
use std::{net::SocketAddr, time::Duration};
use thiserror::Error;

const MAX_CONFIG_BYTES: u64 = 65_536;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("read Proof API configuration: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid Proof API YAML: {0}")]
    Yaml(#[from] serde_yaml_ng::Error),
    #[error("Proof API configuration exceeds 65536 bytes")]
    TooLarge,
    #[error("schemaVersion must be 1")]
    SchemaVersion,
    #[error("prover.artifactsDir must not be empty")]
    EmptyArtifactsDirectory,
    #[error(transparent)]
    Identity(#[from] ToolError),
    #[error("invalid runtime configuration: {0}")]
    Runtime(&'static str),
}

/// Runtime configuration grows here as the remaining engine is integrated.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Config {
    pub schema_version: u32,
    pub prover: ProverConfig,
    pub server: Option<ServerConfig>,
    pub client: Option<ClientConfig>,
    pub rpc: Option<RpcConfig>,
    pub contracts: Option<ContractsConfig>,
    #[serde(default)]
    pub scheduler: SchedulerConfig,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProverConfig {
    pub artifacts_dir: PathBuf,
    pub approved_programs: ApprovedPrograms,
}

impl Config {
    /// Relative artifact paths are resolved from the YAML file's directory.
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        let path = std::path::absolute(path)?;
        let mut bytes = Vec::new();
        File::open(&path)?
            .take(MAX_CONFIG_BYTES + 1)
            .read_to_end(&mut bytes)?;
        if bytes.len() as u64 > MAX_CONFIG_BYTES {
            return Err(ConfigError::TooLarge);
        }
        let mut config: Self = serde_yaml_ng::from_slice(&bytes)?;
        config.validate()?;
        if config.prover.artifacts_dir.is_relative() {
            config.prover.artifacts_dir = path
                .parent()
                .unwrap_or(Path::new("."))
                .join(&config.prover.artifacts_dir);
        }
        Ok(config)
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.schema_version != 1 {
            return Err(ConfigError::SchemaVersion);
        }
        if self.prover.artifacts_dir.as_os_str().is_empty() {
            return Err(ConfigError::EmptyArtifactsDirectory);
        }
        self.prover.approved_programs.validate()?;
        if self.server.is_some()
            || self.client.is_some()
            || self.rpc.is_some()
            || self.contracts.is_some()
        {
            self.validate_runtime_fields()?;
        }
        Ok(())
    }

    /// Every production startup must use the configured approval, never an unpinned loader.
    pub fn load_proving_service(&self) -> Result<ProvingService, ConfigError> {
        self.validate()?;
        Ok(ProvingService::load_approved(
            &self.prover.artifacts_dir,
            &self.prover.approved_programs,
        )?)
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ServerConfig {
    pub listen: SocketAddr,
    #[serde(default = "default_job_timeout")]
    pub max_job_duration_seconds: u64,
    #[serde(default = "default_shutdown")]
    pub shutdown_timeout_seconds: u64,
}
fn default_job_timeout() -> u64 {
    3600
}
fn default_shutdown() -> u64 {
    30
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ClientConfig {
    pub chain_id: String,
    pub client_id: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RpcEndpoint {
    pub proxy_url: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct RpcConfig {
    pub source: RpcEndpoint,
    pub verifier: RpcEndpoint,
    #[serde(alias = "callTimeout", deserialize_with = "duration_millis")]
    pub call_timeout_ms: u64,
    pub max_attempts: usize,
    #[serde(alias = "retryBackoff", deserialize_with = "duration_millis")]
    pub retry_backoff_ms: u64,
    pub max_response_bytes: usize,
    pub expected_source_chain_id: Option<u64>,
    pub expected_verifier_chain_id: Option<u64>,
}
impl Default for RpcConfig {
    fn default() -> Self {
        Self {
            source: RpcEndpoint {
                proxy_url: String::new(),
            },
            verifier: RpcEndpoint {
                proxy_url: String::new(),
            },
            call_timeout_ms: 3000,
            max_attempts: 2,
            retry_backoff_ms: 100,
            max_response_bytes: 16 * 1024 * 1024,
            expected_source_chain_id: None,
            expected_verifier_chain_id: None,
        }
    }
}
impl RpcConfig {
    pub fn clients(&self) -> Result<(RpcClient, RpcClient), ConfigError> {
        if !(1000..=300_000).contains(&self.call_timeout_ms)
            || self.retry_backoff_ms > 60_000
            || self.max_response_bytes > 64 * 1024 * 1024
        {
            return Err(ConfigError::Runtime("RPC limits out of range"));
        }
        let settings = RpcSettings {
            timeout: Duration::from_millis(self.call_timeout_ms),
            max_attempts: self.max_attempts,
            retry_backoff: Duration::from_millis(self.retry_backoff_ms),
            max_response_bytes: self.max_response_bytes,
        };
        Ok((
            RpcClient::with_settings(&self.source.proxy_url, settings.clone())?,
            RpcClient::with_settings(&self.verifier.proxy_url, settings)?,
        ))
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ContractsConfig {
    pub verifier_router: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields, default)]
pub struct SchedulerConfig {
    pub preferred_batch_size: usize,
    pub max_batch_size: usize,
    #[serde(alias = "maxBatchWait", deserialize_with = "duration_millis")]
    pub max_batch_wait_ms: u64,
    pub max_pending_intents: usize,
    pub max_active_jobs: usize,
    pub allow_parallel_update_and_membership: bool,
    pub update_job_deduplication: bool,
}
impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            preferred_batch_size: 10,
            max_batch_size: 32,
            max_batch_wait_ms: 3000,
            max_pending_intents: 1024,
            max_active_jobs: 1,
            allow_parallel_update_and_membership: true,
            update_job_deduplication: true,
        }
    }
}
impl Config {
    pub fn validate_runtime(&self) -> Result<(), ConfigError> {
        self.validate()?;
        self.validate_runtime_fields()
    }
    fn validate_runtime_fields(&self) -> Result<(), ConfigError> {
        let server = self
            .server
            .as_ref()
            .ok_or(ConfigError::Runtime("server is required"))?;
        if server.max_job_duration_seconds == 0
            || server.max_job_duration_seconds > 86_400
            || server.shutdown_timeout_seconds == 0
            || server.shutdown_timeout_seconds > 300
        {
            return Err(ConfigError::Runtime("server timeouts out of range"));
        }
        let client = self
            .client
            .as_ref()
            .ok_or(ConfigError::Runtime("client is required"))?;
        if client.chain_id.trim().is_empty() || client.client_id.trim().is_empty() {
            return Err(ConfigError::Runtime(
                "client chainId and clientId are required",
            ));
        }
        self.rpc
            .as_ref()
            .ok_or(ConfigError::Runtime("rpc is required"))?
            .clients()?;
        let contracts = self
            .contracts
            .as_ref()
            .ok_or(ConfigError::Runtime("contracts is required"))?;
        let router: [u8; 20] =
            prover_script::rpc::fixed_hex(&contracts.verifier_router, "verifierRouter")?;
        if router == [0; 20] {
            return Err(ConfigError::Runtime("verifierRouter must be nonzero"));
        }
        let scheduler = &self.scheduler;
        if !scheduler.update_job_deduplication
            || scheduler.preferred_batch_size == 0
            || scheduler.preferred_batch_size > scheduler.max_batch_size
            || scheduler.max_batch_size > 32
            || scheduler.max_pending_intents < scheduler.max_batch_size
            || scheduler.max_batch_wait_ms < 2000
            || !(1..=64).contains(&scheduler.max_active_jobs)
        {
            return Err(ConfigError::Runtime("scheduler limits out of range"));
        }
        Ok(())
    }
}

fn duration_millis<'de, D: serde::Deserializer<'de>>(deserializer: D) -> Result<u64, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Value {
        Millis(u64),
        Duration(String),
    }
    match Value::deserialize(deserializer)? {
        Value::Millis(value) => Ok(value),
        Value::Duration(value) => {
            let (digits, factor) = if let Some(value) = value.strip_suffix("ms") {
                (value, 1)
            } else if let Some(value) = value.strip_suffix('s') {
                (value, 1000)
            } else if let Some(value) = value.strip_suffix('m') {
                (value, 60_000)
            } else if let Some(value) = value.strip_suffix('h') {
                (value, 3_600_000)
            } else {
                return Err(serde::de::Error::custom("duration requires ms, s, m, or h"));
            };
            digits
                .parse::<u64>()
                .ok()
                .and_then(|v| v.checked_mul(factor))
                .ok_or_else(|| serde::de::Error::custom("invalid or overflowing duration"))
        }
    }
}
