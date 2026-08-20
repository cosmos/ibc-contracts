// SPDX-License-Identifier: Apache-2.0

use alloy::{
    network::Ethereum,
    primitives::{Address, Bytes},
    providers::{Provider, RootProvider},
    sol_types::{SolCall, SolValue},
};
use anyhow::{bail, Context, Result};
use ibc_eureka_solidity_types::{
    dummy::{
        dummy_light_client,
        dummy_light_client_msgs::DummyLightClientMsgs::{
            Height as DummyHeight, Membership, MsgUpdateClient,
        },
    },
    ics26::{
        acknowledgement_commitment,
        router::{multicallCall, routerCalls, routerInstance, updateClientCall},
        IICS02ClientMsgs::Height,
    },
};
use proof_api_lib::utils::{
    eth_eureka::{src_events_to_recv_and_ack_msgs, target_events_to_timeout_msgs},
    RelayEventsParams,
};

pub struct TxBuilder {
    src_provider: RootProvider,
    dst_provider: RootProvider,
    dst_ics26_router: routerInstance<RootProvider, Ethereum>,
}

impl TxBuilder {
    pub fn new(
        src_provider: RootProvider,
        dst_provider: RootProvider,
        dst_ics26_address: Address,
    ) -> Self {
        Self {
            src_provider,
            dst_provider: dst_provider.clone(),
            dst_ics26_router: routerInstance::new(dst_ics26_address, dst_provider),
        }
    }

    pub const fn ics26_router_address(&self) -> &Address {
        self.dst_ics26_router.address()
    }

    pub fn create_client(&self) -> Vec<u8> {
        dummy_light_client::DummyLightClient::deploy_builder(self.dst_provider.clone())
            .calldata()
            .to_vec()
    }

    pub async fn update_client(&self, dst_client_id: &str) -> Result<Vec<u8>> {
        let (latest_source_height, latest_source_timestamp) =
            self.latest_source_consensus_state().await?;
        Ok(update_client_call(
            dst_client_id,
            &dummy_update_msg(latest_source_height, latest_source_timestamp, Vec::new()),
        )
        .abi_encode())
    }

    pub async fn relay_events(&self, params: RelayEventsParams) -> Result<Vec<u8>> {
        let (latest_source_height, latest_source_timestamp) =
            self.latest_source_consensus_state().await?;
        let counterparty = self
            .dst_ics26_router
            .getCounterparty(params.dst_client_id.clone())
            .call()
            .await?;
        let proof_height = Height {
            revisionNumber: 0,
            revisionHeight: latest_source_height,
        };

        let recv_and_ack_msgs = src_events_to_recv_and_ack_msgs(
            params.src_events,
            &params.src_client_id,
            &params.dst_client_id,
            &params.src_packet_seqs,
            &params.dst_packet_seqs,
            &proof_height,
            latest_source_timestamp,
        );
        let timeout_msgs = target_events_to_timeout_msgs(
            params.target_events,
            &params.src_client_id,
            &params.dst_client_id,
            &params.dst_packet_seqs,
            &proof_height,
            latest_source_timestamp,
        );
        let packet_calls: Vec<_> = recv_and_ack_msgs.into_iter().chain(timeout_msgs).collect();
        if packet_calls.is_empty() {
            bail!("no packets collected");
        }

        let memberships = memberships_for_calls(&packet_calls, &counterparty.merklePrefix);
        let update_call = update_client_call(
            &params.dst_client_id,
            &dummy_update_msg(latest_source_height, latest_source_timestamp, memberships),
        )
        .abi_encode();
        let packet_calls = packet_calls.into_iter().map(|msg| match msg {
            routerCalls::timeoutPacket(call) => call.abi_encode(),
            routerCalls::recvPacket(call) => call.abi_encode(),
            routerCalls::ackPacket(call) => call.abi_encode(),
            _ => unreachable!(),
        });
        Ok(multicallCall {
            data: std::iter::once(update_call.into())
                .chain(packet_calls.map(Into::into))
                .collect(),
        }
        .abi_encode())
    }

    async fn latest_source_consensus_state(&self) -> Result<(u64, u64)> {
        let height = self
            .src_provider
            .get_block_number()
            .await
            .context("failed to fetch latest source block number")?;
        let block = self
            .src_provider
            .get_block(height.into())
            .await
            .with_context(|| format!("failed to fetch source block at height {height}"))?
            .with_context(|| format!("source block at height {height} not found"))?;

        Ok((height, block.header.timestamp))
    }
}

fn update_client_call(dst_client_id: &str, msg: &MsgUpdateClient) -> updateClientCall {
    updateClientCall {
        clientId: dst_client_id.to_string(),
        updateMsg: msg.abi_encode().into(),
    }
}

const fn dummy_update_msg(
    revision_height: u64,
    timestamp: u64,
    memberships: Vec<Membership>,
) -> MsgUpdateClient {
    MsgUpdateClient {
        height: DummyHeight {
            revisionNumber: 0,
            revisionHeight: revision_height,
        },
        timestamp,
        memberships,
    }
}

fn memberships_for_calls(calls: &[routerCalls], merkle_prefix: &[Bytes]) -> Vec<Membership> {
    calls
        .iter()
        .filter_map(|call| match call {
            routerCalls::recvPacket(call) => Some(Membership {
                path: prefixed_path(merkle_prefix, &call.msg_.packet.commitment_path()),
                value: call.msg_.packet.commitment().into(),
            }),
            routerCalls::ackPacket(call) => Some(Membership {
                path: prefixed_path(merkle_prefix, &call.msg_.packet.ack_commitment_path()),
                value: acknowledgement_commitment(std::slice::from_ref(&call.msg_.acknowledgement))
                    .into(),
            }),
            routerCalls::timeoutPacket(_) => None,
            _ => unreachable!(),
        })
        .collect()
}

fn prefixed_path(merkle_prefix: &[Bytes], path: &[u8]) -> Vec<Bytes> {
    let (last, prefix) = merkle_prefix
        .split_last()
        .expect("merkle prefix must be non-empty");
    let mut last = last.to_vec();
    last.extend_from_slice(path);
    let mut merkle_prefix = prefix.to_vec();
    merkle_prefix.push(last.into());
    merkle_prefix
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy::primitives::Bytes;
    use ibc_eureka_solidity_types::ics26::{
        router::{ackPacketCall, recvPacketCall, timeoutPacketCall},
        IICS26RouterMsgs::{MsgAckPacket, MsgRecvPacket, MsgTimeoutPacket, Packet},
    };

    fn packet() -> Packet {
        Packet {
            sequence: 7,
            sourceClient: "src".to_string(),
            destClient: "dst".to_string(),
            timeoutTimestamp: u64::MAX,
            payloads: Vec::new(),
        }
    }

    #[test]
    fn prefixed_path_appends_to_last_prefix_element() {
        let path = packet().commitment_path();
        let got = prefixed_path(&[Bytes::from_static(b"ibc")], &path);
        let mut expected = b"ibc".to_vec();
        expected.extend_from_slice(&path);
        assert_eq!(got, vec![Bytes::from(expected)]);
    }

    #[test]
    fn membership_generation_matches_packet_calls() {
        let pkt = packet();
        let ack = Bytes::from_static(b"ack");
        let calls = vec![
            routerCalls::recvPacket(recvPacketCall {
                msg_: MsgRecvPacket {
                    packet: pkt.clone(),
                    proofHeight: Height {
                        revisionNumber: 0,
                        revisionHeight: 1,
                    },
                    proofCommitment: Bytes::new(),
                },
            }),
            routerCalls::ackPacket(ackPacketCall {
                msg_: MsgAckPacket {
                    packet: pkt.clone(),
                    acknowledgement: ack.clone(),
                    proofHeight: Height {
                        revisionNumber: 0,
                        revisionHeight: 1,
                    },
                    proofAcked: Bytes::new(),
                },
            }),
            routerCalls::timeoutPacket(timeoutPacketCall {
                msg_: MsgTimeoutPacket {
                    packet: pkt.clone(),
                    proofHeight: Height {
                        revisionNumber: 0,
                        revisionHeight: 1,
                    },
                    proofTimeout: Bytes::new(),
                },
            }),
        ];
        let memberships = memberships_for_calls(&calls, &[Bytes::from_static(b"ibc")]);
        assert_eq!(memberships.len(), 2);
        assert_eq!(memberships[0].value, Bytes::from(pkt.commitment()));
        assert_eq!(
            memberships[1].value,
            Bytes::from(acknowledgement_commitment(&[ack]))
        );
    }
}
