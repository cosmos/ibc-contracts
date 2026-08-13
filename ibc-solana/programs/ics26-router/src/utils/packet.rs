// SPDX-License-Identifier: Apache-2.0

use crate::errors::RouterError;
use anchor_lang::prelude::*;
use solana_ibc_types::{Packet, Payload};

// TODO: Support multi-payload packets #602
pub fn get_single_payload(packet: &Packet) -> Result<&Payload> {
    match &packet.payloads[..] {
        [payload] => Ok(payload),
        _ => err!(RouterError::InvalidPayloadCount),
    }
}
