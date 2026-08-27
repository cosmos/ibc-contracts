// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable custom-errors,max-line-length,avoid-low-level-calls,gas-small-strings

// solhint-disable-next-line no-global-import
import "forge-std/console.sol";

import { IICS20TransferMsgs } from "../../contracts/msgs/IICS20TransferMsgs.sol";

import { TestERC20 } from "./mocks/TestERC20.sol";
import { ICS20Lib } from "../../contracts/utils/ICS20Lib.sol";
import { ICS24Host } from "../../contracts/utils/ICS24Host.sol";
import { FixtureTest } from "./FixtureTest.t.sol";

contract BenchmarkTest is FixtureTest {
    function test_ICS20TransferWithSP1Fixtures_Plonk() public {
        ICS20TransferWithSP1FixturesTest("acknowledgeMultiPacket_1-plonk.json", "receiveMultiPacket_1-plonk.json", 1);
    }

    function test_ICS20TransferWithSP1Fixtures_Groth16() public {
        ICS20TransferWithSP1FixturesTest(
            "acknowledgeMultiPacket_1-groth16.json", "receiveMultiPacket_1-groth16.json", 1
        );
    }

    function test_ICS20TransferWithSP1Fixtures_50Packets_Plonk() public {
        ICS20TransferWithSP1FixturesTest("acknowledgeMultiPacket_50-plonk.json", "receiveMultiPacket_50-plonk.json", 50);
    }

    function test_ICS20TransferWithSP1Fixtures_25Packets_Groth16() public {
        ICS20TransferWithSP1FixturesTest(
            "acknowledgeMultiPacket_25-groth16.json", "receiveMultiPacket_25-groth16.json", 25
        );
    }

    function test_ICS20TransferWithSP1Fixtures_50Packets_Groth16() public {
        ICS20TransferWithSP1FixturesTest(
            "acknowledgeMultiPacket_50-groth16.json", "receiveMultiPacket_50-groth16.json", 50
        );
    }

    function ICS20TransferWithSP1FixturesTest(string memory ackFix, string memory recvFix, uint64 numPackets) public {
        Fixture memory ackFixture = loadInitialFixture(ackFix);

        // Step 1: Transfer from Ethereum to Cosmos
        uint64 sendGasUsed = 0;
        for (uint64 i = 0; i < numPackets; ++i) {
            sendGasUsed += sendTransfer(ackFixture);
        }
        console.log("Avg (", numPackets, "packets ) Send packet gas used: ", sendGasUsed / numPackets);

        // Step 2: Cosmos has received the packet and commited an acknowledgement, which we will now prove and process
        (bool success,) = address(ics26Router).call(ackFixture.msg);
        console.log(
            "Avg (", numPackets, "packets ) Multicall ack gas used: ", vm.lastCallGas().gasTotalUsed / numPackets
        );
        console.log("Total (", numPackets, "packets ) Multicall ack msg size: ", ackFixture.msg.length);
        assertTrue(success);

        // ack should be deleted
        bytes32 path = ICS24Host.packetCommitmentKeyCalldata(ackFixture.packet.sourceClient, ackFixture.packet.sequence);
        bytes32 storedCommitment = ics26Router.getCommitment(path);
        assertEq(storedCommitment, 0);

        // Step 3: Cosmos has sent the tokens back and commited a packet, which we will now prove and receive
        Fixture memory recvFixture = loadFixture(recvFix);

        (success,) = address(ics26Router).call(recvFixture.msg);
        console.log(
            "Avg (", numPackets, "packets ) Multicall recv gas used: ", vm.lastCallGas().gasTotalUsed / numPackets
        );
        console.log("Total (", numPackets, "packets ) Multicall recv msg size: ", recvFixture.msg.length);
        assertTrue(success);

        // ack is written
        bytes32 storedAck = ics26Router.getCommitment(
            ICS24Host.packetAcknowledgementCommitmentKeyCalldata(
                recvFixture.packet.destClient, recvFixture.packet.sequence
            )
        );
        assertEq(storedAck, ICS24Host.packetAcknowledgementCommitmentBytes32(singleSuccessAck));
    }

    function test_ICS20TransferNativeSdkCoinWithSP1Fixtures_Plonk() public {
        ICS20TransferNativeSdkCoinWithSP1FixtureTest("receiveNativePacket-plonk.json");
    }

    function test_ICS20TransferNativeSdkCoinWithSP1Fixtures_Groth16() public {
        ICS20TransferNativeSdkCoinWithSP1FixtureTest("receiveNativePacket-groth16.json");
    }

    function ICS20TransferNativeSdkCoinWithSP1FixtureTest(string memory recvNatFix) public {
        Fixture memory recvNativeFixture = loadInitialFixture(recvNatFix);

        (bool success,) = address(ics26Router).call(recvNativeFixture.msg);
        console.log("Multicall native recv gas used: ", vm.lastCallGas().gasTotalUsed);
        console.log("Multicall native recv msg size: ", recvNativeFixture.msg.length);
        assertTrue(success);

        bytes32 storedAck = ics26Router.getCommitment(
            ICS24Host.packetAcknowledgementCommitmentKeyCalldata(
                recvNativeFixture.packet.destClient, recvNativeFixture.packet.sequence
            )
        );
        assertEq(storedAck, ICS24Host.packetAcknowledgementCommitmentBytes32(singleSuccessAck));
    }

    function test_ICS20TimeoutWithSP1Fixtures_Plonk() public {
        ICS20TimeoutWithSP1FixtureTest("timeoutMultiPacket_1-plonk.json");
    }

    function test_ICS20TimeoutWithSP1Fixtures_Groth16() public {
        ICS20TimeoutWithSP1FixtureTest("timeoutMultiPacket_1-groth16.json");
    }

    function ICS20TimeoutWithSP1FixtureTest(string memory timeoutFix) public {
        Fixture memory timeoutFixture = loadInitialFixture(timeoutFix);

        // Step 1: Transfer from Ethereum to Cosmos
        vm.warp(timeoutFixture.packet.timeoutTimestamp - 30);
        uint64 sendGasUsed = sendTransfer(timeoutFixture);
        console.log("Send packet gas used: ", sendGasUsed);

        // Step 2: Timeout
        vm.warp(timeoutFixture.packet.timeoutTimestamp + 180);
        (bool success,) = address(ics26Router).call(timeoutFixture.msg);
        console.log("Multicall timeout gas used: ", vm.lastCallGas().gasTotalUsed);
        console.log("Multicall timeout msg size: ", timeoutFixture.msg.length);
        assertTrue(success);

        // ack should be deleted
        bytes32 path =
            ICS24Host.packetCommitmentKeyCalldata(timeoutFixture.packet.sourceClient, timeoutFixture.packet.sequence);
        assertEq(ics26Router.getCommitment(path), 0);
    }

    function sendTransfer(Fixture memory fixture) internal returns (uint64) {
        TestERC20 erc20 = TestERC20(fixture.erc20Address);

        IICS20TransferMsgs.FungibleTokenPacketData memory packetData =
            abi.decode(fixture.packet.payloads[0].value, (IICS20TransferMsgs.FungibleTokenPacketData));

        address user = ICS20Lib.mustHexStringToAddress(packetData.sender);

        uint256 amountToSend = packetData.amount;
        erc20.mint(user, amountToSend);
        vm.prank(user);
        erc20.approve(address(ics20Transfer), amountToSend);

        vm.prank(user);
        ics20Transfer.sendTransfer(
            IICS20TransferMsgs.SendTransferMsg({
                denom: ICS20Lib.mustHexStringToAddress(packetData.denom),
                amount: amountToSend,
                receiver: packetData.receiver,
                sourceClient: fixture.packet.sourceClient,
                destPort: fixture.packet.payloads[0].destPort,
                timeoutTimestamp: fixture.packet.timeoutTimestamp,
                memo: packetData.memo
            })
        );

        uint64 gasUsed = vm.lastCallGas().gasTotalUsed;

        bytes32 path = ICS24Host.packetCommitmentKeyCalldata(fixture.packet.sourceClient, fixture.packet.sequence);
        assertEq(ics26Router.getCommitment(path), ICS24Host.packetCommitmentBytes32(fixture.packet));

        return gasUsed;
    }
}
