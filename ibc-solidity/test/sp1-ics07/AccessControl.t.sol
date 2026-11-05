// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { ILightClientMsgs } from "../../contracts/msgs/ILightClientMsgs.sol";
import { IICS07TendermintMsgs } from "../../contracts/light-clients/sp1-ics07/msgs/IICS07TendermintMsgs.sol";
import { IICS26ManagedLightClientErrors } from "../../contracts/errors/IICS26ManagedLightClientErrors.sol";

import { Pausable } from "@openzeppelin-contracts/utils/Pausable.sol";

import { SP1ICS07MockTest } from "./SP1ICS07MockTest.sol";

contract SP1ICS07AccessControlTest is SP1ICS07MockTest {
    address public unauthorized = makeAddr("unauthorized");

    function test_success_ics26Router() public view {
        assertEq(ics07Tendermint.ICS26_ROUTER(), ics26Router);
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

    function _isFrozen() private view returns (bool) {
        return abi.decode(ics07Tendermint.getClientState(), (IICS07TendermintMsgs.ClientState)).isFrozen;
    }

    function _expectUnauthorizedCaller(address caller) private {
        vm.expectRevert(
            abi.encodeWithSelector(IICS26ManagedLightClientErrors.LightClientUnauthorizedCaller.selector, caller)
        );
    }
}
