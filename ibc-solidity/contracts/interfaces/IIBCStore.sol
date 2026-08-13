// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

/// @title IBC Store Interface
/// @notice The public interface for provable IBC Store.
/// @dev This is the contract whose state is tracked by the counterparty light client.
interface IIBCStore {
    /// @notice Gets the commitment for a given path.
    /// @param hashedPath The hashed path to get the commitment for.
    /// @return The commitment for the given path.
    function getCommitment(bytes32 hashedPath) external view returns (bytes32);
}
