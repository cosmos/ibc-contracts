#![allow(clippy::result_large_err)]

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use axum::{
    body::{to_bytes, Body},
    http::{Request, StatusCode},
};
use proof_api::{
    proto::{self, prover_service_client::ProverServiceClient},
    service::{CallContext, Engine, Service},
    transport::router,
};
use prost::Message;
use tokio::net::TcpListener;
use tonic::Status;
use tower::ServiceExt;

struct Fixture;
#[tonic::async_trait]
impl Engine for Fixture {
    async fn latest_provable_height(
        &self,
        _: CallContext,
        _: proto::LatestProvableHeightRequest,
    ) -> Result<proto::LatestProvableHeightResponse, Status> {
        Ok(proto::LatestProvableHeightResponse {
            height: u64::MAX,
            timestamp: 1_700_000_000,
        })
    }
    async fn state_proof(
        &self,
        _: CallContext,
        request: proto::StateProofRequest,
    ) -> Result<proto::StateProofResponse, Status> {
        if request.height == 99 {
            tokio::time::sleep(Duration::from_secs(1)).await;
        }
        if request.height == 97 {
            return Ok(proto::StateProofResponse {
                proof: vec![0x5a; 5 * 1024 * 1024],
            });
        }
        Ok(proto::StateProofResponse {
            proof: vec![0, 1, 255],
        })
    }
    async fn packet_proofs(
        &self,
        _: CallContext,
        request: proto::PacketProofsRequest,
    ) -> Result<proto::PacketProofsResponse, Status> {
        if request.height == 98 {
            return Ok(proto::PacketProofsResponse { proofs: vec![] });
        }
        Ok(proto::PacketProofsResponse {
            proofs: request
                .packets
                .into_iter()
                .map(|p| p.sequence.to_be_bytes().to_vec())
                .collect(),
        })
    }
}
fn client() -> proto::Client {
    proto::Client {
        chain_id: "chain-b".into(),
        client_id: "client-0".into(),
    }
}
fn app() -> Result<axum::Router, Status> {
    Ok(router(
        Service::new(Arc::new(Fixture), client())?,
        Arc::new(AtomicBool::new(true)),
    ))
}

#[tokio::test]
async fn grpc_over_plaintext_http2_roundtrips_all_methods() -> Result<(), Box<dyn std::error::Error>>
{
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let address = listener.local_addr()?;
    let server = tokio::spawn(async move {
        axum::serve(listener, app()?)
            .await
            .map_err(|e| Status::internal(e.to_string()))
    });
    let mut rpc = ProverServiceClient::connect(format!("http://{address}"))
        .await?
        .max_decoding_message_size(proof_api::transport::MAX_MESSAGE_BYTES)
        .send_compressed(tonic::codec::CompressionEncoding::Gzip)
        .accept_compressed(tonic::codec::CompressionEncoding::Gzip);
    let latest = rpc
        .latest_provable_height(proto::LatestProvableHeightRequest {
            client: Some(client()),
        })
        .await?
        .into_inner();
    assert_eq!(latest.height, u64::MAX);
    assert_eq!(latest.timestamp, 1_700_000_000);
    assert_eq!(
        rpc.state_proof(proto::StateProofRequest {
            client: Some(client()),
            height: 1
        })
        .await?
        .into_inner()
        .proof,
        [0, 1, 255]
    );
    let large = rpc
        .state_proof(proto::StateProofRequest {
            client: Some(client()),
            height: 97,
        })
        .await?
        .into_inner();
    assert_eq!(large.proof.len(), 5 * 1024 * 1024);
    assert!(large.proof.iter().all(|b| *b == 0x5a));
    let packets = vec![
        proto::Packet {
            sequence: 2,
            ..Default::default()
        },
        proto::Packet {
            sequence: 1,
            ..Default::default()
        },
    ];
    let proofs = rpc
        .packet_proofs(proto::PacketProofsRequest {
            client: Some(client()),
            height: 1,
            kind: 1,
            packets,
        })
        .await?
        .into_inner()
        .proofs;
    assert_eq!(
        proofs,
        vec![2u64.to_be_bytes().to_vec(), 1u64.to_be_bytes().to_vec()]
    );
    let error = rpc
        .state_proof(proto::StateProofRequest {
            client: Some(proto::Client {
                chain_id: "wrong".into(),
                ..client()
            }),
            height: 1,
        })
        .await
        .err()
        .ok_or("expected unknown-client error")?;
    assert_eq!(error.code(), tonic::Code::NotFound);
    let mut slow = tonic::Request::new(proto::StateProofRequest {
        client: Some(client()),
        height: 99,
    });
    slow.set_timeout(Duration::from_millis(5));
    // Tonic may cancel locally when its deadline races the server's status trailer.
    assert!(matches!(
        rpc.state_proof(slow)
            .await
            .err()
            .ok_or("expected deadline")?
            .code(),
        tonic::Code::DeadlineExceeded | tonic::Code::Cancelled
    ));
    server.abort();
    Ok(())
}

#[tokio::test]
async fn connect_json_preserves_protobuf_uint64_and_bytes() -> Result<(), Box<dyn std::error::Error>>
{
    let latest = app()?
        .oneshot(
            Request::post("/ibc.v2.prover.ProverService/LatestProvableHeight")
                .header("content-type", "application/json")
                .body(Body::from(
                    r#"{"client":{"chainId":"chain-b","clientId":"client-0"}}"#,
                ))?,
        )
        .await?;
    assert_eq!(latest.status(), StatusCode::OK);
    let body: serde_json::Value =
        serde_json::from_slice(&to_bytes(latest.into_body(), 1024).await?)?;
    assert_eq!(body["height"], u64::MAX.to_string());
    let proof = app()?.oneshot(Request::post("/ibc.v2.prover.ProverService/StateProof").header("content-type", "application/json")
        .body(Body::from(r#"{"client":{"chain_id":"chain-b","client_id":"client-0"},"height":"18446744073709551615"}"#))?).await?;
    assert_eq!(proof.status(), StatusCode::OK);
    let body: serde_json::Value =
        serde_json::from_slice(&to_bytes(proof.into_body(), 1024).await?)?;
    assert_eq!(body["proof"], "AAH/");
    Ok(())
}

#[tokio::test]
async fn connect_binary_errors_deadline_and_cardinality() -> Result<(), Box<dyn std::error::Error>>
{
    let request = proto::StateProofRequest {
        client: Some(client()),
        height: 1,
    };
    let response = app()?
        .oneshot(
            Request::post("/ibc.v2.prover.ProverService/StateProof")
                .header("content-type", "application/proto")
                .body(Body::from(request.encode_to_vec()))?,
        )
        .await?;
    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        proto::StateProofResponse::decode(to_bytes(response.into_body(), 1024).await?)?.proof,
        [0, 1, 255]
    );
    for (body, code, timeout) in [
        (
            serde_json::json!({"client": client(), "height": 0}),
            StatusCode::BAD_REQUEST,
            "1000",
        ),
        (
            serde_json::json!({"client": client(), "height": 99}),
            StatusCode::GATEWAY_TIMEOUT,
            "1",
        ),
        (
            serde_json::json!({"client": client(), "height": 1}),
            StatusCode::BAD_REQUEST,
            "bogus",
        ),
    ] {
        let response = app()?
            .oneshot(
                Request::post("/ibc.v2.prover.ProverService/StateProof")
                    .header("content-type", "application/json")
                    .header("connect-timeout-ms", timeout)
                    .body(Body::from(body.to_string()))?,
            )
            .await?;
        assert_eq!(response.status(), code);
    }
    let response = app()?.oneshot(Request::post("/ibc.v2.prover.ProverService/PacketProofs").header("content-type", "application/json")
        .body(Body::from(serde_json::json!({"client": client(), "height": 98, "kind": 1, "packets": [{"sequence": 1}]}).to_string()))?).await?;
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    Ok(())
}

#[tokio::test]
async fn health_and_readiness_use_runtime_state() -> Result<(), Box<dyn std::error::Error>> {
    let ready = Arc::new(AtomicBool::new(false));
    let app = router(Service::new(Arc::new(Fixture), client())?, ready.clone());
    assert_eq!(
        app.clone()
            .oneshot(Request::get("/healthz").body(Body::empty())?)
            .await?
            .status(),
        StatusCode::OK
    );
    assert_eq!(
        app.clone()
            .oneshot(Request::get("/readyz").body(Body::empty())?)
            .await?
            .status(),
        StatusCode::SERVICE_UNAVAILABLE
    );
    ready.store(true, Ordering::Release);
    assert_eq!(
        app.oneshot(Request::get("/readyz").body(Body::empty())?)
            .await?
            .status(),
        StatusCode::OK
    );
    Ok(())
}

#[tokio::test]
async fn connect_deadline_includes_request_body_reading() -> Result<(), Box<dyn std::error::Error>>
{
    let body = Body::from_stream(tokio_stream::pending::<
        Result<axum::body::Bytes, std::io::Error>,
    >());
    let response = app()?
        .oneshot(
            Request::post("/ibc.v2.prover.ProverService/StateProof")
                .header("content-type", "application/json")
                .header("connect-timeout-ms", "1")
                .body(body)?,
        )
        .await?;
    assert_eq!(response.status(), StatusCode::GATEWAY_TIMEOUT);
    Ok(())
}

#[tokio::test]
async fn connect_gzip_and_metrics_roundtrip() -> Result<(), Box<dyn std::error::Error>> {
    use std::io::Write;
    let mut encoder = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
    encoder.write_all(
        serde_json::json!({"client":client(),"height":1})
            .to_string()
            .as_bytes(),
    )?;
    let app = app()?;
    let response = app
        .clone()
        .oneshot(
            Request::post("/ibc.v2.prover.ProverService/StateProof")
                .header("content-type", "application/json")
                .header("content-encoding", "gzip")
                .body(Body::from(encoder.finish()?))?,
        )
        .await?;
    assert_eq!(response.status(), StatusCode::OK);
    let response = app
        .oneshot(Request::get("/metrics").body(Body::empty())?)
        .await?;
    let metrics = String::from_utf8(to_bytes(response.into_body(), 4096).await?.to_vec())?;
    assert!(metrics.contains("besu_prover_requests_total{method=\"state_proof\",result=\"ok\"} 1"));
    assert!(metrics.contains("besu_prover_inflight_requests 0"));
    Ok(())
}

#[tokio::test]
async fn grpc_web_preserves_messages_and_status_trailers() -> Result<(), Box<dyn std::error::Error>>
{
    for (configured, status) in [
        (client(), "grpc-status:0"),
        (
            proto::Client {
                chain_id: "wrong".into(),
                ..client()
            },
            "grpc-status:5",
        ),
    ] {
        let data = proto::StateProofRequest {
            client: Some(configured),
            height: 1,
        }
        .encode_to_vec();
        let mut body = vec![0];
        body.extend_from_slice(&u32::try_from(data.len())?.to_be_bytes());
        body.extend(data);
        let response = app()?
            .oneshot(
                Request::post("/ibc.v2.prover.ProverService/StateProof")
                    .header("content-type", "application/grpc-web+proto")
                    .body(Body::from(body))?,
            )
            .await?;
        assert_eq!(response.status(), StatusCode::OK);
        let header_status = response
            .headers()
            .get("grpc-status")
            .and_then(|value| value.to_str().ok())
            .map(|value| format!("grpc-status:{value}"));
        let bytes = to_bytes(response.into_body(), 4096).await?;
        let mut offset = 0;
        let mut trailers = header_status.unwrap_or_default();
        let mut proof = None;
        while offset + 5 <= bytes.len() {
            let length = usize::try_from(u32::from_be_bytes(
                bytes[offset + 1..offset + 5].try_into()?,
            ))?;
            let flag = bytes[offset];
            offset += 5;
            let message = bytes
                .get(offset..offset + length)
                .ok_or("truncated frame")?;
            if flag == 0x80 {
                trailers = String::from_utf8(message.to_vec())?;
            } else {
                proof = Some(proto::StateProofResponse::decode(message)?.proof);
            }
            offset += length;
        }
        assert!(trailers.replace(' ', "").contains(status), "{trailers}");
        if status == "grpc-status:0" {
            assert_eq!(proof, Some(vec![0, 1, 255]));
        }
    }
    Ok(())
}
