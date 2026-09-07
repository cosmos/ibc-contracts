// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { RLP } from "@openzeppelin-contracts/utils/RLP.sol";
import { Memory } from "@openzeppelin-contracts/utils/Memory.sol";

import { BesuLightClientBase } from "./BesuLightClientBase.sol";

/// @title Besu QBFT Light Client
/// @notice Verifies Besu QBFT headers and ICS26 router storage proofs.
contract BesuQBFTLightClient is BesuLightClientBase {
    using Memory for *;

    /// @notice Creates a Besu QBFT light client from an initial trusted consensus state.
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
        bytes[] memory extraItems = new bytes[](5);
        extraItems[0] = header.extraDataItems[0].toBytes();
        extraItems[1] = header.extraDataItems[1].toBytes();
        extraItems[2] = header.extraDataItems[2].toBytes();
        extraItems[3] = header.extraDataItems[3].toBytes();
        extraItems[4] = hex"c0";

        bytes memory signingExtraData = RLP.encode(extraItems);
        bytes[] memory headerItems = new bytes[](header.headerItems.length);
        for (uint256 i = 0; i < header.headerItems.length; ++i) {
            headerItems[i] = i == 12 ? RLP.encode(signingExtraData) : header.headerItems[i].toBytes();
        }
        return keccak256(RLP.encode(headerItems));
    }
}
