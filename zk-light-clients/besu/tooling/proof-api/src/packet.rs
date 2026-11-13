//! Canonical packet statements, ported from the Go Besu prover.
use besu_zk_types::sha256;
use tonic::Status;

use crate::proto::{Packet, Payload, ProofKind};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Statement {
    pub terminal_path: Vec<u8>,
    pub full_path: Vec<Vec<u8>>,
    pub path_digest: [u8; 32],
    pub logical_slot: [u8; 32],
    pub expected_commitment: Option<[u8; 32]>,
}

pub fn build_statement(
    kind: ProofKind,
    packet: &Packet,
    prefix: &[Vec<u8>],
    configured_client: &str,
    counterparty: &str,
) -> Result<Statement, Status> {
    if packet.sequence == 0
        || packet.payloads.len() != 1
        || packet.source_client.is_empty()
        || packet.destination_client.is_empty()
    {
        return Err(Status::invalid_argument("invalid packet"));
    }
    let (identifier, tag) = match kind {
        ProofKind::PacketCommitment
            if packet.destination_client == configured_client
                && packet.source_client == counterparty =>
        {
            (&packet.source_client, 1)
        }
        ProofKind::Acknowledgement
            if packet.source_client == configured_client
                && packet.destination_client == counterparty =>
        {
            (&packet.destination_client, 3)
        }
        ProofKind::ReceiptAbsence
            if packet.source_client == configured_client
                && packet.destination_client == counterparty =>
        {
            (&packet.destination_client, 2)
        }
        _ => {
            return Err(Status::invalid_argument(
                "packet direction or proof kind is invalid",
            ))
        }
    };
    let mut terminal_path = identifier.as_bytes().to_vec();
    terminal_path.push(tag);
    terminal_path.extend_from_slice(&packet.sequence.to_be_bytes());
    let mut full_path = prefix.to_vec();
    full_path
        .last_mut()
        .ok_or_else(|| Status::invalid_argument("merkle prefix is empty"))?
        .extend_from_slice(&terminal_path);
    Ok(Statement {
        path_digest: ibc_storage::path_digest(&full_path),
        logical_slot: ibc_storage::logical_storage_slot(&terminal_path),
        expected_commitment: (kind == ProofKind::PacketCommitment)
            .then(|| packet_commitment(packet)),
        terminal_path,
        full_path,
    })
}

#[must_use]
pub fn packet_commitment(packet: &Packet) -> [u8; 32] {
    let payloads: Vec<u8> = packet.payloads.iter().flat_map(payload_hash).collect();
    let mut preimage = Vec::with_capacity(97);
    preimage.push(2);
    preimage.extend_from_slice(&sha256(packet.destination_client.as_bytes()));
    preimage.extend_from_slice(&sha256(&packet.timeout_timestamp.to_be_bytes()));
    preimage.extend_from_slice(&sha256(&payloads));
    sha256(&preimage)
}

fn payload_hash(payload: &Payload) -> [u8; 32] {
    let mut preimage = Vec::with_capacity(160);
    for field in [
        payload.source_port.as_bytes(),
        payload.destination_port.as_bytes(),
        payload.version.as_bytes(),
        payload.encoding.as_bytes(),
        payload.value.as_slice(),
    ] {
        preimage.extend_from_slice(&sha256(field));
    }
    sha256(&preimage)
}
