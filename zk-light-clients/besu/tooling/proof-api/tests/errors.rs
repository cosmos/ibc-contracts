use proof_api::engine::tool_status;
use prover_script::ToolError;
use tonic::Code;

#[test]
fn dependency_failures_are_distinguishable() {
    for (error, code, reason) in [
        (
            ToolError::RpcTransport,
            Code::Unavailable,
            "RPC_UNAVAILABLE",
        ),
        (
            ToolError::RpcHttpStatus(503),
            Code::Unavailable,
            "RPC_UNAVAILABLE",
        ),
        (ToolError::RpcTimeout, Code::DeadlineExceeded, "RPC_TIMEOUT"),
        (
            ToolError::Sp1("transport error".into()),
            Code::Unavailable,
            "PROVER_FAILED",
        ),
        (
            ToolError::ContractGetter {
                signature: "UPDATE_CLIENT_VKEY()",
                reason: "empty return data".into(),
            },
            Code::FailedPrecondition,
            "CONTRACT_GETTER_FAILED",
        ),
        (
            ToolError::VkeyMismatch("membership vkey mismatch".into()),
            Code::FailedPrecondition,
            "VKEY_MISMATCH",
        ),
    ] {
        let status = tool_status(error);
        assert_eq!(status.code(), code);
        assert!(status.message().starts_with(reason), "{status}");
        assert!(!status.message().contains("INTEGRATION_UNAVAILABLE"));
        if reason == "PROVER_FAILED" {
            assert!(status.message().contains("transport error"));
        }
    }
}

#[tokio::test]
async fn missing_and_reverting_getters_name_the_failed_signature(
) -> Result<(), Box<dyn std::error::Error>> {
    use axum::{extract::Path, routing::post, Json};
    use proof_api::contracts::{call, UPDATE_CLIENT_VKEYCall};
    use prover_script::rpc::RpcClient;
    use serde_json::{json, Value};
    async fn endpoint(Path(mode): Path<String>, Json(request): Json<Value>) -> Json<Value> {
        Json(if mode == "revert" {
            json!({"jsonrpc":"2.0", "id":request["id"], "error":{"code":-32000, "message":"execution reverted"}})
        } else {
            json!({"jsonrpc":"2.0", "id":request["id"], "result":"0x"})
        })
    }
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let address = listener.local_addr()?;
    let task = tokio::spawn(async move {
        let _ = axum::serve(
            listener,
            axum::Router::new().route("/:mode", post(endpoint)),
        )
        .await;
    });
    for mode in ["empty", "revert"] {
        let rpc = RpcClient::new(
            &format!("http://{address}/{mode}"),
            std::time::Duration::from_secs(2),
        )?;
        let result = call(&rpc, [1; 20], 1, UPDATE_CLIENT_VKEYCall).await;
        let error = result.err().ok_or("getter unexpectedly succeeded")?;
        let status = tool_status(error);
        assert_eq!(status.code(), Code::FailedPrecondition);
        assert!(status.message().starts_with("CONTRACT_GETTER_FAILED:"));
        assert!(status.message().contains("UPDATE_CLIENT_VKEY()"));
    }
    task.abort();
    let _ = task.await;
    // A transport outage must retain its RPC classification even during eth_call.
    let rpc = RpcClient::new(
        &format!("http://{address}/empty"),
        std::time::Duration::from_secs(2),
    )?;
    let error = call(&rpc, [1; 20], 1, UPDATE_CLIENT_VKEYCall)
        .await
        .err()
        .ok_or("stopped RPC unexpectedly succeeded")?;
    assert!(tool_status(error).message().starts_with("RPC_UNAVAILABLE:"));
    Ok(())
}
