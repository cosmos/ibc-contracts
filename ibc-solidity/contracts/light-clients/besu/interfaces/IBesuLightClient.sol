// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { ILightClient } from "../../../interfaces/ILightClient.sol";

/// @title IBesuLightClient
/// @notice Interface for Besu BFT light clients.
interface IBesuLightClient is ILightClient {
    /// @notice Returns the stored consensus state hash at a revision height.
    /// @dev The hash is `keccak256(abi.encode(IBesuLightClientMsgs.ConsensusState))`.
    /// Reverts with `ConsensusStateNotFound` if no consensus state is stored at the height.
    /// @param revisionHeight The revision height to query.
    /// @return The stored consensus state hash.
    function getConsensusStateHash(uint64 revisionHeight) external view returns (bytes32);
}
