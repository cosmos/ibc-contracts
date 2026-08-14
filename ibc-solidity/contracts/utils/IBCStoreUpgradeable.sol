// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { IIBCStore } from "../interfaces/IIBCStore.sol";
import { IICS26RouterMsgs } from "../msgs/IICS26RouterMsgs.sol";
import { ICS24Host } from "./ICS24Host.sol";
import { IICS24HostErrors } from "../errors/IICS24HostErrors.sol";
import { Initializable } from "@openzeppelin-upgradeable/proxy/utils/Initializable.sol";

/// @title IBC Store Upgradeable
/// @notice This is the contract that stores the provable IBC commitments.
abstract contract IBCStoreUpgradeable is IIBCStore, IICS24HostErrors, Initializable {
    /// @notice Storage of the IBCStore contract
    /// @dev It's implemented on a custom ERC-7201 namespace to reduce the risk of storage collisions when using with
    /// upgradeable contracts.
    /// @param commitments Mapping of all IBC commitments
    /// @param prevSequenceSends Mapping of previous sequence sends for each client
    struct IBCStoreStorage {
        // keccak256(IBC-compatible-store-path) => sha256(IBC-compatible-commitment)
        mapping(bytes32 hashedPath => bytes32 commitment) commitments;
        mapping(string clientId => uint64 prevSeqSend) prevSequenceSends;
    }

    /// @notice ERC-7201 slot for the IBCStore storage
    /// @dev keccak256(abi.encode(uint256(keccak256("ibc.storage.IBCStore")) - 1)) & ~bytes32(uint256(0xff))
    bytes32 private constant IBCSTORE_STORAGE_SLOT = 0x1260944489272988d9df285149b5aa1b0f48f2136d6f416159f840a3e0747600;

    /* solhint-disable no-empty-blocks */

    /// @dev This function has no initialization logic
    // natlint-disable-next-line
    function __IBCStore_init() internal onlyInitializing { }

    /* solhint-enable no-empty-blocks */

    /// @inheritdoc IIBCStore
    function getCommitment(bytes32 hashedPath) public view returns (bytes32) {
        return _getIBCStoreStorage().commitments[hashedPath];
    }

    /// @notice Returns the next sequence send for the given client
    /// @param clientId The client identifier
    /// @return The next sequence send for the given client
    function nextSequenceSend(string calldata clientId) internal returns (uint64) {
        // initial sequence send should be 1, hence we use ++x instead of x++
        return ++_getIBCStoreStorage().prevSequenceSends[clientId];
    }

    /// @notice Commits the packet commitment for a packet if it doesn't already exist
    /// @param packet Packet to commit the commitment for
    function commitPacket(IICS26RouterMsgs.Packet memory packet) internal {
        IBCStoreStorage storage $ = _getIBCStoreStorage();

        bytes32 path = ICS24Host.packetCommitmentKeyCalldata(packet.sourceClient, packet.sequence);
        require(
            $.commitments[path] == 0,
            IBCPacketCommitmentAlreadyExists(
                ICS24Host.packetCommitmentPathCalldata(packet.sourceClient, packet.sequence)
            )
        );

        bytes32 commitment = ICS24Host.packetCommitmentBytes32(packet);
        $.commitments[path] = commitment;
    }

    /// @notice Deletes the packet commitment for the given packet if it exists
    /// @param packet Packet to delete the commitment for
    /// @return True if the packet commitment was found and then deleted, false otherwise
    function checkAndDeletePacketCommitment(IICS26RouterMsgs.Packet calldata packet) internal returns (bool) {
        IBCStoreStorage storage $ = _getIBCStoreStorage();

        bytes32 path = ICS24Host.packetCommitmentKeyCalldata(packet.sourceClient, packet.sequence);
        bytes32 commitment = $.commitments[path];
        if (commitment == 0) {
            return false;
        }
        require(
            commitment == ICS24Host.packetCommitmentBytes32(packet),
            IBCPacketCommitmentMismatch(commitment, ICS24Host.packetCommitmentBytes32(packet))
        );

        delete $.commitments[path];
        return true;
    }

    /// @notice Sets the packet receipt for the given packet if it doesn't already exist
    /// @dev This function reverts if the stored receipt is different from the one being set
    /// @param packet Packet to set the receipt for
    /// @return False if the receipt was already set, true otherwise
    function setPacketReceipt(IICS26RouterMsgs.Packet calldata packet) internal returns (bool) {
        IBCStoreStorage storage $ = _getIBCStoreStorage();

        bytes32 path = ICS24Host.packetReceiptCommitmentKeyCalldata(packet.destClient, packet.sequence);
        bytes32 receipt = ICS24Host.packetReceiptCommitmentBytes32(packet);
        bytes32 storedReceipt = $.commitments[path];
        if (storedReceipt == receipt) {
            return false;
        }
        require(storedReceipt == 0, IBCPacketReceiptMismatch(storedReceipt, receipt));

        $.commitments[path] = receipt;
        return true;
    }

    /// @notice Commits the successful packet acknowledgements for the given packet
    /// @param packet Packet to commit the acknowledgements for
    /// @param acks Acknowledgements to commit
    function commitPacketAcknowledgement(IICS26RouterMsgs.Packet calldata packet, bytes[] memory acks) internal {
        IBCStoreStorage storage $ = _getIBCStoreStorage();

        bytes32 path = ICS24Host.packetAcknowledgementCommitmentKeyCalldata(packet.destClient, packet.sequence);
        require(
            $.commitments[path] == 0,
            IBCPacketAcknowledgementAlreadyExists(
                ICS24Host.packetAcknowledgementCommitmentPathCalldata(packet.destClient, packet.sequence)
            )
        );

        bytes32 commitment = ICS24Host.packetAcknowledgementCommitmentBytes32(acks);
        $.commitments[path] = commitment;
    }

    /// @notice Returns the storage of the IBCStore contract
    /// @return $ The storage of the IBCStore contract
    function _getIBCStoreStorage() private pure returns (IBCStoreStorage storage $) {
        // solhint-disable-next-line no-inline-assembly
        assembly {
            $.slot := IBCSTORE_STORAGE_SLOT
        }
    }
}
