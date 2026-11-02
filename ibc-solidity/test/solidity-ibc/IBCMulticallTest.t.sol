// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable custom-errors,max-line-length,gas-small-strings

import { Test } from "forge-std/Test.sol";

import { IIBCMulticallMsgs } from "../../contracts/msgs/IIBCMulticallMsgs.sol";

import { IIBCMulticall } from "../../contracts/interfaces/IIBCMulticall.sol";
import { IIBCMulticallErrors } from "../../contracts/errors/IIBCMulticallErrors.sol";
import { IAccessManaged } from "@openzeppelin-contracts/access/manager/IAccessManaged.sol";

import { IBCMulticall } from "../../contracts/utils/IBCMulticall.sol";
import { IBCRolesLib } from "../../contracts/utils/IBCRolesLib.sol";
import { MulticallTargetMock } from "./mocks/MulticallTargetMock.sol";
import { DeployAccessManagerWithRoles } from "../../scripts/deployments/DeployAccessManagerWithRoles.sol";
import { AccessManager } from "@openzeppelin-contracts/access/manager/AccessManager.sol";
import { ERC1967Proxy } from "@openzeppelin-contracts/proxy/ERC1967/ERC1967Proxy.sol";
import { Address } from "@openzeppelin-contracts/utils/Address.sol";

contract IBCMulticallTest is Test, DeployAccessManagerWithRoles {
    AccessManager public accessManager;
    IBCMulticall public multicall;
    MulticallTargetMock public target;
    address public relayer = makeAddr("relayer");

    function setUp() public {
        accessManager = new AccessManager(address(this));
        multicall = IBCMulticall(
            address(
                new ERC1967Proxy(
                    address(new IBCMulticall()), abi.encodeCall(IBCMulticall.initialize, (address(accessManager)))
                )
            )
        );
        accessManagerSetMulticallRoles(accessManager, address(multicall), false);
        accessManager.grantRole(IBCRolesLib.RELAYER_ROLE, relayer, 0);

        target = new MulticallTargetMock(address(accessManager));
        bytes4[] memory selectors = new bytes4[](2);
        selectors[0] = MulticallTargetMock.set.selector;
        selectors[1] = MulticallTargetMock.setAndFail.selector;
        accessManager.setTargetFunctionRole(address(target), selectors, IBCRolesLib.RELAYER_ROLE);
    }

    function test_success_allowedFailureIsRolledBack() public {
        IIBCMulticallMsgs.Call[] memory calls = new IIBCMulticallMsgs.Call[](3);
        calls[0] = _call(abi.encodeCall(MulticallTargetMock.set, (1)), false);
        calls[1] = _call(abi.encodeCall(MulticallTargetMock.setAndFail, (2)), true);
        calls[2] = _call(abi.encodeCall(MulticallTargetMock.set, (3)), false);
        bytes memory failure = abi.encodeWithSelector(MulticallTargetMock.MockFailure.selector, 2);

        vm.expectEmit(address(multicall));
        emit IIBCMulticall.CallFailed(1, failure);
        vm.prank(relayer);
        IIBCMulticallMsgs.Result[] memory results = multicall.multicall(calls);

        assertTrue(results[0].success && results[2].success, "other calls should succeed");
        assertEq(results[0].returnData, abi.encode(1));
        assertFalse(results[1].success, "failing call should be reported");
        assertEq(results[1].returnData, failure);
        assertTrue(target.isSet(1) && target.isSet(3), "other calls should take effect");
        assertFalse(target.isSet(2), "failing call should be rolled back");
    }

    function test_failure_strictCallRevertsBatch() public {
        IIBCMulticallMsgs.Call[] memory calls = new IIBCMulticallMsgs.Call[](2);
        calls[0] = _call(abi.encodeCall(MulticallTargetMock.set, (1)), false);
        calls[1] = _call(abi.encodeCall(MulticallTargetMock.setAndFail, (2)), false);

        vm.expectRevert(abi.encodeWithSelector(MulticallTargetMock.MockFailure.selector, 2));
        vm.prank(relayer);
        multicall.multicall(calls);

        assertFalse(target.isSet(1), "earlier calls should be rolled back");
    }

    /// @dev A call allowed to fail that runs out of gas, or fails like the router does when an app runs out of gas,
    /// reverts the batch instead of starving the calls after it
    function test_failure_allowedCallOutOfGas() public {
        bytes[2] memory outOfGasCalls = [
            abi.encodeCall(MulticallTargetMock.burnGas, ()),
            abi.encodeCall(MulticallTargetMock.revertFailedCallback, ())
        ];
        for (uint256 j = 0; j < outOfGasCalls.length; ++j) {
            IIBCMulticallMsgs.Call[] memory calls = new IIBCMulticallMsgs.Call[](3);
            calls[0] = _call(abi.encodeCall(MulticallTargetMock.set, (1)), false);
            calls[1] = _call(outOfGasCalls[j], true);
            calls[2] = _call(abi.encodeCall(MulticallTargetMock.set, (3)), true);

            vm.expectRevert(abi.encodeWithSelector(IIBCMulticallErrors.IBCMulticallCallOutOfGas.selector, 1));
            vm.prank(relayer);
            multicall.multicall{ gas: 1_000_000 }(calls);
        }
    }

    function test_failure_targetWithoutCode() public {
        address noCode = makeAddr("noCode");
        bytes memory emptyCode = abi.encodeWithSelector(Address.AddressEmptyCode.selector, noCode);
        IIBCMulticallMsgs.Call[] memory calls = new IIBCMulticallMsgs.Call[](1);
        calls[0] = IIBCMulticallMsgs.Call({ target: noCode, allowFailure: true, callData: "" });

        vm.prank(relayer);
        IIBCMulticallMsgs.Result[] memory results = multicall.multicall(calls);
        assertFalse(results[0].success, "call without code should fail");
        assertEq(results[0].returnData, emptyCode);

        calls[0].allowFailure = false;
        vm.expectRevert(emptyCode);
        vm.prank(relayer);
        multicall.multicall(calls);
    }

    function test_failure_unauthorizedCaller() public {
        address user = makeAddr("user");

        vm.expectRevert(abi.encodeWithSelector(IAccessManaged.AccessManagedUnauthorized.selector, user));
        vm.prank(user);
        multicall.multicall(new IIBCMulticallMsgs.Call[](0));
    }

    /// @dev The multicall calls targets as itself, so it needs `RELAYER_ROLE` to reach restricted functions
    function test_failure_multicallWithoutRelayerRole() public {
        accessManager.revokeRole(IBCRolesLib.RELAYER_ROLE, address(multicall));
        IIBCMulticallMsgs.Call[] memory calls = new IIBCMulticallMsgs.Call[](1);
        calls[0] = _call(abi.encodeCall(MulticallTargetMock.set, (1)), false);

        vm.expectRevert(abi.encodeWithSelector(IAccessManaged.AccessManagedUnauthorized.selector, address(multicall)));
        vm.prank(relayer);
        multicall.multicall(calls);
    }

    function test_upgrade() public {
        address newLogic = address(new IBCMulticall());

        vm.expectRevert(abi.encodeWithSelector(IAccessManaged.AccessManagedUnauthorized.selector, relayer));
        vm.prank(relayer);
        multicall.upgradeToAndCall(newLogic, "");

        multicall.upgradeToAndCall(newLogic, "");
        assertEq(multicall.authority(), address(accessManager), "authority should be preserved");
    }

    function _call(bytes memory callData, bool allowFailure) private view returns (IIBCMulticallMsgs.Call memory) {
        return IIBCMulticallMsgs.Call({ target: address(target), allowFailure: allowFailure, callData: callData });
    }
}
