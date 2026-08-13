// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

/// @title IICS02PrecompileWrapperErrors
/// @notice Interface for escrow-related errors
interface IICS02PrecompileWrapperErrors {
    /// @notice Unreachable code path
    error Unreachable();

    /// @notice No misbehaviour detected during update
    error NoMisbehaviourDetected();
}
