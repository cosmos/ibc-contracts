use std::{collections::BTreeSet, env, net::SocketAddr, path::PathBuf, sync::Arc, time::Duration};

use axum::{
    extract::{DefaultBodyLimit, State},
    http::{header::AUTHORIZATION, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::{net::TcpListener, sync::Semaphore};

use crate::{
    prover::validate_prover_environment,
    proving_service::ProvingService,
    rpc::{fixed_hex, variable_hex, RpcClient},
    witness::{MembershipIntentInput, StateProofInputs},
    ToolError,
};

pub use crate::proving_service::{MembershipProofResponse, ProofResponse, WitnessMetadata};

const MAX_REQUEST_BYTES: usize = 64 * 1024;
const MAX_RPC_TIMEOUT_SECONDS: u64 = 300;
const MIN_API_TOKEN_BYTES: usize = 32;

#[derive(Debug, Error)]
pub enum HostError {
    #[error("invalid host configuration: {0}")]
    InvalidConfiguration(String),
    #[error("failed to load prover programs: {0}")]
    Program(#[from] ToolError),
    #[error("host listener failed: {0}")]
    Listener(#[from] std::io::Error),
}

pub struct HostConfig {
    bind_address: SocketAddr,
    artifacts_directory: PathBuf,
    authorization_header: Arc<str>,
    allowed_rpc_endpoints: Arc<BTreeSet<String>>,
}

impl HostConfig {
    pub fn from_env() -> Result<Self, HostError> {
        let bind_address = env_value("BESU_PROVER_HOST_BIND")
            .unwrap_or_else(|| "127.0.0.1:8080".to_owned())
            .parse::<SocketAddr>()
            .map_err(|error| {
                HostError::InvalidConfiguration(format!(
                    "BESU_PROVER_HOST_BIND is not a socket address: {error}"
                ))
            })?;
        let artifacts_directory = PathBuf::from(
            env_value("BESU_PROVER_ARTIFACTS_DIR").unwrap_or_else(|| "artifacts".to_owned()),
        );
        let token = required_env("BESU_PROVER_HOST_TOKEN")?;
        if token.len() < MIN_API_TOKEN_BYTES {
            return Err(HostError::InvalidConfiguration(format!(
                "BESU_PROVER_HOST_TOKEN must contain at least {MIN_API_TOKEN_BYTES} bytes"
            )));
        }
        let authorization = format!("Bearer {token}");
        HeaderValue::from_str(&authorization).map_err(|_| {
            HostError::InvalidConfiguration(
                "BESU_PROVER_HOST_TOKEN contains invalid HTTP header bytes".to_owned(),
            )
        })?;

        let allowed_rpc_endpoints =
            parse_allowed_rpc_endpoints(&required_env("BESU_PROVER_ALLOWED_RPC_URLS")?)?;

        if env_value("SP1_PROVER").as_deref() != Some("network") {
            return Err(HostError::InvalidConfiguration(
                "SP1_PROVER must be set to network for the long-running host".to_owned(),
            ));
        }
        required_env("NETWORK_RPC_URL")?;
        validate_prover_environment()?;

        Ok(Self {
            bind_address,
            artifacts_directory,
            authorization_header: Arc::from(authorization),
            allowed_rpc_endpoints: Arc::new(allowed_rpc_endpoints),
        })
    }
}

#[derive(Clone)]
struct HostState {
    proving: ProvingService,
    authorization_header: Arc<str>,
    allowed_rpc_endpoints: Arc<BTreeSet<String>>,
    proof_slot: Arc<Semaphore>,
}

impl HostState {
    fn authorize(&self, headers: &HeaderMap) -> Result<(), ApiError> {
        let supplied = headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .unwrap_or_default();
        if constant_time_eq(supplied.as_bytes(), self.authorization_header.as_bytes()) {
            Ok(())
        } else {
            Err(ApiError::unauthorized())
        }
    }

    fn clients(&self, rpc: &RpcRequest) -> Result<(RpcClient, RpcClient), ApiError> {
        validate_rpc_timeout(rpc.rpc_timeout_seconds)?;
        self.validate_rpc_endpoint(&rpc.source_rpc)?;
        self.validate_rpc_endpoint(&rpc.destination_rpc)?;
        let timeout = Duration::from_secs(rpc.rpc_timeout_seconds);
        Ok((
            RpcClient::new(&rpc.source_rpc, timeout)?,
            RpcClient::new(&rpc.destination_rpc, timeout)?,
        ))
    }

    fn validate_rpc_endpoint(&self, value: &str) -> Result<(), ApiError> {
        let normalized = normalize_rpc_endpoint(value).map_err(ApiError::bad_request)?;
        if self.allowed_rpc_endpoints.contains(&normalized) {
            Ok(())
        } else {
            Err(ApiError::new(
                StatusCode::BAD_REQUEST,
                "rpc_not_allowed",
                "RPC endpoint is not in BESU_PROVER_ALLOWED_RPC_URLS",
            ))
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RpcRequest {
    pub source_rpc: String,
    pub destination_rpc: String,
    pub expected_source_chain_id: u64,
    #[serde(default = "default_rpc_timeout_seconds")]
    pub rpc_timeout_seconds: u64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateProofRequest {
    pub rpc: RpcRequest,
    pub destination_light_client: String,
    pub trusted_height: u64,
    pub target_height: u64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MembershipProofRequest {
    pub rpc: RpcRequest,
    pub source_router: String,
    pub destination_light_client: String,
    pub proof_height: u64,
    pub entries: Vec<MembershipProofEntry>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct MembershipProofEntry {
    pub path: String,
    pub expected_commitment: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct NonMembershipProofRequest {
    pub rpc: RpcRequest,
    pub source_router: String,
    pub destination_light_client: String,
    pub proof_height: u64,
    pub path: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct StatusResponse {
    api_revision: &'static str,
    status: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ErrorResponse {
    code: &'static str,
    message: String,
}

#[derive(Debug)]
struct ApiError {
    status: StatusCode,
    code: &'static str,
    message: String,
}

impl ApiError {
    fn new(status: StatusCode, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
        }
    }

    fn bad_request(message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, "invalid_request", message)
    }

    fn unauthorized() -> Self {
        Self::new(
            StatusCode::UNAUTHORIZED,
            "unauthorized",
            "missing or invalid bearer token",
        )
    }

    fn busy() -> Self {
        Self::new(
            StatusCode::TOO_MANY_REQUESTS,
            "prover_busy",
            "another proof is already in progress; retry later",
        )
    }
}

impl From<ToolError> for ApiError {
    fn from(error: ToolError) -> Self {
        let (status, code) = match &error {
            ToolError::RpcResponseTooLarge => (StatusCode::BAD_GATEWAY, "rpc_failure"),
            ToolError::RpcTimeout => (StatusCode::GATEWAY_TIMEOUT, "rpc_timeout"),
            ToolError::RpcTransport
            | ToolError::RpcHttpStatus(_)
            | ToolError::MalformedRpcResponse
            | ToolError::RpcRejected(_)
            | ToolError::HistoricalStateUnavailable
            | ToolError::NullResult(_) => (StatusCode::BAD_GATEWAY, "rpc_failure"),
            ToolError::Sp1(_) => (StatusCode::BAD_GATEWAY, "sp1_failure"),
            ToolError::Io(_)
            | ToolError::InvalidProgramIdentity(_)
            | ToolError::InvalidArtifact(_)
            | ToolError::ArtifactSerialization(_)
            | ToolError::MissingNetworkPrivateKey
            | ToolError::InvalidNetworkPrivateKey
            | ToolError::InvalidNetworkRpcUrl
            | ToolError::NonUnicodeEnvironment(_)
            | ToolError::UnsupportedSp1Prover(_) => {
                (StatusCode::INTERNAL_SERVER_ERROR, "host_failure")
            }
            _ => (StatusCode::BAD_REQUEST, "invalid_request"),
        };
        Self::new(status, code, error.to_string())
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ErrorResponse {
                code: self.code,
                message: self.message,
            }),
        )
            .into_response()
    }
}

pub async fn serve(config: HostConfig) -> Result<(), HostError> {
    let proving = ProvingService::load(&config.artifacts_directory)?;
    let state = HostState {
        proving,
        authorization_header: config.authorization_header,
        allowed_rpc_endpoints: config.allowed_rpc_endpoints,
        // Groth16 proving is deliberately serialized. The proof API can retry a 429 response.
        proof_slot: Arc::new(Semaphore::new(1)),
    };
    let application = router(state);
    let listener = TcpListener::bind(config.bind_address).await?;
    println!("Besu SP1 host listening on {}", config.bind_address);
    axum::serve(listener, application)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

fn router(state: HostState) -> Router {
    Router::new()
        .route("/healthz", get(health))
        .route("/readyz", get(ready))
        .route("/v1/readyz", get(authorized_ready))
        .route("/v1/proofs/update-client", post(prove_update))
        .route("/v1/proofs/membership", post(prove_membership))
        .route("/v1/proofs/non-membership", post(prove_non_membership))
        .layer(DefaultBodyLimit::max(MAX_REQUEST_BYTES))
        .with_state(state)
}

async fn health() -> Json<StatusResponse> {
    Json(StatusResponse {
        status: "ok",
        api_revision: "sp1-host-http-v2-batch",
    })
}

async fn ready() -> Json<StatusResponse> {
    Json(StatusResponse {
        status: "ready",
        api_revision: "sp1-host-http-v2-batch",
    })
}

async fn authorized_ready(
    State(state): State<HostState>,
    headers: HeaderMap,
) -> Result<Json<StatusResponse>, ApiError> {
    state.authorize(&headers)?;
    Ok(Json(StatusResponse {
        status: "ready",
        api_revision: "sp1-host-http-v2-batch",
    }))
}

async fn prove_update(
    State(state): State<HostState>,
    headers: HeaderMap,
    Json(request): Json<UpdateProofRequest>,
) -> Result<Json<ProofResponse>, ApiError> {
    state.authorize(&headers)?;
    let _permit = state
        .proof_slot
        .clone()
        .try_acquire_owned()
        .map_err(|_| ApiError::busy())?;
    let (source, destination) = state.clients(&request.rpc)?;
    state
        .proving
        .update(
            &source,
            &destination,
            request.rpc.expected_source_chain_id,
            fixed_hex(
                &request.destination_light_client,
                "destination light client",
            )?,
            request.trusted_height,
            request.target_height,
        )
        .await
        .map(Json)
        .map_err(ApiError::from)
}

async fn prove_membership(
    State(state): State<HostState>,
    headers: HeaderMap,
    Json(request): Json<MembershipProofRequest>,
) -> Result<Json<MembershipProofResponse>, ApiError> {
    state.authorize(&headers)?;
    let _permit = state
        .proof_slot
        .clone()
        .try_acquire_owned()
        .map_err(|_| ApiError::busy())?;
    let (inputs, intents) = membership_inputs(&request)?;
    let (source, _destination) = state.clients(&request.rpc)?;
    state
        .proving
        .membership(&source, inputs, intents)
        .await
        .map(Json)
        .map_err(ApiError::from)
}

fn membership_inputs(
    request: &MembershipProofRequest,
) -> Result<(StateProofInputs, Vec<MembershipIntentInput>), ApiError> {
    if request.entries.is_empty() || request.entries.len() > besu_zk_types::MAX_BATCH_MESSAGES {
        return Err(ApiError::bad_request(
            "membership batch size must be 1..=32",
        ));
    }
    let intents = request
        .entries
        .iter()
        .map(|entry| {
            Ok(MembershipIntentInput {
                path: vec![variable_hex(&entry.path, "path")?],
                expected_commitment: fixed_hex(&entry.expected_commitment, "expected commitment")?,
            })
        })
        .collect::<Result<Vec<_>, ApiError>>()?;
    let inputs = StateProofInputs {
        expected_source_chain_id: request.rpc.expected_source_chain_id,
        source_router: fixed_hex(&request.source_router, "source Router")?,
        destination_light_client: fixed_hex(
            &request.destination_light_client,
            "destination light client",
        )?,
        proof_height: request.proof_height,
        path: Vec::new(),
    };
    Ok((inputs, intents))
}

async fn prove_non_membership(
    State(state): State<HostState>,
    headers: HeaderMap,
    Json(request): Json<NonMembershipProofRequest>,
) -> Result<Json<ProofResponse>, ApiError> {
    state.authorize(&headers)?;
    let _permit = state
        .proof_slot
        .clone()
        .try_acquire_owned()
        .map_err(|_| ApiError::busy())?;
    let (source, _destination) = state.clients(&request.rpc)?;
    state
        .proving
        .non_membership(
            &source,
            state_proof_inputs(
                &request.rpc,
                &request.source_router,
                &request.destination_light_client,
                request.proof_height,
                &request.path,
            )?,
        )
        .await
        .map(Json)
        .map_err(ApiError::from)
}

fn state_proof_inputs(
    rpc: &RpcRequest,
    source_router: &str,
    destination_light_client: &str,
    proof_height: u64,
    path: &str,
) -> Result<StateProofInputs, ApiError> {
    Ok(StateProofInputs {
        expected_source_chain_id: rpc.expected_source_chain_id,
        source_router: fixed_hex(source_router, "source Router")?,
        destination_light_client: fixed_hex(destination_light_client, "destination light client")?,
        proof_height,
        path: vec![variable_hex(path, "path")?],
    })
}

fn validate_rpc_timeout(seconds: u64) -> Result<(), ApiError> {
    if (1..=MAX_RPC_TIMEOUT_SECONDS).contains(&seconds) {
        Ok(())
    } else {
        Err(ApiError::bad_request(format!(
            "rpcTimeoutSeconds must be between 1 and {MAX_RPC_TIMEOUT_SECONDS}"
        )))
    }
}

fn default_rpc_timeout_seconds() -> u64 {
    30
}

fn parse_allowed_rpc_endpoints(value: &str) -> Result<BTreeSet<String>, HostError> {
    let endpoints = value
        .split(',')
        .map(str::trim)
        .filter(|endpoint| !endpoint.is_empty())
        .map(normalize_rpc_endpoint)
        .collect::<Result<BTreeSet<_>, _>>()
        .map_err(HostError::InvalidConfiguration)?;
    if endpoints.is_empty() {
        return Err(HostError::InvalidConfiguration(
            "BESU_PROVER_ALLOWED_RPC_URLS must contain at least one endpoint".to_owned(),
        ));
    }
    Ok(endpoints)
}

fn normalize_rpc_endpoint(value: &str) -> Result<String, String> {
    let endpoint = reqwest::Url::parse(value).map_err(|_| "invalid RPC URL".to_owned())?;
    if !matches!(endpoint.scheme(), "http" | "https") || endpoint.host_str().is_none() {
        return Err("RPC URL must use HTTP(S) and include a host".to_owned());
    }
    Ok(endpoint.to_string())
}

fn required_env(name: &'static str) -> Result<String, HostError> {
    env_value(name)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| HostError::InvalidConfiguration(format!("{name} must be set and non-empty")))
}

fn env_value(name: &'static str) -> Option<String> {
    env::var(name).ok()
}

fn constant_time_eq(left: &[u8], right: &[u8]) -> bool {
    if left.len() != right.len() {
        return false;
    }
    left.iter()
        .zip(right)
        .fold(0_u8, |difference, (left, right)| {
            difference | (left ^ right)
        })
        == 0
}

async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};

        if let Ok(mut terminate) = signal(SignalKind::terminate()) {
            tokio::select! {
                result = tokio::signal::ctrl_c() => {
                    if let Err(error) = result {
                        eprintln!("failed to listen for Ctrl-C: {error}");
                    }
                }
                _ = terminate.recv() => {}
            }
            return;
        }
    }

    if let Err(error) = tokio::signal::ctrl_c().await {
        eprintln!("failed to listen for shutdown signal: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn membership_accepts_api_batch_and_enforces_size_and_commitments(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let body = serde_json::json!({
            "rpc": {"sourceRpc": "http://chain-a:8545", "destinationRpc": "http://chain-b:8545",
                "expectedSourceChainId": 41001, "rpcTimeoutSeconds": 30},
            "sourceRouter": format!("0x{}", "11".repeat(20)),
            "destinationLightClient": format!("0x{}", "22".repeat(20)),
            "proofHeight": 12,
            "entries": [
                {"path": "0x01", "expectedCommitment": format!("0x{}", "33".repeat(32))},
                {"path": "0x02", "expectedCommitment": format!("0x{}", "44".repeat(32))}
            ]
        });
        let mut request: MembershipProofRequest = serde_json::from_value(body)?;
        let (inputs, intents) =
            membership_inputs(&request).map_err(|error| format!("{error:?}"))?;
        assert_eq!(inputs.proof_height, 12);
        assert_eq!(intents.len(), 2);
        assert_eq!(intents[1].path, vec![vec![2]]);
        assert_eq!(intents[1].expected_commitment, [0x44; 32]);
        request.entries[0].expected_commitment = "0x01".to_owned();
        assert!(membership_inputs(&request).is_err());
        request.entries = vec![request.entries[1].clone(); 33];
        assert!(membership_inputs(&request).is_err());
        request.entries.clear();
        assert!(membership_inputs(&request).is_err());
        Ok(())
    }

    #[test]
    fn rpc_endpoint_allowlist_is_canonical_and_rejects_empty_input(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let allowed =
            parse_allowed_rpc_endpoints("http://chain-a:8545, https://chain-b.example.test/rpc")?;
        assert!(allowed.contains("http://chain-a:8545/"));
        assert!(allowed.contains("https://chain-b.example.test/rpc"));
        assert!(parse_allowed_rpc_endpoints(" , ").is_err());
        Ok(())
    }

    #[test]
    fn authorization_comparison_is_exact() {
        assert!(constant_time_eq(b"Bearer secret", b"Bearer secret"));
        assert!(!constant_time_eq(b"Bearer secret", b"Bearer other"));
        assert!(!constant_time_eq(b"Bearer secret", b"Bearer secret-longer"));
    }

    #[test]
    fn rpc_timeout_is_bounded() {
        assert!(validate_rpc_timeout(1).is_ok());
        assert!(validate_rpc_timeout(MAX_RPC_TIMEOUT_SECONDS).is_ok());
        assert!(validate_rpc_timeout(0).is_err());
        assert!(validate_rpc_timeout(MAX_RPC_TIMEOUT_SECONDS + 1).is_err());
    }
}
