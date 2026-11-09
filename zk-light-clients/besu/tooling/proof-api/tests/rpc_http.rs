use axum::{
    extract::State,
    http::{header::LOCATION, StatusCode},
    response::{IntoResponse, Response},
    routing::post,
    Json,
};
use prover_script::{
    rpc::{RpcClient, RpcSettings},
    ToolError,
};
use serde_json::{json, Value};
use std::{
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

async fn endpoint(State(calls): State<Arc<AtomicUsize>>, Json(request): Json<Value>) -> Response {
    let count = calls.fetch_add(1, Ordering::SeqCst);
    match request["method"].as_str().unwrap_or("") {
        "retry" if count == 0 => StatusCode::SERVICE_UNAVAILABLE.into_response(),
        "large" => Json(json!({"jsonrpc":"2.0", "id":request["id"], "result":"x".repeat(4096)}))
            .into_response(),
        "redirect" => (StatusCode::FOUND, [(LOCATION, "http://127.0.0.1:9/")]).into_response(),
        "slow" => {
            tokio::time::sleep(Duration::from_millis(100)).await;
            Json(json!({"jsonrpc":"2.0", "id":request["id"], "result":true})).into_response()
        }
        _ => Json(json!({"jsonrpc":"2.0", "id":request["id"], "result":true})).into_response(),
    }
}
#[tokio::test]
async fn rpc_limits_retries_ids_redirects_and_deadlines() -> Result<(), Box<dyn std::error::Error>>
{
    let calls = Arc::new(AtomicUsize::new(0));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let address = listener.local_addr()?;
    let app = axum::Router::new()
        .route("/", post(endpoint))
        .with_state(calls.clone());
    let task = tokio::spawn(async move {
        let _ = axum::serve(listener, app).await;
    });
    let settings = RpcSettings {
        timeout: Duration::from_millis(50),
        max_attempts: 2,
        retry_backoff: Duration::ZERO,
        max_response_bytes: 1024,
    };
    let client = RpcClient::with_settings(&format!("http://{address}/"), settings)?;
    assert_eq!(client.call("retry", json!([])).await?, json!(true));
    assert_eq!(calls.load(Ordering::SeqCst), 2);
    assert_eq!(client.call("second-id", json!([])).await?, json!(true));
    assert!(matches!(
        client.call("large", json!([])).await,
        Err(ToolError::RpcResponseTooLarge)
    ));
    assert!(matches!(
        client.call("redirect", json!([])).await,
        Err(ToolError::RpcTransport)
    ));
    let before = calls.load(Ordering::SeqCst);
    assert!(matches!(
        client.call("slow", json!([])).await,
        Err(ToolError::RpcTimeout)
    ));
    assert_eq!(
        calls.load(Ordering::SeqCst),
        before + 1,
        "timeouts must not blindly retry"
    );
    task.abort();
    Ok(())
}
