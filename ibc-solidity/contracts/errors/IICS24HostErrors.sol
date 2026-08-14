// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

/// @title IICS24HostErrors
/// @notice Interface for ICS24Host errors
interface IICS24HostErrors {
    /// @notice Packet commitment already exists
    /// @param path commitment path
    error IBCPacketCommitmentAlreadyExists(bytes path);

    /// @notice Packet acknowledgement already exists
    /// @param path commitment path
    error IBCPacketAcknowledgementAlreadyExists(bytes path);

    /// @notice Merkle prefix is invalid
    /// @param prefix The invalid prefix
    error InvalidMerklePrefix(bytes[] prefix);

    /// @notice Multi-payload packets are not supported
    error IBCMultiPayloadPacketNotSupported();

    /// @notice IBC packet commitment mismatch
    /// @param expected stored packet commitment
    /// @param actual actual packet commitment
    error IBCPacketCommitmentMismatch(bytes32 expected, bytes32 actual);

    /// @notice No acknowledgements to process
    error NoAcknowledgements();

    /// @notice IBC packet receipt mismatch
    /// @param expected stored packet receipt
    /// @param actual actual packet receipt
    error IBCPacketReceiptMismatch(bytes32 expected, bytes32 actual);
}
