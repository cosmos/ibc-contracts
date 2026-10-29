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

    /// @notice Call struct for the `executeBatchWithGas` function.
    /// @param target The target address to call
    /// @param data The data to send to the target address
    /// @param value The value to send to the target address
    /// @param gas The gas to call the target address with, which the relayer must provide, plus the 2300 gas stipend
    /// if `value` is not zero
    struct CallWithGas {
        address target;
        bytes data;
        uint256 value;
        uint256 gas;
    }
}
