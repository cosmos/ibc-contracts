// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

/// @title ICS27 Account Messages
/// @notice Interface defining ICS27 Account Messages
interface IICS27AccountMsgs {
    /// @notice Call struct for the `executeBatch` function.
    /// @param target The target address to call
    /// @param data The data to send to the target address
    /// @param value The value to send to the target address
    struct Call {
        address target;
        bytes data;
        uint256 value;
    }
}
