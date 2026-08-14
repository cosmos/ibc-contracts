// SPDX-License-Identifier: Apache-2.0

//! Defines the [`ProofApiModule`] trait that must be implemented by all proof API modules.

use crate::api::proof_api_service_server::ProofApiService;
use std::marker::{Send, Sync};

/// The `ProofApiModule` trait defines the interface for interacting with a proof API module.
#[tonic::async_trait]
pub trait ProofApiModule: Send + Sync + 'static {
    /// Returns the name of the proof API module.
    fn name(&self) -> &'static str;

    /// Creates a proof API service of the given module type with the provided config.
    async fn create_service(
        &self,
        config: serde_json::Value,
    ) -> anyhow::Result<Box<dyn ProofApiService>>;
}
