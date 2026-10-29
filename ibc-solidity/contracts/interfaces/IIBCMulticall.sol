// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { IIBCMulticallMsgs } from "../msgs/IIBCMulticallMsgs.sol";

/// @title IBC Multicall Interface
/// @notice Batches calls, such as packet relays, where each call can be allowed to fail without reverting the batch
interface IIBCMulticall {
    /// @notice Emitted when a call that is allowed to fail reverts
    /// @param index The index of the call in the batch
    /// @param returnData The revert data of the call
    event CallFailed(uint256 indexed index, bytes returnData);

    /// @notice Initializes the contract instead of a constructor
    /// @dev Meant to be called only once from the proxy
    /// @param authority The address of the AccessManager contract
    function initialize(address authority) external;

    /// @notice Executes the calls in order
    /// @dev Reverts with the revert data of the first failing call that is not allowed to fail. A call to an address
    /// without code fails with `Address.AddressEmptyCode`. A call that is allowed to fail but most likely ran out of
    /// gas reverts the batch with `IIBCMulticallErrors.IBCMulticallCallOutOfGas`, so a batch never lands with calls
    /// dropped
    /// for lack of gas. The effects of a
    /// failing call that is allowed to fail are rolled back while the rest of the batch is kept.
    /// @param calls The calls to execute
    /// @return results The outcome of each call
    function multicall(IIBCMulticallMsgs.Call[] calldata calls)
        external
        returns (IIBCMulticallMsgs.Result[] memory results);
}
