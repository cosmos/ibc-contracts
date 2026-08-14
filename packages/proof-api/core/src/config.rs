// SPDX-License-Identifier: Apache-2.0

//! Defines the top level configuration for the proof API.

use std::str::FromStr;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use tracing::Level;

/// The top level configuration for the proof API.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ProofApiConfig {
    /// The configuration for the proof API modules.
    pub modules: Vec<ModuleConfig>,
    /// The configuration for the proof API server.
    pub server: ServerConfig,
    /// The configuration for observability.
    #[serde(default)]
    pub observability: ObservabilityConfig,
}

/// The configuration for the proof API modules.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ModuleConfig {
    /// The name of the module.
    pub name: String,
    /// The source chain identifier for the module.
    /// Used to route requests to the correct module.
    pub src_chain: String,
    /// The destination chain identifier for the module.
    /// Used to route requests to the correct module.
    pub dst_chain: String,
    /// The custom configuration for the module.
    pub config: Value,
    /// Whether the module is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,
}

/// The configuration for the proof API server.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ServerConfig {
    /// The address to bind the server to.
    pub address: String,
    /// The port to bind the server to.
    pub port: u16,
}

/// Returns true, used as a default value for boolean fields.
const fn default_true() -> bool {
    true
}

/// Configuration for observability.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default)]
pub struct ObservabilityConfig {
    /// The log level to use.
    pub level: String,
    /// Whether to use OpenTelemetry for distributed tracing.
    pub use_otel: bool,
    /// The service name to use for OpenTelemetry.
    pub service_name: String,
    /// The OpenTelemetry collector endpoint.
    pub otel_endpoint: Option<String>,
}

impl Default for ObservabilityConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            use_otel: false,
            service_name: "proof-api".to_string(),
            otel_endpoint: None,
        }
    }
}

impl ObservabilityConfig {
    /// Returns the log level as a `tracing::Level`.
    #[must_use]
    pub fn level(&self) -> Level {
        Level::from_str(&self.level).unwrap_or(Level::INFO)
    }
}
