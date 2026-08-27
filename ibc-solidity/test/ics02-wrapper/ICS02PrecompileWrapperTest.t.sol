// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable gas-custom-errors

import { Test } from "forge-std/Test.sol";

import { ILightClientMsgs } from "../../contracts/msgs/ILightClientMsgs.sol";
import { IICS02ClientMsgs } from "../../contracts/msgs/IICS02ClientMsgs.sol";

import { ICS02PrecompileWrapper } from "../../contracts/light-clients/ics02-wrapper/ICS02PrecompileWrapper.sol";
import {
    IICS02PrecompileWrapperErrors
} from "../../contracts/light-clients/ics02-wrapper/errors/IICS02PrecompileWrapperErrors.sol";
import { IICS02Precompile } from "../../contracts/light-clients/ics02-wrapper/interfaces/IICS02Precompile.sol";

contract ICS02PrecompileWrapperTest is Test {
    address public constant ICS02_ADDRESS = 0x0000000000000000000000000000000000000807;

    string public constant TEST_CLIENT_ID = "07-tendermint-0";
    ICS02PrecompileWrapper public ics02Wrapper;

    function setUp() public {
        ics02Wrapper = new ICS02PrecompileWrapper(TEST_CLIENT_ID);
    }

    function test_success_goClientId() public view {
        assertEq(ics02Wrapper.ibcGoClientId(), TEST_CLIENT_ID);
    }

    function test_success_getClientState() public {
        bytes memory mockClientState = "clientState";
        vm.mockCall(
            ICS02_ADDRESS,
            abi.encodeCall(IICS02Precompile.getClientState, (TEST_CLIENT_ID)),
            abi.encode(mockClientState)
        );

        bytes memory clientState = ics02Wrapper.getClientState();
        assertEq(clientState, mockClientState);
    }

    function test_failure_getClientState() public {
        bytes memory revertData = "revertData";
        vm.mockCallRevert(ICS02_ADDRESS, abi.encodeCall(IICS02Precompile.getClientState, (TEST_CLIENT_ID)), revertData);

        vm.expectRevert(revertData);
        ics02Wrapper.getClientState();
    }

    function test_success_updateClient_update() public {
        bytes memory updateMsg = "updateMsg";
        vm.mockCall(
            ICS02_ADDRESS,
            abi.encodeCall(IICS02Precompile.updateClient, (TEST_CLIENT_ID, updateMsg)),
            abi.encode(IICS02Precompile.UpdateResult.Update)
        );

        ILightClientMsgs.UpdateResult result = ics02Wrapper.updateClient(updateMsg);
        assertTrue(result == ILightClientMsgs.UpdateResult.Update);
    }

    function test_success_updateClient_misbehaviour() public {
        bytes memory updateMsg = "updateMsg";
        vm.mockCall(
            ICS02_ADDRESS,
            abi.encodeCall(IICS02Precompile.updateClient, (TEST_CLIENT_ID, updateMsg)),
            abi.encode(IICS02Precompile.UpdateResult.Misbehaviour)
        );

        ILightClientMsgs.UpdateResult result = ics02Wrapper.updateClient(updateMsg);
        assertTrue(result == ILightClientMsgs.UpdateResult.Misbehaviour);
    }

    function test_failure_updateClient() public {
        // Case 1: Call reverts
        bytes memory updateMsg = "updateMsg";
        bytes memory revertData = "revertData";
        vm.mockCallRevert(
            ICS02_ADDRESS, abi.encodeCall(IICS02Precompile.updateClient, (TEST_CLIENT_ID, updateMsg)), revertData
        );

        vm.expectRevert(revertData);
        ics02Wrapper.updateClient(updateMsg);

        // Case 2: Unreachable branch
        vm.mockCall(
            ICS02_ADDRESS,
            abi.encodeCall(IICS02Precompile.updateClient, (TEST_CLIENT_ID, updateMsg)),
            abi.encode(uint8(2))
        );

        vm.expectRevert();
        ics02Wrapper.updateClient(updateMsg);
    }

    function testFuzz_success_verifyMembership(uint64 timestamp) public {
        ILightClientMsgs.MsgVerifyMembership memory msg_ = ILightClientMsgs.MsgVerifyMembership({
            proof: "proof",
            proofHeight: IICS02ClientMsgs.Height({ revisionNumber: 1, revisionHeight: 10 }),
            path: new bytes[](0),
            value: "value"
        });

        vm.mockCall(
            ICS02_ADDRESS,
            abi.encodeCall(
                IICS02Precompile.verifyMembership, (TEST_CLIENT_ID, msg_.proof, msg_.proofHeight, msg_.path, msg_.value)
            ),
            abi.encode(timestamp)
        );

        uint256 result = ics02Wrapper.verifyMembership(msg_);
        assertEq(result, timestamp);
    }

    function test_failure_verifyMembership() public {
        ILightClientMsgs.MsgVerifyMembership memory msg_ = ILightClientMsgs.MsgVerifyMembership({
            proof: "proof",
            proofHeight: IICS02ClientMsgs.Height({ revisionNumber: 1, revisionHeight: 10 }),
            path: new bytes[](0),
            value: "value"
        });

        bytes memory revertData = "revertData";

        vm.mockCallRevert(
            ICS02_ADDRESS,
            abi.encodeCall(
                IICS02Precompile.verifyMembership, (TEST_CLIENT_ID, msg_.proof, msg_.proofHeight, msg_.path, msg_.value)
            ),
            revertData
        );

        vm.expectRevert(revertData);
        ics02Wrapper.verifyMembership(msg_);
    }

    function testFuzz_success_verifyNonMembership(uint64 timestamp) public {
        ILightClientMsgs.MsgVerifyNonMembership memory msg_ = ILightClientMsgs.MsgVerifyNonMembership({
            proof: "proof",
            proofHeight: IICS02ClientMsgs.Height({ revisionNumber: 1, revisionHeight: 10 }),
            path: new bytes[](0)
        });

        vm.mockCall(
            ICS02_ADDRESS,
            abi.encodeCall(
                IICS02Precompile.verifyNonMembership, (TEST_CLIENT_ID, msg_.proof, msg_.proofHeight, msg_.path)
            ),
            abi.encode(timestamp)
        );

        uint256 result = ics02Wrapper.verifyNonMembership(msg_);
        assertEq(result, timestamp);
    }

    function test_failure_verifyNonMembership() public {
        ILightClientMsgs.MsgVerifyNonMembership memory msg_ = ILightClientMsgs.MsgVerifyNonMembership({
            proof: "proof",
            proofHeight: IICS02ClientMsgs.Height({ revisionNumber: 1, revisionHeight: 10 }),
            path: new bytes[](0)
        });

        bytes memory revertData = "revertData";

        vm.mockCallRevert(
            ICS02_ADDRESS,
            abi.encodeCall(
                IICS02Precompile.verifyNonMembership, (TEST_CLIENT_ID, msg_.proof, msg_.proofHeight, msg_.path)
            ),
            revertData
        );

        vm.expectRevert(revertData);
        ics02Wrapper.verifyNonMembership(msg_);
    }

    function test_success_misbehaviour() public {
        bytes memory misbehaviourMsg = "misbehaviourMsg";
        vm.mockCall(
            ICS02_ADDRESS,
            abi.encodeCall(IICS02Precompile.updateClient, (TEST_CLIENT_ID, misbehaviourMsg)),
            abi.encode(IICS02Precompile.UpdateResult.Misbehaviour)
        );

        ics02Wrapper.misbehaviour(misbehaviourMsg);
    }

    function test_failure_misbehaviour() public {
        // Case 1: Successful update (no misbehaviour)
        bytes memory misbehaviourMsg = "misbehaviourMsg";
        vm.mockCall(
            ICS02_ADDRESS,
            abi.encodeCall(IICS02Precompile.updateClient, (TEST_CLIENT_ID, misbehaviourMsg)),
            abi.encode(IICS02Precompile.UpdateResult.Update)
        );

        vm.expectRevert(abi.encodeWithSelector(IICS02PrecompileWrapperErrors.NoMisbehaviourDetected.selector));
        ics02Wrapper.misbehaviour(misbehaviourMsg);

        // Case 2: Call reverts
        bytes memory revertData = "revertData";
        vm.mockCallRevert(
            ICS02_ADDRESS, abi.encodeCall(IICS02Precompile.updateClient, (TEST_CLIENT_ID, misbehaviourMsg)), revertData
        );

        vm.expectRevert(revertData);
        ics02Wrapper.misbehaviour(misbehaviourMsg);
    }
}
