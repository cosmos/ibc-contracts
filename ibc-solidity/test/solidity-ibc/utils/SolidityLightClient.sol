// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable no-empty-blocks,gas-custom-errors

import {ILightClientMsgs} from "../../../contracts/msgs/ILightClientMsgs.sol";
import {ILightClient} from "../../../contracts/interfaces/ILightClient.sol";

import {ICS26Router} from "../../../contracts/ICS26Router.sol";

contract SolidityLightClient is ILightClient {
    ICS26Router private immutable _COUNTERPARTY_ICS26;

    constructor(ICS26Router counterpartyIcs26) {
        _COUNTERPARTY_ICS26 = counterpartyIcs26;
    }

    function updateClient(bytes calldata) external pure returns (ILightClientMsgs.UpdateResult) {
        revert("not implemented");
    }

    function verifyMembership(ILightClientMsgs.MsgVerifyMembership calldata msg_) external view returns (uint256) {
        require(msg_.path.length == 1, "only support single path");
        bytes32 solidityPath = keccak256(msg_.path[0]);
        bytes32 commitment = _COUNTERPARTY_ICS26.getCommitment(solidityPath);
        require(commitment != bytes32(0), "invalid path");
        require(keccak256(abi.encodePacked(commitment)) == keccak256(msg_.value), "invalid commitment");
        return block.timestamp;
    }

    function verifyNonMembership(ILightClientMsgs.MsgVerifyNonMembership calldata msg_)
        external
        view
        returns (uint256)
    {
        require(msg_.path.length == 1, "only support single path");
        bytes32 solidityPath = keccak256(msg_.path[0]);
        bytes32 commitment = _COUNTERPARTY_ICS26.getCommitment(solidityPath);
        require(commitment == bytes32(0), "invalid path");
        return block.timestamp;
    }

    function misbehaviour(bytes calldata) external pure {
        revert("not implemented");
    }

    function getClientState() external view returns (bytes memory) {
        return abi.encodePacked(address(_COUNTERPARTY_ICS26));
    }
}
