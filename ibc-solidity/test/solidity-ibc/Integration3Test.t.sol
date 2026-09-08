// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable custom-errors,max-line-length,max-states-count

import { Test } from "forge-std/Test.sol";

import { IICS26RouterMsgs } from "../../contracts/msgs/IICS26RouterMsgs.sol";

import { IERC20 } from "@openzeppelin-contracts/token/ERC20/IERC20.sol";

import { IbcImpl } from "./utils/IbcImpl.sol";
import { TestHelper } from "./utils/TestHelper.sol";
import { IntegrationEnv } from "./utils/IntegrationEnv.sol";
import { Strings } from "@openzeppelin-contracts/utils/Strings.sol";
import { ICS20Lib } from "../../contracts/utils/ICS20Lib.sol";

contract Integration3Test is Test {
    IbcImpl public ibcImplA;
    IbcImpl public ibcImplB;
    IbcImpl public ibcImplC;
    // ibcImplA <--> ibcImplB <--> ibcImplC

    TestHelper public th = new TestHelper();
    IntegrationEnv public integrationEnv;

    function setUp() public {
        // Set up the environment
        integrationEnv = new IntegrationEnv();

        // Deploy the IBC implementation
        ibcImplA = new IbcImpl(integrationEnv.permit2());
        ibcImplB = new IbcImpl(integrationEnv.permit2());
        ibcImplC = new IbcImpl(integrationEnv.permit2());

        // Add the counterparty implementations
        string memory clientId;
        clientId = ibcImplA.addCounterpartyImpl(ibcImplB, th.FIRST_CLIENT_ID());
        assertEq(clientId, th.FIRST_CLIENT_ID());

        clientId = ibcImplB.addCounterpartyImpl(ibcImplA, th.FIRST_CLIENT_ID());
        assertEq(clientId, th.FIRST_CLIENT_ID());

        clientId = ibcImplB.addCounterpartyImpl(ibcImplC, th.FIRST_CLIENT_ID());
        assertEq(clientId, th.SECOND_CLIENT_ID());

        clientId = ibcImplC.addCounterpartyImpl(ibcImplB, th.SECOND_CLIENT_ID());
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
        assertEq(
            ibcImplB.ics26Router().getClient(th.SECOND_CLIENT_ID()).getClientState(),
            abi.encodePacked(address(ibcImplC.ics26Router()))
        );
        assertEq(
            ibcImplC.ics26Router().getClient(th.FIRST_CLIENT_ID()).getClientState(),
            abi.encodePacked(address(ibcImplB.ics26Router()))
        );
    }

    // solhint-disable-next-line function-max-lines
    function testFuzz_success_forwardAndBack(uint256 amount) public {
        // There are three chains in this scenario: A -> B -> C
        vm.assume(amount > 0);

        // Send from A -> B
        address user = integrationEnv.createAndFundUser(amount);
        address receiverB = integrationEnv.createUser();

        IICS26RouterMsgs.Packet memory sentPacket =
            ibcImplA.sendTransferAsUser(integrationEnv.erc20(), user, Strings.toHexString(receiverB), amount);
        assertEq(integrationEnv.erc20().balanceOf(user), 0, "sender balance mismatch");

        // Receive the packet on B
        bytes[] memory acks = ibcImplB.recvPacket(sentPacket);
        assertEq(acks.length, 1, "ack length mismatch");
        assertEq(acks, th.SINGLE_SUCCESS_ACK(), "ack mismatch");

        // run the receive packet queries
        assert(ibcImplB.relayerHelper().isPacketReceived(sentPacket));
        assert(ibcImplB.relayerHelper().isPacketReceiveSuccessful(sentPacket));

        // Check that a new IBCERC20 token was created
        string memory expDenomPath = string.concat(
            ICS20Lib.DEFAULT_PORT_ID,
            "/",
            th.FIRST_CLIENT_ID(),
            "/",
            Strings.toHexString(address(integrationEnv.erc20()))
        );
        IERC20 tokenOnB = IERC20(ibcImplB.ics20Transfer().ibcERC20Contract(expDenomPath));
        assertTrue(address(tokenOnB) != address(0), "IBCERC20 token not found");
        assertEq(tokenOnB.balanceOf(receiverB), amount, "receiver balance mismatch");

        // ack the packet on A for completion
        ibcImplA.ackPacket(sentPacket, acks);

        // Send from B -> C
        address receiverC = integrationEnv.createUser();
        sentPacket = ibcImplB.sendTransferAsUser(
            tokenOnB, receiverB, Strings.toHexString(receiverC), amount, th.SECOND_CLIENT_ID()
        );
        assertEq(tokenOnB.balanceOf(receiverB), 0, "sender balance mismatch");

        // Receive the packet on C
        acks = ibcImplC.recvPacket(sentPacket);
        assertEq(acks.length, 1, "ack length mismatch");
        assertEq(acks, th.SINGLE_SUCCESS_ACK(), "ack mismatch");

        // run the receive packet queries
        assert(ibcImplC.relayerHelper().isPacketReceived(sentPacket));
        assert(ibcImplC.relayerHelper().isPacketReceiveSuccessful(sentPacket));

        // Check that a new IBCERC20 token was created
        string memory expDenomPath2 = string.concat(
            ICS20Lib.DEFAULT_PORT_ID,
            "/",
            th.FIRST_CLIENT_ID(),
            "/",
            ICS20Lib.DEFAULT_PORT_ID,
            "/",
            th.FIRST_CLIENT_ID(),
            "/",
            Strings.toHexString(address(integrationEnv.erc20()))
        );
        IERC20 tokenOnC = IERC20(ibcImplC.ics20Transfer().ibcERC20Contract(expDenomPath2));
        assertTrue(address(tokenOnC) != address(0), "IBCERC20 token not found");
        assertEq(tokenOnC.balanceOf(receiverC), amount, "receiver balance mismatch");

        // ack the packet on B for completion
        ibcImplB.ackPacket(sentPacket, acks);

        // Transfer the tokens back: C -> B -> A

        // Send from C -> B
        sentPacket = ibcImplC.sendTransferAsUser(tokenOnC, receiverC, Strings.toHexString(receiverB), amount);
        assertEq(tokenOnC.balanceOf(receiverC), 0, "sender balance mismatch");

        // Receive the packet on B
        acks = ibcImplB.recvPacket(sentPacket);
        assertEq(acks.length, 1, "ack length mismatch");
        assertEq(acks, th.SINGLE_SUCCESS_ACK(), "ack mismatch");

        // run the receive packet queries
        assert(ibcImplB.relayerHelper().isPacketReceived(sentPacket));
        assert(ibcImplB.relayerHelper().isPacketReceiveSuccessful(sentPacket));

        // Check that the IBCERC20 token is the same as before
        assertTrue(address(tokenOnB) != address(0), "IBCERC20 token not found");
        assertEq(tokenOnB.balanceOf(receiverB), amount, "receiver balance mismatch");

        // ack the packet on C for completion
        ibcImplC.ackPacket(sentPacket, acks);

        // Send from B -> A
        sentPacket = ibcImplB.sendTransferAsUser(tokenOnB, receiverB, Strings.toHexString(user), amount);
        assertEq(tokenOnB.balanceOf(receiverB), 0, "sender balance mismatch");

        // Receive the packet on A
        acks = ibcImplA.recvPacket(sentPacket);
        assertEq(acks.length, 1, "ack length mismatch");
        assertEq(acks, th.SINGLE_SUCCESS_ACK(), "ack mismatch");

        // run the receive packet queries
        assert(ibcImplA.relayerHelper().isPacketReceived(sentPacket));
        assert(ibcImplA.relayerHelper().isPacketReceiveSuccessful(sentPacket));

        // Receive the original tokens
        assertEq(integrationEnv.erc20().balanceOf(user), amount, "sender balance mismatch");
        assertEq(tokenOnB.totalSupply(), 0, "totalSupply mismatch");
        assertEq(tokenOnC.totalSupply(), 0, "totalSupply mismatch");
    }
}
