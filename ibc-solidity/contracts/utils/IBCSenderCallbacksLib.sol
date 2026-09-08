// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { IIBCAppCallbacks } from "../msgs/IIBCAppCallbacks.sol";
import { IIBCSenderCallbacks } from "../interfaces/IIBCSenderCallbacks.sol";

import { ERC165Checker } from "@openzeppelin-contracts/utils/introspection/ERC165Checker.sol";

/// @title IBC Callbacks Library
/// @notice This library provides utility functions for IBC Apps to make callbacks to sender contracts.
library IBCSenderCallbacksLib {
    /// @notice Checks if the given address implements the IIBCSenderCallbacks interface.
    /// @param sender The address to check
    /// @return bool True if the address implements IIBCSenderCallbacks, false otherwise
    function _supportsCallbacks(address sender) private view returns (bool) {
        return ERC165Checker.supportsInterface(sender, type(IIBCSenderCallbacks).interfaceId);
    }

    /// @notice Make a callback to the sender contract when a packet is acknowledged if it supports IIBCSenderCallbacks.
    /// @param callbackAddress The address of the callback contract
    /// @param success Whether the packet was successfully received by the destination chain
    /// @param msg_ The callback message containing details about the acknowledgement
    function ackPacketCallback(
        address callbackAddress,
        bool success,
        IIBCAppCallbacks.OnAcknowledgementPacketCallback calldata msg_
    )
        internal
    {
        if (_supportsCallbacks(callbackAddress)) {
            IIBCSenderCallbacks(callbackAddress).onAckPacket(success, msg_);
        }
    }

    /// @notice Make a callback to the sender contract when a packet times out if it supports IIBCSenderCallbacks.
    /// @param callbackAddress The address of the callback contract
    /// @param msg_ The callback message containing details about the timeout
    function timeoutPacketCallback(
        address callbackAddress,
        IIBCAppCallbacks.OnTimeoutPacketCallback calldata msg_
    )
        internal
    {
        if (_supportsCallbacks(callbackAddress)) {
            IIBCSenderCallbacks(callbackAddress).onTimeoutPacket(msg_);
        }
    }
}
