// SPDX-License-Identifier: Apache-2.0

//! This module defines [`TxBuilder`] which is responsible for building transactions to be sent to
//! the Cosmos SDK chain from events received from another Cosmos SDK chain.

use std::{collections::HashMap, str::FromStr};

use anyhow::Result;
use ibc_core_host_types::identifiers::ChainId;
use ibc_eureka_utils::{light_block::LightBlockExt, rpc::TendermintRpcExt};
use ibc_proto_eureka::{
    cosmos::tx::v1beta1::TxBody,
    google::protobuf::{Any, Duration},
    ibc::{
        core::client::v1::{Height, MsgCreateClient, MsgUpdateClient},
        lightclients::tendermint::v1::ClientState,
    },
};
use ics23;
use prost::Message;
use tendermint_rpc::HttpClient;

use proof_api_lib::{
    chain::CosmosSdk, events::EurekaEventWithHeight,
    tendermint_client::build_tendermint_client_state, tx_builder::TxBuilderService, utils::cosmos,
};

/// The `TxBuilder` produces txs to [`CosmosSdk`] based on events from [`CosmosSdk`].
#[allow(dead_code)]
pub struct TxBuilder {
    /// The HTTP client for the source chain.
    pub source_tm_client: HttpClient,
    /// The HTTP client for the target chain.
    pub target_tm_client: HttpClient,
    /// The signer address for the Cosmos messages.
    pub signer_address: String,
}

impl TxBuilder {
    /// Creates a new `TxBuilder`.
    #[must_use]
    pub const fn new(
        source_tm_client: HttpClient,
        target_tm_client: HttpClient,
        signer_address: String,
    ) -> Self {
        Self {
            source_tm_client,
            target_tm_client,
            signer_address,
        }
    }
}

#[async_trait::async_trait]
impl TxBuilderService<CosmosSdk, CosmosSdk> for TxBuilder {
    #[tracing::instrument(skip_all, err(Debug))]
    async fn relay_events(
        &self,
        src_events: Vec<EurekaEventWithHeight>,
        target_events: Vec<EurekaEventWithHeight>,
        src_client_id: String,
        dst_client_id: String,
        src_packet_seqs: Vec<u64>,
        dst_packet_seqs: Vec<u64>,
    ) -> Result<Vec<u8>> {
        let client_state = ClientState::decode(
            self.target_tm_client
                .client_state(dst_client_id.clone())
                .await?
                .value
                .as_slice(),
        )?;

        let target_light_block = self.source_tm_client.get_light_block(None).await?;
        let revision_height = target_light_block.height().value();
        let revision_number = client_state
            .latest_height
            .ok_or_else(|| anyhow::anyhow!("No latest height found"))?
            .revision_number;

        let now_since_unix = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?;

        let mut timeout_msgs = cosmos::target_events_to_timeout_msgs(
            target_events,
            &src_client_id,
            &dst_client_id,
            &dst_packet_seqs,
            &self.signer_address,
            now_since_unix.as_secs(),
        );

        let (mut recv_msgs, mut ack_msgs) = cosmos::src_events_to_recv_and_ack_msgs(
            src_events,
            &src_client_id,
            &dst_client_id,
            &src_packet_seqs,
            &dst_packet_seqs,
            &self.signer_address,
            now_since_unix.as_secs(),
        );

        let target_height = Height {
            revision_number,
            revision_height,
        };

        cosmos::inject_tendermint_proofs(
            &mut recv_msgs,
            &mut ack_msgs,
            &mut timeout_msgs,
            &self.source_tm_client,
            &target_height,
        )
        .await?;

        let trusted_light_block = self
            .source_tm_client
            .get_light_block(Some(
                client_state
                    .latest_height
                    .ok_or_else(|| anyhow::anyhow!("No latest height found"))?
                    .revision_height,
            ))
            .await?;
        let proposed_header = target_light_block.into_header(&trusted_light_block);
        let update_msg = MsgUpdateClient {
            client_id: dst_client_id,
            client_message: Some(Any::from_msg(&proposed_header)?),
            signer: self.signer_address.clone(),
        };

        let all_msgs = std::iter::once(Any::from_msg(&update_msg))
            .chain(timeout_msgs.into_iter().map(|m| Any::from_msg(&m)))
            .chain(recv_msgs.into_iter().map(|m| Any::from_msg(&m)))
            .chain(ack_msgs.into_iter().map(|m| Any::from_msg(&m)))
            .collect::<Result<Vec<_>, _>>()?;
        if all_msgs.len() == 1 {
            // The update message is the only message.
            anyhow::bail!("No messages to relay to Cosmos");
        }

        tracing::debug!(
            "Messages to be relayed to Cosmos: {:?}",
            all_msgs[1..].to_vec()
        );

        let tx_body = TxBody {
            messages: all_msgs,
            ..Default::default()
        };
        Ok(tx_body.encode_to_vec())
    }

    #[tracing::instrument(skip_all, err(Debug))]
    async fn create_client(&self, parameters: &HashMap<String, String>) -> Result<Vec<u8>> {
        if !parameters.is_empty() {
            anyhow::bail!("Parameters are not supported for creating an `07-tendermint` client");
        }

        let latest_light_block = self.source_tm_client.get_light_block(None).await?;

        tracing::info!(
            "Creating client at height: {}",
            latest_light_block.height().value()
        );

        let chain_id =
            ChainId::from_str(latest_light_block.signed_header.header.chain_id.as_str())?;
        let height = Height {
            revision_number: chain_id.revision_number(),
            revision_height: latest_light_block.height().value(),
        };
        let unbonding_period = self
            .source_tm_client
            .sdk_staking_params()
            .await?
            .unbonding_time
            .ok_or_else(|| anyhow::anyhow!("No unbonding time found"))?;
        // Defaults to the recommended 2/3 of the UnbondingPeriod
        let trusting_period = Duration {
            seconds: 2 * (unbonding_period.seconds / 3),
            nanos: 0,
        };

        let client_state = build_tendermint_client_state(
            chain_id.to_string(),
            height,
            trusting_period,
            unbonding_period,
            vec![ics23::iavl_spec(), ics23::tendermint_spec()],
        );

        let consensus_state = latest_light_block.to_consensus_state();

        let msg = MsgCreateClient {
            client_state: Some(Any::from_msg(&client_state)?),
            consensus_state: Some(consensus_state.into()),
            signer: self.signer_address.clone(),
        };

        Ok(TxBody {
            messages: vec![Any::from_msg(&msg)?],
            ..Default::default()
        }
        .encode_to_vec())
    }

    #[tracing::instrument(skip_all, err(Debug))]
    async fn update_client(&self, dst_client_id: String) -> Result<Vec<u8>> {
        let client_state = ClientState::decode(
            self.target_tm_client
                .client_state(dst_client_id.clone())
                .await?
                .value
                .as_slice(),
        )?;

        let target_light_block = self.source_tm_client.get_light_block(None).await?;
        let trusted_light_block = self
            .source_tm_client
            .get_light_block(Some(
                client_state
                    .latest_height
                    .ok_or_else(|| anyhow::anyhow!("No latest height found"))?
                    .revision_height,
            ))
            .await?;

        tracing::info!(
            "Generating tx to update '{}' from height: {} to height: {}",
            dst_client_id,
            trusted_light_block.height().value(),
            target_light_block.height().value()
        );

        let proposed_header = target_light_block.into_header(&trusted_light_block);
        let update_msg = MsgUpdateClient {
            client_id: dst_client_id,
            client_message: Some(Any::from_msg(&proposed_header)?),
            signer: self.signer_address.clone(),
        };

        Ok(TxBody {
            messages: vec![Any::from_msg(&update_msg)?],
            ..Default::default()
        }
        .encode_to_vec())
    }
}
