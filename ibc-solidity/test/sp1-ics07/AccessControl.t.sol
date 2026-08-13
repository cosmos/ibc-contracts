// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { ILightClientMsgs } from "../../contracts/msgs/ILightClientMsgs.sol";

import { IAccessControl } from "@openzeppelin-contracts/access/IAccessControl.sol";

import { SP1ICS07MockTest } from "./SP1ICS07MockTest.sol";

contract SP1ICS07AccessControlTest is SP1ICS07MockTest {
    function test_success_setProofSubmitter() public {
        bytes32 defaultAdminRole = ics07Tendermint.DEFAULT_ADMIN_ROLE();
        bytes32 proofSubmitterRole = ics07Tendermint.PROOF_SUBMITTER_ROLE();

        // Ensure that the submitter is the role manager
        assert(ics07Tendermint.hasRole(defaultAdminRole, roleManager));
        assert(ics07Tendermint.hasRole(proofSubmitterRole, roleManager));

        address newSubmitter = makeAddr("newSubmitter");
        vm.prank(roleManager);
        ics07Tendermint.grantRole(proofSubmitterRole, newSubmitter);
        assert(ics07Tendermint.hasRole(proofSubmitterRole, newSubmitter));

        vm.prank(roleManager);
        ics07Tendermint.revokeRole(proofSubmitterRole, newSubmitter);
        assertFalse(ics07Tendermint.hasRole(proofSubmitterRole, newSubmitter));
    }

    function test_failure_setProofSubmitter() public {
        bytes32 defaultAdminRole = ics07Tendermint.DEFAULT_ADMIN_ROLE();
        bytes32 proofSubmitterRole = ics07Tendermint.PROOF_SUBMITTER_ROLE();

        address unauthorized = makeAddr("unauthorized");
        address newSubmitter = makeAddr("newSubmitter");
        vm.expectRevert(
            abi.encodeWithSelector(
                IAccessControl.AccessControlUnauthorizedAccount.selector, unauthorized, defaultAdminRole
            )
        );
        vm.prank(unauthorized);
        ics07Tendermint.grantRole(proofSubmitterRole, newSubmitter);
        assertFalse(ics07Tendermint.hasRole(proofSubmitterRole, newSubmitter));

        vm.expectRevert(
            abi.encodeWithSelector(
                IAccessControl.AccessControlUnauthorizedAccount.selector, unauthorized, defaultAdminRole
            )
        );
        vm.prank(unauthorized);
        ics07Tendermint.revokeRole(proofSubmitterRole, roleManager);
        assert(ics07Tendermint.hasRole(proofSubmitterRole, roleManager));
    }

    function test_success_roleRenounce() public {
        bytes32 defaultAdminRole = ics07Tendermint.DEFAULT_ADMIN_ROLE();
        bytes32 proofSubmitterRole = ics07Tendermint.PROOF_SUBMITTER_ROLE();

        // give the submitter the role
        address newSubmitter = makeAddr("newSubmitter");
        vm.prank(roleManager);
        ics07Tendermint.grantRole(proofSubmitterRole, newSubmitter);
        assert(ics07Tendermint.hasRole(proofSubmitterRole, newSubmitter));

        // renounce the submitter role
        vm.prank(newSubmitter);
        ics07Tendermint.renounceRole(proofSubmitterRole, newSubmitter);
        assertFalse(ics07Tendermint.hasRole(proofSubmitterRole, newSubmitter));

        // renounce the admin role
        vm.prank(roleManager);
        ics07Tendermint.renounceRole(defaultAdminRole, roleManager);
        assertFalse(ics07Tendermint.hasRole(defaultAdminRole, roleManager));
    }

    function test_failure_roleRenounce() public {
        bytes32 defaultAdminRole = ics07Tendermint.DEFAULT_ADMIN_ROLE();
        bytes32 proofSubmitterRole = ics07Tendermint.PROOF_SUBMITTER_ROLE();

        // give the submitter the role
        address unauthorized = makeAddr("unauthorized");
        address newSubmitter = makeAddr("newSubmitter");
        vm.prank(roleManager);
        ics07Tendermint.grantRole(proofSubmitterRole, newSubmitter);
        assert(ics07Tendermint.hasRole(proofSubmitterRole, newSubmitter));

        // fail to renounce the submitter role
        vm.expectRevert(abi.encodeWithSelector(IAccessControl.AccessControlBadConfirmation.selector));
        vm.prank(unauthorized);
        ics07Tendermint.renounceRole(proofSubmitterRole, newSubmitter);
        assert(ics07Tendermint.hasRole(proofSubmitterRole, newSubmitter));

        // fail to renounce the admin role
        vm.expectRevert(abi.encodeWithSelector(IAccessControl.AccessControlBadConfirmation.selector));
        vm.prank(unauthorized);
        ics07Tendermint.renounceRole(defaultAdminRole, roleManager);
        assert(ics07Tendermint.hasRole(defaultAdminRole, roleManager));
    }

    function test_success_updateClient() public {
        bytes32 proofSubmitterRole = ics07Tendermint.PROOF_SUBMITTER_ROLE();

        // role manager is the submitter
        bytes memory updateMsg = newUpdateClientMsg();
        vm.prank(roleManager);
        ILightClientMsgs.UpdateResult res = ics07Tendermint.updateClient(updateMsg);
        assert(res == ILightClientMsgs.UpdateResult.Update);

        // submitter is not the role manager
        updateMsg = newUpdateClientMsg();
        vm.prank(proofSubmitter);
        res = ics07Tendermint.updateClient(updateMsg);
        assert(res == ILightClientMsgs.UpdateResult.Update);

        // role manager allows anyone to update the client
        vm.prank(roleManager);
        ics07Tendermint.grantRole(proofSubmitterRole, address(0));

        // anyone can update the client
        address anyAddr = makeAddr("anyAddr");
        updateMsg = newUpdateClientMsg();
        vm.prank(anyAddr);
        res = ics07Tendermint.updateClient(updateMsg);
        assert(res == ILightClientMsgs.UpdateResult.Update);
    }

    function test_failure_updateClient() public {
        // unauthorized account
        address unauthorized = makeAddr("unauthorized");
        bytes memory updateMsg = newUpdateClientMsg();
        vm.expectRevert(
            abi.encodeWithSelector(
                IAccessControl.AccessControlUnauthorizedAccount.selector,
                unauthorized,
                ics07Tendermint.PROOF_SUBMITTER_ROLE()
            )
        );
        vm.prank(unauthorized);
        ics07Tendermint.updateClient(updateMsg);
    }

    function test_success_verifyMembership() public {
        bytes32 proofSubmitterRole = ics07Tendermint.PROOF_SUBMITTER_ROLE();

        // role manager is the submitter
        ILightClientMsgs.MsgVerifyMembership memory membershipMsg = newMembershipMsg(1);
        vm.prank(roleManager);
        ics07Tendermint.verifyMembership(membershipMsg);

        // submitter is not the role manager
        vm.prank(proofSubmitter);
        ics07Tendermint.verifyMembership(membershipMsg);

        // role manager allows anyone to verify membership
        vm.prank(roleManager);
        ics07Tendermint.grantRole(proofSubmitterRole, address(0));

        // anyone can verify membership
        address anyAddr = makeAddr("anyAddr");
        vm.prank(anyAddr);
        ics07Tendermint.verifyMembership(membershipMsg);
    }

    function test_failure_verifyMembership() public {
        // unauthorized account
        address unauthorized = makeAddr("unauthorized");
        ILightClientMsgs.MsgVerifyMembership memory membershipMsg = newMembershipMsg(1);
        vm.expectRevert(
            abi.encodeWithSelector(
                IAccessControl.AccessControlUnauthorizedAccount.selector,
                unauthorized,
                ics07Tendermint.PROOF_SUBMITTER_ROLE()
            )
        );
        vm.prank(unauthorized);
        ics07Tendermint.verifyMembership(membershipMsg);
    }

    function test_success_verifyNonMembership() public {
        bytes32 proofSubmitterRole = ics07Tendermint.PROOF_SUBMITTER_ROLE();

        // role manager is the submitter
        ILightClientMsgs.MsgVerifyNonMembership memory membershipMsg = newNonMembershipMsg(1);
        vm.prank(roleManager);
        ics07Tendermint.verifyNonMembership(membershipMsg);

        // submitter is not the role manager
        vm.prank(proofSubmitter);
        ics07Tendermint.verifyNonMembership(membershipMsg);

        // role manager allows anyone to verify membership
        vm.prank(roleManager);
        ics07Tendermint.grantRole(proofSubmitterRole, address(0));

        // anyone can verify membership
        address anyAddr = makeAddr("anyAddr");
        vm.prank(anyAddr);
        ics07Tendermint.verifyNonMembership(membershipMsg);
    }

    function test_failure_verifyNonMembership() public {
        // unauthorized account
        address unauthorized = makeAddr("unauthorized");
        ILightClientMsgs.MsgVerifyNonMembership memory membershipMsg = newNonMembershipMsg(1);
        vm.expectRevert(
            abi.encodeWithSelector(
                IAccessControl.AccessControlUnauthorizedAccount.selector,
                unauthorized,
                ics07Tendermint.PROOF_SUBMITTER_ROLE()
            )
        );
        vm.prank(unauthorized);
        ics07Tendermint.verifyNonMembership(membershipMsg);
    }

    function test_success_misbehaviour() public {
        bytes32 proofSubmitterRole = ics07Tendermint.PROOF_SUBMITTER_ROLE();

        // role manager is the submitter
        bytes memory misbehaviourMsg = newMisbehaviourMsg();
        vm.prank(roleManager);
        ics07Tendermint.misbehaviour(misbehaviourMsg);

        // restart the test since client is frozen
        setUp();

        // submitter is not the role manager
        vm.prank(proofSubmitter);
        ics07Tendermint.misbehaviour(misbehaviourMsg);

        // restart the test since client is frozen
        setUp();

        // role manager allows anyone to submit misbehaviour
        vm.prank(roleManager);
        ics07Tendermint.grantRole(proofSubmitterRole, address(0));

        // anyone can submit misbehaviour
        address anyAddr = makeAddr("anyAddr");
        vm.prank(anyAddr);
        ics07Tendermint.misbehaviour(misbehaviourMsg);
    }

    function test_failure_misbehaviour() public {
        // unauthorized account
        address unauthorized = makeAddr("unauthorized");
        bytes memory misbehaviourMsg = newMisbehaviourMsg();
        vm.expectRevert(
            abi.encodeWithSelector(
                IAccessControl.AccessControlUnauthorizedAccount.selector,
                unauthorized,
                ics07Tendermint.PROOF_SUBMITTER_ROLE()
            )
        );
        vm.prank(unauthorized);
        ics07Tendermint.misbehaviour(misbehaviourMsg);
    }
}
