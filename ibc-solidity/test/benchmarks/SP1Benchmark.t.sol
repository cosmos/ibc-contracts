// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable custom-errors,max-line-length,avoid-low-level-calls,gas-small-strings

import { IICS20TransferMsgs } from "../../contracts/msgs/IICS20TransferMsgs.sol";
import { IICS20Transfer } from "../../contracts/interfaces/IICS20Transfer.sol";

import { TestERC20 } from "../solidity-ibc/mocks/TestERC20.sol";
import { ICS20Lib } from "../../contracts/utils/ICS20Lib.sol";
import { ICS24Host } from "../../contracts/utils/ICS24Host.sol";
import { FixtureTest } from "../solidity-ibc/FixtureTest.t.sol";

contract SP1Benchmark is FixtureTest {
    string internal constant SNAPSHOT_GROUP = "SP1E2E";

    function testBenchmark_ICS20Transfer_Plonk() public {
        _benchmarkICS20Transfer(
            "acknowledgeMultiPacket_1-plonk.json", "receiveMultiPacket_1-plonk.json", "erc20.plonk.1", 1
        );
    }

    function testBenchmark_ICS20Transfer_Groth16() public {
        _benchmarkICS20Transfer(
            "acknowledgeMultiPacket_1-groth16.json", "receiveMultiPacket_1-groth16.json", "erc20.groth16.1", 1
        );
    }

    function testBenchmark_ICS20Transfer_50Packets_Plonk() public {
        _benchmarkICS20Transfer(
            "acknowledgeMultiPacket_50-plonk.json", "receiveMultiPacket_50-plonk.json", "erc20.plonk.50", 50
        );
    }

    function testBenchmark_ICS20Transfer_25Packets_Groth16() public {
        _benchmarkICS20Transfer(
            "acknowledgeMultiPacket_25-groth16.json", "receiveMultiPacket_25-groth16.json", "erc20.groth16.25", 25
        );
    }

    function testBenchmark_ICS20Transfer_50Packets_Groth16() public {
        _benchmarkICS20Transfer(
            "acknowledgeMultiPacket_50-groth16.json", "receiveMultiPacket_50-groth16.json", "erc20.groth16.50", 50
        );
    }

    function testBenchmark_ICS20TransferNativeSdkCoin_Plonk() public {
        _benchmarkICS20TransferNativeSdkCoin("receiveNativePacket-plonk.json", "native.plonk.1");
    }

    function testBenchmark_ICS20TransferNativeSdkCoin_Groth16() public {
        _benchmarkICS20TransferNativeSdkCoin("receiveNativePacket-groth16.json", "native.groth16.1");
    }

    function testBenchmark_ICS20Timeout_Plonk() public {
        _benchmarkICS20Timeout("timeoutMultiPacket_1-plonk.json", "timeout.plonk.1");
    }

    function testBenchmark_ICS20Timeout_Groth16() public {
        _benchmarkICS20Timeout("timeoutMultiPacket_1-groth16.json", "timeout.groth16.1");
    }

    function _benchmarkICS20Transfer(
        string memory acknowledgementFixture,
        string memory receiveFixture,
        string memory snapshotPrefix,
        uint64 packetCount
    )
        internal
    {
        Fixture memory ackFixture = loadInitialFixture(acknowledgementFixture);

        uint256 totalSendGas;
        for (uint64 i = 0; i < packetCount; ++i) {
            totalSendGas += _sendTransfer(ackFixture, snapshotPrefix);
        }
        vm.snapshotValue(SNAPSHOT_GROUP, string.concat(snapshotPrefix, ".send.gas"), totalSendGas / packetCount);

        (bool success,) = address(ics26Router).call(ackFixture.msg);
        vm.snapshotGasLastFrame(SNAPSHOT_GROUP, string.concat(snapshotPrefix, ".ack.gas"));
        vm.snapshotValue(SNAPSHOT_GROUP, string.concat(snapshotPrefix, ".ack.calldata"), ackFixture.msg.length);
        assertTrue(success);

        bytes32 path = ICS24Host.packetCommitmentKeyCalldata(ackFixture.packet.sourceClient, ackFixture.packet.sequence);
        assertEq(ics26Router.getCommitment(path), 0);

        Fixture memory recvFixture = loadFixture(receiveFixture);
        (success,) = address(ics26Router).call(recvFixture.msg);
        vm.snapshotGasLastFrame(SNAPSHOT_GROUP, string.concat(snapshotPrefix, ".recv.gas"));
        vm.snapshotValue(SNAPSHOT_GROUP, string.concat(snapshotPrefix, ".recv.calldata"), recvFixture.msg.length);
        assertTrue(success);

        bytes32 storedAck = ics26Router.getCommitment(
            ICS24Host.packetAcknowledgementCommitmentKeyCalldata(
                recvFixture.packet.destClient, recvFixture.packet.sequence
            )
        );
        assertEq(storedAck, ICS24Host.packetAcknowledgementCommitmentBytes32(singleSuccessAck));
    }

    function _benchmarkICS20TransferNativeSdkCoin(string memory receiveFixture, string memory snapshotPrefix) internal {
        Fixture memory recvNativeFixture = loadInitialFixture(receiveFixture);

        (bool success,) = address(ics26Router).call(recvNativeFixture.msg);
        vm.snapshotGasLastFrame(SNAPSHOT_GROUP, string.concat(snapshotPrefix, ".recv.gas"));
        vm.snapshotValue(SNAPSHOT_GROUP, string.concat(snapshotPrefix, ".recv.calldata"), recvNativeFixture.msg.length);
        assertTrue(success);

        bytes32 storedAck = ics26Router.getCommitment(
            ICS24Host.packetAcknowledgementCommitmentKeyCalldata(
                recvNativeFixture.packet.destClient, recvNativeFixture.packet.sequence
            )
        );
        assertEq(storedAck, ICS24Host.packetAcknowledgementCommitmentBytes32(singleSuccessAck));
    }

    function _benchmarkICS20Timeout(string memory timeoutFixture, string memory snapshotPrefix) internal {
        Fixture memory fixture = loadInitialFixture(timeoutFixture);

        vm.warp(fixture.packet.timeoutTimestamp - 30);
        _sendTransfer(fixture, snapshotPrefix);

        vm.warp(fixture.packet.timeoutTimestamp + 180);
        (bool success,) = address(ics26Router).call(fixture.msg);
        vm.snapshotGasLastFrame(SNAPSHOT_GROUP, string.concat(snapshotPrefix, ".timeout.gas"));
        vm.snapshotValue(SNAPSHOT_GROUP, string.concat(snapshotPrefix, ".timeout.calldata"), fixture.msg.length);
        assertTrue(success);

        bytes32 path = ICS24Host.packetCommitmentKeyCalldata(fixture.packet.sourceClient, fixture.packet.sequence);
        assertEq(ics26Router.getCommitment(path), 0);
    }

    function _sendTransfer(Fixture memory fixture, string memory snapshotPrefix) internal returns (uint256 gasUsed) {
        TestERC20 erc20 = TestERC20(fixture.erc20Address);
        IICS20TransferMsgs.FungibleTokenPacketData memory packetData =
            abi.decode(fixture.packet.payloads[0].value, (IICS20TransferMsgs.FungibleTokenPacketData));
        address user = ICS20Lib.mustHexStringToAddress(packetData.sender);
        IICS20TransferMsgs.SendTransferMsg memory message = IICS20TransferMsgs.SendTransferMsg({
            denom: ICS20Lib.mustHexStringToAddress(packetData.denom),
            amount: packetData.amount,
            receiver: packetData.receiver,
            sourceClient: fixture.packet.sourceClient,
            destPort: fixture.packet.payloads[0].destPort,
            timeoutTimestamp: fixture.packet.timeoutTimestamp,
            memo: packetData.memo
        });

        erc20.mint(user, packetData.amount);
        vm.prank(user);
        erc20.approve(address(ics20Transfer), packetData.amount);

        vm.prank(user);
        ics20Transfer.sendTransfer(message);
        gasUsed = vm.snapshotGasLastFrame(SNAPSHOT_GROUP, string.concat(snapshotPrefix, ".send.gas"));
        vm.snapshotValue(
            SNAPSHOT_GROUP,
            string.concat(snapshotPrefix, ".send.calldata"),
            abi.encodeCall(IICS20Transfer.sendTransfer, (message)).length
        );

        bytes32 path = ICS24Host.packetCommitmentKeyCalldata(fixture.packet.sourceClient, fixture.packet.sequence);
        assertEq(ics26Router.getCommitment(path), ICS24Host.packetCommitmentBytes32(fixture.packet));
    }
}
