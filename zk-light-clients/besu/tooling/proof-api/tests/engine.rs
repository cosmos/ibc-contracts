#![allow(clippy::result_large_err)]
//! Full engine tests use authenticated MPT fixtures and an injected proof backend.
//! SP1 proving remains covered by the real-proof deployment gate, not these fake proof bytes.
use alloy_primitives::B256;
use axum::routing::post;
use proof_api::{
    backend::ProofBackend,
    config::Config,
    engine::ProductionEngine,
    proto,
    service::{CallContext, Engine},
};
use proof_api_integration::runtime::{Intent, Route};
use prover_script::{
    artifact::{MembershipBatchArtifact, ProofArtifact},
    proving_service::{MembershipProofResponse, ProofResponse, WitnessMetadata},
};
use serde_json::json;
use std::sync::{
    atomic::{AtomicBool, AtomicUsize, Ordering},
    Arc,
};
use std::time::Duration;
use tonic::Status;

#[path = "../examples/support/mod.rs"]
mod rpc_fixture;
use rpc_fixture::{hex, rpc, Chain};
struct Backend {
    chain: Arc<Chain>,
    calls: AtomicUsize,
    change_after: AtomicBool,
    bad_values: AtomicBool,
    fail_update: AtomicBool,
}
fn metadata() -> WitnessMetadata {
    WitnessMetadata {
        serialized_bytes: 1,
        sha256: hex([1; 32]),
    }
}
#[allow(clippy::needless_pass_by_value)]
fn artifact(role: &str, key: u8, values: Vec<u8>) -> Result<ProofResponse, Status> {
    Ok(ProofResponse {
        proof: ProofArtifact::new(role, role.as_bytes(), &hex([key; 32]), &values, &[1, 2, 3])
            .map_err(|_| Status::internal("fixture"))?,
        witness: metadata(),
    })
}
#[tonic::async_trait]
impl ProofBackend for Backend {
    async fn update(
        &self,
        _: u64,
        _: [u8; 20],
        trusted: u64,
        target: u64,
    ) -> Result<ProofResponse, Status> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        tokio::time::sleep(Duration::from_millis(25)).await;
        if self.fail_update.load(Ordering::SeqCst) {
            return Err(Status::unavailable("test proof failure"));
        }
        artifact(
            "update-client",
            1,
            besu_public_values::encode_update(&besu_zk_types::UpdatePublicValues {
                schema_version: 1,
                trusted_height: trusted,
                trusted_validator_set_hash: [4; 32],
                verified_height: target,
                state_root: self.chain.fixture.state_root,
                block_timestamp: 1000,
                new_validator_set_hash: [5; 32],
            }),
        )
    }
    async fn membership(
        &self,
        route: Route,
        intents: Vec<Intent>,
    ) -> Result<MembershipProofResponse, Status> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        if self.change_after.load(Ordering::SeqCst) {
            self.chain.changed.store(true, Ordering::SeqCst);
        }
        let leaves = intents
            .iter()
            .map(|i| {
                batch_merkle::membership_leaf(
                    ibc_storage::path_digest(&i.path),
                    i.expected_commitment,
                )
            })
            .collect::<Vec<_>>();
        let values = besu_zk_types::MembershipPublicValues {
            schema_version: 2,
            proof_height: route.proof_height,
            state_root: if self.bad_values.load(Ordering::SeqCst) {
                [0; 32]
            } else {
                self.chain.fixture.state_root
            },
            source_router: route.source_router,
            batch_root: batch_merkle::root(&leaves).map_err(|_| Status::internal("fixture"))?,
            entry_count: u32::try_from(intents.len()).map_err(|_| Status::internal("fixture"))?,
        };
        let witness = besu_zk_types::MembershipWitness {
            schema_version: 2,
            proof_height: route.proof_height,
            state_root: self.chain.fixture.state_root,
            source_router: route.source_router,
            account_proof: vec![],
            entries: intents
                .into_iter()
                .map(|i| besu_zk_types::MembershipEntryWitness {
                    path: i.path,
                    expected_commitment: i.expected_commitment,
                    storage_proof: vec![],
                })
                .collect(),
        };
        let proof = MembershipBatchArtifact::new(
            b"membership",
            &hex([2; 32]),
            &witness,
            &besu_public_values::encode_membership(&values),
            &[1, 2, 3],
        )
        .map_err(|_| Status::internal("fixture"))?;
        Ok(MembershipProofResponse {
            proof,
            witness: metadata(),
        })
    }
    async fn non_membership(
        &self,
        route: Route,
        path: Vec<Vec<u8>>,
    ) -> Result<ProofResponse, Status> {
        artifact(
            "non-membership",
            3,
            besu_public_values::encode_non_membership(&besu_zk_types::NonMembershipPublicValues {
                schema_version: 1,
                proof_height: route.proof_height,
                state_root: self.chain.fixture.state_root,
                source_router: route.source_router,
                path_digest: ibc_storage::path_digest(&path),
            }),
        )
    }
}
struct Fixture {
    engine: ProductionEngine,
    backend: Arc<Backend>,
    server: tokio::task::JoinHandle<()>,
}
impl Drop for Fixture {
    fn drop(&mut self) {
        self.engine.stop();
        self.server.abort();
    }
}
async fn fixture() -> Result<Fixture, Box<dyn std::error::Error>> {
    let chain = Arc::new(Chain {
        fixture: prover_script::golden::membership_witness_with_proof_depth(0)?,
        changed: AtomicBool::new(false),
        accepted: AtomicBool::new(false),
        vkeys: [
            B256::from([1; 32]),
            B256::from([2; 32]),
            B256::from([3; 32]),
        ],
    });
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let address = listener.local_addr()?;
    let router = axum::Router::new()
        .route("/:side", post(rpc))
        .with_state(chain.clone());
    let server = tokio::spawn(async move {
        let _ = axum::serve(listener, router).await;
    });
    let mut programs = serde_json::Map::new();
    for (role, key) in [
        ("update-client", 1),
        ("membership", 2),
        ("non-membership", 3),
    ] {
        programs.insert(role.into(),json!({"elfSha256":hex(besu_zk_types::sha256(role.as_bytes())), "programVKey":hex([key;32])}));
    }
    let config: Config = serde_json::from_value(json!({"schemaVersion":1,
        "prover":{"artifactsDir":"unused", "approvedPrograms":programs}, "server":{"listen":"127.0.0.1:0"},
        "client":{"chainId":"chain-b", "clientId":"client-0"}, "contracts":{"verifierRouter":hex([0x55;20])},
        "rpc":{"source":{"proxyUrl":format!("http://{address}/source")}, "verifier":{"proxyUrl":format!("http://{address}/verifier")},
            "expectedSourceChainId":1, "expectedVerifierChainId":2},
        "scheduler":{"preferredBatchSize":2, "maxBatchSize":32} }))?;
    let backend = Arc::new(Backend {
        chain,
        calls: AtomicUsize::new(0),
        change_after: AtomicBool::new(false),
        bad_values: AtomicBool::new(false),
        fail_update: AtomicBool::new(false),
    });
    let engine = ProductionEngine::with_backend(config, backend.clone()).await?;
    Ok(Fixture {
        engine,
        backend,
        server,
    })
}
fn context() -> CallContext {
    CallContext {
        deadline: tokio::time::Instant::now() + Duration::from_secs(10),
    }
}
fn packet(sequence: u64, reverse: bool) -> proto::Packet {
    proto::Packet {
        sequence,
        source_client: if reverse { "client-0" } else { "client-1" }.into(),
        destination_client: if reverse { "client-1" } else { "client-0" }.into(),
        timeout_timestamp: 999,
        payloads: vec![proto::Payload {
            source_port: "a".into(),
            destination_port: "b".into(),
            version: "1".into(),
            encoding: "json".into(),
            value: vec![1],
        }],
    }
}
fn request(kind: i32, packets: Vec<proto::Packet>) -> proto::PacketProofsRequest {
    proto::PacketProofsRequest {
        client: None,
        height: 12,
        kind,
        packets,
    }
}

#[tokio::test]
async fn shares_batches_but_each_response_has_its_own_full_envelope(
) -> Result<(), Box<dyn std::error::Error>> {
    let f = fixture().await?;
    let (a, b) = tokio::join!(
        f.engine
            .packet_proofs(context(), request(1, vec![packet(1, false)])),
        f.engine
            .packet_proofs(context(), request(1, vec![packet(2, false)]))
    );
    let a = besu_public_values::decode_membership_envelope(&a?.proofs[0])?;
    let b = besu_public_values::decode_membership_envelope(&b?.proofs[0])?;
    assert_eq!(a.mode, 1);
    assert_eq!(b.mode, 1);
    assert_eq!(a.batch_root, b.batch_root);
    assert_eq!(f.backend.calls.load(Ordering::SeqCst), 1);
    let result = f
        .engine
        .packet_proofs(
            context(),
            request(1, (1..=80).map(|i| packet(i, false)).collect()),
        )
        .await?;
    assert_eq!(result.proofs.len(), 80);
    let mut roots = std::collections::BTreeSet::new();
    for proof in result.proofs {
        let envelope = besu_public_values::decode_membership_envelope(&proof)?;
        let first = roots.insert(envelope.batch_root);
        assert_eq!(envelope.mode, if first { 1 } else { 2 });
    }
    assert_eq!(roots.len(), 3);
    Ok(())
}
#[tokio::test]
async fn updates_dedupe_and_acknowledgements_use_authenticated_storage(
) -> Result<(), Box<dyn std::error::Error>> {
    let f = fixture().await?;
    let input = proto::StateProofRequest {
        client: None,
        height: 12,
    };
    let (a, b) = tokio::join!(
        f.engine.state_proof(context(), input.clone()),
        f.engine.state_proof(context(), input)
    );
    assert_eq!(a?.proof, b?.proof);
    assert_eq!(f.backend.calls.load(Ordering::SeqCst), 1);
    let result = f
        .engine
        .packet_proofs(
            context(),
            request(2, vec![packet(7, true), packet(7, true)]),
        )
        .await?;
    assert_eq!(result.proofs.len(), 2);
    assert!(f
        .engine
        .packet_proofs(context(), request(2, vec![packet(8, true)]))
        .await
        .is_err());
    let absent = f
        .engine
        .packet_proofs(context(), request(3, vec![packet(8, true)]))
        .await?;
    assert!(!absent.proofs[0].is_empty());
    let mut future = packet(8, true);
    future.timeout_timestamp = 1001;
    assert!(f
        .engine
        .packet_proofs(context(), request(3, vec![future]))
        .await
        .is_err());
    Ok(())
}
#[tokio::test]
async fn rejects_wrong_public_values_and_source_changes_after_proving(
) -> Result<(), Box<dyn std::error::Error>> {
    let f = fixture().await?;
    f.backend.bad_values.store(true, Ordering::SeqCst);
    assert!(f
        .engine
        .packet_proofs(
            context(),
            request(1, vec![packet(1, false), packet(2, false)])
        )
        .await
        .is_err());
    f.backend.bad_values.store(false, Ordering::SeqCst);
    f.backend.change_after.store(true, Ordering::SeqCst);
    assert!(f
        .engine
        .packet_proofs(
            context(),
            request(1, vec![packet(3, false), packet(4, false)])
        )
        .await
        .is_err());
    Ok(())
}

#[tokio::test]
async fn failed_update_is_shared_and_released_for_retry() -> Result<(), Box<dyn std::error::Error>>
{
    let f = fixture().await?;
    let latest = f
        .engine
        .latest_provable_height(
            context(),
            proto::LatestProvableHeightRequest { client: None },
        )
        .await?;
    assert_eq!(latest.height, 12);
    assert_eq!(latest.timestamp, 1000);
    assert!(f
        .engine
        .state_proof(
            context(),
            proto::StateProofRequest {
                client: None,
                height: 11
            }
        )
        .await
        .is_err());
    f.backend.fail_update.store(true, Ordering::SeqCst);
    let input = proto::StateProofRequest {
        client: None,
        height: 12,
    };
    let (a, b) = tokio::join!(
        f.engine.state_proof(context(), input.clone()),
        f.engine.state_proof(context(), input.clone())
    );
    assert!(a.is_err() && b.is_err());
    assert_eq!(f.backend.calls.load(Ordering::SeqCst), 1);
    f.backend.fail_update.store(false, Ordering::SeqCst);
    assert!(!f
        .engine
        .state_proof(context(), input)
        .await?
        .proof
        .is_empty());
    assert_eq!(f.backend.calls.load(Ordering::SeqCst), 2);
    f.engine.stop();
    f.engine.stopped().await;
    assert!(f.engine.check_integration().await.is_err());
    Ok(())
}
