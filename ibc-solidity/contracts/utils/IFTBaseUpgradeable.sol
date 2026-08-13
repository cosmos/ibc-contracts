// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { IIFTMsgs } from "../msgs/IIFTMsgs.sol";
import { IIBCAppCallbacks } from "../msgs/IIBCAppCallbacks.sol";
import { IICS27GMPMsgs } from "../msgs/IICS27GMPMsgs.sol";

import { IIFT } from "../interfaces/IIFT.sol";
import { IICS27GMP } from "../interfaces/IICS27GMP.sol";
import { IIFTSendCallConstructor } from "../interfaces/IIFTSendCallConstructor.sol";
import { IIBCSenderCallbacks } from "../interfaces/IIBCSenderCallbacks.sol";
import { IIFTErrors } from "../errors/IIFTErrors.sol";

import { ReentrancyGuardTransient } from "@openzeppelin-contracts/utils/ReentrancyGuardTransient.sol";
import { ERC20Upgradeable } from "@openzeppelin-upgradeable/token/ERC20/ERC20Upgradeable.sol";
import { IBCCallbackReceiver } from "../utils/IBCCallbackReceiver.sol";
import { ERC165Checker } from "@openzeppelin-contracts/utils/introspection/ERC165Checker.sol";

/**
 * @title IFT Base Upgradeable Contract
 * @notice Abstract base contract for Interchain Fungible Tokens
 *
 * @dev Extend this contract and implement the ERC20 constructor to create an IFT token
 * @dev _update in ERC20Upgradeable can be overriden to add custom logic on minting and burning such as rate limiting,
 * and whitelisting.
 */
abstract contract IFTBaseUpgradeable is
    IIFTErrors,
    IIFT,
    ERC20Upgradeable,
    IBCCallbackReceiver,
    ReentrancyGuardTransient
{
    /// @notice Storage for IFT-specific state
    /// @param _ics27Gmp The ICS27-GMP contract for sending cross-chain messages
    /// @param _iftBridges Mapping of client IDs to their bridge configurations
    /// @param _pendingTransfers Mapping of (clientId, sequence) to pending transfer info
    struct IFTBaseStorage {
        IICS27GMP _ics27Gmp;
        mapping(string clientId => IIFTMsgs.IFTBridge bridge) _iftBridges;
        mapping(string clientId => mapping(uint64 seq => IIFTMsgs.PendingTransfer info)) _pendingTransfers;
    }

    /// @notice ERC-7201 slot for the IFT storage
    /// @dev keccak256(abi.encode(uint256(keccak256("ibc.storage.IFT")) - 1)) & ~bytes32(uint256(0xff))
    bytes32 private constant IFT_STORAGE_SLOT = 0x35d0029e62ce5824ad5e38215107659b8aa50b0046e8bc44a0f4a32b87d61a00;

    /// @notice Default timeout duration for IFT transfers (15 minutes)
    uint64 private constant DEFAULT_TIMEOUT_DURATION = 15 minutes;

    /// @notice Initializer for IFTBase contract
    /// @param erc20Name The name of the ERC20 token
    /// @param erc20Symbol The symbol of the ERC20 token
    /// @param ics27Gmp The address of the ICS27-GMP contract
    function __IFTBase_init(
        string memory erc20Name,
        string memory erc20Symbol,
        address ics27Gmp
    )
        internal
        onlyInitializing
    {
        __ERC20_init(erc20Name, erc20Symbol);

        __IFTBase_init_unchained(ics27Gmp);
    }

    /// @notice Unchained initializer for IFTBase
    /// @param ics27Gmp The address of the ICS27-GMP contract
    function __IFTBase_init_unchained(address ics27Gmp) internal onlyInitializing {
        IFTBaseStorage storage $ = _getIFTBaseStorage();
        $._ics27Gmp = IICS27GMP(ics27Gmp);
    }

    /// @inheritdoc IIFT
    function registerIFTBridge(
        string calldata clientId,
        string calldata counterpartyIFTAddress,
        address iftSendCallConstructor
    )
        external
    {
        _onlyAuthority();
        require(bytes(clientId).length > 0, IFTEmptyClientId());
        require(bytes(counterpartyIFTAddress).length > 0, IFTEmptyCounterpartyAddress());
        require(iftSendCallConstructor != address(0), IFTZeroAddressConstructor());
        require(
            ERC165Checker.supportsInterface(iftSendCallConstructor, type(IIFTSendCallConstructor).interfaceId),
            IFTInvalidConstructorInterface(iftSendCallConstructor)
        );

        IFTBaseStorage storage $ = _getIFTBaseStorage();
        $._iftBridges[clientId] = IIFTMsgs.IFTBridge({
            clientId: clientId,
            counterpartyIFTAddress: counterpartyIFTAddress,
            iftSendCallConstructor: IIFTSendCallConstructor(iftSendCallConstructor)
        });

        emit IFTBridgeRegistered(clientId, counterpartyIFTAddress, iftSendCallConstructor);
    }

    /// @inheritdoc IIFT
    function removeIFTBridge(string calldata clientId) external {
        _onlyAuthority();
        IFTBaseStorage storage $ = _getIFTBaseStorage();

        IIFTMsgs.IFTBridge memory bridge = $._iftBridges[clientId];
        require(bytes(bridge.clientId).length > 0, IFTBridgeNotFound(clientId));
        delete $._iftBridges[clientId];

        emit IFTBridgeRemoved(clientId);
    }

    /// @inheritdoc IIFT
    function iftTransfer(
        string calldata clientId,
        string calldata receiver,
        uint256 amount,
        uint64 timeoutTimestamp
    )
        external
        nonReentrant
    {
        _iftTransfer(_msgSender(), clientId, receiver, amount, timeoutTimestamp);
    }

    /// @inheritdoc IIFT
    function iftTransfer(string calldata clientId, string calldata receiver, uint256 amount) external nonReentrant {
        uint64 timeoutTimestamp = uint64(block.timestamp) + DEFAULT_TIMEOUT_DURATION;
        _iftTransfer(_msgSender(), clientId, receiver, amount, timeoutTimestamp);
    }

    /// @notice Internal implementation of iftTransfer
    /// @param sender The address initiating the transfer
    /// @param clientId The IBC client identifier
    /// @param receiver The receiver address on the counterparty chain
    /// @param amount The amount of tokens to transfer
    /// @param timeoutTimestamp The timeout timestamp for the IBC packet
    function _iftTransfer(
        address sender,
        string calldata clientId,
        string calldata receiver,
        uint256 amount,
        uint64 timeoutTimestamp
    )
        internal
    {
        require(bytes(clientId).length > 0, IFTEmptyClientId());
        require(bytes(receiver).length > 0, IFTEmptyReceiver());
        require(amount > 0, IFTZeroAmount());
        require(timeoutTimestamp > block.timestamp, IFTTimeoutInPast(timeoutTimestamp, uint64(block.timestamp)));

        _burn(sender, amount); // Implemented in the ERC20 base contract

        IFTBaseStorage storage $ = _getIFTBaseStorage();
        IIFTMsgs.IFTBridge memory bridge = $._iftBridges[clientId];
        require(keccak256(bytes(bridge.clientId)) == keccak256(bytes(clientId)), IFTBridgeNotFound(clientId));

        bytes memory payload = bridge.iftSendCallConstructor.constructMintCall(receiver, amount);
        uint64 seq = $._ics27Gmp
            .sendCall(
                IICS27GMPMsgs.SendCallMsg({
                    sourceClient: bridge.clientId,
                    receiver: bridge.counterpartyIFTAddress,
                    salt: "",
                    payload: payload,
                    timeoutTimestamp: timeoutTimestamp,
                    memo: ""
                })
            );

        $._pendingTransfers[bridge.clientId][seq] = IIFTMsgs.PendingTransfer({ sender: sender, amount: amount });

        emit IFTTransferInitiated(bridge.clientId, seq, sender, receiver, amount);
    }

    /// @inheritdoc IIFT
    function iftMint(address receiver, uint256 amount) external nonReentrant {
        IFTBaseStorage storage $ = _getIFTBaseStorage();

        IICS27GMPMsgs.AccountIdentifier memory accountId = $._ics27Gmp.getAccountIdentifier(_msgSender());
        IIFTMsgs.IFTBridge memory bridge = $._iftBridges[accountId.clientId];

        require(bytes(bridge.clientId).length > 0, IFTBridgeNotFound(accountId.clientId));
        require(
            keccak256(bytes(bridge.clientId)) == keccak256(bytes(accountId.clientId)),
            IFTBridgeNotFound(accountId.clientId)
        );
        require(
            keccak256(bytes(bridge.counterpartyIFTAddress)) == keccak256(bytes(accountId.sender)),
            IFTUnauthorizedMint(bridge.counterpartyIFTAddress, accountId.sender)
        );
        require(accountId.salt.length == 0, IFTUnexpectedSalt(accountId.salt));

        _mint(receiver, amount); // Implemented in the ERC20 base contract

        emit IFTMintReceived(accountId.clientId, receiver, amount);
    }

    /// @inheritdoc IIFT
    function getIFTBridge(string calldata clientId) external view returns (IIFTMsgs.IFTBridge memory) {
        IIFTMsgs.IFTBridge memory bridge = _getIFTBaseStorage()._iftBridges[clientId];
        require(bytes(bridge.clientId).length > 0, IFTBridgeNotFound(clientId));
        return bridge;
    }

    /// @inheritdoc IIFT
    function getPendingTransfer(
        string calldata clientId,
        uint64 sequence
    )
        external
        view
        returns (IIFTMsgs.PendingTransfer memory)
    {
        IIFTMsgs.PendingTransfer memory pending = _getIFTBaseStorage()._pendingTransfers[clientId][sequence];
        require(pending.amount > 0, IFTPendingTransferNotFound(clientId, sequence));
        return pending;
    }

    /// @inheritdoc IIFT
    function ics27() external view returns (IICS27GMP) {
        return _getIFTBaseStorage()._ics27Gmp;
    }

    /// @inheritdoc IIBCSenderCallbacks
    function onAckPacket(
        bool success,
        IIBCAppCallbacks.OnAcknowledgementPacketCallback calldata msg_
    )
        external
        override(IIBCSenderCallbacks)
        nonReentrant
    {
        _onlyICS27GMP(); // Ensure only ICS27-GMP can call this function

        if (!success) {
            _refundPendingTransfer(msg_.sourceClient, msg_.sequence);
            return;
        }

        IFTBaseStorage storage $ = _getIFTBaseStorage();
        IIFTMsgs.PendingTransfer memory pending = $._pendingTransfers[msg_.sourceClient][msg_.sequence];

        require(pending.amount > 0, IFTPendingTransferNotFound(msg_.sourceClient, msg_.sequence));

        delete $._pendingTransfers[msg_.sourceClient][msg_.sequence];

        emit IFTTransferCompleted(msg_.sourceClient, msg_.sequence, pending.sender, pending.amount);
    }

    /// @inheritdoc IIBCSenderCallbacks
    function onTimeoutPacket(IIBCAppCallbacks.OnTimeoutPacketCallback calldata msg_)
        external
        override(IIBCSenderCallbacks)
        nonReentrant
    {
        _onlyICS27GMP(); // Ensure only ICS27-GMP can call this function

        _refundPendingTransfer(msg_.sourceClient, msg_.sequence);
    }

    /// @notice Refunds a pending transfer back to the sender
    /// @param clientId The IBC client identifier
    /// @param sequence The packet sequence number
    function _refundPendingTransfer(string memory clientId, uint64 sequence) internal {
        IFTBaseStorage storage $ = _getIFTBaseStorage();
        IIFTMsgs.PendingTransfer memory pending = $._pendingTransfers[clientId][sequence];

        require(pending.amount > 0, IFTPendingTransferNotFound(clientId, sequence));

        _mint(pending.sender, pending.amount); // Implemented in the ERC20 base contract
        delete $._pendingTransfers[clientId][sequence];

        emit IFTTransferRefunded(clientId, sequence, pending.sender, pending.amount);
    }

    /// @notice Ensures the caller is the ICS27-GMP contract
    function _onlyICS27GMP() internal view {
        IFTBaseStorage storage $ = _getIFTBaseStorage();
        require(_msgSender() == address($._ics27Gmp), IFTOnlyICS27GMP(_msgSender()));
    }

    /// @notice Ensures the caller is the authority
    /// @dev Must be implemented by the inheriting contract
    function _onlyAuthority() internal virtual;

    /// @notice Returns the IFT storage
    /// @return $ The IFT storage struct
    function _getIFTBaseStorage() internal pure returns (IFTBaseStorage storage $) {
        // solhint-disable-next-line no-inline-assembly
        assembly {
            $.slot := IFT_STORAGE_SLOT
        }
    }
}
