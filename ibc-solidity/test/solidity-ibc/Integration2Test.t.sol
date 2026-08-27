// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable custom-errors,max-line-length,max-states-count

import {Test} from "forge-std/Test.sol";

import {IICS26RouterMsgs} from "../../contracts/msgs/IICS26RouterMsgs.sol";
import {IICS27GMPMsgs} from "../../contracts/msgs/IICS27GMPMsgs.sol";

import {IERC20} from "@openzeppelin-contracts/token/ERC20/IERC20.sol";
import {ISignatureTransfer} from "@uniswap/permit2/src/interfaces/ISignatureTransfer.sol";
import {IICS26Router} from "../../contracts/interfaces/IICS26Router.sol";
import {IICS26RouterErrors} from "../../contracts/errors/IICS26RouterErrors.sol";
import {ILightClient} from "../../contracts/interfaces/ILightClient.sol";
import {IICS27Account} from "../../contracts/interfaces/IICS27Account.sol";

import {IbcImpl} from "./utils/IbcImpl.sol";
import {TestHelper} from "./utils/TestHelper.sol";
import {IntegrationEnv} from "./utils/IntegrationEnv.sol";
import {Strings} from "@openzeppelin-contracts/utils/Strings.sol";
import {ICS24Host} from "../../contracts/utils/ICS24Host.sol";
import {ICS20Lib} from "../../contracts/utils/ICS20Lib.sol";
import {ICS27Lib} from "../../contracts/utils/ICS27Lib.sol";
import {ERC1967Proxy} from "@openzeppelin-contracts/proxy/ERC1967/ERC1967Proxy.sol";
import {RefImplIBCERC20} from "./utils/RefImplIBCERC20.sol";

contract Integration2Test is Test {
    IbcImpl public ibcImplA;
    IbcImpl public ibcImplB;

    TestHelper public th = new TestHelper();
    IntegrationEnv public integrationEnv = new IntegrationEnv();

    function setUp() public {
        // Deploy the IBC implementation
        ibcImplA = new IbcImpl(integrationEnv.permit2());
        ibcImplB = new IbcImpl(integrationEnv.permit2());

        // Add the counterparty implementations
        string memory clientId;
        clientId = ibcImplA.addCounterpartyImpl(ibcImplB, th.FIRST_CLIENT_ID());
        assertEq(clientId, th.FIRST_CLIENT_ID());

        clientId = ibcImplB.addCounterpartyImpl(ibcImplA, th.FIRST_CLIENT_ID());
        assertEq(clientId, th.FIRST_CLIENT_ID());
    }

    function test_deployment() public view {
        // Check that the counterparty implementations are set correctly
        assertEq(
            ibcImplA.ics26Router().getClient(th.FIRST_CLIENT_ID()).getClientState(),
            abi.encodePacked(address(ibcImplB.ics26Router()))
        );
        assertEq(
            ibcImplB.ics26Router().getClient(th.FIRST_CLIENT_ID()).getClientState(),
            abi.encodePacked(address(ibcImplA.ics26Router()))
        );
    }

    function setup_createForeignDenomOnImplA(address receiver, uint256 amount) public returns (IERC20) {
        return setup_createForeignDenomOnImplA(receiver, amount, th.FIRST_CLIENT_ID());
    }
    /// @notice Create a foreign ibc denom on ibcImplA and client on a specified user
    /// @dev We do this by transferring the native erc20 from the counterparty chain

    function setup_createForeignDenomOnImplA(address receiver, uint256 amount, string memory clientId)
        public
        returns (IERC20)
    {
        address user = integrationEnv.createAndFundUser(amount);

        IICS26RouterMsgs.Packet memory sentPacket =
            ibcImplB.sendTransferAsUser(integrationEnv.erc20(), user, Strings.toHexString(receiver), amount, clientId);

        bytes[] memory acks = ibcImplA.recvPacket(sentPacket);
        assertEq(acks.length, 1, "ack length mismatch");
        assertEq(acks, th.SINGLE_SUCCESS_ACK(), "ack mismatch");

        ibcImplB.ackPacket(sentPacket, acks);

        string memory expDenomPath = string.concat(
            ICS20Lib.DEFAULT_PORT_ID,
            "/",
            th.FIRST_CLIENT_ID(),
            "/",
            Strings.toHexString(address(integrationEnv.erc20()))
        );
        address ibcERC20 = ibcImplA.ics20Transfer().ibcERC20Contract(expDenomPath);

        return IERC20(ibcERC20);
    }

    function testFuzz_success_native_sendICS20PacketWithAllowance(uint256 amount) public {
        vm.assume(amount > 0);

        address user = integrationEnv.createAndFundUser(amount);
        string memory receiver = th.randomString();

        IICS26RouterMsgs.Packet memory sentPacket =
            ibcImplA.sendTransferAsUser(integrationEnv.erc20(), user, receiver, amount);
        assertEq(integrationEnv.erc20().balanceOf(user), 0, "user balance mismatch");

        // check that the packet was committed correctly
        bytes32 path = ICS24Host.packetCommitmentKeyCalldata(sentPacket.sourceClient, sentPacket.sequence);
        bytes32 expCommitment = ICS24Host.packetCommitmentBytes32(sentPacket);
        bytes32 storedCommitment = ibcImplA.ics26Router().getCommitment(path);
        assertEq(storedCommitment, expCommitment, "packet commitment mismatch");

        // check that the escrow was created and funded correctly
        address escrow = ibcImplA.ics20Transfer().getEscrow(sentPacket.sourceClient);
        assertEq(integrationEnv.erc20().balanceOf(escrow), amount, "escrow balance mismatch");
    }

    function testFuzz_success_foreign_sendICS20PacketWithAllowance(uint256 amount) public {
        vm.assume(amount > 0);

        address user = integrationEnv.createUser();
        IERC20 ibcERC20 = setup_createForeignDenomOnImplA(user, amount);
        string memory receiver = th.randomString();

        IICS26RouterMsgs.Packet memory sentPacket = ibcImplA.sendTransferAsUser(ibcERC20, user, receiver, amount);
        assertEq(ibcERC20.balanceOf(user), 0, "user balance mismatch");

        // check that the packet was committed correctly
        bytes32 path = ICS24Host.packetCommitmentKeyCalldata(sentPacket.sourceClient, sentPacket.sequence);
        bytes32 expCommitment = ICS24Host.packetCommitmentBytes32(sentPacket);
        bytes32 storedCommitment = ibcImplA.ics26Router().getCommitment(path);
        assertEq(storedCommitment, expCommitment, "packet commitment mismatch");

        // check that the tokens were burned correctly
        address escrow = ibcImplA.ics20Transfer().getEscrow(sentPacket.sourceClient);
        assertEq(ibcERC20.balanceOf(escrow), 0, "escrow balance mismatch");
        assertEq(ibcERC20.balanceOf(user), 0, "user balance mismatch");
    }

    function testFuzz_success_native_sendICS20PacketWithPermit2(uint256 amount) public {
        vm.assume(amount > 0);

        address user = integrationEnv.createAndFundUser(amount);
        string memory receiver = th.randomString();

        ISignatureTransfer.PermitTransferFrom memory permit;
        bytes memory signature;
        (permit, signature) = integrationEnv.getPermitAndSignature(user, address(ibcImplA.ics20Transfer()), amount);

        IICS26RouterMsgs.Packet memory sentPacket =
            ibcImplA.sendTransferAsUser(integrationEnv.erc20(), user, receiver, permit, signature);
        assertEq(integrationEnv.erc20().balanceOf(user), 0, "user balance mismatch");

        // check that the packet was committed correctly
        bytes32 path = ICS24Host.packetCommitmentKeyCalldata(sentPacket.sourceClient, sentPacket.sequence);
        bytes32 expCommitment = ICS24Host.packetCommitmentBytes32(sentPacket);
        bytes32 storedCommitment = ibcImplA.ics26Router().getCommitment(path);
        assertEq(storedCommitment, expCommitment, "packet commitment mismatch");

        // check that the escrow was created and funded correctly
        address escrow = ibcImplA.ics20Transfer().getEscrow(sentPacket.sourceClient);
        assertEq(integrationEnv.erc20().balanceOf(escrow), amount, "escrow balance mismatch");
    }

    function testFuzz_success_foreign_sendICS20PacketWithPermit2(uint256 amount) public {
        vm.assume(amount > 0);

        address user = integrationEnv.createUser();
        IERC20 ibcERC20 = setup_createForeignDenomOnImplA(user, amount);
        string memory receiver = th.randomString();

        ISignatureTransfer.PermitTransferFrom memory permit;
        bytes memory signature;
        (permit, signature) =
            integrationEnv.getPermitAndSignature(user, address(ibcImplA.ics20Transfer()), amount, address(ibcERC20));

        IICS26RouterMsgs.Packet memory sentPacket =
            ibcImplA.sendTransferAsUser(ibcERC20, user, receiver, permit, signature);
        assertEq(ibcERC20.balanceOf(user), 0, "user balance mismatch");

        // check that the packet was committed correctly
        bytes32 path = ICS24Host.packetCommitmentKeyCalldata(sentPacket.sourceClient, sentPacket.sequence);
        bytes32 expCommitment = ICS24Host.packetCommitmentBytes32(sentPacket);
        bytes32 storedCommitment = ibcImplA.ics26Router().getCommitment(path);
        assertEq(storedCommitment, expCommitment, "packet commitment mismatch");

        // check that the tokens were burned correctly
        address escrow = ibcImplA.ics20Transfer().getEscrow(sentPacket.sourceClient);
        assertEq(ibcERC20.balanceOf(escrow), 0, "escrow balance mismatch");
        assertEq(ibcERC20.balanceOf(user), 0, "user balance mismatch");
    }

    function testFuzz_success_native_recvICS20Packet(uint256 amount) public {
        // We will send a packet from A to B and then receive it on B
        vm.assume(amount > 0);

        address user = integrationEnv.createAndFundUser(amount);
        address receiver = integrationEnv.createUser();

        IICS26RouterMsgs.Packet memory sentPacket =
            ibcImplA.sendTransferAsUser(integrationEnv.erc20(), user, Strings.toHexString(receiver), amount);

        // run the receive packet queries
        assertFalse(ibcImplB.relayerHelper().isPacketReceived(sentPacket));
        assertFalse(ibcImplB.relayerHelper().isPacketReceiveSuccessful(sentPacket));

        // Receive the packet on B
        bytes[] memory acks = ibcImplB.recvPacket(sentPacket);
        assertEq(acks.length, 1, "ack length mismatch");
        assertEq(acks, th.SINGLE_SUCCESS_ACK(), "ack mismatch");

        // run the receive packet queries
        assert(ibcImplB.relayerHelper().isPacketReceived(sentPacket));
        assert(ibcImplB.relayerHelper().isPacketReceiveSuccessful(sentPacket));

        // Verify that the packet acknowledgement was written correctly
        bytes32 path = ICS24Host.packetAcknowledgementCommitmentKeyCalldata(sentPacket.destClient, sentPacket.sequence);
        bytes32 expAckCommitment = ICS24Host.packetAcknowledgementCommitmentBytes32(th.SINGLE_SUCCESS_ACK());
        bytes32 storedAckCommitment = ibcImplB.ics26Router().getCommitment(path);
        assertEq(storedAckCommitment, expAckCommitment, "ack commitment mismatch");

        // Verify that the packet receipt was set correctly
        bytes32 receiptPath =
            keccak256(ICS24Host.packetReceiptCommitmentPathCalldata(sentPacket.destClient, sentPacket.sequence));
        bytes32 expReceipt = ICS24Host.packetReceiptCommitmentBytes32(sentPacket);
        bytes32 storedReceipt = ibcImplB.ics26Router().getCommitment(receiptPath);
        assertEq(storedReceipt, expReceipt, "receipt mismatch");

        // Check that a new IBCERC20 token was created
        string memory expDenomPath = string.concat(
            ICS20Lib.DEFAULT_PORT_ID,
            "/",
            th.FIRST_CLIENT_ID(),
            "/",
            Strings.toHexString(address(integrationEnv.erc20()))
        );
        IERC20 token = IERC20(ibcImplB.ics20Transfer().ibcERC20Contract(expDenomPath));
        assertTrue(address(token) != address(0), "IBCERC20 token not found");
        assertEq(token.balanceOf(receiver), amount, "receiver balance mismatch");

        // Check replay protection
        IICS26RouterMsgs.MsgRecvPacket memory msgRecvPacket;
        msgRecvPacket.packet = sentPacket;
        vm.recordLogs();
        ibcImplB.ics26Router().recvPacket(msgRecvPacket);
        th.getValueFromEvent(IICS26Router.Noop.selector);
    }

    function testFuzz_success_custom_recvICS20Packet(uint256 amount) public {
        // We will send a packet from A to B and then receive it on B
        vm.assume(amount > 0);

        string memory expDenomPath = string.concat(
            ICS20Lib.DEFAULT_PORT_ID,
            "/",
            th.FIRST_CLIENT_ID(),
            "/",
            Strings.toHexString(address(integrationEnv.erc20()))
        );

        address customERC20 = address(
            new ERC1967Proxy(
                address(new RefImplIBCERC20()),
                abi.encodeCall(
                    RefImplIBCERC20.initialize,
                    (makeAddr("owner"), address(ibcImplB.ics20Transfer()), "Test ERC20", "TERC20")
                )
            )
        );
        ibcImplB.ics20Transfer().setCustomERC20(expDenomPath, customERC20);

        address user = integrationEnv.createAndFundUser(amount);
        address receiver = integrationEnv.createUser();

        IICS26RouterMsgs.Packet memory sentPacket =
            ibcImplA.sendTransferAsUser(integrationEnv.erc20(), user, Strings.toHexString(receiver), amount);

        // run the receive packet queries
        assertFalse(ibcImplB.relayerHelper().isPacketReceived(sentPacket));
        assertFalse(ibcImplB.relayerHelper().isPacketReceiveSuccessful(sentPacket));

        // Receive the packet on B
        bytes[] memory acks = ibcImplB.recvPacket(sentPacket);
        assertEq(acks.length, 1, "ack length mismatch");
        assertEq(acks, th.SINGLE_SUCCESS_ACK(), "ack mismatch");

        // run the receive packet queries
        assert(ibcImplB.relayerHelper().isPacketReceived(sentPacket));
        assert(ibcImplB.relayerHelper().isPacketReceiveSuccessful(sentPacket));

        // Verify that the packet acknowledgement was written correctly
        bytes32 path = ICS24Host.packetAcknowledgementCommitmentKeyCalldata(sentPacket.destClient, sentPacket.sequence);
        bytes32 expAckCommitment = ICS24Host.packetAcknowledgementCommitmentBytes32(th.SINGLE_SUCCESS_ACK());
        bytes32 storedAckCommitment = ibcImplB.ics26Router().getCommitment(path);
        assertEq(storedAckCommitment, expAckCommitment, "ack commitment mismatch");

        // Verify that the packet receipt was set correctly
        bytes32 receiptPath =
            keccak256(ICS24Host.packetReceiptCommitmentPathCalldata(sentPacket.destClient, sentPacket.sequence));
        bytes32 expReceipt = ICS24Host.packetReceiptCommitmentBytes32(sentPacket);
        bytes32 storedReceipt = ibcImplB.ics26Router().getCommitment(receiptPath);
        assertEq(storedReceipt, expReceipt, "receipt mismatch");

        IERC20 token = IERC20(ibcImplB.ics20Transfer().ibcERC20Contract(expDenomPath));
        assertEq(address(token), customERC20, "custom token address mismatch");
        assertEq(token.balanceOf(receiver), amount, "receiver balance mismatch");

        // Check replay protection
        IICS26RouterMsgs.MsgRecvPacket memory msgRecvPacket;
        msgRecvPacket.packet = sentPacket;
        vm.recordLogs();
        ibcImplB.ics26Router().recvPacket(msgRecvPacket);
        th.getValueFromEvent(IICS26Router.Noop.selector);
    }

    function testFuzz_success_foreign_recvICS20Packet(uint256 amount) public {
        // We will send a packet from A to B and then receive it on B
        vm.assume(amount > 0);

        address user = integrationEnv.createUser();
        IERC20 ibcERC20 = setup_createForeignDenomOnImplA(user, amount);
        address receiver = integrationEnv.createUser();

        IICS26RouterMsgs.Packet memory sentPacket =
            ibcImplA.sendTransferAsUser(ibcERC20, user, Strings.toHexString(receiver), amount);

        // run the receive packet queries
        assertFalse(ibcImplB.relayerHelper().isPacketReceived(sentPacket));
        assertFalse(ibcImplB.relayerHelper().isPacketReceiveSuccessful(sentPacket));

        // Receive the packet on B
        bytes[] memory acks = ibcImplB.recvPacket(sentPacket);
        assertEq(acks.length, 1, "ack length mismatch");
        assertEq(acks, th.SINGLE_SUCCESS_ACK(), "ack mismatch");

        // run the receive packet queries
        assert(ibcImplB.relayerHelper().isPacketReceived(sentPacket));
        assert(ibcImplB.relayerHelper().isPacketReceiveSuccessful(sentPacket));

        // Verify that the packet acknowledgement was written correctly
        bytes32 path = ICS24Host.packetAcknowledgementCommitmentKeyCalldata(sentPacket.destClient, sentPacket.sequence);
        bytes32 expAckCommitment = ICS24Host.packetAcknowledgementCommitmentBytes32(th.SINGLE_SUCCESS_ACK());
        bytes32 storedAckCommitment = ibcImplB.ics26Router().getCommitment(path);
        assertEq(storedAckCommitment, expAckCommitment, "ack commitment mismatch");

        // Verify that the packet receipt was set correctly
        bytes32 receiptPath =
            keccak256(ICS24Host.packetReceiptCommitmentPathCalldata(sentPacket.destClient, sentPacket.sequence));
        bytes32 expReceipt = ICS24Host.packetReceiptCommitmentBytes32(sentPacket);
        bytes32 storedReceipt = ibcImplB.ics26Router().getCommitment(receiptPath);
        assertEq(storedReceipt, expReceipt, "receipt mismatch");

        // Check that the receiver got the tokens
        assertEq(integrationEnv.erc20().balanceOf(receiver), amount, "receiver balance mismatch");
        uint256 supplyAfterSend = ibcERC20.totalSupply();
        assertEq(supplyAfterSend, 0); // Burned

        // Check replay protection
        IICS26RouterMsgs.MsgRecvPacket memory msgRecvPacket;
        msgRecvPacket.packet = sentPacket;
        vm.recordLogs();
        ibcImplB.ics26Router().recvPacket(msgRecvPacket);
        th.getValueFromEvent(IICS26Router.Noop.selector);
    }

    function testFuzz_success_native_ackPacket(uint256 amount) public {
        // We will send a packet from A to B and then receive it on B
        vm.assume(amount > 0);

        address user = integrationEnv.createAndFundUser(amount);
        address receiver = integrationEnv.createUser();

        IICS26RouterMsgs.Packet memory sentPacket =
            ibcImplA.sendTransferAsUser(integrationEnv.erc20(), user, Strings.toHexString(receiver), amount);
        bytes[] memory acks = ibcImplB.recvPacket(sentPacket);

        // Acknowledge the packet on A
        ibcImplA.ackPacket(sentPacket, acks);

        // Verify that the packet commitment was deleted
        bytes32 path = ICS24Host.packetCommitmentKeyCalldata(sentPacket.sourceClient, sentPacket.sequence);
        bytes32 storedCommitment = ibcImplA.ics26Router().getCommitment(path);
        assertTrue(storedCommitment == 0, "packet commitment not deleted");

        // Verify that the tokens were transferred
        assertEq(integrationEnv.erc20().balanceOf(user), 0, "user balance mismatch");

        // Check replay protection
        vm.expectEmit();
        emit IICS26Router.Noop();
        ibcImplA.ackPacket(sentPacket, acks);
    }

    function testFuzz_success_foreign_ackPacket(uint256 amount) public {
        // We will send a packet from A to B and then receive it on B
        vm.assume(amount > 0);

        address user = integrationEnv.createUser();
        IERC20 ibcERC20 = setup_createForeignDenomOnImplA(user, amount);
        address receiver = integrationEnv.createUser();

        IICS26RouterMsgs.Packet memory sentPacket =
            ibcImplA.sendTransferAsUser(ibcERC20, user, Strings.toHexString(receiver), amount);
        bytes[] memory acks = ibcImplB.recvPacket(sentPacket);

        // Acknowledge the packet on A
        ibcImplA.ackPacket(sentPacket, acks);

        // Verify that the packet commitment was deleted
        bytes32 path = ICS24Host.packetCommitmentKeyCalldata(sentPacket.sourceClient, sentPacket.sequence);
        bytes32 storedCommitment = ibcImplA.ics26Router().getCommitment(path);
        assertTrue(storedCommitment == 0, "packet commitment not deleted");

        // Verify that the tokens were transferred
        assertEq(ibcERC20.balanceOf(user), 0, "user balance mismatch");

        // Check replay protection
        vm.expectEmit();
        emit IICS26Router.Noop();
        ibcImplA.ackPacket(sentPacket, acks);
    }

    function testFuzz_success_native_errAckPacket(uint256 amount) public {
        // We will send a packet from A to B and then receive it on B
        vm.assume(amount > 0);

        address user = integrationEnv.createAndFundUser(amount);
        string memory invalidReceiver = th.INVALID_ID();

        IICS26RouterMsgs.Packet memory sentPacket =
            ibcImplA.sendTransferAsUser(integrationEnv.erc20(), user, invalidReceiver, amount);
        bytes[] memory acks = ibcImplB.recvPacket(sentPacket);

        // Acknowledge the packet on A
        ibcImplA.ackPacket(sentPacket, acks);

        // Verify that the packet commitment was deleted
        bytes32 path = ICS24Host.packetCommitmentKeyCalldata(sentPacket.sourceClient, sentPacket.sequence);
        bytes32 storedCommitment = ibcImplA.ics26Router().getCommitment(path);
        assertTrue(storedCommitment == 0, "packet commitment not deleted");

        // Verify that the tokens were refunded
        assertEq(integrationEnv.erc20().balanceOf(user), amount, "user balance mismatch");

        // Check replay protection
        vm.expectEmit();
        emit IICS26Router.Noop();
        ibcImplA.ackPacket(sentPacket, acks);
    }

    function testFuzz_success_foreign_errAckPacket(uint256 amount) public {
        // We will send a packet from A to B and then receive it on B
        vm.assume(amount > 0);

        address user = integrationEnv.createUser();
        IERC20 ibcERC20 = setup_createForeignDenomOnImplA(user, amount);
        string memory invalidReceiver = th.INVALID_ID();

        IICS26RouterMsgs.Packet memory sentPacket = ibcImplA.sendTransferAsUser(ibcERC20, user, invalidReceiver, amount);
        bytes[] memory acks = ibcImplB.recvPacket(sentPacket);

        // Acknowledge the packet on A
        ibcImplA.ackPacket(sentPacket, acks);

        // Verify that the packet commitment was deleted
        bytes32 path = ICS24Host.packetCommitmentKeyCalldata(sentPacket.sourceClient, sentPacket.sequence);
        bytes32 storedCommitment = ibcImplA.ics26Router().getCommitment(path);
        assertTrue(storedCommitment == 0, "packet commitment not deleted");

        // Verify that the tokens were refunded
        assertEq(ibcERC20.balanceOf(user), amount, "user balance mismatch");

        // Check replay protection
        vm.expectEmit();
        emit IICS26Router.Noop();
        ibcImplA.ackPacket(sentPacket, acks);
    }

    function testFuzz_success_native_timeoutPacket(uint256 amount) public {
        // We will send a packet from A to B and then time it out on A
        vm.assume(amount > 0);

        address user = integrationEnv.createAndFundUser(amount);
        address receiver = integrationEnv.createUser();

        IICS26RouterMsgs.Packet memory sentPacket = ibcImplA.sendTransferAsUser(
            integrationEnv.erc20(), user, Strings.toHexString(receiver), amount, uint64(block.timestamp + 10 seconds)
        );

        // Set the block timestamp to the timeout
        vm.warp(block.timestamp + 30 seconds);

        // Fail to receive the packet on Chain B
        vm.expectRevert(
            abi.encodeWithSelector(
                IICS26RouterErrors.IBCInvalidTimeoutTimestamp.selector, sentPacket.timeoutTimestamp, block.timestamp
            )
        );
        ibcImplB.recvPacket(sentPacket);

        // Timeout the packet on Chain A
        ibcImplA.timeoutPacket(sentPacket);

        // commitment should be deleted
        bytes32 storedCommitment =
            ibcImplA.relayerHelper().queryPacketCommitment(sentPacket.sourceClient, sentPacket.sequence);
        assertEq(storedCommitment, 0);

        // transfer should be reverted
        uint256 senderBalanceAfterTimeout = integrationEnv.erc20().balanceOf(user);
        uint256 contractBalanceAfterTimeout =
            integrationEnv.erc20().balanceOf(ibcImplA.ics20Transfer().getEscrow(th.FIRST_CLIENT_ID()));
        assertEq(senderBalanceAfterTimeout, amount);
        assertEq(contractBalanceAfterTimeout, 0);

        // Check replay protection
        vm.expectEmit();
        emit IICS26Router.Noop();
        ibcImplA.timeoutPacket(sentPacket);
    }

    function testFuzz_success_foreign_timeoutPacket(uint256 amount) public {
        // We will send a packet from A to B and then time it out on A
        vm.assume(amount > 0);

        address user = integrationEnv.createUser();
        IERC20 ibcERC20 = setup_createForeignDenomOnImplA(user, amount);
        address receiver = integrationEnv.createUser();

        IICS26RouterMsgs.Packet memory sentPacket = ibcImplA.sendTransferAsUser(
            ibcERC20, user, Strings.toHexString(receiver), amount, uint64(block.timestamp + 10 seconds)
        );

        // Set the block timestamp to the timeout
        vm.warp(block.timestamp + 30 seconds);

        // Fail to receive the packet on Chain B
        vm.expectRevert(
            abi.encodeWithSelector(
                IICS26RouterErrors.IBCInvalidTimeoutTimestamp.selector, sentPacket.timeoutTimestamp, block.timestamp
            )
        );
        ibcImplB.recvPacket(sentPacket);

        // Timeout the packet on Chain A
        ibcImplA.timeoutPacket(sentPacket);

        // commitment should be deleted
        bytes32 storedCommitment =
            ibcImplA.relayerHelper().queryPacketCommitment(sentPacket.sourceClient, sentPacket.sequence);
        assertEq(storedCommitment, 0);

        // transfer should be reverted
        uint256 senderBalanceAfterTimeout = ibcERC20.balanceOf(user);
        uint256 contractBalanceAfterTimeout =
            ibcERC20.balanceOf(ibcImplA.ics20Transfer().getEscrow(th.FIRST_CLIENT_ID()));
        assertEq(senderBalanceAfterTimeout, amount);
        assertEq(contractBalanceAfterTimeout, 0);

        // Check replay protection
        vm.expectEmit();
        emit IICS26Router.Noop();
        ibcImplA.timeoutPacket(sentPacket);
    }

    function test_success_sendGmp() public {
        address user = integrationEnv.createUser();
        string memory receiver = th.randomString();

        bytes memory mockPayload = bytes("mock");
        IICS26RouterMsgs.Packet memory sentPacket = ibcImplA.sendGmpAsUser(user, receiver, mockPayload);

        // check the fields of the packet
        assertEq(sentPacket.sourceClient, th.FIRST_CLIENT_ID(), "source client mismatch");
        assertEq(sentPacket.destClient, th.FIRST_CLIENT_ID(), "dest client mismatch");
        assertEq(sentPacket.timeoutTimestamp, th.DEFAULT_TIMEOUT_TIMESTAMP(), "timeout timestamp mismatch");
        assertEq(sentPacket.payloads.length, 1, "payload length mismatch");
        assertEq(sentPacket.payloads[0].sourcePort, ICS27Lib.DEFAULT_PORT_ID, "source port mismatch");
        assertEq(sentPacket.payloads[0].destPort, ICS27Lib.DEFAULT_PORT_ID, "dest port mismatch");
        assertEq(sentPacket.payloads[0].version, ICS27Lib.ICS27_VERSION, "version mismatch");
        assertEq(sentPacket.payloads[0].encoding, ICS27Lib.ICS27_ENCODING, "encoding mismatch");

        IICS27GMPMsgs.GMPPacketData memory gmpPacketData =
            abi.decode(sentPacket.payloads[0].value, (IICS27GMPMsgs.GMPPacketData));
        assertEq(gmpPacketData.sender, Strings.toChecksumHexString(user), "sender mismatch");
        assertEq(gmpPacketData.receiver, receiver, "receiver mismatch");
        assertEq(gmpPacketData.payload, mockPayload, "payload mismatch");
        assertEq(gmpPacketData.salt, bytes(""), "salt mismatch");
        assertEq(gmpPacketData.memo, "", "memo mismatch");

        // check that the packet was committed correctly
        bytes32 path = ICS24Host.packetCommitmentKeyCalldata(sentPacket.sourceClient, sentPacket.sequence);
        bytes32 expCommitment = ICS24Host.packetCommitmentBytes32(sentPacket);
        bytes32 storedCommitment = ibcImplA.ics26Router().getCommitment(path);
        assertEq(storedCommitment, expCommitment, "packet commitment mismatch");
    }

    function testFuzz_success_fullGmp(uint16 saltLen) public {
        address user = integrationEnv.createUser();
        address receiver = makeAddr("receiver");

        // any function call as payload
        bytes memory payload = abi.encodeCall(ILightClient.misbehaviour, (bytes("any")));
        bytes memory callResp = bytes("any response");
        vm.mockCall(receiver, payload, callResp);

        // precompute account address
        IICS27GMPMsgs.AccountIdentifier memory accountId = IICS27GMPMsgs.AccountIdentifier({
            clientId: th.FIRST_CLIENT_ID(), sender: Strings.toChecksumHexString(user), salt: vm.randomBytes(saltLen)
        });
        address computedAccount = ibcImplB.ics27Gmp().getOrComputeAccountAddress(accountId);

        // send packet
        IICS26RouterMsgs.Packet memory sentPacket =
            ibcImplA.sendGmpAsUser(user, Strings.toChecksumHexString(receiver), payload, accountId.salt);

        // Receive the packet on B
        vm.expectCall(receiver, 0, payload);
        bytes[] memory acks = ibcImplB.recvPacket(sentPacket);
        assertEq(acks.length, 1, "ack length mismatch");
        assertEq(acks[0], ICS27Lib.acknowledgement(callResp), "ack mismatch");

        // run the receive packet queries
        assert(ibcImplB.relayerHelper().isPacketReceived(sentPacket));
        assert(ibcImplB.relayerHelper().isPacketReceiveSuccessful(sentPacket));

        // Verify that the account has been created
        address storedAccount = ibcImplB.ics27Gmp().getOrComputeAccountAddress(accountId);
        assertEq(storedAccount, address(computedAccount), "account address mismatch");
        assertEq(IICS27Account(computedAccount).ics27(), address(ibcImplB.ics27Gmp()), "account nor deployed");

        // Acknowledge the packet on A
        ibcImplA.ackPacket(sentPacket, acks);

        // commitment should be deleted
        bytes32 storedCommitment =
            ibcImplA.relayerHelper().queryPacketCommitment(sentPacket.sourceClient, sentPacket.sequence);
        assertEq(storedCommitment, 0);
    }

    function testFuzz_success_errorGmp(uint16 saltLen) public {
        address user = integrationEnv.createUser();
        address receiver = makeAddr("receiver");

        // any function call as payload
        bytes memory payload = abi.encodeCall(ILightClient.misbehaviour, (bytes("any")));
        bytes memory callResp = bytes("any response");
        vm.mockCallRevert(receiver, payload, callResp);

        // precompute account address
        IICS27GMPMsgs.AccountIdentifier memory accountId = IICS27GMPMsgs.AccountIdentifier({
            clientId: th.FIRST_CLIENT_ID(), sender: Strings.toChecksumHexString(user), salt: vm.randomBytes(saltLen)
        });

        // send packet
        IICS26RouterMsgs.Packet memory sentPacket =
            ibcImplA.sendGmpAsUser(user, Strings.toHexString(receiver), payload, accountId.salt);

        // Receive the packet on B
        vm.expectCall(receiver, 0, payload);
        bytes[] memory acks = ibcImplB.recvPacket(sentPacket);
        assertEq(acks.length, 1, "ack length mismatch");
        assertEq(acks, th.SINGLE_ERROR_ACK(), "ack mismatch");

        // run the receive packet queries
        assert(ibcImplB.relayerHelper().isPacketReceived(sentPacket));
        assertFalse(ibcImplB.relayerHelper().isPacketReceiveSuccessful(sentPacket));

        // Acknowledge the packet on A
        ibcImplA.ackPacket(sentPacket, acks);

        // commitment should be deleted
        bytes32 storedCommitment =
            ibcImplA.relayerHelper().queryPacketCommitment(sentPacket.sourceClient, sentPacket.sequence);
        assertEq(storedCommitment, 0);
    }

    function testFuzz_success_timeoutGmp(uint16 saltLen) public {
        address user = integrationEnv.createUser();
        address receiver = makeAddr("receiver");

        // any function call as payload
        bytes memory payload = abi.encodeCall(ILightClient.misbehaviour, (bytes("any")));
        bytes memory callResp = bytes("any response");
        vm.mockCallRevert(receiver, payload, callResp);

        // precompute account address
        IICS27GMPMsgs.AccountIdentifier memory accountId = IICS27GMPMsgs.AccountIdentifier({
            clientId: th.FIRST_CLIENT_ID(), sender: Strings.toChecksumHexString(user), salt: vm.randomBytes(saltLen)
        });

        // send packet
        uint64 timeoutTimestamp = uint64(block.timestamp + 10 seconds);
        IICS26RouterMsgs.Packet memory sentPacket =
            ibcImplA.sendGmpAsUser(user, Strings.toHexString(receiver), payload, accountId.salt, "", timeoutTimestamp);

        // Set the block timestamp to the timeout
        vm.warp(block.timestamp + 30 seconds);

        // Fail to receive the packet on Chain B
        vm.expectRevert(
            abi.encodeWithSelector(
                IICS26RouterErrors.IBCInvalidTimeoutTimestamp.selector, sentPacket.timeoutTimestamp, block.timestamp
            )
        );
        ibcImplB.recvPacket(sentPacket);

        // run the receive packet queries
        assertFalse(ibcImplB.relayerHelper().isPacketReceived(sentPacket));
        assertFalse(ibcImplB.relayerHelper().isPacketReceiveSuccessful(sentPacket));

        // Timeout the packet on Chain A
        ibcImplA.timeoutPacket(sentPacket);

        // commitment should be deleted
        bytes32 storedCommitment =
            ibcImplA.relayerHelper().queryPacketCommitment(sentPacket.sourceClient, sentPacket.sequence);
        assertEq(storedCommitment, 0);
    }
}
