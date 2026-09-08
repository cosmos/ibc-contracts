// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { IIBCAppCallbacks } from "../msgs/IIBCAppCallbacks.sol";

/// @title IBC Application Interface
/// @notice IIBCApp is an interface for all IBC application contracts to implement.
/// @dev All functions in this interface must only be called by the ICS26Router contract.
interface IIBCApp {
    /// @notice Called when a packet is received from the counterparty chain.
    /// @param msg_ The callback message
    /// @return The acknowledgement data
    function onRecvPacket(IIBCAppCallbacks.OnRecvPacketCallback calldata msg_) external returns (bytes memory);

    /// @notice Called when a packet acknowledgement is received from the counterparty chain.
    /// @param msg_ The callback message
    function onAcknowledgementPacket(IIBCAppCallbacks.OnAcknowledgementPacketCallback calldata msg_) external;

    /// @notice Called when a packet is timed out.
    /// @param msg_ The callback message
    function onTimeoutPacket(IIBCAppCallbacks.OnTimeoutPacketCallback calldata msg_) external;
}
