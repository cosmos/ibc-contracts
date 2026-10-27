// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable
// custom-errors,max-line-length,max-states-count,var-name-mixedcase,gas-small-strings

import { IICS26RouterMsgs } from "../../../../contracts/msgs/IICS26RouterMsgs.sol";
import { IICS26Router, IICS26RouterAccessControlled } from "../../../../contracts/interfaces/IICS26Router.sol";
import { IIFTMsgs } from "../../../../contracts/msgs/IIFTMsgs.sol";
import { IIBCMulticallMsgs } from "../../../../contracts/msgs/IIBCMulticallMsgs.sol";

import { IIFT } from "../../../../contracts/interfaces/IIFT.sol";
import { IIFTRateLimit } from "../../../../contracts/interfaces/IIFTRateLimit.sol";
import { IIFTErrors } from "../../../../contracts/errors/IIFTErrors.sol";
import { IIBCMulticall } from "../../../../contracts/interfaces/IIBCMulticall.sol";

import { IbcImpl } from "../../utils/IbcImpl.sol";
import { LightClientDriverTest } from "../drivers/LightClientDriverTest.sol";
import { TestHelper } from "../../utils/TestHelper.sol";
import { IntegrationEnv } from "../../utils/IntegrationEnv.sol";
import { IFTAccessManaged } from "../../../../contracts/utils/IFTAccessManaged.sol";
import { IBCMulticall } from "../../../../contracts/utils/IBCMulticall.sol";
import { EVMIFTSendCallConstructor } from "../../../../contracts/utils/EVMIFTSendCallConstructor.sol";
import { ICS24Host } from "../../../../contracts/utils/ICS24Host.sol";
import { Strings } from "@openzeppelin-contracts/utils/Strings.sol";
import { RateLimiter } from "@openzeppelin-contracts/utils/RateLimiter.sol";
import { ERC1967Proxy } from "@openzeppelin-contracts/proxy/ERC1967/ERC1967Proxy.sol";

abstract contract IFTIntegrationSuite is LightClientDriverTest {
    IbcImpl public ibcImplA;
    IbcImpl public ibcImplB;

    TestHelper public th = new TestHelper();
    IntegrationEnv public integrationEnv;

    IFTAccessManaged public iftOnA;
    IFTAccessManaged public iftOnB;
    EVMIFTSendCallConstructor public sendCallConstructor;

    string public constant TOKEN_NAME = "Test IFT";
    string public constant TOKEN_SYMBOL = "TIFT";

    // Generous limits so that the fuzzed amounts below are never rate limited
    uint208 public constant MAX_RATE_LIMIT_CAPACITY = type(uint208).max;
    uint48 public constant RATE_LIMIT_WINDOW = 1 days;

    function setUp() public {
        integrationEnv = new IntegrationEnv();

        ibcImplA = new IbcImpl(integrationEnv.permit2());
        ibcImplB = new IbcImpl(integrationEnv.permit2());

        string memory clientId;
        clientId = ibcImplA.addClient(th.FIRST_CLIENT_ID(), _newDriver(ibcImplA.ics26Router(), ibcImplB.ics26Router()));
        assertEq(clientId, th.FIRST_CLIENT_ID());

        clientId = ibcImplB.addClient(th.FIRST_CLIENT_ID(), _newDriver(ibcImplB.ics26Router(), ibcImplA.ics26Router()));
        assertEq(clientId, th.FIRST_CLIENT_ID());

        sendCallConstructor = new EVMIFTSendCallConstructor();

        IFTAccessManaged impl = new IFTAccessManaged();

        ERC1967Proxy proxyA = new ERC1967Proxy(
            address(impl),
            abi.encodeCall(
                IFTAccessManaged.initialize,
                (address(ibcImplA.accessManager()), TOKEN_NAME, TOKEN_SYMBOL, address(ibcImplA.ics27Gmp()))
            )
        );
        iftOnA = IFTAccessManaged(address(proxyA));

        ERC1967Proxy proxyB = new ERC1967Proxy(
            address(impl),
            abi.encodeCall(
                IFTAccessManaged.initialize,
                (address(ibcImplB.accessManager()), TOKEN_NAME, TOKEN_SYMBOL, address(ibcImplB.ics27Gmp()))
            )
        );
        iftOnB = IFTAccessManaged(address(proxyB));

        _setupBridgePermissions();
        _registerBridges();
        _setRateLimits(MAX_RATE_LIMIT_CAPACITY, RATE_LIMIT_WINDOW);
    }

    function _setupBridgePermissions() internal {
        uint64 ADMIN_ROLE = 0;

        bytes4[] memory selectors = new bytes4[](2);
        selectors[0] = IIFT.registerIFTBridge.selector;
        selectors[1] = IIFTRateLimit.setIFTRateLimit.selector;

        ibcImplA.accessManager().setTargetFunctionRole(address(iftOnA), selectors, ADMIN_ROLE);
        ibcImplB.accessManager().setTargetFunctionRole(address(iftOnB), selectors, ADMIN_ROLE);
    }

    function _registerBridges() internal {
        iftOnA.registerIFTBridge(
            th.FIRST_CLIENT_ID(), Strings.toChecksumHexString(address(iftOnB)), address(sendCallConstructor)
        );

        iftOnB.registerIFTBridge(
            th.FIRST_CLIENT_ID(), Strings.toChecksumHexString(address(iftOnA)), address(sendCallConstructor)
        );
    }

    function _setRateLimits(uint208 capacity, uint48 window) internal {
        iftOnA.setIFTRateLimit(capacity, window);
        iftOnB.setIFTRateLimit(capacity, window);
    }

    function test_deployment() public view {
        IIFTMsgs.IFTBridge memory bridgeA = iftOnA.getIFTBridge(th.FIRST_CLIENT_ID());
        assertEq(bridgeA.clientId, th.FIRST_CLIENT_ID());
        assertEq(bridgeA.counterpartyIFTAddress, Strings.toChecksumHexString(address(iftOnB)));
        assertEq(address(bridgeA.iftSendCallConstructor), address(sendCallConstructor));

        IIFTMsgs.IFTBridge memory bridgeB = iftOnB.getIFTBridge(th.FIRST_CLIENT_ID());
        assertEq(bridgeB.clientId, th.FIRST_CLIENT_ID());
        assertEq(bridgeB.counterpartyIFTAddress, Strings.toChecksumHexString(address(iftOnA)));
        assertEq(address(bridgeB.iftSendCallConstructor), address(sendCallConstructor));
    }

    function testFuzz_success_iftTransferAcrossChains(uint256 amount) public {
        amount = bound(amount, 1, MAX_RATE_LIMIT_CAPACITY);

        address sender = integrationEnv.createUser();
        address receiver = integrationEnv.createUser();
        string memory clientId = th.FIRST_CLIENT_ID();

        deal(address(iftOnA), sender, amount, true);
        assertEq(iftOnA.balanceOf(sender), amount);

        vm.startPrank(sender);
        vm.recordLogs();
        iftOnA.iftTransfer(clientId, Strings.toHexString(receiver), amount);
        vm.stopPrank();

        assertEq(iftOnA.balanceOf(sender), 0, "tokens should be burned from sender");

        IICS26RouterMsgs.Packet memory sentPacket = _extractPacketFromLogs();

        IIFTMsgs.PendingTransfer memory pending = iftOnA.getPendingTransfer(clientId, sentPacket.sequence);
        assertEq(pending.sender, sender);
        assertEq(pending.amount, amount);

        bytes[] memory acks = ibcImplB.recvPacket(sentPacket);
        assertEq(acks.length, 1, "should have one ack");

        assertEq(iftOnB.balanceOf(receiver), amount, "receiver should have received tokens");

        ibcImplA.ackPacket(sentPacket, acks);

        vm.expectRevert(
            abi.encodeWithSelector(IIFTErrors.IFTPendingTransferNotFound.selector, clientId, sentPacket.sequence)
        );
        iftOnA.getPendingTransfer(clientId, sentPacket.sequence);
    }

    function testFuzz_success_roundTripTransfer(uint256 amount) public {
        amount = bound(amount, 1, MAX_RATE_LIMIT_CAPACITY);

        address userA = integrationEnv.createUser();
        address userB = integrationEnv.createUser();

        deal(address(iftOnA), userA, amount, true);

        vm.startPrank(userA);
        vm.recordLogs();
        iftOnA.iftTransfer(th.FIRST_CLIENT_ID(), Strings.toHexString(userB), amount);
        vm.stopPrank();

        IICS26RouterMsgs.Packet memory packetAtoB = _extractPacketFromLogs();
        bytes[] memory acksAtoB = ibcImplB.recvPacket(packetAtoB);
        ibcImplA.ackPacket(packetAtoB, acksAtoB);

        assertEq(iftOnA.balanceOf(userA), 0);
        assertEq(iftOnB.balanceOf(userB), amount);

        vm.startPrank(userB);
        vm.recordLogs();
        iftOnB.iftTransfer(th.FIRST_CLIENT_ID(), Strings.toHexString(userA), amount);
        vm.stopPrank();

        IICS26RouterMsgs.Packet memory packetBtoA = _extractPacketFromLogs();
        bytes[] memory acksBtoA = ibcImplA.recvPacket(packetBtoA);
        ibcImplB.ackPacket(packetBtoA, acksBtoA);

        assertEq(iftOnB.balanceOf(userB), 0);
        assertEq(iftOnA.balanceOf(userA), amount);
    }

    function testFuzz_timeout_refundsTokens(uint256 amount) public {
        amount = bound(amount, 1, MAX_RATE_LIMIT_CAPACITY);

        address sender = integrationEnv.createUser();
        address receiver = integrationEnv.createUser();
        string memory clientId = th.FIRST_CLIENT_ID();

        deal(address(iftOnA), sender, amount, true);

        uint64 shortTimeout = uint64(block.timestamp) + 1;

        vm.startPrank(sender);
        vm.recordLogs();
        iftOnA.iftTransfer(clientId, Strings.toHexString(receiver), amount, shortTimeout);
        vm.stopPrank();

        IICS26RouterMsgs.Packet memory sentPacket = _extractPacketFromLogs();

        assertEq(iftOnA.balanceOf(sender), 0, "tokens should be burned");

        vm.warp(block.timestamp + 100);

        ibcImplA.cheatPacketCommitment(sentPacket);
        ibcImplA.timeoutPacket(sentPacket);

        assertEq(iftOnA.balanceOf(sender), amount, "tokens should be refunded");

        vm.expectRevert(
            abi.encodeWithSelector(IIFTErrors.IFTPendingTransferNotFound.selector, clientId, sentPacket.sequence)
        );
        iftOnA.getPendingTransfer(clientId, sentPacket.sequence);
    }

    /// @dev A mint rejected on the destination, here by its inbound rate limit, is acknowledged with an error. A refund
    /// rejected on the source keeps the packet relayable until it succeeds.
    function testFuzz_rejectedMint_errorAckRefundsAfterRetry(uint256 amount) public {
        amount = bound(amount, 2, MAX_RATE_LIMIT_CAPACITY);

        address sender = integrationEnv.createUser();
        address receiver = integrationEnv.createUser();
        string memory clientId = th.FIRST_CLIENT_ID();

        deal(address(iftOnA), sender, amount, true);

        vm.startPrank(sender);
        vm.recordLogs();
        iftOnA.iftTransfer(clientId, Strings.toHexString(receiver), amount);
        vm.stopPrank();

        IICS26RouterMsgs.Packet memory sentPacket = _extractPacketFromLogs();

        // The destination rejects the mint and writes an error acknowledgement without minting
        iftOnB.setIFTRateLimit(uint208(amount - 1), RATE_LIMIT_WINDOW);
        bytes[] memory acks = ibcImplB.recvPacket(sentPacket);
        assertEq(acks[0], ICS24Host.UNIVERSAL_ERROR_ACK, "mint rejection should be an error ack");
        assertEq(iftOnB.totalSupply(), 0, "nothing should be minted on the destination");

        // The source rejects the refund, so the whole ack relay reverts and the packet stays relayable
        iftOnA.setIFTRateLimit(uint208(amount - 1), RATE_LIMIT_WINDOW);
        vm.expectRevert(RateLimiter.RateLimitExceeded.selector);
        ibcImplA.ackPacket(sentPacket, acks);
        _assertStillPending(sentPacket, sender, amount);

        // Once the refund fits, relaying the same ack again refunds the sender
        iftOnA.setIFTRateLimit(MAX_RATE_LIMIT_CAPACITY, RATE_LIMIT_WINDOW);
        ibcImplA.ackPacket(sentPacket, acks);
        assertEq(iftOnA.balanceOf(sender), amount, "tokens should be refunded");
        vm.expectRevert(
            abi.encodeWithSelector(IIFTErrors.IFTPendingTransferNotFound.selector, clientId, sentPacket.sequence)
        );
        iftOnA.getPendingTransfer(clientId, sentPacket.sequence);
    }

    /// @dev A refund rejected on timeout, here by the source's inbound rate limit, keeps the packet relayable until it
    /// succeeds
    function testFuzz_rejectedTimeoutRefund_refundsAfterRetry(uint256 amount) public {
        amount = bound(amount, 2, MAX_RATE_LIMIT_CAPACITY);

        address sender = integrationEnv.createUser();
        address receiver = integrationEnv.createUser();
        string memory clientId = th.FIRST_CLIENT_ID();

        deal(address(iftOnA), sender, amount, true);

        uint64 shortTimeout = uint64(block.timestamp) + 1;

        vm.startPrank(sender);
        vm.recordLogs();
        iftOnA.iftTransfer(clientId, Strings.toHexString(receiver), amount, shortTimeout);
        vm.stopPrank();

        IICS26RouterMsgs.Packet memory sentPacket = _extractPacketFromLogs();

        vm.warp(block.timestamp + 100);
        ibcImplA.cheatPacketCommitment(sentPacket);

        // The source rejects the refund, so the whole timeout relay reverts and the packet stays relayable
        iftOnA.setIFTRateLimit(uint208(amount - 1), RATE_LIMIT_WINDOW);
        vm.expectRevert(RateLimiter.RateLimitExceeded.selector);
        ibcImplA.timeoutPacket(sentPacket);
        _assertStillPending(sentPacket, sender, amount);

        // Once the refund fits, relaying the timeout again refunds the sender
        iftOnA.setIFTRateLimit(MAX_RATE_LIMIT_CAPACITY, RATE_LIMIT_WINDOW);
        ibcImplA.timeoutPacket(sentPacket);
        assertEq(iftOnA.balanceOf(sender), amount, "tokens should be refunded");
        vm.expectRevert(
            abi.encodeWithSelector(IIFTErrors.IFTPendingTransferNotFound.selector, clientId, sentPacket.sequence)
        );
        iftOnA.getPendingTransfer(clientId, sentPacket.sequence);
    }

    function testFuzz_multipleTransfersInFlight(uint256 amount1, uint256 amount2) public {
        // Both transfers share the outbound bucket, so together they must fit in the capacity
        amount1 = bound(amount1, 1, MAX_RATE_LIMIT_CAPACITY / 2);
        amount2 = bound(amount2, 1, MAX_RATE_LIMIT_CAPACITY / 2);

        address sender = integrationEnv.createUser();
        address receiver1 = integrationEnv.createUser();
        address receiver2 = integrationEnv.createUser();

        deal(address(iftOnA), sender, amount1 + amount2, true);

        vm.startPrank(sender);
        vm.recordLogs();
        iftOnA.iftTransfer(th.FIRST_CLIENT_ID(), Strings.toHexString(receiver1), amount1);
        IICS26RouterMsgs.Packet memory packet1 = _extractPacketFromLogs();

        vm.recordLogs();
        iftOnA.iftTransfer(th.FIRST_CLIENT_ID(), Strings.toHexString(receiver2), amount2);
        IICS26RouterMsgs.Packet memory packet2 = _extractPacketFromLogs();
        vm.stopPrank();

        assertEq(packet2.sequence, packet1.sequence + 1, "sequences should be consecutive");

        bytes[] memory acks2 = ibcImplB.recvPacket(packet2);
        ibcImplA.ackPacket(packet2, acks2);
        assertEq(iftOnB.balanceOf(receiver2), amount2);

        bytes[] memory acks1 = ibcImplB.recvPacket(packet1);
        ibcImplA.ackPacket(packet1, acks1);
        assertEq(iftOnB.balanceOf(receiver1), amount1);
    }

    /// @dev Two refunds that only fit the rate limit one at a time are relayed in one batch where each may fail. The
    /// first refund lands and the second packet stays relayable until its refund fits.
    function test_success_partialTimeoutBatch() public {
        uint256 amount = 1000;
        address sender = integrationEnv.createUser();
        string memory clientId = th.FIRST_CLIENT_ID();
        string memory receiverStr = Strings.toHexString(integrationEnv.createUser());
        uint64 timeout = uint64(block.timestamp) + 1;

        deal(address(iftOnA), sender, 2 * amount, true);
        IICS26RouterMsgs.Packet[2] memory packets;
        for (uint256 i = 0; i < 2; ++i) {
            vm.recordLogs();
            vm.prank(sender);
            iftOnA.iftTransfer(clientId, receiverStr, amount, timeout);
            packets[i] = _extractPacketFromLogs();
        }
        vm.warp(block.timestamp + 100);

        IIBCMulticallMsgs.Call[] memory calls = new IIBCMulticallMsgs.Call[](2);
        for (uint256 i = 0; i < 2; ++i) {
            calls[i] = IIBCMulticallMsgs.Call({
                target: address(ibcImplA.ics26Router()),
                allowFailure: true,
                callData: abi.encodeCall(
                    IICS26RouterAccessControlled.timeoutPacket, (ibcImplA.timeoutPacketMsg(packets[i]))
                )
            });
        }

        iftOnA.setIFTRateLimit(uint208(amount), RATE_LIMIT_WINDOW);
        IBCMulticall multicall = ibcImplA.ibcMulticall();
        vm.expectEmit(address(multicall));
        emit IIBCMulticall.CallFailed(1, abi.encodeWithSelector(RateLimiter.RateLimitExceeded.selector));
        multicall.multicall(calls);

        assertEq(iftOnA.balanceOf(sender), amount, "only the first transfer should be refunded");
        assertEq(iftOnA.getPendingTransfer(clientId, packets[1].sequence).amount, amount, "should stay pending");

        // Once the refund fits, relaying the failed timeout again refunds the sender
        iftOnA.setIFTRateLimit(MAX_RATE_LIMIT_CAPACITY, RATE_LIMIT_WINDOW);
        ibcImplA.timeoutPacket(packets[1]);
        assertEq(iftOnA.balanceOf(sender), 2 * amount, "both transfers should be refunded");
    }

    function test_upgrade_success() public {
        IFTAccessManaged newImpl = new IFTAccessManaged();

        iftOnA.upgradeToAndCall(address(newImpl), "");

        IIFTMsgs.IFTBridge memory bridge = iftOnA.getIFTBridge(th.FIRST_CLIENT_ID());
        assertEq(bridge.clientId, th.FIRST_CLIENT_ID(), "bridge should be preserved after upgrade");
    }

    function test_upgrade_unauthorizedCaller_reverts() public {
        IFTAccessManaged newImpl = new IFTAccessManaged();

        address unauthorizedUser = integrationEnv.createUser();

        vm.prank(unauthorizedUser);
        vm.expectRevert();
        iftOnA.upgradeToAndCall(address(newImpl), "");
    }

    function _assertStillPending(IICS26RouterMsgs.Packet memory packet, address sender, uint256 amount) internal view {
        bytes32 path = ICS24Host.packetCommitmentKeyCalldata(packet.sourceClient, packet.sequence);
        assertEq(
            ibcImplA.ics26Router().getCommitment(path),
            ICS24Host.packetCommitmentBytes32(packet),
            "packet commitment should be kept"
        );
        assertEq(iftOnA.getPendingTransfer(packet.sourceClient, packet.sequence).amount, amount, "should stay pending");
        assertEq(iftOnA.balanceOf(sender), 0, "nothing should be refunded");
    }

    function _extractPacketFromLogs() internal view returns (IICS26RouterMsgs.Packet memory) {
        bytes memory packetBz = th.getValueFromEvent(IICS26Router.SendPacket.selector);
        return abi.decode(packetBz, (IICS26RouterMsgs.Packet));
    }
}
