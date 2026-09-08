// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { IICS26RouterMsgs } from "../msgs/IICS26RouterMsgs.sol";

import { IIBCStore } from "../interfaces/IIBCStore.sol";
import { IRelayerHelper } from "../interfaces/IRelayerHelper.sol";

import { ICS24Host } from "./ICS24Host.sol";

/// @title Relayer Helper
/// @notice RelayerHelper is a helper contract for relayers, providing utility queries.
/// @dev Upcoming features are tracked in #424
contract RelayerHelper is IRelayerHelper {
    /// @inheritdoc IRelayerHelper
    address public immutable ICS26_ROUTER;

    /// @notice Constructs the RelayerHelper contract by setting the ICS26 router address.
    /// @param _ics26Router The address of the ICS26 router contract
    constructor(address _ics26Router) {
        ICS26_ROUTER = _ics26Router;
    }

    /// @inheritdoc IRelayerHelper
    function isPacketReceived(IICS26RouterMsgs.Packet calldata packet) public view returns (bool) {
        bytes32 expReceipt = ICS24Host.packetReceiptCommitmentBytes32(packet);
        return expReceipt == queryPacketReceipt(packet.destClient, packet.sequence);
    }

    /// @inheritdoc IRelayerHelper
    function isPacketReceiveSuccessful(IICS26RouterMsgs.Packet calldata packet) external view returns (bool) {
        if (!isPacketReceived(packet)) {
            return false;
        }

        bytes[] memory errorAck = new bytes[](1);
        errorAck[0] = ICS24Host.UNIVERSAL_ERROR_ACK;
        bytes32 errorAckCommitment = ICS24Host.packetAcknowledgementCommitmentBytes32(errorAck);
        bytes32 storedAckCommitment = queryAckCommitment(packet.destClient, packet.sequence);
        return storedAckCommitment != 0 && storedAckCommitment != errorAckCommitment;
    }

    /// @inheritdoc IRelayerHelper
    function queryPacketReceipt(string calldata clientId, uint64 sequence) public view returns (bytes32) {
        bytes32 path = ICS24Host.packetReceiptCommitmentKeyCalldata(clientId, sequence);
        return IIBCStore(ICS26_ROUTER).getCommitment(path);
    }

    /// @inheritdoc IRelayerHelper
    function queryPacketCommitment(string calldata clientId, uint64 sequence) public view returns (bytes32) {
        bytes32 path = ICS24Host.packetCommitmentKeyCalldata(clientId, sequence);
        return IIBCStore(ICS26_ROUTER).getCommitment(path);
    }

    /// @inheritdoc IRelayerHelper
    function queryAckCommitment(string calldata clientId, uint64 sequence) public view returns (bytes32) {
        bytes32 path = ICS24Host.packetAcknowledgementCommitmentKeyCalldata(clientId, sequence);
        return IIBCStore(ICS26_ROUTER).getCommitment(path);
    }
}
