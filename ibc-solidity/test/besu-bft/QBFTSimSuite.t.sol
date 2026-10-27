// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { Test } from "forge-std/Test.sol";
import { IAccessControl } from "@openzeppelin-contracts/access/IAccessControl.sol";
import { Math } from "@openzeppelin-contracts/utils/math/Math.sol";
import { Memory } from "@openzeppelin-contracts/utils/Memory.sol";
import { RLP } from "@openzeppelin-contracts/utils/RLP.sol";

import { IICS02ClientMsgs } from "../../contracts/msgs/IICS02ClientMsgs.sol";
import { IICS26RouterMsgs } from "../../contracts/msgs/IICS26RouterMsgs.sol";
import { ILightClientMsgs } from "../../contracts/msgs/ILightClientMsgs.sol";
import { IBesuLightClient } from "../../contracts/light-clients/besu/interfaces/IBesuLightClient.sol";
import { IBesuLightClientMsgs } from "../../contracts/light-clients/besu/msgs/IBesuLightClientMsgs.sol";
import { IBesuLightClientErrors } from "../../contracts/light-clients/besu/errors/IBesuLightClientErrors.sol";
import { BesuLightClientBase } from "../../contracts/light-clients/besu/BesuLightClientBase.sol";
import { ICS24Host } from "../../contracts/utils/ICS24Host.sol";

import { QBFTSimSuite } from "./utils/QBFTSimSuite.sol";
import { SimHeader } from "./utils/SimHeader.sol";

/// @dev Every scenario runs once per consensus mode through `QBFTSimSuiteQBFTTest` and `QBFTSimSuiteIBFT2Test`.
abstract contract QBFTSimSuiteTest is Test {
    uint64 internal constant TRUSTING_PERIOD = 1 days;
    uint64 internal constant MAX_CLOCK_DRIFT = 10;

    QBFTSimSuite internal sim;
    IBesuLightClient internal client;

    function _mode() internal pure virtual returns (SimHeader.Mode);

    function setUp() public {
        sim = _setUp(4);
    }

    function _setUp(uint256 validatorCount) internal returns (QBFTSimSuite s) {
        s = new QBFTSimSuite(_mode());
        s.addValidators(validatorCount);
        s.produceBlocks(2);
        client = s.deployLightClient(TRUSTING_PERIOD, MAX_CLOCK_DRIFT, IBesuLightClientMsgs.TrustThreshold(2, 3));
    }

    function test_honestChain() public {
        uint256[3] memory counts = [uint256(1), 4, 16];
        for (uint256 c = 0; c < counts.length; ++c) {
            QBFTSimSuite s = _setUp(counts[c]);
            s.produceBlocks(2);
            assertEq(uint8(client.updateClient(s.updateMsg(2, 3))), uint8(ILightClientMsgs.UpdateResult.Update));
            assertEq(uint8(client.updateClient(s.updateMsg(3, 4))), uint8(ILightClientMsgs.UpdateResult.Update));
            assertEq(uint8(client.updateClient(s.updateMsg(2, 4))), uint8(ILightClientMsgs.UpdateResult.NoOp));
        }
    }

    /// @dev Filling in an older height after a newer one stores it without moving `latestHeight` back.
    function test_backfillKeepsLatestHeight() public {
        sim.produceBlocks(2);
        client.updateClient(sim.updateMsg(2, 4));
        assertEq(uint8(client.updateClient(sim.updateMsg(2, 3))), uint8(ILightClientMsgs.UpdateResult.Update));

        IBesuLightClientMsgs.ClientState memory state =
            abi.decode(client.getClientState(), (IBesuLightClientMsgs.ClientState));
        assertEq(state.latestHeight.revisionHeight, 4);
        assertEq(client.getConsensusStateHash(3), keccak256(abi.encode(sim.consensusState(3))));
    }

    function test_updateClient_revertMalformedHeader() public {
        SimHeader.Data memory h = sim.nextBlock();
        bytes memory headerRlp = SimHeader.encode(h);

        _expectUpdateRevert(
            _rewriteHeader(headerRlp, 14, type(uint256).max, ""),
            abi.encodeWithSelector(IBesuLightClientErrors.InvalidHeaderFormat.selector, 14)
        );
        _expectUpdateRevert(
            _rewriteHeader(headerRlp, 20, 12, RLP.encode(bytes(hex"c0"))),
            abi.encodeWithSelector(IBesuLightClientErrors.InvalidExtraDataFormat.selector, 0)
        );
    }

    function test_accessControl() public {
        address manager = makeAddr("manager");
        address relayer = makeAddr("relayer");
        IBesuLightClient restricted = sim.deployLightClient(
            TRUSTING_PERIOD, MAX_CLOCK_DRIFT, IBesuLightClientMsgs.TrustThreshold(2, 3), manager
        );
        IAccessControl acl = IAccessControl(address(restricted));
        bytes32 role = BesuLightClientBase(address(restricted)).PROOF_SUBMITTER_ROLE();
        assertTrue(acl.hasRole(acl.getRoleAdmin(role), manager));
        assertTrue(acl.hasRole(role, manager));
        assertFalse(acl.hasRole(role, address(0)));

        sim.produceBlock();
        bytes memory update = sim.updateMsg(2, 3);
        bytes memory unauthorized =
            abi.encodeWithSelector(IAccessControl.AccessControlUnauthorizedAccount.selector, relayer, role);
        ILightClientMsgs.MsgVerifyMembership memory membership;
        ILightClientMsgs.MsgVerifyNonMembership memory nonMembership;

        vm.startPrank(relayer);
        vm.expectRevert(unauthorized);
        restricted.updateClient(update);
        vm.expectRevert(unauthorized);
        restricted.verifyMembership(membership);
        vm.expectRevert(unauthorized);
        restricted.verifyNonMembership(nonMembership);
        vm.stopPrank();
        // misbehaviour is permissionless, see `test_misbehaviourIsPermissionless`.

        vm.prank(manager);
        acl.grantRole(role, relayer);
        vm.prank(relayer);
        assertEq(uint8(restricted.updateClient(update)), uint8(ILightClientMsgs.UpdateResult.Update));

        // Without a role manager, anyone may submit and no one administers the role.
        IAccessControl open = IAccessControl(address(client));
        assertTrue(open.hasRole(role, address(0)));
        assertFalse(open.hasRole(open.getRoleAdmin(role), manager));
        vm.prank(relayer);
        assertEq(uint8(client.updateClient(update)), uint8(ILightClientMsgs.UpdateResult.Update));
    }

    function _expectUpdateRevert(bytes memory headerRlp, bytes memory revertData) internal {
        bytes memory update = abi.encode(
            IBesuLightClientMsgs.MsgUpdateClient(headerRlp, IICS02ClientMsgs.Height(0, 2), sim.consensusState(2))
        );
        vm.expectRevert(revertData);
        client.updateClient(update);
    }

    /// @dev Re-encodes the first `length` items of a header RLP, replacing item `index` with an encoded `item`.
    function _rewriteHeader(
        bytes memory headerRlp,
        uint256 length,
        uint256 index,
        bytes memory item
    )
        internal
        pure
        returns (bytes memory)
    {
        Memory.Slice[] memory items = RLP.decodeList(headerRlp);
        bytes[] memory out = new bytes[](length);
        for (uint256 i = 0; i < length; ++i) {
            out[i] = i == index ? item : Memory.toBytes(items[i]);
        }
        return RLP.encode(out);
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
    /// after the honest height-4 header is accepted. Submitting both stored states as misbehaviour freezes the client,
    /// even after both are past their trusting period.
    function test_conflictingConsensusStatesTimeNonMonotonicity() public {
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
            IBesuLightClientMsgs.MsgSubmitMisbehaviour(
                IBesuLightClientMsgs.MisbehaviourType.ConflictingConsensusStates,
                abi.encode(
                    IBesuLightClientMsgs.MsgConflictingConsensusStatesMisbehaviour({
                        height1: IICS02ClientMsgs.Height(0, 3),
                        height2: IICS02ClientMsgs.Height(0, 4),
                        consensusStatePreimage1: _consensusState(forged),
                        consensusStatePreimage2: honestState
                    })
                )
            )
        );

        vm.warp(honestState.timestamp + TRUSTING_PERIOD);
        vm.expectEmit(address(client));
        emit IBesuLightClient.TimeNonMonotonicity(4, 3, honestState.timestamp, forged.timestamp);
        client.misbehaviour(misbehaviour);

        vm.expectRevert(abi.encodeWithSelector(IBesuLightClientErrors.FrozenClientState.selector));
        client.misbehaviour(misbehaviour);
    }

    /// @dev Two validly signed headers at the same height prove a double sign, whether or not either is stored.
    function test_conflictingHeadersDoubleSign() public {
        SimHeader.Data memory a = sim.seal(sim.nextBlock());
        SimHeader.Data memory b = sim.nextBlock();
        b.stateRoot = keccak256("equivocation");
        b = sim.seal(b);
        bytes memory misbehaviour = _conflictingHeadersMisbehaviour(sim.updateMsg(2, a), sim.updateMsg(2, b));

        uint256 snapshot = vm.snapshotState();
        for (uint256 stored = 0; stored < 2; ++stored) {
            vm.revertToState(snapshot);
            if (stored == 1) {
                client.updateClient(sim.updateMsg(2, a));
            }

            vm.expectEmit(address(client));
            emit IBesuLightClient.DoubleSign(
                3, keccak256(abi.encode(_consensusState(a))), keccak256(abi.encode(_consensusState(b)))
            );
            client.misbehaviour(misbehaviour);
            assertTrue(_isFrozen(client));
        }
    }

    /// @dev Two validly signed headers whose timestamps do not increase with height freeze the client in either
    /// order, even though neither is stored and the forged header is ahead of the clock.
    function test_conflictingHeadersTimeNonMonotonicity() public {
        SimHeader.Data memory forged = sim.nextBlock();
        sim.produceBlocks(2);
        forged.timestamp = uint64(vm.getBlockTimestamp()) + MAX_CLOCK_DRIFT + 1;
        forged = sim.seal(forged);
        bytes memory forgedUpdate = sim.updateMsg(2, forged);
        bytes memory honestUpdate = sim.updateMsg(2, 4);
        uint64 honestTimestamp = sim.blockAt(4).timestamp;

        uint256 snapshot = vm.snapshotState();
        bytes[2] memory misbehaviours = [
            _conflictingHeadersMisbehaviour(forgedUpdate, honestUpdate),
            _conflictingHeadersMisbehaviour(honestUpdate, forgedUpdate)
        ];
        for (uint256 i = 0; i < misbehaviours.length; ++i) {
            vm.revertToState(snapshot);
            vm.expectEmit(address(client));
            emit IBesuLightClient.TimeNonMonotonicity(4, 3, honestTimestamp, forged.timestamp);
            client.misbehaviour(misbehaviours[i]);
            assertTrue(_isFrozen(client));
        }
    }

    /// @dev A header not newer than its own trusted consensus state is evidence on its own, as in `updateClient`,
    /// even when paired with a valid later header or with itself.
    function test_conflictingHeadersTimeNonMonotonicityAgainstTrustedState() public {
        uint64 trustedTimestamp = sim.blockAt(2).timestamp;
        SimHeader.Data memory forged = sim.nextBlock();
        forged.timestamp = trustedTimestamp;
        forged = sim.seal(forged);
        sim.produceBlocks(2);
        bytes memory forgedUpdate = sim.updateMsg(2, forged);
        bytes memory honestUpdate = sim.updateMsg(2, 4);

        uint256 snapshot = vm.snapshotState();
        bytes[3] memory misbehaviours = [
            _conflictingHeadersMisbehaviour(forgedUpdate, honestUpdate),
            _conflictingHeadersMisbehaviour(honestUpdate, forgedUpdate),
            _conflictingHeadersMisbehaviour(forgedUpdate, forgedUpdate)
        ];
        for (uint256 i = 0; i < misbehaviours.length; ++i) {
            vm.revertToState(snapshot);
            vm.expectEmit(address(client));
            emit IBesuLightClient.TimeNonMonotonicity(3, 2, trustedTimestamp, trustedTimestamp);
            client.misbehaviour(misbehaviours[i]);
            assertTrue(_isFrozen(client));
        }
    }

    function test_conflictingHeadersMisbehaviourRejections() public {
        address[] memory two = new address[](2);
        (two[0], two[1]) = (sim.validators()[0], sim.validators()[1]);
        bytes memory underSigned = sim.updateMsg(2, sim.seal(sim.nextBlock(), two));
        sim.produceBlocks(2);
        bytes memory update3 = sim.updateMsg(2, 3);
        bytes memory update4 = sim.updateMsg(2, 4);

        vm.expectRevert(abi.encodeWithSelector(IBesuLightClientErrors.NoMisbehaviourDetected.selector));
        client.misbehaviour(_conflictingHeadersMisbehaviour(update3, update3));

        vm.expectRevert(abi.encodeWithSelector(IBesuLightClientErrors.NoMisbehaviourDetected.selector));
        client.misbehaviour(_conflictingHeadersMisbehaviour(update4, update3));

        vm.expectRevert(
            abi.encodeWithSelector(IBesuLightClientErrors.InsufficientTrustedValidatorOverlap.selector, 2, 3)
        );
        client.misbehaviour(_conflictingHeadersMisbehaviour(update3, underSigned));

        uint64 trustedTimestamp = sim.blockAt(2).timestamp;
        vm.warp(trustedTimestamp + TRUSTING_PERIOD);
        vm.expectRevert(
            abi.encodeWithSelector(
                IBesuLightClientErrors.ConsensusStateExpired.selector,
                trustedTimestamp,
                vm.getBlockTimestamp(),
                TRUSTING_PERIOD
            )
        );
        client.misbehaviour(_conflictingHeadersMisbehaviour(update3, update4));

        assertFalse(_isFrozen(client));
    }

    /// @dev Proof submission is gated, but anyone can submit misbehaviour.
    function test_misbehaviourIsPermissionless() public {
        IBesuLightClient gated = sim.deployLightClient(
            TRUSTING_PERIOD, MAX_CLOCK_DRIFT, IBesuLightClientMsgs.TrustThreshold(2, 3), address(this)
        );
        SimHeader.Data memory a = sim.seal(sim.nextBlock());
        SimHeader.Data memory b = sim.nextBlock();
        b.stateRoot = keccak256("equivocation");
        b = sim.seal(b);
        bytes memory update = sim.updateMsg(2, a);
        bytes memory misbehaviour = _conflictingHeadersMisbehaviour(update, sim.updateMsg(2, b));
        address anyone = makeAddr("anyone");

        vm.expectPartialRevert(IAccessControl.AccessControlUnauthorizedAccount.selector);
        vm.prank(anyone);
        gated.updateClient(update);

        vm.prank(anyone);
        gated.misbehaviour(misbehaviour);
        assertTrue(_isFrozen(gated));
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

    function test_unsortedOrDuplicateSigners() public {
        SimHeader.Data memory h = sim.seal(sim.nextBlock());
        (h.commitSeals[0], h.commitSeals[1]) = (h.commitSeals[1], h.commitSeals[0]);
        bytes memory update = sim.updateMsg(2, h);
        vm.expectRevert(abi.encodeWithSelector(IBesuLightClientErrors.UnsortedCommitSealSigners.selector, 0));
        client.updateClient(update);

        uint256[] memory keys = new uint256[](4);
        for (uint256 i = 0; i < 3; ++i) {
            keys[i] = sim.validatorKey(sim.validators()[i]);
        }
        keys[3] = keys[2];
        update = sim.updateMsg(2, sim.sealWithKeys(sim.nextBlock(), keys));
        vm.expectRevert(abi.encodeWithSelector(IBesuLightClientErrors.UnsortedCommitSealSigners.selector, 2));
        client.updateClient(update);
    }

    function test_validatorChurn() public {
        _rotateValidators(1, 1);
        client.updateClient(sim.updateMsg(2, 4));
        assertEq(sim.blockAt(4).validators.length, 4);

        // Replacing the whole set in one block also changes it too much, but the overlap check comes first.
        for (uint256 i = 0; i < 4; ++i) {
            sim.removeValidator(sim.validators()[0]);
        }
        sim.addValidators(4);
        sim.produceBlock();
        bytes memory update = sim.updateMsg(4, 5);
        vm.expectRevert(
            abi.encodeWithSelector(IBesuLightClientErrors.InsufficientTrustedValidatorOverlap.selector, 0, 3)
        );
        client.updateClient(update);
    }

    /// @dev Two validator changes need at least two blocks: a header that makes both in one block freezes the client.
    function test_invalidValidatorSetChangeUpdate() public {
        sim.removeValidator(sim.validators()[0]);
        sim.addValidator();
        sim.produceBlocks(2);
        bytes memory update3 = sim.updateMsg(2, 3);
        bytes memory update4 = sim.updateMsg(2, 4);

        uint256 snapshot = vm.snapshotState();
        assertEq(uint8(client.updateClient(update4)), uint8(ILightClientMsgs.UpdateResult.Update));
        vm.revertToState(snapshot);

        vm.expectEmit(address(client));
        emit IBesuLightClient.InvalidValidatorSetChange(3, 2, 2);
        assertEq(uint8(client.updateClient(update3)), uint8(ILightClientMsgs.UpdateResult.Misbehaviour));

        vm.expectRevert(abi.encodeWithSelector(IBesuLightClientErrors.ConsensusStateNotFound.selector, 3));
        client.getConsensusStateHash(3);
        assertTrue(_isFrozen(client));
    }

    /// @dev The honest chain rotates one validator out and one in by height 4, and a forged height-5 header signed by
    /// the original validators reverts the rotation. Each is valid against height 2, but heights 4 and 5 differ by two
    /// validators, so they freeze the client as stored consensus states or as unstored headers, in either order.
    function test_invalidValidatorSetChangeMisbehaviour() public {
        address[] memory trusted = sim.validators();
        _rotateValidators(1, 1);
        SimHeader.Data memory forged = sim.nextBlock();
        forged.validators = trusted;
        forged = sim.seal(forged);
        bytes memory honestUpdate = sim.updateMsg(2, 4);
        bytes memory forgedUpdate = sim.updateMsg(2, forged);

        bytes[3] memory misbehaviours = [
            abi.encode(
                IBesuLightClientMsgs.MsgSubmitMisbehaviour(
                    IBesuLightClientMsgs.MisbehaviourType.ConflictingConsensusStates,
                    abi.encode(
                        IBesuLightClientMsgs.MsgConflictingConsensusStatesMisbehaviour({
                            height1: IICS02ClientMsgs.Height(0, 5),
                            height2: IICS02ClientMsgs.Height(0, 4),
                            consensusStatePreimage1: _consensusState(forged),
                            consensusStatePreimage2: sim.consensusState(4)
                        })
                    )
                )
            ),
            _conflictingHeadersMisbehaviour(honestUpdate, forgedUpdate),
            _conflictingHeadersMisbehaviour(forgedUpdate, honestUpdate)
        ];
        uint256 snapshot = vm.snapshotState();
        for (uint256 i = 0; i < misbehaviours.length; ++i) {
            vm.revertToState(snapshot);
            if (i == 0) {
                assertEq(uint8(client.updateClient(honestUpdate)), uint8(ILightClientMsgs.UpdateResult.Update));
                assertEq(uint8(client.updateClient(forgedUpdate)), uint8(ILightClientMsgs.UpdateResult.Update));
            }

            vm.expectEmit(address(client));
            emit IBesuLightClient.InvalidValidatorSetChange(5, 4, 2);
            client.misbehaviour(misbehaviours[i]);
            assertTrue(_isFrozen(client));
        }
    }

    function test_trustLevel() public {
        IBesuLightClient lenient =
            sim.deployLightClient(TRUSTING_PERIOD, MAX_CLOCK_DRIFT, IBesuLightClientMsgs.TrustThreshold(1, 3));
        IBesuLightClient strict =
            sim.deployLightClient(TRUSTING_PERIOD, MAX_CLOCK_DRIFT, IBesuLightClientMsgs.TrustThreshold(3, 4));

        address[] memory three = new address[](3);
        (three[0], three[1], three[2]) = (sim.validators()[0], sim.validators()[1], sim.validators()[2]);
        bytes memory update = sim.updateMsg(2, sim.seal(sim.nextBlock(), three));
        vm.expectRevert(
            abi.encodeWithSelector(IBesuLightClientErrors.InsufficientTrustedValidatorOverlap.selector, 3, 4)
        );
        strict.updateClient(update);

        // Rotating half of the validators leaves 2 of 4 trusted signers: below 2/3, above 1/3.
        _rotateValidators(2, 2);
        update = sim.updateMsg(2, 6);
        vm.expectRevert(
            abi.encodeWithSelector(IBesuLightClientErrors.InsufficientTrustedValidatorOverlap.selector, 2, 3)
        );
        client.updateClient(update);
        assertEq(uint8(lenient.updateClient(update)), uint8(ILightClientMsgs.UpdateResult.Update));
    }

    /// @dev Exactly one third of the trusted set is not enough at a 1/3 trust level: more than a third must sign.
    function test_trustLevelOneThirdIsStrict() public {
        uint256[2] memory counts = [uint256(3), 6];
        for (uint256 c = 0; c < counts.length; ++c) {
            QBFTSimSuite s = _setUp(counts[c]);
            IBesuLightClient lenient =
                s.deployLightClient(TRUSTING_PERIOD, MAX_CLOCK_DRIFT, IBesuLightClientMsgs.TrustThreshold(1, 3));

            uint256 third = counts[c] / 3;
            address[] memory signers = new address[](third);
            for (uint256 i = 0; i < third; ++i) {
                signers[i] = s.validators()[i];
            }
            bytes memory update = s.updateMsg(2, s.seal(s.nextBlock(), signers));
            vm.expectRevert(
                abi.encodeWithSelector(
                    IBesuLightClientErrors.InsufficientTrustedValidatorOverlap.selector, third, third + 1
                )
            );
            lenient.updateClient(update);
        }
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

    function _conflictingHeadersMisbehaviour(
        bytes memory update1,
        bytes memory update2
    )
        internal
        pure
        returns (bytes memory)
    {
        return abi.encode(
            IBesuLightClientMsgs.MsgSubmitMisbehaviour(
                IBesuLightClientMsgs.MisbehaviourType.ConflictingHeaders,
                abi.encode(
                    IBesuLightClientMsgs.MsgConflictingHeadersMisbehaviour(
                        abi.decode(update1, (IBesuLightClientMsgs.MsgUpdateClient)),
                        abi.decode(update2, (IBesuLightClientMsgs.MsgUpdateClient))
                    )
                )
            )
        );
    }

    function _consensusState(SimHeader.Data memory h)
        internal
        pure
        returns (IBesuLightClientMsgs.ConsensusState memory)
    {
        return IBesuLightClientMsgs.ConsensusState(h.timestamp, h.stateRoot, h.validators);
    }

    function _isFrozen(IBesuLightClient lc) internal view returns (bool) {
        return abi.decode(lc.getClientState(), (IBesuLightClientMsgs.ClientState)).isFrozen;
    }

    /// @dev Rotates a random part of the trusted set and signs with a random subset of the new set: the update must
    /// succeed iff the signers reach the trust level of the trusted set and a BFT quorum of the new set.
    function testFuzz_updateClient_signerSubset(
        uint256 trustedCount,
        uint256 removed,
        uint256 added,
        uint256 signerMask
    )
        public
    {
        trustedCount = bound(trustedCount, 1, 7);
        removed = bound(removed, 0, trustedCount - 1);
        added = bound(added, 0, 3);
        sim = _setUp(trustedCount);
        _rotateValidators(removed, added);

        address[] memory trusted = sim.consensusState(2).validators;
        address[] memory current = sim.validators();
        uint256 signerCount = 0;
        for (uint256 i = 0; i < current.length; ++i) {
            signerCount += (signerMask >> i) & 1;
        }
        address[] memory signers = new address[](signerCount);
        uint256 overlap = 0;
        uint256 k = 0;
        for (uint256 i = 0; i < current.length; ++i) {
            if ((signerMask >> i) & 1 == 1) {
                signers[k] = current[i];
                ++k;
                if (_contains(trusted, current[i])) {
                    ++overlap;
                }
            }
        }
        bytes memory update = sim.updateMsg(2, sim.seal(sim.nextBlock(), signers));

        uint256 requiredOverlap = 2 * trusted.length / 3 + 1;
        uint256 requiredQuorum = Math.ceilDiv(2 * current.length, 3);
        if (overlap < requiredOverlap) {
            vm.expectRevert(
                abi.encodeWithSelector(
                    IBesuLightClientErrors.InsufficientTrustedValidatorOverlap.selector, overlap, requiredOverlap
                )
            );
        } else if (signerCount < requiredQuorum) {
            vm.expectRevert(
                abi.encodeWithSelector(
                    IBesuLightClientErrors.InsufficientValidatorQuorum.selector, signerCount, requiredQuorum
                )
            );
        } else {
            assertEq(uint8(client.updateClient(update)), uint8(ILightClientMsgs.UpdateResult.Update));
            return;
        }
        client.updateClient(update);
    }

    /// @dev Removes the lowest `removed` validators, then adds `added` new ones, producing a block after each change as
    /// header-validator voting does.
    function _rotateValidators(uint256 removed, uint256 added) internal {
        for (uint256 i = 0; i < removed; ++i) {
            sim.removeValidator(sim.validators()[0]);
            sim.produceBlock();
        }
        for (uint256 i = 0; i < added; ++i) {
            sim.addValidator();
            sim.produceBlock();
        }
    }

    function _contains(address[] memory set, address value) internal pure returns (bool) {
        for (uint256 i = 0; i < set.length; ++i) {
            if (set[i] == value) {
                return true;
            }
        }
        return false;
    }
}

contract QBFTSimSuiteQBFTTest is QBFTSimSuiteTest {
    function _mode() internal pure override returns (SimHeader.Mode) {
        return SimHeader.Mode.QBFT;
    }
}

contract QBFTSimSuiteIBFT2Test is QBFTSimSuiteTest {
    function _mode() internal pure override returns (SimHeader.Mode) {
        return SimHeader.Mode.IBFT2;
    }
}
