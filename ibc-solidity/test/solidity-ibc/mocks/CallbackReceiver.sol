// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable no-empty-blocks

import {IIBCSenderCallbacks} from "../../../contracts/interfaces/IIBCSenderCallbacks.sol";
import {IIBCAppCallbacks} from "../../../contracts/msgs/IIBCAppCallbacks.sol";
import {IBCCallbackReceiver} from "../../../contracts/utils/IBCCallbackReceiver.sol";

/// @title CallbackReceiver
/// @notice A contract that implements the IIBCSenderCallbacks interface to receive callbacks from IBC applications.
contract CallbackReceiver is IBCCallbackReceiver {
    /// @inheritdoc IIBCSenderCallbacks
    function onAckPacket(bool success, IIBCAppCallbacks.OnAcknowledgementPacketCallback calldata msg_)
        external
        override
    {
        // Handle the acknowledgement logic here
        // For example, emit an event or update state
    }

    /// @inheritdoc IIBCSenderCallbacks
    function onTimeoutPacket(IIBCAppCallbacks.OnTimeoutPacketCallback calldata msg_) external override {
        // Handle the timeout logic here
        // For example, emit an event or update state
    }
}
