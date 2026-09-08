// SPDX-License-Identifier: Apache-2.0

//! Observability configuration for the proof API.
//!
//! Stdout logging + optional OpenTelemetry OTLP export.

use anyhow::{Context, Result};
use opentelemetry::{global, trace::TracerProvider as _, KeyValue};
use opentelemetry_appender_tracing::layer::OpenTelemetryTracingBridge;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::logs::SdkLoggerProvider;
use opentelemetry_sdk::{
    propagation::TraceContextPropagator,
    resource::Resource,
    trace::{Sampler, SdkTracerProvider, SpanExporter, Tracer},
};
use proof_api_core::config::ObservabilityConfig;
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter, Registry};

/// Guard that shuts down OpenTelemetry on drop (if enabled).
pub struct ObservabilityGuard {
    otel_tracer_provider: Option<SdkTracerProvider>,
    otel_logger_provider: Option<SdkLoggerProvider>,
}

impl Drop for ObservabilityGuard {
    fn drop(&mut self) {
        if let Some(provider) = self.otel_tracer_provider.take() {
            let _ = provider.shutdown();
        }
        if let Some(provider) = self.otel_logger_provider.take() {
            // There is no shutdown on SdkLoggerProvider in 0.30; drop flushes processors.
            drop(provider);
        }
    }
}

/// Initialize the global tracing subscriber.
///
/// Keep the returned guard alive for the program's lifetime to ensure a
/// clean OpenTelemetry shutdown.
#[allow(clippy::missing_errors_doc)]
pub fn init_observability(config: &ObservabilityConfig) -> Result<ObservabilityGuard> {
    // Set up global propagator for context propagation
    global::set_text_map_propagator(TraceContextPropagator::new());

    // NOTE: Do not call `LogTracer::init()` here.
    // `SubscriberInitExt::try_init()` will install the log bridge when the
    // `tracing-subscriber` crate is built with the `tracing-log` feature.
    // Calling it explicitly can cause a double-initialization error.

    // Using a closure instead of a function allows Rust to infer the correct
    // return type for each usage, avoiding type system constraints that would
    // arise from specifying a concrete return type like `fmt::Layer<Registry>`.
    let create_fmt_layer = || {
        fmt::layer()
            .pretty()
            .with_target(true)
            .with_line_number(true)
            .with_file(true)
    };

    let (otel_tracer_provider, otel_logger_provider) = if config.use_otel {
        match (setup_otlp_tracer(config), setup_otlp_logger(config)) {
            (Ok((tracer, tracer_provider)), Ok(logger_provider)) => {
                let subscriber = Registry::default()
                    .with(EnvFilter::new(config.level().as_str().to_lowercase()))
                    .with(create_fmt_layer())
                    .with(tracing_opentelemetry::layer().with_tracer(tracer))
                    .with(OpenTelemetryTracingBridge::new(&logger_provider));

                try_init_subscriber(subscriber)?;
                (Some(tracer_provider), Some(logger_provider))
            }
            (Err(e), _) | (_, Err(e)) => {
                eprintln!("OpenTelemetry disabled: {e}");
                (None, None)
            }
        }
    } else {
        // Create a fmt subscriber without OpenTelemetry
        let subscriber = Registry::default()
            .with(EnvFilter::new(config.level().as_str().to_lowercase()))
            .with(create_fmt_layer());

        try_init_subscriber(subscriber)?;
        (None, None)
    };

    Ok(ObservabilityGuard {
        otel_tracer_provider,
        otel_logger_provider,
    })
}

/// Initialize the subscriber and handle errors.
fn try_init_subscriber(subscriber: impl SubscriberInitExt) -> Result<()> {
    subscriber
        .try_init()
        .context("Failed to set global default subscriber")
}

/// Build an OTLP tracer and a gRPC provider
fn setup_otlp_tracer(config: &ObservabilityConfig) -> Result<(Tracer, SdkTracerProvider)> {
    let resource = Resource::builder()
        .with_attributes(vec![
            KeyValue::new("service.name", config.service_name.clone()),
            KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
        ])
        .build();

    let exporter = build_otlp_grpc_exporter(config)?;

    let provider = SdkTracerProvider::builder()
        .with_resource(resource)
        .with_sampler(Sampler::AlwaysOn)
        .with_batch_exporter(exporter)
        .build();

    let tracer = provider.tracer(config.service_name.clone());

    Ok((tracer, provider))
}

fn build_otlp_grpc_exporter(config: &ObservabilityConfig) -> Result<impl SpanExporter> {
    let mut exporter_builder = opentelemetry_otlp::SpanExporter::builder().with_tonic();

    if let Some(endpoint) = &config.otel_endpoint {
        exporter_builder = exporter_builder.with_endpoint(endpoint);
    }

    Ok(exporter_builder.build()?)
}

/// Build an OTLP logger provider for exporting logs over gRPC
fn setup_otlp_logger(config: &ObservabilityConfig) -> Result<SdkLoggerProvider> {
    let resource = Resource::builder()
        .with_attributes(vec![
            KeyValue::new("service.name", config.service_name.clone()),
            KeyValue::new("service.version", env!("CARGO_PKG_VERSION")),
        ])
        .build();

    // Build OTLP logs exporter via tonic
    let mut exporter_builder = opentelemetry_otlp::LogExporter::builder().with_tonic();
    if let Some(endpoint) = &config.otel_endpoint {
        exporter_builder = exporter_builder.with_endpoint(endpoint);
    }
    let exporter = exporter_builder.build()?;

    let provider = SdkLoggerProvider::builder()
        .with_resource(resource)
        .with_batch_exporter(exporter)
        .build();

    Ok(provider)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::sync::Once;
    static INIT: Once = Once::new();

    #[tokio::test]
    async fn init_without_otel_succeeds() {
        let config = ObservabilityConfig {
            level: "info".to_string(),
            use_otel: false,
            service_name: "test-proof-api".to_string(),
            otel_endpoint: None,
        };

        // Initialize at most once to avoid global subscriber conflicts across tests
        INIT.call_once(|| {
            let _ = init_observability(&config);
        });

        // Ensure tracing macro does not panic
        tracing::info!("hello from tracing crate");
    }

    #[tokio::test]
    async fn setup_otlp_logger_builds_with_endpoint() {
        let config = ObservabilityConfig {
            level: "info".to_string(),
            use_otel: true,
            service_name: "test-proof-api".to_string(),
            otel_endpoint: Some("http://127.0.0.1:4317".to_string()),
        };

        let provider = setup_otlp_logger(&config);
        assert!(provider.is_ok());
    }

    #[tokio::test]
    async fn setup_otlp_tracer_builds_with_endpoint() {
        let config = ObservabilityConfig {
            level: "info".to_string(),
            use_otel: true,
            service_name: "test-proof-api".to_string(),
            otel_endpoint: Some("http://127.0.0.1:4317".to_string()),
        };

        let tracer_provider = setup_otlp_tracer(&config);
        assert!(tracer_provider.is_ok());
    }

    #[tokio::test]
    async fn init_with_otel_no_endpoint_succeeds() {
        let config = ObservabilityConfig {
            level: "debug".to_string(),
            use_otel: true,
            service_name: "test-proof-api".to_string(),
            otel_endpoint: None,
        };

        // Initialize at most once
        INIT.call_once(|| {
            let _ = init_observability(&config);
        });
    }
}
