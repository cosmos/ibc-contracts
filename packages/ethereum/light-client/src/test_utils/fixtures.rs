// SPDX-License-Identifier: Apache-2.0

//! Test fixtures types and ulitiies for the Ethereum light client

use std::path::PathBuf;

use ibc_eureka_solidity_types::ics26::IICS26RouterMsgs;
use ibc_proto_eureka::{
    cosmos::tx::v1beta1::TxBody,
    ibc::core::{
        channel::v2::{MsgAcknowledgement, MsgRecvPacket, MsgTimeout, Packet},
        client::v1::MsgUpdateClient,
    },
};
use prost::Message;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{client_state::ClientState, consensus_state::ConsensusState};

/// A test fixture with an ordered list of light client operations from the e2e test
#[derive(Serialize, Deserialize, JsonSchema, PartialEq, Eq, Clone, Debug)]
pub struct StepsFixture {
    /// steps is a list of light client operations
    pub steps: Vec<Step>,
}

/// Step is a light client operation such as an initial state, commitment proof, or update client
#[derive(Serialize, Deserialize, JsonSchema, PartialEq, Eq, Clone, Debug)]
pub struct Step {
    /// name is the name of the operation, only used for documentation and easy of reading
    pub name: String,
    /// data is the operation data as a JSON object to be deserialized into the appropriate type
    pub data: Value,
}

/// The initial state of the light client in the e2e tests
#[derive(Serialize, Deserialize, JsonSchema, PartialEq, Eq, Clone, Debug)]
pub struct InitialState {
    /// The client state at the initial state
    pub client_state: ClientState,
    /// The consensus state at the initial state
    pub consensus_state: ConsensusState,
}

/// Operation to update the light client
#[derive(Serialize, Deserialize, JsonSchema, PartialEq, Eq, Clone, Debug)]
pub struct RelayerMessages {
    /// The headers used to update the light client, in order, as a `TxBody`, encoded as hex
    pub relayer_tx_body: String,
}

impl RelayerMessages {
    /// Get the SDK messages from the relayer tx
    /// # Panics
    /// Panics if the relayer tx or any of the messages cannot be decoded
    /// # Returns
    /// A tuple with the SDK messages contained in the SDK tx
    #[must_use]
    pub fn get_sdk_msgs(
        &self,
    ) -> (
        Vec<MsgUpdateClient>,
        Vec<MsgRecvPacket>,
        Vec<MsgAcknowledgement>,
        Vec<MsgTimeout>,
    ) {
        let tx_body_bz = hex::decode(self.relayer_tx_body.clone()).unwrap();
        let tx_body = TxBody::decode(tx_body_bz.as_slice()).unwrap();

        tx_body.messages.iter().fold(
            (Vec::new(), Vec::new(), Vec::new(), Vec::new()),
            |(mut update_clients, mut recv_msgs, mut ack_msgs, mut timeout_msgs), msg| {
                match msg.type_url.as_str() {
                    "/ibc.core.client.v1.MsgUpdateClient" => {
                        // Decode as MsgUpdateClient
                        update_clients.push(MsgUpdateClient::decode(msg.value.as_slice()).unwrap());
                    }
                    "/ibc.core.channel.v2.MsgRecvPacket" => {
                        // Decode as MsgRecvPacket
                        recv_msgs.push(MsgRecvPacket::decode(msg.value.as_slice()).unwrap());
                    }
                    "/ibc.core.channel.v2.MsgAcknowledgement" => {
                        // Decode as MsgAcknowledgement
                        ack_msgs.push(MsgAcknowledgement::decode(msg.value.as_slice()).unwrap());
                    }
                    "/ibc.core.channel.v2.MsgTimeout" => {
                        // Decode as MsgTimeout
                        timeout_msgs.push(MsgTimeout::decode(msg.value.as_slice()).unwrap());
                    }
                    _ => panic!("Unknown message type: {}", msg.type_url),
                }
                (update_clients, recv_msgs, ack_msgs, timeout_msgs)
            },
        )
    }
}

/// Get the commitment path and value for the given packet
/// # Panics
/// Panics if the packet is missing or the path cannot be constructed
/// # Returns
/// A tuple with the commitment path and value
#[must_use]
pub fn get_packet_paths(packet: Packet) -> (Vec<u8>, Vec<u8>, Vec<u8>) {
    let ics26_packet: IICS26RouterMsgs::Packet = packet.into();
    (
        ics26_packet.commitment_path(),
        ics26_packet.commitment(),
        ics26_packet.receipt_commitment_path(),
    )
}

impl StepsFixture {
    /// Deserializes the data at the given step into the given type
    /// # Panics
    /// Panics if the data cannot be deserialized into the given type
    #[must_use]
    pub fn get_data_at_step<T>(&self, step: usize) -> T
    where
        T: serde::de::DeserializeOwned,
    {
        serde_json::from_value(self.steps[step].data.clone()).unwrap()
    }
}

/// load loads a test fixture from a JSON file
/// # Panics
/// Panics if the file cannot be opened or the contents cannot be deserialized
#[must_use]
pub fn load<T>(name: &str) -> T
where
    T: serde::de::DeserializeOwned,
{
    // Construct the path relative to the Cargo manifest directory
    let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    path.push("src/test_utils/fixtures");
    path.push(format!("{name}.json"));

    // Open the file and deserialize its contents
    let file = std::fs::File::open(path).unwrap();
    serde_json::from_reader(file).unwrap()
}
