// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable no-empty-blocks,gas-custom-errors

import { Vm } from "forge-std/Vm.sol";
import { Test } from "forge-std/Test.sol";

import { IICS26RouterMsgs } from "../../../contracts/msgs/IICS26RouterMsgs.sol";
import { IICS20TransferMsgs } from "../../../contracts/msgs/IICS20TransferMsgs.sol";

import { ICS20Lib } from "../../../contracts/utils/ICS20Lib.sol";
import { ICS24Host } from "../../../contracts/utils/ICS24Host.sol";

contract TestHelper is Test {
    /// @notice The first client ID used for testing
    string public constant FIRST_CLIENT_ID = "client-0";
    /// @notice The second client ID used for testing
    string public constant SECOND_CLIENT_ID = "client-1";
    /// @notice Invalid ID used for testing
    string public constant INVALID_ID = "invalid";
    /// @notice The default starting balance for the ERC20 token
    uint256 public constant DEFAULT_ERC20_STARTING_BALANCE = type(uint256).max;

    /// @notice The default merkle prefix used in cosmos chains
    bytes[] private _cosmosMerklePrefix = [bytes("ibc"), bytes("")];
    /// @notice Empty merkle prefix used for testing
    bytes[] private _emptyMerklePrefix = [bytes("")];

    bytes[] private _singleSuccessAck = [ICS20Lib.SUCCESSFUL_ACKNOWLEDGEMENT_JSON];
    bytes[] private _singleErrorAck = [ICS24Host.UNIVERSAL_ERROR_ACK];

    /// @notice The default merkle prefix used in cosmos chains
    function COSMOS_MERKLE_PREFIX() external view returns (bytes[] memory) {
        return _cosmosMerklePrefix;
    }

    /// @notice Empty merkle prefix used for testing
    function EMPTY_MERKLE_PREFIX() external view returns (bytes[] memory) {
        return _emptyMerklePrefix;
    }

    function SINGLE_SUCCESS_ACK() external view returns (bytes[] memory) {
        return _singleSuccessAck;
    }

    function SINGLE_ERROR_ACK() external view returns (bytes[] memory) {
        return _singleErrorAck;
    }

    /// @notice The default timeout timestamp for testing
    function DEFAULT_TIMEOUT_TIMESTAMP() external view returns (uint64) {
        return uint64(block.timestamp + 1000);
    }

    /// @dev retuns a random base64 string
    function randomString() public view returns (string memory) {
        uint256 randomNum = vm.randomUint();
        return vm.toBase64(abi.encodePacked(randomNum));
    }

    /// @notice Get FungibleTokenPacketData from the packet
    function getFTPD(IICS26RouterMsgs.Packet memory packet)
        public
        pure
        returns (IICS20TransferMsgs.FungibleTokenPacketData memory)
    {
        require(packet.payloads.length == 1, "Packet must have 1 payload");
        return abi.decode(packet.payloads[0].value, (IICS20TransferMsgs.FungibleTokenPacketData));
    }

    /// @dev Searches all the logs for the given event selector and returns the first value found
    function getValueFromEvent(bytes32 eventSelector) public view returns (bytes memory) {
        Vm.Log[] memory events = vm.getRecordedLogs();
        for (uint256 i = 0; i < events.length; ++i) {
            Vm.Log memory log = events[i];
            for (uint256 j = 0; j < log.topics.length; ++j) {
                if (log.topics[j] == eventSelector) {
                    return log.data;
                }
            }
        }
        // solhint-disable-next-line gas-custom-errors
        revert("Event not found");
    }
}
