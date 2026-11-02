// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { ILightClient } from "../../../interfaces/ILightClient.sol";

/// @title IBesuLightClient
/// @notice Interface for Besu BFT light clients.
interface IBesuLightClient is ILightClient {
    /// @notice Emitted when two validly signed headers commit different consensus states at the same height.
    /// @dev The client is permanently frozen when this is emitted.
    /// @param revisionHeight The height at which the double sign occurred.
    /// @param consensusStateHash1 The first consensus state hash: the one stored at `revisionHeight` for
    /// `updateClient`, or the one derived from the first header of a `MsgConflictingHeadersMisbehaviour`.
    /// @param consensusStateHash2 The second consensus state hash: the one derived from the submitted header for
    /// `updateClient`, or from the second header of a `MsgConflictingHeadersMisbehaviour`.
    event DoubleSign(uint64 indexed revisionHeight, bytes32 consensusStateHash1, bytes32 consensusStateHash2);

    /// @notice Emitted when a time monotonicity violation is detected between two validly signed headers.
    /// @dev The client is permanently frozen when this is emitted.
    /// @param height2 The height of the second trusted consensus state.
    /// @param height1 The height of the first trusted consensus state. Less than `height2`.
    /// @param timestamp2 The timestamp of the second trusted consensus state.
    /// @param timestamp1 The timestamp of the first trusted consensus state.
    // solhint-disable-next-line gas-indexed-events
    event TimeNonMonotonicity(uint64 indexed height2, uint64 height1, uint64 timestamp2, uint64 timestamp1);

    /// @notice Emitted when the timestamps of two validly signed consensus states increase by less than one second per
    /// block, which a chain with a block period of at least one second cannot produce.
    /// @dev The client is permanently frozen when this is emitted. Sub-second block periods are not supported.
    /// @param height2 The height of the second trusted consensus state.
    /// @param height1 The height of the first trusted consensus state. Less than `height2`.
    /// @param timestamp2 The timestamp of the second trusted consensus state. Greater than `timestamp1` and less than
    /// `timestamp1 + height2 - height1`.
    /// @param timestamp1 The timestamp of the first trusted consensus state.
    // solhint-disable-next-line gas-indexed-events
    event SubSecondBlockTime(uint64 indexed height2, uint64 height1, uint64 timestamp2, uint64 timestamp1);

    /// @notice Emitted when two validly signed consensus states have validator sets that differ by more validators
    /// than their height difference, which a header-validator vote of at most one change per block cannot produce.
    /// @dev The client is permanently frozen when this is emitted.
    /// @param height2 The height of the second trusted consensus state.
    /// @param height1 The height of the first trusted consensus state. Less than `height2`.
    /// @param changes The number of validators in exactly one of the two validator sets. Greater than
    /// `height2 - height1`.
    // solhint-disable-next-line gas-indexed-events
    event InvalidValidatorSetChange(uint64 indexed height2, uint64 height1, uint256 changes);

    /// @notice Returns the stored consensus state hash at a revision height.
    /// @dev The hash is `keccak256(abi.encode(IBesuLightClientMsgs.ConsensusState))`.
    /// Reverts with `ConsensusStateNotFound` if no consensus state is stored at the height.
    /// @param revisionHeight The revision height to query.
    /// @return The stored consensus state hash.
    function getConsensusStateHash(uint64 revisionHeight) external view returns (bytes32);
}
