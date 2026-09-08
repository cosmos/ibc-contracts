// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable custom-errors,max-line-length,no-inline-assembly,gas-small-strings

import { Test } from "forge-std/Test.sol";

import { IICS27AccountMsgs } from "../../contracts/msgs/IICS27AccountMsgs.sol";
import { IICS27Errors } from "../../contracts/errors/IICS27Errors.sol";

import { ERC1967Proxy } from "@openzeppelin-contracts/proxy/ERC1967/ERC1967Proxy.sol";
import { ICS27Account } from "../../contracts/utils/ICS27Account.sol";
import { Errors } from "@openzeppelin-contracts/utils/Errors.sol";

contract ICS27AccountTest is Test {
    address public ics27 = makeAddr("ics27");
    ICS27Account public ics27Account;

    function setUp() public {
        address accountLogic = address(new ICS27Account());

        ERC1967Proxy accountProxy =
            new ERC1967Proxy(address(accountLogic), abi.encodeCall(ICS27Account.initialize, (ics27)));

        ics27Account = ICS27Account(address(accountProxy));
        assertEq(ics27Account.ics27(), ics27, "ICS27 address should match");
    }

    function test_success_functionCall() public {
        address target = makeAddr("target");
        bytes memory data = "someData";
        bytes memory resp = "mockedResponse";

        vm.mockCall(target, data, resp);
        vm.expectCall(target, data);
        vm.prank(ics27);
        ics27Account.functionCall(target, data);
    }

    function test_failure_functionCall() public {
        address target = makeAddr("target");
        bytes memory data = "someData";
        bytes memory resp = "mockedResponse";

        // Unauthorized call
        address unauthorized = makeAddr("unauthorized");
        vm.expectRevert(abi.encodeWithSelector(IICS27Errors.ICS27Unauthorized.selector, ics27, unauthorized));
        vm.prank(unauthorized);
        ics27Account.functionCall(target, data);

        // Call reverts
        vm.mockCallRevert(target, data, resp);
        vm.expectRevert(resp);
        vm.prank(ics27);
        ics27Account.functionCall(target, data);
    }

    function testFuzz_success_sendValue(uint256 amount) public {
        vm.deal(address(ics27Account), amount);
        address payable recipient = payable(makeAddr("recipient"));

        // Assert initial balances
        assertEq(address(ics27Account).balance, amount, "ICS27Account balance should match amount");
        assertEq(recipient.balance, 0, "Recipient balance should be zero");

        // Send value
        vm.prank(address(ics27Account));
        ics27Account.sendValue(recipient, amount);

        // Assert final balances
        assertEq(address(ics27Account).balance, 0, "ICS27Account balance should be zero after send");
        assertEq(recipient.balance, amount, "Recipient balance should match amount after send");
    }

    function testFuzz_failure_sendValue(uint256 amount) public {
        vm.assume(amount < type(uint256).max); // Avoid overflow in test

        vm.deal(address(ics27Account), amount);
        address payable recipient = payable(makeAddr("recipient"));

        // Unauthorized call
        address unauthorized = makeAddr("unauthorized");
        vm.expectRevert(
            abi.encodeWithSelector(IICS27Errors.ICS27Unauthorized.selector, address(ics27Account), unauthorized)
        );
        vm.prank(address(unauthorized));
        ics27Account.sendValue(recipient, amount);

        // Insufficient balance
        vm.prank(address(ics27Account));
        vm.expectRevert(abi.encodeWithSelector(Errors.InsufficientBalance.selector, amount, amount + 1));
        ics27Account.sendValue(recipient, amount + 1);

        // Assert initial balances
        assertEq(address(ics27Account).balance, amount, "ICS27Account balance should match amount");
        assertEq(recipient.balance, 0, "Recipient balance should be zero");
    }

    function testFuzz_success_execute(uint256 value) public {
        vm.deal(address(ics27Account), value);

        address target = address(new TestCallContract());
        bytes memory data = abi.encodeCall(TestCallContract.payableCall, ());
        bytes memory resp = abi.encode("mockedResponse");

        // Assert initial balances
        assertEq(address(ics27Account).balance, value, "ICS27Account balance should match amount");
        assertEq(target.balance, 0, "Target balance should be zero");

        vm.expectCall(target, value, data);
        vm.prank(address(ics27Account));
        bytes memory result = ics27Account.execute(target, data, value);
        assertEq(result, resp, "Result should match mocked response");

        // Assert final balance
        assertEq(address(ics27Account).balance, 0, "ICS27Account balance should be zero after execute");
        assertEq(target.balance, value, "Target balance should match value after execute");
    }

    function testFuzz_failure_execute(uint256 value) public {
        vm.assume(value < type(uint256).max); // Avoid overflow in test

        vm.deal(address(ics27Account), value);

        address target = address(new TestCallContract());
        bytes memory data = abi.encodeCall(TestCallContract.payableCall, ());
        bytes memory resp = abi.encode("mockedResponse");

        // Unauthorized call
        address unauthorized = makeAddr("unauthorized");
        vm.expectRevert(
            abi.encodeWithSelector(IICS27Errors.ICS27Unauthorized.selector, address(ics27Account), unauthorized)
        );
        vm.prank(unauthorized);
        ics27Account.execute(target, data, value);

        // Call reverts
        vm.mockCallRevert(target, data, resp);
        vm.expectRevert(resp);
        vm.prank(address(ics27Account));
        ics27Account.execute(target, data, value);

        // Insufficient balance
        vm.prank(address(ics27Account));
        vm.expectRevert(abi.encodeWithSelector(Errors.InsufficientBalance.selector, value, value + 1));
        ics27Account.execute(target, data, value + 1);

        // Assert initial balances
        assertEq(address(ics27Account).balance, value, "ICS27Account balance should match amount");
        assertEq(target.balance, 0, "Target balance should still be zero");
    }

    function testFuzz_success_executeBatch(uint256 totalValue, uint8 numCalls) public {
        vm.assume(numCalls > 0);

        vm.deal(address(ics27Account), totalValue);

        address target = address(new TestCallContract());
        bytes memory data = abi.encodeCall(TestCallContract.payableCall, ());
        bytes[] memory expResp = new bytes[](numCalls);

        // Assert initial balances
        assertEq(address(ics27Account).balance, totalValue, "ICS27Account balance should match totalValue");
        assertEq(target.balance, 0, "Target balance should be zero");

        // Prepare calls
        IICS27AccountMsgs.Call[] memory calls = new IICS27AccountMsgs.Call[](numCalls);
        uint256 valuePerCall = totalValue / numCalls;
        for (uint256 i = 0; i < numCalls; ++i) {
            calls[i] = IICS27AccountMsgs.Call({ target: target, data: data, value: valuePerCall });

            expResp[i] = abi.encode("mockedResponse");
        }

        vm.expectCall(target, valuePerCall, data, numCalls);
        vm.prank(address(ics27Account));

        bytes[] memory results = ics27Account.executeBatch(calls);
        assertEq(results.length, numCalls, "Results length should match number of calls");
        assertEq(results, expResp, "Results should match expected responses");

        // Assert final balance
        assertEq(
            address(ics27Account).balance,
            totalValue % numCalls,
            "ICS27Account balance should be zero after executeBatch"
        );
        assertEq(target.balance, valuePerCall * numCalls, "Target balance should match totalValue after executeBatch");
    }

    function testFuzz_failure_executeBatch(uint256 totalValue, uint8 numCalls) public {
        vm.assume(numCalls > 0 && totalValue < type(uint256).max); // Avoid overflow in test

        vm.deal(address(ics27Account), totalValue);

        address target = address(new TestCallContract());
        bytes memory data = abi.encodeCall(TestCallContract.payableCall, ());

        IICS27AccountMsgs.Call[] memory calls = new IICS27AccountMsgs.Call[](numCalls);
        for (uint256 i = 0; i < numCalls; ++i) {
            calls[i] = IICS27AccountMsgs.Call({ target: target, data: data, value: totalValue / numCalls });
        }

        // Unauthorized call
        address unauthorized = makeAddr("unauthorized");
        vm.expectRevert(
            abi.encodeWithSelector(IICS27Errors.ICS27Unauthorized.selector, address(ics27Account), unauthorized)
        );
        vm.prank(unauthorized);
        ics27Account.executeBatch(calls);

        // Call reverts
        bytes memory revertData = "revertData";
        bytes memory revertResp = "revertResponse";
        calls[numCalls - 1].data = revertData;

        vm.mockCallRevert(target, revertData, revertResp);
        vm.expectRevert(revertResp);
        vm.prank(address(ics27Account));
        ics27Account.executeBatch(calls);
        calls[numCalls - 1].data = data; // Reset to original data

        // Insufficient balance
        calls[0].value = totalValue + 1; // Set first call to exceed balance
        vm.prank(address(ics27Account));
        vm.expectRevert(abi.encodeWithSelector(Errors.InsufficientBalance.selector, totalValue, totalValue + 1));
        ics27Account.executeBatch(calls);

        // Assert initial balances
        assertEq(address(ics27Account).balance, totalValue, "ICS27Account balance should match totalValue");
        assertEq(target.balance, 0, "Target balance should still be zero");
    }

    function test_success_delegateExecute() public {
        address target = address(new TestCallContract());
        bytes32 testSlot = TestCallContract(target).TEST_STORAGE_SLOT();
        bytes32 testValue = keccak256("testValue");
        bytes memory data = abi.encodeCall(TestCallContract.writeCall, (testValue));
        bytes memory resp = abi.encode("mockedResponse");

        vm.expectCall(target, data);
        vm.prank(address(ics27Account));
        bytes memory result = ics27Account.delegateExecute(target, data);
        assertEq(result, resp, "Result should match mocked response");

        // Assert storage was written
        bytes32 storedValue = vm.load(address(ics27Account), bytes32(testSlot));
        assertEq(storedValue, testValue, "Storage value should match written value");
    }

    function test_failure_delegateExecute() public {
        address target = address(new TestCallContract());
        bytes32 testSlot = TestCallContract(target).TEST_STORAGE_SLOT();
        bytes32 testValue = keccak256("testValue");
        bytes memory data = abi.encodeCall(TestCallContract.writeCall, (testValue));
        bytes memory errResp = abi.encode("mockedError");

        // Unauthorized call
        address unauthorized = makeAddr("unauthorized");
        vm.expectRevert(
            abi.encodeWithSelector(IICS27Errors.ICS27Unauthorized.selector, address(ics27Account), unauthorized)
        );
        vm.prank(unauthorized);
        ics27Account.delegateExecute(target, data);

        // Call reverts
        vm.mockCallRevert(target, data, errResp);
        vm.expectRevert(errResp);
        vm.prank(address(ics27Account));
        ics27Account.delegateExecute(target, data);

        // Assert storage was not written
        bytes32 storedValue = vm.load(address(ics27Account), bytes32(testSlot));
        assertEq(storedValue, bytes32(0), "Storage value should be empty");
    }
}

contract TestCallContract {
    struct Storage {
        bytes32 value;
    }

    bytes32 public constant TEST_STORAGE_SLOT = keccak256("test.storage.slot");

    // TODO: Remove this once the foundry bug is addressed
    // https://github.com/foundry-rs/foundry/issues/10812
    function payableCall() external payable returns (bytes memory) {
        return "mockedResponse";
    }

    function writeCall(bytes32 value) external returns (bytes memory) {
        Storage storage $ = _getStorage();
        $.value = value;
        return "mockedResponse";
    }

    function _getStorage() internal pure returns (Storage storage $) {
        bytes32 slot = TEST_STORAGE_SLOT;
        assembly {
            $.slot := slot
        }
    }
}
