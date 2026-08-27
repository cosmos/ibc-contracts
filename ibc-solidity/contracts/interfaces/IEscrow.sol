// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import {IERC20} from "@openzeppelin-contracts/token/ERC20/IERC20.sol";

/// @title Escrow Contract Interface
/// @notice The interface for locking tokens in escrow in ICS20
interface IEscrow {
    /// @notice Send tokens to the specified address
    /// @dev This function can only be called by the ICS20 contract
    /// @param token The token to send
    /// @param to The address to send the tokens to
    /// @param amount The amount of tokens to send
    function send(IERC20 token, address to, uint256 amount) external;

    /// @notice Received tokens from the specified address
    /// @dev This function can only be called by the ICS20 contract
    /// @param token The token received
    /// @param from The address that sent the tokens
    /// @param amount The amount of tokens received
    function recvCallback(address token, address from, uint256 amount) external;

    /// @notice Get the ICS20 contract address
    /// @return The ICS20 contract address
    function ics20() external view returns (address);

    /// @notice Initializes the IBCERC20 contract
    /// @dev This initializes the contract to the latest version from an empty state
    /// @param ics20_ The ICS20 contract address, can send funds from the escrow
    /// @param authority The address of the AccessManager contract
    function initialize(address ics20_, address authority) external;
}
