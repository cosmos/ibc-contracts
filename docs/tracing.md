# Tracing and Distributed Observability

Tracing is a framework for instrumenting Rust programs to collect structured, event-based diagnostic information across distributed systems. It enables developers to understand request flows and failure patterns in microservice architectures.

## Core Concepts

### Tracing Fundamentals

**Tracing** enables you to follow a single request as it flows through multiple services, providing end-to-end visibility into distributed operations.

**Subscribers** handle the collection and processing of trace data. A subscriber should typically be a global static instance, though case-specific subscribers can be used when needed. The most common pattern combines stdout output with OpenTelemetry-compatible subscribers.

**Unique Identification**: Every trace must have a unique identifier - either a TraceID (16-byte OpenTelemetry standard) or CCID (Correlation Context Identifier).

### Spans and Context

- **Span**: The smallest unit in a trace, representing a period of time and work in the program
- **Inject**: Taking the current span context, serializing it, and sending it to downstream calls
- **Extract**: Discovering the span context from incoming requests and creating child spans
- **Context Accumulation**: Context can accumulate across service boundaries - be mindful of payload size

### OpenTelemetry Integration

OpenTelemetry provides three major capabilities:

- **Distributed context propagation** - Traces across service boundaries
- **Application tracing** - Internal operation visibility
- **Application metrics** - Quantitative performance data

#### Span Context Components

- **TraceID**: 16-byte array uniquely identifying the entire trace
- **SpanID**: 8-byte array uniquely identifying this span within the trace
- **TraceFlags**: Metadata about trace sampling and processing
- **IsRemote**: Boolean indicating if context was propagated from a remote parent

### Trace Narrative Structure

Traces help build a relational graph showing the complete story of a request:

```
Span name: What happened
Span tags: Why it happened
Span logs: How it happened
```

## Performance Considerations

**Distributed request tracing** works best when operations complete within a reasonable timeframe (typically minutes). For longer-running operations, consider alternative approaches or sampling strategies.

**Overhead**: If every service emits spans with basic attributes that require no runtime overhead (pre-calculated string values), the total overhead per request is minimal - approximately 25 bytes for context headers and negligible CPU cycles for decoding.

## Instrumentation Checklist

### Essential Requirements

- **Span Lifecycle**: Ensure all created spans are properly finished, even during unrecoverable errors
- **Span Kinds**: Set appropriate `SpanKind` for egress and ingress operations
- **Infrastructure Context**: Include identifying information about the underlying infrastructure:
  - Hostname or application instance ID
  - Application server version
  - Region or availability zone
- **Namespace Attributes**: Use consistent namespacing for attributes
- **Unit Specification**: Attributes with numeric values must include units in the key name
  - ✅ Good: `payload_size_bytes`, `latency_seconds`
  - ❌ Bad: `payload_size`, `latency`
- **Version Tracking**: Version attributes are critical for debugging across deployments

### Debugging Considerations

When implementing tracing, consider:

- **Unknown unknowns**: What information might be needed during incident response?
- **Performance impact**: How will this instrumentation affect production performance?
- **Error context**: What information is needed when things go wrong?

> **Conditional Instrumentation**: In Rust, you can use conditional fields in the `#[instrument]` macro based on error conditions by leveraging the `err` parameter and field visibility controls.

## Rust Implementation Guide

### Essential Dependencies

The following crates provide a complete tracing solution:

- **`tracing`** - Core instrumentation framework for Rust code
- **`tracing-subscriber`** - Event listening, filtering, and export configuration
- **`tracing-log`** - Collects log events from third-party libraries into your subscriber

  ```rust
  LogTracer::init().expect("Failed to set logger");
  ```

- **`opentelemetry`** - OpenTelemetry API-level interfaces for tracing and spans
- **`opentelemetry_sdk`** - OpenTelemetry SDK implementation
- **`tracing-opentelemetry`** - Compatibility layer between tracing and OpenTelemetry
- **`opentelemetry-otlp`** - OTLP protocol implementation for exporting to Jaeger and other backends

### Critical Setup Rules

⚠️ **Important**: Any trace events generated outside an active subscriber context will be lost.

⚠️ **Library Constraint**: Libraries should **never** install a global subscriber using `set_global_default()`. This will cause conflicts when executables attempt to configure their own subscribers.

💡 **Runtime Configuration**: Tracing filters can be changed during runtime using the [reload functionality](https://docs.rs/tracing-subscriber/latest/tracing_subscriber/reload/index.html).

## gRPC Tracing Implementation

### Automatic Trace Context Propagation

For gRPC services, implement automatic trace context extraction using interceptors rather than manual calls in each service method.

#### Server-Side Implementation

```rust
use opentelemetry::trace::TraceContextExt;
use tonic::{metadata::MetadataMap, Request, Status};
use tracing::Span;
use tracing_opentelemetry::OpenTelemetrySpanExt;

/// Automatic gRPC server interceptor for trace context extraction
pub fn tracing_interceptor<T>(request: Request<T>) -> Result<Request<T>, Status> {
    // Extract parent context from gRPC metadata
    let parent_context = opentelemetry::global::get_text_map_propagator(|propagator| {
        propagator.extract(&MetadataExtractor(request.metadata()))
    });

    // Set parent context for current span
    Span::current().set_parent(parent_context);

    // Extract and record trace ID
    let span = Span::current();
    let context = span.context();
    let otel_span = context.span();
    let trace_id = otel_span.span_context().trace_id();
    Span::current().record("trace_id", trace_id.to_string());

    Ok(request)
}
```

#### Server Configuration

```rust
// Apply interceptor automatically to all gRPC methods
Server::builder()
    .add_service(
        ProofApiServiceServer::with_interceptor(proof_api_router, tracing_interceptor)
    )
    .serve(socket_addr)
    .await?;
```

#### Service Method Declaration

```rust
#[instrument(
    skip(self, request),
    fields(
        src_chain = %request.get_ref().src_chain,
        dst_chain = %request.get_ref().dst_chain,
        trace_id = tracing::field::Empty  // Reserved for interceptor population
    )
)]
async fn my_service_method(&self, request: Request<MyRequest>) -> Result<Response<MyResponse>, Status> {
    // No manual trace extraction needed - handled by interceptor
    let inner_request = request.get_ref();
    // ... business logic
}
```

### Benefits of Interceptor Pattern

- **Consistency**: Impossible to forget trace context extraction
- **Clean Code**: Service methods focus on business logic
- **Single Source of Truth**: Centralized trace handling logic
- **Automatic Propagation**: Works for all gRPC methods without manual intervention

### Span Deep Dive

A span is a lightweight handle consisting of an ID and reference to the current subscriber. Conceptually, it serves as a key to the subscriber's internal storage.

#### Span Creation Process

1. **Metadata Generation**: The macro builds static metadata for the span
2. **Subscriber Interaction**: Calls `Subscriber::new_span(&Attributes)`
3. **Layer Processing**: All layers receive `on_new_span` callbacks with the attributes
4. **Handle Return**: A `tracing::Span` handle is returned containing the span ID and cloned dispatch

⚠️ **Important**: A span does not become current until it's explicitly entered.

#### Manual Span Creation

```rust
pub async fn subscribe(/* */) -> HttpResponse {
    let request_id = Uuid::new_v4();
    let request_span = tracing::info_span!(
        "Adding a new subscriber",
        %request_id,
        subscriber_email = %form.email,
        subscriber_name = %form.name
    );
    // ⚠️ Using `enter()` in async functions is dangerous!
    let _guard = request_span.enter();
}
```

#### Span Stack and Threading

- **Stack Behavior**: `span.enter()` pushes the span onto the current span stack
- **Scope**: The stack of entered spans is thread-local (or task-local with `#[instrument]`)
- **Thread Safety**: `tracing::Span` itself is `Send + Sync`, but `current_span()` is thread-local
- **Async Caution**: Manually entering spans in async functions can lead to incorrect context if the future migrates between threads

#### Span Lifecycle

- **Multiple Entry/Exit**: You can enter or exit a span multiple times during its lifetime
- **Final Closure**: Once a span is closed, it cannot be reopened
- **Automatic Management**: Use `#[instrument]` to avoid manual span management in async code

### Subscriber Architecture

The `tracing::Subscriber` trait defines the core interface for trace collection. The `Registry` is the primary implementation provided by `tracing-subscriber`.

**Key Implementations:**

- **Registry**: Standard subscriber for collecting and organizing spans
- **tokio-console**: Specialized subscriber for debugging and profiling async applications (requires `tokio-unstable` feature)

### Registry Responsibilities

The Registry acts as the foundation layer that:

- **Metadata Storage**: Maintains span metadata and attributes
- **Relationship Tracking**: Records parent-child relationships between spans
- **Lifecycle Management**: Tracks which spans are active, entered, and closed
- **Delegation**: Routes events to appropriate layers for processing

**Note**: The Registry itself does not record traces - it coordinates with layers that handle the actual recording and export.

## Best Practices

### Code Instrumentation

**Always include line numbers** in trace output for easier debugging:

```rust
fmt::layer()
    .with_line_number(true)
    .with_file(true)
```

**Never leak credentials** in trace data:

```rust
#[instrument(skip(self, credential), fields(username = %credential.username))]
async fn authenticate(&self, credential: &Credentials) -> Result<User, AuthError> {
    // credential details are skipped, only username is recorded
}
```

**Use `secrecy::Secret`** to automatically mask sensitive data:

```rust
use secrecy::Secret;
let password: Secret<String> = Secret::new("sensitive_data".to_string());
```

### Library Guidelines

Libraries should:

- **Only use the `tracing` crate** for instrumentation
- **Provide useful information** to downstream consumers through structured spans
- **Never install global subscribers** - leave that to the application binary

Reference: [Tracing Library Guidelines](https://github.com/tokio-rs/tracing)

### Advanced Patterns

**Deferred Field Population** using `tracing::field::Empty`:

This is helpful for context propagation.

```rust
#[instrument(fields(
    trace_id = tracing::field::Empty,
    result = tracing::field::Empty
))]
async fn process_request() -> Result<String, Error> {
    // Fields get populated later by interceptors or during execution
    let result = do_work().await?;
    Span::current().record("result", &result);
    Ok(result)
}
```

**Rich Error Context** with `tracing-error` and `eyre`:

```rust
tracing_subscriber::registry()
    .with(filter_layer)
    .with(fmt_layer)
    .with(tracing_error::ErrorSubscriber::default())
    .init();

color_eyre::install()?;
```

**Human-Readable Development Traces**:

```rust
fmt::layer()
    .pretty()
    .with_target(true)
    .with_line_number(true)
```

## Anti-patterns

### Common Mistakes to Avoid

#### ❌ Over-logging instead of spanning

- When in doubt, create a new span rather than additional log statements
- Spans provide structure and timing information that logs cannot

#### ❌ Including `self` in instrumentation unnecessarily

- Skip `self` from instrumentation unless the instance provides meaningful context
- Use `skip(self)` to avoid noise in traces

#### ❌ Timestamp overhead in development

- Disable timestamp generation in non-production environments for uncluttered logs
- Enable only when needed for debugging

#### ❌ Double error handling

- Automatic error logging can violate the "handle errors once" principle:

```rust
// ❌ This automatically logs errors, potentially duplicating error handling
#[instrument(level = "trace", err(Debug/Display))]
async fn risky_operation() -> Result<(), Error> {
    // If this fails, it gets logged automatically AND returned
}
```

### Development Tools Consideration

**Enhanced Error Reporting**: Use [`color-eyre`](https://docs.rs/color-eyre/0.6.5/color_eyre/) for rich error context in development.
Eyre is a fork of `anyhow` and `color-eyre` gives nice looking error messages.

## Implementation Examples

### Basic Subscriber Setup

```rust
pub fn get_subscriber(
    name: String,
    env_filter: String,
    sink: Sink,
) -> impl Subscriber + Send + Sync
where
    Sink: for<'a> MakeWriter<'a> + Send + Sync + 'static,
{
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(env_filter));
    let formatting_layer = BunyanFormattingLayer::new(name, sink);
    Registry::default()
        .with(env_filter)
        .with(JsonStorageLayer)
        .with(formatting_layer)
}

pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    LogTracer::init().expect("Failed to set logger");
    set_global_default(subscriber).expect("Failed to set subscriber");
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let subscriber = get_subscriber("my_service".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);
    // Application logic here
}
```

### Test Environment Setup

```rust
use once_cell::sync::Lazy;

// Ensure tracing is initialized only once across tests
static TRACING: Lazy<()> = Lazy::new(|| {
    let default_filter_level = "info".to_string();
    let subscriber_name = "test".to_string();

    let sink = if std::env::var("MUTE_TEST_LOG").is_ok() {
        std::io::sink()
    } else {
        std::io::stdout()
    };

    let subscriber = get_subscriber(subscriber_name, default_filter_level, sink);
    init_subscriber(subscriber);
});

pub fn init_test_tracing() {
    Lazy::force(&TRACING);
}

#[tokio::test]
async fn my_test() {
    init_test_tracing();
    // Test logic with tracing enabled
}
```

## Reference Implementation

For a complete example of production-ready tracing setup, see:

- `programs/proof-api/src/observability.rs` - Production subscriber configuration (traces + OTLP logs)
- `packages/proof-api/lib/src/utils/tracing_layer.rs` - gRPC trace propagation
- `packages/proof-api/core/src/builder.rs` - Automatic interceptor integration

### OTEL logs export

When `use_otel` is true in the proof-api config, logs are exported via OTLP gRPC to the same `otel_endpoint` as traces. The proof-api bridges `tracing` events to OpenTelemetry logs using an appender layer, while retaining pretty-printed console output.

Config snippet:

```json
{
  "observability": {
    "level": "info",
    "use_otel": true,
    "service_name": "proof-api",
    "otel_endpoint": "http://localhost:4317"
  }
}
```

## Local observability for e2e (Grafana + Tempo + Prometheus)

This project includes a local Grafana observability stack you can use during e2e runs.

### Start the local stack

```bash
cd scripts/local-grafana-stack
docker compose up -d
```

- Grafana: http://localhost:3002 (anonymous access enabled)
- Tempo (traces backend): internal service at `tempo:3200` (Grafana datasource)
- Prometheus: http://localhost:9090
- Loki (logs backend): http://localhost:3100 (Grafana datasource)
- Alloy (collector): OTLP gRPC on `0.0.0.0:4317`. Port `4318` is published, but `config.alloy` defines no OTLP HTTP receiver

### Enable proof-api tracing to local stack in e2e

Set the environment variable before running the e2e tests:

```bash
export ENABLE_LOCAL_OBSERVABILITY=true
```

Behavior when enabled:
- Observability config in the generated proof-api `config.json` will be set to:

```json
{
  "observability": {
    "level": "<from RUST_LOG or 'info' if unset>",
    "use_otel": true,
    "service_name": "proof-api",
    "otel_endpoint": "http://127.0.0.1:4317"
  }
}
```

- The proof-api respects `RUST_LOG` for log level; set it if you want a different level:

```bash
export RUST_LOG=debug
```

- Prometheus metrics are served at `http://0.0.0.0:9000/metrics` by the proof-api and scraped by Prometheus if you configure it. The local stack already includes Prometheus; you can add a scrape config there if desired.

### Validate traces in Grafana

1. Open Grafana at http://localhost:3002
2. Go to the Tempo datasource and run a trace search for recent activity.
3. Generate e2e traffic (run tests). You should see spans with `service.name = proof-api`.

### Notes

- e2e proof-api runs on the host, so `http://127.0.0.1:4317` reaches the Alloy collector in Docker.
- The OTLP transport is gRPC on 4317 as configured in `scripts/local-grafana-stack/config.alloy`.
- If you need HTTP instead, add an `http` endpoint to the OTLP receiver in `config.alloy`, switch the endpoint to `http://127.0.0.1:4318`, and ensure the proof-api exporter supports HTTP (it is built with the gRPC transport only).
