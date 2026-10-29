// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { IICS27AccountMsgs } from "../msgs/IICS27AccountMsgs.sol";

import { IICS27Errors } from "../errors/IICS27Errors.sol";
import { IICS27Account } from "../interfaces/IICS27Account.sol";

import { ContextUpgradeable } from "@openzeppelin-upgradeable/utils/ContextUpgradeable.sol";
import { Address } from "@openzeppelin-contracts/utils/Address.sol";
import { Errors } from "@openzeppelin-contracts/utils/Errors.sol";
import { LowLevelCall } from "@openzeppelin-contracts/utils/LowLevelCall.sol";

/// @title ICS27Account
/// @notice This contract is the implementation of the ICS27 Account, which is used to make the GMP calls
/// @dev `ICS26Router.recvPacket` fails the relay instead of writing an error acknowledgement when the app reverts
/// without data, which is how running out of gas reverts. Like openzeppelin's `Address`, calls bubble up revert data
/// and replace empty revert data with `FailedCall`, but not when the call most likely ran out of gas: then they revert
/// without data, so the router fails the relay.
contract ICS27Account is IICS27Errors, IICS27Account, ContextUpgradeable {
    /// @notice Storage of the ICS27Account contract
    /// @dev It's implemented on a custom ERC-7201 namespace to reduce the risk of storage collisions when using with
    /// upgradeable contracts.
    /// @param _ics27 The ICS27GMP contract address. Immutable.
    /// @custom:storage-location erc7201:ibc.storage.ICS27Account
    struct ICS27AccountStorage {
        address _ics27;
    }

    /// @notice ERC-7201 slot for the ICS27Account storage
    /// @dev keccak256(abi.encode(uint256(keccak256("ibc.storage.ICS27Account")) - 1)) & ~bytes32(uint256(0xff))
    bytes32 private constant ICS27ACCOUNT_STORAGE_SLOT =
        0x319583b012a10c350515da7d8fdefe3c302490627bf79c0be5b739020ce32c00;

    /// @notice A call that fails without revert data and leaves less than 1/OUT_OF_GAS_DIVISOR of the gas it started
    /// with is treated as running out of gas
    /// @dev Each frame below the target keeps 1/64 of its gas when a call runs out of gas, so this covers running out
    /// of gas up to about 8 frames below the target
    uint256 private constant OUT_OF_GAS_DIVISOR = 8;

    /// @notice Gas left for the cost of a `CALL` on top of the gas it forwards
    uint256 private constant CALL_GAS_OVERHEAD = 40_000;

    /// @dev This contract is meant to be deployed by a proxy, so the constructor is not used
    // natlint-disable-next-line MissingNotice
    constructor() {
        _disableInitializers();
    }

    /// @inheritdoc IICS27Account
    function initialize(address ics27_) external initializer {
        __Context_init();

        ICS27AccountStorage storage $ = _getICS27AccountStorage();
        $._ics27 = ics27_;
    }

    /// @inheritdoc IICS27Account
    function ics27() external view returns (address) {
        return _getICS27AccountStorage()._ics27;
    }

    /// @inheritdoc IICS27Account
    function sendValue(address payable recipient, uint256 amount) external onlySelf {
        _checkBalance(amount);
        uint256 gasBefore = gasleft();
        if (!LowLevelCall.callNoReturn(recipient, amount, "")) {
            _revert(gasBefore);
        }
    }

    /// @inheritdoc IICS27Account
    function functionCall(address target, bytes calldata data) external onlyICS27 returns (bytes memory) {
        return _call(target, data, 0, gasleft());
    }

    /// @inheritdoc IICS27Account
    function execute(address target, bytes calldata data, uint256 value) external onlySelf returns (bytes memory) {
        return _call(target, data, value, gasleft());
    }

    /// @inheritdoc IICS27Account
    function executeWithGas(
        address target,
        bytes calldata data,
        uint256 value,
        uint256 gas
    )
        external
        onlySelf
        returns (bytes memory)
    {
        return _callWithGas(target, data, value, gas);
    }

    /// @inheritdoc IICS27Account
    function executeBatch(IICS27AccountMsgs.Call[] calldata calls) external onlySelf returns (bytes[] memory) {
        bytes[] memory results = new bytes[](calls.length);
        for (uint256 i = 0; i < calls.length; ++i) {
            IICS27AccountMsgs.Call calldata call = calls[i];
            results[i] = _call(call.target, call.data, call.value, gasleft());
        }
        return results;
    }

    /// @inheritdoc IICS27Account
    function executeBatchWithGas(IICS27AccountMsgs.CallWithGas[] calldata calls)
        external
        onlySelf
        returns (bytes[] memory)
    {
        bytes[] memory results = new bytes[](calls.length);
        for (uint256 i = 0; i < calls.length; ++i) {
            IICS27AccountMsgs.CallWithGas calldata call = calls[i];
            results[i] = _callWithGas(call.target, call.data, call.value, call.gas);
        }
        return results;
    }

    /// @notice Calls `target` like `_call`, but runs out of gas if there is not enough gas left to forward `gas`
    /// @dev Runs out of gas instead of reverting, so that the account calling itself sees it as running out of gas too
    /// @param target The target address to call
    /// @param data The data to send to the target address
    /// @param value The value to send to the target address
    /// @param gas The gas to call the target address with
    /// @return The result of the call
    function _callWithGas(address target, bytes memory data, uint256 value, uint256 gas)
        private
        returns (bytes memory)
    {
        // A call gets at most 63/64 of the gas left after its own cost, and running out of its gas must leave more
        // than 1/OUT_OF_GAS_DIVISOR of it, or the call would be treated as the relayer not providing enough gas
        if (gasleft() < gas + gas / 63 + gas / OUT_OF_GAS_DIVISOR + CALL_GAS_OVERHEAD) {
            // solhint-disable-next-line no-inline-assembly
            assembly {
                invalid()
            }
        }
        return _call(target, data, value, gas);
    }

    /// @notice Calls `target` like openzeppelin's `Address.functionCallWithValue`, but reverts like `_revert`
    /// @param target The target address to call
    /// @param data The data to send to the target address
    /// @param value The value to send to the target address
    /// @param gas_ The gas to call the target address with
    /// @return The result of the call
    function _call(address target, bytes memory data, uint256 value, uint256 gas_) private returns (bytes memory) {
        _checkBalance(value);
        bool success;
        // Like `LowLevelCall.callNoReturn`, which cannot set the gas, so the return data is only copied by `_result`
        // solhint-disable-next-line no-inline-assembly
        assembly ("memory-safe") {
            success := call(gas_, target, value, add(data, 0x20), mload(data), 0x00, 0x00)
        }
        return _result(target, success, gas_);
    }

    /// @notice Returns the result of the last call, reverting like `_revert` if it failed
    /// @dev Like openzeppelin's `Address`, a successful call to an address without code is an error
    /// @param target The address that was called
    /// @param success Whether the call succeeded
    /// @param gas The gas the call was made with
    /// @return The result of the call
    function _result(address target, bool success, uint256 gas) private view returns (bytes memory) {
        if (!success) {
            _revert(gas);
        }
        require(LowLevelCall.returnDataSize() != 0 || target.code.length != 0, Address.AddressEmptyCode(target));
        return LowLevelCall.returnData();
    }

    /// @notice Bubbles up the revert data of the last call, or reverts with `FailedCall` if it has none
    /// @dev Reverts without data instead if the call most likely ran out of gas, see `OUT_OF_GAS_DIVISOR`
    /// @param gas The gas the call was made with
    function _revert(uint256 gas) private view {
        if (LowLevelCall.returnDataSize() != 0) {
            LowLevelCall.bubbleRevert();
        }
        if (gasleft() < gas / OUT_OF_GAS_DIVISOR) {
            // Reverts without data, like running out of gas, so the router fails the relay
            // solhint-disable-next-line gas-custom-errors, reason-string
            revert();
        }
        revert Errors.FailedCall();
    }

    /// @notice Reverts if this account cannot send `value`
    /// @param value The value to send
    function _checkBalance(uint256 value) private view {
        require(address(this).balance >= value, Errors.InsufficientBalance(address(this).balance, value));
    }

    /// @notice Returns the storage of the ICS27Account contract
    /// @return $ The storage of the ICS27Account contract
    function _getICS27AccountStorage() private pure returns (ICS27AccountStorage storage $) {
        // solhint-disable-next-line no-inline-assembly
        assembly {
            $.slot := ICS27ACCOUNT_STORAGE_SLOT
        }
    }

    /// @notice Modifier to restrict access to the ICS27 contract
    modifier onlyICS27() {
        address ics27_ = _getICS27AccountStorage()._ics27;
        require(_msgSender() == ics27_, ICS27Unauthorized(ics27_, _msgSender()));
        _;
    }

    /// @notice Modifier to restrict access to this contract
    modifier onlySelf() {
        require(_msgSender() == address(this), ICS27Unauthorized(address(this), _msgSender()));
        _;
    }
}
