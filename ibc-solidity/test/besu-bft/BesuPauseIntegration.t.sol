// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { IICS26RouterMsgs } from "../../contracts/msgs/IICS26RouterMsgs.sol";

import { IPausable } from "../../contracts/interfaces/IPausable.sol";
import { ICS26Router } from "../../contracts/ICS26Router.sol";
import { IBCRolesLib } from "../../contracts/utils/IBCRolesLib.sol";
import { ICS24Host } from "../../contracts/utils/ICS24Host.sol";
import { Pausable } from "@openzeppelin-contracts/utils/Pausable.sol";
import { Strings } from "@openzeppelin-contracts/utils/Strings.sol";

import { IbcImpl } from "../solidity-ibc/utils/IbcImpl.sol";
import { IntegrationEnv } from "../solidity-ibc/utils/IntegrationEnv.sol";
import { TestHelper } from "../solidity-ibc/utils/TestHelper.sol";
import { WithBesuQBFTSim } from "./utils/BesuSimDriver.sol";
import { WithFreshChains } from "../solidity-ibc/integration/drivers/LightClientDriverTest.sol";

/// @dev Pending packets relayed through real routers: a paused light client rejects their proofs without changing
/// packet state, and the same messages complete once it is unpaused.
contract BesuPauseIntegrationTest is WithBesuQBFTSim, WithFreshChains {
    uint256 internal constant AMOUNT = 1000;

    IbcImpl internal ibcImplA;
    IbcImpl internal ibcImplB;
    TestHelper internal th = new TestHelper();
    IntegrationEnv internal integrationEnv = new IntegrationEnv();
    address internal pauser = makeAddr("pauser");

    function setUp() public {
        ibcImplA = new IbcImpl(integrationEnv.permit2());
        ibcImplB = new IbcImpl(integrationEnv.permit2());
        ibcImplA.addClient(th.FIRST_CLIENT_ID(), _newDriver(ibcImplA.ics26Router(), ibcImplB.ics26Router()));
        ibcImplB.addClient(th.FIRST_CLIENT_ID(), _newDriver(ibcImplB.ics26Router(), ibcImplA.ics26Router()));

        ibcImplA.accessManager().grantRole(IBCRolesLib.PAUSER_ROLE, pauser, 0);
        ibcImplA.accessManager().grantRole(IBCRolesLib.UNPAUSER_ROLE, pauser, 0);
        ibcImplB.accessManager().grantRole(IBCRolesLib.PAUSER_ROLE, pauser, 0);
        ibcImplB.accessManager().grantRole(IBCRolesLib.UNPAUSER_ROLE, pauser, 0);
    }

    function test_recvAndAckPacketWhilePaused() public {
        ICS26Router routerA = ibcImplA.ics26Router();
        ICS26Router routerB = ibcImplB.ics26Router();
        address user = integrationEnv.createAndFundUser(AMOUNT);
        IICS26RouterMsgs.Packet memory packet = ibcImplA.sendTransferAsUser(
            integrationEnv.erc20(), user, Strings.toHexString(integrationEnv.createUser()), AMOUNT
        );
        bytes32 receiptKey =
            keccak256(ICS24Host.packetReceiptCommitmentPathCalldata(packet.destClient, packet.sequence));

        IICS26RouterMsgs.MsgRecvPacket memory recvMsg = ibcImplB.recvPacketMsg(packet);
        _setPaused(ibcImplB, true);
        vm.expectRevert(Pausable.EnforcedPause.selector);
        routerB.recvPacket(recvMsg);
        assertEq(routerB.getCommitment(receiptKey), 0);

        _setPaused(ibcImplB, false);
        routerB.recvPacket(recvMsg);
        assertNotEq(routerB.getCommitment(receiptKey), 0);

        IICS26RouterMsgs.MsgAckPacket memory ackMsg = ibcImplA.ackPacketMsg(packet, th.SINGLE_SUCCESS_ACK()[0]);
        _setPaused(ibcImplA, true);
        vm.expectRevert(Pausable.EnforcedPause.selector);
        routerA.ackPacket(ackMsg);
        assertEq(_commitment(packet), ICS24Host.packetCommitmentBytes32(packet));

        _setPaused(ibcImplA, false);
        routerA.ackPacket(ackMsg);
        assertEq(_commitment(packet), 0);
    }

    function test_timeoutPacketWhilePaused() public {
        ICS26Router routerA = ibcImplA.ics26Router();
        address user = integrationEnv.createAndFundUser(AMOUNT);
        IICS26RouterMsgs.Packet memory packet = ibcImplA.sendTransferAsUser(
            integrationEnv.erc20(),
            user,
            Strings.toHexString(integrationEnv.createUser()),
            AMOUNT,
            uint64(block.timestamp + 10 seconds)
        );
        vm.warp(block.timestamp + 30 seconds);

        IICS26RouterMsgs.MsgTimeoutPacket memory timeoutMsg = ibcImplA.timeoutPacketMsg(packet);
        _setPaused(ibcImplA, true);
        vm.expectRevert(Pausable.EnforcedPause.selector);
        routerA.timeoutPacket(timeoutMsg);
        assertEq(_commitment(packet), ICS24Host.packetCommitmentBytes32(packet));
        assertEq(integrationEnv.erc20().balanceOf(user), 0);

        _setPaused(ibcImplA, false);
        routerA.timeoutPacket(timeoutMsg);
        assertEq(_commitment(packet), 0);
        assertEq(integrationEnv.erc20().balanceOf(user), AMOUNT);
    }

    function _setPaused(IbcImpl impl, bool paused) internal {
        IPausable client = IPausable(address(impl.ics26Router().getClient(th.FIRST_CLIENT_ID())));
        vm.prank(pauser);
        if (paused) {
            client.pause();
        } else {
            client.unpause();
        }
    }

    function _commitment(IICS26RouterMsgs.Packet memory packet) internal view returns (bytes32) {
        return ibcImplA.ics26Router()
            .getCommitment(ICS24Host.packetCommitmentKeyCalldata(packet.sourceClient, packet.sequence));
    }
}
