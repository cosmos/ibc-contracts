// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable custom-errors,max-line-length,no-inline-assembly,gas-small-strings,no-empty-blocks

import { Test } from "forge-std/Test.sol";

import { IICS27AccountMsgs } from "../../contracts/msgs/IICS27AccountMsgs.sol";
import { IICS27Errors } from "../../contracts/errors/IICS27Errors.sol";

import { ERC1967Proxy } from "@openzeppelin-contracts/proxy/ERC1967/ERC1967Proxy.sol";
import { ICS27Account } from "../../contracts/utils/ICS27Account.sol";
import { Errors } from "@openzeppelin-contracts/utils/Errors.sol";

contract ICS27AccountTest is Test {
    address public ics27 = makeAddr("ics27");
    ICS27Account public ics27Account;

    string private constant EXECUTE_WITH_GAS = "execute(address,bytes,uint256,uint256)";

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

        // Call reverts without data
        vm.mockCallRevert(target, data, "");
        vm.expectRevert(Errors.FailedCall.selector);
        vm.prank(ics27);
        ics27Account.functionCall(target, data);
    }

    function test_failure_functionCallOutOfGas() public {
        address target = address(new TestCallContract());

        // Target runs out of gas
        vm.expectRevert(bytes(""));
        vm.prank(ics27);
        ics27Account.functionCall{ gas: 1_000_000 }(target, abi.encodeCall(TestCallContract.burnGas, ()));

        // A call of the target runs out of gas
        bytes memory nestedData = abi.encodeCall(TestCallContract.nestedBurnGas, ());
        vm.expectRevert(bytes(""));
        vm.prank(ics27);
        ics27Account.functionCall{ gas: 1_000_000 }(target, nestedData);

        // Same, through the account calling itself
        IICS27AccountMsgs.Call[] memory calls = new IICS27AccountMsgs.Call[](1);
        calls[0] = IICS27AccountMsgs.Call({ target: target, data: nestedData, value: 0 });
        vm.expectRevert(bytes(""));
        vm.prank(ics27);
        ics27Account.functionCall{ gas: 1_000_000 }(
            address(ics27Account), abi.encodeWithSignature("executeBatch((address,bytes,uint256)[])", calls)
        );

        // Recipient of a transfer runs out of gas, through the account calling itself
        address payable burnGasRecipient = payable(address(new TestBurnGasRecipient()));
        vm.expectRevert(bytes(""));
        vm.prank(ics27);
        ics27Account.functionCall{ gas: 1_000_000 }(
            address(ics27Account), abi.encodeCall(ICS27Account.sendValue, (burnGasRecipient, 0))
        );
    }

    function test_success_executeWithGas() public {
        address target = address(new TestCallContract());
        bytes memory data = abi.encodeCall(TestCallContract.payableCall, ());
        uint256 gas_ = 100_000;
        vm.deal(address(ics27Account), 2);

        vm.expectCall(target, 1, uint64(gas_), data);
        vm.prank(address(ics27Account));
        ics27Account.execute(target, data, 1, gas_);

        IICS27AccountMsgs.CallWithGas[] memory calls = new IICS27AccountMsgs.CallWithGas[](1);
        calls[0] = IICS27AccountMsgs.CallWithGas({ target: target, data: data, value: 1, gas: gas_ });
        vm.expectCall(target, 1, uint64(gas_), data);
        vm.prank(address(ics27Account));
        ics27Account.executeBatch(calls);

        assertEq(target.balance, 2, "Target balance should match value after execute");
    }

    function testFuzz_success_executeWithGasIsStable(uint64 gasLimit) public {
        address target = address(new TestCallContract());
        bytes memory data =
            abi.encodeWithSignature(EXECUTE_WITH_GAS, target, abi.encodeCall(TestCallContract.gasLeft, ()), 0, 100_000);

        vm.prank(ics27);
        bytes memory expected = ics27Account.functionCall{ gas: 30_000_000 }(address(ics27Account), data);

        // The target sees the same gas whatever gas the relayer provides, or the relay runs out of gas
        vm.prank(ics27);
        // solhint-disable-next-line avoid-low-level-calls
        (bool success, bytes memory result) = address(ics27Account).call{ gas: bound(gasLimit, 100_000, 300_000) }(
            abi.encodeCall(ICS27Account.functionCall, (address(ics27Account), data))
        );
        if (success) {
            assertEq(abi.decode(result, (bytes)), expected, "Result should not depend on the gas provided");
        } else {
            assertEq(result, bytes(""), "Not enough gas should revert without data");
        }
    }

    function test_failure_executeWithGas() public {
        address target = address(new TestCallContract());
        bytes memory burnGasData = abi.encodeCall(TestCallContract.burnGas, ());
        uint256 gas_ = 100_000;

        // Not enough gas for the call runs out of gas, through the account calling itself
        vm.expectRevert(bytes(""));
        vm.prank(ics27);
        ics27Account.functionCall{ gas: 150_000 }(
            address(ics27Account), abi.encodeWithSignature(EXECUTE_WITH_GAS, target, burnGasData, 0, gas_)
        );

        // Running out of the gas given to the call is an error, whatever gas the relayer provides
        vm.expectRevert(Errors.FailedCall.selector);
        vm.prank(ics27);
        ics27Account.functionCall{ gas: 1_000_000 }(
            address(ics27Account), abi.encodeWithSignature(EXECUTE_WITH_GAS, target, burnGasData, 0, gas_)
        );

        IICS27AccountMsgs.CallWithGas[] memory calls = new IICS27AccountMsgs.CallWithGas[](1);
        calls[0] = IICS27AccountMsgs.CallWithGas({ target: target, data: burnGasData, value: 0, gas: gas_ });
        vm.expectRevert(Errors.FailedCall.selector);
        vm.prank(ics27);
        ics27Account.functionCall{ gas: 1_000_000 }(
            address(ics27Account), abi.encodeWithSignature("executeBatch((address,bytes,uint256,uint256)[])", calls)
        );

        // Unauthorized call
        address unauthorized = makeAddr("unauthorized");
        vm.expectRevert(
            abi.encodeWithSelector(IICS27Errors.ICS27Unauthorized.selector, address(ics27Account), unauthorized)
        );
        vm.prank(unauthorized);
        ics27Account.executeBatch(calls);
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

        // Recipient reverts
        bytes memory resp = abi.encode("mockedResponse");
        vm.mockCallRevert(recipient, bytes(""), resp);
        vm.prank(address(ics27Account));
        vm.expectRevert(resp);
        ics27Account.sendValue(recipient, amount);

        // Recipient reverts without data, as it has no receive function
        address payable noReceive = payable(address(new TestCallContract()));
        vm.prank(address(ics27Account));
        vm.expectRevert(Errors.FailedCall.selector);
        ics27Account.sendValue(noReceive, amount);

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

        // Call reverts without data
        vm.mockCallRevert(target, data, "");
        vm.expectRevert(Errors.FailedCall.selector);
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

        // Call reverts without data
        vm.mockCallRevert(target, revertData, "");
        vm.expectRevert(Errors.FailedCall.selector);
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
}

contract TestCallContract {
    // TODO: Remove this once the foundry bug is addressed
    // https://github.com/foundry-rs/foundry/issues/10812
    function payableCall() external payable returns (bytes memory) {
        return "mockedResponse";
    }

    function gasLeft() external view returns (uint256) {
        return gasleft();
    }

    function burnGas() external pure {
        while (true) { }
    }

    /// @dev Runs a call out of gas, whose empty revert data is bubbled up
    function nestedBurnGas() external view {
        this.burnGas();
    }
}

contract TestBurnGasRecipient {
    receive() external payable {
        while (true) { }
    }
}
