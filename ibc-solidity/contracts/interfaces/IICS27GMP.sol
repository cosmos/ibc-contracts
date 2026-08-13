// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { IICS27GMPMsgs } from "../msgs/IICS27GMPMsgs.sol";

/// @title ICS27 General Message Passing (GMP) Interface
/// @notice Interface for the ICS27 General Message Passing (GMP) contract
interface IICS27GMP {
    /// @notice The address of the ICS26Router contract
    /// @return The address of the ICS26Router contract
    function ics26() external view returns (address);

    /// @notice Retrieve the Account beacon contract address
    /// @return The account beacon contract address
    function getAccountBeacon() external view returns (address);

    /// @notice Retrieve the Account (proxy) contract address
    /// @dev This is view instead of pure in case we change the proxy bytecode
    /// @param accountId The account identifier
    /// @return The (proxy) Account contract address
    function getOrComputeAccountAddress(IICS27GMPMsgs.AccountIdentifier calldata accountId)
        external
        view
        returns (address);

    /// @notice Retrieve the AccountIdentifier for a given Account (proxy) contract address
    /// @dev Reverts if the account does not exist
    /// @param account The (proxy) Account contract address
    /// @return The AccountIdentifier
    function getAccountIdentifier(address account) external view returns (IICS27GMPMsgs.AccountIdentifier memory);

    /// @notice Send a GMP packet by calling IICS26Router.sendPacket
    /// @param msg_ The message for sending a GMP packet
    /// @return sequence The sequence number of the packet created
    function sendCall(IICS27GMPMsgs.SendCallMsg calldata msg_) external returns (uint64 sequence);

    /// @notice Initializes the contract instead of a constructor
    /// @dev Meant to be called only once from the proxy
    /// @param ics26_ The ICS26Router contract address
    /// @param accountLogic The address of the ICS27Account logic contract
    /// @param authority The address of the AccessManager contract
    function initialize(address ics26_, address accountLogic, address authority) external;

    /// @notice Upgrades the implementation of the account beacon contract
    /// @dev The caller must be the ICS26Router admin
    /// @param newAccountLogic The address of the new account logic contract
    function upgradeAccountTo(address newAccountLogic) external;
}
