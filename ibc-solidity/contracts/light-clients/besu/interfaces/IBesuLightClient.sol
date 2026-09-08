// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { ILightClient } from "../../../interfaces/ILightClient.sol";

/// @title IBesuLightClient
/// @notice Interface for Besu BFT light clients.
interface IBesuLightClient is ILightClient {
    /// @notice Returns the trusted consensus state at a revision height.
    /// @param revisionHeight The revision height to query.
    /// @return The ABI-encoded consensus state.
    function getConsensusState(uint64 revisionHeight) external view returns (bytes memory);
}
