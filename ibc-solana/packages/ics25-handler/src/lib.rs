// SPDX-License-Identifier: Apache-2.0

//! ICS-25 Vector Commitments for IBC light clients on Solana
//!
//! This crate defines the standard interface that all IBC light clients must implement
//! to be compatible with the ICS-26 router for membership and non-membership verification.
//!
//! ICS-25 specifies the interface for vector commitment schemes used in IBC to verify
//! inclusion or non-inclusion of values at specific paths in a commitment root.

use anchor_lang::prelude::*;

// Include generated discriminators from build.rs
include!(concat!(env!("OUT_DIR"), "/discriminators.rs"));

/// Standard message structure for membership verification
/// All light clients must accept this structure
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct MembershipMsg {
    /// The height at which to verify
    pub height: u64,
    /// The merkle proof
    pub proof: Vec<u8>,
    /// The merkle path to the value
    pub path: Vec<Vec<u8>>,
    /// The value to verify
    pub value: Vec<u8>,
}

/// Standard message structure for non-membership verification
/// All light clients must accept this structure
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq, Eq)]
pub struct NonMembershipMsg {
    /// The height at which to verify
    pub height: u64,
    /// The merkle proof
    pub proof: Vec<u8>,
    /// The merkle path to the value
    pub path: Vec<Vec<u8>>,
}

/// Client status returned by `client_status` instruction via `set_return_data`
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum ClientStatus {
    Active = 0,
    Frozen = 1,
    Expired = 2,
}

impl core::fmt::Display for ClientStatus {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        core::fmt::Debug::fmt(self, f)
    }
}

impl From<ClientStatus> for u8 {
    fn from(status: ClientStatus) -> Self {
        status as Self
    }
}

impl TryFrom<u8> for ClientStatus {
    type Error = u8;

    fn try_from(value: u8) -> core::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Active),
            1 => Ok(Self::Frozen),
            2 => Ok(Self::Expired),
            other => Err(other),
        }
    }
}
