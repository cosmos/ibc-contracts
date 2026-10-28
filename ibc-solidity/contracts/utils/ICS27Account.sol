// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { IICS27AccountMsgs } from "../msgs/IICS27AccountMsgs.sol";

import { IICS27Errors } from "../errors/IICS27Errors.sol";
import { IICS27Account } from "../interfaces/IICS27Account.sol";

import { ContextUpgradeable } from "@openzeppelin-upgradeable/utils/ContextUpgradeable.sol";
import { Address } from "@openzeppelin-contracts/utils/Address.sol";
import { Errors } from "@openzeppelin-contracts/utils/Errors.sol";

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
        // solhint-disable-next-line avoid-low-level-calls
        (bool success, bytes memory returndata) = recipient.call{ value: amount }("");
        _verifyCallResult(success, returndata, gasBefore);
    }

    /// @inheritdoc IICS27Account
    function functionCall(address target, bytes calldata data) external onlyICS27 returns (bytes memory) {
        return _call(target, data, 0);
    }

    /// @inheritdoc IICS27Account
    function execute(address target, bytes calldata data, uint256 value) external onlySelf returns (bytes memory) {
        return _call(target, data, value);
    }

    /// @inheritdoc IICS27Account
    function executeBatch(IICS27AccountMsgs.Call[] calldata calls) external onlySelf returns (bytes[] memory) {
        bytes[] memory results = new bytes[](calls.length);
        for (uint256 i = 0; i < calls.length; ++i) {
            IICS27AccountMsgs.Call calldata call = calls[i];
            results[i] = _call(call.target, call.data, call.value);
        }
        return results;
    }

    /// @inheritdoc IICS27Account
    function delegateExecute(address target, bytes calldata data) external onlySelf returns (bytes memory) {
        uint256 gasBefore = gasleft();
        // solhint-disable-next-line avoid-low-level-calls
        (bool success, bytes memory returndata) = target.delegatecall(data);
        return _verifyCallResultFromTarget(target, success, returndata, gasBefore);
    }

    /// @notice Calls `target` like openzeppelin's `Address.functionCallWithValue`, but reverts like
    /// `_verifyCallResult`
    /// @param target The target address to call
    /// @param data The data to send to the target address
    /// @param value The value to send to the target address
    /// @return The result of the call
    function _call(address target, bytes memory data, uint256 value) private returns (bytes memory) {
        _checkBalance(value);
        uint256 gasBefore = gasleft();
        // solhint-disable-next-line avoid-low-level-calls
        (bool success, bytes memory returndata) = target.call{ value: value }(data);
        return _verifyCallResultFromTarget(target, success, returndata, gasBefore);
    }

    /// @notice Like `_verifyCallResult`, but a successful call to an address without code is an error, like in
    /// openzeppelin's `Address.functionCall`
    /// @param target The address that was called
    /// @param success Whether the call succeeded
    /// @param returndata The data returned by the call
    /// @param gasBefore The gas left before the call
    /// @return The data returned by the call
    function _verifyCallResultFromTarget(
        address target,
        bool success,
        bytes memory returndata,
        uint256 gasBefore
    )
        private
        view
        returns (bytes memory)
    {
        _verifyCallResult(success, returndata, gasBefore);
        require(returndata.length != 0 || target.code.length != 0, Address.AddressEmptyCode(target));
        return returndata;
    }

    /// @notice Reverts like openzeppelin's `Address.verifyCallResult` if the call failed, but without data if it
    /// failed without revert data and most likely ran out of gas, see `OUT_OF_GAS_DIVISOR`
    /// @param success Whether the call succeeded
    /// @param returndata The data returned by the call
    /// @param gasBefore The gas left before the call
    function _verifyCallResult(bool success, bytes memory returndata, uint256 gasBefore) private view {
        if (!success && returndata.length == 0 && gasleft() < gasBefore / OUT_OF_GAS_DIVISOR) {
            // solhint-disable-next-line gas-custom-errors, reason-string
            revert();
        }
        Address.verifyCallResult(success, returndata);
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
