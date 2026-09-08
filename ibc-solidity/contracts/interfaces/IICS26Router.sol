// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { IICS26RouterMsgs } from "../msgs/IICS26RouterMsgs.sol";
import { IIBCApp } from "./IIBCApp.sol";

/// @title ICS26 Router Restricted Interface
/// @notice Interface for the access controlled functions of the IBC Eureka Core Router
interface IICS26RouterAccessControlled {
    /// @notice Adds an IBC application to the router
    /// @param portId The custom port identifier.
    /// @param app The address of the IBC application contract
    function addIBCApp(string calldata portId, address app) external;

    /// @notice Receives a packet
    /// @param msg The message for receiving packets
    function recvPacket(IICS26RouterMsgs.MsgRecvPacket calldata msg) external;

    /// @notice Acknowledges a packet
    /// @param msg The message for acknowledging packets
    function ackPacket(IICS26RouterMsgs.MsgAckPacket calldata msg) external;

    /// @notice Timeouts a packet
    /// @param msg The message for timing out packets
    function timeoutPacket(IICS26RouterMsgs.MsgTimeoutPacket calldata msg) external;
}

/// @title ICS26 Router Interface
/// @notice Interface for the IBC Eureka Core Router
interface IICS26Router is IICS26RouterAccessControlled {
    /// @notice Returns the address of the IBC application given the port identifier
    /// @param portId The port identifier
    /// @return The address of the IBC application contract
    function getIBCApp(string calldata portId) external view returns (IIBCApp);

    /// @notice Adds an IBC application to the router
    /// @dev The port identifier is the address of the IBC application contract.
    /// @param app The address of the IBC application contract
    function addIBCApp(address app) external;

    /// @notice Sends a packet
    /// @param msg The message for sending packets
    /// @return The sequence number of the packet
    function sendPacket(IICS26RouterMsgs.MsgSendPacket calldata msg) external returns (uint64);

    /// @notice Initializes the contract instead of a constructor
    /// @dev This initializes the contract to the latest version from an empty state
    /// @dev Meant to be called only once from the proxy
    /// @param authority The address of the AccessManager contract
    function initialize(address authority) external;

    // --------------------- Events --------------------- //

    /// @notice Emitted when an IBC application is added to the router
    /// @param portId The port identifier
    /// @param app The address of the IBC application contract
    // solhint-disable-next-line gas-indexed-events
    event IBCAppAdded(string portId, address app);
    /// @notice Emitted when an error occurs during the IBC application's recvPacket callback
    /// @param reason The error message
    event IBCAppRecvPacketCallbackError(bytes reason);
    /// @notice Emitted when a packet is sent
    /// @param clientId The source client identifier
    /// @param sequence The sequence number of the packet
    /// @param packet The sent packet
    event SendPacket(string indexed clientId, uint256 indexed sequence, IICS26RouterMsgs.Packet packet);
    /// @notice Emitted when a packet acknowledgement is written
    /// @param clientId The destination client identifier
    /// @param sequence The sequence number of the packet
    /// @param packet The packet that was acknowledged
    /// @param acknowledgements The list of acknowledgements data
    event WriteAcknowledgement(
        string indexed clientId, uint256 indexed sequence, IICS26RouterMsgs.Packet packet, bytes[] acknowledgements
    );
    /// @notice Emitted when a packet is timed out
    /// @param clientId The source client identifier
    /// @param sequence The sequence number of the packet
    /// @param packet The packet that was timed out
    event TimeoutPacket(string indexed clientId, uint256 indexed sequence, IICS26RouterMsgs.Packet packet);
    /// @notice Emitted when a packet is acknowledged
    /// @param clientId The source client identifier
    /// @param sequence The sequence number of the packet
    /// @param packet The packet that was acknowledged
    /// @param acknowledgement The acknowledgement data
    event AckPacket(
        string indexed clientId, uint256 indexed sequence, IICS26RouterMsgs.Packet packet, bytes acknowledgement
    );
    /// @notice Emitted when a redundant relay occurs
    event Noop();
}
