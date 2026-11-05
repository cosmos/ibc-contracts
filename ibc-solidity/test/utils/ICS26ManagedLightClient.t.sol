// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { Test } from "forge-std/Test.sol";

import { IICS26ManagedLightClientErrors } from "../../contracts/errors/IICS26ManagedLightClientErrors.sol";
import { ICS26ManagedLightClient } from "../../contracts/light-clients/utils/ICS26ManagedLightClient.sol";
import { IBCRolesLib } from "../../contracts/utils/IBCRolesLib.sol";

import { Pausable } from "@openzeppelin-contracts/utils/Pausable.sol";
import { AccessManager } from "@openzeppelin-contracts/access/manager/AccessManager.sol";
import { IAccessManaged } from "@openzeppelin-contracts/access/manager/IAccessManaged.sol";

/// @dev Light client stub with one entry point in each gating shape the light clients use.
contract ICS26ManagedLightClientMock is ICS26ManagedLightClient {
    constructor(address ics26Router) ICS26ManagedLightClient(ics26Router) { }

    /// @dev Gated like `updateClient`, `verifyMembership` and `verifyNonMembership`.
    function gated() external view whenNotPaused onlyICS26Router { }
    // solhint-disable-previous-line no-empty-blocks

    /// @dev Gated like `misbehaviour`, which stays callable while paused.
    function gatedWhilePaused() external view onlyICS26Router { }
    // solhint-disable-previous-line no-empty-blocks
}

/// @dev Covers the router binding and pause authorization shared by every light client. Each light client suite only
/// tests which of its entry points are gated.
contract ICS26ManagedLightClientTest is Test {
    ICS26ManagedLightClientMock internal client;
    AccessManager internal accessManager;

    address internal ics26Router = makeAddr("ics26Router");
    address internal pauser = makeAddr("pauser");
    address internal unpauser = makeAddr("unpauser");
    address internal unauthorized = makeAddr("unauthorized");

    function setUp() public {
        accessManager = new AccessManager(address(this));
        accessManager.grantRole(IBCRolesLib.PAUSER_ROLE, pauser, 0);
        accessManager.grantRole(IBCRolesLib.UNPAUSER_ROLE, unpauser, 0);
        vm.mockCall(ics26Router, abi.encodeCall(IAccessManaged.authority, ()), abi.encode(address(accessManager)));

        client = new ICS26ManagedLightClientMock(ics26Router);
    }

    function test_success_ics26Router() public view {
        assertEq(client.ICS26_ROUTER(), ics26Router);
    }

    function test_failure_zeroICS26Router() public {
        vm.expectRevert(abi.encodeWithSelector(IICS26ManagedLightClientErrors.LightClientZeroICS26Router.selector));
        new ICS26ManagedLightClientMock(address(0));
    }

    function test_onlyICS26Router() public {
        address[3] memory callers = [unauthorized, pauser, address(this)];
        for (uint256 i = 0; i < callers.length; ++i) {
            vm.startPrank(callers[i]);
            _expectUnauthorizedCaller(callers[i]);
            client.gated();
            _expectUnauthorizedCaller(callers[i]);
            client.gatedWhilePaused();
            vm.stopPrank();
        }

        vm.startPrank(ics26Router);
        client.gated();
        client.gatedWhilePaused();
        vm.stopPrank();
    }

    function test_success_pauseAndUnpause() public {
        vm.expectEmit(address(client));
        emit Pausable.Paused(pauser);
        vm.prank(pauser);
        client.pause();
        assertTrue(client.paused());

        vm.startPrank(ics26Router);
        vm.expectRevert(Pausable.EnforcedPause.selector);
        client.gated();
        client.gatedWhilePaused();
        vm.stopPrank();

        vm.expectEmit(address(client));
        emit Pausable.Unpaused(unpauser);
        vm.prank(unpauser);
        client.unpause();
        assertFalse(client.paused());

        vm.prank(ics26Router);
        client.gated();
    }

    function test_failure_pause() public {
        // neither an unrelated account, the router nor the unpauser can pause
        address[3] memory callers = [unauthorized, ics26Router, unpauser];
        for (uint256 i = 0; i < callers.length; ++i) {
            _expectUnauthorizedRole(callers[i], IBCRolesLib.PAUSER_ROLE);
            vm.prank(callers[i]);
            client.pause();
        }

        // a pauser with an execution delay cannot pause, since scheduled operations are not supported
        address delayedPauser = makeAddr("delayedPauser");
        accessManager.grantRole(IBCRolesLib.PAUSER_ROLE, delayedPauser, 1 days);
        _expectUnauthorizedRole(delayedPauser, IBCRolesLib.PAUSER_ROLE);
        vm.prank(delayedPauser);
        client.pause();

        assertFalse(client.paused());

        // pausing twice is rejected
        vm.prank(pauser);
        client.pause();
        vm.expectRevert(Pausable.EnforcedPause.selector);
        vm.prank(pauser);
        client.pause();
    }

    function test_failure_unpause() public {
        // unpausing an unpaused client is rejected
        vm.expectRevert(Pausable.ExpectedPause.selector);
        vm.prank(unpauser);
        client.unpause();

        vm.prank(pauser);
        client.pause();

        // neither an unrelated account, the router nor the pauser can unpause
        address[3] memory callers = [unauthorized, ics26Router, pauser];
        for (uint256 i = 0; i < callers.length; ++i) {
            _expectUnauthorizedRole(callers[i], IBCRolesLib.UNPAUSER_ROLE);
            vm.prank(callers[i]);
            client.unpause();
        }

        // an unpauser with an execution delay cannot unpause, since scheduled operations are not supported
        address delayedUnpauser = makeAddr("delayedUnpauser");
        accessManager.grantRole(IBCRolesLib.UNPAUSER_ROLE, delayedUnpauser, 1 days);
        _expectUnauthorizedRole(delayedUnpauser, IBCRolesLib.UNPAUSER_ROLE);
        vm.prank(delayedUnpauser);
        client.unpause();

        assertTrue(client.paused());
    }

    function test_success_pauseFollowsRouterAuthority() public {
        AccessManager newAccessManager = new AccessManager(address(this));
        address newPauser = makeAddr("newPauser");
        newAccessManager.grantRole(IBCRolesLib.PAUSER_ROLE, newPauser, 0);

        vm.mockCall(ics26Router, abi.encodeCall(IAccessManaged.authority, ()), abi.encode(address(newAccessManager)));

        // the pauser of the previous authority is no longer recognized
        _expectUnauthorizedRole(pauser, IBCRolesLib.PAUSER_ROLE);
        vm.prank(pauser);
        client.pause();

        vm.prank(newPauser);
        client.pause();
        assertTrue(client.paused());
    }

    function _expectUnauthorizedCaller(address caller) private {
        vm.expectRevert(
            abi.encodeWithSelector(IICS26ManagedLightClientErrors.LightClientUnauthorizedCaller.selector, caller)
        );
    }

    function _expectUnauthorizedRole(address caller, uint64 roleId) private {
        vm.expectRevert(
            abi.encodeWithSelector(IICS26ManagedLightClientErrors.LightClientUnauthorizedRole.selector, caller, roleId)
        );
    }
}
