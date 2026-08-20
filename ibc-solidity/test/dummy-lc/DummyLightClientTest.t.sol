// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import { Test } from "forge-std/Test.sol";

import { DummyLightClient } from "../../contracts/light-clients/dummy/DummyLightClient.sol";
import { ILightClientMsgs } from "../../contracts/msgs/ILightClientMsgs.sol";
import { IICS02ClientMsgs } from "../../contracts/msgs/IICS02ClientMsgs.sol";

contract DummyLightClientTest is Test {
    DummyLightClient internal client;

    IICS02ClientMsgs.Height internal height = IICS02ClientMsgs.Height({ revisionNumber: 1, revisionHeight: 100 });
    uint64 internal constant TIMESTAMP = 1_700_000_000;
    bytes[] internal path;
    bytes internal value = bytes("value");

    function setUp() public {
        client = new DummyLightClient();
        path.push(bytes("ibc"));
        path.push(bytes("commitments"));
    }

    function test_updateClient_stores_state() public {
        client.updateClient(_updateMsg(height, TIMESTAMP, path, value));

        DummyLightClient.ClientState memory clientState =
            abi.decode(client.getClientState(), (DummyLightClient.ClientState));
        assertEq(clientState.latestHeight.revisionNumber, height.revisionNumber);
        assertEq(clientState.latestHeight.revisionHeight, height.revisionHeight);
        assertEq(clientState.latestTimestamp, TIMESTAMP);
    }

    function test_verifyMembership_succeeds() public {
        client.updateClient(_updateMsg(height, TIMESTAMP, path, value));

        uint256 timestamp = client.verifyMembership(_membershipMsg(height, path, value));

        assertEq(timestamp, TIMESTAMP);
    }

    function test_verifyMembership_succeeds_for_empty_value() public {
        bytes memory emptyValue = bytes("");
        client.updateClient(_updateMsg(height, TIMESTAMP, path, emptyValue));

        uint256 timestamp = client.verifyMembership(_membershipMsg(height, path, emptyValue));

        assertEq(timestamp, TIMESTAMP);
    }

    function test_verifyMembership_reverts_for_wrong_value() public {
        client.updateClient(_updateMsg(height, TIMESTAMP, path, value));

        vm.expectRevert(DummyLightClient.ValueMismatch.selector);
        client.verifyMembership(_membershipMsg(height, path, bytes("wrong")));
    }

    function test_verifyMembership_reverts_for_absent_path() public {
        client.updateClient(_updateMsg(height, TIMESTAMP, path, value));

        vm.expectRevert(DummyLightClient.MissingMembership.selector);
        client.verifyMembership(_membershipMsg(height, _otherPath(), value));
    }

    function test_verifyMembership_reverts_for_unknown_height() public {
        vm.expectRevert(DummyLightClient.UnknownHeight.selector);
        client.verifyMembership(_membershipMsg(height, path, value));
    }

    function test_verifyNonMembership_succeeds() public {
        client.updateClient(_updateMsg(height, TIMESTAMP, path, value));

        uint256 timestamp = client.verifyNonMembership(_nonMembershipMsg(height, _otherPath()));

        assertEq(timestamp, TIMESTAMP);
    }

    function test_verifyNonMembership_reverts_for_present_path() public {
        client.updateClient(_updateMsg(height, TIMESTAMP, path, value));

        vm.expectRevert(DummyLightClient.MembershipExists.selector);
        client.verifyNonMembership(_nonMembershipMsg(height, path));
    }

    function test_updateClient_overwrites_same_height_path() public {
        bytes memory newValue = bytes("new value");
        client.updateClient(_updateMsg(height, TIMESTAMP, path, value));
        client.updateClient(_updateMsg(height, TIMESTAMP + 1, path, newValue));

        vm.expectRevert(DummyLightClient.ValueMismatch.selector);
        client.verifyMembership(_membershipMsg(height, path, value));

        assertEq(client.verifyMembership(_membershipMsg(height, path, newValue)), TIMESTAMP + 1);
    }

    function _updateMsg(
        IICS02ClientMsgs.Height memory height_,
        uint64 timestamp,
        bytes[] memory path_,
        bytes memory value_
    )
        internal
        pure
        returns (bytes memory)
    {
        DummyLightClient.Membership[] memory memberships = new DummyLightClient.Membership[](1);
        memberships[0] = DummyLightClient.Membership({ path: path_, value: value_ });
        return abi.encode(
            DummyLightClient.MsgUpdateClient({ height: height_, timestamp: timestamp, memberships: memberships })
        );
    }

    function _membershipMsg(
        IICS02ClientMsgs.Height memory height_,
        bytes[] memory path_,
        bytes memory value_
    )
        internal
        pure
        returns (ILightClientMsgs.MsgVerifyMembership memory)
    {
        return ILightClientMsgs.MsgVerifyMembership({
            proof: bytes("ignored"), proofHeight: height_, path: path_, value: value_
        });
    }

    function _nonMembershipMsg(
        IICS02ClientMsgs.Height memory height_,
        bytes[] memory path_
    )
        internal
        pure
        returns (ILightClientMsgs.MsgVerifyNonMembership memory)
    {
        return ILightClientMsgs.MsgVerifyNonMembership({ proof: bytes("ignored"), proofHeight: height_, path: path_ });
    }

    function _otherPath() internal pure returns (bytes[] memory) {
        bytes[] memory other = new bytes[](2);
        other[0] = bytes("ibc");
        other[1] = bytes("other");
        return other;
    }
}
