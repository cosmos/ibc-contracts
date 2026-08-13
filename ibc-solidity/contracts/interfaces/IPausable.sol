// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

/// @title IPausable
/// @notice Interface for pausable contracts for internal use.
interface IPausable {
    /// @notice Pauses the contract
    /// @dev This call is restricted
    function pause() external;

    /// @notice Unpauses the contract
    /// @dev This call is restricted
    function unpause() external;
}
