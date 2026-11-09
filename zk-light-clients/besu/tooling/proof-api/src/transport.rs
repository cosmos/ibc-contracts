use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Duration;
use tokio::time::{timeout_at, Instant};

use axum::{
    body::{to_bytes, Body},
    extract::{Request, State},
    http::{header::CONTENT_TYPE, Method, StatusCode},
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use prost::Message;
use serde::{de::DeserializeOwned, Serialize};
use tonic::{Code, Status};
use tower::{Layer, ServiceExt};

use crate::{
    proto::{
        self,
        prover_service_server::{ProverService, ProverServiceServer},
    },
    service::{CallContext, Engine, Service},
};

pub const MAX_MESSAGE_BYTES: usize = 16 * 1024 * 1024;
const PREFIX: &str = "/ibc.v2.prover.ProverService/";

struct Transport<E> {
    service: Service<E>,
    ready: Arc<AtomicBool>,
    metrics: Arc<crate::observability::Metrics>,
}
impl<E> Clone for Transport<E> {
    fn clone(&self) -> Self {
        Self {
            service: self.service.clone(),
            ready: self.ready.clone(),
            metrics: self.metrics.clone(),
        }
    }
}

/// HTTP/1 Connect unary and HTTP/2 gRPC share the same engine and listener.
pub fn router<E: Engine>(service: Service<E>, ready: Arc<AtomicBool>) -> Router {
    let metrics = service.metrics.clone();
    Router::new()
        .route(
            "/metrics",
            get(|State(state): State<Transport<E>>| async move {
                (
                    [(CONTENT_TYPE, "text/plain; version=0.0.4")],
                    state.metrics.render(),
                )
            }),
        )
        .route("/healthz", get(|| async { "ok\n" }))
        .route(
            "/readyz",
            get(|State(state): State<Transport<E>>| async move {
                if state.ready.load(Ordering::Acquire) {
                    (StatusCode::OK, "ready\n")
                } else {
                    (StatusCode::SERVICE_UNAVAILABLE, "not ready\n")
                }
            }),
        )
        .fallback(dispatch::<E>)
        .with_state(Transport {
            service,
            ready,
            metrics,
        })
}

async fn dispatch<E: Engine>(State(state): State<Transport<E>>, mut request: Request) -> Response {
    let content_type = request
        .headers()
        .get(CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    let codec = content_type.split(';').next().unwrap_or("").trim();
    if matches!(
        codec,
        "application/grpc"
            | "application/grpc+proto"
            | "application/grpc-web"
            | "application/grpc-web+proto"
            | "application/grpc-web-text"
            | "application/grpc-web-text+proto"
    ) {
        let mut metadata_request = tonic::Request::new(());
        *metadata_request.metadata_mut() =
            tonic::metadata::MetadataMap::from_headers(request.headers().clone());
        let context = match crate::service::context(&metadata_request) {
            Ok(context) => context,
            Err(status) => return status.into_http().map(Body::new),
        };
        request.extensions_mut().insert(context);
        let server = ProverServiceServer::new(state.service)
            .accept_compressed(tonic::codec::CompressionEncoding::Gzip)
            .send_compressed(tonic::codec::CompressionEncoding::Gzip)
            .max_decoding_message_size(MAX_MESSAGE_BYTES)
            .max_encoding_message_size(MAX_MESSAGE_BYTES);
        let server = tonic_web::GrpcWebLayer::new().layer(server);
        return match timeout_at(
            context.deadline,
            server.oneshot(request.map(tonic::body::boxed)),
        )
        .await
        {
            Ok(Ok(response)) => response.map(Body::new),
            Ok(Err(never)) => match never {},
            Err(_) => Status::deadline_exceeded("deadline exceeded")
                .into_http()
                .map(Body::new),
        };
    }
    let context = match connect_context(request.headers()) {
        Ok(context) => context,
        Err(status) => return connect_error(&status),
    };
    request.extensions_mut().insert(context);
    match timeout_at(context.deadline, connect(state.service, request)).await {
        Ok(Ok(response)) => response,
        Ok(Err(status)) => connect_error(&status),
        Err(_) => connect_error(&Status::deadline_exceeded("deadline exceeded")),
    }
}

fn connect_context(headers: &axum::http::HeaderMap) -> Result<CallContext, Status> {
    let duration = if let Some(value) = headers.get("connect-timeout-ms") {
        let value = value
            .to_str()
            .map_err(|_| Status::invalid_argument("invalid timeout"))?;
        if value.is_empty() || value.len() > 10 || !value.bytes().all(|b| b.is_ascii_digit()) {
            return Err(Status::invalid_argument("invalid timeout"));
        }
        Duration::from_millis(
            value
                .parse()
                .map_err(|_| Status::invalid_argument("invalid timeout"))?,
        )
    } else {
        Duration::from_secs(60)
    };
    Ok(CallContext {
        deadline: Instant::now()
            .checked_add(duration)
            .ok_or_else(|| Status::invalid_argument("invalid timeout"))?,
    })
}

async fn connect<E: Engine>(service: Service<E>, request: Request) -> Result<Response, Status> {
    if request.method() != Method::POST {
        return Ok(StatusCode::METHOD_NOT_ALLOWED.into_response());
    }
    let (parts, body) = request.into_parts();
    let json = match parts
        .headers
        .get(CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .split(';')
        .next()
        .unwrap_or("")
        .trim()
    {
        "application/json" => true,
        "application/proto" => false,
        _ => return Ok(StatusCode::UNSUPPORTED_MEDIA_TYPE.into_response()),
    };
    if parts
        .headers
        .get("connect-protocol-version")
        .is_some_and(|v| v != "1")
    {
        return Err(Status::invalid_argument(
            "unsupported Connect protocol version",
        ));
    }
    let compression = parts
        .headers
        .get("content-encoding")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("identity");
    if compression != "identity" && compression != "gzip" {
        return Err(Status::unimplemented("unsupported request compression"));
    }
    let data = to_bytes(body, MAX_MESSAGE_BYTES)
        .await
        .map_err(|_| Status::resource_exhausted("request exceeds message limit"))?;
    let data = if compression == "gzip" {
        use std::io::Read;
        let mut decoded = Vec::new();
        flate2::read::GzDecoder::new(data.as_ref())
            .take((MAX_MESSAGE_BYTES + 1) as u64)
            .read_to_end(&mut decoded)
            .map_err(|_| Status::invalid_argument("invalid gzip request"))?;
        if decoded.len() > MAX_MESSAGE_BYTES {
            return Err(Status::resource_exhausted("request exceeds message limit"));
        }
        decoded
    } else {
        data.to_vec()
    };
    let method = parts.uri.path().strip_prefix(PREFIX).unwrap_or("");
    macro_rules! call {
        ($input:ty, $method:ident) => {{
            let input: $input = decode(&data, json)?;
            let mut request = tonic::Request::new(input);
            *request.extensions_mut() = parts.extensions;
            let response = service.$method(request).await?.into_inner();
            encode(&response, json)
        }};
    }
    match method {
        "LatestProvableHeight" => call!(proto::LatestProvableHeightRequest, latest_provable_height),
        "StateProof" => call!(proto::StateProofRequest, state_proof),
        "PacketProofs" => call!(proto::PacketProofsRequest, packet_proofs),
        _ => Err(Status::unimplemented("unknown method")),
    }
}

fn decode<T: Message + Default + DeserializeOwned>(bytes: &[u8], json: bool) -> Result<T, Status> {
    if json {
        serde_json::from_slice(bytes).map_err(|_| Status::invalid_argument("invalid protobuf JSON"))
    } else {
        T::decode(bytes).map_err(|_| Status::invalid_argument("invalid protobuf"))
    }
}

fn encode<T: Message + Serialize>(message: &T, json: bool) -> Result<Response, Status> {
    let (codec, bytes) = if json {
        (
            "application/json",
            serde_json::to_vec(message)
                .map_err(|_| Status::internal("response encoding failed"))?,
        )
    } else {
        ("application/proto", message.encode_to_vec())
    };
    if bytes.len() > MAX_MESSAGE_BYTES {
        return Err(Status::resource_exhausted("response exceeds message limit"));
    }
    Ok(([(CONTENT_TYPE, codec)], bytes).into_response())
}

fn connect_error(status: &Status) -> Response {
    let (http, code) = match status.code() {
        Code::Cancelled => (499, "canceled"),
        Code::Unknown => (500, "unknown"),
        Code::InvalidArgument => (400, "invalid_argument"),
        Code::DeadlineExceeded => (504, "deadline_exceeded"),
        Code::NotFound => (404, "not_found"),
        Code::AlreadyExists => (409, "already_exists"),
        Code::PermissionDenied => (403, "permission_denied"),
        Code::ResourceExhausted => (429, "resource_exhausted"),
        Code::FailedPrecondition => (400, "failed_precondition"),
        Code::Aborted => (409, "aborted"),
        Code::OutOfRange => (400, "out_of_range"),
        Code::Unimplemented => (501, "unimplemented"),
        Code::Unavailable => (503, "unavailable"),
        Code::DataLoss => (500, "data_loss"),
        Code::Unauthenticated => (401, "unauthenticated"),
        _ => (500, "internal"),
    };
    (
        StatusCode::from_u16(http).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR),
        Json(serde_json::json!({"code": code, "message": status.message()})),
    )
        .into_response()
}
