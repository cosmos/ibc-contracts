// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

/// @title ICS26 Managed Light Client Errors
/// @notice Interface for errors of light clients bound to an `ICS26Router`
interface IICS26ManagedLightClientErrors {
    /// @notice The `ICS26Router` address cannot be the zero address
    error LightClientZeroICS26Router();

    /// @notice The caller is not the `ICS26Router` that this light client is bound to
    /// @param caller The unauthorized caller
    error LightClientUnauthorizedCaller(address caller);

    /// @notice The caller does not have the required role, without an execution delay, in the router's authority
    /// @param caller The unauthorized caller
    /// @param roleId The required `AccessManager` role
    error LightClientUnauthorizedRole(address caller, uint64 roleId);
}
