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

    /// @notice Emitted when a newer header is submitted with a timestamp that is less than or equal to the trusted
    /// consensus state timestamp.
    /// @dev The client is permanently frozen when this is emitted.
    /// @param revisionHeight The height of the newly submitted header.
    /// @param trustedHeight The height of the trusted consensus state.
    /// @param newTimestamp The timestamp of the newly submitted header.
    /// @param trustedTimestamp The timestamp of the trusted consensus state.
    event TimeNonMonotonicity(
        uint64 indexed revisionHeight, uint64 trustedHeight, uint64 newTimestamp, uint64 trustedTimestamp
    );

    /// @notice Returns the stored consensus state hash at a revision height.
    /// @dev The hash is `keccak256(abi.encode(IBesuLightClientMsgs.ConsensusState))`.
    /// Reverts with `ConsensusStateNotFound` if no consensus state is stored at the height.
    /// @param revisionHeight The revision height to query.
    /// @return The stored consensus state hash.
    function getConsensusStateHash(uint64 revisionHeight) external view returns (bytes32);
}
