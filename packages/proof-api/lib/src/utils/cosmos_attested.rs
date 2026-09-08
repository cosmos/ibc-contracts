// SPDX-License-Identifier: Apache-2.0

//! Utilities for attested relays targeting Cosmos SDK chains.

use std::time::Duration;

use crate::aggregator::{rpc::AggregatedAttestation, Aggregator};
use crate::utils::attestor::{
    collect_send_and_ack_packets_with_height, collect_timeout_packets, fetch_attestations,
};
use crate::utils::{cosmos, wait_for_condition, RelayEventsParams};
use anyhow::Result;
use ibc_proto_eureka::cosmos::tx::v1beta1::TxBody;
use ibc_proto_eureka::{
    google::protobuf::Any,
    ibc::{
        core::{
            channel::v2::{MsgAcknowledgement, MsgRecvPacket, MsgTimeout},
            client::v1::{Height, MsgCreateClient, MsgUpdateClient},
        },
        lightclients::attestations::v1::{
            AttestationProof, ClientState as IbcGoAttestorClientState,
            ConsensusState as IbcGoAttestorConsensusState,
        },
    },
};
use prost::Message;

/// Parameter key for attestor addresses.
pub const PARAM_ATTESTOR_ADDRESSES: &str = "attestor_addresses";
/// Parameter key for minimum required signatures.
pub const PARAM_MIN_REQUIRED_SIGS: &str = "min_required_sigs";
/// Parameter key for height.
pub const PARAM_HEIGHT: &str = "height";
/// Parameter key for timestamp.
pub const PARAM_TIMESTAMP: &str = "timestamp";

/// Nanoseconds per second for timestamp conversion.
pub const NANOS_PER_SECOND: u64 = 1_000_000_000;

/// Build the client message `Any` for `MsgUpdateClient`.
///
/// # Errors
/// Returns an error if message encoding fails.
fn build_client_message(
    attested_data: Vec<u8>,
    signatures: Vec<Vec<u8>>,
    _height: u64,
    _timestamp: u64,
) -> Result<Any> {
    let attestation_proof = AttestationProof {
        attestation_data: attested_data,
        signatures,
    };
    Any::from_msg(&attestation_proof).map_err(Into::into)
}

/// Build membership proof bytes from attested data and signatures.
///
/// # Errors
/// Returns an error if proof serialization fails.
fn build_membership_proof(attested_data: Vec<u8>, signatures: Vec<Vec<u8>>) -> Vec<u8> {
    AttestationProof {
        attestation_data: attested_data,
        signatures,
    }
    .encode_to_vec()
}

/// Builds an `MsgUpdateClient` using the specified proof builder.
///
/// # Errors
/// Returns an error if message encoding fails.
pub fn build_update_client_message(
    client_id: &str,
    signer: &str,
    attested_data: Vec<u8>,
    signatures: Vec<Vec<u8>>,
    height: u64,
    timestamp: u64,
) -> Result<MsgUpdateClient> {
    let client_message = build_client_message(attested_data, signatures, height, timestamp)?;
    Ok(MsgUpdateClient {
        client_id: client_id.to_string(),
        client_message: Some(client_message),
        signer: signer.to_string(),
    })
}

/// Injects attestor proofs into Cosmos SDK messages using the specified proof builder.
///
/// # Errors
/// Returns an error if proof serialization fails.
#[allow(clippy::too_many_arguments)]
pub fn inject_cosmos_attestor_proofs(
    recv_msgs: &mut [MsgRecvPacket],
    ack_msgs: &mut [MsgAcknowledgement],
    timeout_msgs: &mut [MsgTimeout],
    send_attestation: Option<AggregatedAttestation>,
    ack_attestation: Option<AggregatedAttestation>,
    receipt_attestation: Option<AggregatedAttestation>,
    proof_height: u64,
) -> Result<()> {
    if let Some(send_attest) = send_attestation {
        let proof = build_membership_proof(send_attest.attested_data, send_attest.signatures);
        for msg in recv_msgs.iter_mut() {
            msg.proof_commitment.clone_from(&proof);
            msg.proof_height = Some(Height {
                revision_height: proof_height,
                ..Default::default()
            });
        }
    }

    if let Some(ack_attest) = ack_attestation {
        let proof = build_membership_proof(ack_attest.attested_data, ack_attest.signatures);
        for msg in ack_msgs.iter_mut() {
            msg.proof_acked.clone_from(&proof);
            msg.proof_height = Some(Height {
                revision_height: proof_height,
                ..Default::default()
            });
        }
    }

    if let Some(receipt_attest) = receipt_attestation {
        let proof = build_membership_proof(receipt_attest.attested_data, receipt_attest.signatures);
        for msg in timeout_msgs.iter_mut() {
            msg.proof_unreceived.clone_from(&proof);
            msg.proof_height = Some(Height {
                revision_height: proof_height,
                ..Default::default()
            });
        }
    }

    Ok(())
}

/// Builds a `MsgCreateClient` for native ibc-go attestor light client.
///
/// # Arguments
/// * `addrs_hex` - Comma or space separated attestor addresses (hex strings)
/// * `min_required_sigs` - Minimum number of signatures required
/// * `height` - Initial height
/// * `timestamp` - Initial timestamp in seconds
/// * `signer_address` - Cosmos signer address
///
/// # Errors
/// Returns an error if message encoding fails.
pub fn build_ibc_go_attestor_create_client_msg(
    addrs_hex: &str,
    min_required_sigs: u32,
    height: u64,
    timestamp: u64,
    signer_address: &str,
) -> Result<MsgCreateClient> {
    tracing::info!("Creating ibc-go attestor light client at height {height}");

    let attestor_addresses: Vec<String> = addrs_hex
        .split(&[',', ' '][..])
        .filter(|s| !s.is_empty())
        .map(ToString::to_string)
        .collect();

    let client_state = IbcGoAttestorClientState {
        attestor_addresses,
        min_required_sigs,
        latest_height: height,
        is_frozen: false,
    };

    let consensus_state = IbcGoAttestorConsensusState {
        timestamp: timestamp.saturating_mul(NANOS_PER_SECOND),
    };

    Ok(MsgCreateClient {
        client_state: Some(Any::from_msg(&client_state)?),
        consensus_state: Some(Any::from_msg(&consensus_state)?),
        signer: signer_address.to_string(),
    })
}

/// Builds an update client transaction for an attested light client.
///
/// Fetches the latest state attestation from the aggregator and builds
/// the `MsgUpdateClient`. Automatically selects the appropriate proof builder
/// based on the client ID prefix.
///
/// # Errors
/// Returns an error if fetching attestation or message encoding fails.
///
/// # Panics
/// Panics if the state attestation does not contain a timestamp.
pub async fn build_attestor_update_client_tx(
    aggregator: &Aggregator,
    dst_client_id: &str,
    signer_address: &str,
) -> Result<Vec<u8>> {
    tracing::info!("Updating attested light client: {}", dst_client_id);

    let current_height = aggregator.get_latest_height().await?;

    tracing::info!(
        "Fetching state attestation for client {} at height {}",
        dst_client_id,
        current_height
    );

    let state = aggregator.get_state_attestation(current_height).await?;

    tracing::info!(
        "Received state attestation: {} signatures at height {}",
        state.signatures.len(),
        state.height
    );

    let timestamp = state.timestamp.expect("state attestation must contain ts");
    let update_msg = build_update_client_message(
        dst_client_id,
        signer_address,
        state.attested_data,
        state.signatures,
        state.height,
        timestamp,
    )?;

    tracing::info!(
        "Built MsgUpdateClient for client {} at height {}",
        dst_client_id,
        state.height
    );

    let tx_body = TxBody {
        messages: vec![Any::from_msg(&update_msg)?],
        ..Default::default()
    };

    Ok(tx_body.encode_to_vec())
}

/// Builds an attestor `MsgCreateClient` from parameters.
///
/// # Errors
/// Returns an error if required parameters are missing or message encoding fails.
pub fn build_attestor_create_client_msg<S: std::hash::BuildHasher>(
    parameters: &std::collections::HashMap<String, String, S>,
    signer_address: &str,
) -> Result<MsgCreateClient> {
    let height: u64 = parameters
        .get(PARAM_HEIGHT)
        .ok_or_else(|| anyhow::anyhow!(format!("Missing `{PARAM_HEIGHT}` parameter")))?
        .parse()?;

    let timestamp: u64 = parameters
        .get(PARAM_TIMESTAMP)
        .ok_or_else(|| anyhow::anyhow!(format!("Missing `{PARAM_TIMESTAMP}` parameter")))?
        .parse()?;

    let addrs_hex = parameters.get(PARAM_ATTESTOR_ADDRESSES).ok_or_else(|| {
        anyhow::anyhow!(format!("Missing `{PARAM_ATTESTOR_ADDRESSES}` parameter"))
    })?;

    let min_required_sigs: u32 = parameters
        .get(PARAM_MIN_REQUIRED_SIGS)
        .ok_or_else(|| anyhow::anyhow!(format!("Missing `{PARAM_MIN_REQUIRED_SIGS}` parameter")))?
        .parse()?;

    build_ibc_go_attestor_create_client_msg(
        addrs_hex,
        min_required_sigs,
        height,
        timestamp,
        signer_address,
    )
}

/// Builds an attestor create client transaction.
///
/// Wraps `build_attestor_create_client_msg` result in a `TxBody` and encodes it.
///
/// # Errors
/// Returns an error if required parameters are missing or message encoding fails.
pub fn build_attestor_create_client_tx<S: std::hash::BuildHasher>(
    parameters: &std::collections::HashMap<String, String, S>,
    signer_address: &str,
) -> Result<Vec<u8>> {
    let msg = build_attestor_create_client_msg(parameters, signer_address)?;

    Ok(TxBody {
        messages: vec![Any::from_msg(&msg)?],
        ..Default::default()
    }
    .encode_to_vec())
}

/// Builds a relay events transaction for an attested light client.
///
/// This function handles the complete flow of:
/// 1. Collecting and encoding packets from source and target events
/// 2. Fetching attestations from the aggregator
/// 3. Building IBC messages (recv, ack, timeout)
/// 4. Injecting proofs into the messages
/// 5. Building the final transaction
///
/// Automatically selects the appropriate proof builder based on the client ID prefix.
///
/// # Errors
/// Returns an error if attestation fetching or message encoding fails.
///
/// # Panics
/// Panics if the state attestation does not contain a timestamp.
pub async fn build_attestor_relay_events_tx(
    aggregator: &Aggregator,
    params: RelayEventsParams,
    signer_address: &str,
) -> Result<Vec<u8>> {
    build_attestor_relay_events_tx_with(aggregator, params, signer_address).await
}

#[allow(clippy::too_many_lines)]
async fn build_attestor_relay_events_tx_with(
    aggregator: &Aggregator,
    params: RelayEventsParams,
    signer_address: &str,
) -> Result<Vec<u8>> {
    tracing::info!(
        "Building attested relay transaction for {} source events and {} target events",
        params.src_events.len(),
        params.target_events.len()
    );

    let (send_packets, ack_packets, mut relay_height) = collect_send_and_ack_packets_with_height(
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
        let timeout_height = params
            .timeout_relay_height
            .ok_or_else(|| anyhow::anyhow!("timeout_relay_height required for timeout packets"))?;
        // Use max of src_events height and timeout height
        relay_height = Some(relay_height.map_or(timeout_height, |h| h.max(timeout_height)));
    }

    let relay_height = relay_height.ok_or_else(|| anyhow::anyhow!("No packets collected"))?;

    wait_for_condition(Duration::from_mins(15), Duration::from_secs(1), || async {
        let finalized_height = aggregator.get_latest_height().await?;
        Ok(finalized_height >= relay_height)
    })
    .await?;

    let attestations = fetch_attestations(
        aggregator,
        send_packets,
        ack_packets,
        timeout_packets,
        relay_height,
    )
    .await?;

    let state = attestations.state;
    let proof_height = state.height;

    tracing::info!(
        "Received state attestation: {} signatures at height {}",
        state.signatures.len(),
        proof_height
    );

    let timestamp = state.timestamp.expect("state attestation must contain ts");
    let update_msg = build_update_client_message(
        &params.dst_client_id,
        signer_address,
        state.attested_data,
        state.signatures,
        state.height,
        timestamp,
    )?;

    let mut timeout_msgs = cosmos::target_events_to_timeout_msgs(
        params.target_events,
        &params.src_client_id,
        &params.dst_client_id,
        &params.dst_packet_seqs,
        signer_address,
        timestamp,
    );

    let (mut recv_msgs, mut ack_msgs) = cosmos::src_events_to_recv_and_ack_msgs(
        params.src_events,
        &params.src_client_id,
        &params.dst_client_id,
        &params.src_packet_seqs,
        &params.dst_packet_seqs,
        signer_address,
        timestamp,
    );

    tracing::debug!("Timeout messages: #{}", timeout_msgs.len());
    tracing::debug!("Recv messages: #{}", recv_msgs.len());
    tracing::debug!("Ack messages: #{}", ack_msgs.len());

    inject_cosmos_attestor_proofs(
        &mut recv_msgs,
        &mut ack_msgs,
        &mut timeout_msgs,
        attestations.send_attestation,
        attestations.ack_attestation,
        attestations.timeout_attestation,
        proof_height,
    )?;

    let all_msgs = std::iter::once(Any::from_msg(&update_msg))
        .chain(recv_msgs.into_iter().map(|m| Any::from_msg(&m)))
        .chain(timeout_msgs.into_iter().map(|m| Any::from_msg(&m)))
        .chain(ack_msgs.into_iter().map(|m| Any::from_msg(&m)))
        .collect::<Result<Vec<_>, _>>()?;

    tracing::debug!("Total messages: #{}", all_msgs.len());

    let tx_body = TxBody {
        messages: all_msgs,
        ..Default::default()
    };

    Ok(tx_body.encode_to_vec())
}
