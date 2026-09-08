// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

/// @title IIFTErrors
/// @notice Interface for IFT errors
interface IIFTErrors {
    /// @notice Client ID cannot be empty
    error IFTEmptyClientId();

    /// @notice Receiver address cannot be empty
    error IFTEmptyReceiver();

    /// @notice Transfer amount must be greater than zero
    error IFTZeroAmount();

    /// @notice No bridge registered for the given client ID
    /// @param clientId The client identifier
    error IFTBridgeNotFound(string clientId);

    /// @notice Mint caller is not authorized (not from registered counterparty bridge)
    /// @param expected The expected counterparty address
    /// @param actual The actual sender address from account identifier
    error IFTUnauthorizedMint(string expected, string actual);

    /// @notice Salt must be empty for IFT mints
    /// @param salt The non-empty salt received
    error IFTUnexpectedSalt(bytes salt);

    /// @notice No pending transfer found for the given client ID and sequence
    /// @param clientId The client identifier
    /// @param sequence The packet sequence number
    error IFTPendingTransferNotFound(string clientId, uint64 sequence);

    /// @notice Caller is not the ICS27-GMP contract
    /// @param caller The actual caller address
    error IFTOnlyICS27GMP(address caller);

    /// @notice Failed to parse receiver address
    /// @param receiver The invalid receiver string
    error IFTInvalidReceiver(string receiver);

    /// @notice IFT send call constructor cannot be zero address
    error IFTZeroAddressConstructor();

    /// @notice Counterparty IFT address cannot be empty
    error IFTEmptyCounterpartyAddress();

    /// @notice Timeout timestamp must be in the future
    /// @param timeout The invalid timeout timestamp
    /// @param currentTime The current block timestamp
    error IFTTimeoutInPast(uint64 timeout, uint64 currentTime);

    /// @notice IFT send call constructor does not support required interface
    /// @param callConstructor The address that failed the interface check
    error IFTInvalidConstructorInterface(address callConstructor);
}
