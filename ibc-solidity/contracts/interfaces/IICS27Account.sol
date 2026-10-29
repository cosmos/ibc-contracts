// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { IICS27AccountMsgs } from "../msgs/IICS27AccountMsgs.sol";

/// @title IICS27Account
/// @notice Interface for ICS27 Accounts to implement
interface IICS27Account {
    /// @notice Get the ICS27 contract address
    /// @return The ICS27 contract address
    function ics27() external view returns (address);

    /// @notice Like openzeppelin's `Address.sendValue`, but reverts like `functionCall` when the transfer fails.
    /// @dev This function can only be called by self.
    /// @param recipient The address to send the value to
    /// @param amount The amount of wei to send
    function sendValue(address payable recipient, uint256 amount) external;

    /// @notice Performs a Solidity function call using a low level `call`.
    /// @dev Like openzeppelin's `Address.functionCall`, but reverts without data when the call fails without revert
    /// data and most likely ran out of gas, so the router fails the relay instead of writing an error acknowledgement.
    /// @dev This function can only be called by ICS27.
    /// @param target The target address to call
    /// @param data The data to send to the target address
    /// @return result The result of the call
    function functionCall(address target, bytes calldata data) external returns (bytes memory result);

    /// @notice Performs a Solidity function call using a low level `call`.
    /// @dev Like openzeppelin's `Address.functionCallWithValue`, but reverts like `functionCall`.
    /// @dev This function can only be called by self.
    /// @param target The target address to call
    /// @param data The data to send to the target address
    /// @param value The value to send to the target address
    /// @return result The result of the call
    function execute(address target, bytes calldata data, uint256 value) external returns (bytes memory result);

    /// @notice Performs multiple Solidity function calls using a low level `call`.
    /// @dev Like openzeppelin's `Address.functionCallWithValue`, but reverts like `functionCall`.
    /// @dev This function can only be called by self.
    /// @param calls The array of `Call` structs to execute
    /// @return results The array of results from the calls
    function executeBatch(IICS27AccountMsgs.Call[] calldata calls) external returns (bytes[] memory results);

    /// @notice Performs a Solidity function call using a low level `call` with exactly `gas` gas.
    /// @dev Like `execute`, but runs out of gas if the relayer did not provide enough gas for `gas`.
    /// @dev This function can only be called by self.
    /// @param target The target address to call
    /// @param data The data to send to the target address
    /// @param value The value to send to the target address
    /// @param gas The gas to call the target address with
    /// @return result The result of the call
    function executeWithGas(
        address target,
        bytes calldata data,
        uint256 value,
        uint256 gas
    )
        external
        returns (bytes memory result);

    /// @notice Performs multiple Solidity function calls using a low level `call`, each with exactly its `gas`.
    /// @dev Like `executeBatch`, but runs out of gas if the relayer did not provide enough gas for a call.
    /// @dev This function can only be called by self.
    /// @param calls The array of `CallWithGas` structs to execute
    /// @return results The array of results from the calls
    function executeBatchWithGas(IICS27AccountMsgs.CallWithGas[] calldata calls)
        external
        returns (bytes[] memory results);

    /// @notice Initializes the ICS27Account contract
    /// @dev This function is meant to be called by a proxy
    /// @param ics27_ The ICS27GMP contract address
    function initialize(address ics27_) external;
}
