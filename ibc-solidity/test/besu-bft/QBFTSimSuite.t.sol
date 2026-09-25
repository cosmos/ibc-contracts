// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { Test } from "forge-std/Test.sol";
import { Strings } from "@openzeppelin-contracts/utils/Strings.sol";

import { IICS02ClientMsgs } from "../../contracts/msgs/IICS02ClientMsgs.sol";
import { IICS26RouterMsgs } from "../../contracts/msgs/IICS26RouterMsgs.sol";
import { ILightClientMsgs } from "../../contracts/msgs/ILightClientMsgs.sol";
import { IBesuLightClient } from "../../contracts/light-clients/besu/interfaces/IBesuLightClient.sol";
import { IBesuLightClientErrors } from "../../contracts/light-clients/besu/errors/IBesuLightClientErrors.sol";
import { IBesuLightClientMsgs } from "../../contracts/light-clients/besu/msgs/IBesuLightClientMsgs.sol";
import { ICS24Host } from "../../contracts/utils/ICS24Host.sol";

import { IbcImpl } from "../solidity-ibc/utils/IbcImpl.sol";
import { IntegrationEnv } from "../solidity-ibc/utils/IntegrationEnv.sol";
import { TestHelper } from "../solidity-ibc/utils/TestHelper.sol";
import { QBFTSimSuite } from "./utils/QBFTSimSuite.sol";
import { SimHeader } from "./utils/SimHeader.sol";

contract QBFTSimSuiteTest is Test {
    uint64 internal constant TRUSTING_PERIOD = 1 days;
    uint64 internal constant MAX_CLOCK_DRIFT = 10;

    QBFTSimSuite internal sim;
    IBesuLightClient internal client;

    function setUp() public {
        sim = _setUp(SimHeader.Mode.QBFT, 4);
    }

    function _setUp(SimHeader.Mode mode, uint256 validatorCount) internal returns (QBFTSimSuite s) {
        s = new QBFTSimSuite(mode);
        s.addValidators(validatorCount);
        s.produceBlocks(2);
        client = s.deployLightClient(TRUSTING_PERIOD, MAX_CLOCK_DRIFT);
    }

    function test_honestChain() public {
        SimHeader.Mode[2] memory modes = [SimHeader.Mode.QBFT, SimHeader.Mode.IBFT2];
        uint256[3] memory counts = [uint256(1), 4, 16];
        for (uint256 m = 0; m < modes.length; ++m) {
            for (uint256 c = 0; c < counts.length; ++c) {
                QBFTSimSuite s = _setUp(modes[m], counts[c]);
                s.produceBlocks(2);
                assertEq(uint8(client.updateClient(s.updateMsg(2, 3))), uint8(ILightClientMsgs.UpdateResult.Update));
                assertEq(uint8(client.updateClient(s.updateMsg(3, 4))), uint8(ILightClientMsgs.UpdateResult.Update));
                assertEq(uint8(client.updateClient(s.updateMsg(2, 4))), uint8(ILightClientMsgs.UpdateResult.NoOp));
            }
        }
    }

    function test_doubleSign() public {
        SimHeader.Data memory a = sim.nextBlock();
        SimHeader.Data memory b = sim.nextBlock();
        b.stateRoot = keccak256("equivocation");
        a = sim.seal(a);
        b = sim.seal(b);
        sim.commit(a);

        bytes memory honest = sim.updateMsg(2, a);
        client.updateClient(honest);
        bytes memory conflicting = sim.updateMsg(2, b);
        assertEq(uint8(client.updateClient(conflicting)), uint8(ILightClientMsgs.UpdateResult.Misbehaviour));

        vm.expectRevert(abi.encodeWithSelector(IBesuLightClientErrors.FrozenClientState.selector));
        client.updateClient(honest);
    }

    /// @dev A header not strictly newer than its trusted consensus state freezes the client instead of updating it.
    function test_timeNonMonotonicityUpdate() public {
        SimHeader.Data memory h = sim.nextBlock();
        h.timestamp = sim.blockAt(2).timestamp;
        sim.commit(sim.seal(h));
        bytes memory update = sim.updateMsg(2, 3);

        vm.expectEmit(address(client));
        emit IBesuLightClient.TimeNonMonotonicity(3, 2, h.timestamp, h.timestamp);
        assertEq(uint8(client.updateClient(update)), uint8(ILightClientMsgs.UpdateResult.Misbehaviour));

        vm.expectRevert(abi.encodeWithSelector(IBesuLightClientErrors.ConsensusStateNotFound.selector, 3));
        client.getConsensusStateHash(3);
        vm.expectRevert(abi.encodeWithSelector(IBesuLightClientErrors.FrozenClientState.selector));
        client.updateClient(update);
    }

    /// @dev Each update is only checked against its own trusted height, so a forged height-3 header timestamped
    /// after the honest height-4 header is accepted. Submitting both stored states as misbehaviour freezes the client.
    function test_timeNonMonotonicityMisbehaviour() public {
        SimHeader.Data memory forged = sim.nextBlock();
        sim.produceBlocks(2);
        forged.timestamp = sim.blockAt(4).timestamp + 1;
        forged = sim.seal(forged);

        bytes memory honestUpdate = sim.updateMsg(2, 4);
        bytes memory forgedUpdate = sim.updateMsg(2, forged);
        assertEq(uint8(client.updateClient(honestUpdate)), uint8(ILightClientMsgs.UpdateResult.Update));
        assertEq(uint8(client.updateClient(forgedUpdate)), uint8(ILightClientMsgs.UpdateResult.Update));

        IBesuLightClientMsgs.ConsensusState memory honestState = sim.consensusState(4);
        bytes memory misbehaviour = abi.encode(
            IBesuLightClientMsgs.MsgTimeNonMonotonicityMisbehaviour({
                height1: IICS02ClientMsgs.Height(0, 3),
                height2: IICS02ClientMsgs.Height(0, 4),
                consensusStatePreimage1: IBesuLightClientMsgs.ConsensusState(
                    forged.timestamp, forged.stateRoot, forged.validators
                ),
                consensusStatePreimage2: honestState
            })
        );

        vm.expectEmit(address(client));
        emit IBesuLightClient.TimeNonMonotonicity(4, 3, honestState.timestamp, forged.timestamp);
        client.misbehaviour(misbehaviour);

        vm.expectRevert(abi.encodeWithSelector(IBesuLightClientErrors.FrozenClientState.selector));
        client.misbehaviour(misbehaviour);
    }

    function test_timestampFromFuture() public {
        SimHeader.Data memory h = sim.nextBlock();
        h.timestamp = uint64(vm.getBlockTimestamp()) + MAX_CLOCK_DRIFT + 1;
        bytes memory update = sim.updateMsg(2, sim.seal(h));
        vm.expectRevert(
            abi.encodeWithSelector(
                IBesuLightClientErrors.HeaderFromFuture.selector, vm.getBlockTimestamp(), h.timestamp, MAX_CLOCK_DRIFT
            )
        );
        client.updateClient(update);
    }

    function test_lowQuorumAndUnknownSigner() public {
        address[] memory two = new address[](2);
        (two[0], two[1]) = (sim.validators()[0], sim.validators()[1]);
        bytes memory update = sim.updateMsg(2, sim.seal(sim.nextBlock(), two));
        vm.expectRevert(
            abi.encodeWithSelector(IBesuLightClientErrors.InsufficientTrustedValidatorOverlap.selector, 2, 3)
        );
        client.updateClient(update);

        uint256[] memory keys = new uint256[](4);
        for (uint256 i = 0; i < 3; ++i) {
            keys[i] = sim.validatorKey(sim.validators()[i]);
        }
        keys[3] = 0xdead;
        update = sim.updateMsg(2, sim.sealWithKeys(sim.nextBlock(), keys));
        vm.expectRevert(
            abi.encodeWithSelector(IBesuLightClientErrors.UnknownCommitSealSigner.selector, vm.addr(0xdead))
        );
        client.updateClient(update);
    }

    function test_validatorChurn() public {
        sim.removeValidator(sim.validators()[0]);
        sim.addValidator();
        sim.produceBlock();
        client.updateClient(sim.updateMsg(2, 3));
        assertEq(sim.blockAt(3).validators.length, 4);

        for (uint256 i = 0; i < 4; ++i) {
            sim.removeValidator(sim.validators()[0]);
        }
        sim.addValidators(4);
        sim.produceBlock();
        bytes memory update = sim.updateMsg(3, 4);
        vm.expectRevert(
            abi.encodeWithSelector(IBesuLightClientErrors.InsufficientTrustedValidatorOverlap.selector, 0, 3)
        );
        client.updateClient(update);
    }

    function test_packetProofs() public {
        IICS26RouterMsgs.Packet memory packet;
        packet.sequence = 7;
        packet.sourceClient = "client-0";
        packet.destClient = "client-1";
        packet.payloads = new IICS26RouterMsgs.Payload[](1);
        packet.payloads[0] = IICS26RouterMsgs.Payload("transfer", "transfer", "1", "json", "hello");
        bytes memory commitmentPath = ICS24Host.packetCommitmentPathCalldata(packet.sourceClient, packet.sequence);
        bytes memory receiptPath = ICS24Host.packetReceiptCommitmentPathCalldata(packet.destClient, packet.sequence);

        sim.commitPacket(packet);
        sim.produceBlock();
        client.updateClient(sim.updateMsg(2, 3));

        assertEq(client.verifyMembership(sim.membershipMsg(3, commitmentPath)), sim.blockAt(3).timestamp);
        assertEq(client.verifyNonMembership(sim.nonMembershipMsg(3, receiptPath)), sim.blockAt(3).timestamp);
        client.verifyNonMembership(sim.nonMembershipMsg(2, commitmentPath));

        sim.setCommitment(commitmentPath, bytes32(0));
        sim.produceBlock();
        client.updateClient(sim.updateMsg(3, 4));
        client.verifyNonMembership(sim.nonMembershipMsg(4, commitmentPath));
        client.verifyMembership(sim.membershipMsg(3, commitmentPath));
    }

    /// @dev Accounts added later must not leak into the state trie rebuilt for earlier heights.
    function test_historicalProofsExcludeLaterAccounts() public {
        bytes memory path = ICS24Host.packetCommitmentPathCalldata("client-0", 1);
        sim.setCommitment(path, keccak256("commitment"));
        sim.produceBlock();
        client.updateClient(sim.updateMsg(2, 3));

        sim.addAccounts(3);
        sim.produceBlock();

        assertEq(sim.stateRootAt(3), sim.blockAt(3).stateRoot);
        assertEq(client.verifyMembership(sim.membershipMsg(3, path)), sim.blockAt(3).timestamp);
    }

    /// @dev Mirrors a commitment written by a real ICS26 router and proves it through the light client.
    function test_syncCommitmentFromRouter() public {
        TestHelper th = new TestHelper();
        IntegrationEnv env = new IntegrationEnv();
        IbcImpl ibcImpl = new IbcImpl(env.permit2());
        ibcImpl.addCounterpartyImpl(ibcImpl, th.FIRST_CLIENT_ID());
        address user = env.createAndFundUser(100);

        IICS26RouterMsgs.Packet memory packet =
            ibcImpl.sendTransferAsUser(env.erc20(), user, Strings.toHexString(user), 100, th.FIRST_CLIENT_ID());
        bytes memory path = ICS24Host.packetCommitmentPathCalldata(packet.sourceClient, packet.sequence);
        sim.syncCommitment(ibcImpl.ics26Router(), path);
        sim.produceBlock();
        client.updateClient(sim.updateMsg(2, 3));

        ILightClientMsgs.MsgVerifyMembership memory msg_ = sim.membershipMsg(3, path);
        assertEq(msg_.value, abi.encodePacked(ICS24Host.packetCommitmentBytes32(packet)));
        client.verifyMembership(msg_);
    }
}
