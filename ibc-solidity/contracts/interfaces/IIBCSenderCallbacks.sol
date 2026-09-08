// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { IIBCAppCallbacks } from "../msgs/IIBCAppCallbacks.sol";

/**
 * @title IBC Sender Callbacks Interface
 * @notice If a contract which implements this interface sends a packet using an IBC application,
 * then it will receive callbacks for acknowledgement and timeout events.
 * @dev If this interface is implemented, then IERC165 should also be implemented
 */
interface IIBCSenderCallbacks {
    /// @notice Called when a packet acknowledgement is received by the IBC application.
    /// @param success Whether the packet was successfully received by the destination chain
    /// @param msg_ The callback message
    function onAckPacket(bool success, IIBCAppCallbacks.OnAcknowledgementPacketCallback calldata msg_) external;

    /// @notice Called when a packet is timed out by the IBC application.
    /// @param msg_ The callback message
    function onTimeoutPacket(IIBCAppCallbacks.OnTimeoutPacketCallback calldata msg_) external;
}
