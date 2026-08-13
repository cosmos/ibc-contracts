// SPDX-License-Identifier: Apache-2.0

//! ICS07 Tendermint light client types for Solana
//!
//! These types define the messages for the ICS07 Tendermint light client.

use anchor_lang::prelude::*;

pub use solana_ibc_constants::ASSEMBLE_UPDATE_CLIENT_STATIC_ACCOUNTS;

/// Update client message for ICS07 Tendermint
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct UpdateClientMsg {
    pub client_message: Vec<u8>, // Serialized Tendermint header
}

/// Ed25519 signature data for pre-verification
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct SignatureData {
    pub signature_hash: [u8; 32],
    pub pubkey: [u8; 32],
    pub msg: Vec<u8>,
    pub signature: [u8; 64],
}

/// Offset of `is_valid` field in `SignatureVerification` account data.
/// Equals Anchor discriminator length since `is_valid` is the first field.
pub const SIGNATURE_VERIFICATION_IS_VALID_OFFSET: usize =
    solana_ibc_constants::ANCHOR_DISCRIMINATOR_LEN;
