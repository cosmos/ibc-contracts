use std::{future::Future, sync::Arc, time::Duration};

use crate::observability::Metrics;
use prost::Message;
use tokio::time::{timeout_at, Instant};
use tonic::{Request, Response, Status};

use crate::proto::{self, prover_service_server::ProverService};

/// The deadline applies to RPC, scheduler admission, and proof waiting as well as transport.
#[derive(Clone, Copy, Debug)]
pub struct CallContext {
    pub deadline: Instant,
}

#[tonic::async_trait]
pub trait Engine: Send + Sync + 'static {
    async fn latest_provable_height(
        &self,
        context: CallContext,
        request: proto::LatestProvableHeightRequest,
    ) -> Result<proto::LatestProvableHeightResponse, Status>;
    async fn state_proof(
        &self,
        context: CallContext,
        request: proto::StateProofRequest,
    ) -> Result<proto::StateProofResponse, Status>;
    async fn packet_proofs(
        &self,
        context: CallContext,
        request: proto::PacketProofsRequest,
    ) -> Result<proto::PacketProofsResponse, Status>;
}

pub struct Service<E> {
    engine: Arc<E>,
    client: proto::Client,
    pub metrics: Arc<Metrics>,
}

impl<E> Clone for Service<E> {
    fn clone(&self) -> Self {
        Self {
            engine: self.engine.clone(),
            client: self.client.clone(),
            metrics: self.metrics.clone(),
        }
    }
}

impl<E: Engine> Service<E> {
    pub fn new(engine: Arc<E>, client: proto::Client) -> Result<Self, Status> {
        if client.chain_id.trim().is_empty() || client.client_id.trim().is_empty() {
            return Err(Status::invalid_argument(
                "configured client pair is required",
            ));
        }
        Ok(Self {
            engine,
            client,
            metrics: Arc::new(Metrics::default()),
        })
    }

    fn check_client(&self, client: Option<&proto::Client>) -> Result<(), Status> {
        let client = client.ok_or_else(|| Status::invalid_argument("INVALID_ARGUMENT"))?;
        if *client != self.client {
            return Err(Status::not_found("UNKNOWN_CLIENT"));
        }
        Ok(())
    }
}

#[tonic::async_trait]
impl<E: Engine> ProverService for Service<E> {
    async fn latest_provable_height(
        &self,
        request: Request<proto::LatestProvableHeightRequest>,
    ) -> Result<Response<proto::LatestProvableHeightResponse>, Status> {
        let input = request.get_ref();
        let measurement = self.metrics.start("latest_provable_height").with_request(
            input.client.as_ref(),
            None,
            None,
            &[],
        );
        let result = async {
            let context = context(&request)?;
            self.check_client(request.get_ref().client.as_ref())?;
            bounded(
                context,
                self.engine
                    .latest_provable_height(context, request.into_inner()),
            )
            .await
            .map(Response::new)
        }
        .await;
        measurement.finish(
            result.as_ref().err(),
            result.as_ref().map_or(0, |r| r.get_ref().encoded_len()),
        );
        result
    }

    async fn state_proof(
        &self,
        request: Request<proto::StateProofRequest>,
    ) -> Result<Response<proto::StateProofResponse>, Status> {
        let input = request.get_ref();
        let measurement = self.metrics.start("state_proof").with_request(
            input.client.as_ref(),
            Some(input.height),
            None,
            &[],
        );
        let result = async {
            let context = context(&request)?;
            self.check_client(request.get_ref().client.as_ref())?;
            if request.get_ref().height == 0 {
                return Err(Status::invalid_argument("INVALID_ARGUMENT"));
            }
            let response = bounded(
                context,
                self.engine.state_proof(context, request.into_inner()),
            )
            .await?;
            if response.proof.is_empty() {
                return Err(Status::internal("engine returned empty proof"));
            }
            Ok(Response::new(response))
        }
        .await;
        measurement.finish(
            result.as_ref().err(),
            result.as_ref().map_or(0, |r| r.get_ref().encoded_len()),
        );
        result
    }

    async fn packet_proofs(
        &self,
        request: Request<proto::PacketProofsRequest>,
    ) -> Result<Response<proto::PacketProofsResponse>, Status> {
        let input = request.get_ref();
        let measurement = self.metrics.start("packet_proofs").with_request(
            input.client.as_ref(),
            Some(input.height),
            Some(input.kind),
            &input.packets,
        );
        let result = async {
        let context = context(&request)?;
        let input = request.get_ref();
        self.check_client(input.client.as_ref())?;
        if input.height == 0
            || !matches!(proto::ProofKind::try_from(input.kind), Ok(kind) if kind != proto::ProofKind::Unspecified)
        {
            return Err(Status::invalid_argument("INVALID_ARGUMENT"));
        }
        let count = input.packets.len();
        let response = bounded(
            context,
            self.engine.packet_proofs(context, request.into_inner()),
        )
        .await?;
        if response.proofs.len() != count || response.proofs.iter().any(Vec::is_empty) {
            return Err(Status::internal(
                "engine returned invalid proof cardinality or empty proof",
            ));
        }
        Ok(Response::new(response))
            }.await;
        measurement.finish(
            result.as_ref().err(),
            result.as_ref().map_or(0, |r| r.get_ref().encoded_len()),
        );
        result
    }
}

async fn bounded<T>(
    context: CallContext,
    work: impl Future<Output = Result<T, Status>>,
) -> Result<T, Status> {
    timeout_at(context.deadline, work)
        .await
        .map_err(|_| Status::deadline_exceeded("deadline exceeded"))?
}

pub(crate) fn context<T>(request: &Request<T>) -> Result<CallContext, Status> {
    if let Some(context) = request.extensions().get::<CallContext>() {
        return Ok(*context);
    }
    let duration = request
        .metadata()
        .get("grpc-timeout")
        .map(|value| {
            value
                .to_str()
                .map_err(|_| Status::invalid_argument("invalid timeout"))
                .and_then(parse_timeout)
        })
        .transpose()?
        .unwrap_or(Duration::from_secs(60));
    let deadline = Instant::now()
        .checked_add(duration)
        .ok_or_else(|| Status::invalid_argument("invalid timeout"))?;
    Ok(CallContext { deadline })
}

fn parse_timeout(value: &str) -> Result<Duration, Status> {
    let invalid = || Status::invalid_argument("invalid timeout");
    if !(2..=9).contains(&value.len()) {
        return Err(invalid());
    }
    let (digits, unit) = value.split_at(value.len() - 1);
    if !digits.bytes().all(|b| b.is_ascii_digit()) {
        return Err(invalid());
    }
    let amount = digits.parse::<u64>().map_err(|_| invalid())?;
    match unit {
        "H" => Ok(Duration::from_secs(amount * 3600)),
        "M" => Ok(Duration::from_secs(amount * 60)),
        "S" => Ok(Duration::from_secs(amount)),
        "m" => Ok(Duration::from_millis(amount)),
        "u" => Ok(Duration::from_micros(amount)),
        "n" => Ok(Duration::from_nanos(amount)),
        _ => Err(invalid()),
    }
}
