// SPDX-License-Identifier: Apache-2.0

//! ICS-25 Vector Commitments for IBC light clients on Solana
//!
//! This crate defines the standard interface that all IBC light clients must implement
//! to be compatible with the ICS-26 router for membership and non-membership verification.
//!
//! ICS-25 specifies the interface for vector commitment schemes used in IBC to verify
//! inclusion or non-inclusion of values at specific paths in a commitment root.

use anchor_lang::prelude::*;

/// Standard instruction discriminators for IBC light client operations
pub mod discriminators {
    use solana_ibc_macros::discriminator;

    /// Discriminator for `verify_membership` instruction
    /// Computed from `sha256("global:verify_membership")`
    pub const VERIFY_MEMBERSHIP: [u8; 8] = discriminator!("verify_membership");

    /// Discriminator for `verify_non_membership` instruction
    /// Computed from `sha256("global:verify_non_membership")`
    pub const VERIFY_NON_MEMBERSHIP: [u8; 8] = discriminator!("verify_non_membership");

    /// Discriminator for `client_status` instruction
    /// Computed from `sha256("global:client_status")`
    pub const CLIENT_STATUS: [u8; 8] = discriminator!("client_status");
}

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

#[cfg(test)]
mod tests {
    use super::discriminators::*;

    /// Bytes as emitted in the deployed light client IDLs.
    #[test]
    fn discriminators_match_deployed_idl() {
        assert_eq!(VERIFY_MEMBERSHIP, [101, 53, 78, 0, 103, 151, 236, 209]);
        assert_eq!(VERIFY_NON_MEMBERSHIP, [231, 161, 86, 239, 111, 236, 14, 74]);
        assert_eq!(CLIENT_STATUS, [75, 93, 126, 21, 10, 112, 109, 139]);
    }
}
