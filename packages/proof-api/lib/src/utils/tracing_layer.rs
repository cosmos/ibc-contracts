// SPDX-License-Identifier: Apache-2.0

//! Tonic interceptor for automatic trace context extraction and trace ID recording.

use opentelemetry::trace::TraceContextExt;
use tonic::{metadata::MetadataMap, Request, Status};
use tracing::Span;
use tracing_opentelemetry::OpenTelemetrySpanExt;

/// OpenTelemetry injector for gRPC metadata.
pub struct MetadataInjector<'a>(pub &'a mut MetadataMap);

impl opentelemetry::propagation::Injector for MetadataInjector<'_> {
    fn set(&mut self, key: &str, value: String) {
        match tonic::metadata::MetadataKey::from_bytes(key.as_bytes()) {
            Ok(key) => match tonic::metadata::MetadataValue::try_from(&value) {
                Ok(value) => {
                    self.0.insert(key, value);
                }
                Err(error) => tracing::debug!(value, error = %error, "parse metadata value"),
            },
            Err(error) => tracing::debug!(key, error = %error, "parse metadata key"),
        }
    }
}

/// OpenTelemetry extractor for gRPC metadata.
pub struct MetadataExtractor<'a>(pub &'a MetadataMap);

impl opentelemetry::propagation::Extractor for MetadataExtractor<'_> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key)?.to_str().ok()
    }

    fn keys(&self) -> Vec<&str> {
        self.0
            .keys()
            .map(|k| match k {
                tonic::metadata::KeyRef::Ascii(key) => key.as_str(),
                tonic::metadata::KeyRef::Binary(key) => key.as_str(),
            })
            .collect()
    }
}

/// Tonic interceptor function that automatically extracts trace context and records trace ID.
/// This can be used with `.interceptor()` on tonic services.
///
/// # Errors
/// This function never returns an error, but uses `Result` to match tonic's interceptor signature.
#[allow(clippy::result_large_err)]
pub fn tracing_interceptor<T>(request: Request<T>) -> Result<Request<T>, Status> {
    let parent_context = opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.extract(&MetadataExtractor(request.metadata()))
    });

    #[allow(clippy::let_unit_value, clippy::ignored_unit_patterns)]
    let _ = Span::current().set_parent(parent_context);

    // Extract and record the trace ID
    let span = Span::current();
    let context = span.context();
    let otel_span = context.span();
    let trace_id = otel_span.span_context().trace_id();
    span.record("trace_id", trace_id.to_string());

    Ok(request)
}
