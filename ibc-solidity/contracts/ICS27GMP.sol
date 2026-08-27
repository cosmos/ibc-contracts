// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { IICS26RouterMsgs } from "./msgs/IICS26RouterMsgs.sol";
import { IICS27GMPMsgs } from "./msgs/IICS27GMPMsgs.sol";
import { IIBCAppCallbacks } from "./msgs/IIBCAppCallbacks.sol";

import { IICS26Router } from "./interfaces/IICS26Router.sol";
import { IIBCApp } from "./interfaces/IIBCApp.sol";
import { IICS27GMP } from "./interfaces/IICS27GMP.sol";
import { IICS27Account } from "./interfaces/IICS27Account.sol";
import { IICS27Errors } from "./errors/IICS27Errors.sol";
import { IPausable } from "./interfaces/IPausable.sol";

import { ReentrancyGuardTransient } from "@openzeppelin-contracts/utils/ReentrancyGuardTransient.sol";
import { MulticallUpgradeable } from "@openzeppelin-upgradeable/utils/MulticallUpgradeable.sol";
import { Strings } from "@openzeppelin-contracts/utils/Strings.sol";
import { Create2 } from "@openzeppelin-contracts/utils/Create2.sol";
import { UUPSUpgradeable } from "@openzeppelin-contracts/proxy/utils/UUPSUpgradeable.sol";
import { UpgradeableBeacon } from "@openzeppelin-contracts/proxy/beacon/UpgradeableBeacon.sol";
import { ICS24Host } from "./utils/ICS24Host.sol";
import { ICS27Lib } from "./utils/ICS27Lib.sol";
import { IBCSenderCallbacksLib } from "./utils/IBCSenderCallbacksLib.sol";
import { AccessManagedUpgradeable } from "@openzeppelin-upgradeable/access/manager/AccessManagedUpgradeable.sol";
import { PausableUpgradeable } from "@openzeppelin-upgradeable/utils/PausableUpgradeable.sol";

/// @title ICS27 General Message Passing
/// @notice This contract is the implementation of the ics27-2 IBC specification for general message passing.
contract ICS27GMP is
    IICS27Errors,
    IICS27GMP,
    IIBCApp,
    IPausable,
    ReentrancyGuardTransient,
    MulticallUpgradeable,
    AccessManagedUpgradeable,
    UUPSUpgradeable,
    PausableUpgradeable
{
    /// @notice Storage of the ICS27GMP contract
    /// @dev It's implemented on a custom ERC-7201 namespace to reduce the risk of storage collisions when using with
    /// upgradeable contracts.
    /// @param _accounts The mapping of account identifiers to account contracts.
    /// @param _accountIds The mapping of account contracts to account identifiers. (only for reverse lookup)
    /// @param _ics26 The ICS26Router contract address. Immutable.
    /// @param _accountBeacon The address of the ICS27Account beacon contract. Immutable.
    /// @custom:storage-location erc7201:ibc.storage.ICS27GMP
    struct ICS27GMPStorage {
        mapping(bytes32 accountIdHash => IICS27Account account) _accounts;
        mapping(address account => IICS27GMPMsgs.AccountIdentifier accountId) _accountIds;
        IICS26Router _ics26;
        UpgradeableBeacon _accountBeacon;
    }

    /// @notice ERC-7201 slot for the ICS27GMP storage
    /// @dev keccak256(abi.encode(uint256(keccak256("ibc.storage.ICS27GMP")) - 1)) & ~bytes32(uint256(0xff))
    bytes32 private constant ICS27GMP_STORAGE_SLOT = 0xe73deb02cd654f25b90ec94b434589ea350a706e2446d278b41c3a86dc8f4500;

    /// @dev This contract is meant to be deployed by a proxy, so the constructor is not used
    // natlint-disable-next-line MissingNotice
    constructor() {
        _disableInitializers();
    }

    /// @inheritdoc IICS27GMP
    function initialize(address ics26_, address accountLogic, address authority) external initializer {
        __Multicall_init();
        __Pausable_init();
        __AccessManaged_init(authority);

        ICS27GMPStorage storage $ = _getICS27GMPStorage();
        $._ics26 = IICS26Router(ics26_);
        $._accountBeacon = new UpgradeableBeacon(accountLogic, address(this));
    }

    /// @inheritdoc IPausable
    function pause() external restricted {
        _pause();
    }

    /// @inheritdoc IPausable
    function unpause() external restricted {
        _unpause();
    }

    /// @inheritdoc IICS27GMP
    function ics26() external view returns (address) {
        return address(_getICS27GMPStorage()._ics26);
    }

    /// @inheritdoc IICS27GMP
    function getAccountBeacon() external view returns (address) {
        return address(_getICS27GMPStorage()._accountBeacon);
    }

    /// @inheritdoc IICS27GMP
    function getAccountIdentifier(address account) external view returns (IICS27GMPMsgs.AccountIdentifier memory) {
        ICS27GMPStorage storage $ = _getICS27GMPStorage();

        IICS27GMPMsgs.AccountIdentifier memory accountId = $._accountIds[account];
        require(bytes(accountId.clientId).length != 0, ICS27AccountNotFound(account));
        return accountId;
    }

    /// @inheritdoc IICS27GMP
    function sendCall(IICS27GMPMsgs.SendCallMsg calldata msg_) external nonReentrant whenNotPaused returns (uint64) {
        require(msg_.payload.length != 0, ICS27PayloadEmpty());

        IICS27GMPMsgs.GMPPacketData memory packetData = IICS27GMPMsgs.GMPPacketData({
            sender: Strings.toChecksumHexString(_msgSender()),
            receiver: msg_.receiver,
            salt: msg_.salt,
            payload: msg_.payload,
            memo: msg_.memo
        });

        return _getICS27GMPStorage()
            ._ics26
            .sendPacket(
                IICS26RouterMsgs.MsgSendPacket({
                    sourceClient: msg_.sourceClient,
                    timeoutTimestamp: msg_.timeoutTimestamp,
                    payload: IICS26RouterMsgs.Payload({
                        sourcePort: ICS27Lib.DEFAULT_PORT_ID,
                        destPort: ICS27Lib.DEFAULT_PORT_ID,
                        version: ICS27Lib.ICS27_VERSION,
                        encoding: ICS27Lib.ICS27_ENCODING,
                        value: abi.encode(packetData)
                    })
                })
            );
    }

    /// @inheritdoc IIBCApp
    function onRecvPacket(IIBCAppCallbacks.OnRecvPacketCallback calldata msg_)
        external
        nonReentrant
        whenNotPaused
        onlyRouter
        returns (bytes memory)
    {
        require(
            keccak256(bytes(msg_.payload.version)) == ICS27Lib.KECCAK256_ICS27_VERSION,
            ICS27UnexpectedVersion(ICS27Lib.ICS27_VERSION, msg_.payload.version)
        );
        require(
            keccak256(bytes(msg_.payload.sourcePort)) == ICS27Lib.KECCAK256_DEFAULT_PORT_ID,
            ICS27InvalidPort(ICS27Lib.DEFAULT_PORT_ID, msg_.payload.sourcePort)
        );
        require(
            keccak256(bytes(msg_.payload.encoding)) == ICS27Lib.KECCAK256_ICS27_ENCODING,
            ICS27UnexpectedEncoding(ICS27Lib.ICS27_ENCODING, msg_.payload.encoding)
        );
        require(
            keccak256(bytes(msg_.payload.destPort)) == ICS27Lib.KECCAK256_DEFAULT_PORT_ID,
            ICS27InvalidPort(ICS27Lib.DEFAULT_PORT_ID, msg_.payload.destPort)
        );

        IICS27GMPMsgs.GMPPacketData memory packetData = abi.decode(msg_.payload.value, (IICS27GMPMsgs.GMPPacketData));
        require(packetData.payload.length != 0, ICS27PayloadEmpty());

        IICS27GMPMsgs.AccountIdentifier memory accountId = IICS27GMPMsgs.AccountIdentifier({
            clientId: msg_.destinationClient, sender: packetData.sender, salt: packetData.salt
        });
        IICS27Account account = _getOrCreateAccount(accountId);

        (bool success, address receiver) = Strings.tryParseAddress(packetData.receiver);
        require(success, ICS27InvalidReceiver(packetData.receiver));

        bytes memory result = account.functionCall(receiver, packetData.payload);
        return ICS27Lib.acknowledgement(result);
    }

    /// @inheritdoc IIBCApp
    function onAcknowledgementPacket(IIBCAppCallbacks.OnAcknowledgementPacketCallback calldata msg_)
        external
        nonReentrant
        whenNotPaused
        onlyRouter
    {
        IICS27GMPMsgs.GMPPacketData memory packetData = abi.decode(msg_.payload.value, (IICS27GMPMsgs.GMPPacketData));

        (bool success, address sender) = Strings.tryParseAddress(packetData.sender);
        require(success, ICS27InvalidSender(packetData.sender));

        IBCSenderCallbacksLib.ackPacketCallback(
            sender, keccak256(msg_.acknowledgement) != ICS24Host.KECCAK256_UNIVERSAL_ERROR_ACK, msg_
        );
    }

    /// @inheritdoc IIBCApp
    function onTimeoutPacket(IIBCAppCallbacks.OnTimeoutPacketCallback calldata msg_)
        external
        nonReentrant
        whenNotPaused
        onlyRouter
    {
        IICS27GMPMsgs.GMPPacketData memory packetData = abi.decode(msg_.payload.value, (IICS27GMPMsgs.GMPPacketData));

        (bool success, address sender) = Strings.tryParseAddress(packetData.sender);
        require(success, ICS27InvalidSender(packetData.sender));

        IBCSenderCallbacksLib.timeoutPacketCallback(sender, msg_);
    }

    /// @notice Creates or retrieves an account contract for the given account identifier
    /// @param accountId The account identifier
    /// @return account The account contract address
    function _getOrCreateAccount(IICS27GMPMsgs.AccountIdentifier memory accountId) private returns (IICS27Account) {
        ICS27GMPStorage storage $ = _getICS27GMPStorage();

        bytes32 accountIdHash = keccak256(abi.encode(accountId));
        IICS27Account account = $._accounts[accountIdHash];
        if (address(account) != address(0)) {
            return account;
        }

        bytes memory bytecode = ICS27Lib.getBeaconProxyBytecode(address($._accountBeacon), address(this));
        address accountAddress = Create2.deploy(0, accountIdHash, bytecode);

        $._accounts[accountIdHash] = IICS27Account(accountAddress);
        $._accountIds[accountAddress] = accountId;
        return IICS27Account(accountAddress);
    }

    /// @inheritdoc IICS27GMP
    function getOrComputeAccountAddress(IICS27GMPMsgs.AccountIdentifier calldata accountId)
        external
        view
        returns (address)
    {
        ICS27GMPStorage storage $ = _getICS27GMPStorage();

        bytes32 accountIdHash = keccak256(abi.encode(accountId));
        address account = address($._accounts[accountIdHash]);
        if (account != address(0)) {
            return account;
        }

        bytes32 codeHash = ICS27Lib.getBeaconProxyCodeHash(address($._accountBeacon), address(this));
        return Create2.computeAddress(accountIdHash, codeHash);
    }

    /// @notice Returns the storage of the ICS27GMP contract
    /// @return $ The storage of the ICS27GMP contract
    function _getICS27GMPStorage() private pure returns (ICS27GMPStorage storage $) {
        // solhint-disable-next-line no-inline-assembly
        assembly {
            $.slot := ICS27GMP_STORAGE_SLOT
        }
    }

    /// @inheritdoc IICS27GMP
    function upgradeAccountTo(address newEscrowLogic) external restricted {
        _getICS27GMPStorage()._accountBeacon.upgradeTo(newEscrowLogic);
    }

    /// @inheritdoc UUPSUpgradeable
    function _authorizeUpgrade(address) internal override restricted { }
    // solhint-disable-previous-line no-empty-blocks

    /// @notice Modifier to check if the caller is the ICS26Router contract
    modifier onlyRouter() {
        address router = address(_getICS27GMPStorage()._ics26);
        require(_msgSender() == router, ICS27Unauthorized(router, _msgSender()));
        _;
    }
}
