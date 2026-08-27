// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { IICS26RouterMsgs } from "./msgs/IICS26RouterMsgs.sol";
import { IICS20TransferMsgs } from "./msgs/IICS20TransferMsgs.sol";
import { IIBCAppCallbacks } from "./msgs/IIBCAppCallbacks.sol";

import { IICS20Errors } from "./errors/IICS20Errors.sol";
import { IEscrow } from "./interfaces/IEscrow.sol";
import { IIBCApp } from "./interfaces/IIBCApp.sol";
import { IERC20 } from "@openzeppelin-contracts/token/ERC20/IERC20.sol";
import { IICS20Transfer, IICS20TransferAccessControlled } from "./interfaces/IICS20Transfer.sol";
import { IICS26Router } from "./interfaces/IICS26Router.sol";
import { ISignatureTransfer } from "@uniswap/permit2/src/interfaces/ISignatureTransfer.sol";
import { IMintableAndBurnable } from "./interfaces/IMintableAndBurnable.sol";
import { IIBCERC20 } from "./interfaces/IIBCERC20.sol";
import { IPausable } from "./interfaces/IPausable.sol";

import { ReentrancyGuardTransient } from "@openzeppelin-contracts/utils/ReentrancyGuardTransient.sol";
import { SafeERC20 } from "@openzeppelin-contracts/token/ERC20/utils/SafeERC20.sol";
import { MulticallUpgradeable } from "@openzeppelin-upgradeable/utils/MulticallUpgradeable.sol";
import { ICS20Lib } from "./utils/ICS20Lib.sol";
import { ICS24Host } from "./utils/ICS24Host.sol";
import { Strings } from "@openzeppelin-contracts/utils/Strings.sol";
import { Bytes } from "@openzeppelin-contracts/utils/Bytes.sol";
import { UUPSUpgradeable } from "@openzeppelin-contracts/proxy/utils/UUPSUpgradeable.sol";
import { BeaconProxy } from "@openzeppelin-contracts/proxy/beacon/BeaconProxy.sol";
import { UpgradeableBeacon } from "@openzeppelin-contracts/proxy/beacon/UpgradeableBeacon.sol";
import { IBCSenderCallbacksLib } from "./utils/IBCSenderCallbacksLib.sol";
import { PausableUpgradeable } from "@openzeppelin-upgradeable/utils/PausableUpgradeable.sol";
import { AccessManagedUpgradeable } from "@openzeppelin-upgradeable/access/manager/AccessManagedUpgradeable.sol";

using SafeERC20 for IERC20;

/// @title ICS20Transfer
/// @notice An implementation of the ics20-1 IBC specification for fungible token transfers.
contract ICS20Transfer is
    IICS20Errors,
    IICS20Transfer,
    IIBCApp,
    IPausable,
    ReentrancyGuardTransient,
    MulticallUpgradeable,
    UUPSUpgradeable,
    AccessManagedUpgradeable,
    PausableUpgradeable
{
    /// @notice Storage of the ICS20Transfer contract
    /// @dev It's implemented on a custom ERC-7201 namespace to reduce the risk of storage collisions when using with
    /// upgradeable contracts.
    /// @param _escrows The escrow contract per client.
    /// @param _ibcERC20Contracts Mapping of non-native denoms to their respective IBCERC20 contracts
    /// @param _ibcERC20Denoms Mapping of IBCERC20 contracts to their respective denoms.
    /// @param _ics26 The ICS26Router contract address. Immutable.
    /// @param _ibcERC20Beacon The address of the IBCERC20 beacon contract. Immutable.
    /// @param _escrowBeacon The address of the Escrow beacon contract. Immutable.
    /// @param _permit2 The permit2 contract. Immutable.
    /// @custom:storage-location erc7201:ibc.storage.ICS20Transfer
    struct ICS20TransferStorage {
        mapping(string clientId => IEscrow escrow) _escrows;
        mapping(string => IMintableAndBurnable) _ibcERC20Contracts;
        mapping(address => string) _ibcERC20Denoms;
        IICS26Router _ics26;
        UpgradeableBeacon _ibcERC20Beacon;
        UpgradeableBeacon _escrowBeacon;
        ISignatureTransfer _permit2;
    }

    /// @notice ERC-7201 slot for the ICS20Transfer storage
    /// @dev keccak256(abi.encode(uint256(keccak256("ibc.storage.ICS20Transfer")) - 1)) & ~bytes32(uint256(0xff))
    bytes32 private constant ICS20TRANSFER_STORAGE_SLOT =
        0x823f7a8ea9ae6df0eb03ec5e1682d7a2839417ad8a91774118e6acf2e8d2f800;

    /// @dev This contract is meant to be deployed by a proxy, so the constructor is not used
    // natlint-disable-next-line MissingNotice
    constructor() {
        _disableInitializers();
    }

    /// @inheritdoc IICS20Transfer
    function initialize(
        address ics26Router,
        address escrowLogic,
        address ibcERC20Logic,
        address permit2,
        address authority
    )
        public
        onlyVersion(0)
        reinitializer(2)
    {
        __Multicall_init();
        __Pausable_init();
        __AccessManaged_init(authority);

        ICS20TransferStorage storage $ = _getICS20TransferStorage();
        $._ics26 = IICS26Router(ics26Router);
        $._ibcERC20Beacon = new UpgradeableBeacon(ibcERC20Logic, address(this));
        $._escrowBeacon = new UpgradeableBeacon(escrowLogic, address(this));
        $._permit2 = ISignatureTransfer(permit2);
    }

    /// @inheritdoc IPausable
    function pause() external restricted {
        _pause();
    }

    /// @inheritdoc IPausable
    function unpause() external restricted {
        _unpause();
    }

    /// @inheritdoc IICS20Transfer
    function getEscrow(string calldata clientId) external view returns (address) {
        return address(_getICS20TransferStorage()._escrows[clientId]);
    }

    /// @inheritdoc IICS20Transfer
    function getEscrowBeacon() external view returns (address) {
        return address(_getICS20TransferStorage()._escrowBeacon);
    }

    /// @inheritdoc IICS20Transfer
    function getIBCERC20Beacon() external view returns (address) {
        return address(_getICS20TransferStorage()._ibcERC20Beacon);
    }

    /// @inheritdoc IICS20Transfer
    function ics26() external view returns (address) {
        return address(_getICS20TransferStorage()._ics26);
    }

    /// @inheritdoc IICS20Transfer
    function getPermit2() external view returns (address) {
        return address(_getICS20TransferStorage()._permit2);
    }

    /// @inheritdoc IICS20Transfer
    function ibcERC20Denom(address token) external view returns (string memory) {
        return _getICS20TransferStorage()._ibcERC20Denoms[token];
    }

    /// @inheritdoc IICS20Transfer
    function ibcERC20Contract(string calldata denom) external view returns (address) {
        address contractAddress = address(_getICS20TransferStorage()._ibcERC20Contracts[denom]);
        require(contractAddress != address(0), ICS20DenomNotFound(denom));
        return contractAddress;
    }

    /// @inheritdoc IICS20Transfer
    // NOTE: Reentrancy disabled for this function via the `nonReentrant` modifier.
    // slither-disable-next-line reentrancy-no-eth
    function sendTransfer(IICS20TransferMsgs.SendTransferMsg calldata msg_)
        external
        whenNotPaused
        nonReentrant
        returns (uint64)
    {
        require(msg_.amount > 0, IICS20Errors.ICS20InvalidAmount(0));
        // transfer the tokens to us (requires the allowance to be set)
        IEscrow escrow = _getOrCreateEscrow(msg_.sourceClient);
        _transferFrom(_msgSender(), address(escrow), msg_.denom, msg_.amount);
        escrow.recvCallback(msg_.denom, _msgSender(), msg_.amount);

        return _sendTransferFromEscrowWithSender(msg_, address(escrow), _msgSender());
    }

    /// @inheritdoc IICS20Transfer
    // NOTE: Reentrancy disabled for this function via the `nonReentrant` modifier.
    // slither-disable-next-line reentrancy-no-eth
    function sendTransferWithPermit2(
        IICS20TransferMsgs.SendTransferMsg calldata msg_,
        ISignatureTransfer.PermitTransferFrom calldata permit,
        bytes calldata signature
    )
        external
        whenNotPaused
        nonReentrant
        returns (uint64)
    {
        require(msg_.amount > 0, IICS20Errors.ICS20InvalidAmount(0));
        require(
            permit.permitted.token == msg_.denom,
            IICS20Errors.ICS20Permit2TokenMismatch(permit.permitted.token, msg_.denom)
        );
        // transfer the tokens to us with permit
        IEscrow escrow = _getOrCreateEscrow(msg_.sourceClient);
        _transferFromWithPermit2(_msgSender(), address(escrow), msg_.denom, msg_.amount, permit, signature);
        escrow.recvCallback(msg_.denom, _msgSender(), msg_.amount);

        return _sendTransferFromEscrowWithSender(msg_, address(escrow), _msgSender());
    }

    /// @inheritdoc IICS20TransferAccessControlled
    // NOTE: Reentrancy disabled for this function via the `nonReentrant` modifier.
    // slither-disable-next-line reentrancy-no-eth
    function sendTransferWithSender(
        IICS20TransferMsgs.SendTransferMsg calldata msg_,
        address sender
    )
        external
        whenNotPaused
        nonReentrant
        restricted
        returns (uint64)
    {
        require(msg_.amount > 0, IICS20Errors.ICS20InvalidAmount(0));
        // transfer the tokens to us (requires the allowance to be set)
        IEscrow escrow = _getOrCreateEscrow(msg_.sourceClient);
        _transferFrom(_msgSender(), address(escrow), msg_.denom, msg_.amount);
        escrow.recvCallback(msg_.denom, _msgSender(), msg_.amount);

        return _sendTransferFromEscrowWithSender(msg_, address(escrow), sender);
    }

    /// @notice Send a transfer after the funds have been transferred to escrow
    /// @param msg_ The message for sending a transfer
    /// @param escrow The address of the escrow contract
    /// @param sender The address of the sender, used to refund the tokens if the packet fails
    /// @return sequence The sequence number of the packet created
    // NOTE: We disable reentrancy for public functions.
    // slither-disable-next-line reentrancy-no-eth
    function _sendTransferFromEscrowWithSender(
        IICS20TransferMsgs.SendTransferMsg calldata msg_,
        address escrow,
        address sender
    )
        private
        returns (uint64)
    {
        string memory fullDenomPath = _getICS20TransferStorage()._ibcERC20Denoms[msg_.denom];
        if (bytes(fullDenomPath).length == 0) {
            // if the denom is not mapped, it is a native token
            fullDenomPath = Strings.toHexString(msg_.denom);
        } else {
            // we have a mapped denom, so we need to check if the token is returning to source
            bytes memory prefix = ICS20Lib.getDenomPrefix(ICS20Lib.DEFAULT_PORT_ID, msg_.sourceClient);
            // if the denom is prefixed by the port and channel on which we are sending
            // the token, then we must be returning the token back to the chain they originated from
            bool returningToSource = ICS20Lib.hasPrefix(bytes(fullDenomPath), prefix);
            if (returningToSource) {
                // token is returning to source, it is an IBCERC20 and we must burn the token (not keep it in escrow)
                IMintableAndBurnable(msg_.denom).burn(escrow, msg_.amount);
            }
        }

        IICS20TransferMsgs.FungibleTokenPacketData memory packetData = IICS20TransferMsgs.FungibleTokenPacketData({
            denom: fullDenomPath,
            sender: Strings.toHexString(sender),
            receiver: msg_.receiver,
            amount: msg_.amount,
            memo: msg_.memo
        });

        return _getICS26Router()
            .sendPacket(
                IICS26RouterMsgs.MsgSendPacket({
                    sourceClient: msg_.sourceClient,
                    timeoutTimestamp: msg_.timeoutTimestamp,
                    payload: IICS26RouterMsgs.Payload({
                        sourcePort: ICS20Lib.DEFAULT_PORT_ID,
                        destPort: ICS20Lib.DEFAULT_PORT_ID,
                        version: ICS20Lib.ICS20_VERSION,
                        encoding: ICS20Lib.ICS20_ENCODING,
                        value: abi.encode(packetData)
                    })
                })
            );
    }

    /// @inheritdoc IICS20TransferAccessControlled
    function setCustomERC20(string calldata denom, address token) external restricted {
        ICS20TransferStorage storage $ = _getICS20TransferStorage();
        require(address($._ibcERC20Contracts[denom]) == address(0), IICS20Errors.ICS20DenomAlreadyExists(denom));
        require(
            bytes($._ibcERC20Denoms[token]).length == 0, IICS20Errors.ICS20TokenAlreadyExists($._ibcERC20Denoms[token])
        );

        $._ibcERC20Contracts[denom] = IMintableAndBurnable(token);
        $._ibcERC20Denoms[token] = denom;
    }

    /// @inheritdoc IIBCApp
    // NOTE: Reentrancy disabled for this function via the `nonReentrant` modifier.
    // slither-disable-next-line reentrancy-no-eth
    function onRecvPacket(IIBCAppCallbacks.OnRecvPacketCallback calldata msg_)
        // solhint-disable-previous-line function-max-lines
        external
        onlyRouter
        nonReentrant
        whenNotPaused
        returns (bytes memory)
    {
        // Since this function mostly returns acks, also when it fails, the ics26router (the caller) will log the ack
        require(
            keccak256(bytes(msg_.payload.version)) == ICS20Lib.KECCAK256_ICS20_VERSION,
            ICS20UnexpectedVersion(ICS20Lib.ICS20_VERSION, msg_.payload.version)
        );
        require(
            keccak256(bytes(msg_.payload.sourcePort)) == ICS20Lib.KECCAK256_DEFAULT_PORT_ID,
            ICS20InvalidPort(ICS20Lib.DEFAULT_PORT_ID, msg_.payload.sourcePort)
        );
        require(
            keccak256(bytes(msg_.payload.encoding)) == ICS20Lib.KECCAK256_ICS20_ENCODING,
            ICS20UnexpectedEncoding(ICS20Lib.ICS20_ENCODING, msg_.payload.encoding)
        );
        require(
            keccak256(bytes(msg_.payload.destPort)) == ICS20Lib.KECCAK256_DEFAULT_PORT_ID,
            ICS20InvalidPort(ICS20Lib.DEFAULT_PORT_ID, msg_.payload.destPort)
        );

        IICS20TransferMsgs.FungibleTokenPacketData memory packetData =
            abi.decode(msg_.payload.value, (IICS20TransferMsgs.FungibleTokenPacketData));
        require(packetData.amount > 0, ICS20InvalidAmount(0));

        address receiver = ICS20Lib.mustHexStringToAddress(packetData.receiver);
        IEscrow escrow = _getOrCreateEscrow(msg_.destinationClient);
        bytes memory denomBz = bytes(packetData.denom);
        bytes memory prefix = ICS20Lib.getDenomPrefix(msg_.payload.sourcePort, msg_.sourceClient);

        // This is the prefix that would have been prefixed to the denomination
        // on sender chain IF and only if the token originally came from the
        // receiving chain.
        //
        // NOTE: We use SourcePort and SourceChannel here, because the counterparty
        // chain would have prefixed with DestPort and DestChannel when originally
        // receiving this token.
        bool returningToOrigin = ICS20Lib.hasPrefix(denomBz, prefix);
        address erc20Address;
        if (returningToOrigin) {
            // we are the origin source of this token:
            // Case 1: Forwarded IBCERC20
            // Case 2: Native ERC20

            // remove the first hop to unwind the trace
            bytes memory newDenom = Bytes.slice(denomBz, prefix.length);

            bool isAddress;
            (isAddress, erc20Address) = Strings.tryParseAddress(string(newDenom));
            if (!isAddress || erc20Address == address(0)) {
                // Case 1: Forwarded IBCERC20
                // we are the origin source and the token must be an IBCERC20 (since it is not a native token)
                erc20Address = address(_getICS20TransferStorage()._ibcERC20Contracts[string(newDenom)]);
                require(erc20Address != address(0), ICS20DenomNotFound(string(newDenom)));
            } // else: Case 2: "Native" ERC20
        } else {
            // we are not origin source, i.e. sender chain is the origin source: add denom trace and mint vouchers
            bytes memory newDenomPrefix = ICS20Lib.getDenomPrefix(msg_.payload.destPort, msg_.destinationClient);
            // NOTE: encodePacked is required by the ICS20 spec.
            // slither-disable-next-line encode-packed-collision
            bytes memory newDenom = abi.encodePacked(newDenomPrefix, denomBz);

            erc20Address = _getOrCreateIBCERC20(newDenom, address(escrow));
            IMintableAndBurnable(erc20Address).mint(address(escrow), packetData.amount);
        }

        // transfer the tokens to the receiver
        escrow.send(IERC20(erc20Address), receiver, packetData.amount);

        return ICS20Lib.SUCCESSFUL_ACKNOWLEDGEMENT_JSON;
    }

    /// @inheritdoc IIBCApp
    function onAcknowledgementPacket(IIBCAppCallbacks.OnAcknowledgementPacketCallback calldata msg_)
        external
        onlyRouter
        nonReentrant
        whenNotPaused
    {
        IICS20TransferMsgs.FungibleTokenPacketData memory packetData =
            abi.decode(msg_.payload.value, (IICS20TransferMsgs.FungibleTokenPacketData));

        if (keccak256(msg_.acknowledgement) == ICS24Host.KECCAK256_UNIVERSAL_ERROR_ACK) {
            // if the acknowledgement is an error, we must refund the tokens to the sender
            (, address sender) = _refundTokens(msg_.payload.sourcePort, msg_.sourceClient, packetData);
            IBCSenderCallbacksLib.ackPacketCallback(sender, false, msg_);
        } else {
            address sender = ICS20Lib.mustHexStringToAddress(packetData.sender);
            IBCSenderCallbacksLib.ackPacketCallback(sender, true, msg_);
        }
    }

    /// @inheritdoc IIBCApp
    function onTimeoutPacket(IIBCAppCallbacks.OnTimeoutPacketCallback calldata msg_)
        external
        onlyRouter
        nonReentrant
        whenNotPaused
    {
        IICS20TransferMsgs.FungibleTokenPacketData memory packetData =
            abi.decode(msg_.payload.value, (IICS20TransferMsgs.FungibleTokenPacketData));
        (, address sender) = _refundTokens(msg_.payload.sourcePort, msg_.sourceClient, packetData);
        IBCSenderCallbacksLib.timeoutPacketCallback(sender, msg_);
    }

    /// @notice Refund the tokens to the sender
    /// @param sourcePort The source port of the packet
    /// @param sourceClient The source client of the packet
    /// @param packetData The packet data
    /// @return The address of the erc20 contract that was refunded
    /// @return The address that received the refunded tokens, i.e. sender of the packet
    function _refundTokens(
        string calldata sourcePort,
        string calldata sourceClient,
        IICS20TransferMsgs.FungibleTokenPacketData memory packetData
    )
        private
        returns (address, address)
    {
        ICS20TransferStorage storage $ = _getICS20TransferStorage();
        IEscrow escrow = $._escrows[sourceClient];
        require(address(escrow) != address(0), IICS20Errors.ICS20EscrowNotFound(sourceClient));

        address refundee = ICS20Lib.mustHexStringToAddress(packetData.sender);
        address erc20Address;

        // if the denom is prefixed by the port and channel on which we are sending
        // the token, then we must be returning the token back to the chain they originated from
        bytes memory prefix = ICS20Lib.getDenomPrefix(sourcePort, sourceClient);
        bool isDestSource = ICS20Lib.hasPrefix(bytes(packetData.denom), prefix);
        if (isDestSource) {
            // receiving chain is source of the token, so we've received and mapped this token before
            erc20Address = address($._ibcERC20Contracts[packetData.denom]);
            require(erc20Address != address(0), ICS20DenomNotFound(packetData.denom));
            // if the token was returning to source, it was burned on send, so we mint it back now
            IMintableAndBurnable(erc20Address).mint(address(escrow), packetData.amount);
        } else {
            // the receiving chain is not the source of the token, so the token is either a native token
            // or we are a middle chain and the token was minted (and mapped) here.
            erc20Address = address($._ibcERC20Contracts[packetData.denom]);
            if (erc20Address == address(0)) {
                // the token is not mapped, so the token must be native
                erc20Address = ICS20Lib.mustHexStringToAddress(packetData.denom);
                require(erc20Address != address(0), ICS20DenomNotFound(packetData.denom));
            }
        }

        escrow.send(IERC20(erc20Address), refundee, packetData.amount);
        return (erc20Address, refundee);
    }

    /// @notice Transfer tokens from sender to receiver
    /// @param sender The sender of the tokens
    /// @param receiver The receiver of the tokens
    /// @param tokenContract The address of the token contract
    /// @param amount The amount of tokens to transfer
    function _transferFrom(address sender, address receiver, address tokenContract, uint256 amount) private {
        // we snapshot current balance of this token
        uint256 ourStartingBalance = IERC20(tokenContract).balanceOf(receiver);

        IERC20(tokenContract).safeTransferFrom(sender, receiver, amount);

        // check what this particular ERC20 implementation actually gave us, since it doesn't
        // have to be at all related to the _amount
        uint256 actualEndingBalance = IERC20(tokenContract).balanceOf(receiver);

        uint256 expectedEndingBalance = ourStartingBalance + amount;
        // a very strange ERC20 may trigger this condition, if we didn't have this we would
        // underflow, so it's mostly just an error message printer
        // NOTE: This is not a security check, but rather a sanity check.
        // slither-disable-next-line incorrect-equality
        require(
            actualEndingBalance > ourStartingBalance && actualEndingBalance == expectedEndingBalance,
            ICS20UnexpectedERC20Balance(expectedEndingBalance, actualEndingBalance)
        );
    }

    /// @notice Transfer tokens from sender to receiver using permit2
    /// @param sender The sender of the tokens
    /// @param receiver The receiver of the tokens
    /// @param tokenContract The address of the token contract
    /// @param amount The amount of tokens to transfer
    /// @param permit The permit data
    /// @param signature The signature of the permit data
    // NOTE: Balance reentrancy here is intentionally allowed, as we are checking the balance before and after
    // slither-disable-next-line reentrancy-balance
    function _transferFromWithPermit2(
        address sender,
        address receiver,
        address tokenContract,
        uint256 amount,
        ISignatureTransfer.PermitTransferFrom calldata permit,
        bytes calldata signature
    )
        private
    {
        // we snapshot current balance of this token
        uint256 ourStartingBalance = IERC20(tokenContract).balanceOf(receiver);

        _getPermit2()
            .permitTransferFrom(
                permit,
                ISignatureTransfer.SignatureTransferDetails({ to: receiver, requestedAmount: amount }),
                sender,
                signature
            );

        // check what this particular ERC20 implementation actually gave us, since it doesn't
        // have to be at all related to the _amount
        uint256 actualEndingBalance = IERC20(tokenContract).balanceOf(receiver);

        uint256 expectedEndingBalance = ourStartingBalance + amount;
        // a very strange ERC20 may trigger this condition, if we didn't have this we would
        // underflow, so it's mostly just an error message printer
        // NOTE: This is not a security check, but rather a sanity check.
        // slither-disable-next-line incorrect-equality
        require(
            actualEndingBalance > ourStartingBalance && actualEndingBalance == expectedEndingBalance,
            ICS20UnexpectedERC20Balance(expectedEndingBalance, actualEndingBalance)
        );
    }

    /// @notice Finds a contract in the foreign mapping, or creates a new IBCERC20 contract
    /// @param fullDenomPath The full path denom to find or create the contract for (which will be the name for the
    /// token)
    /// @param escrow The escrow contract address to use for the IBCERC20 contract
    /// @return The address of the erc20 contract
    function _getOrCreateIBCERC20(bytes memory fullDenomPath, address escrow) private returns (address) {
        ICS20TransferStorage storage $ = _getICS20TransferStorage();

        // check if denom already has a foreign registered contract
        address erc20Contract = address($._ibcERC20Contracts[string(fullDenomPath)]);
        if (erc20Contract == address(0)) {
            // nothing exists, so we create new erc20 contract and register it in the mapping
            BeaconProxy ibcERC20Proxy = new BeaconProxy(
                address($._ibcERC20Beacon),
                abi.encodeCall(IIBCERC20.initialize, (address(this), escrow, string(fullDenomPath)))
            );
            $._ibcERC20Contracts[string(fullDenomPath)] = IMintableAndBurnable(address(ibcERC20Proxy));
            $._ibcERC20Denoms[address(ibcERC20Proxy)] = string(fullDenomPath);
            erc20Contract = address(ibcERC20Proxy);

            emit IBCERC20ContractCreated(erc20Contract, string(fullDenomPath));
        }

        return erc20Contract;
    }

    /// @notice Returns the escrow contract for a client
    /// @param clientId The client ID
    /// @return The escrow contract address
    function _getOrCreateEscrow(string memory clientId) private returns (IEscrow) {
        ICS20TransferStorage storage $ = _getICS20TransferStorage();

        IEscrow escrow = $._escrows[clientId];
        if (address(escrow) == address(0)) {
            escrow = IEscrow(
                address(
                    new BeaconProxy(
                        address($._escrowBeacon), abi.encodeCall(IEscrow.initialize, (address(this), authority()))
                    )
                )
            );
            $._escrows[clientId] = escrow;
        }

        return escrow;
    }

    /// @notice Returns the storage of the ICS20Transfer contract
    /// @return $ The storage of the ICS20Transfer contract
    function _getICS20TransferStorage() private pure returns (ICS20TransferStorage storage $) {
        // solhint-disable-next-line no-inline-assembly
        assembly {
            $.slot := ICS20TRANSFER_STORAGE_SLOT
        }
    }

    /// @inheritdoc UUPSUpgradeable
    // solhint-disable-next-line no-empty-blocks
    function _authorizeUpgrade(address) internal override restricted { }

    /// @inheritdoc IICS20TransferAccessControlled
    function upgradeEscrowTo(address newEscrowLogic) external restricted {
        _getICS20TransferStorage()._escrowBeacon.upgradeTo(newEscrowLogic);
    }

    /// @inheritdoc IICS20TransferAccessControlled
    function upgradeIBCERC20To(address newIBCERC20Logic) external restricted {
        _getICS20TransferStorage()._ibcERC20Beacon.upgradeTo(newIBCERC20Logic);
    }

    /// @notice Returns the ICS26Router contract
    /// @return The ICS26Router contract address
    function _getICS26Router() private view returns (IICS26Router) {
        return _getICS20TransferStorage()._ics26;
    }

    /// @notice Returns the permit2 contract
    /// @return The permit2 contract address
    function _getPermit2() private view returns (ISignatureTransfer) {
        return _getICS20TransferStorage()._permit2;
    }

    /// @notice Modifier to check if the caller is the ICS26Router contract
    modifier onlyRouter() {
        require(_msgSender() == address(_getICS26Router()), ICS20Unauthorized(_msgSender()));
        _;
    }

    /// @notice Modifier to check if the initialization version matches the expected version
    /// @param version The expected current version of the contract
    modifier onlyVersion(uint256 version) {
        require(_getInitializedVersion() == version, InvalidInitialization());
        _;
    }
}
