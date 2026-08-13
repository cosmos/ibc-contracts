// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { ILightClientMsgs } from "./msgs/ILightClientMsgs.sol";
import { IICS26RouterMsgs } from "./msgs/IICS26RouterMsgs.sol";
import { IICS02ClientMsgs } from "./msgs/IICS02ClientMsgs.sol";
import { IIBCAppCallbacks } from "./msgs/IIBCAppCallbacks.sol";

import { IICS26RouterErrors } from "./errors/IICS26RouterErrors.sol";
import { IIBCApp } from "./interfaces/IIBCApp.sol";
import { IICS26Router, IICS26RouterAccessControlled } from "./interfaces/IICS26Router.sol";

import { ReentrancyGuardTransient } from "@openzeppelin-contracts/utils/ReentrancyGuardTransient.sol";
import { IBCStoreUpgradeable } from "./utils/IBCStoreUpgradeable.sol";
import { Strings } from "@openzeppelin-contracts/utils/Strings.sol";
import { IBCIdentifiers } from "./utils/IBCIdentifiers.sol";
import { ICS24Host } from "./utils/ICS24Host.sol";
import { ICS02ClientUpgradeable } from "./utils/ICS02ClientUpgradeable.sol";
import { MulticallUpgradeable } from "@openzeppelin-upgradeable/utils/MulticallUpgradeable.sol";
import { UUPSUpgradeable } from "@openzeppelin-contracts/proxy/utils/UUPSUpgradeable.sol";

/// @title IBC Eureka Router
/// @notice The core router for the IBC Eureka protocol
contract ICS26Router is
    IICS26RouterErrors,
    IICS26Router,
    ICS02ClientUpgradeable,
    IBCStoreUpgradeable,
    ReentrancyGuardTransient,
    MulticallUpgradeable,
    UUPSUpgradeable
{
    /// @notice Storage of the ICS26Router contract
    /// @dev It's implemented on a custom ERC-7201 namespace to reduce the risk of storage collisions when using with
    /// upgradeable contracts.
    /// @param _apps The mapping of port identifiers to IBC application contracts
    /// @custom:storage-location erc7201:ibc.storage.ICS26Router
    struct ICS26RouterStorage {
        mapping(string => IIBCApp) _apps;
    }

    /// @notice ERC-7201 slot for the ICS26Router storage
    /// @dev keccak256(abi.encode(uint256(keccak256("ibc.storage.ICS26Router")) - 1)) & ~bytes32(uint256(0xff))
    bytes32 private constant ICS26ROUTER_STORAGE_SLOT =
        0xc5779f3c2c21083eefa6d04f6a698bc0d8c10db124ad5e0df6ef394b6d7bf600;

    /// @notice The maximum timeout duration for a packet
    uint256 private constant MAX_TIMEOUT_DURATION = 1 days;

    /// @dev This contract is meant to be deployed by a proxy, so the constructor is not used
    // natlint-disable-next-line MissingNotice
    constructor() {
        _disableInitializers();
    }

    /// @inheritdoc IICS26Router
    function initialize(address authority) external onlyVersion(0) reinitializer(2) {
        __Multicall_init();
        __IBCStore_init();
        __ICS02Client_init(authority);
    }

    /// @inheritdoc IICS26Router
    function getIBCApp(string calldata portId) public view returns (IIBCApp) {
        IIBCApp app = _getICS26RouterStorage()._apps[portId];
        require(address(app) != address(0), IBCAppNotFound(portId));
        return app;
    }

    /// @inheritdoc IICS26Router
    function addIBCApp(address app) external nonReentrant {
        string memory portId = Strings.toHexString(app);
        _addIBCApp(portId, app);
    }

    /// @inheritdoc IICS26RouterAccessControlled
    function addIBCApp(string calldata portId, address app) external nonReentrant restricted {
        require(bytes(portId).length != 0, IBCInvalidPortIdentifier(portId));
        // NOTE: We do not allow port-ids to be addresses, as this would conflict with the permissionless port-ids
        // slither-disable-next-line unused-return
        (bool isAddress,) = Strings.tryParseAddress(portId);
        require(!isAddress, IBCInvalidPortIdentifier(portId));
        require(IBCIdentifiers.validateCustomIBCIdentifier(bytes(portId)), IBCInvalidPortIdentifier(portId));
        _addIBCApp(portId, app);
    }

    /// @notice This function adds an app to the app router
    /// @dev This function assumes that the portId has already been generated and validated.
    /// @param portId The port identifier
    /// @param app The address of the app contract
    function _addIBCApp(string memory portId, address app) private {
        ICS26RouterStorage storage $ = _getICS26RouterStorage();
        require(address($._apps[portId]) == address(0), IBCPortAlreadyExists(portId));
        $._apps[portId] = IIBCApp(app);
        emit IBCAppAdded(portId, app);
    }

    /// @inheritdoc IICS26Router
    function sendPacket(IICS26RouterMsgs.MsgSendPacket calldata msg_) external nonReentrant returns (uint64) {
        address ibcApp = address(getIBCApp(msg_.payload.sourcePort));
        require(ibcApp == _msgSender(), IBCUnauthorizedSender(_msgSender()));

        string memory counterpartyId = getCounterparty(msg_.sourceClient).clientId;

        require(
            msg_.timeoutTimestamp > block.timestamp, IBCInvalidTimeoutTimestamp(msg_.timeoutTimestamp, block.timestamp)
        );
        require(
            // solhint-disable-next-line gas-strict-inequalities
            msg_.timeoutTimestamp - block.timestamp <= MAX_TIMEOUT_DURATION,
            IBCInvalidTimeoutDuration(MAX_TIMEOUT_DURATION, msg_.timeoutTimestamp - block.timestamp)
        );

        uint64 sequence = nextSequenceSend(msg_.sourceClient);

        // TODO: Support multi-payload packets #93
        IICS26RouterMsgs.Packet memory packet = IICS26RouterMsgs.Packet({
            sequence: sequence,
            sourceClient: msg_.sourceClient,
            destClient: counterpartyId,
            timeoutTimestamp: msg_.timeoutTimestamp,
            payloads: new IICS26RouterMsgs.Payload[](1)
        });
        packet.payloads[0] = msg_.payload;

        commitPacket(packet);

        emit SendPacket(msg_.sourceClient, sequence, packet);
        return sequence;
    }

    /// @inheritdoc IICS26RouterAccessControlled
    // NOTE: Reentrancy disabled for this function via the `nonReentrant` modifier.
    // slither-disable-next-line reentrancy-no-eth
    function recvPacket(IICS26RouterMsgs.MsgRecvPacket calldata msg_) external nonReentrant restricted {
        // TODO: Support multi-payload packets (#93)
        require(msg_.packet.payloads.length == 1, IBCMultiPayloadPacketNotSupported());
        IICS26RouterMsgs.Payload calldata payload = msg_.packet.payloads[0];

        IICS02ClientMsgs.CounterpartyInfo memory cInfo = getCounterparty(msg_.packet.destClient);
        require(
            keccak256(bytes(cInfo.clientId)) == keccak256(bytes(msg_.packet.sourceClient)),
            IBCInvalidCounterparty(cInfo.clientId, msg_.packet.sourceClient)
        );

        require(
            msg_.packet.timeoutTimestamp > block.timestamp,
            IBCInvalidTimeoutTimestamp(msg_.packet.timeoutTimestamp, block.timestamp)
        );

        bytes memory commitmentPath =
            ICS24Host.packetCommitmentPathCalldata(msg_.packet.sourceClient, msg_.packet.sequence);
        bytes32 commitmentBz = ICS24Host.packetCommitmentBytes32(msg_.packet);

        ILightClientMsgs.MsgVerifyMembership memory membershipMsg = ILightClientMsgs.MsgVerifyMembership({
            proof: msg_.proofCommitment,
            proofHeight: msg_.proofHeight,
            path: ICS24Host.prefixedPath(cInfo.merklePrefix, commitmentPath),
            value: abi.encodePacked(commitmentBz)
        });
        // NOTE: The verification timestamp is not used here in the IBC Eureka Specifications
        // slither-disable-next-line unused-return
        getClient(msg_.packet.destClient).verifyMembership(membershipMsg);

        // recvPacket will no-op if the packet receipt already exists
        // This no-op check must happen after the membership verification for proofs to be cached
        bool receiptAlreadySet = !setPacketReceipt(msg_.packet);
        if (receiptAlreadySet) {
            emit Noop();
            return;
        }

        bytes[] memory acks = new bytes[](1);
        try getIBCApp(payload.destPort)
            .onRecvPacket(
                IIBCAppCallbacks.OnRecvPacketCallback({
                sourceClient: msg_.packet.sourceClient,
                destinationClient: msg_.packet.destClient,
                sequence: msg_.packet.sequence,
                payload: payload,
                relayer: _msgSender()
            })
            ) returns (
            bytes memory ack
        ) {
            require(ack.length != 0, IBCAsyncAcknowledgementNotSupported());
            require(keccak256(ack) != ICS24Host.KECCAK256_UNIVERSAL_ERROR_ACK, IBCErrorUniversalAcknowledgement());
            acks[0] = ack;
        } catch (bytes memory reason) {
            require(reason.length != 0, IBCFailedCallback()); // covers OOG
            emit IBCAppRecvPacketCallbackError(reason);
            acks[0] = ICS24Host.UNIVERSAL_ERROR_ACK;
        }

        commitPacketAcknowledgement(msg_.packet, acks);
        emit WriteAcknowledgement(msg_.packet.destClient, msg_.packet.sequence, msg_.packet, acks);
    }

    /// @inheritdoc IICS26RouterAccessControlled
    // NOTE: Reentrancy disabled for this function via the `nonReentrant` modifier.
    // slither-disable-next-line reentrancy-no-eth
    function ackPacket(IICS26RouterMsgs.MsgAckPacket calldata msg_) external nonReentrant restricted {
        // TODO: Support multi-payload packets #93
        require(msg_.packet.payloads.length == 1, IBCMultiPayloadPacketNotSupported());
        IICS26RouterMsgs.Payload calldata payload = msg_.packet.payloads[0];

        IICS02ClientMsgs.CounterpartyInfo memory cInfo = getCounterparty(msg_.packet.sourceClient);
        require(
            keccak256(bytes(cInfo.clientId)) == keccak256(bytes(msg_.packet.destClient)),
            IBCInvalidCounterparty(cInfo.clientId, msg_.packet.destClient)
        );

        bytes memory commitmentPath =
            ICS24Host.packetAcknowledgementCommitmentPathCalldata(msg_.packet.destClient, msg_.packet.sequence);
        bytes[] memory acks = new bytes[](1);
        acks[0] = msg_.acknowledgement;
        bytes32 commitmentBz = ICS24Host.packetAcknowledgementCommitmentBytes32(acks);

        // verify the packet acknowledgement
        ILightClientMsgs.MsgVerifyMembership memory membershipMsg = ILightClientMsgs.MsgVerifyMembership({
            proof: msg_.proofAcked,
            proofHeight: msg_.proofHeight,
            path: ICS24Host.prefixedPath(cInfo.merklePrefix, commitmentPath),
            value: abi.encodePacked(commitmentBz)
        });
        // NOTE: The verification timestamp is not used here in the IBC Eureka Specifications
        // slither-disable-next-line unused-return
        getClient(msg_.packet.sourceClient).verifyMembership(membershipMsg);

        // ackPacket will no-op if the packet commitment does not exist
        // This no-op check must happen after the membership verification for proofs to be cached
        bool commitmentFound = checkAndDeletePacketCommitment(msg_.packet);
        if (!commitmentFound) {
            emit Noop();
            return;
        }

        getIBCApp(payload.sourcePort)
            .onAcknowledgementPacket(
                IIBCAppCallbacks.OnAcknowledgementPacketCallback({
                sourceClient: msg_.packet.sourceClient,
                destinationClient: msg_.packet.destClient,
                sequence: msg_.packet.sequence,
                payload: payload,
                acknowledgement: msg_.acknowledgement,
                relayer: _msgSender()
            })
            );

        emit AckPacket(msg_.packet.sourceClient, msg_.packet.sequence, msg_.packet, msg_.acknowledgement);
    }

    /// @inheritdoc IICS26RouterAccessControlled
    function timeoutPacket(IICS26RouterMsgs.MsgTimeoutPacket calldata msg_) external nonReentrant restricted {
        // TODO: Support multi-payload packets #93
        require(msg_.packet.payloads.length == 1, IBCMultiPayloadPacketNotSupported());
        IICS26RouterMsgs.Payload calldata payload = msg_.packet.payloads[0];

        IICS02ClientMsgs.CounterpartyInfo memory cInfo = getCounterparty(msg_.packet.sourceClient);
        require(
            keccak256(bytes(cInfo.clientId)) == keccak256(bytes(msg_.packet.destClient)),
            IBCInvalidCounterparty(cInfo.clientId, msg_.packet.destClient)
        );

        bytes memory receiptPath =
            ICS24Host.packetReceiptCommitmentPathCalldata(msg_.packet.destClient, msg_.packet.sequence);
        ILightClientMsgs.MsgVerifyNonMembership memory nonMembershipMsg = ILightClientMsgs.MsgVerifyNonMembership({
            proof: msg_.proofTimeout,
            proofHeight: msg_.proofHeight,
            path: ICS24Host.prefixedPath(cInfo.merklePrefix, receiptPath)
        });
        uint256 counterpartyTimestamp = getClient(msg_.packet.sourceClient).verifyNonMembership(nonMembershipMsg);
        require(
            // solhint-disable-next-line gas-strict-inequalities
            counterpartyTimestamp >= msg_.packet.timeoutTimestamp,
            IBCInvalidTimeoutTimestamp(msg_.packet.timeoutTimestamp, counterpartyTimestamp)
        );

        // timeoutPacket will no-op if the packet commitment does not exist
        // This no-op check must happen after the membership verification for proofs to be cached
        bool commitmentFound = checkAndDeletePacketCommitment(msg_.packet);
        if (!commitmentFound) {
            emit Noop();
            return;
        }

        getIBCApp(payload.sourcePort)
            .onTimeoutPacket(
                IIBCAppCallbacks.OnTimeoutPacketCallback({
                sourceClient: msg_.packet.sourceClient,
                destinationClient: msg_.packet.destClient,
                sequence: msg_.packet.sequence,
                payload: payload,
                relayer: _msgSender()
            })
            );

        emit TimeoutPacket(msg_.packet.sourceClient, msg_.packet.sequence, msg_.packet);
    }

    /// @notice Returns the storage of the ICS26Router contract
    /// @return $ The storage of the ICS26Router contract
    function _getICS26RouterStorage() private pure returns (ICS26RouterStorage storage $) {
        // solhint-disable-next-line no-inline-assembly
        assembly {
            $.slot := ICS26ROUTER_STORAGE_SLOT
        }
    }

    /// @inheritdoc UUPSUpgradeable
    function _authorizeUpgrade(address) internal override restricted { }
    // solhint-disable-previous-line no-empty-blocks

    /// @notice Modifier to check if the initialization version matches the expected version
    /// @param version The expected current version of the contract
    modifier onlyVersion(uint256 version) {
        require(_getInitializedVersion() == version, InvalidInitialization());
        _;
    }
}
