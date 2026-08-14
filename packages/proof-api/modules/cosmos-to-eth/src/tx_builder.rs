// SPDX-License-Identifier: Apache-2.0

//! This module defines [`TxBuilder`] which is responsible for building transactions to be sent to
//! the Ethereum chain from events received from the Cosmos SDK chain.

use std::{collections::HashMap, str::FromStr};

use alloy::{
    network::Ethereum,
    primitives::{keccak256, Address},
    providers::Provider,
    sol_types::{SolCall, SolValue},
};
use anyhow::Result;
use ibc_core_host_types::identifiers::ChainId;
use ibc_eureka_solidity_types::{
    ics26::{
        router::{multicallCall, routerCalls, routerInstance, updateClientCall},
        IICS02ClientMsgs::Height,
    },
    msgs::{
        IICS07TendermintMsgs::{ClientState, ConsensusState, TrustThreshold},
        ISP1Msgs::SP1Proof,
        IUpdateClientMsgs::MsgUpdateClient,
    },
    sp1_ics07::sp1_ics07_tendermint,
};
use ibc_eureka_utils::{light_block::LightBlockExt, rpc::TendermintRpcExt};
use sp1_ics07_tendermint_prover::{
    programs::{SP1ICS07TendermintPrograms, SP1Program},
    prover::{SP1ICS07TendermintProver, Sp1Prover, SupportedZkAlgorithm},
};
use sp1_sdk::HashableKey;
use tendermint_rpc::HttpClient;

use proof_api_lib::{
    aggregator::Aggregator,
    chain::{CosmosSdk, EthEureka},
    events::EurekaEventWithHeight,
    tx_builder::TxBuilderService,
    utils::{
        eth_attested::{
            build_eth_attestor_create_client_calldata, build_eth_attestor_relay_events_tx,
            build_eth_attestor_update_client_calldata,
        },
        eth_eureka::{self, inject_sp1_proof},
        RelayEventsParams,
    },
};

/// The `TxBuilder` produces txs to [`EthEureka`] based on events from [`CosmosSdk`].
#[allow(dead_code)]
pub struct TxBuilder<P>
where
    P: Provider + Clone,
{
    /// The IBC Eureka router instance.
    pub ics26_router: routerInstance<P, Ethereum>,
    /// The HTTP client for the Cosmos SDK.
    pub tm_client: HttpClient,
    /// SP1 prover for generating proofs.
    pub sp1_prover: Sp1Prover,
    /// The SP1 programs for ICS07 Tendermint.
    pub sp1_programs: SP1ICS07TendermintPrograms,
}

impl<P> TxBuilder<P>
where
    P: Provider + Clone,
{
    /// Create a new [`TxBuilder`] instance.
    pub const fn new(
        ics26_address: Address,
        provider: P,
        tm_client: HttpClient,
        sp1_prover: Sp1Prover,
        sp1_programs: SP1ICS07TendermintPrograms,
    ) -> Self {
        Self {
            ics26_router: routerInstance::new(ics26_address, provider),
            tm_client,
            sp1_prover,
            sp1_programs,
        }
    }

    /// Get the client state for a given client ID.
    /// # Errors
    /// Returns an error if the client state cannot be retrieved.
    pub async fn client_state(&self, client_id: String) -> Result<ClientState> {
        let ics07_address = self.ics26_router.getClient(client_id).call().await?;
        Ok(
            sp1_ics07_tendermint::new(ics07_address, self.ics26_router.provider().clone())
                .clientState()
                .call()
                .await?
                .into(),
        )
    }

    /// Get the metadata for the transaction builder.
    pub fn metadata(&self) -> HashMap<String, String> {
        HashMap::from([
            (
                "update_client_vkey".to_string(),
                self.sp1_programs.update_client.get_vkey().bytes32(),
            ),
            (
                "membership_vkey".to_string(),
                self.sp1_programs.membership.get_vkey().bytes32(),
            ),
            (
                "update_client_and_membership_vkey".to_string(),
                self.sp1_programs
                    .update_client_and_membership
                    .get_vkey()
                    .bytes32(),
            ),
            (
                "misbehaviour_vkey".to_string(),
                self.sp1_programs.misbehaviour.get_vkey().bytes32(),
            ),
        ])
    }
}

/// The key for the SP1 verifier in the parameters map.
const SP1_VERIFIER: &str = "sp1_verifier";
/// The key for the zk algorithm in the parameters map.
const ZK_ALGORITHM: &str = "zk_algorithm";
/// The key for the role manager in the parameters map.
const ROLE_MANAGER: &str = "role_manager";

#[async_trait::async_trait]
impl<P> TxBuilderService<EthEureka, CosmosSdk> for TxBuilder<P>
where
    P: Provider + Clone,
{
    #[tracing::instrument(skip_all, err(Debug))]
    async fn relay_events(
        &self,
        src_events: Vec<EurekaEventWithHeight>,
        dest_events: Vec<EurekaEventWithHeight>,
        src_client_id: String,
        dst_client_id: String,
        src_packet_seqs: Vec<u64>,
        dst_packet_seqs: Vec<u64>,
    ) -> Result<Vec<u8>> {
        let now_since_unix = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?;

        let latest_light_block = self.tm_client.get_light_block(None).await?;
        let revision_height = latest_light_block.height().value();
        let chain_id =
            ChainId::from_str(latest_light_block.signed_header.header.chain_id.as_str())?;
        let latest_height = Height {
            revisionNumber: chain_id.revision_number(),
            revisionHeight: revision_height,
        };

        let timeout_msgs = eth_eureka::target_events_to_timeout_msgs(
            dest_events,
            &src_client_id,
            &dst_client_id,
            &dst_packet_seqs,
            &latest_height,
            now_since_unix.as_secs(),
        );

        let recv_and_ack_msgs = eth_eureka::src_events_to_recv_and_ack_msgs(
            src_events,
            &src_client_id,
            &dst_client_id,
            &src_packet_seqs,
            &dst_packet_seqs,
            &latest_height,
            now_since_unix.as_secs(),
        );

        let mut all_msgs = timeout_msgs
            .into_iter()
            .chain(recv_and_ack_msgs.into_iter())
            .collect::<Vec<_>>();
        if all_msgs.is_empty() {
            anyhow::bail!("No messages to relay to Ethereum");
        }

        tracing::debug!("Messages to be relayed to Ethereum: {:?}", all_msgs);

        let client_state = self.client_state(dst_client_id).await?;

        inject_sp1_proof(
            &self.sp1_prover,
            &self.sp1_programs.update_client_and_membership,
            &mut all_msgs,
            &self.tm_client,
            latest_light_block,
            client_state,
            now_since_unix.as_nanos(),
        )
        .await?;

        let calls = all_msgs.into_iter().map(|msg| match msg {
            routerCalls::timeoutPacket(call) => call.abi_encode(),
            routerCalls::recvPacket(call) => call.abi_encode(),
            routerCalls::ackPacket(call) => call.abi_encode(),
            _ => unreachable!(),
        });

        let multicall_tx = multicallCall {
            data: calls.map(Into::into).collect(),
        };

        Ok(multicall_tx.abi_encode())
    }

    #[tracing::instrument(skip_all, err(Debug))]
    async fn create_client(&self, parameters: &HashMap<String, String>) -> Result<Vec<u8>> {
        // Check if parameters only include correct keys
        parameters
            .keys()
            .find(|k| ![SP1_VERIFIER, ZK_ALGORITHM, ROLE_MANAGER].contains(&k.as_str()))
            .map_or(Ok(()), |param| {
                Err(anyhow::anyhow!("Unexpected parameter: `{param}`, only `{SP1_VERIFIER}` and `{ZK_ALGORITHM}` are allowed"))
            })?;

        let latest_light_block = self.tm_client.get_light_block(None).await?;

        tracing::info!(
            "Creating client at height: {}",
            latest_light_block.height().value()
        );

        let role_admin = parameters
            .get(ROLE_MANAGER)
            .map_or(Ok(Address::ZERO), |a| {
                Address::from_str(a.as_str()).map_err(|e| anyhow::anyhow!(e))
            })?;
        let sp1_verifier = Address::from_str(
            parameters
                .get(SP1_VERIFIER)
                .ok_or_else(|| anyhow::anyhow!("Missing `{SP1_VERIFIER}` parameter"))?,
        )?;
        let zk_algorithm = parameters
            .get(ZK_ALGORITHM)
            .map_or(Ok(SupportedZkAlgorithm::Groth16), |z| {
                SupportedZkAlgorithm::from_str(z.as_str())
            })?;
        let default_trust_threshold = TrustThreshold {
            numerator: 1,
            denominator: 3,
        };
        let unbonding_period = self
            .tm_client
            .sdk_staking_params()
            .await?
            .unbonding_time
            .ok_or_else(|| anyhow::anyhow!("No unbonding time found"))?
            .seconds
            .try_into()?;
        let trusting_period = 2 * (unbonding_period / 3);

        let client_state = latest_light_block.to_sol_client_state(
            default_trust_threshold,
            unbonding_period,
            trusting_period,
            zk_algorithm,
        )?;

        let consensus_state = ConsensusState::from(latest_light_block.to_consensus_state());
        let consensus_state_hash = keccak256(consensus_state.abi_encode());

        Ok(sp1_ics07_tendermint::deploy_builder(
            self.ics26_router.provider().clone(),
            self.sp1_programs
                .update_client
                .get_vkey()
                .bytes32_raw()
                .into(),
            self.sp1_programs.membership.get_vkey().bytes32_raw().into(),
            self.sp1_programs
                .update_client_and_membership
                .get_vkey()
                .bytes32_raw()
                .into(),
            self.sp1_programs
                .misbehaviour
                .get_vkey()
                .bytes32_raw()
                .into(),
            sp1_verifier,
            client_state.abi_encode().into(),
            consensus_state_hash,
            role_admin,
        )
        .calldata()
        .to_vec())
    }

    #[tracing::instrument(skip_all, err(Debug))]
    async fn update_client(&self, dst_client_id: String) -> Result<Vec<u8>> {
        let now_since_unix = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?;

        let client_state = self.client_state(dst_client_id.clone()).await?;
        let trusted_block_height = client_state.latestHeight.revisionHeight;

        let trusted_light_block = self
            .tm_client
            .get_light_block(Some(trusted_block_height))
            .await?;

        let latest_light_block = self.tm_client.get_light_block(None).await?;

        tracing::info!(
            "Generating tx to update '{}' from height: {} to height: {}",
            dst_client_id,
            trusted_light_block.height().value(),
            latest_light_block.height().value()
        );

        let proposed_header = latest_light_block.into_header(&trusted_light_block);

        let update_client_prover = SP1ICS07TendermintProver::new(
            client_state.zkAlgorithm,
            &self.sp1_prover,
            &self.sp1_programs.update_client,
        )
        .await;

        let trusted_consensus_state = trusted_light_block.to_consensus_state().into();
        let proof_data = update_client_prover
            .generate_proof(
                &client_state,
                &trusted_consensus_state,
                &proposed_header,
                now_since_unix.as_nanos(),
            )
            .await;

        let update_msg = MsgUpdateClient {
            sp1Proof: SP1Proof::new(
                &self.sp1_programs.update_client.get_vkey().bytes32(),
                proof_data.bytes(),
                proof_data.public_values.to_vec(),
            ),
        };

        Ok(updateClientCall {
            clientId: dst_client_id,
            updateMsg: update_msg.abi_encode().into(),
        }
        .abi_encode())
    }
}

/// Transaction builder for attested relay from Cosmos to Ethereum.
pub struct AttestedTxBuilder<P: Provider + Clone> {
    aggregator: Aggregator,
    ics26_address: Address,
    provider: P,
}

impl<P: Provider + Clone> AttestedTxBuilder<P> {
    /// Create a new [`AttestedTxBuilder`] instance.
    pub const fn new(aggregator: Aggregator, ics26_address: Address, provider: P) -> Self {
        Self {
            aggregator,
            ics26_address,
            provider,
        }
    }

    /// Returns the ICS26 router address.
    pub const fn ics26_address(&self) -> &Address {
        &self.ics26_address
    }

    /// Relay events from Cosmos to Ethereum using attestations.
    ///
    /// # Errors
    /// Returns an error if attestation retrieval or transaction building fails.
    pub async fn relay_events(&self, params: RelayEventsParams) -> Result<Vec<u8>> {
        build_eth_attestor_relay_events_tx(&self.aggregator, params).await
    }

    /// Create a client on Ethereum using attestations.
    ///
    /// # Errors
    /// Returns an error if transaction building fails.
    pub fn create_client(&self, parameters: &HashMap<String, String>) -> Result<Vec<u8>> {
        build_eth_attestor_create_client_calldata(parameters, self.provider.clone())
    }

    /// Update a client on Ethereum using attestations.
    ///
    /// # Errors
    /// Returns an error when called (not yet implemented).
    #[allow(clippy::unused_async)]
    pub async fn update_client(&self, dst_client_id: String) -> Result<Vec<u8>> {
        build_eth_attestor_update_client_calldata(&self.aggregator, dst_client_id).await
    }
}
