// SPDX-License-Identifier: Apache-2.0

//! Proof API utilities for `solidity-ibc-eureka` chains.

use crate::events::{EurekaEvent, EurekaEventWithHeight};
use alloy::{primitives::Bytes, sol_types::SolValue};
use anyhow::Result;
use futures::future;
use ibc_eureka_solidity_types::{
    ics26::{
        router::{ackPacketCall, recvPacketCall, routerCalls},
        IICS02ClientMsgs::Height,
        IICS26RouterMsgs::{MsgAckPacket, MsgRecvPacket, MsgTimeoutPacket},
    },
    msgs::{
        IICS07TendermintMsgs::ClientState,
        IMembershipMsgs::{KVPair, MembershipProof, SP1MembershipAndUpdateClientProof},
        ISP1Msgs::SP1Proof,
    },
};
use ibc_eureka_utils::{light_block::LightBlockExt, rpc::TendermintRpcExt};
use sp1_ics07_tendermint_prover::{
    programs::UpdateClientAndMembershipProgram,
    prover::{SP1ICS07TendermintProver, Sp1Prover},
};
use sp1_sdk::HashableKey;
use tendermint_light_client_verifier::types::LightBlock;
use tendermint_rpc::HttpClient;

/// Converts a list of [`EurekaEvent`]s to a list of [`routerCalls::timeoutPacket`]s with empty
/// proofs.
///
/// # Arguments
/// - `target_events`: The list of target events to convert.
/// - `src_client_id`: The source client ID.
/// - `dst_client_id`: The destination client ID.
/// - `dst_packet_seqs`: The list of dest packet sequences to filter by. If empty, no filtering.
/// - `target_height`: The target height for the proofs.
/// - `now`: The current time.
#[must_use]
pub fn target_events_to_timeout_msgs(
    target_events: Vec<EurekaEventWithHeight>,
    src_client_id: &str,
    dst_client_id: &str,
    dst_packet_seqs: &[u64],
    target_height: &Height,
    now: u64,
) -> Vec<routerCalls> {
    target_events
        .into_iter()
        .filter_map(|e| match e.event {
            EurekaEvent::SendPacket(packet) => (now >= packet.timeoutTimestamp
                && packet.sourceClient == dst_client_id
                && packet.destClient == src_client_id
                && (dst_packet_seqs.is_empty() || dst_packet_seqs.contains(&packet.sequence)))
            .then_some(routerCalls::timeoutPacket(
                ibc_eureka_solidity_types::ics26::router::timeoutPacketCall {
                    msg_: MsgTimeoutPacket {
                        packet,
                        proofHeight: target_height.clone(),
                        proofTimeout: Bytes::default(),
                    },
                },
            )),
            EurekaEvent::WriteAcknowledgement(..) => None,
        })
        .collect()
}

/// Converts a list of [`EurekaEvent`]s to a list of [`routerCalls::recvPacket`]s and
/// [`routerCalls::ackPacket`]s with empty proofs.
///
/// # Arguments
/// - `src_events`: The list of source events to convert.
/// - `src_client_id`: The source client ID.
/// - `dst_client_id`: The destination client ID.
/// - `src_packet_seqs`: The list of source packet sequences to filter by. If empty, no filtering.
/// - `dst_packet_seqs`: The list of dest packet sequences to filter by. If empty, no filtering.
/// - `target_height`: The target height for the proofs.
/// - `now`: The current time.
#[must_use]
pub fn src_events_to_recv_and_ack_msgs(
    src_events: Vec<EurekaEventWithHeight>,
    src_client_id: &str,
    dst_client_id: &str,
    src_packet_seqs: &[u64],
    dst_packet_seqs: &[u64],
    target_height: &Height,
    now: u64,
) -> Vec<routerCalls> {
    src_events
        .into_iter()
        .filter_map(|e| match e.event {
            EurekaEvent::SendPacket(packet) => (packet.timeoutTimestamp > now
                && packet.sourceClient == src_client_id
                && packet.destClient == dst_client_id
                && (src_packet_seqs.is_empty() || src_packet_seqs.contains(&packet.sequence)))
            .then_some(routerCalls::recvPacket(recvPacketCall {
                msg_: MsgRecvPacket {
                    packet,
                    proofHeight: target_height.clone(),
                    proofCommitment: Bytes::default(),
                },
            })),
            EurekaEvent::WriteAcknowledgement(packet, acks) => {
                (packet.sourceClient == dst_client_id
                    && packet.destClient == src_client_id
                    && (dst_packet_seqs.is_empty() || dst_packet_seqs.contains(&packet.sequence)))
                .then_some(routerCalls::ackPacket(ackPacketCall {
                    msg_: MsgAckPacket {
                        packet,
                        acknowledgement: acks[0].clone(), // TODO: handle multiple acks (#93)
                        proofHeight: target_height.clone(),
                        proofAcked: Bytes::default(),
                    },
                }))
            }
        })
        .collect()
}

/// Generates and injects an SP1 proof into the first message in `msgs`.
/// # Errors
/// Returns an error if the sp1 proof cannot be generated.
pub async fn inject_sp1_proof(
    sp1_prover: &Sp1Prover,
    uc_and_mem_program: &UpdateClientAndMembershipProgram,
    msgs: &mut [routerCalls],
    tm_client: &HttpClient,
    target_light_block: LightBlock,
    client_state: ClientState,
    now: u128,
) -> Result<()> {
    let target_height = target_light_block.height().value();

    let ibc_paths = msgs
        .iter()
        .map(|msg| match msg {
            routerCalls::timeoutPacket(call) => call.msg_.packet.receipt_commitment_path(),
            routerCalls::recvPacket(call) => call.msg_.packet.commitment_path(),
            routerCalls::ackPacket(call) => call.msg_.packet.ack_commitment_path(),
            _ => unreachable!(),
        })
        .map(|path| vec![b"ibc".into(), path]);

    let kv_proofs: Vec<(_, _)> = future::try_join_all(ibc_paths.into_iter().map(|path| async {
        let (value, proof) = tm_client.prove_path(&path, target_height).await?;
        let kv_pair = KVPair {
            path: path.into_iter().map(Into::into).collect(),
            value: value.into(),
        };
        anyhow::Ok((kv_pair, proof))
    }))
    .await?;

    let trusted_light_block = tm_client
        .get_light_block(Some(client_state.latestHeight.revisionHeight))
        .await?;

    // Get the proposed header from the target light block.
    let proposed_header = target_light_block.into_header(&trusted_light_block);

    let uc_and_mem_prover =
        SP1ICS07TendermintProver::new(client_state.zkAlgorithm, sp1_prover, uc_and_mem_program)
            .await;

    let uc_and_mem_proof = uc_and_mem_prover
        .generate_proof(
            &client_state,
            &trusted_light_block.to_consensus_state().into(),
            &proposed_header,
            now,
            kv_proofs,
        )
        .await;

    let sp1_proof = MembershipProof::from(SP1MembershipAndUpdateClientProof {
        sp1Proof: SP1Proof::new(
            &uc_and_mem_prover.vkey.bytes32(),
            uc_and_mem_proof.bytes(),
            uc_and_mem_proof.public_values.to_vec(),
        ),
    });

    // inject proof
    match msgs.first_mut() {
        Some(routerCalls::timeoutPacket(ref mut call)) => {
            *call.msg_.proofTimeout = sp1_proof.abi_encode().into();
        }
        Some(routerCalls::recvPacket(ref mut call)) => {
            *call.msg_.proofCommitment = sp1_proof.abi_encode().into();
        }
        Some(routerCalls::ackPacket(ref mut call)) => {
            *call.msg_.proofAcked = sp1_proof.abi_encode().into();
        }
        _ => unreachable!(),
    }

    Ok(())
}
