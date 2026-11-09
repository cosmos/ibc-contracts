use crate::{
    backend::{BatchBackend, NativeBackend, ProofBackend},
    config::Config,
    contracts as abi,
    integration::{self, BlockSnapshot, Identity, Snapshot},
    proto,
    service::{CallContext, Engine},
};
use proof_api_integration::runtime::{BatchError, Batcher, Intent, Route};
use prover_script::{
    rpc::{fixed_hex, quantity, variable_hex, ConsensusState, RpcClient},
    ToolError,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::Arc,
    time::Duration,
};
use tokio::{
    sync::{watch, Mutex, Semaphore},
    time::timeout,
};
use tonic::Status;

#[derive(Clone)]
pub struct ProductionEngine {
    inner: Arc<Inner>,
}
type UpdateJobs = BTreeMap<u64, watch::Sender<Option<Result<Arc<Vec<u8>>, Status>>>>;
struct Inner {
    config: Arc<Config>,
    source: RpcClient,
    verifier: RpcClient,
    baseline: Identity,
    backend: Arc<dyn ProofBackend>,
    membership: Batcher<BatchBackend>,
    permits: Arc<Semaphore>,
    updates: Mutex<UpdateJobs>,
    job_timeout: Duration,
}
impl ProductionEngine {
    pub async fn from_config(config: Config) -> Result<Self, Status> {
        config
            .validate_runtime()
            .map_err(|_| Status::failed_precondition("invalid runtime configuration"))?;
        if std::env::var("SP1_PROVER").as_deref() != Ok("network")
            || std::env::var("NETWORK_RPC_URL").map_or(true, |v| v.is_empty())
        {
            return Err(Status::failed_precondition(
                "SP1_PROVER=network and NETWORK_RPC_URL are required",
            ));
        }
        prover_script::prover::validate_prover_environment().map_err(tool_status)?;
        let proving = config
            .load_proving_service()
            .map_err(|_| Status::failed_precondition("configured artifact approval failed"))?;
        let (source, verifier) = config
            .rpc
            .as_ref()
            .ok_or_else(invalid_config)?
            .clients()
            .map_err(|_| invalid_config())?;
        let backend = Arc::new(NativeBackend {
            proving,
            source: source.clone(),
            verifier: verifier.clone(),
        });
        Self::with_backend(config, backend).await
    }

    /// Injection is used by integration tests; the production constructor always uses SP1.
    pub async fn with_backend(
        config: Config,
        backend: Arc<dyn ProofBackend>,
    ) -> Result<Self, Status> {
        config.validate_runtime().map_err(|_| invalid_config())?;
        let (source, verifier) = config
            .rpc
            .as_ref()
            .ok_or_else(invalid_config)?
            .clients()
            .map_err(|_| invalid_config())?;
        let snapshot = snapshot(&config, &source, &verifier).await?;
        let scheduler = &config.scheduler;
        let max_jobs = if scheduler.allow_parallel_update_and_membership {
            scheduler.max_active_jobs
        } else {
            1
        };
        let permits = Arc::new(Semaphore::new(max_jobs));
        let job_timeout = Duration::from_secs(
            config
                .server
                .as_ref()
                .ok_or_else(invalid_config)?
                .max_job_duration_seconds,
        );
        let membership = Batcher::start(
            proof_api_integration::SchedulerConfig {
                preferred_batch_size: scheduler.preferred_batch_size,
                max_batch_size: scheduler.max_batch_size,
                max_batch_wait_ms: scheduler.max_batch_wait_ms,
                max_pending_intents: scheduler.max_pending_intents,
                allow_parallel_update_and_membership: scheduler
                    .allow_parallel_update_and_membership,
            },
            max_jobs,
            permits.clone(),
            job_timeout,
            Arc::new(BatchBackend(backend.clone())),
        )
        .map_err(batch_status)?;
        Ok(Self {
            inner: Arc::new(Inner {
                config: Arc::new(config),
                source,
                verifier,
                baseline: snapshot.identity,
                backend,
                membership,
                permits,
                updates: Mutex::new(BTreeMap::new()),
                job_timeout,
            }),
        })
    }

    pub async fn check_integration(&self) -> Result<(), Status> {
        if self.inner.membership.is_stopped() {
            return Err(Status::unavailable("scheduler stopped"));
        }
        self.inner.integration().await.map(|_| ())
    }
    pub fn stop(&self) {
        self.inner.membership.stop();
        self.inner.permits.close();
    }
    pub async fn stopped(&self) {
        self.inner.membership.stopped().await;
    }
}
fn invalid_config() -> Status {
    Status::failed_precondition("runtime configuration is incomplete")
}
#[must_use]
#[allow(clippy::needless_pass_by_value)]
pub fn tool_status(error: ToolError) -> Status {
    let message = error.to_string();
    match error {
        ToolError::RpcTimeout => Status::deadline_exceeded(format!("RPC_TIMEOUT: {message}")),
        ToolError::RpcTransport | ToolError::RpcHttpStatus(_) => {
            Status::unavailable(format!("RPC_UNAVAILABLE: {message}"))
        }
        ToolError::Sp1(_) => Status::unavailable(format!("PROVER_FAILED: {message}")),
        ToolError::ContractGetter { .. } => Status::failed_precondition(format!("CONTRACT_GETTER_FAILED: {message}; verify the deployed contract ABI and getter availability")),
        ToolError::VkeyMismatch(_) => Status::failed_precondition(format!("VKEY_MISMATCH: {message}")),
        _ => Status::failed_precondition(format!("INTEGRATION_VALIDATION_FAILED: {message}")),
    }
}
#[allow(clippy::needless_pass_by_value)]
fn batch_status(error: BatchError) -> Status {
    match error {
        BatchError::Capacity => {
            Status::resource_exhausted("SCHEDULER_CAPACITY_EXHAUSTED: membership queue is full")
        }
        BatchError::Conflict => Status::invalid_argument("conflicting membership intents"),
        BatchError::Stopped => Status::unavailable("scheduler stopped"),
        BatchError::Proof(reason) => crate::backend::decode_batch_failure(&reason),
    }
}
fn invalid_proof() -> Status {
    Status::failed_precondition("proof does not match request snapshot")
}
async fn snapshot(
    config: &Config,
    source: &RpcClient,
    verifier: &RpcClient,
) -> Result<Snapshot, Status> {
    integration::validate(
        source,
        verifier,
        config.rpc.as_ref().ok_or_else(invalid_config)?,
        config.client.as_ref().ok_or_else(invalid_config)?,
        config.contracts.as_ref().ok_or_else(invalid_config)?,
        &config.prover.approved_programs,
    )
    .await
    .map_err(tool_status)
}
impl Inner {
    async fn integration(&self) -> Result<Snapshot, Status> {
        let current = snapshot(&self.config, &self.source, &self.verifier).await?;
        if current.identity != self.baseline {
            return Err(Status::failed_precondition("integration identity changed"));
        }
        Ok(current)
    }
    async fn recheck(&self, before: &Snapshot, source: &BlockSnapshot) -> Result<Snapshot, Status> {
        let after = self.integration().await?;
        if after.identity != before.identity
            || integration::block(&self.source, &quantity(source.height))
                .await
                .map_err(tool_status)?
                != *source
        {
            return Err(Status::failed_precondition(
                "snapshot changed while proving",
            ));
        }
        Ok(after)
    }
    async fn consensus(
        &self,
        snapshot: &Snapshot,
        height: u64,
    ) -> Result<Option<ConsensusState>, Status> {
        abi::consensus(
            &self.verifier,
            snapshot.identity.light_client,
            snapshot.verifier.height,
            height,
        )
        .await
        .map_err(tool_status)
    }
    async fn client_state(
        &self,
        snapshot: &Snapshot,
    ) -> Result<besu_zk_types::ClientState, Status> {
        let encoded = abi::call(
            &self.verifier,
            snapshot.identity.light_client,
            snapshot.verifier.height,
            abi::getClientStateCall {},
        )
        .await
        .map_err(tool_status)?;
        besu_public_values::decode_client_state(&encoded).map_err(|_| invalid_proof())
    }
    async fn state_proof(&self, height: u64) -> Result<Vec<u8>, Status> {
        let snapshot = self.integration().await?;
        let client = self.client_state(&snapshot).await?;
        if height <= client.latest_height {
            return Err(Status::failed_precondition(
                "update height is already accepted",
            ));
        }
        let trusted = self
            .consensus(&snapshot, client.latest_height)
            .await?
            .ok_or_else(invalid_proof)?;
        let target = integration::block(&self.source, &quantity(height))
            .await
            .map_err(tool_status)?;
        let _permit = self
            .permits
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| Status::unavailable("stopping"))?;
        let artifact = self
            .backend
            .update(
                snapshot.identity.source_chain_id,
                snapshot.identity.light_client,
                client.latest_height,
                height,
            )
            .await?
            .proof;
        if artifact.schema_version != 1 {
            return Err(invalid_proof());
        }
        validate_identity(
            &self.config.prover.approved_programs.update,
            "update-client",
            &artifact.program,
            &artifact.elf_sha256,
            &artifact.program_v_key,
        )?;
        let bytes = hex_bytes(&artifact.public_values)?;
        let values = besu_public_values::decode_update(&bytes).map_err(|_| invalid_proof())?;
        if values.trusted_height != client.latest_height
            || values.trusted_validator_set_hash != trusted.validator_set_hash
            || values.verified_height != height
            || values.state_root != target.state_root
            || values.block_timestamp != target.timestamp
        {
            return Err(invalid_proof());
        }
        let envelope = hex_bytes(&artifact.proof_envelope)?;
        validate_envelope(&envelope, &bytes, &hex_bytes(&artifact.proof_bytes)?)?;
        let after = self.recheck(&snapshot, &target).await?;
        if self.consensus(&after, client.latest_height).await? != Some(trusted) {
            return Err(invalid_proof());
        }
        Ok(envelope)
    }
}

#[tonic::async_trait]
impl Engine for ProductionEngine {
    async fn latest_provable_height(
        &self,
        _: CallContext,
        _: proto::LatestProvableHeightRequest,
    ) -> Result<proto::LatestProvableHeightResponse, Status> {
        let snapshot = self.inner.integration().await?;
        let client = self.inner.client_state(&snapshot).await?;
        if snapshot.source.height == 0
            || snapshot.source.height <= client.latest_height
            || self
                .inner
                .consensus(&snapshot, client.latest_height)
                .await?
                .is_none()
        {
            return Err(Status::failed_precondition("no newer provable height"));
        }
        Ok(proto::LatestProvableHeightResponse {
            height: snapshot.source.height,
            timestamp: snapshot.source.timestamp,
        })
    }
    async fn state_proof(
        &self,
        _: CallContext,
        request: proto::StateProofRequest,
    ) -> Result<proto::StateProofResponse, Status> {
        let mut jobs = self.inner.updates.lock().await;
        let mut result = if let Some(existing) = jobs.get(&request.height) {
            existing.subscribe()
        } else {
            if jobs.len() >= self.inner.config.scheduler.max_pending_intents {
                return Err(Status::resource_exhausted("update capacity exhausted"));
            }
            let (sender, receiver) = watch::channel(None);
            jobs.insert(request.height, sender.clone());
            let inner = self.inner.clone();
            tokio::spawn(async move {
                let result = timeout(inner.job_timeout, inner.state_proof(request.height))
                    .await
                    .unwrap_or_else(|_| {
                        Err(Status::deadline_exceeded("proof job deadline exceeded"))
                    })
                    .map(Arc::new);
                inner.updates.lock().await.remove(&request.height);
                let _ = sender.send(Some(result));
            });
            receiver
        };
        drop(jobs);
        loop {
            if let Some(value) = result.borrow_and_update().clone() {
                return Ok(proto::StateProofResponse {
                    proof: value?.as_ref().clone(),
                });
            }
            result
                .changed()
                .await
                .map_err(|_| Status::unavailable("update job stopped"))?;
        }
    }
    async fn packet_proofs(
        &self,
        _: CallContext,
        request: proto::PacketProofsRequest,
    ) -> Result<proto::PacketProofsResponse, Status> {
        if request.packets.is_empty() {
            return Ok(proto::PacketProofsResponse { proofs: Vec::new() });
        }
        let kind = proto::ProofKind::try_from(request.kind)
            .map_err(|_| Status::invalid_argument("invalid kind"))?;
        let snapshot = self.inner.integration().await?;
        let target = integration::block(&self.inner.source, &quantity(request.height))
            .await
            .map_err(tool_status)?;
        let consensus = self.inner.consensus(&snapshot, request.height).await?;
        validate_consensus(consensus.as_ref(), &target)?;
        let client = self
            .inner
            .config
            .client
            .as_ref()
            .ok_or_else(invalid_config)?;
        let mut statements = Vec::with_capacity(request.packets.len());
        for packet in &request.packets {
            let statement = crate::packet::build_statement(
                kind,
                packet,
                &snapshot.identity.prefix,
                &client.client_id,
                &snapshot.identity.counterparty_client_id,
            )?;
            if statement.full_path.len() != 1 {
                return Err(Status::failed_precondition(
                    "one canonical path element is required",
                ));
            }
            if kind == proto::ProofKind::ReceiptAbsence
                && target.timestamp < packet.timeout_timestamp
            {
                return Err(Status::failed_precondition("packet has not timed out"));
            }
            statements.push(statement);
        }
        let route = Route {
            source_chain_id: snapshot.identity.source_chain_id,
            source_router: snapshot.identity.source_router,
            destination_light_client: snapshot.identity.light_client,
            proof_height: request.height,
        };
        let proofs = if kind == proto::ProofKind::ReceiptAbsence {
            self.inner
                .non_membership_proofs(&route, &target, &statements)
                .await?
        } else {
            self.inner
                .membership_proofs(kind, &route, &target, &statements)
                .await?
        };
        let after = self.inner.recheck(&snapshot, &target).await?;
        let current = self.inner.consensus(&after, request.height).await?;
        validate_consensus(current.as_ref(), &target)?;
        if consensus.is_some() && current != consensus {
            return Err(invalid_proof());
        }
        Ok(proto::PacketProofsResponse { proofs })
    }
}
fn validate_consensus(
    state: Option<&ConsensusState>,
    target: &BlockSnapshot,
) -> Result<(), Status> {
    if state
        .is_some_and(|v| v.state_root != target.state_root || v.block_timestamp != target.timestamp)
    {
        return Err(invalid_proof());
    }
    Ok(())
}
fn validate_identity(
    expected: &prover_script::identity::ProgramIdentity,
    role: &str,
    program: &str,
    elf: &str,
    key: &str,
) -> Result<(), Status> {
    if program != role
        || !elf.eq_ignore_ascii_case(&expected.elf_sha256)
        || !key.eq_ignore_ascii_case(&expected.program_v_key)
    {
        return Err(invalid_proof());
    }
    Ok(())
}
fn hex_bytes(value: &str) -> Result<Vec<u8>, Status> {
    variable_hex(value, "proof artifact").map_err(tool_status)
}
fn validate_envelope(envelope: &[u8], values: &[u8], proof: &[u8]) -> Result<(), Status> {
    let decoded =
        besu_public_values::decode_proof_envelope(envelope).map_err(|_| invalid_proof())?;
    if decoded.public_values != values || decoded.proof_bytes != proof || proof.is_empty() {
        return Err(invalid_proof());
    }
    Ok(())
}

impl Inner {
    async fn non_membership_proofs(
        &self,
        route: &Route,
        target: &BlockSnapshot,
        statements: &[crate::packet::Statement],
    ) -> Result<Vec<Vec<u8>>, Status> {
        let mut proofs = Vec::with_capacity(statements.len());

        for statement in statements {
            let _permit = self
                .permits
                .clone()
                .acquire_owned()
                .await
                .map_err(|_| Status::unavailable("stopping"))?;
            let artifact = self
                .backend
                .non_membership(route.clone(), statement.full_path.clone())
                .await?
                .proof;
            if artifact.schema_version != 1 {
                return Err(invalid_proof());
            }
            validate_identity(
                &self.config.prover.approved_programs.non_membership,
                "non-membership",
                &artifact.program,
                &artifact.elf_sha256,
                &artifact.program_v_key,
            )?;
            let values_bytes = hex_bytes(&artifact.public_values)?;
            let values = besu_public_values::decode_non_membership(&values_bytes)
                .map_err(|_| invalid_proof())?;
            if values.proof_height != route.proof_height
                || values.state_root != target.state_root
                || values.source_router != route.source_router
                || values.path_digest != statement.path_digest
            {
                return Err(invalid_proof());
            }
            let envelope = hex_bytes(&artifact.proof_envelope)?;
            validate_envelope(&envelope, &values_bytes, &hex_bytes(&artifact.proof_bytes)?)?;
            proofs.push(envelope);
        }
        Ok(proofs)
    }
    async fn membership_proofs(
        &self,
        kind: proto::ProofKind,
        route: &Route,
        target: &BlockSnapshot,
        statements: &[crate::packet::Statement],
    ) -> Result<Vec<Vec<u8>>, Status> {
        let mut proofs = Vec::with_capacity(statements.len());

        let expected = if kind == proto::ProofKind::Acknowledgement {
            integration::acknowledgement_commitments(
                &self.source,
                target,
                route.source_router,
                &statements
                    .iter()
                    .map(|s| s.logical_slot)
                    .collect::<Vec<_>>(),
            )
            .await
            .map_err(tool_status)?
        } else {
            statements
                .iter()
                .map(|s| s.expected_commitment.ok_or_else(invalid_proof))
                .collect::<Result<Vec<_>, _>>()?
        };
        let intents = statements
            .iter()
            .zip(&expected)
            .map(|(s, c)| Intent {
                path: s.full_path.clone(),
                expected_commitment: *c,
            })
            .collect();
        let batches = self
            .membership
            .prove(route.clone(), intents)
            .await
            .map_err(batch_status)?;
        let mut emitted = BTreeSet::new();
        for ((statement, commitment), response) in statements.iter().zip(expected).zip(batches) {
            proofs.push(membership_envelope(
                &self.config.prover.approved_programs.membership,
                route,
                target,
                statement,
                commitment,
                &response.proof,
                &mut emitted,
            )?);
        }
        Ok(proofs)
    }
}
// Keep artifact binding and branch verification together for review.
#[allow(clippy::too_many_lines)]
fn membership_envelope(
    approved: &prover_script::identity::ProgramIdentity,
    route: &Route,
    target: &BlockSnapshot,
    statement: &crate::packet::Statement,
    commitment: [u8; 32],
    artifact: &prover_script::artifact::MembershipBatchArtifact,
    emitted: &mut BTreeSet<[u8; 32]>,
) -> Result<Vec<u8>, Status> {
    if artifact.schema_version != 2 {
        return Err(invalid_proof());
    }
    validate_identity(
        approved,
        "membership",
        &artifact.program,
        &artifact.elf_sha256,
        &artifact.program_v_key,
    )?;
    let values_bytes = hex_bytes(&artifact.public_values)?;
    let proof_bytes = hex_bytes(&artifact.proof_bytes)?;
    if proof_bytes.is_empty() {
        return Err(invalid_proof());
    }
    let values =
        besu_public_values::decode_membership(&values_bytes).map_err(|_| invalid_proof())?;
    let root: [u8; 32] = fixed_hex(&artifact.batch_root, "batchRoot").map_err(tool_status)?;
    if values.proof_height != route.proof_height
        || artifact.proof_height != route.proof_height
        || values.state_root != target.state_root
        || values.source_router != route.source_router
        || values.batch_root != root
        || values.entry_count != artifact.entry_count
        || values.entry_count == 0
        || values.entry_count > 32
        || artifact.entries.len() != values.entry_count as usize
    {
        return Err(invalid_proof());
    }
    let mut indices = BTreeSet::new();
    let mut paths = BTreeSet::new();
    for entry in &artifact.entries {
        if !indices.insert(entry.leaf_index)
            || !paths.insert(entry.path_digest.to_ascii_lowercase())
        {
            return Err(invalid_proof());
        }
    }
    let entry = artifact
        .entries
        .iter()
        .find(|entry| {
            fixed_hex::<32>(&entry.path_digest, "pathDigest").ok() == Some(statement.path_digest)
                && fixed_hex::<32>(&entry.expected_commitment, "expectedCommitment").ok()
                    == Some(commitment)
        })
        .ok_or_else(invalid_proof)?;
    let branch = entry
        .merkle_proof
        .iter()
        .map(|h| fixed_hex::<32>(h, "merkle sibling").map_err(tool_status))
        .collect::<Result<Vec<_>, _>>()?;
    batch_merkle::verify(
        root,
        batch_merkle::membership_leaf(statement.path_digest, commitment),
        entry.leaf_index as usize,
        values.entry_count as usize,
        &branch,
    )
    .map_err(|_| invalid_proof())?;
    let full = emitted.insert(root);
    Ok(besu_public_values::encode_membership_envelope(
        &besu_zk_types::MembershipBatchProofEnvelope {
            mode: if full {
                besu_zk_types::MEMBERSHIP_ENVELOPE_FULL
            } else {
                besu_zk_types::MEMBERSHIP_ENVELOPE_CACHED
            },
            public_values: if full { values_bytes } else { Vec::new() },
            proof_bytes: if full { proof_bytes } else { Vec::new() },
            batch_root: root,
            entry_count: values.entry_count,
            leaf_index: entry.leaf_index,
            merkle_proof: branch,
        },
    ))
}
