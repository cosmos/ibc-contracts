// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

/// @title IBC Multicall Messages
/// @notice Interface defining the IBC multicall data structures
interface IIBCMulticallMsgs {
    /// @notice A call to execute as part of a multicall
    /// @param target The address of the contract to call
    /// @param allowFailure Whether the multicall continues if this call reverts
    /// @param callData The calldata to call the target with
    struct Call {
        address target;
        bool allowFailure;
        bytes callData;
    }

    /// @notice The outcome of a call executed as part of a multicall
    /// @param success Whether the call succeeded
    /// @param returnData The return data of the call, or its revert data if it failed
    struct Result {
        bool success;
        bytes returnData;
    }
}
