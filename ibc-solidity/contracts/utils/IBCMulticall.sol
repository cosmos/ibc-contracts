// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { IIBCMulticallMsgs } from "../msgs/IIBCMulticallMsgs.sol";

import { IIBCMulticallErrors } from "../errors/IIBCMulticallErrors.sol";
import { IICS26RouterErrors } from "../errors/IICS26RouterErrors.sol";
import { IIBCMulticall } from "../interfaces/IIBCMulticall.sol";

import { AccessManagedUpgradeable } from "@openzeppelin-upgradeable/access/manager/AccessManagedUpgradeable.sol";
import { UUPSUpgradeable } from "@openzeppelin-contracts/proxy/utils/UUPSUpgradeable.sol";
import { Address } from "@openzeppelin-contracts/utils/Address.sol";
import { LowLevelCall } from "@openzeppelin-contracts/utils/LowLevelCall.sol";

/// @title IBC Multicall
/// @notice Lets relayers batch packet relays where a failing relay does not revert the rest of the batch
/// @dev Map `multicall` to `RELAYER_ROLE` and grant this contract `RELAYER_ROLE`, so only relayers can call it and it
/// can relay to the router. See `DeployAccessManagerWithRoles.accessManagerSetMulticallRoles`.
/// @dev The router reports this contract as the relayer to IBC applications.
/// @dev Built for relaying to `ICS26Router`: a call that fails like the router does when it runs out of gas reverts the
/// batch even if it is allowed to fail, see `IBCMulticallCallOutOfGas`.
contract IBCMulticall is IIBCMulticallErrors, IIBCMulticall, AccessManagedUpgradeable, UUPSUpgradeable {
    // natlint-disable-next-line MissingNotice
    constructor() {
        _disableInitializers();
    }

    /// @inheritdoc IIBCMulticall
    function initialize(address authority) external initializer {
        __AccessManaged_init(authority);
    }

    /// @inheritdoc IIBCMulticall
    function multicall(IIBCMulticallMsgs.Call[] calldata calls)
        external
        restricted
        returns (IIBCMulticallMsgs.Result[] memory results)
    {
        results = new IIBCMulticallMsgs.Result[](calls.length);
        for (uint256 i = 0; i < calls.length; ++i) {
            IIBCMulticallMsgs.Call calldata calli = calls[i];
            // solhint-disable-next-line avoid-low-level-calls
            (bool success, bytes memory returnData) = calli.target.call(calli.callData);
            // A call to an address without code succeeds without doing anything, so it is treated as a failure
            if (success && returnData.length == 0 && calli.target.code.length == 0) {
                success = false;
                returnData = abi.encodeWithSelector(Address.AddressEmptyCode.selector, calli.target);
            }
            if (!success) {
                if (!calli.allowFailure) {
                    LowLevelCall.bubbleRevert(returnData);
                }
                // A call that ran out of gas leaves too little gas for the calls after it, so reporting it would let a
                // batch that is short on gas land without them, and gas estimation would settle on such a limit
                require(!_isOutOfGas(returnData), IBCMulticallCallOutOfGas(i));
                emit CallFailed(i, returnData);
            }
            results[i] = IIBCMulticallMsgs.Result({ success: success, returnData: returnData });
        }
    }

    /// @notice Returns whether a call to the router failed the way it does when it runs out of gas
    /// @dev Running out of gas leaves empty revert data, which the router turns into `IBCFailedCallback` when it
    /// happens in an app callback. Both can also come from a revert without a reason, which the router treats as fatal
    /// too.
    /// @param returnData The revert data of the call
    /// @return Whether the call most likely ran out of gas
    function _isOutOfGas(bytes memory returnData) private pure returns (bool) {
        return returnData.length == 0
            || keccak256(returnData) == keccak256(abi.encodeWithSelector(IICS26RouterErrors.IBCFailedCallback.selector));
    }

    /// @inheritdoc UUPSUpgradeable
    function _authorizeUpgrade(address) internal override restricted { }
    // solhint-disable-previous-line no-empty-blocks
}
