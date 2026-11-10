// SPDX-License-Identifier: Apache-2.0

//! Utilities for attested relays targeting Ethereum chains.

use std::{collections::HashMap, time::Duration};

use crate::aggregator::rpc::AggregatedAttestation;
use crate::aggregator::Aggregator;
use crate::events::EurekaEventWithHeight;
use crate::utils::attestor::{
    collect_send_and_ack_packets_with_height, collect_timeout_packets, fetch_attestations,
    AttestationData,
};
use crate::utils::{
    eth_eureka::{self, PacketCall},
    wait_for_condition, RelayEventsParams,
};
use alloy::{
    primitives::{Address, Bytes},
    providers::Provider,
    sol_types::SolValue,
};
use anyhow::Result;
use ibc_eureka_solidity_types::{
    ibc_multicall::multicall_calldata, ics26::IICS02ClientMsgs::Height as ICS26Height,
    msgs::IAttestationMsgs::AttestationProof,
};

/// Parameter key for minimum required signatures.
pub const MIN_REQUIRED_SIGS: &str = "min_required_sigs";
/// Parameter key for initial height.
pub const HEIGHT: &str = "height";
/// Parameter key for initial timestamp.
pub const TIMESTAMP: &str = "timestamp";
/// Parameter key for attestor addresses.
pub const ATTESTOR_ADDRESSES: &str = "attestor_addresses";

/// Parsed parameters for creating an attestor light client.
pub struct AttestorClientParams {
    /// Minimum required signatures for attestation verification.
    pub min_required_sigs: u8,
    /// Initial height for the client state.
    pub height: u64,
    /// Initial timestamp for the consensus state.
    pub timestamp: u64,
    /// List of attestor addresses.
    pub attestor_addresses: Vec<Address>,
}

/// Parses parameters for creating an attestor light client.
///
/// # Errors
/// Returns an error if required parameters are missing or invalid.
pub fn parse_attestor_client_params<S: std::hash::BuildHasher>(
    parameters: &HashMap<String, String, S>,
) -> Result<AttestorClientParams> {
    let min_required_sigs: u8 = parameters
        .get(MIN_REQUIRED_SIGS)
        .ok_or_else(|| anyhow::anyhow!("Missing `{MIN_REQUIRED_SIGS}` parameter"))?
        .parse()?;

    let height: u64 = parameters
        .get(HEIGHT)
        .ok_or_else(|| anyhow::anyhow!("Missing `{HEIGHT}` parameter"))?
        .parse()?;

    let timestamp: u64 = parameters
        .get(TIMESTAMP)
        .ok_or_else(|| anyhow::anyhow!("Missing `{TIMESTAMP}` parameter"))?
        .parse()?;

    let addrs_hex = parameters
        .get(ATTESTOR_ADDRESSES)
        .ok_or_else(|| anyhow::anyhow!("Missing `{ATTESTOR_ADDRESSES}` parameter"))?;

    let attestor_addresses: Vec<Address> = addrs_hex
        .split(&[',', ' '][..])
        .filter(|s| !s.is_empty())
        .map(|s| Address::parse_checksummed(s, None))
        .collect::<Result<_, _>>()
        .map_err(|_| anyhow::anyhow!("failed to parse ethereum address list"))?;

    Ok(AttestorClientParams {
        min_required_sigs,
        height,
        timestamp,
        attestor_addresses,
    })
}

/// Builds an ABI-encoded `AttestationProof` from attested data and signatures.
#[must_use]
pub fn build_eth_attestor_proof(attested_data: Vec<u8>, signatures: Vec<Vec<u8>>) -> Vec<u8> {
    AttestationProof {
        attestationData: Bytes::from_iter(attested_data),
        signatures: signatures.into_iter().map(Bytes::from).collect(),
    }
    .abi_encode()
}

/// Injects ETH attestor proofs into router messages.
///
/// Constructs ABI-encoded `AttestationProof` from the provided attestations and injects them
/// into the appropriate proof fields of each message type.
///
/// # Arguments
/// - `msgs`: The mutable slice of packet calls to inject proofs into.
/// - `recv_attestation`: Optional attestation for recv packet proofs.
/// - `ack_attestation`: Optional attestation for ack packet proofs.
/// - `timeout_attestation`: Optional attestation for timeout packet proofs.
pub fn inject_eth_attestor_proofs(
    msgs: &mut [PacketCall],
    recv_attestation: Option<AggregatedAttestation>,
    ack_attestation: Option<AggregatedAttestation>,
    timeout_attestation: Option<AggregatedAttestation>,
) {
    let to_proof = |a: AggregatedAttestation| -> Vec<u8> {
        build_eth_attestor_proof(a.attested_data, a.signatures)
    };

    let recv_proof = recv_attestation.map(to_proof);
    let ack_proof = ack_attestation.map(to_proof);
    let timeout_proof = timeout_attestation.map(to_proof);

    for msg in msgs.iter_mut() {
        let proof = match msg {
            PacketCall::Recv(_) => &recv_proof,
            PacketCall::Ack(_) => &ack_proof,
            PacketCall::Timeout(_) => &timeout_proof,
        };
        if let Some(proof) = proof {
            msg.set_proof(Bytes::from_iter(proof.clone()));
        }
    }
}

/// Input data for building an Ethereum multicall transaction.
pub struct MulticallInput {
    /// The ICS26 router address that the multicall calls.
    pub ics26_address: Address,
    /// Attestation data (state + packet attestations).
    pub attestations: AttestationData,
    /// Source chain events (send packets, write acks).
    pub src_events: Vec<EurekaEventWithHeight>,
    /// Target chain events (for timeouts).
    pub target_events: Vec<EurekaEventWithHeight>,
    /// Source client ID.
    pub src_client_id: String,
    /// Destination client ID.
    pub dst_client_id: String,
    /// Source packet sequences to filter (empty means all).
    pub src_packet_seqs: Vec<u64>,
    /// Destination packet sequences to filter (empty means all).
    pub dst_packet_seqs: Vec<u64>,
}

/// Builds an ABI-encoded `IBCMulticall` transaction for the Ethereum IBC router.
///
/// Creates an update client message followed by recv, ack, and timeout messages. The update
/// client message must succeed, while each packet message carries its own proof and is allowed
/// to fail without reverting the rest of the batch.
///
/// # Errors
/// Returns an error if system time cannot be retrieved.
pub fn build_eth_multicall(input: MulticallInput) -> Result<Vec<u8>> {
    let state = input.attestations.state;
    let proof_height = state.height;

    tracing::info!(
        "Received state attestation: {} signatures at height {}",
        state.signatures.len(),
        proof_height
    );

    let update_call = eth_eureka::update_client_calldata(
        input.dst_client_id.clone(),
        build_eth_attestor_proof(state.attested_data, state.signatures).into(),
    );

    let now_since_unix = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?;

    let height = ICS26Height {
        revisionNumber: 0,
        revisionHeight: proof_height,
    };

    let timeout_msgs = eth_eureka::target_events_to_timeout_msgs(
        input.target_events,
        &input.src_client_id,
        &input.dst_client_id,
        &input.dst_packet_seqs,
        &height,
        now_since_unix.as_secs(),
    );
    let recv_and_ack_msgs = eth_eureka::src_events_to_recv_and_ack_msgs(
        input.src_events,
        &input.src_client_id,
        &input.dst_client_id,
        &input.src_packet_seqs,
        &input.dst_packet_seqs,
        &height,
        now_since_unix.as_secs(),
    )?;

    tracing::debug!("Timeout messages: #{}", timeout_msgs.len());
    tracing::debug!("Recv & ack messages: #{}", recv_and_ack_msgs.len());

    let mut all_packet_msgs: Vec<_> = recv_and_ack_msgs.into_iter().chain(timeout_msgs).collect();

    inject_eth_attestor_proofs(
        &mut all_packet_msgs,
        input.attestations.send_attestation,
        input.attestations.ack_attestation,
        input.attestations.timeout_attestation,
    );

    let all_calls = std::iter::once(update_call)
        .chain(all_packet_msgs.iter().map(|call| call.abi_encode().into()));

    Ok(multicall_calldata(input.ics26_address, all_calls, 1))
}

/// Builds attested relay transactions to the ICS26 router of an Ethereum chain.
pub struct EthAttestedTxBuilder<P> {
    aggregator: Aggregator,
    ics26_address: Address,
    provider: P,
}

impl<P: Provider + Clone> EthAttestedTxBuilder<P> {
    /// Creates a builder for the ICS26 router at `ics26_address`.
    pub const fn new(aggregator: Aggregator, ics26_address: Address, provider: P) -> Self {
        Self {
            aggregator,
            ics26_address,
            provider,
        }
    }

    /// Builds the `IBCMulticall` relay transaction for `params`.
    ///
    /// Collects the packets to relay, waits for the aggregator to finalize the relay height,
    /// fetches the attestations and builds the `IBCMulticall` transaction that updates the client
    /// and relays the packets through the router.
    ///
    /// # Errors
    /// Returns an error if attestation fetching or transaction building fails.
    pub async fn relay_events(&self, params: RelayEventsParams) -> Result<Vec<u8>> {
        tracing::info!(
            "Building relay transaction from aggregator for {} source events and {} timeout events",
            params.src_events.len(),
            params.target_events.len()
        );

        let (send_packets, ack_packets, mut relay_height) =
            collect_send_and_ack_packets_with_height(
                &params.src_events,
                &params.src_client_id,
                &params.dst_client_id,
                &params.src_packet_seqs,
                &params.dst_packet_seqs,
            );

        let timeout_packets = collect_timeout_packets(
            &params.target_events,
            &params.src_client_id,
            &params.dst_client_id,
            &params.dst_packet_seqs,
        );

        if timeout_packets.is_empty() {
            tracing::debug!("No timeout packets collected");
        } else {
            let timeout_height = params.timeout_relay_height.ok_or_else(|| {
                anyhow::anyhow!("timeout_relay_height required for timeout packets")
            })?;
            // Use max of src_events height and timeout height
            relay_height = Some(relay_height.map_or(timeout_height, |h| h.max(timeout_height)));
        }

        let relay_height = relay_height.ok_or_else(|| anyhow::anyhow!("No packets collected"))?;
        wait_for_condition(Duration::from_mins(15), Duration::from_secs(1), || async {
            let finalized_height = self.aggregator.get_latest_height().await?;
            Ok(finalized_height >= relay_height)
        })
        .await?;

        let attestations = fetch_attestations(
            &self.aggregator,
            send_packets,
            ack_packets,
            timeout_packets,
            relay_height,
        )
        .await?;

        build_eth_multicall(MulticallInput {
            ics26_address: self.ics26_address,
            attestations,
            src_events: params.src_events,
            target_events: params.target_events,
            src_client_id: params.src_client_id,
            dst_client_id: params.dst_client_id,
            src_packet_seqs: params.src_packet_seqs,
            dst_packet_seqs: params.dst_packet_seqs,
        })
    }

    /// Builds the deployment calldata of an attestor light client bound to the router.
    ///
    /// # Errors
    /// Returns an error if parameter parsing fails.
    pub fn create_client<S: std::hash::BuildHasher>(
        &self,
        parameters: &HashMap<String, String, S>,
    ) -> Result<Vec<u8>> {
        let params = parse_attestor_client_params(parameters)?;

        Ok(
            ibc_eureka_solidity_types::attestation::light_client::deploy_builder(
                self.provider.clone(),
                params.attestor_addresses,
                params.min_required_sigs,
                params.height,
                params.timestamp,
                self.ics26_address,
            )
            .calldata()
            .to_vec(),
        )
    }

    /// Builds the `IBCMulticall` transaction that updates `dst_client_id` to the aggregator's
    /// latest state attestation.
    ///
    /// # Errors
    /// Returns an error if fetching attestation fails.
    pub async fn update_client(&self, dst_client_id: &str) -> Result<Vec<u8>> {
        let current_height = self.aggregator.get_latest_height().await?;
        let state = self
            .aggregator
            .get_state_attestation(current_height)
            .await?;

        Ok(eth_eureka::update_client_multicall_calldata(
            self.ics26_address,
            dst_client_id.to_string(),
            build_eth_attestor_proof(state.attested_data, state.signatures).into(),
        ))
    }
}
