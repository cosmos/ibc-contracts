// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { ILightClientMsgs } from "../../contracts/msgs/ILightClientMsgs.sol";

import { IAccessManaged } from "@openzeppelin-contracts/access/manager/IAccessManaged.sol";
import { IBCRolesLib } from "../../contracts/utils/IBCRolesLib.sol";
import { SP1ICS07Tendermint } from "../../contracts/light-clients/sp1-ics07/SP1ICS07Tendermint.sol";

import { SP1ICS07MockTest } from "./SP1ICS07MockTest.sol";

contract SP1ICS07AccessManagedTest is SP1ICS07MockTest {
    address public unauthorized = makeAddr("unauthorized");

    function test_failure_constructor_invalidAuthority() public {
        address[2] memory invalidAuthorities = [address(0), makeAddr("eoa")];
        for (uint256 i = 0; i < invalidAuthorities.length; ++i) {
            vm.expectRevert(
                abi.encodeWithSelector(IAccessManaged.AccessManagedInvalidAuthority.selector, invalidAuthorities[i])
            );
            new SP1ICS07Tendermint(
                MOCK_VKEY,
                MOCK_VKEY,
                MOCK_VKEY,
                MOCK_VKEY,
                address(0),
                abi.encode(mockClientState(1)),
                keccak256(abi.encode(newMockConsensusState(1))),
                invalidAuthorities[i]
            );
        }
    }

    function test_success_updateClient() public {
        bytes memory updateMsg = newUpdateClientMsg();
        vm.prank(relayer);
        ILightClientMsgs.UpdateResult res = ics07Tendermint.updateClient(updateMsg);
        assert(res == ILightClientMsgs.UpdateResult.Update);

        // anyone can update the client once the relayer selectors are public
        accessManagerSetLightClientTargetRoles(accessManager, address(ics07Tendermint), true, false);
        updateMsg = newUpdateClientMsg();
        vm.prank(unauthorized);
        res = ics07Tendermint.updateClient(updateMsg);
        assert(res == ILightClientMsgs.UpdateResult.Update);
    }

    function test_failure_updateClient() public {
        bytes memory updateMsg = newUpdateClientMsg();
        vm.expectRevert(abi.encodeWithSelector(IAccessManaged.AccessManagedUnauthorized.selector, unauthorized));
        vm.prank(unauthorized);
        ics07Tendermint.updateClient(updateMsg);

        // a revoked relayer can no longer update the client
        accessManager.revokeRole(IBCRolesLib.RELAYER_ROLE, relayer);
        vm.expectRevert(abi.encodeWithSelector(IAccessManaged.AccessManagedUnauthorized.selector, relayer));
        vm.prank(relayer);
        ics07Tendermint.updateClient(updateMsg);
    }

    function test_success_verifyMembership() public {
        ILightClientMsgs.MsgVerifyMembership memory membershipMsg = newMembershipMsg(1);
        vm.prank(relayer);
        ics07Tendermint.verifyMembership(membershipMsg);

        accessManagerSetLightClientTargetRoles(accessManager, address(ics07Tendermint), true, false);
        vm.prank(unauthorized);
        ics07Tendermint.verifyMembership(membershipMsg);
    }

    function test_failure_verifyMembership() public {
        ILightClientMsgs.MsgVerifyMembership memory membershipMsg = newMembershipMsg(1);
        vm.expectRevert(abi.encodeWithSelector(IAccessManaged.AccessManagedUnauthorized.selector, unauthorized));
        vm.prank(unauthorized);
        ics07Tendermint.verifyMembership(membershipMsg);
    }

    function test_success_verifyNonMembership() public {
        ILightClientMsgs.MsgVerifyNonMembership memory nonMembershipMsg = newNonMembershipMsg(1);
        vm.prank(relayer);
        ics07Tendermint.verifyNonMembership(nonMembershipMsg);

        accessManagerSetLightClientTargetRoles(accessManager, address(ics07Tendermint), true, false);
        vm.prank(unauthorized);
        ics07Tendermint.verifyNonMembership(nonMembershipMsg);
    }

    function test_failure_verifyNonMembership() public {
        ILightClientMsgs.MsgVerifyNonMembership memory nonMembershipMsg = newNonMembershipMsg(1);
        vm.expectRevert(abi.encodeWithSelector(IAccessManaged.AccessManagedUnauthorized.selector, unauthorized));
        vm.prank(unauthorized);
        ics07Tendermint.verifyNonMembership(nonMembershipMsg);
    }

    function test_success_misbehaviour() public {
        bytes memory misbehaviourMsg = newMisbehaviourMsg();
        vm.prank(relayer);
        ics07Tendermint.misbehaviour(misbehaviourMsg);

        // restart the test since client is frozen
        setUp();

        // anyone can submit misbehaviour once the misbehaviour selector is public
        accessManagerSetLightClientTargetRoles(accessManager, address(ics07Tendermint), false, true);
        vm.prank(unauthorized);
        ics07Tendermint.misbehaviour(misbehaviourMsg);
    }

    function test_failure_misbehaviour() public {
        bytes memory misbehaviourMsg = newMisbehaviourMsg();
        vm.expectRevert(abi.encodeWithSelector(IAccessManaged.AccessManagedUnauthorized.selector, unauthorized));
        vm.prank(unauthorized);
        ics07Tendermint.misbehaviour(misbehaviourMsg);

        // making only the relayer selectors public does not open misbehaviour
        accessManagerSetLightClientTargetRoles(accessManager, address(ics07Tendermint), true, false);
        vm.expectRevert(abi.encodeWithSelector(IAccessManaged.AccessManagedUnauthorized.selector, unauthorized));
        vm.prank(unauthorized);
        ics07Tendermint.misbehaviour(misbehaviourMsg);
    }
}
