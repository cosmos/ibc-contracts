// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable no-empty-blocks

import {ILightClientMsgs} from "../../../contracts/msgs/ILightClientMsgs.sol";
import {ILightClient} from "../../../contracts/interfaces/ILightClient.sol";

contract DummyLightClient is ILightClient, ILightClientMsgs {
    UpdateResult public updateResult;
    uint64 public membershipResult;
    bool public membershipShouldFail;
    bytes public latestUpdateMsg;

    error MembershipShouldFail();

    constructor(UpdateResult updateResult_, uint64 membershipResult_, bool membershipShouldFail_) {
        updateResult = updateResult_;
        membershipResult = membershipResult_;
        membershipShouldFail = membershipShouldFail_;
    }

    function updateClient(bytes calldata updateMsg) external returns (UpdateResult) {
        latestUpdateMsg = updateMsg;
        return updateResult;
    }

    function verifyMembership(MsgVerifyMembership calldata) external view returns (uint256) {
        if (membershipShouldFail) {
            revert MembershipShouldFail();
        }
        return membershipResult;
    }

    function verifyNonMembership(MsgVerifyNonMembership calldata) external view returns (uint256) {
        if (membershipShouldFail) {
            revert MembershipShouldFail();
        }
        return membershipResult;
    }

    function misbehaviour(bytes calldata misbehaviourMsg) external {}

    // custom functions to return values we want
    function setUpdateResult(UpdateResult updateResult_) external {
        updateResult = updateResult_;
    }

    function setMembershipResult(uint64 membershipResult_, bool shouldFail) external {
        membershipResult = membershipResult_;
        membershipShouldFail = shouldFail;
    }

    function getClientState() external pure returns (bytes memory) {
        return bytes("");
    }
}
