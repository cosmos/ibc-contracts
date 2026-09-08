// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { IICS26RouterMsgs } from "../msgs/IICS26RouterMsgs.sol";

/// @title IRelayerHelper
/// @notice Interface for the RelayerHelper contract
interface IRelayerHelper {
    /// @notice Returns the underlying ICS26Router contract address
    /// @return ICS26Router contract address
    function ICS26_ROUTER() external view returns (address);

    /// @notice Returns whether or not a packet was received
    /// @param packet The packet to check
    /// @return True if the packet was received, false otherwise
    function isPacketReceived(IICS26RouterMsgs.Packet calldata packet) external view returns (bool);

    /// @notice Returns whether or not a packet was received successfully
    /// @param packet The packet to check
    /// @return True if the packet was received and the application callback was successful, false otherwise
    function isPacketReceiveSuccessful(IICS26RouterMsgs.Packet calldata packet) external view returns (bool);

    /// @notice Returns the packet receipt for a given packet.
    /// @param clientId The packet destination client identifier.
    /// @param sequence The packet sequence number.
    /// @return The packet receipt for the given packet.
    function queryPacketReceipt(string calldata clientId, uint64 sequence) external view returns (bytes32);

    /// @notice Returns the packet commitment for a given packet.
    /// @param clientId The packet source client identifier.
    /// @param sequence The packet sequence number.
    /// @return The packet commitment for the given packet.
    function queryPacketCommitment(string calldata clientId, uint64 sequence) external view returns (bytes32);

    /// @notice Returns the packet acknowledgement commitment for a given packet.
    /// @param clientId The packet destination client identifier.
    /// @param sequence The packet sequence number.
    /// @return The packet acknowledgement commitment for the given packet.
    function queryAckCommitment(string calldata clientId, uint64 sequence) external view returns (bytes32);
}
