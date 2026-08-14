// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

/// @title IAttestationLightClientMsgs
/// @notice Types for the attestor-based IBC light client
interface IAttestationLightClientMsgs {
    /// @notice Client state for the attestor light client
    /// @param attestorAddresses Fixed set of attestor EOAs
    /// @param minRequiredSigs Quorum threshold
    /// @param latestHeight Highest known height
    /// @param isFrozen Reserved for misbehaviour (not used in this version)
    struct ClientState {
        address[] attestorAddresses;
        uint8 minRequiredSigs;
        uint64 latestHeight;
        bool isFrozen;
    }
}
