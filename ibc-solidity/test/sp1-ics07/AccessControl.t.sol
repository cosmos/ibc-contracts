// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { ILightClientMsgs } from "../../contracts/msgs/ILightClientMsgs.sol";
import { IICS07TendermintMsgs } from "../../contracts/light-clients/sp1-ics07/msgs/IICS07TendermintMsgs.sol";
import { IICS26ManagedLightClientErrors } from "../../contracts/errors/IICS26ManagedLightClientErrors.sol";

import { SP1ICS07Tendermint } from "../../contracts/light-clients/sp1-ics07/SP1ICS07Tendermint.sol";
import { IBCRolesLib } from "../../contracts/utils/IBCRolesLib.sol";

import { Pausable } from "@openzeppelin-contracts/utils/Pausable.sol";
import { AccessManager } from "@openzeppelin-contracts/access/manager/AccessManager.sol";
import { IAccessManaged } from "@openzeppelin-contracts/access/manager/IAccessManaged.sol";

import { SP1ICS07MockTest } from "./SP1ICS07MockTest.sol";

contract SP1ICS07AccessControlTest is SP1ICS07MockTest {
    address public unauthorized = makeAddr("unauthorized");

    function test_success_ics26Router() public view {
        assertEq(ics07Tendermint.ICS26_ROUTER(), ics26Router);
    }

    function test_failure_zeroICS26Router() public {
        vm.expectRevert(abi.encodeWithSelector(IICS26ManagedLightClientErrors.LightClientZeroICS26Router.selector));
        new SP1ICS07Tendermint(
            MOCK_VKEY,
            MOCK_VKEY,
            MOCK_VKEY,
            MOCK_VKEY,
            address(0),
            abi.encode(mockClientState(1)),
            bytes32(0),
            address(0)
        );
    }

    function test_success_updateClient() public {
        bytes memory updateMsg = newUpdateClientMsg();
        vm.prank(ics26Router);
        ILightClientMsgs.UpdateResult res = ics07Tendermint.updateClient(updateMsg);
        assertEq(uint8(res), uint8(ILightClientMsgs.UpdateResult.Update));
    }

    function test_failure_updateClient() public {
        bytes memory updateMsg = newUpdateClientMsg();
        _expectUnauthorizedCaller(unauthorized);
        vm.prank(unauthorized);
        ics07Tendermint.updateClient(updateMsg);
    }

    function test_success_verifyMembership() public {
        ILightClientMsgs.MsgVerifyMembership memory membershipMsg = newMembershipMsg(1);
        vm.prank(ics26Router);
        ics07Tendermint.verifyMembership(membershipMsg);
    }

    function test_failure_verifyMembership() public {
        ILightClientMsgs.MsgVerifyMembership memory membershipMsg = newMembershipMsg(1);
        _expectUnauthorizedCaller(unauthorized);
        vm.prank(unauthorized);
        ics07Tendermint.verifyMembership(membershipMsg);
    }

    function test_success_verifyNonMembership() public {
        ILightClientMsgs.MsgVerifyNonMembership memory nonMembershipMsg = newNonMembershipMsg(1);
        vm.prank(ics26Router);
        ics07Tendermint.verifyNonMembership(nonMembershipMsg);
    }

    function test_failure_verifyNonMembership() public {
        ILightClientMsgs.MsgVerifyNonMembership memory nonMembershipMsg = newNonMembershipMsg(1);
        _expectUnauthorizedCaller(unauthorized);
        vm.prank(unauthorized);
        ics07Tendermint.verifyNonMembership(nonMembershipMsg);
    }

    function test_success_misbehaviour() public {
        bytes memory misbehaviourMsg = newMisbehaviourMsg();
        vm.prank(ics26Router);
        ics07Tendermint.misbehaviour(misbehaviourMsg);
        assertTrue(_isFrozen());
    }

    function test_failure_misbehaviour() public {
        bytes memory misbehaviourMsg = newMisbehaviourMsg();
        _expectUnauthorizedCaller(unauthorized);
        vm.prank(unauthorized);
        ics07Tendermint.misbehaviour(misbehaviourMsg);
    }

    function test_success_pauseAndUnpause() public {
        vm.prank(pauser);
        ics07Tendermint.pause();
        assertTrue(ics07Tendermint.paused());

        bytes memory updateMsg = newUpdateClientMsg();
        ILightClientMsgs.MsgVerifyMembership memory membershipMsg = newMembershipMsg(1);
        ILightClientMsgs.MsgVerifyNonMembership memory nonMembershipMsg = newNonMembershipMsg(1);

        vm.startPrank(ics26Router);
        vm.expectRevert(Pausable.EnforcedPause.selector);
        ics07Tendermint.updateClient(updateMsg);
        vm.expectRevert(Pausable.EnforcedPause.selector);
        ics07Tendermint.verifyMembership(membershipMsg);
        vm.expectRevert(Pausable.EnforcedPause.selector);
        ics07Tendermint.verifyNonMembership(nonMembershipMsg);
        vm.stopPrank();

        vm.prank(unpauser);
        ics07Tendermint.unpause();
        assertFalse(ics07Tendermint.paused());

        vm.startPrank(ics26Router);
        ics07Tendermint.updateClient(updateMsg);
        ics07Tendermint.verifyMembership(membershipMsg);
        ics07Tendermint.verifyNonMembership(nonMembershipMsg);
        vm.stopPrank();
    }

    function test_success_misbehaviourWhilePaused() public {
        vm.prank(pauser);
        ics07Tendermint.pause();

        bytes memory misbehaviourMsg = newMisbehaviourMsg();
        vm.prank(ics26Router);
        ics07Tendermint.misbehaviour(misbehaviourMsg);
        assertTrue(_isFrozen());
    }

    function test_failure_pause() public {
        // neither an unrelated account, the router nor the unpauser can pause
        address[3] memory callers = [unauthorized, ics26Router, unpauser];
        for (uint256 i = 0; i < callers.length; ++i) {
            _expectUnauthorizedRole(callers[i], IBCRolesLib.PAUSER_ROLE);
            vm.prank(callers[i]);
            ics07Tendermint.pause();
        }

        // a pauser with an execution delay cannot pause, since scheduled operations are not supported
        address delayedPauser = makeAddr("delayedPauser");
        accessManager.grantRole(IBCRolesLib.PAUSER_ROLE, delayedPauser, 1 days);
        _expectUnauthorizedRole(delayedPauser, IBCRolesLib.PAUSER_ROLE);
        vm.prank(delayedPauser);
        ics07Tendermint.pause();

        assertFalse(ics07Tendermint.paused());
    }

    function test_failure_unpause() public {
        vm.prank(pauser);
        ics07Tendermint.pause();

        // neither an unrelated account, the router nor the pauser can unpause
        address[3] memory callers = [unauthorized, ics26Router, pauser];
        for (uint256 i = 0; i < callers.length; ++i) {
            _expectUnauthorizedRole(callers[i], IBCRolesLib.UNPAUSER_ROLE);
            vm.prank(callers[i]);
            ics07Tendermint.unpause();
        }

        assertTrue(ics07Tendermint.paused());
    }

    function test_success_pauseFollowsRouterAuthority() public {
        AccessManager newAccessManager = new AccessManager(address(this));
        address newPauser = makeAddr("newPauser");
        newAccessManager.grantRole(IBCRolesLib.PAUSER_ROLE, newPauser, 0);

        vm.mockCall(ics26Router, abi.encodeCall(IAccessManaged.authority, ()), abi.encode(address(newAccessManager)));

        // the pauser of the previous authority is no longer recognized
        _expectUnauthorizedRole(pauser, IBCRolesLib.PAUSER_ROLE);
        vm.prank(pauser);
        ics07Tendermint.pause();

        vm.prank(newPauser);
        ics07Tendermint.pause();
        assertTrue(ics07Tendermint.paused());
    }

    function _isFrozen() private view returns (bool) {
        return abi.decode(ics07Tendermint.getClientState(), (IICS07TendermintMsgs.ClientState)).isFrozen;
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
