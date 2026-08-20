// SPDX-License-Identifier: Apache-2.0

//! Solidity types for ICS26Router.sol

use alloy_primitives::hex;
use ibc_proto_eureka::ibc::core::channel::v2::{Packet, Payload};
use sha2::{Digest, Sha256};

/// The EVM storage slot for the `ICS26Router`'s provable IBC store.
pub const ICS26_IBC_STORAGE_SLOT: [u8; 32] =
    hex!("0x1260944489272988d9df285149b5aa1b0f48f2136d6f416159f840a3e0747600");

#[cfg(feature = "rpc")]
alloy_sol_types::sol!(
    #[sol(rpc)]
    #[derive(Debug, PartialEq, Eq)]
    router,
    "../../ibc-solidity/abi/ICS26Router.json"
);

// NOTE: Some environments won't compile with the `rpc` features.
#[cfg(not(feature = "rpc"))]
alloy_sol_types::sol!(
    #[derive(Debug, PartialEq, Eq)]
    router,
    "../../ibc-solidity/abi/ICS26Router.json"
);

impl IICS26RouterMsgs::Packet {
    /// Returns the commitment path for the packet.
    #[must_use]
    pub fn commitment_path(&self) -> Vec<u8> {
        let mut path = Vec::new();
        path.extend_from_slice(self.sourceClient.as_bytes());
        path.push(1_u8);
        path.extend_from_slice(&self.sequence.to_be_bytes());
        path
    }

    /// Returns the packet commitment
    #[must_use]
    pub fn commitment(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        // Hash the destination_client field
        let dest_id_hash = Sha256::digest(self.destClient.as_bytes());
        buf.extend_from_slice(&dest_id_hash);

        // Convert the timeout timestamp to big-endian bytes and hash it.
        let timeout_bytes = self.timeoutTimestamp.to_be_bytes();
        let timeout_hash = Sha256::digest(timeout_bytes);
        buf.extend_from_slice(&timeout_hash);

        // Concatenate the hash of each payload.
        let mut app_bytes = Vec::new();
        for payload in &self.payloads {
            app_bytes.extend_from_slice(&payload.commitment_hash());
        }
        let app_hash = Sha256::digest(&app_bytes);
        buf.extend_from_slice(&app_hash);

        // Prepend the version byte (2)
        let mut final_buf = vec![2u8];
        final_buf.extend_from_slice(&buf);

        // Compute the final hash
        let final_hash = Sha256::digest(&final_buf);
        final_hash.to_vec()
    }

    /// Returns the commitment path for the receipt.
    #[must_use]
    pub fn receipt_commitment_path(&self) -> Vec<u8> {
        let mut path = Vec::new();
        path.extend_from_slice(self.destClient.as_bytes());
        path.push(2_u8);
        path.extend_from_slice(&self.sequence.to_be_bytes());
        path
    }

    /// Returns the commitment path for the acknowledgement.
    #[must_use]
    pub fn ack_commitment_path(&self) -> Vec<u8> {
        let mut path = Vec::new();
        path.extend_from_slice(self.destClient.as_bytes());
        path.push(3_u8);
        path.extend_from_slice(&self.sequence.to_be_bytes());
        path
    }
}

/// Returns the acknowledgement commitment for one or more acknowledgements.
#[must_use]
pub fn acknowledgement_commitment(acks: &[alloy_primitives::Bytes]) -> Vec<u8> {
    let mut buf = vec![2_u8];
    for ack in acks {
        buf.extend_from_slice(&Sha256::digest(ack));
    }
    Sha256::digest(&buf).to_vec()
}

impl From<Packet> for IICS26RouterMsgs::Packet {
    fn from(packet: Packet) -> Self {
        Self {
            sequence: packet.sequence,
            sourceClient: packet.source_client,
            destClient: packet.destination_client,
            timeoutTimestamp: packet.timeout_timestamp,
            payloads: packet.payloads.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<IICS26RouterMsgs::Packet> for Packet {
    fn from(packet: IICS26RouterMsgs::Packet) -> Self {
        Self {
            sequence: packet.sequence,
            source_client: packet.sourceClient,
            destination_client: packet.destClient,
            timeout_timestamp: packet.timeoutTimestamp,
            payloads: packet.payloads.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<Payload> for IICS26RouterMsgs::Payload {
    fn from(payload: Payload) -> Self {
        Self {
            sourcePort: payload.source_port,
            destPort: payload.destination_port,
            version: payload.version,
            encoding: payload.encoding,
            value: payload.value.into(),
        }
    }
}

impl From<IICS26RouterMsgs::Payload> for Payload {
    fn from(payload: IICS26RouterMsgs::Payload) -> Self {
        Self {
            source_port: payload.sourcePort,
            destination_port: payload.destPort,
            version: payload.version,
            encoding: payload.encoding,
            value: payload.value.into(),
        }
    }
}

impl IICS26RouterMsgs::Payload {
    /// Returns the commitment path for the payload.
    #[must_use]
    pub fn commitment_hash(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        // Hash source_port
        let source_hash = Sha256::digest(self.sourcePort.as_bytes());
        buf.extend_from_slice(&source_hash);

        // Hash destination_port
        let dest_hash = Sha256::digest(self.destPort.as_bytes());
        buf.extend_from_slice(&dest_hash);

        // Hash version
        let payload_version_hash = Sha256::digest(self.version.as_bytes());
        buf.extend_from_slice(&payload_version_hash);

        // Hash encoding
        let payload_encoding_hash = Sha256::digest(self.encoding.as_bytes());
        buf.extend_from_slice(&payload_encoding_hash);

        // Hash value
        let payload_value_hash = Sha256::digest(&self.value);
        buf.extend_from_slice(&payload_value_hash);

        // Hash the concatenated result
        let final_hash = Sha256::digest(&buf);
        final_hash.to_vec()
    }
}
