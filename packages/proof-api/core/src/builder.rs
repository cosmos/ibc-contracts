// SPDX-License-Identifier: Apache-2.0

//! Defines the [`ProofApiBuilder`] struct that is used to build the proof API server.

use std::collections::HashMap;

use super::modules::ProofApiModule;
use crate::{
    api::{
        self,
        proof_api_service_server::{ProofApiService, ProofApiServiceServer},
    },
    config::ProofApiConfig,
};
use proof_api_lib::utils::tracing_layer::tracing_interceptor;
use tonic::{transport::Server, Request, Response};
use tracing::{error, info, instrument};

/// The `ProofApiBuilder` struct is used to build the proof API server.
#[derive(Default)]
#[allow(clippy::module_name_repetitions)]
pub struct ProofApiBuilder {
    /// The proof API modules that can create services from configuration.
    modules: HashMap<String, Box<dyn ProofApiModule>>,
}

/// The `ProofApiRouter` routes requests to the service configured for each chain pair.
#[derive(Default)]
struct ProofApiRouter {
    /// Mapping of (`src_chain`, `dst_chain`) to the proof API service.
    services: HashMap<(String, String), Box<dyn ProofApiService>>,
}

impl ProofApiBuilder {
    /// Create a new `ProofApiBuilder` instance.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a proof API module to the proof API binary.
    /// # Panics
    /// Panics if the module has already been added.
    #[allow(clippy::missing_errors_doc)]
    #[instrument(skip(self, module), fields(module_name = %module.name()))]
    pub fn add_module<T: ProofApiModule>(&mut self, module: T) {
        assert!(
            !self.modules.contains_key(module.name()),
            "Proof API module already added"
        );

        self.modules
            .insert(module.name().to_string(), Box::new(module));
    }

    /// Start the proof API server.
    /// # Errors
    /// Returns an error if the server fails to start.
    #[instrument(skip(self, config), name = "proof_api_start", err(Debug))]
    pub async fn start(&self, config: ProofApiConfig) -> anyhow::Result<()> {
        let socket_addr = format!("{}:{}", config.server.address, config.server.port);
        info!(%socket_addr, "Starting proof API server...");
        let socket_addr = socket_addr.parse::<std::net::SocketAddr>()?;

        let reflection_service = tonic_reflection::server::Builder::configure()
            .register_encoded_file_descriptor_set(api::FILE_DESCRIPTOR_SET)
            .build_v1()?;

        let mut proof_api_router = ProofApiRouter::default();

        for c in config.modules.into_iter().filter(|c| c.enabled) {
            let module = self.modules.get(&c.name).map(|v| &**v).ok_or_else(|| {
                anyhow::anyhow!("Module {} not found in proof API builder", c.name)
            })?;

            proof_api_router.add_module(
                c.src_chain,
                c.dst_chain,
                module.create_service(c.config).await?,
            );
            info!(module_name = %c.name, "Service added successfully");
        }

        // Start the gRPC server
        info!(%socket_addr, "Starting gRPC server");
        Server::builder()
            .add_service(ProofApiServiceServer::with_interceptor(
                proof_api_router,
                tracing_interceptor,
            ))
            .add_service(reflection_service)
            .serve(socket_addr)
            .await?;

        info!("Proof API server stopped");
        Ok(())
    }
}

impl ProofApiRouter {
    #[allow(clippy::result_large_err)]
    fn get_module(
        &self,
        src_chain: &str,
        dst_chain: &str,
    ) -> Result<&dyn ProofApiService, tonic::Status> {
        self.services
            .get(&(src_chain.to_string(), dst_chain.to_string()))
            .map(|v| &**v)
            .ok_or_else(|| {
                tonic::Status::not_found(format!(
                    "Module not found for src_chain: {src_chain}, dst_chain: {dst_chain}",
                ))
            })
    }

    fn add_module(
        &mut self,
        src_chain: String,
        dst_chain: String,
        module: Box<dyn ProofApiService>,
    ) {
        self.services.insert((src_chain, dst_chain), module);
    }
}

#[tonic::async_trait]
impl ProofApiService for ProofApiRouter {
    #[instrument(
        skip(self, request),
        fields(
            src_chain = %request.get_ref().src_chain,
            dst_chain = %request.get_ref().dst_chain,
            trace_id = tracing::field::Empty
        )
    )]
    async fn info(
        &self,
        request: Request<api::InfoRequest>,
    ) -> Result<Response<api::InfoResponse>, tonic::Status> {
        let inner_request = request.get_ref();
        let src_chain = inner_request.src_chain.clone();
        let dst_chain = inner_request.dst_chain.clone();

        crate::metrics::track_metrics("info", &src_chain, &dst_chain, || async move {
            let inner_request = request.get_ref();
            self.get_module(&inner_request.src_chain, &inner_request.dst_chain)?
                .info(request)
                .await
                .map_err(|e| {
                    error!(error = %e, "Info request failed");
                    tonic::Status::internal("Failed to get info. See logs for more details.")
                })
        })
        .await
    }

    #[instrument(
        skip(self, request),
        fields(
            src_chain = %request.get_ref().src_chain,
            dst_chain = %request.get_ref().dst_chain,
            src_client_id = %request.get_ref().src_client_id,
            trace_id = tracing::field::Empty,
        )
    )]
    async fn relay_by_tx(
        &self,
        request: Request<api::RelayByTxRequest>,
    ) -> Result<Response<api::RelayByTxResponse>, tonic::Status> {
        let inner_request = request.get_ref();
        let src_chain = inner_request.src_chain.clone();
        let dst_chain = inner_request.dst_chain.clone();

        crate::metrics::track_metrics("relay_by_tx", &src_chain, &dst_chain, || async move {
            let inner_request = request.get_ref();
            self.get_module(&inner_request.src_chain, &inner_request.dst_chain)?
                .relay_by_tx(request)
                .await
                .map_err(|e| {
                    error!(error = %e, "Relay by tx request failed");
                    tonic::Status::internal("Failed to relay by tx. See logs for more details.")
                })
        })
        .await
    }

    #[instrument(
        skip(self, request),
        fields(
            src_chain = %request.get_ref().src_chain,
            dst_chain = %request.get_ref().dst_chain,
            trace_id = tracing::field::Empty
        )
    )]
    async fn create_client(
        &self,
        request: Request<api::CreateClientRequest>,
    ) -> Result<Response<api::CreateClientResponse>, tonic::Status> {
        let inner_request = request.get_ref();
        let src_chain = inner_request.src_chain.clone();
        let dst_chain = inner_request.dst_chain.clone();

        crate::metrics::track_metrics("create_client", &src_chain, &dst_chain, || async move {
            let inner_request = request.get_ref();
            self.get_module(&inner_request.src_chain, &inner_request.dst_chain)?
                .create_client(request)
                .await
                .map_err(|e| {
                    error!(error = %e, "Create client request failed");
                    tonic::Status::internal("Failed to create client. See logs for more details.")
                })
        })
        .await
    }

    #[instrument(
        skip(self, request),
        fields(
            src_chain = %request.get_ref().src_chain,
            dst_chain = %request.get_ref().dst_chain,
            trace_id = tracing::field::Empty
        )
    )]
    async fn update_client(
        &self,
        request: Request<api::UpdateClientRequest>,
    ) -> Result<Response<api::UpdateClientResponse>, tonic::Status> {
        let inner_request = request.get_ref();
        let src_chain = inner_request.src_chain.clone();
        let dst_chain = inner_request.dst_chain.clone();

        crate::metrics::track_metrics("update_client", &src_chain, &dst_chain, || async move {
            let inner_request = request.get_ref();
            self.get_module(&inner_request.src_chain, &inner_request.dst_chain)?
                .update_client(request)
                .await
                .map_err(|e| {
                    error!(error = %e, "Update client request failed");
                    tonic::Status::internal("Failed to update client. See logs for more details.")
                })
        })
        .await
    }
}
