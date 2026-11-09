use proof_api_integration::runtime::{BatchExecutor, Intent, Route, WorkFuture};
use prover_script::{
    proving_service::{MembershipProofResponse, ProofResponse, ProvingService},
    rpc::RpcClient,
    witness::{MembershipIntentInput, StateProofInputs},
};
use std::sync::Arc;
use tonic::Status;

#[tonic::async_trait]
pub trait ProofBackend: Send + Sync + 'static {
    async fn update(
        &self,
        chain: u64,
        client: [u8; 20],
        trusted: u64,
        target: u64,
    ) -> Result<ProofResponse, Status>;
    async fn membership(
        &self,
        route: Route,
        intents: Vec<Intent>,
    ) -> Result<MembershipProofResponse, Status>;
    async fn non_membership(
        &self,
        route: Route,
        path: Vec<Vec<u8>>,
    ) -> Result<ProofResponse, Status>;
}

pub struct NativeBackend {
    pub proving: ProvingService,
    pub source: RpcClient,
    pub verifier: RpcClient,
}
#[tonic::async_trait]
impl ProofBackend for NativeBackend {
    async fn update(
        &self,
        chain: u64,
        client: [u8; 20],
        trusted: u64,
        target: u64,
    ) -> Result<ProofResponse, Status> {
        self.proving
            .update(&self.source, &self.verifier, chain, client, trusted, target)
            .await
            .map_err(crate::engine::tool_status)
    }
    async fn membership(
        &self,
        route: Route,
        intents: Vec<Intent>,
    ) -> Result<MembershipProofResponse, Status> {
        let inputs = inputs(&route, Vec::new());
        self.proving
            .membership(
                &self.source,
                inputs,
                intents
                    .into_iter()
                    .map(|i| MembershipIntentInput {
                        path: i.path,
                        expected_commitment: i.expected_commitment,
                    })
                    .collect(),
            )
            .await
            .map_err(crate::engine::tool_status)
    }
    async fn non_membership(
        &self,
        route: Route,
        path: Vec<Vec<u8>>,
    ) -> Result<ProofResponse, Status> {
        self.proving
            .non_membership(&self.source, inputs(&route, path))
            .await
            .map_err(crate::engine::tool_status)
    }
}
fn inputs(route: &Route, path: Vec<Vec<u8>>) -> StateProofInputs {
    StateProofInputs {
        expected_source_chain_id: route.source_chain_id,
        source_router: route.source_router,
        destination_light_client: route.destination_light_client,
        proof_height: route.proof_height,
        path,
    }
}

pub struct BatchBackend(pub Arc<dyn ProofBackend>);
impl BatchExecutor for BatchBackend {
    type Output = MembershipProofResponse;
    fn prove(&self, route: Route, intents: Vec<Intent>) -> WorkFuture<Self::Output> {
        let backend = self.0.clone();
        Box::pin(async move {
            backend.membership(route, intents).await.map_err(|status| {
                serde_json::json!({"code": status.code() as i32, "message": status.message()})
                    .to_string()
            })
        })
    }
}

// The scheduler is transport independent and carries failures as strings.
pub(crate) fn decode_batch_failure(reason: &str) -> Status {
    #[derive(serde::Deserialize)]
    struct Failure {
        code: i32,
        message: String,
    }
    match serde_json::from_str::<Failure>(reason) {
        Ok(failure) => Status::new(tonic::Code::from_i32(failure.code), failure.message),
        Err(_) if reason == "proof job deadline exceeded" => {
            Status::deadline_exceeded(reason.to_owned())
        }
        Err(_) => Status::unavailable(format!("MEMBERSHIP_PROVING_FAILED: {reason}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn batching_preserves_backend_reason_and_code() {
        let reason = serde_json::json!({"code": tonic::Code::DeadlineExceeded as i32, "message": "RPC_TIMEOUT: RPC transport timed out"}).to_string();
        let status = decode_batch_failure(&reason);
        assert_eq!(status.code(), tonic::Code::DeadlineExceeded);
        assert_eq!(status.message(), "RPC_TIMEOUT: RPC transport timed out");
        assert_eq!(
            decode_batch_failure("proof job deadline exceeded").code(),
            tonic::Code::DeadlineExceeded
        );
    }
}
