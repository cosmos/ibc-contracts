// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { ILightClient } from "../../../interfaces/ILightClient.sol";

/// @title IBesuLightClient
/// @notice Interface for Besu BFT light clients.
interface IBesuLightClient is ILightClient {
    /// @notice Emitted when two validly signed headers commit different consensus states at the same height.
    /// @dev The client is permanently frozen when this is emitted.
    /// @param revisionHeight The height at which the double sign occurred.
    /// @param trustedConsensusStateHash The consensus state hash already stored at `revisionHeight`.
    /// @param conflictingConsensusStateHash The conflicting consensus state hash derived from the submitted header.
    event DoubleSign(
        uint64 indexed revisionHeight, bytes32 trustedConsensusStateHash, bytes32 conflictingConsensusStateHash
    );

    /// @notice Emitted when a time monotonicity violation is detected between two validly signed headers.
    /// @dev The client is permanently frozen when this is emitted.
    /// @param height2 The height of the second trusted consensus state.
    /// @param height1 The height of the first trusted consensus state. Less than `height2`.
    /// @param timestamp2 The timestamp of the second trusted consensus state.
    /// @param timestamp1 The timestamp of the first trusted consensus state.
    event TimeNonMonotonicity(uint64 indexed height2, uint64 height1, uint64 timestamp2, uint64 timestamp1);

    /// @notice Returns the stored consensus state hash at a revision height.
    /// @dev The hash is `keccak256(abi.encode(IBesuLightClientMsgs.ConsensusState))`.
    /// Reverts with `ConsensusStateNotFound` if no consensus state is stored at the height.
    /// @param revisionHeight The revision height to query.
    /// @return The stored consensus state hash.
    function getConsensusStateHash(uint64 revisionHeight) external view returns (bytes32);
}
