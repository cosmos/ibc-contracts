// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable gas-indexed-events

import { IIFTMsgs } from "../msgs/IIFTMsgs.sol";
import { IICS27GMP } from "./IICS27GMP.sol";

/// @title IFT Interface
/// @notice Interface for Interchain Fungible Token contracts
interface IIFT {
    /// @notice Emitted when a new IBC bridge is registered
    /// @param clientId The IBC client identifier on the local chain
    /// @param counterpartyIFTAddress The address of the IFT contract on the counterparty chain
    /// @param iftSendCallConstructor The address of the call constructor for the counterparty chain
    event IFTBridgeRegistered(string clientId, string counterpartyIFTAddress, address iftSendCallConstructor);

    /// @notice Emitted when an existing IBC bridge is removed
    /// @param clientId The IBC client identifier that was removed
    event IFTBridgeRemoved(string clientId);

    /// @notice Emitted when a bridge transfer is initiated
    /// @param clientId The IBC client identifier over which the transfer is being made
    /// @param sequence The IBC sequence number of the packet
    /// @param sender The address of the sender who initiated the transfer
    /// @param receiver The address of the receiver on the counterparty chain
    /// @param amount The amount of tokens transferred
    event IFTTransferInitiated(
        string clientId, uint64 sequence, address indexed sender, string receiver, uint256 amount
    );

    /// @notice Emitted when tokens are minted in response to an IFT transfer from a counterparty chain
    /// @param clientId The IBC client identifier over which the transfer was made
    /// @param receiver The address of the receiver on the local chain
    /// @param amount The amount of tokens minted
    event IFTMintReceived(string clientId, address indexed receiver, uint256 amount);

    /// @notice Emitted when a bridge transfer is successfully completed
    /// @param clientId The IBC client identifier over which the transfer was made
    /// @param sequence The IBC sequence number of the packet
    /// @param sender The address of the sender who initiated the transfer
    /// @param amount The amount of tokens transferred
    event IFTTransferCompleted(string clientId, uint64 sequence, address indexed sender, uint256 amount);

    /// @notice Emitted when a bridge transfer is refunded due to failure or timeout
    /// @param clientId The IBC client identifier over which the transfer was made
    /// @param sequence The IBC sequence number of the packet
    /// @param sender The address of the sender who is refunded
    /// @param amount The amount of tokens refunded
    event IFTTransferRefunded(string clientId, uint64 sequence, address indexed sender, uint256 amount);

    /// @notice Registers a new IBC bridge to a counterparty IFT contract
    /// @dev Only callable by the authority. Both sides must register before transfers can succeed.
    /// @param clientId The IBC client identifier on the local chain
    /// @param counterpartyIFTAddress The address of the IFT contract on the counterparty chain
    /// @param iftSendCallConstructor The address of the call constructor for the counterparty chain
    function registerIFTBridge(
        string calldata clientId,
        string calldata counterpartyIFTAddress,
        address iftSendCallConstructor
    )
        external;

    /// @notice Removes an existing IBC bridge
    /// @dev Only callable by the authority. Existing pending transfers will still be processable.
    /// @param clientId The IBC client identifier to remove
    function removeIFTBridge(string calldata clientId) external;

    /// @notice Initiates a transfer of tokens to a counterparty chain via IBC
    /// @dev Burns tokens from the sender and sends a GMP packet to mint on the counterparty
    /// @param clientId The IBC client identifier over which the transfer is being made
    /// @param receiver The address of the receiver on the counterparty chain
    /// @param amount The amount of tokens to transfer
    /// @param timeoutTimestamp The timeout timestamp for the IBC packet (unix seconds)
    function iftTransfer(
        string calldata clientId,
        string calldata receiver,
        uint256 amount,
        uint64 timeoutTimestamp
    )
        external;

    /// @notice Initiates a transfer with default timeout (15 minutes)
    /// @dev Burns tokens from the sender and sends a GMP packet to mint on the counterparty
    /// @param clientId The IBC client identifier over which the transfer is being made
    /// @param receiver The address of the receiver on the counterparty chain
    /// @param amount The amount of tokens to transfer
    function iftTransfer(string calldata clientId, string calldata receiver, uint256 amount) external;

    /// @notice Mints tokens on the local chain in response to an IFT transfer from a counterparty chain
    /// @dev Only callable by an ICS27-GMP account controlled by a registered counterparty bridge
    /// @param receiver The address of the receiver on the local chain
    /// @param amount The amount of tokens to mint
    function iftMint(address receiver, uint256 amount) external;

    /// @notice Retrieves the IBC bridge information for a given client ID
    /// @param clientId The IBC client identifier
    /// @return The IFTBridge structure for the given client ID
    function getIFTBridge(string calldata clientId) external view returns (IIFTMsgs.IFTBridge memory);

    /// @notice Retrieves a pending transfer by client ID and sequence
    /// @param clientId The IBC client identifier
    /// @param sequence The IBC sequence number
    /// @return The PendingTransfer structure
    function getPendingTransfer(
        string calldata clientId,
        uint64 sequence
    )
        external
        view
        returns (IIFTMsgs.PendingTransfer memory);

    /// @notice Returns the ICS27-GMP contract address
    /// @return The ICS27-GMP interface
    function ics27() external view returns (IICS27GMP);
}
