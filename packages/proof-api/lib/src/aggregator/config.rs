// SPDX-License-Identifier: Apache-2.0

use anyhow::{Context, Result};
use std::{collections::HashSet, fs, path::Path};

/// Aggregator config
#[derive(Debug, serde::Serialize, serde::Deserialize, Clone)]
pub struct Config {
    /// The configuration for the attestor signer.
    pub attestor: AttestorConfig,
    /// The configuration for caching behavior.
    #[serde(default)]
    pub cache: CacheConfig,
}

impl Config {
    /// Reads config from a file
    ///
    /// # Errors
    /// - Fails if file does not exist
    /// - Fails if the config is invalid
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file '{}'", path.display()))?;

        let config: Self =
            serde_json::from_str(&content).context("Failed to parse JSON configuration")?;

        config.validate()?;

        Ok(config)
    }

    /// Validates the parsed config
    ///
    /// # Errors
    /// - Fails if attestaor config is invalid
    /// - Fails if cache config is invalid
    pub fn validate(&self) -> Result<()> {
        self.attestor.validate()?;
        self.cache.validate()?;
        Ok(())
    }
}

/// Attestor config
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct AttestorConfig {
    /// Timeout
    pub attestor_query_timeout_ms: u64,
    /// Quorum
    pub quorum_threshold: usize,
    /// Endpoints
    pub attestor_endpoints: Vec<String>,
}

impl AttestorConfig {
    fn validate(&self) -> Result<()> {
        self.validate_endpoints()?;
        self.validate_quorum_threshold()?;
        self.validate_timeout()?;
        Ok(())
    }

    fn validate_endpoints(&self) -> Result<()> {
        anyhow::ensure!(
            !self.attestor_endpoints.is_empty(),
            "At least one attestor endpoint must be specified"
        );

        self.attestor_endpoints
            .iter()
            .enumerate()
            .try_for_each(|(index, endpoint)| Self::validate_single_endpoint(endpoint, index))?;

        let unique_count = self.attestor_endpoints.iter().collect::<HashSet<_>>().len();

        anyhow::ensure!(
            unique_count == self.attestor_endpoints.len(),
            "Duplicate endpoints found: {} unique out of {} total",
            unique_count,
            self.attestor_endpoints.len()
        );

        Ok(())
    }

    fn validate_single_endpoint(endpoint: &str, index: usize) -> Result<()> {
        let trimmed_endpoint = endpoint.trim();

        anyhow::ensure!(
            !trimmed_endpoint.is_empty(),
            "Endpoint at index {index} cannot be empty or whitespace-only",
        );

        anyhow::ensure!(
            trimmed_endpoint.starts_with("http://") || trimmed_endpoint.starts_with("https://"),
            "Endpoint at index {index} must start with 'http://' or 'https://': '{trimmed_endpoint}'",
        );

        Ok(())
    }

    fn validate_quorum_threshold(&self) -> Result<()> {
        let endpoint_count = self.attestor_endpoints.len();

        anyhow::ensure!(
            self.quorum_threshold >= defaults::MIN_QUORUM_THRESHOLD,
            "Quorum threshold must be at least {}, got {}",
            defaults::MIN_QUORUM_THRESHOLD,
            self.quorum_threshold
        );

        anyhow::ensure!(
            self.quorum_threshold <= endpoint_count,
            "Quorum threshold ({}) cannot exceed number of endpoints ({})",
            self.quorum_threshold,
            endpoint_count
        );

        Ok(())
    }

    fn validate_timeout(&self) -> Result<()> {
        let timeout_range = defaults::MIN_TIMEOUT_MS..=defaults::MAX_TIMEOUT_MS;

        anyhow::ensure!(
            timeout_range.contains(&self.attestor_query_timeout_ms),
            "Query timeout must be between {}ms and {}ms, got {}ms",
            defaults::MIN_TIMEOUT_MS,
            defaults::MAX_TIMEOUT_MS,
            self.attestor_query_timeout_ms
        );

        Ok(())
    }
}

#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
/// Aggregator cache config
pub struct CacheConfig {
    #[serde(default = "defaults::default_state_cache_max_entries")]
    pub(crate) state_cache_max_entries: u64,
    #[serde(default = "defaults::default_packet_cache_max_entries")]
    pub(crate) packet_cache_max_entries: u64,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            state_cache_max_entries: defaults::DEFAULT_STATE_CACHE_MAX_ENTRIES,
            packet_cache_max_entries: defaults::DEFAULT_PACKET_CACHE_MAX_ENTRIES,
        }
    }
}

impl CacheConfig {
    fn validate(&self) -> Result<()> {
        anyhow::ensure!(
            self.state_cache_max_entries <= defaults::MAX_CACHE_ENTRIES,
            "State cache max entries must be at most {}, got {}",
            defaults::MAX_CACHE_ENTRIES,
            self.state_cache_max_entries
        );

        anyhow::ensure!(
            self.packet_cache_max_entries <= defaults::MAX_CACHE_ENTRIES,
            "Packet cache max entries must be at most {}, got {}",
            defaults::MAX_CACHE_ENTRIES,
            self.packet_cache_max_entries
        );

        Ok(())
    }
}

/// Default values for configuration
mod defaults {

    pub const MIN_TIMEOUT_MS: u64 = 10;
    pub const MAX_TIMEOUT_MS: u64 = 60_000;
    pub const MIN_QUORUM_THRESHOLD: usize = 1;

    pub const DEFAULT_STATE_CACHE_MAX_ENTRIES: u64 = 100_000;
    pub const DEFAULT_PACKET_CACHE_MAX_ENTRIES: u64 = 100_000;
    pub const MAX_CACHE_ENTRIES: u64 = 100_000_000;

    pub const fn default_state_cache_max_entries() -> u64 {
        DEFAULT_STATE_CACHE_MAX_ENTRIES
    }

    pub const fn default_packet_cache_max_entries() -> u64 {
        DEFAULT_PACKET_CACHE_MAX_ENTRIES
    }
}
