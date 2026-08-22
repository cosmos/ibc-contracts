// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { IICS02ClientMsgs } from "../../../msgs/IICS02ClientMsgs.sol";

/// @title Dummy Light Client Messages
/// @notice Defines message and state types for the dummy light client.
interface IDummyLightClientMsgs {
    /// @notice Dummy client state returned by `getClientState`.
    /// @param latestHeight Latest updated height.
    /// @param latestTimestamp Timestamp for the latest updated height.
    struct ClientState {
        IICS02ClientMsgs.Height latestHeight;
        uint64 latestTimestamp;
    }

    /// @notice Membership record supplied in an update.
    /// @param path Merkle path for the record.
    /// @param value Value for the record.
    struct Membership {
        bytes[] path;
        bytes value;
    }

    /// @notice Update message accepted by the dummy client.
    /// @param height Height whose timestamp and membership records are being set.
    /// @param timestamp Consensus timestamp for the height.
    /// @param memberships Membership records for the height.
    struct MsgUpdateClient {
        IICS02ClientMsgs.Height height;
        uint64 timestamp;
        Membership[] memberships;
    }
}
