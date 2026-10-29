// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { IICS26ManagedLightClient } from "../../interfaces/IICS26ManagedLightClient.sol";
import { IICS26ManagedLightClientErrors } from "../../errors/IICS26ManagedLightClientErrors.sol";
import { IPausable } from "../../interfaces/IPausable.sol";

import { IBCRolesLib } from "../../utils/IBCRolesLib.sol";
import { Pausable } from "@openzeppelin-contracts/utils/Pausable.sol";
import { IAccessManaged } from "@openzeppelin-contracts/access/manager/IAccessManaged.sol";
import { IAccessManager } from "@openzeppelin-contracts/access/manager/IAccessManager.sol";

/// @title ICS26 Managed Light Client
/// @notice Base for light clients that are only driven by a single `ICS26Router` and paused through its authority.
/// @dev Light clients gate `updateClient`, `verifyMembership`, `verifyNonMembership` and `misbehaviour` with
/// `onlyICS26Router`, and the first three also with `whenNotPaused`. Pause roles are looked up in the router's current
/// `AccessManager`, so no per-client role configuration is needed.
abstract contract ICS26ManagedLightClient is IICS26ManagedLightClient, IICS26ManagedLightClientErrors, Pausable {
    /// @inheritdoc IICS26ManagedLightClient
    address public immutable ICS26_ROUTER;

    /// @notice Binds the light client to an `ICS26Router`.
    /// @param ics26Router The `ICS26Router` that may drive this light client.
    constructor(address ics26Router) {
        require(ics26Router != address(0), LightClientZeroICS26Router());
        ICS26_ROUTER = ics26Router;
    }

    /// @inheritdoc IPausable
    /// @dev The caller must have `IBCRolesLib.PAUSER_ROLE` in the router's authority.
    function pause() external onlyAuthorityRole(IBCRolesLib.PAUSER_ROLE) {
        _pause();
    }

    /// @inheritdoc IPausable
    /// @dev The caller must have `IBCRolesLib.UNPAUSER_ROLE` in the router's authority.
    function unpause() external onlyAuthorityRole(IBCRolesLib.UNPAUSER_ROLE) {
        _unpause();
    }

    /// @notice Reverts unless the caller is `ICS26_ROUTER`.
    modifier onlyICS26Router() {
        require(_msgSender() == ICS26_ROUTER, LightClientUnauthorizedCaller(_msgSender()));
        _;
    }

    /// @notice Reverts unless the caller has `roleId` in the router's authority.
    /// @dev Members with an execution delay are rejected, since this contract cannot consume scheduled operations.
    /// @param roleId The required `AccessManager` role.
    modifier onlyAuthorityRole(uint64 roleId) {
        IAccessManager authority = IAccessManager(IAccessManaged(ICS26_ROUTER).authority());
        (bool isMember, uint32 executionDelay) = authority.hasRole(roleId, _msgSender());
        require(isMember && executionDelay == 0, LightClientUnauthorizedRole(_msgSender(), roleId));
        _;
    }
}
