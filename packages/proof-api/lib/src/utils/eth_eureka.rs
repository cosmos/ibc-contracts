// SPDX-License-Identifier: Apache-2.0

//! Proof API utilities for `ibc-contracts` chains.

use crate::events::{EurekaEvent, EurekaEventWithHeight};
use alloy::{
    primitives::{Address, Bytes},
    sol_types::{SolCall, SolValue},
};
use anyhow::{anyhow, Result};
use futures::future;
use ibc_eureka_solidity_types::{
    ibc_multicall::multicall_calldata,
    ics26::{
        router::{ackPacketCall, recvPacketCall, timeoutPacketCall, updateClientCall},
        IICS02ClientMsgs::Height,
        IICS26RouterMsgs::{MsgAckPacket, MsgRecvPacket, MsgTimeoutPacket, Packet},
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

/// A packet call to the ICS26 router.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PacketCall {
    /// `recvPacket`, proven by the packet commitment on the source chain.
    Recv(recvPacketCall),
    /// `ackPacket`, proven by the acknowledgement commitment on the source chain.
    Ack(ackPacketCall),
    /// `timeoutPacket`, proven by the absence of a packet receipt on the source chain.
    Timeout(timeoutPacketCall),
}

impl PacketCall {
    /// The packet this call relays.
    #[must_use]
    pub const fn packet(&self) -> &Packet {
        match self {
            Self::Recv(call) => &call.msg_.packet,
            Self::Ack(call) => &call.msg_.packet,
            Self::Timeout(call) => &call.msg_.packet,
        }
    }

    /// The source chain commitment path proven by this call.
    #[must_use]
    pub fn commitment_path(&self) -> Vec<u8> {
        match self {
            Self::Recv(call) => call.msg_.packet.commitment_path(),
            Self::Ack(call) => call.msg_.packet.ack_commitment_path(),
            Self::Timeout(call) => call.msg_.packet.receipt_commitment_path(),
        }
    }

    /// Sets the proof of [`Self::commitment_path`].
    pub fn set_proof(&mut self, proof: Bytes) {
        match self {
            Self::Recv(call) => call.msg_.proofCommitment = proof,
            Self::Ack(call) => call.msg_.proofAcked = proof,
            Self::Timeout(call) => call.msg_.proofTimeout = proof,
        }
    }

    /// The ABI-encoded router calldata.
    #[must_use]
    pub fn abi_encode(&self) -> Vec<u8> {
        match self {
            Self::Recv(call) => call.abi_encode(),
            Self::Ack(call) => call.abi_encode(),
            Self::Timeout(call) => call.abi_encode(),
        }
    }
}

/// Encodes the router `updateClient` call for `client_id`.
#[must_use]
pub fn update_client_calldata(client_id: String, update_msg: Bytes) -> Bytes {
    updateClientCall {
        clientId: client_id,
        updateMsg: update_msg,
    }
    .abi_encode()
    .into()
}

/// Encodes an `IBCMulticall` call that only updates `client_id` on the router at `ics26_address`.
#[must_use]
pub fn update_client_multicall_calldata(
    ics26_address: Address,
    client_id: String,
    update_msg: Bytes,
) -> Vec<u8> {
    multicall_calldata(
        ics26_address,
        [update_client_calldata(client_id, update_msg)],
        1,
    )
}

/// Converts a list of [`EurekaEvent`]s to a list of [`PacketCall::Timeout`]s with empty proofs.
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
) -> Vec<PacketCall> {
    target_events
        .into_iter()
        .filter_map(|e| match e.event {
            EurekaEvent::SendPacket(packet) => (now >= packet.timeoutTimestamp
                && packet.sourceClient == dst_client_id
                && packet.destClient == src_client_id
                && (dst_packet_seqs.is_empty() || dst_packet_seqs.contains(&packet.sequence)))
            .then_some(PacketCall::Timeout(timeoutPacketCall {
                msg_: MsgTimeoutPacket {
                    packet,
                    proofHeight: target_height.clone(),
                    proofTimeout: Bytes::default(),
                },
            })),
            EurekaEvent::WriteAcknowledgement(..) => None,
        })
        .collect()
}

/// Converts a list of [`EurekaEvent`]s to a list of [`PacketCall::Recv`]s and
/// [`PacketCall::Ack`]s with empty proofs.
///
/// # Arguments
/// - `src_events`: The list of source events to convert.
/// - `src_client_id`: The source client ID.
/// - `dst_client_id`: The destination client ID.
/// - `src_packet_seqs`: The list of source packet sequences to filter by. If empty, no filtering.
/// - `dst_packet_seqs`: The list of dest packet sequences to filter by. If empty, no filtering.
/// - `target_height`: The target height for the proofs.
/// - `now`: The current time.
///
/// # Errors
/// Returns an error when a selected acknowledgement event contains no acknowledgements.
pub fn src_events_to_recv_and_ack_msgs(
    src_events: Vec<EurekaEventWithHeight>,
    src_client_id: &str,
    dst_client_id: &str,
    src_packet_seqs: &[u64],
    dst_packet_seqs: &[u64],
    target_height: &Height,
    now: u64,
) -> Result<Vec<PacketCall>> {
    let mut calls = Vec::new();
    for event in src_events {
        match event.event {
            EurekaEvent::SendPacket(packet)
                if packet.timeoutTimestamp > now
                    && packet.sourceClient == src_client_id
                    && packet.destClient == dst_client_id
                    && (src_packet_seqs.is_empty()
                        || src_packet_seqs.contains(&packet.sequence)) =>
            {
                calls.push(PacketCall::Recv(recvPacketCall {
                    msg_: MsgRecvPacket {
                        packet,
                        proofHeight: target_height.clone(),
                        proofCommitment: Bytes::default(),
                    },
                }));
            }
            EurekaEvent::WriteAcknowledgement(packet, acks)
                if packet.sourceClient == dst_client_id
                    && packet.destClient == src_client_id
                    && (dst_packet_seqs.is_empty()
                        || dst_packet_seqs.contains(&packet.sequence)) =>
            {
                let acknowledgement = acks.into_iter().next().ok_or_else(|| {
                    anyhow!(
                        "write acknowledgement event for packet {} has no acknowledgements",
                        packet.sequence
                    )
                })?;
                calls.push(PacketCall::Ack(ackPacketCall {
                    msg_: MsgAckPacket {
                        packet,
                        acknowledgement, // TODO: handle multiple acks (#93)
                        proofHeight: target_height.clone(),
                        proofAcked: Bytes::default(),
                    },
                }));
            }
            _ => {}
        }
    }
    Ok(calls)
}

/// Generates and injects an SP1 proof into the first message in `msgs`.
/// # Errors
/// Returns an error if the sp1 proof cannot be generated.
pub async fn inject_sp1_proof(
    sp1_prover: &Sp1Prover,
    uc_and_mem_program: &UpdateClientAndMembershipProgram,
    msgs: &mut [PacketCall],
    tm_client: &HttpClient,
    target_light_block: LightBlock,
    client_state: ClientState,
    now: u128,
) -> Result<()> {
    let target_height = target_light_block.height().value();

    let ibc_paths = msgs
        .iter()
        .map(|msg| vec![b"ibc".into(), msg.commitment_path()]);

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
    drop(uc_and_mem_prover);

    // inject proof
    if let Some(msg) = msgs.first_mut() {
        msg.set_proof(sp1_proof.abi_encode().into());
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::sol_types::SolInterface;
    use ibc_eureka_solidity_types::ics26::{router::routerCalls, IICS26RouterMsgs::Payload};

    fn packet() -> Packet {
        Packet {
            sequence: 42,
            sourceClient: "client-0".to_string(),
            destClient: "client-1".to_string(),
            timeoutTimestamp: 1_700_000_000,
            payloads: vec![Payload {
                sourcePort: "transfer".to_string(),
                destPort: "transfer".to_string(),
                version: "ics20-1".to_string(),
                encoding: "application/x-solidity-abi".to_string(),
                value: Bytes::from_static(b"payload value"),
            }],
        }
    }

    fn height() -> Height {
        Height {
            revisionNumber: 1,
            revisionHeight: 1234,
        }
    }

    /// Every packet call kind with a non-empty proof, paired with the router call it replaces.
    fn packet_calls() -> Vec<(PacketCall, routerCalls)> {
        let proof = Bytes::from_static(b"proof bytes");
        let recv = recvPacketCall {
            msg_: MsgRecvPacket {
                packet: packet(),
                proofHeight: height(),
                proofCommitment: proof.clone(),
            },
        };
        let ack = ackPacketCall {
            msg_: MsgAckPacket {
                packet: packet(),
                acknowledgement: Bytes::from_static(b"ack"),
                proofHeight: height(),
                proofAcked: proof.clone(),
            },
        };
        let timeout = timeoutPacketCall {
            msg_: MsgTimeoutPacket {
                packet: packet(),
                proofHeight: height(),
                proofTimeout: proof,
            },
        };
        vec![
            (
                PacketCall::Recv(recv.clone()),
                routerCalls::recvPacket(recv),
            ),
            (PacketCall::Ack(ack.clone()), routerCalls::ackPacket(ack)),
            (
                PacketCall::Timeout(timeout.clone()),
                routerCalls::timeoutPacket(timeout),
            ),
        ]
    }

    /// `set_proof` replaces the proof of the call's own kind: clearing it changes the calldata,
    /// and setting the original proof back restores the router call.
    #[test]
    fn set_proof_replaces_the_calls_proof() {
        for (mut call, router_call) in packet_calls() {
            call.set_proof(Bytes::new());
            assert_ne!(call.abi_encode(), router_call.abi_encode());
            call.set_proof(Bytes::from_static(b"proof bytes"));
            assert_eq!(call.abi_encode(), router_call.abi_encode());
        }
    }

    #[test]
    fn update_client_multicall_matches_router_call_encoding() {
        let ics26_address = Address::repeat_byte(0x26);
        let update_msg = Bytes::from_static(b"update msg");
        let expected = multicall_calldata(
            ics26_address,
            [updateClientCall {
                clientId: "client-1".to_string(),
                updateMsg: update_msg.clone(),
            }
            .abi_encode()
            .into()],
            1,
        );

        assert_eq!(
            update_client_multicall_calldata(ics26_address, "client-1".to_string(), update_msg),
            expected
        );
    }

    #[test]
    fn rejects_selected_event_without_acknowledgements() {
        let event = EurekaEventWithHeight {
            event: EurekaEvent::WriteAcknowledgement(
                Packet {
                    sequence: 7,
                    sourceClient: "dst".to_string(),
                    destClient: "src".to_string(),
                    timeoutTimestamp: u64::MAX,
                    payloads: vec![],
                },
                vec![],
            ),
            height: 1,
        };

        let error = src_events_to_recv_and_ack_msgs(
            vec![event],
            "src",
            "dst",
            &[],
            &[],
            &Height {
                revisionNumber: 0,
                revisionHeight: 1,
            },
            0,
        )
        .unwrap_err();
        assert!(error.to_string().contains("has no acknowledgements"));
    }
}
