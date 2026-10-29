// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { IPausable } from "./IPausable.sol";

/// @title ICS26 Managed Light Client Interface
/// @notice Interface for light clients that are bound to a single `ICS26Router` and are paused through its authority.
interface IICS26ManagedLightClient is IPausable {
    /// @notice The `ICS26Router` that this light client is bound to.
    /// @dev Only this address can call `updateClient`, `verifyMembership`, `verifyNonMembership` and `misbehaviour`.
    /// @dev `pause` and `unpause` are authorized by the router's `AccessManager` through `PAUSER_ROLE` and
    /// `UNPAUSER_ROLE` from `IBCRolesLib`.
    /// @return The `ICS26Router` address
    function ICS26_ROUTER() external view returns (address);
}
