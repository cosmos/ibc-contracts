// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { RLP } from "@openzeppelin-contracts/utils/RLP.sol";
import { Memory } from "@openzeppelin-contracts/utils/Memory.sol";

import { IBesuLightClientMsgs } from "./msgs/IBesuLightClientMsgs.sol";

import { BesuLightClientBase } from "./BesuLightClientBase.sol";
import { Header } from "./utils/Header.sol";

/// @title Besu QBFT Light Client
/// @notice Verifies Besu QBFT headers and ICS26 router storage proofs.
contract BesuQBFTLightClient is BesuLightClientBase {
    using Memory for *;

    /// @notice Creates a Besu QBFT light client from an initial trusted client and consensus state.
    /// @param initialClientState Initial client state. Its `latestHeight` is the height of `initialConsensusState`.
    /// @param initialConsensusState Initial trusted consensus state at `initialClientState.latestHeight`.
    /// @param roleManager Address that administers proof submission; if zero, proof submission is open.
    constructor(
        IBesuLightClientMsgs.ClientState memory initialClientState,
        IBesuLightClientMsgs.ConsensusState memory initialConsensusState,
        address roleManager
    )
        BesuLightClientBase(initialClientState, initialConsensusState, roleManager)
    { }

    /// @inheritdoc BesuLightClientBase
    function _commitSealDigest(Header.Data memory header) internal pure override returns (bytes32) {
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
