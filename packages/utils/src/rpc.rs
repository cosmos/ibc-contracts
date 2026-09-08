// SPDX-License-Identifier: Apache-2.0

//! RPC client for interacting with a Tendermint node.

use core::str::FromStr;
use std::{collections::HashMap, env, time::Duration};

use anyhow::Result;

use cosmos_sdk_proto::{
    cosmos::staking::v1beta1::{Params, QueryParamsRequest, QueryParamsResponse},
    prost::Message,
    traits::MessageExt,
    Any,
};
use ibc_core_client_types::proto::v1::{
    QueryClientStateRequest, QueryClientStateResponse, QueryConsensusStateRequest,
    QueryConsensusStateResponse,
};
use ibc_core_commitment_types::merkle::MerkleProof;
use ibc_proto_eureka::ibc::core::channel::v2::{
    QueryPacketAcknowledgementRequest, QueryPacketAcknowledgementResponse,
    QueryPacketCommitmentRequest, QueryPacketCommitmentResponse, QueryPacketReceiptRequest,
    QueryPacketReceiptResponse,
};
use tendermint::{
    block::{signed_header::SignedHeader, Height},
    validator::Set,
};
use tendermint_light_client_verifier::types::{LightBlock, ValidatorSet};
use tendermint_rpc::{Client, HttpClient, Paging, Url};

use crate::merkle::convert_tm_to_ics_merkle_proof;

/// An extension trait for [`HttpClient`] that provides additional methods for
/// obtaining light blocks.
#[async_trait::async_trait]
pub trait TendermintRpcExt {
    /// Creates a new instance of the Tendermint RPC client from the given URL.
    fn from_rpc_url(rpc_url: &str) -> Self;

    /// Creates a new instance of the Tendermint RPC client from the environment variables.
    ///
    /// # Panics
    /// Panics if the `TENDERMINT_RPC_URL` environment variable is not set or if the URL is
    /// invalid.
    #[must_use]
    fn from_env() -> Self;
    /// Gets a light block for a specific block height.
    /// If `block_height` is `None`, the latest block is fetched.
    ///
    /// # Errors
    /// Returns an error if the RPC request fails or if the response cannot be parsed.
    async fn get_light_block(&self, block_height: Option<u64>) -> Result<LightBlock>;
    /// Queries the Cosmos SDK for staking parameters.
    async fn sdk_staking_params(&self) -> Result<Params>;
    /// Fetches the client state from the Tendermint node.
    async fn client_state(&self, client_id: String) -> Result<Any>;
    /// Fetches the Ethereum consensus state from the light client on cosmos.
    /// If the revision height is 0, the latest height is fetched.
    async fn consensus_state(&self, client_id: String, revision_height: u64) -> Result<Any>;
    /// Proves a path in the chain's Merkle tree and returns the value at the path and the proof.
    /// If the value is empty, then this is a non-inclusion proof.
    async fn prove_path(&self, path: &[Vec<u8>], height: u64) -> Result<(Vec<u8>, MerkleProof)>;

    /// Fetches the packet commitment from the Tendermint node (IBC V2).
    async fn v2_packet_commitment(
        &self,
        client_id: String,
        seq: u64,
        height: u64,
        prove: bool,
    ) -> Result<QueryPacketCommitmentResponse>;

    /// Fetches the packet receipt from the Tendermint node (IBC V2).
    /// Used for timeout proofs to verify a packet was NOT received.
    async fn v2_packet_receipt(
        &self,
        client_id: String,
        seq: u64,
        height: u64,
    ) -> Result<QueryPacketReceiptResponse>;

    /// Fetches the packet acknowledgement from the Tendermint node (IBC V2).
    /// Used for ack proofs to verify a packet was acknowledged.
    async fn v2_packet_acknowledgement(
        &self,
        client_id: String,
        seq: u64,
        height: u64,
    ) -> Result<QueryPacketAcknowledgementResponse>;
}

#[async_trait::async_trait]
impl TendermintRpcExt for HttpClient {
    fn from_rpc_url(rpc_url: &str) -> Self {
        Self::builder(
            Url::from_str(rpc_url)
                .expect("Failed to parse tendermint RPC URL")
                .try_into()
                .expect("Failed to convert tendermint RPC URL"),
        )
        .client(
            reqwest_0_11::ClientBuilder::new()
                // Send an explicit User-Agent. reqwest sends none by default, and some RPC
                // providers reject empty User-Agent requests with 403 Forbidden.
                .user_agent("proof-api")
                .connect_timeout(Duration::from_secs(10))
                .timeout(Duration::from_secs(30))
                .pool_idle_timeout(Duration::from_secs(10))
                .build()
                .expect("Failed to create reqwest client"),
        )
        .build()
        .expect("Failed to create tendermint HTTP client")
    }

    fn from_env() -> Self {
        Self::from_rpc_url(&env::var("TENDERMINT_RPC_URL").expect("TENDERMINT_RPC_URL not set"))
    }

    async fn get_light_block(&self, block_height: Option<u64>) -> Result<LightBlock> {
        let peer_id = self.status().await?.node_info.id;
        let commit_response;
        let height: u64;
        if let Some(block_height) = block_height {
            height = block_height;
            let commit_height: Height = block_height.try_into()?;
            commit_response = self.commit(commit_height).await?;
        } else {
            commit_response = self.latest_commit().await?;
            height = commit_response.signed_header.header.height.value();
        }
        let mut signed_header = commit_response.signed_header;

        let validator_response = self
            .validators(TryInto::<Height>::try_into(height)?, Paging::All)
            .await?;
        let validators = Set::with_proposer(
            validator_response.validators,
            signed_header.header().proposer_address,
        )?;

        let next_height = height + 1;
        let next_validator_response = self
            .validators(TryInto::<Height>::try_into(next_height)?, Paging::All)
            .await?;
        let next_validators = Set::with_proposer(
            next_validator_response.validators,
            // WARN: This proposer is likely to be incorrect,
            // but it is not used in the light block verification,
            // and required by ibc-go's validate basic.
            signed_header.header().proposer_address,
        )?;

        sort_signatures_by_validators_power_desc(&mut signed_header, &validators);
        Ok(LightBlock::new(
            signed_header,
            validators,
            next_validators,
            peer_id,
        ))
    }

    async fn sdk_staking_params(&self) -> Result<Params> {
        let res = self
            .abci_query(
                Some("/cosmos.staking.v1beta1.Query/Params".to_string()),
                QueryParamsRequest::default().to_bytes()?,
                None,
                false,
            )
            .await?;

        if res.code.is_err() {
            anyhow::bail!(
                "ABCI /cosmos.staking.v1beta1.Query/Params returned non-zero code {}: {}",
                res.code.value(),
                res.log,
            );
        }

        QueryParamsResponse::decode(res.value.as_slice())?
            .params
            .ok_or_else(|| anyhow::anyhow!("No staking params found"))
    }

    async fn client_state(&self, client_id: String) -> Result<Any> {
        let res = self
            .abci_query(
                Some("/ibc.core.client.v1.Query/ClientState".to_string()),
                QueryClientStateRequest { client_id }.to_bytes()?,
                None,
                false,
            )
            .await?;

        if res.code.is_err() {
            anyhow::bail!(
                "ABCI /ibc.core.client.v1.Query/ClientState returned non-zero code {}: {}",
                res.code.value(),
                res.log,
            );
        }

        QueryClientStateResponse::decode(res.value.as_slice())?
            .client_state
            .ok_or_else(|| anyhow::anyhow!("No client state found"))
    }

    async fn consensus_state(&self, client_id: String, revision_height: u64) -> Result<Any> {
        let res = self
            .abci_query(
                Some("/ibc.core.client.v1.Query/ConsensusState".to_string()),
                QueryConsensusStateRequest {
                    client_id,
                    revision_number: 0,
                    revision_height,
                    latest_height: revision_height == 0,
                }
                .encode_to_vec(),
                None,
                false,
            )
            .await?;

        if res.code.is_err() {
            anyhow::bail!(
                "ABCI /ibc.core.client.v1.Query/ConsensusState returned non-zero code {}: {}",
                res.code.value(),
                res.log,
            );
        }

        QueryConsensusStateResponse::decode(res.value.as_slice())?
            .consensus_state
            .ok_or_else(|| anyhow::anyhow!("No consensus state found"))
    }

    async fn prove_path(&self, path: &[Vec<u8>], height: u64) -> Result<(Vec<u8>, MerkleProof)> {
        let res = self
            .abci_query(
                Some(format!("store/{}/key", std::str::from_utf8(&path[0])?)),
                path[1..].concat(),
                // Proof height should be the block before the target block.
                Some((height - 1).try_into()?),
                true,
            )
            .await?;

        if res.code.is_err() {
            anyhow::bail!(
                "ABCI store/{} query returned non-zero code {}: {}",
                std::str::from_utf8(&path[0])?,
                res.code.value(),
                res.log,
            );
        }

        if res.height.value() + 1 != height {
            anyhow::bail!("Proof height mismatch");
        }

        if res.key.as_slice() != path[1].as_slice() {
            anyhow::bail!("Key mismatch");
        }

        let vm_proof = convert_tm_to_ics_merkle_proof(
            &res.proof
                .ok_or_else(|| anyhow::anyhow!("Proof could not be retrieved"))?,
        )?;
        if vm_proof.proofs.is_empty() {
            anyhow::bail!("Empty proof");
        }

        anyhow::Ok((res.value, vm_proof))
    }
    //
    // https://github.com/cosmos/ibc-go/blob/release/v10.3.x/modules/core/04-channel/v2/keeper/grpc_query.go#L60
    async fn v2_packet_commitment(
        &self,
        client_id: String,
        seq: u64,
        height: u64,
        prove: bool,
    ) -> Result<QueryPacketCommitmentResponse> {
        let request = QueryPacketCommitmentRequest {
            client_id,
            sequence: seq,
        };

        let res = self
            .abci_query(
                Some("/ibc.core.channel.v2.Query/PacketCommitment".to_string()),
                request.encode_to_vec(),
                Some(height.try_into()?),
                prove,
            )
            .await?;

        if res.code.is_err() {
            anyhow::bail!(
                "ABCI /ibc.core.channel.v2.Query/PacketCommitment returned non-zero code {}: {}",
                res.code.value(),
                res.log,
            );
        }

        // V1 & V2 share the same response type
        QueryPacketCommitmentResponse::decode(res.value.as_slice())
            .map_err(|e| anyhow::anyhow!("Failed to decode QueryPacketCommitmentResponse: {e}"))
    }

    // https://github.com/cosmos/ibc-go/blob/release/v10.3.x/modules/core/04-channel/v2/keeper/grpc_query.go#L206
    async fn v2_packet_receipt(
        &self,
        client_id: String,
        seq: u64,
        height: u64,
    ) -> Result<QueryPacketReceiptResponse> {
        let request = QueryPacketReceiptRequest {
            client_id,
            sequence: seq,
        };

        let res = self
            .abci_query(
                Some("/ibc.core.channel.v2.Query/PacketReceipt".to_string()),
                request.encode_to_vec(),
                Some(height.try_into()?),
                false,
            )
            .await?;

        if res.code.is_err() {
            anyhow::bail!(
                "ABCI /ibc.core.channel.v2.Query/PacketReceipt returned non-zero code {}: {}",
                res.code.value(),
                res.log,
            );
        }

        QueryPacketReceiptResponse::decode(res.value.as_slice())
            .map_err(|e| anyhow::anyhow!("Failed to decode QueryPacketReceiptResponse: {e}"))
    }

    // https://github.com/cosmos/ibc-go/blob/release/v10.3.x/modules/core/04-channel/v2/keeper/grpc_query.go#L122
    async fn v2_packet_acknowledgement(
        &self,
        client_id: String,
        seq: u64,
        height: u64,
    ) -> Result<QueryPacketAcknowledgementResponse> {
        let request = QueryPacketAcknowledgementRequest {
            client_id,
            sequence: seq,
        };

        let res = self
            .abci_query(
                Some("/ibc.core.channel.v2.Query/PacketAcknowledgement".to_string()),
                request.encode_to_vec(),
                Some(height.try_into()?),
                false,
            )
            .await?;

        if res.code.is_err() {
            anyhow::bail!(
                "ABCI /ibc.core.channel.v2.Query/PacketAcknowledgement returned non-zero code {}: {}",
                res.code.value(),
                res.log,
            );
        }

        QueryPacketAcknowledgementResponse::decode(res.value.as_slice()).map_err(|e| {
            anyhow::anyhow!("Failed to decode QueryPacketAcknowledgementResponse: {e}")
        })
    }
}

/// Sorts the signatures in the signed header based on the descending order of validators' power.
fn sort_signatures_by_validators_power_desc(
    signed_header: &mut SignedHeader,
    validators_set: &ValidatorSet,
) {
    let validator_powers: HashMap<_, _> = validators_set
        .validators()
        .iter()
        .map(|v| (v.address, v.power()))
        .collect();

    signed_header.commit.signatures.sort_by(|a, b| {
        let power_a = a
            .validator_address()
            .and_then(|addr| validator_powers.get(&addr))
            .unwrap_or(&0);
        let power_b = b
            .validator_address()
            .and_then(|addr| validator_powers.get(&addr))
            .unwrap_or(&0);
        power_b.cmp(power_a)
    });
}
