// SPDX-License-Identifier: Apache-2.0

//! Convert Solana misbehaviour from protobuf to Borsh format.

use std::io::{self, Read};

use anyhow::{Context, Result};
use borsh_0_10::BorshSerialize;
use ibc_client_tendermint::types::Misbehaviour;
use ibc_proto::ibc::lightclients::tendermint::v1::Misbehaviour as RawMisbehaviour;
use prost::Message;
use solana_ibc_borsh_header::conversions::misbehaviour_to_borsh;

/// Run the misbehaviour to borsh conversion.
///
/// Reads protobuf-encoded misbehaviour from stdin, converts to Borsh format,
/// and outputs hex-encoded bytes to stdout.
///
/// # Errors
/// Returns an error if reading from stdin, protobuf decoding, or borsh serialization fails.
pub fn run() -> Result<()> {
    let mut input = Vec::new();
    io::stdin()
        .read_to_end(&mut input)
        .context("Failed to read from stdin")?;

    let raw = RawMisbehaviour::decode(input.as_slice())
        .context("Failed to decode protobuf Misbehaviour")?;

    let misbehaviour =
        Misbehaviour::try_from(raw).context("Failed to convert RawMisbehaviour to Misbehaviour")?;

    let borsh_misbehaviour = misbehaviour_to_borsh(&misbehaviour);

    let borsh_bytes = borsh_misbehaviour
        .try_to_vec()
        .context("Failed to serialize to Borsh")?;

    println!("{}", hex::encode(borsh_bytes));

    Ok(())
}
