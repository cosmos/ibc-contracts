// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import { BesuLightClientBase } from "./BesuLightClientBase.sol";

/// @title Besu IBFT2 Light Client
/// @notice Verifies Besu IBFT 2.0 headers and ICS26 router storage proofs.
contract BesuIBFT2LightClient is BesuLightClientBase {
    /// @notice Creates a Besu IBFT2 light client from an initial trusted consensus state.
    /// @param ibcRouter Counterparty ICS26 router address whose storage is proven.
    /// @param initialTrustedHeight Initial trusted Besu height.
    /// @param initialTrustedTimestamp Initial trusted header timestamp in seconds.
    /// @param initialTrustedStorageRoot Initial trusted storage root of `ibcRouter`.
    /// @param initialTrustedValidators Initial trusted validator set.
    /// @param trustingPeriod Maximum age in seconds for trusted consensus states.
    /// @param maxClockDrift Maximum allowed future drift in seconds for submitted headers.
    /// @param roleManager Address that administers proof submission; if zero, proof submission is open.
    constructor(
        address ibcRouter,
        uint64 initialTrustedHeight,
        uint64 initialTrustedTimestamp,
        bytes32 initialTrustedStorageRoot,
        address[] memory initialTrustedValidators,
        uint64 trustingPeriod,
        uint64 maxClockDrift,
        address roleManager
    )
        BesuLightClientBase(
            ibcRouter,
            initialTrustedHeight,
            initialTrustedTimestamp,
            initialTrustedStorageRoot,
            initialTrustedValidators,
            trustingPeriod,
            maxClockDrift,
            roleManager
        )
    { }

    /// @inheritdoc BesuLightClientBase
    function _commitSealDigest(ParsedHeader memory header) internal pure override returns (bytes32) {
        bytes[] memory extraItems = new bytes[](4);
        extraItems[0] = _rlpItemBytes(header.extraDataItems[0]);
        extraItems[1] = _rlpItemBytes(header.extraDataItems[1]);
        extraItems[2] = _rlpItemBytes(header.extraDataItems[2]);
        extraItems[3] = _rlpItemBytes(header.extraDataItems[3]);

        bytes memory signingExtraData = _encodeRlpList(extraItems);
        bytes[] memory headerItems = new bytes[](header.headerItems.length);
        for (uint256 i = 0; i < header.headerItems.length; ++i) {
            headerItems[i] = i == 12 ? _encodeRlpBytes(signingExtraData) : _rlpItemBytes(header.headerItems[i]);
        }
        return keccak256(_encodeRlpList(headerItems));
    }
}
