// SPDX-License-Identifier: Apache-2.0

//! Reusable aggregator for attester proof API modules

#[allow(clippy::all, clippy::pedantic, clippy::nursery)]
/// RPC definitions for the attestor
pub mod rpc {
    tonic::include_proto!("aggregator");
    tonic::include_proto!("ibc_attestor");
}

mod attestor_data;
mod config;
mod quorum;

use futures::future::join_all;
use moka::future::Cache;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use tokio::time::{timeout, Duration};
use tonic::{transport::Channel, Request, Status};
use tracing::error as tracing_error;
use {
    attestor_data::AttestatorData,
    rpc::{
        attestation_service_client::AttestationServiceClient, AggregatedAttestation, Attestation,
        LatestHeightRequest, PacketAttestationRequest, StateAttestationRequest,
    },
};

/// Result of fetching attestations for packets
/// Contains both the state attestation and the packet-specific attestation
#[derive(Debug, Clone)]
pub struct AttestationResult {
    /// State attestation for the height
    pub state: AggregatedAttestation,
    /// Packet-specific attestation (send/ack/receipt)
    pub packet: AggregatedAttestation,
}

pub use config::{AttestorConfig, CacheConfig, Config};

#[derive(Clone)]
enum AttestationQuery {
    Packet(Vec<Vec<u8>>, u64, rpc::CommitmentType), // packets, height, commitment_type
    State(u64),
}

type AttestorClient = (String, AttestationServiceClient<Channel>);
type PacketCacheKey = ([u8; 32], u64, rpc::CommitmentType);

/// Signature aggregator service that collects and aggregates attestations from multiple attestors.
/// # Architecture
/// The aggregator operates in two phases:
/// 1. **Packet Attestation**: Queries attestors for attestations of specific packets at a height
/// 2. **State Attestation**: Queries attestors for state attestations at the determined height
///
/// Both phases require a quorum of valid attestations before proceeding. Results are cached
/// using the following cache keys:
/// - State cache: `height -> aggregated attestation`
/// - Packet cache: `(packets_hash, height, commitment_type) -> aggregated attestation`
#[derive(Debug)]
pub struct Aggregator {
    quorum_threshold: usize,
    attestor_timeout_duration: Duration,
    attestor_clients: Vec<AttestorClient>,
    state_cache: Cache<u64, AggregatedAttestation>,
    packet_cache: Cache<PacketCacheKey, AggregatedAttestation>,
}

impl Aggregator {
    /// Creates aggregator from the [`Config`]
    ///
    /// # Errors
    /// - Fails if clients cannot be created
    pub async fn from_config(config: Config) -> anyhow::Result<Self> {
        let attestor_clients = Self::create_clients(&config.attestor).await?;

        Ok(Self {
            quorum_threshold: config.attestor.quorum_threshold,
            attestor_timeout_duration: Duration::from_millis(
                config.attestor.attestor_query_timeout_ms,
            ),
            attestor_clients,
            state_cache: Cache::new(config.cache.state_cache_max_entries),
            packet_cache: Cache::new(config.cache.packet_cache_max_entries),
        })
    }

    async fn create_clients(config: &AttestorConfig) -> anyhow::Result<Vec<AttestorClient>> {
        let futures = config.attestor_endpoints.iter().map(|endpoint| async move {
            AttestationServiceClient::connect(endpoint.clone())
                .await
                .map(|client| (endpoint.clone(), client))
        });

        join_all(futures)
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| anyhow::anyhow!("Failed to connect to attestor: {e}"))
    }
}

impl Aggregator {
    /// Get only state attestation at a given height
    ///
    /// # Errors
    /// - Fails if attestation cannot be fetched
    pub async fn get_state_attestation(
        &self,
        height: u64,
    ) -> anyhow::Result<AggregatedAttestation> {
        let state_attestation = self
            .state_cache
            .try_get_with(height, async {
                let state_attestations = self
                    .query_attestations(AttestationQuery::State(height))
                    .await;

                let quorumed_aggregation =
                    agg_quorumed_attestations(self.quorum_threshold, state_attestations)
                        .map_err(|e| Status::failed_precondition(e.to_string()))?;

                Ok(quorumed_aggregation)
            })
            .await
            .map_err(|e: Arc<Status>| (*e).clone())?;

        Ok(state_attestation)
    }

    /// Get attestations
    ///
    /// # Errors
    /// - Fails if attesations cannot be fetched
    /// - Fails if input packets are empty
    pub async fn get_attestations(
        &self,
        packets: Vec<Vec<u8>>,
        height: u64,
        commitment_type: rpc::CommitmentType,
    ) -> anyhow::Result<AttestationResult> {
        if packets.is_empty() {
            return Err(anyhow::anyhow!("Packets cannot be empty"));
        }

        if packets.iter().any(std::vec::Vec::is_empty) {
            return Err(anyhow::anyhow!("Packet cannot be empty"));
        }

        let mut packets = packets;
        let packet_cache_key = Self::make_packet_cache_key(&mut packets, height, commitment_type);

        let packet_attestation = self
            .packet_cache
            .try_get_with(packet_cache_key, async {
                let packet_attestations = self
                    .query_attestations(AttestationQuery::Packet(packets, height, commitment_type))
                    .await;

                let quorumed_aggregation =
                    agg_quorumed_attestations(self.quorum_threshold, packet_attestations)
                        .map_err(|e| Status::failed_precondition(e.to_string()))?;

                Ok(quorumed_aggregation)
            })
            .await
            .map_err(|e: Arc<Status>| (*e).clone())?;

        let state_attestation = self
            .get_state_attestation(packet_attestation.height)
            .await?;

        Ok(AttestationResult {
            state: state_attestation,
            packet: packet_attestation,
        })
    }

    async fn query_attestations(&self, query: AttestationQuery) -> Vec<Option<Attestation>> {
        let timeout_duration = self.attestor_timeout_duration;
        let query_futures = self.attestor_clients.iter().map(|(endpoint, client)| {
            let query = query.clone();
            let mut client = client.clone();
            async move {
                let response = timeout(timeout_duration, async {
                    match query {
                        AttestationQuery::Packet(packets, height, commitment_type) => {
                            let request = Request::new(PacketAttestationRequest {
                                packets,
                                height,
                                commitment_type: commitment_type as i32,
                            });
                            client
                                .packet_attestation(request)
                                .await
                                .map(|r| r.into_inner().attestation)
                        }
                        AttestationQuery::State(height) => {
                            let request = Request::new(StateAttestationRequest { height });
                            client
                                .state_attestation(request)
                                .await
                                .map(|r| r.into_inner().attestation)
                        }
                    }
                })
                .await;

                match response {
                    Ok(Ok(attestation)) => (endpoint, Ok(attestation)),
                    Ok(Err(status)) => (endpoint, Err(status)),
                    Err(_) => (
                        endpoint,
                        Err(Status::deadline_exceeded(format!(
                            "Request timed out after {timeout_duration:?}"
                        ))),
                    ),
                }
            }
        });

        let results = join_all(query_futures).await;

        let successful_responses = results
            .into_iter()
            .filter_map(|(endpoint, result)| match result {
                Ok(attestation) => Some(attestation),
                Err(e) => {
                    tracing_error!("Attestor [{endpoint}] failed, error: {e:?}");
                    None
                }
            })
            .collect();

        successful_responses
    }

    fn make_packet_cache_key(
        packets: &mut [Vec<u8>],
        height: u64,
        commitment_type: rpc::CommitmentType,
    ) -> PacketCacheKey {
        packets.sort();

        let mut hasher = Sha256::new();
        for packet in packets {
            hasher.update(packet.len().to_be_bytes());
            hasher.update(packet);
        }

        (hasher.finalize().into(), height, commitment_type)
    }

    /// Get the latest height from the attested chain.
    /// Returns the minimum height reported by a quorum of attestors.
    ///
    /// # Errors
    /// - Fails if quorum cannot be reached
    pub async fn get_latest_height(&self) -> anyhow::Result<u64> {
        let query_futures = self
            .attestor_clients
            .iter()
            .map(|(endpoint, client)| self.query_attestor_height(endpoint, client.clone()));

        let heights: Vec<u64> = join_all(query_futures)
            .await
            .into_iter()
            .flatten()
            .collect();

        quorum::select_quorum_height(heights, self.quorum_threshold)
            .map_err(|e| anyhow::anyhow!("{e}"))
    }

    async fn query_attestor_height(
        &self,
        endpoint: &str,
        mut client: AttestationServiceClient<Channel>,
    ) -> Option<u64> {
        let request = Request::new(LatestHeightRequest {});

        match timeout(
            self.attestor_timeout_duration,
            client.latest_height(request),
        )
        .await
        {
            Ok(Ok(response)) => Some(response.into_inner().height),
            Ok(Err(e)) => {
                tracing_error!("Attestor [{endpoint}] failed to get latest height: {e:?}");
                None
            }
            Err(_) => {
                tracing_error!(
                    "Attestor [{endpoint}] timed out after {:?}",
                    self.attestor_timeout_duration
                );
                None
            }
        }
    }
}

/// Process attestations and create an aggregate response if the quorum is met.
fn agg_quorumed_attestations(
    quorum_threshold: usize,
    attestations: Vec<Option<Attestation>>,
) -> Result<AggregatedAttestation, anyhow::Error> {
    let mut attestator_data = AttestatorData::new();

    attestations.into_iter().flatten().for_each(|attestation| {
        if let Err(e) = attestator_data.insert(attestation) {
            tracing_error!("Invalid attestation, continuing with other responses: {e:#?}");
        }
    });

    attestator_data
        .agg_quorumed_attestations(quorum_threshold)
        .ok_or_else(|| anyhow::anyhow!("Quorum not met"))
}

#[cfg(test)]
mod tests {
    use super::{rpc::CommitmentType, Aggregator};

    #[test]
    fn packet_cache_key_has_complete_canonical_identity() {
        let key = |mut packets: Vec<Vec<u8>>, commitment_type| {
            Aggregator::make_packet_cache_key(&mut packets, 7, commitment_type)
        };

        assert_eq!(
            key(vec![vec![2], vec![1]], CommitmentType::Packet),
            key(vec![vec![1], vec![2]], CommitmentType::Packet)
        );
        assert_ne!(
            key(vec![vec![1], vec![2]], CommitmentType::Packet),
            key(vec![vec![1, 2]], CommitmentType::Packet)
        );
        assert_ne!(
            key(vec![vec![1], vec![2]], CommitmentType::Packet),
            key(vec![vec![1], vec![2]], CommitmentType::Ack)
        );
    }
}
