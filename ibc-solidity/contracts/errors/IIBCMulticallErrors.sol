// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

/// @title IIBCMulticallErrors
/// @notice Interface for IBCMulticall errors
interface IIBCMulticallErrors {
    /// @notice A call that is allowed to fail most likely ran out of gas, so the batch is reverted instead of dropping
    /// it
    /// @dev Raised for empty revert data and for the router's `IBCFailedCallback`, which it raises for an app callback
    /// that reverted without a reason
    /// @param index The index of the call in the batch
    error IBCMulticallCallOutOfGas(uint256 index);
}
