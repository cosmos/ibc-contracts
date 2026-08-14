// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable gas-struct-packing

import { IICS26RouterMsgs } from "./IICS26RouterMsgs.sol";

/// @title IBC Application Callbacks
/// @notice Interface defining IBC application callbacks
interface IIBCAppCallbacks {
    /// @notice Callback message for receiving a packet.
    /// @param sourceClient The source client identifier
    /// @param destinationClient The destination client identifier
    /// @param sequence The sequence number of the packet
    /// @param payload The packet payload
    /// @param relayer The relayer of this message
    struct OnRecvPacketCallback {
        string sourceClient;
        string destinationClient;
        uint64 sequence;
        IICS26RouterMsgs.Payload payload;
        address relayer;
    }

    /// @notice Callback message for acknowledging a packet.
    /// @param sourceClient The source client identifier
    /// @param destinationClient The destination client identifier
    /// @param sequence The sequence number of the packet
    /// @param payload The packet payload
    /// @param acknowledgement The acknowledgement
    /// @param relayer The relayer of this message
    struct OnAcknowledgementPacketCallback {
        string sourceClient;
        string destinationClient;
        uint64 sequence;
        IICS26RouterMsgs.Payload payload;
        bytes acknowledgement;
        address relayer;
    }

    /// @notice Called when a packet is to be timed out by this IBC application.
    /// @param sourceClient The source client identifier
    /// @param destinationClient The destination client identifier
    /// @param sequence The sequence number of the packet
    /// @param payload The packet payload
    /// @param relayer The relayer of this message
    struct OnTimeoutPacketCallback {
        string sourceClient;
        string destinationClient;
        uint64 sequence;
        IICS26RouterMsgs.Payload payload;
        address relayer;
    }
}
