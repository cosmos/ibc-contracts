// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable gas-struct-packing,function-max-lines,gas-small-strings

import { Test } from "forge-std/Test.sol";
import { stdJson } from "forge-std/StdJson.sol";
import { RLP } from "@openzeppelin-contracts/utils/RLP.sol";
import { Memory } from "@openzeppelin-contracts/utils/Memory.sol";
import { SafeCast } from "@openzeppelin-contracts/utils/math/SafeCast.sol";
import { ECDSA } from "@openzeppelin-contracts/utils/cryptography/ECDSA.sol";

import { ILightClientMsgs } from "../../contracts/msgs/ILightClientMsgs.sol";
import { IICS02ClientMsgs } from "../../contracts/msgs/IICS02ClientMsgs.sol";
import { BesuLightClient } from "../../contracts/light-clients/besu/BesuLightClient.sol";
import { IBesuLightClient } from "../../contracts/light-clients/besu/interfaces/IBesuLightClient.sol";
import { IBesuLightClientMsgs } from "../../contracts/light-clients/besu/msgs/IBesuLightClientMsgs.sol";
import { IBesuLightClientErrors } from "../../contracts/light-clients/besu/errors/IBesuLightClientErrors.sol";
import { Header } from "../../contracts/light-clients/besu/utils/Header.sol";
import { TrieProof } from "../../contracts/light-clients/utils/TrieProof.sol";

/// @dev Successful update input and expected consensus state.
/// @dev `headerRlp` is the header in its commit seal signing form and `commitSeals` its sorted commit seals.
struct BesuUpdateFixture {
    uint64 height;
    bytes headerRlp;
    bytes[] commitSeals;
    uint64 trustedHeight;
    uint64 expectedTimestamp;
    bytes32 expectedStateRoot;
    address[] expectedValidators;
}

/// @dev Update input expected to be rejected.
struct BesuRejectionUpdateFixture {
    uint64 height;
    bytes headerRlp;
    bytes[] commitSeals;
    uint64 trustedHeight;
}

/// @dev Membership or non-membership storage proof and expected timestamp.
/// @dev `proof` holds the raw storage proof nodes and `accountProof` the raw account proof nodes, both as
/// `abi.encode(bytes[])`; the tests wrap them into `IBesuLightClientMsgs.MembershipProof` together with the
/// consensus state preimage for `proofHeight`.
struct BesuProofFixture {
    bytes proof;
    bytes accountProof;
    uint64 proofHeight;
    bytes path;
    bytes value;
    uint64 expectedTimestamp;
}

/// @dev Successful or rejected update, including any scenario-specific setup.
struct BesuUpdateTestCase {
    string name;
    uint64 timestamp;
    bytes update;
    bytes preUpdate;
    bytes expectedRevert;
    BesuUpdateFixture expectedState;
}

/// @dev Successful or rejected membership verification.
/// @dev `timestamp` is the block timestamp at which the membership proof is verified.
struct BesuMembershipTestCase {
    string name;
    uint64 timestamp;
    ILightClientMsgs.MsgVerifyMembership message;
    bytes expectedRevert;
    uint64 expectedTimestamp;
}

/// @dev Successful or rejected non-membership verification.
/// @dev `timestamp` is the block timestamp at which the non-membership proof is verified.
struct BesuNonMembershipTestCase {
    string name;
    uint64 timestamp;
    ILightClientMsgs.MsgVerifyNonMembership message;
    bytes expectedRevert;
    uint64 expectedTimestamp;
}

/// @dev Proof verification that relies on a storage root cached by an earlier verification in the same transaction.
/// @dev The cache is primed by a membership or non-membership verification at `fixture.nonAdjacentUpdate.height`;
/// `proof` is the `abi.encode(IBesuLightClientMsgs.MembershipProof)` carried by the second call.
struct BesuCachedStorageRootTestCase {
    string name;
    bool primeWithMembership;
    bool verifyMembership;
    bytes proof;
    uint64 proofHeight;
    bytes expectedRevert;
    uint64 expectedTimestamp;
}

/// @dev Rejected conflicting consensus states misbehaviour, submitted once `fixture.nonAdjacentUpdate` is stored.
/// @dev `timestamp` is the block timestamp at which the misbehaviour is submitted.
struct BesuMisbehaviourTestCase {
    string name;
    uint64 timestamp;
    bytes misbehaviour;
    bytes expectedRevert;
}

/// @dev Initial states passed to the constructor, and the expected revert if any.
/// @dev `timestamp` is the block timestamp at deployment.
struct BesuConstructorTestCase {
    string name;
    uint64 timestamp;
    IBesuLightClientMsgs.ClientState clientState;
    IBesuLightClientMsgs.ConsensusState consensusState;
    bytes expectedRevert;
}

/// @dev Update signed by the first `signerCount` of four synthetic trusted validators for a header whose validator
/// set grows to `headerValidatorCount` (a superset of the trusted set), so the trusted overlap and quorum checks use
/// different denominators.
struct BesuQuorumTestCase {
    string name;
    uint256 signerCount;
    uint256 headerValidatorCount;
    IBesuLightClientMsgs.TrustThreshold trustLevel;
    bytes expectedRevert;
}

/// @dev Complete fixture shared by the Besu BFT light-client tests.
struct BesuFixture {
    address routerAddress;
    uint64 initialTrustedHeight;
    uint64 initialTrustedTimestamp;
    bytes32 initialTrustedStateRoot;
    address[] initialTrustedValidators;
    uint64 trustingPeriod;
    uint64 maxClockDrift;
    BesuUpdateFixture adjacentUpdate;
    BesuUpdateFixture nonAdjacentUpdate;
    BesuRejectionUpdateFixture lowQuorumUpdate;
    BesuRejectionUpdateFixture conflictingUpdate;
    BesuRejectionUpdateFixture lowOverlapUpdate;
    BesuProofFixture membership;
    BesuProofFixture nonMembership;
}

abstract contract BesuLightClientFixtureTestBase is Test {
    using stdJson for string;
    using RLP for bytes;
    using Memory for Memory.Slice;
    using Memory for bytes;

    string internal constant FIXTURE_DIR = "/test/besu-bft/fixtures/";
    /// @dev ERC-7201 namespace slot of `IBCStoreUpgradeable`, the second element of the Besu merkle prefix.
    bytes32 internal constant IBCSTORE_STORAGE_SLOT =
        0x1260944489272988d9df285149b5aa1b0f48f2136d6f416159f840a3e0747600;
    /// @dev secp256k1 curve order.
    uint256 internal constant SECP256K1_N = 0xFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFEBAAEDCE6AF48A03BBFD25E8CD0364141;

    BesuFixture internal fixture;
    IBesuLightClientMsgs.TrustThreshold internal trustLevel = IBesuLightClientMsgs.TrustThreshold(2, 3);
    IBesuLightClient internal client;
    address internal ics26Router = makeAddr("ics26Router");

    function setUp() public virtual {
        fixture = _loadFixture(_fixtureFile());
        vm.warp(fixture.initialTrustedTimestamp);
        client = _deployClient(_initialClientState(), _initialConsensusState());
    }

    function tableConstructorTest(BesuConstructorTestCase memory deployment) public {
        vm.warp(deployment.timestamp);
        if (deployment.expectedRevert.length != 0) {
            vm.expectRevert(deployment.expectedRevert);
        }
        client = _deployClient(deployment.clientState, deployment.consensusState);

        if (deployment.expectedRevert.length == 0) {
            assertEq(client.getClientState(), abi.encode(deployment.clientState));
            assertEq(
                client.getConsensusStateHash(deployment.clientState.latestHeight.revisionHeight),
                _consensusStateHash(deployment.consensusState)
            );
        }
    }

    /// @dev Deploys a client trusting four synthetic validators, then submits the non-adjacent header re-signed by
    /// `quorum.signerCount` of them with its validator set grown to `quorum.headerValidatorCount`.
    function tableQuorumTest(BesuQuorumTestCase memory quorum) public {
        fixture.initialTrustedValidators = _syntheticValidators(4);
        trustLevel = quorum.trustLevel;
        client = _deployClient(_initialClientState(), _initialConsensusState());

        address[] memory headerValidators = _syntheticValidators(quorum.headerValidatorCount);
        uint256[] memory signerKeys = new uint256[](quorum.signerCount);
        for (uint256 i = 0; i < signerKeys.length; ++i) {
            signerKeys[i] = i + 1;
        }

        vm.warp(fixture.initialTrustedTimestamp + 1);
        if (quorum.expectedRevert.length != 0) {
            vm.expectRevert(quorum.expectedRevert);
        }
        vm.prank(ics26Router);
        ILightClientMsgs.UpdateResult result =
            client.updateClient(_signedValidatorsUpdate(headerValidators, signerKeys));

        if (quorum.expectedRevert.length == 0) {
            assertEq(uint8(result), uint8(ILightClientMsgs.UpdateResult.Update));
            BesuUpdateFixture memory expectedState = fixture.nonAdjacentUpdate;
            expectedState.expectedValidators = headerValidators;
            _assertClientState(expectedState);
        }
    }

    function test_getConsensusStateHash_revertUnknownHeight() public {
        uint64 unknownHeight = fixture.initialTrustedHeight + 1000;
        vm.expectRevert(abi.encodeWithSelector(IBesuLightClientErrors.ConsensusStateNotFound.selector, unknownHeight));
        client.getConsensusStateHash(unknownHeight);
    }

    function tableVerifyNonMembershipTest(BesuNonMembershipTestCase memory nonMembership) public {
        vm.warp(fixture.initialTrustedTimestamp + 1);
        vm.startPrank(ics26Router);
        client.updateClient(_encodeUpdate(fixture.nonAdjacentUpdate));
        vm.warp(nonMembership.timestamp);

        if (nonMembership.expectedRevert.length != 0) {
            vm.expectRevert(nonMembership.expectedRevert);
        }
        uint256 timestamp = client.verifyNonMembership(nonMembership.message);
        vm.stopPrank();

        if (nonMembership.expectedRevert.length == 0) {
            assertEq(timestamp, nonMembership.expectedTimestamp);
        }
    }

    /// @dev Primes the transient storage root cache with one verified proof, then verifies a second proof that
    /// relies on the cache within the same transaction.
    function tableCachedStorageRootTest(BesuCachedStorageRootTestCase memory cachedStorageRoot) public {
        vm.warp(fixture.initialTrustedTimestamp + 1);
        vm.startPrank(ics26Router);
        client.updateClient(_encodeUpdate(fixture.adjacentUpdate));
        client.updateClient(_encodeUpdate(fixture.nonAdjacentUpdate));
        if (cachedStorageRoot.primeWithMembership) {
            client.verifyMembership(
                _membershipMessage(0, _prefixedPath(fixture.membership.path), fixture.membership.value)
            );
        } else {
            client.verifyNonMembership(_nonMembershipMessage(fixture.nonMembership.proofHeight));
        }

        if (cachedStorageRoot.expectedRevert.length != 0) {
            vm.expectRevert(cachedStorageRoot.expectedRevert);
        }
        uint256 timestamp = cachedStorageRoot.verifyMembership
            ? client.verifyMembership(
                _membershipMessageWithProof(cachedStorageRoot.proof, cachedStorageRoot.proofHeight)
            )
            : client.verifyNonMembership(
                _nonMembershipMessageWithProof(cachedStorageRoot.proof, cachedStorageRoot.proofHeight)
            );
        vm.stopPrank();

        if (cachedStorageRoot.expectedRevert.length == 0) {
            assertEq(timestamp, cachedStorageRoot.expectedTimestamp);
        }
    }

    function test_updateClient_noOpOnSameState() public {
        vm.warp(fixture.initialTrustedTimestamp + 1);
        vm.startPrank(ics26Router);
        client.updateClient(_encodeUpdate(fixture.nonAdjacentUpdate));
        bytes32 storedHash = client.getConsensusStateHash(fixture.nonAdjacentUpdate.height);

        ILightClientMsgs.UpdateResult result = client.updateClient(_encodeUpdate(fixture.nonAdjacentUpdate));
        vm.stopPrank();

        assertEq(uint8(result), uint8(ILightClientMsgs.UpdateResult.NoOp));
        assertEq(client.getConsensusStateHash(fixture.nonAdjacentUpdate.height), storedHash);
        _assertClientState(fixture.nonAdjacentUpdate);
    }

    function tableUpdateClientTest(BesuUpdateTestCase memory update) public {
        vm.warp(update.timestamp);
        vm.startPrank(ics26Router);
        if (update.preUpdate.length != 0) {
            client.updateClient(update.preUpdate);
        }

        if (update.expectedRevert.length == 0) {
            ILightClientMsgs.UpdateResult result = client.updateClient(update.update);

            assertEq(uint8(result), uint8(ILightClientMsgs.UpdateResult.Update));
            _assertClientState(update.expectedState);
            return;
        }

        bytes memory clientStateBefore = client.getClientState();
        bytes32 consensusStateHashBefore = client.getConsensusStateHash(fixture.initialTrustedHeight);

        vm.expectRevert(update.expectedRevert);
        client.updateClient(update.update);

        assertEq(client.getClientState(), clientStateBefore);
        assertEq(client.getConsensusStateHash(fixture.initialTrustedHeight), consensusStateHashBefore);
    }

    function tableVerifyMembershipTest(BesuMembershipTestCase memory membership) public {
        vm.warp(fixture.initialTrustedTimestamp + 1);
        vm.startPrank(ics26Router);
        client.updateClient(_encodeUpdate(fixture.nonAdjacentUpdate));
        vm.warp(membership.timestamp);

        if (membership.expectedRevert.length != 0) {
            vm.expectRevert(membership.expectedRevert);
        }
        uint256 timestamp = client.verifyMembership(membership.message);
        vm.stopPrank();

        if (membership.expectedRevert.length == 0) {
            assertEq(timestamp, membership.expectedTimestamp);
        }
    }

    function test_updateClient_doubleSignFreezesClient() public {
        vm.warp(fixture.initialTrustedTimestamp + 1);
        vm.startPrank(ics26Router);
        client.updateClient(_encodeUpdate(fixture.nonAdjacentUpdate));
        uint64 height = fixture.conflictingUpdate.height;
        bytes32 trustedHash = client.getConsensusStateHash(height);
        Header.Data memory header = Header.decodeRlp(fixture.conflictingUpdate.headerRlp);
        bytes32 conflictingHash = _consensusStateHash(
            IBesuLightClientMsgs.ConsensusState({
                timestamp: header.timestamp, stateRoot: header.stateRoot, validators: header.validators
            })
        );

        vm.expectEmit(address(client));
        emit IBesuLightClient.DoubleSign(height, trustedHash, conflictingHash);
        ILightClientMsgs.UpdateResult result = client.updateClient(_encodeUpdate(fixture.conflictingUpdate));

        assertEq(uint8(result), uint8(ILightClientMsgs.UpdateResult.Misbehaviour));
        assertEq(client.getConsensusStateHash(height), trustedHash);
        assertTrue(_clientState().isFrozen);

        bytes memory frozenErr = abi.encodeWithSelector(IBesuLightClientErrors.FrozenClientState.selector);
        vm.expectRevert(frozenErr);
        client.updateClient(_encodeUpdate(fixture.nonAdjacentUpdate));
        vm.expectRevert(frozenErr);
        client.verifyMembership(_membershipMessage(0, _prefixedPath(fixture.membership.path), fixture.membership.value));
        vm.expectRevert(frozenErr);
        client.verifyNonMembership(_nonMembershipMessage(fixture.nonMembership.proofHeight));
        vm.expectRevert(frozenErr);
        client.misbehaviour(bytes(""));
        vm.stopPrank();
    }

    /// @dev Live fixture headers have monotonic timestamps, so only rejections are covered here; accepted
    /// misbehaviour is covered by `QBFTSimSuiteTest`.
    function tableMisbehaviourTest(BesuMisbehaviourTestCase memory misbehaviour) public {
        vm.warp(fixture.initialTrustedTimestamp + 1);
        vm.startPrank(ics26Router);
        client.updateClient(_encodeUpdate(fixture.nonAdjacentUpdate));
        bytes memory clientStateBefore = client.getClientState();
        vm.warp(misbehaviour.timestamp);

        vm.expectRevert(misbehaviour.expectedRevert);
        client.misbehaviour(misbehaviour.misbehaviour);
        vm.stopPrank();

        assertEq(client.getClientState(), clientStateBefore);
    }

    /// @dev Converts the header to the other protocol's signing form, IBFT2 to QBFT or QBFT to IBFT2, keeping the
    /// commit seals. The changed digest recovers arbitrary signers, which must be rejected.
    function test_updateClient_revertOtherSigningForm() public {
        vm.warp(fixture.initialTrustedTimestamp + 1);

        BesuUpdateFixture memory update = fixture.nonAdjacentUpdate;
        Memory.Slice[] memory headerItems = update.headerRlp.decodeList();
        Memory.Slice[] memory extraItems = RLP.readBytes(headerItems[12]).decodeList();
        bytes[] memory encodedExtraItems = new bytes[](extraItems.length == 4 ? 5 : 4);
        for (uint256 i = 0; i < 4; ++i) {
            encodedExtraItems[i] = extraItems[i].toBytes();
        }
        if (extraItems.length == 4) {
            encodedExtraItems[4] = hex"c0";
        }
        update.headerRlp = _encodeHeader(headerItems, encodedExtraItems);

        bytes32 digest = keccak256(update.headerRlp);
        address previousSigner = ECDSA.recover(digest, update.commitSeals[0]);
        bytes memory expectedRevert =
            abi.encodeWithSelector(IBesuLightClientErrors.UnknownCommitSealSigner.selector, previousSigner);
        for (uint256 i = 1; i < update.commitSeals.length; ++i) {
            address signer = ECDSA.recover(digest, update.commitSeals[i]);
            if (previousSigner >= signer) {
                expectedRevert =
                    abi.encodeWithSelector(IBesuLightClientErrors.UnsortedCommitSealSigners.selector, i - 1);
                break;
            }
            previousSigner = signer;
        }

        vm.expectRevert(expectedRevert);
        vm.prank(ics26Router);
        client.updateClient(_encodeUpdate(update));
    }

    /// @dev Each case starts from the fixture's initial states; failure cases then change the one field under test.
    function fixtureDeployment() public view returns (BesuConstructorTestCase[] memory testCases) {
        uint64 timestamp = fixture.initialTrustedTimestamp;

        testCases = new BesuConstructorTestCase[](22);
        testCases[0] = _constructorCase("success: fixture values", timestamp, "");
        testCases[1] = _constructorCase(
            "failure: zero router", timestamp, abi.encodeWithSelector(IBesuLightClientErrors.InvalidIbcRouter.selector)
        );
        testCases[1].clientState.ibcRouter = address(0);
        testCases[2] = _constructorCase(
            "failure: non-zero revision number",
            timestamp,
            abi.encodeWithSelector(IBesuLightClientErrors.InvalidRevisionNumber.selector, 1)
        );
        testCases[2].clientState.latestHeight.revisionNumber = 1;
        testCases[3] = _constructorCase(
            "failure: zero height",
            timestamp,
            abi.encodeWithSelector(IBesuLightClientErrors.InvalidHeaderHeight.selector)
        );
        testCases[3].clientState.latestHeight.revisionHeight = 0;
        testCases[4] = _constructorCase(
            "failure: frozen client state",
            timestamp,
            abi.encodeWithSelector(IBesuLightClientErrors.FrozenClientState.selector)
        );
        testCases[4].clientState.isFrozen = true;
        testCases[5] = _constructorCase(
            "failure: zero trusting period",
            timestamp,
            abi.encodeWithSelector(IBesuLightClientErrors.InvalidTrustingPeriod.selector, 0)
        );
        testCases[5].clientState.trustingPeriod = 0;
        testCases[6] = _trustLevelConstructorCase("success: minimum trust level", 1, 3, "");
        testCases[7] = _trustLevelConstructorCase("success: maximum trust level", 2, 3, "");
        testCases[8] = _trustLevelConstructorCase(
            "failure: zero trust level denominator",
            0,
            0,
            abi.encodeWithSelector(IBesuLightClientErrors.InvalidTrustLevel.selector, 0, 0)
        );
        testCases[9] = _trustLevelConstructorCase(
            "failure: trust level below one third",
            1,
            4,
            abi.encodeWithSelector(IBesuLightClientErrors.InvalidTrustLevel.selector, 1, 4)
        );
        testCases[10] = _trustLevelConstructorCase(
            "failure: trust level above two thirds",
            3,
            4,
            abi.encodeWithSelector(IBesuLightClientErrors.InvalidTrustLevel.selector, 3, 4)
        );
        testCases[11] = _constructorCase(
            "failure: zero timestamp",
            timestamp,
            abi.encodeWithSelector(IBesuLightClientErrors.InvalidHeaderTimestamp.selector)
        );
        testCases[11].consensusState.timestamp = 0;
        testCases[12] = _constructorCase(
            "failure: empty validators",
            timestamp,
            abi.encodeWithSelector(IBesuLightClientErrors.EmptyValidatorSet.selector)
        );
        testCases[12].consensusState.validators = new address[](0);
        testCases[13] = _constructorCase(
            "failure: expired consensus state",
            timestamp + fixture.trustingPeriod,
            abi.encodeWithSelector(
                IBesuLightClientErrors.ConsensusStateExpired.selector,
                timestamp,
                timestamp + fixture.trustingPeriod,
                fixture.trustingPeriod
            )
        );
        testCases[14] =
            _constructorCase("success: consensus state at max clock drift", timestamp - fixture.maxClockDrift, "");
        testCases[15] = _constructorCase(
            "failure: consensus state beyond max clock drift",
            timestamp - fixture.maxClockDrift - 1,
            abi.encodeWithSelector(
                IBesuLightClientErrors.HeaderFromFuture.selector,
                timestamp - fixture.maxClockDrift - 1,
                timestamp,
                fixture.maxClockDrift
            )
        );
        testCases[16] = _constructorCase(
            "failure: zero state root",
            timestamp,
            abi.encodeWithSelector(IBesuLightClientErrors.InvalidStateRoot.selector, bytes32(0))
        );
        testCases[16].consensusState.stateRoot = bytes32(0);
        testCases[17] = _constructorCase(
            "failure: empty trie state root",
            timestamp,
            abi.encodeWithSelector(IBesuLightClientErrors.InvalidStateRoot.selector, TrieProof.EMPTY_ROOT_HASH)
        );
        testCases[17].consensusState.stateRoot = TrieProof.EMPTY_ROOT_HASH;
        testCases[18] = _constructorCase("success: max trusting period", timestamp, "");
        testCases[18].clientState.trustingPeriod = 30 days;
        testCases[19] = _constructorCase(
            "failure: trusting period above max",
            timestamp,
            abi.encodeWithSelector(IBesuLightClientErrors.InvalidTrustingPeriod.selector, 30 days + 1)
        );
        testCases[19].clientState.trustingPeriod = 30 days + 1;
        testCases[20] = _constructorCase("success: max clock drift", timestamp, "");
        testCases[20].clientState.maxClockDrift = 12;
        testCases[21] = _constructorCase(
            "failure: clock drift above max",
            timestamp,
            abi.encodeWithSelector(IBesuLightClientErrors.InvalidMaxClockDrift.selector, 13)
        );
        testCases[21].clientState.maxClockDrift = 13;
    }

    /// @dev Quorum thresholds are ceil(2n / 3): 3 of 4, 4 of 6 and 5 of 7. The trusted set has 4 validators, so the
    /// trusted overlap requirement is capped at 2 for any trust level and the quorum check rejects under-signed
    /// headers.
    function fixtureQuorum() public pure returns (BesuQuorumTestCase[] memory testCases) {
        IBesuLightClientMsgs.TrustThreshold memory twoThirds = IBesuLightClientMsgs.TrustThreshold(2, 3);
        testCases = new BesuQuorumTestCase[](5);
        testCases[0] = BesuQuorumTestCase("success: three of four signers", 3, 4, twoThirds, "");
        testCases[1] = BesuQuorumTestCase("success: four signers with validator set grown to six", 4, 6, twoThirds, "");
        testCases[2] = BesuQuorumTestCase(
            "failure: three signers with validator set grown to six",
            3,
            6,
            twoThirds,
            abi.encodeWithSelector(IBesuLightClientErrors.InsufficientValidatorQuorum.selector, 3, 4)
        );
        testCases[3] = BesuQuorumTestCase(
            "failure: four signers with validator set grown to seven",
            4,
            7,
            twoThirds,
            abi.encodeWithSelector(IBesuLightClientErrors.InsufficientValidatorQuorum.selector, 4, 5)
        );
        testCases[4] = BesuQuorumTestCase(
            "failure: two of four signers",
            2,
            4,
            twoThirds,
            abi.encodeWithSelector(IBesuLightClientErrors.InsufficientValidatorQuorum.selector, 2, 3)
        );
    }

    function _trustLevelConstructorCase(
        string memory name,
        uint8 numerator,
        uint8 denominator,
        bytes memory expectedRevert
    )
        private
        view
        returns (BesuConstructorTestCase memory)
    {
        BesuConstructorTestCase memory testCase =
            _constructorCase(name, fixture.initialTrustedTimestamp, expectedRevert);
        testCase.clientState.trustLevel = IBesuLightClientMsgs.TrustThreshold(numerator, denominator);
        return testCase;
    }

    /// @dev Constructor case deploying fresh copies of the fixture's initial client and consensus states.
    function _constructorCase(
        string memory name,
        uint64 timestamp,
        bytes memory expectedRevert
    )
        private
        view
        returns (BesuConstructorTestCase memory)
    {
        return BesuConstructorTestCase({
            name: name,
            timestamp: timestamp,
            clientState: _initialClientState(),
            consensusState: _initialConsensusState(),
            expectedRevert: expectedRevert
        });
    }

    function fixtureMembership() public view returns (BesuMembershipTestCase[] memory testCases) {
        bytes[] memory slotlessPath = new bytes[](2);
        slotlessPath[0] = abi.encodePacked(fixture.routerAddress);
        slotlessPath[1] = fixture.membership.path;

        bytes memory wrongValue = abi.encodePacked(bytes32(uint256(1)));
        uint64 timestamp = fixture.initialTrustedTimestamp + 1;
        testCases = new BesuMembershipTestCase[](14);
        testCases[0] = BesuMembershipTestCase({
            name: "success",
            timestamp: timestamp,
            message: _membershipMessage(0, _prefixedPath(fixture.membership.path), fixture.membership.value),
            expectedRevert: "",
            expectedTimestamp: fixture.membership.expectedTimestamp
        });
        testCases[1] = BesuMembershipTestCase({
            name: "failure: wrong revision number",
            timestamp: timestamp,
            message: _membershipMessage(1, _prefixedPath(fixture.membership.path), fixture.membership.value),
            expectedRevert: abi.encodeWithSelector(IBesuLightClientErrors.InvalidRevisionNumber.selector, 1),
            expectedTimestamp: 0
        });
        testCases[2] = BesuMembershipTestCase({
            name: "failure: path without storage slot",
            timestamp: timestamp,
            message: _membershipMessage(0, slotlessPath, fixture.membership.value),
            expectedRevert: abi.encodeWithSelector(IBesuLightClientErrors.InvalidPathLength.selector, 3, 2),
            expectedTimestamp: 0
        });
        testCases[3] = BesuMembershipTestCase({
            name: "failure: wrong commitment value",
            timestamp: timestamp,
            message: _membershipMessage(0, _prefixedPath(fixture.membership.path), wrongValue),
            expectedRevert: abi.encodeWithSelector(
                IBesuLightClientErrors.InvalidCommitmentValue.selector,
                bytes32(uint256(1)),
                abi.decode(fixture.membership.value, (bytes32))
            ),
            expectedTimestamp: 0
        });

        IBesuLightClientMsgs.ConsensusState memory expectedPreimage =
            _provenConsensusState(fixture.membership.proofHeight);
        IBesuLightClientMsgs.ConsensusState memory tamperedPreimage =
            _provenConsensusState(fixture.membership.proofHeight);
        ++tamperedPreimage.timestamp;
        ILightClientMsgs.MsgVerifyMembership memory wrongPreimageMessage =
            _membershipMessage(0, _prefixedPath(fixture.membership.path), fixture.membership.value);
        wrongPreimageMessage.proof = _encodeProof(fixture.membership, tamperedPreimage);

        testCases[4] = BesuMembershipTestCase({
            name: "failure: wrong consensus state preimage",
            timestamp: timestamp,
            message: wrongPreimageMessage,
            expectedRevert: abi.encodeWithSelector(
                IBesuLightClientErrors.ConsensusStatePreimageMismatch.selector,
                _consensusStateHash(expectedPreimage),
                _consensusStateHash(tamperedPreimage)
            ),
            expectedTimestamp: 0
        });

        uint64 unknownHeight = fixture.membership.proofHeight + 1000;
        ILightClientMsgs.MsgVerifyMembership memory unknownHeightMessage =
            _membershipMessage(0, _prefixedPath(fixture.membership.path), fixture.membership.value);
        unknownHeightMessage.proofHeight.revisionHeight = unknownHeight;

        testCases[5] = BesuMembershipTestCase({
            name: "failure: unknown proof height",
            timestamp: timestamp,
            message: unknownHeightMessage,
            expectedRevert: abi.encodeWithSelector(
                IBesuLightClientErrors.ConsensusStateNotFound.selector, unknownHeight
            ),
            expectedTimestamp: 0
        });

        uint64 expiredAt = expectedPreimage.timestamp + fixture.trustingPeriod;
        testCases[6] = BesuMembershipTestCase({
            name: "failure: expired consensus state",
            timestamp: expiredAt,
            message: _membershipMessage(0, _prefixedPath(fixture.membership.path), fixture.membership.value),
            expectedRevert: abi.encodeWithSelector(
                IBesuLightClientErrors.ConsensusStateExpired.selector,
                expectedPreimage.timestamp,
                expiredAt,
                fixture.trustingPeriod
            ),
            expectedTimestamp: 0
        });

        testCases[7] = BesuMembershipTestCase({
            name: "failure: storage root not cached",
            timestamp: timestamp,
            message: _membershipMessageWithProof(
                _cachedProof(fixture.membership, expectedPreimage), fixture.membership.proofHeight
            ),
            expectedRevert: abi.encodeWithSelector(
                IBesuLightClientErrors.StorageRootNotInCache.selector, fixture.membership.proofHeight
            ),
            expectedTimestamp: 0
        });

        ILightClientMsgs.MsgVerifyMembership memory tamperedAccountProofMessage =
            _membershipMessage(0, _prefixedPath(fixture.membership.path), fixture.membership.value);
        tamperedAccountProofMessage.proof = _encodeProof(fixture.membership, expectedPreimage, _tamperedAccountProof());

        testCases[8] = BesuMembershipTestCase({
            name: "failure: tampered account proof",
            timestamp: timestamp,
            message: tamperedAccountProofMessage,
            expectedRevert: abi.encodeWithSelector(
                TrieProof.TrieProofTraversalError.selector, TrieProof.ProofError.INVALID_ROOT
            ),
            expectedTimestamp: 0
        });

        testCases[9] = BesuMembershipTestCase({
            name: "failure: wrong value length",
            timestamp: timestamp,
            message: _membershipMessage(0, _prefixedPath(fixture.membership.path), abi.encodePacked(bytes31(0))),
            expectedRevert: abi.encodeWithSelector(IBesuLightClientErrors.InvalidValueLength.selector, 32, 31),
            expectedTimestamp: 0
        });

        address wrongRouter = address(uint160(fixture.routerAddress) ^ 1);
        testCases[10] = BesuMembershipTestCase({
            name: "failure: wrong counterparty router",
            timestamp: timestamp,
            message: _membershipMessageWithPathElement(0, abi.encodePacked(wrongRouter)),
            expectedRevert: abi.encodeWithSelector(
                IBesuLightClientErrors.CounterpartyRouterMismatch.selector, fixture.routerAddress, wrongRouter
            ),
            expectedTimestamp: 0
        });

        // A 32-byte ABI-encoded address is not accepted in place of the raw 20-byte address.
        testCases[11] = BesuMembershipTestCase({
            name: "failure: non-raw counterparty router",
            timestamp: timestamp,
            message: _membershipMessageWithPathElement(0, abi.encode(fixture.routerAddress)),
            expectedRevert: abi.encodeWithSelector(IBesuLightClientErrors.InvalidPathElementLength.selector, 20, 32),
            expectedTimestamp: 0
        });

        bytes32 wrongSlot = IBCSTORE_STORAGE_SLOT ^ bytes32(uint256(1));
        testCases[12] = BesuMembershipTestCase({
            name: "failure: wrong storage slot",
            timestamp: timestamp,
            message: _membershipMessageWithPathElement(1, abi.encodePacked(wrongSlot)),
            expectedRevert: abi.encodeWithSelector(
                IBesuLightClientErrors.StorageSlotMismatch.selector, IBCSTORE_STORAGE_SLOT, wrongSlot
            ),
            expectedTimestamp: 0
        });

        testCases[13] = BesuMembershipTestCase({
            name: "failure: wrong storage slot length",
            timestamp: timestamp,
            message: _membershipMessageWithPathElement(1, abi.encodePacked(bytes31(IBCSTORE_STORAGE_SLOT))),
            expectedRevert: abi.encodeWithSelector(IBesuLightClientErrors.InvalidPathElementLength.selector, 32, 31),
            expectedTimestamp: 0
        });
    }

    function fixtureNonMembership() public view returns (BesuNonMembershipTestCase[] memory testCases) {
        uint64 proofHeight = fixture.nonMembership.proofHeight;
        IBesuLightClientMsgs.ConsensusState memory expectedPreimage = _provenConsensusState(proofHeight);

        uint64 timestamp = fixture.initialTrustedTimestamp + 1;
        testCases = new BesuNonMembershipTestCase[](10);
        testCases[0] = BesuNonMembershipTestCase({
            name: "success",
            timestamp: timestamp,
            message: _nonMembershipMessage(proofHeight),
            expectedRevert: "",
            expectedTimestamp: fixture.nonMembership.expectedTimestamp
        });

        ILightClientMsgs.MsgVerifyNonMembership memory wrongRevisionMessage = _nonMembershipMessage(proofHeight);
        wrongRevisionMessage.proofHeight.revisionNumber = 1;
        testCases[1] = BesuNonMembershipTestCase({
            name: "failure: wrong revision number",
            timestamp: timestamp,
            message: wrongRevisionMessage,
            expectedRevert: abi.encodeWithSelector(IBesuLightClientErrors.InvalidRevisionNumber.selector, 1),
            expectedTimestamp: 0
        });

        ILightClientMsgs.MsgVerifyNonMembership memory slotlessPathMessage = _nonMembershipMessage(proofHeight);
        slotlessPathMessage.path = new bytes[](2);
        slotlessPathMessage.path[0] = abi.encodePacked(fixture.routerAddress);
        slotlessPathMessage.path[1] = fixture.nonMembership.path;
        testCases[2] = BesuNonMembershipTestCase({
            name: "failure: path without storage slot",
            timestamp: timestamp,
            message: slotlessPathMessage,
            expectedRevert: abi.encodeWithSelector(IBesuLightClientErrors.InvalidPathLength.selector, 3, 2),
            expectedTimestamp: 0
        });

        IBesuLightClientMsgs.ConsensusState memory tamperedPreimage = _provenConsensusState(proofHeight);
        tamperedPreimage.stateRoot = bytes32(uint256(tamperedPreimage.stateRoot) ^ 1);
        testCases[3] = BesuNonMembershipTestCase({
            name: "failure: wrong consensus state preimage",
            timestamp: timestamp,
            message: _nonMembershipMessageWithProof(_encodeProof(fixture.nonMembership, tamperedPreimage), proofHeight),
            expectedRevert: abi.encodeWithSelector(
                IBesuLightClientErrors.ConsensusStatePreimageMismatch.selector,
                _consensusStateHash(expectedPreimage),
                _consensusStateHash(tamperedPreimage)
            ),
            expectedTimestamp: 0
        });

        uint64 unknownHeight = proofHeight + 1000;
        ILightClientMsgs.MsgVerifyNonMembership memory unknownHeightMessage = _nonMembershipMessage(proofHeight);
        unknownHeightMessage.proofHeight.revisionHeight = unknownHeight;
        testCases[4] = BesuNonMembershipTestCase({
            name: "failure: unknown proof height",
            timestamp: timestamp,
            message: unknownHeightMessage,
            expectedRevert: abi.encodeWithSelector(
                IBesuLightClientErrors.ConsensusStateNotFound.selector, unknownHeight
            ),
            expectedTimestamp: 0
        });

        testCases[5] = BesuNonMembershipTestCase({
            name: "failure: storage root not cached",
            timestamp: timestamp,
            message: _nonMembershipMessageWithProof(_cachedProof(fixture.nonMembership, expectedPreimage), proofHeight),
            expectedRevert: abi.encodeWithSelector(IBesuLightClientErrors.StorageRootNotInCache.selector, proofHeight),
            expectedTimestamp: 0
        });

        uint64 expiredAt = expectedPreimage.timestamp + fixture.trustingPeriod;
        testCases[6] = BesuNonMembershipTestCase({
            name: "failure: expired consensus state",
            timestamp: expiredAt,
            message: _nonMembershipMessage(proofHeight),
            expectedRevert: abi.encodeWithSelector(
                IBesuLightClientErrors.ConsensusStateExpired.selector,
                expectedPreimage.timestamp,
                expiredAt,
                fixture.trustingPeriod
            ),
            expectedTimestamp: 0
        });

        // The membership proof is a valid inclusion proof, so it cannot prove its key absent.
        ILightClientMsgs.MsgVerifyNonMembership memory existingKeyMessage =
            _nonMembershipMessageWithProof(_encodeProof(fixture.membership, expectedPreimage), proofHeight);
        existingKeyMessage.path = _prefixedPath(fixture.membership.path);
        testCases[7] = BesuNonMembershipTestCase({
            name: "failure: key exists",
            timestamp: timestamp,
            message: existingKeyMessage,
            expectedRevert: abi.encodeWithSelector(IBesuLightClientErrors.InvalidExclusionProof.selector),
            expectedTimestamp: 0
        });

        address wrongRouter = address(uint160(fixture.routerAddress) ^ 1);
        ILightClientMsgs.MsgVerifyNonMembership memory wrongRouterMessage = _nonMembershipMessage(proofHeight);
        wrongRouterMessage.path[0] = abi.encodePacked(wrongRouter);
        testCases[8] = BesuNonMembershipTestCase({
            name: "failure: wrong counterparty router",
            timestamp: timestamp,
            message: wrongRouterMessage,
            expectedRevert: abi.encodeWithSelector(
                IBesuLightClientErrors.CounterpartyRouterMismatch.selector, fixture.routerAddress, wrongRouter
            ),
            expectedTimestamp: 0
        });

        bytes32 wrongSlot = IBCSTORE_STORAGE_SLOT ^ bytes32(uint256(1));
        ILightClientMsgs.MsgVerifyNonMembership memory wrongSlotMessage = _nonMembershipMessage(proofHeight);
        wrongSlotMessage.path[1] = abi.encodePacked(wrongSlot);
        testCases[9] = BesuNonMembershipTestCase({
            name: "failure: wrong storage slot",
            timestamp: timestamp,
            message: wrongSlotMessage,
            expectedRevert: abi.encodeWithSelector(
                IBesuLightClientErrors.StorageSlotMismatch.selector, IBCSTORE_STORAGE_SLOT, wrongSlot
            ),
            expectedTimestamp: 0
        });
    }

    function fixtureCachedStorageRoot() public view returns (BesuCachedStorageRootTestCase[] memory testCases) {
        uint64 proofHeight = fixture.nonMembership.proofHeight;
        IBesuLightClientMsgs.ConsensusState memory preimage = _provenConsensusState(proofHeight);
        bytes memory cachedMembershipProof = _cachedProof(fixture.membership, preimage);
        bytes memory cachedNonMembershipProof = _cachedProof(fixture.nonMembership, preimage);

        testCases = new BesuCachedStorageRootTestCase[](6);
        testCases[0] = BesuCachedStorageRootTestCase({
            name: "success: membership primes membership",
            primeWithMembership: true,
            verifyMembership: true,
            proof: cachedMembershipProof,
            proofHeight: proofHeight,
            expectedRevert: "",
            expectedTimestamp: fixture.membership.expectedTimestamp
        });
        testCases[1] = BesuCachedStorageRootTestCase({
            name: "success: membership primes non-membership",
            primeWithMembership: true,
            verifyMembership: false,
            proof: cachedNonMembershipProof,
            proofHeight: proofHeight,
            expectedRevert: "",
            expectedTimestamp: fixture.nonMembership.expectedTimestamp
        });
        testCases[2] = BesuCachedStorageRootTestCase({
            name: "success: non-membership primes membership",
            primeWithMembership: false,
            verifyMembership: true,
            proof: cachedMembershipProof,
            proofHeight: proofHeight,
            expectedRevert: "",
            expectedTimestamp: fixture.membership.expectedTimestamp
        });
        testCases[3] = BesuCachedStorageRootTestCase({
            name: "success: non-membership primes non-membership",
            primeWithMembership: false,
            verifyMembership: false,
            proof: cachedNonMembershipProof,
            proofHeight: proofHeight,
            expectedRevert: "",
            expectedTimestamp: fixture.nonMembership.expectedTimestamp
        });

        // The cache is keyed by height, so a root cached at one height must not serve another trusted height.
        uint64 otherHeight = fixture.adjacentUpdate.height;
        testCases[4] = BesuCachedStorageRootTestCase({
            name: "failure: other height not cached",
            primeWithMembership: false,
            verifyMembership: false,
            proof: _cachedProof(fixture.nonMembership, _expectedConsensusState(fixture.adjacentUpdate)),
            proofHeight: otherHeight,
            expectedRevert: abi.encodeWithSelector(IBesuLightClientErrors.StorageRootNotInCache.selector, otherHeight),
            expectedTimestamp: 0
        });

        // A cached root never bypasses verification of an account proof that is actually supplied.
        testCases[5] = BesuCachedStorageRootTestCase({
            name: "failure: tampered account proof after caching",
            primeWithMembership: true,
            verifyMembership: true,
            proof: _encodeProof(fixture.membership, preimage, _tamperedAccountProof()),
            proofHeight: proofHeight,
            expectedRevert: abi.encodeWithSelector(
                TrieProof.TrieProofTraversalError.selector, TrieProof.ProofError.INVALID_ROOT
            ),
            expectedTimestamp: 0
        });
    }

    function fixtureUpdate() public view returns (BesuUpdateTestCase[] memory testCases) {
        BesuUpdateFixture memory emptyExpectedState;

        testCases = new BesuUpdateTestCase[](36);
        testCases[0] = BesuUpdateTestCase({
            name: "success: valid adjacent update",
            timestamp: fixture.initialTrustedTimestamp + 1,
            update: _encodeUpdate(fixture.adjacentUpdate),
            preUpdate: "",
            expectedRevert: "",
            expectedState: fixture.adjacentUpdate
        });
        testCases[1] = BesuUpdateTestCase({
            name: "success: valid non-adjacent update",
            timestamp: fixture.initialTrustedTimestamp + 1,
            update: _encodeUpdate(fixture.nonAdjacentUpdate),
            preUpdate: "",
            expectedRevert: "",
            expectedState: fixture.nonAdjacentUpdate
        });
        testCases[2] = BesuUpdateTestCase({
            name: "failure: zero timestamp",
            timestamp: fixture.initialTrustedTimestamp + 1,
            update: _headerItemUpdate(11, 0),
            preUpdate: "",
            expectedRevert: abi.encodeWithSelector(IBesuLightClientErrors.InvalidHeaderTimestamp.selector),
            expectedState: emptyExpectedState
        });
        testCases[3] = BesuUpdateTestCase({
            name: "failure: expired trusted state",
            timestamp: fixture.initialTrustedTimestamp + fixture.trustingPeriod + 1,
            update: _encodeUpdate(fixture.nonAdjacentUpdate),
            preUpdate: "",
            expectedRevert: abi.encodeWithSelector(
                IBesuLightClientErrors.ConsensusStateExpired.selector,
                fixture.initialTrustedTimestamp,
                fixture.initialTrustedTimestamp + fixture.trustingPeriod + 1,
                fixture.trustingPeriod
            ),
            expectedState: emptyExpectedState
        });
        testCases[4] = BesuUpdateTestCase({
            name: "failure: trusted state expired at boundary",
            timestamp: fixture.initialTrustedTimestamp + fixture.trustingPeriod,
            update: _encodeUpdate(fixture.nonAdjacentUpdate),
            preUpdate: "",
            expectedRevert: abi.encodeWithSelector(
                IBesuLightClientErrors.ConsensusStateExpired.selector,
                fixture.initialTrustedTimestamp,
                fixture.initialTrustedTimestamp + fixture.trustingPeriod,
                fixture.trustingPeriod
            ),
            expectedState: emptyExpectedState
        });
        testCases[5] = BesuUpdateTestCase({
            name: "failure: insufficient trusted overlap",
            timestamp: fixture.initialTrustedTimestamp + 1,
            update: _encodeUpdate(fixture.lowOverlapUpdate),
            preUpdate: "",
            expectedRevert: abi.encodeWithSelector(
                IBesuLightClientErrors.InsufficientTrustedValidatorOverlap.selector, 1, 2
            ),
            expectedState: emptyExpectedState
        });
        testCases[6] = BesuUpdateTestCase({
            // The low quorum fixture keeps 2 of 4 seals with an unchanged validator set: enough trusted overlap, as the
            // requirement is capped at floor(2n / 3) - 1, but short of the ceil(2n / 3) quorum.
            name: "failure: insufficient quorum with partial seals",
            timestamp: fixture.initialTrustedTimestamp + 1,
            update: _encodeUpdate(fixture.lowQuorumUpdate),
            preUpdate: "",
            expectedRevert: abi.encodeWithSelector(IBesuLightClientErrors.InsufficientValidatorQuorum.selector, 2, 3),
            expectedState: emptyExpectedState
        });

        IBesuLightClientMsgs.MsgUpdateClient memory wrongRevisionUpdate =
            abi.decode(_encodeUpdate(fixture.nonAdjacentUpdate), (IBesuLightClientMsgs.MsgUpdateClient));
        wrongRevisionUpdate.trustedHeight.revisionNumber = 1;

        testCases[7] = BesuUpdateTestCase({
            name: "failure: wrong revision number",
            timestamp: fixture.initialTrustedTimestamp + 1,
            update: abi.encode(wrongRevisionUpdate),
            preUpdate: "",
            expectedRevert: abi.encodeWithSelector(IBesuLightClientErrors.InvalidRevisionNumber.selector, 1),
            expectedState: emptyExpectedState
        });

        IBesuLightClientMsgs.MsgUpdateClient memory unknownTrustedHeightUpdate =
            abi.decode(_encodeUpdate(fixture.nonAdjacentUpdate), (IBesuLightClientMsgs.MsgUpdateClient));
        unknownTrustedHeightUpdate.trustedHeight.revisionHeight = fixture.nonAdjacentUpdate.height - 1;

        testCases[8] = BesuUpdateTestCase({
            name: "failure: unknown trusted height",
            timestamp: fixture.initialTrustedTimestamp + 1,
            update: abi.encode(unknownTrustedHeightUpdate),
            preUpdate: "",
            expectedRevert: abi.encodeWithSelector(
                IBesuLightClientErrors.ConsensusStateNotFound.selector, fixture.nonAdjacentUpdate.height - 1
            ),
            expectedState: emptyExpectedState
        });

        IBesuLightClientMsgs.MsgUpdateClient memory wrongPreimageUpdate =
            abi.decode(_encodeUpdate(fixture.nonAdjacentUpdate), (IBesuLightClientMsgs.MsgUpdateClient));
        wrongPreimageUpdate.consensusStatePreimage.validators[0] = address(0xdead);

        testCases[9] = BesuUpdateTestCase({
            name: "failure: wrong consensus state preimage",
            timestamp: fixture.initialTrustedTimestamp + 1,
            update: abi.encode(wrongPreimageUpdate),
            preUpdate: "",
            expectedRevert: abi.encodeWithSelector(
                IBesuLightClientErrors.ConsensusStatePreimageMismatch.selector,
                _consensusStateHash(_initialConsensusState()),
                _consensusStateHash(wrongPreimageUpdate.consensusStatePreimage)
            ),
            expectedState: emptyExpectedState
        });

        testCases[10] = BesuUpdateTestCase({
            name: "failure: empty validators",
            timestamp: fixture.initialTrustedTimestamp + 1,
            update: _validatorsUpdate(new address[](0)),
            preUpdate: "",
            expectedRevert: abi.encodeWithSelector(IBesuLightClientErrors.EmptyValidatorSet.selector),
            expectedState: emptyExpectedState
        });

        address[] memory validators = fixture.nonAdjacentUpdate.expectedValidators;
        validators[0] = address(0);
        testCases[11] = BesuUpdateTestCase({
            name: "failure: zero validator",
            timestamp: fixture.initialTrustedTimestamp + 1,
            update: _validatorsUpdate(validators),
            preUpdate: "",
            expectedRevert: abi.encodeWithSelector(IBesuLightClientErrors.InvalidValidatorAddress.selector, address(0)),
            expectedState: emptyExpectedState
        });

        validators = fixture.nonAdjacentUpdate.expectedValidators;
        (validators[1], validators[2]) = (validators[2], validators[1]);
        testCases[12] = BesuUpdateTestCase({
            name: "failure: descending validators",
            timestamp: fixture.initialTrustedTimestamp + 1,
            update: _validatorsUpdate(validators),
            preUpdate: "",
            expectedRevert: abi.encodeWithSelector(IBesuLightClientErrors.UnsortedValidatorSet.selector, 1),
            expectedState: emptyExpectedState
        });

        validators = fixture.nonAdjacentUpdate.expectedValidators;
        validators[2] = validators[1];
        testCases[13] = BesuUpdateTestCase({
            name: "failure: duplicate validators",
            timestamp: fixture.initialTrustedTimestamp + 1,
            update: _validatorsUpdate(validators),
            preUpdate: "",
            expectedRevert: abi.encodeWithSelector(IBesuLightClientErrors.UnsortedValidatorSet.selector, 1),
            expectedState: emptyExpectedState
        });
        (bytes memory unknownSignerUpdate, address unknownSigner) = _unknownSignerUpdate();
        testCases[14] = BesuUpdateTestCase({
            name: "failure: unknown commit seal signer",
            timestamp: fixture.initialTrustedTimestamp + 1,
            update: unknownSignerUpdate,
            preUpdate: "",
            expectedRevert: abi.encodeWithSelector(
                IBesuLightClientErrors.UnknownCommitSealSigner.selector, unknownSigner
            ),
            expectedState: emptyExpectedState
        });

        testCases[15] = BesuUpdateTestCase({
            name: "failure: unsorted commit seal signers",
            timestamp: fixture.initialTrustedTimestamp + 1,
            update: _swappedSealsUpdate(),
            preUpdate: "",
            expectedRevert: abi.encodeWithSelector(IBesuLightClientErrors.UnsortedCommitSealSigners.selector, 0),
            expectedState: emptyExpectedState
        });

        uint256 overflow = uint256(type(uint64).max) + 1;
        testCases[16] = BesuUpdateTestCase({
            name: "failure: height overflows uint64",
            timestamp: fixture.initialTrustedTimestamp + 1,
            update: _headerItemUpdate(8, overflow),
            preUpdate: "",
            expectedRevert: abi.encodeWithSelector(SafeCast.SafeCastOverflowedUintDowncast.selector, 64, overflow),
            expectedState: emptyExpectedState
        });
        testCases[17] = BesuUpdateTestCase({
            name: "failure: timestamp overflows uint64",
            timestamp: fixture.initialTrustedTimestamp + 1,
            update: _headerItemUpdate(11, overflow),
            preUpdate: "",
            expectedRevert: abi.encodeWithSelector(SafeCast.SafeCastOverflowedUintDowncast.selector, 64, overflow),
            expectedState: emptyExpectedState
        });

        uint64 height = fixture.nonAdjacentUpdate.height;
        IBesuLightClientMsgs.MsgUpdateClient memory sameHeightUpdate =
            abi.decode(_encodeUpdate(fixture.nonAdjacentUpdate), (IBesuLightClientMsgs.MsgUpdateClient));
        sameHeightUpdate.trustedHeight.revisionHeight = height;
        testCases[18] = BesuUpdateTestCase({
            name: "failure: trusted height equals header height",
            timestamp: fixture.initialTrustedTimestamp + 1,
            update: abi.encode(sameHeightUpdate),
            preUpdate: "",
            expectedRevert: abi.encodeWithSelector(
                IBesuLightClientErrors.InvalidTrustedHeight.selector, height, height
            ),
            expectedState: emptyExpectedState
        });

        IBesuLightClientMsgs.MsgUpdateClient memory higherTrustedHeightUpdate =
            abi.decode(_encodeUpdate(fixture.adjacentUpdate), (IBesuLightClientMsgs.MsgUpdateClient));
        higherTrustedHeightUpdate.trustedHeight.revisionHeight = height;
        higherTrustedHeightUpdate.consensusStatePreimage = _expectedConsensusState(fixture.nonAdjacentUpdate);
        testCases[19] = BesuUpdateTestCase({
            name: "failure: trusted height above header height",
            timestamp: fixture.initialTrustedTimestamp + 1,
            update: abi.encode(higherTrustedHeightUpdate),
            preUpdate: _encodeUpdate(fixture.nonAdjacentUpdate),
            expectedRevert: abi.encodeWithSelector(
                IBesuLightClientErrors.InvalidTrustedHeight.selector, height, fixture.adjacentUpdate.height
            ),
            expectedState: emptyExpectedState
        });

        testCases[20] = BesuUpdateTestCase({
            name: "failure: zero height",
            timestamp: fixture.initialTrustedTimestamp + 1,
            update: _headerItemUpdate(8, 0),
            preUpdate: "",
            expectedRevert: abi.encodeWithSelector(IBesuLightClientErrors.InvalidHeaderHeight.selector),
            expectedState: emptyExpectedState
        });
        testCases[21] = BesuUpdateTestCase({
            name: "failure: wrong ommers hash",
            timestamp: fixture.initialTrustedTimestamp + 1,
            update: _headerItemUpdate(1, 0),
            preUpdate: "",
            expectedRevert: abi.encodeWithSelector(IBesuLightClientErrors.InvalidOmmersHash.selector, 0),
            expectedState: emptyExpectedState
        });
        testCases[22] = BesuUpdateTestCase({
            name: "failure: wrong difficulty",
            timestamp: fixture.initialTrustedTimestamp + 1,
            update: _headerItemUpdate(7, 2),
            preUpdate: "",
            expectedRevert: abi.encodeWithSelector(IBesuLightClientErrors.InvalidDifficulty.selector, 2),
            expectedState: emptyExpectedState
        });
        testCases[23] = BesuUpdateTestCase({
            name: "failure: wrong mix hash",
            timestamp: fixture.initialTrustedTimestamp + 1,
            update: _headerItemUpdate(13, 0),
            preUpdate: "",
            expectedRevert: abi.encodeWithSelector(IBesuLightClientErrors.InvalidMixHash.selector, 0),
            expectedState: emptyExpectedState
        });
        testCases[24] = BesuUpdateTestCase({
            name: "failure: wrong nonce",
            timestamp: fixture.initialTrustedTimestamp + 1,
            update: _headerItemUpdate(14, 1),
            preUpdate: "",
            expectedRevert: abi.encodeWithSelector(IBesuLightClientErrors.InvalidNonce.selector, hex"01"),
            expectedState: emptyExpectedState
        });

        bytes memory seal = _firstSeal();
        testCases[25] = BesuUpdateTestCase({
            name: "failure: short commit seal",
            timestamp: fixture.initialTrustedTimestamp + 1,
            update: _firstSealUpdate(seal.asSlice().slice(0, 64).toBytes()),
            preUpdate: "",
            expectedRevert: abi.encodeWithSelector(IBesuLightClientErrors.InvalidECDSASignatureLength.selector, 64),
            expectedState: emptyExpectedState
        });
        testCases[26] = BesuUpdateTestCase({
            name: "failure: zero commit seal",
            timestamp: fixture.initialTrustedTimestamp + 1,
            update: _firstSealUpdate(new bytes(65)),
            preUpdate: "",
            expectedRevert: abi.encodeWithSelector(ECDSA.ECDSAInvalidSignature.selector),
            expectedState: emptyExpectedState
        });

        // The malleable twin of a valid seal recovers the same signer but must be rejected.
        bytes32 highS = bytes32(SECP256K1_N - uint256(seal.asSlice().load(32)));
        testCases[27] = BesuUpdateTestCase({
            name: "failure: high-s commit seal",
            timestamp: fixture.initialTrustedTimestamp + 1,
            update: _firstSealUpdate(abi.encodePacked(seal.asSlice().load(0), highS, bytes1(55 - uint8(seal[64])))),
            preUpdate: "",
            expectedRevert: abi.encodeWithSelector(ECDSA.ECDSAInvalidSignatureS.selector, highS),
            expectedState: emptyExpectedState
        });

        // Relayers must convert Besu's raw 0/1 recovery ids to 27/28.
        testCases[30] = BesuUpdateTestCase({
            name: "failure: raw recovery id commit seal",
            timestamp: fixture.initialTrustedTimestamp + 1,
            update: _firstSealUpdate(
                abi.encodePacked(seal.asSlice().slice(0, 64).toBytes(), bytes1(uint8(seal[64]) - 27))
            ),
            preUpdate: "",
            expectedRevert: abi.encodeWithSelector(ECDSA.ECDSAInvalidSignature.selector),
            expectedState: emptyExpectedState
        });
        testCases[28] = BesuUpdateTestCase({
            name: "failure: zero state root",
            timestamp: fixture.initialTrustedTimestamp + 1,
            update: _headerItemUpdate(3, 0),
            preUpdate: "",
            expectedRevert: abi.encodeWithSelector(IBesuLightClientErrors.InvalidStateRoot.selector, bytes32(0)),
            expectedState: emptyExpectedState
        });
        testCases[29] = BesuUpdateTestCase({
            name: "failure: empty trie state root",
            timestamp: fixture.initialTrustedTimestamp + 1,
            update: _headerItemUpdate(3, uint256(TrieProof.EMPTY_ROOT_HASH)),
            preUpdate: "",
            expectedRevert: abi.encodeWithSelector(
                IBesuLightClientErrors.InvalidStateRoot.selector, TrieProof.EMPTY_ROOT_HASH
            ),
            expectedState: emptyExpectedState
        });

        _setCommitSealUpdateCases(testCases, 31);
    }

    /// @dev Fills `testCases[offset:offset + 5]` with commit seal and `extraData` shape failures. Kept apart from
    /// `fixtureUpdate` to stay within the stack limit when compiled without via-IR, as `forge coverage` does.
    function _setCommitSealUpdateCases(BesuUpdateTestCase[] memory testCases, uint256 offset) internal view {
        BesuUpdateFixture memory emptyExpectedState;

        BesuUpdateFixture memory noSealsUpdate = fixture.nonAdjacentUpdate;
        noSealsUpdate.commitSeals = new bytes[](0);
        testCases[offset] = BesuUpdateTestCase({
            name: "failure: empty commit seals",
            timestamp: fixture.initialTrustedTimestamp + 1,
            update: _encodeUpdate(noSealsUpdate),
            preUpdate: "",
            expectedRevert: abi.encodeWithSelector(IBesuLightClientErrors.EmptyCommitSeals.selector),
            expectedState: emptyExpectedState
        });

        bytes[] memory extraItems = _signingExtraItems(3);
        testCases[offset + 1] = BesuUpdateTestCase({
            name: "failure: three extraData items",
            timestamp: fixture.initialTrustedTimestamp + 1,
            update: _extraDataUpdate(extraItems),
            preUpdate: "",
            expectedRevert: abi.encodeWithSelector(IBesuLightClientErrors.InvalidExtraDataFormat.selector, 3),
            expectedState: emptyExpectedState
        });

        extraItems = _signingExtraItems(6);
        extraItems[4] = hex"c0";
        extraItems[5] = hex"c0";
        testCases[offset + 2] = BesuUpdateTestCase({
            name: "failure: six extraData items",
            timestamp: fixture.initialTrustedTimestamp + 1,
            update: _extraDataUpdate(extraItems),
            preUpdate: "",
            expectedRevert: abi.encodeWithSelector(IBesuLightClientErrors.InvalidExtraDataFormat.selector, 6),
            expectedState: emptyExpectedState
        });

        // A header submitted with its commit seals still in `extraData`, as Besu returns it.
        bytes[] memory encodedSeals = new bytes[](fixture.nonAdjacentUpdate.commitSeals.length);
        for (uint256 i = 0; i < encodedSeals.length; ++i) {
            encodedSeals[i] = RLP.encode(fixture.nonAdjacentUpdate.commitSeals[i]);
        }
        extraItems = _signingExtraItems(5);
        extraItems[4] = RLP.encode(encodedSeals);
        testCases[offset + 3] = BesuUpdateTestCase({
            name: "failure: commit seals in extraData",
            timestamp: fixture.initialTrustedTimestamp + 1,
            update: _extraDataUpdate(extraItems),
            preUpdate: "",
            expectedRevert: abi.encodeWithSelector(IBesuLightClientErrors.NonEmptyCommitSealList.selector),
            expectedState: emptyExpectedState
        });

        extraItems = _signingExtraItems(5);
        extraItems[4] = hex"80";
        testCases[offset + 4] = BesuUpdateTestCase({
            name: "failure: empty string as commit seal list",
            timestamp: fixture.initialTrustedTimestamp + 1,
            update: _extraDataUpdate(extraItems),
            preUpdate: "",
            expectedRevert: abi.encodeWithSelector(IBesuLightClientErrors.NonEmptyCommitSealList.selector),
            expectedState: emptyExpectedState
        });
    }

    function fixtureMisbehaviour() public view returns (BesuMisbehaviourTestCase[] memory testCases) {
        uint64 height1 = fixture.initialTrustedHeight;
        uint64 height2 = fixture.nonAdjacentUpdate.height;
        IBesuLightClientMsgs.ConsensusState memory state1 = _initialConsensusState();
        IBesuLightClientMsgs.ConsensusState memory state2 = _expectedConsensusState(fixture.nonAdjacentUpdate);
        uint64 timestamp = fixture.initialTrustedTimestamp + 1;

        testCases = new BesuMisbehaviourTestCase[](8);
        testCases[0] = BesuMisbehaviourTestCase({
            name: "failure: monotonic timestamps",
            timestamp: timestamp,
            misbehaviour: _encodeMisbehaviour(_misbehaviourMsg(height1, state1, height2, state2)),
            expectedRevert: abi.encodeWithSelector(IBesuLightClientErrors.NoMisbehaviourDetected.selector)
        });
        testCases[1] = BesuMisbehaviourTestCase({
            name: "failure: equal heights",
            timestamp: timestamp,
            misbehaviour: _encodeMisbehaviour(_misbehaviourMsg(height2, state2, height2, state2)),
            expectedRevert: abi.encodeWithSelector(IBesuLightClientErrors.NoMisbehaviourDetected.selector)
        });
        testCases[2] = BesuMisbehaviourTestCase({
            name: "failure: monotonic timestamps in descending height order",
            timestamp: timestamp,
            misbehaviour: _encodeMisbehaviour(_misbehaviourMsg(height2, state2, height1, state1)),
            expectedRevert: abi.encodeWithSelector(IBesuLightClientErrors.NoMisbehaviourDetected.selector)
        });

        IBesuLightClientMsgs.MsgConflictingConsensusStatesMisbehaviour memory wrongRevision =
            _misbehaviourMsg(height1, state1, height2, state2);
        wrongRevision.height1.revisionNumber = 1;
        testCases[3] = BesuMisbehaviourTestCase({
            name: "failure: wrong revision number for height1",
            timestamp: timestamp,
            misbehaviour: _encodeMisbehaviour(wrongRevision),
            expectedRevert: abi.encodeWithSelector(IBesuLightClientErrors.InvalidRevisionNumber.selector, 1)
        });

        wrongRevision = _misbehaviourMsg(height1, state1, height2, state2);
        wrongRevision.height2.revisionNumber = 1;
        testCases[4] = BesuMisbehaviourTestCase({
            name: "failure: wrong revision number for height2",
            timestamp: timestamp,
            misbehaviour: _encodeMisbehaviour(wrongRevision),
            expectedRevert: abi.encodeWithSelector(IBesuLightClientErrors.InvalidRevisionNumber.selector, 1)
        });
        testCases[5] = BesuMisbehaviourTestCase({
            name: "failure: unknown height",
            timestamp: timestamp,
            misbehaviour: _encodeMisbehaviour(_misbehaviourMsg(height1, state1, height2 + 1, state2)),
            expectedRevert: abi.encodeWithSelector(IBesuLightClientErrors.ConsensusStateNotFound.selector, height2 + 1)
        });

        IBesuLightClientMsgs.ConsensusState memory wrongState2 = _expectedConsensusState(fixture.nonAdjacentUpdate);
        wrongState2.timestamp = state1.timestamp;
        testCases[6] = BesuMisbehaviourTestCase({
            name: "failure: wrong consensus state preimage",
            timestamp: timestamp,
            misbehaviour: _encodeMisbehaviour(_misbehaviourMsg(height1, state1, height2, wrongState2)),
            expectedRevert: abi.encodeWithSelector(
                IBesuLightClientErrors.ConsensusStatePreimageMismatch.selector,
                _consensusStateHash(state2),
                _consensusStateHash(wrongState2)
            )
        });

        // Stored states remain valid evidence after their trusting period, so the timestamps are still compared.
        testCases[7] = BesuMisbehaviourTestCase({
            name: "failure: monotonic timestamps past trusting period",
            timestamp: state2.timestamp + fixture.trustingPeriod,
            misbehaviour: _encodeMisbehaviour(_misbehaviourMsg(height1, state1, height2, state2)),
            expectedRevert: abi.encodeWithSelector(IBesuLightClientErrors.NoMisbehaviourDetected.selector)
        });
    }

    /// @dev Adds a commit seal from a non-validator key to `nonAdjacentUpdate` and returns that signer.
    /// The seal is inserted at its sorted position.
    function _unknownSignerUpdate() internal view returns (bytes memory, address) {
        BesuUpdateFixture memory update = fixture.nonAdjacentUpdate;
        bytes32 digest = keccak256(update.headerRlp);
        uint256 unknownSignerKey = 0xdead;
        address unknownSigner = vm.addr(unknownSignerKey);
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(unknownSignerKey, digest);

        bytes[] memory seals = new bytes[](update.commitSeals.length + 1);
        uint256 offset = 0;
        for (uint256 i = 0; i < update.commitSeals.length; ++i) {
            if (offset == 0 && ECDSA.recover(digest, update.commitSeals[i]) > unknownSigner) {
                seals[i] = abi.encodePacked(r, s, v);
                offset = 1;
            }
            seals[i + offset] = update.commitSeals[i];
        }
        if (offset == 0) {
            seals[update.commitSeals.length] = abi.encodePacked(r, s, v);
        }
        update.commitSeals = seals;
        return (_encodeUpdate(update), unknownSigner);
    }

    /// @dev Swaps the first two commit seals of `nonAdjacentUpdate`, breaking their signer order.
    function _swappedSealsUpdate() internal view returns (bytes memory) {
        BesuUpdateFixture memory update = fixture.nonAdjacentUpdate;
        (update.commitSeals[0], update.commitSeals[1]) = (update.commitSeals[1], update.commitSeals[0]);
        return _encodeUpdate(update);
    }

    /// @dev First commit seal of `nonAdjacentUpdate`.
    function _firstSeal() internal view returns (bytes memory) {
        return fixture.nonAdjacentUpdate.commitSeals[0];
    }

    /// @dev Replaces the first commit seal of `nonAdjacentUpdate` with `seal`.
    function _firstSealUpdate(bytes memory seal) internal view returns (bytes memory) {
        BesuUpdateFixture memory update = fixture.nonAdjacentUpdate;
        update.commitSeals[0] = seal;
        return _encodeUpdate(update);
    }

    /// @dev Re-encodes the non-adjacent update header with `validators` and commit seals signed by `signerKeys`.
    function _signedValidatorsUpdate(
        address[] memory validators,
        uint256[] memory signerKeys
    )
        internal
        view
        returns (bytes memory)
    {
        BesuUpdateFixture memory update = fixture.nonAdjacentUpdate;
        update.headerRlp = _validatorsHeader(validators);

        // Seals must be ordered by signer address.
        for (uint256 i = 1; i < signerKeys.length; ++i) {
            for (uint256 j = i; j > 0 && vm.addr(signerKeys[j - 1]) > vm.addr(signerKeys[j]); --j) {
                (signerKeys[j - 1], signerKeys[j]) = (signerKeys[j], signerKeys[j - 1]);
            }
        }
        bytes32 digest = keccak256(update.headerRlp);
        update.commitSeals = new bytes[](signerKeys.length);
        for (uint256 i = 0; i < signerKeys.length; ++i) {
            (uint8 v, bytes32 r, bytes32 s) = vm.sign(signerKeys[i], digest);
            update.commitSeals[i] = abi.encodePacked(r, s, v);
        }
        return _encodeUpdate(update);
    }

    /// @dev Sorted addresses of the synthetic validator keys `1..count`.
    function _syntheticValidators(uint256 count) internal pure returns (address[] memory validators) {
        validators = new address[](count);
        for (uint256 i = 0; i < count; ++i) {
            address validator = vm.addr(i + 1);
            uint256 j = i;
            while (j > 0 && validators[j - 1] > validator) {
                validators[j] = validators[j - 1];
                --j;
            }
            validators[j] = validator;
        }
    }

    /// @dev Re-encodes a header with its `extraData` items replaced by `encodedExtraItems`.
    function _encodeHeader(
        Memory.Slice[] memory headerItems,
        bytes[] memory encodedExtraItems
    )
        internal
        pure
        returns (bytes memory)
    {
        bytes[] memory encodedHeaderItems = new bytes[](headerItems.length);
        for (uint256 i = 0; i < headerItems.length; ++i) {
            encodedHeaderItems[i] = i == 12 ? RLP.encode(RLP.encode(encodedExtraItems)) : headerItems[i].toBytes();
        }
        return RLP.encode(encodedHeaderItems);
    }

    /// @dev Array of `length` encoded items starting with the first four `extraData` items of the non-adjacent update
    /// header. Items past the fourth are left empty.
    function _signingExtraItems(uint256 length) internal view returns (bytes[] memory encodedExtraItems) {
        Memory.Slice[] memory headerItems = fixture.nonAdjacentUpdate.headerRlp.decodeList();
        Memory.Slice[] memory extraItems = RLP.readBytes(headerItems[12]).decodeList();
        encodedExtraItems = new bytes[](length);
        for (uint256 i = 0; i < length && i < 4; ++i) {
            encodedExtraItems[i] = extraItems[i].toBytes();
        }
    }

    /// @dev Re-encodes the non-adjacent update header with its `extraData` items replaced by `encodedExtraItems`.
    function _extraDataUpdate(bytes[] memory encodedExtraItems) internal view returns (bytes memory) {
        BesuUpdateFixture memory update = fixture.nonAdjacentUpdate;
        update.headerRlp = _encodeHeader(update.headerRlp.decodeList(), encodedExtraItems);
        return _encodeUpdate(update);
    }

    /// @dev Non-adjacent update header with its `extraData` validators replaced by `validators`.
    function _validatorsHeader(address[] memory validators) internal view returns (bytes memory) {
        Memory.Slice[] memory headerItems = fixture.nonAdjacentUpdate.headerRlp.decodeList();
        Memory.Slice[] memory extraItems = RLP.readBytes(headerItems[12]).decodeList();
        bytes[] memory encodedValidators = new bytes[](validators.length);
        for (uint256 i = 0; i < validators.length; ++i) {
            encodedValidators[i] = RLP.encode(abi.encodePacked(validators[i]));
        }
        bytes[] memory encodedExtraItems = new bytes[](extraItems.length);
        for (uint256 i = 0; i < extraItems.length; ++i) {
            encodedExtraItems[i] = i == 1 ? RLP.encode(encodedValidators) : extraItems[i].toBytes();
        }
        return _encodeHeader(headerItems, encodedExtraItems);
    }

    /// @dev Re-encodes the non-adjacent update header with `validators`, keeping its commit seals.
    function _validatorsUpdate(address[] memory validators) internal view returns (bytes memory) {
        BesuUpdateFixture memory update = fixture.nonAdjacentUpdate;
        update.headerRlp = _validatorsHeader(validators);
        return _encodeUpdate(update);
    }

    /// @dev Re-encodes the non-adjacent update header with `headerItems[index]` replaced by `value`.
    function _headerItemUpdate(uint256 index, uint256 value) internal view returns (bytes memory) {
        BesuUpdateFixture memory update = fixture.nonAdjacentUpdate;
        Memory.Slice[] memory headerItems = update.headerRlp.decodeList();
        bytes[] memory encodedHeaderItems = new bytes[](headerItems.length);
        for (uint256 i = 0; i < headerItems.length; ++i) {
            encodedHeaderItems[i] = i == index ? RLP.encode(value) : headerItems[i].toBytes();
        }
        update.headerRlp = RLP.encode(encodedHeaderItems);
        return _encodeUpdate(update);
    }

    function _clientState() internal view returns (IBesuLightClientMsgs.ClientState memory) {
        return abi.decode(client.getClientState(), (IBesuLightClientMsgs.ClientState));
    }

    function _assertClientState(BesuUpdateFixture memory update) internal view {
        IBesuLightClientMsgs.ClientState memory clientState = _clientState();
        assertEq(clientState.ibcRouter, fixture.routerAddress);
        assertEq(clientState.latestHeight.revisionNumber, 0);
        assertEq(clientState.latestHeight.revisionHeight, update.height);
        assertEq(clientState.trustingPeriod, fixture.trustingPeriod);
        assertEq(clientState.maxClockDrift, fixture.maxClockDrift);
        assertFalse(clientState.isFrozen);
        assertEq(clientState.trustLevel.numerator, trustLevel.numerator);
        assertEq(clientState.trustLevel.denominator, trustLevel.denominator);

        assertEq(client.getConsensusStateHash(update.height), _consensusStateHash(_expectedConsensusState(update)));
    }

    /// @dev Client state passed to the constructor, trusting `fixture.initialTrustedHeight`.
    function _initialClientState() internal view returns (IBesuLightClientMsgs.ClientState memory) {
        return IBesuLightClientMsgs.ClientState({
            ibcRouter: fixture.routerAddress,
            latestHeight: IICS02ClientMsgs.Height({ revisionNumber: 0, revisionHeight: fixture.initialTrustedHeight }),
            trustingPeriod: fixture.trustingPeriod,
            maxClockDrift: fixture.maxClockDrift,
            isFrozen: false,
            trustLevel: trustLevel
        });
    }

    /// @dev Consensus state committed by the constructor at `fixture.initialTrustedHeight`.
    function _initialConsensusState() internal view returns (IBesuLightClientMsgs.ConsensusState memory) {
        return IBesuLightClientMsgs.ConsensusState({
            timestamp: fixture.initialTrustedTimestamp,
            stateRoot: fixture.initialTrustedStateRoot,
            validators: fixture.initialTrustedValidators
        });
    }

    /// @dev Consensus state the client is expected to store after applying `update`.
    function _expectedConsensusState(BesuUpdateFixture memory update)
        internal
        pure
        returns (IBesuLightClientMsgs.ConsensusState memory)
    {
        return IBesuLightClientMsgs.ConsensusState({
            timestamp: update.expectedTimestamp,
            stateRoot: update.expectedStateRoot,
            validators: update.expectedValidators
        });
    }

    /// @dev Preimage of the trusted consensus state referenced by an update at `trustedHeight`.
    function _trustedConsensusState(uint64 trustedHeight)
        internal
        view
        returns (IBesuLightClientMsgs.ConsensusState memory)
    {
        assertEq(trustedHeight, fixture.initialTrustedHeight, "unsupported trusted height");
        return _initialConsensusState();
    }

    /// @dev Preimage of the consensus state that storage proofs at `proofHeight` are verified against.
    function _provenConsensusState(uint64 proofHeight)
        internal
        view
        returns (IBesuLightClientMsgs.ConsensusState memory)
    {
        assertEq(proofHeight, fixture.nonAdjacentUpdate.height, "unsupported proof height");
        return _expectedConsensusState(fixture.nonAdjacentUpdate);
    }

    function _consensusStateHash(IBesuLightClientMsgs.ConsensusState memory consensusState)
        internal
        pure
        returns (bytes32)
    {
        return keccak256(abi.encode(consensusState));
    }

    function _encodeUpdate(BesuUpdateFixture memory update) internal view returns (bytes memory) {
        return _encodeUpdate(update.headerRlp, update.commitSeals, update.trustedHeight);
    }

    function _encodeUpdate(BesuRejectionUpdateFixture memory update) internal view returns (bytes memory) {
        return _encodeUpdate(update.headerRlp, update.commitSeals, update.trustedHeight);
    }

    function _encodeUpdate(
        bytes memory headerRlp,
        bytes[] memory commitSeals,
        uint64 trustedHeight
    )
        internal
        view
        returns (bytes memory)
    {
        return abi.encode(
            IBesuLightClientMsgs.MsgUpdateClient({
                headerRlp: headerRlp,
                commitSeals: commitSeals,
                trustedHeight: IICS02ClientMsgs.Height({ revisionNumber: 0, revisionHeight: trustedHeight }),
                consensusStatePreimage: _trustedConsensusState(trustedHeight)
            })
        );
    }

    function _misbehaviourMsg(
        uint64 height1,
        IBesuLightClientMsgs.ConsensusState memory state1,
        uint64 height2,
        IBesuLightClientMsgs.ConsensusState memory state2
    )
        internal
        pure
        returns (IBesuLightClientMsgs.MsgConflictingConsensusStatesMisbehaviour memory)
    {
        return IBesuLightClientMsgs.MsgConflictingConsensusStatesMisbehaviour({
            height1: IICS02ClientMsgs.Height({ revisionNumber: 0, revisionHeight: height1 }),
            height2: IICS02ClientMsgs.Height({ revisionNumber: 0, revisionHeight: height2 }),
            consensusStatePreimage1: state1,
            consensusStatePreimage2: state2
        });
    }

    function _encodeMisbehaviour(IBesuLightClientMsgs.MsgConflictingConsensusStatesMisbehaviour memory misbehaviour)
        internal
        pure
        returns (bytes memory)
    {
        return abi.encode(
            IBesuLightClientMsgs.MsgSubmitMisbehaviour(
                IBesuLightClientMsgs.MisbehaviourType.ConflictingConsensusStates, abi.encode(misbehaviour)
            )
        );
    }

    /// @dev Wraps fixture account and storage proof nodes and a consensus state preimage into the client proof format.
    function _encodeProof(
        BesuProofFixture memory proofFixture,
        IBesuLightClientMsgs.ConsensusState memory preimage
    )
        internal
        pure
        returns (bytes memory)
    {
        return _encodeProof(proofFixture, preimage, abi.decode(proofFixture.accountProof, (bytes[])));
    }

    /// @dev Wraps fixture storage proof nodes, explicit account proof nodes, and a consensus state preimage into the
    /// client proof format.
    function _encodeProof(
        BesuProofFixture memory proofFixture,
        IBesuLightClientMsgs.ConsensusState memory preimage,
        bytes[] memory accountProofNodes
    )
        internal
        pure
        returns (bytes memory)
    {
        return abi.encode(
            IBesuLightClientMsgs.MembershipProof({
                consensusStatePreimage: preimage,
                accountProofNodes: accountProofNodes,
                proofNodes: abi.decode(proofFixture.proof, (bytes[]))
            })
        );
    }

    /// @dev Proof that omits the account proof and relies on the transiently cached storage root.
    function _cachedProof(
        BesuProofFixture memory proofFixture,
        IBesuLightClientMsgs.ConsensusState memory preimage
    )
        internal
        pure
        returns (bytes memory)
    {
        return _encodeProof(proofFixture, preimage, new bytes[](0));
    }

    /// @dev Fixture account proof whose root node no longer hashes to the trusted state root.
    function _tamperedAccountProof() internal view returns (bytes[] memory nodes) {
        nodes = abi.decode(fixture.membership.accountProof, (bytes[]));
        nodes[0][0] ^= 0x01;
    }

    /// @dev The router path for `path` under the fixture router's `[routerAddress, IBCSTORE_STORAGE_SLOT, ""]` merkle
    /// prefix.
    function _prefixedPath(bytes memory path) internal view returns (bytes[] memory) {
        bytes[] memory out = new bytes[](3);
        out[0] = abi.encodePacked(fixture.routerAddress);
        out[1] = abi.encodePacked(IBCSTORE_STORAGE_SLOT);
        out[2] = path;
        return out;
    }

    function _membershipMessage(
        uint64 revisionNumber,
        bytes[] memory path,
        bytes memory value
    )
        internal
        view
        returns (ILightClientMsgs.MsgVerifyMembership memory)
    {
        return ILightClientMsgs.MsgVerifyMembership({
            proof: _encodeProof(fixture.membership, _provenConsensusState(fixture.membership.proofHeight)),
            proofHeight: IICS02ClientMsgs.Height({
                revisionNumber: revisionNumber, revisionHeight: fixture.membership.proofHeight
            }),
            path: path,
            value: value
        });
    }

    /// @dev Fixture membership message whose prefixed path has the element at `index` replaced by `element`.
    function _membershipMessageWithPathElement(
        uint256 index,
        bytes memory element
    )
        internal
        view
        returns (ILightClientMsgs.MsgVerifyMembership memory message)
    {
        message = _membershipMessage(0, _prefixedPath(fixture.membership.path), fixture.membership.value);
        message.path[index] = element;
    }

    /// @dev Membership message for the fixture path and value that carries an explicit proof and proof height.
    function _membershipMessageWithProof(
        bytes memory proof,
        uint64 proofHeight
    )
        internal
        view
        returns (ILightClientMsgs.MsgVerifyMembership memory)
    {
        return ILightClientMsgs.MsgVerifyMembership({
            proof: proof,
            proofHeight: IICS02ClientMsgs.Height({ revisionNumber: 0, revisionHeight: proofHeight }),
            path: _prefixedPath(fixture.membership.path),
            value: fixture.membership.value
        });
    }

    function _nonMembershipMessage(uint64 proofHeight)
        internal
        view
        returns (ILightClientMsgs.MsgVerifyNonMembership memory)
    {
        return _nonMembershipMessageWithProof(
            _encodeProof(fixture.nonMembership, _provenConsensusState(proofHeight)), proofHeight
        );
    }

    /// @dev Non-membership message for the fixture path that carries an explicit proof and proof height.
    function _nonMembershipMessageWithProof(
        bytes memory proof,
        uint64 proofHeight
    )
        internal
        view
        returns (ILightClientMsgs.MsgVerifyNonMembership memory)
    {
        return ILightClientMsgs.MsgVerifyNonMembership({
            proof: proof,
            proofHeight: IICS02ClientMsgs.Height({ revisionNumber: 0, revisionHeight: proofHeight }),
            path: _prefixedPath(fixture.nonMembership.path)
        });
    }

    function _loadFixture(string memory fileName) internal view returns (BesuFixture memory) {
        string memory root = vm.projectRoot();
        string memory path = string.concat(root, FIXTURE_DIR, fileName);
        string memory json = vm.readFile(path);

        return BesuFixture({
            routerAddress: json.readAddress(".routerAddress"),
            initialTrustedHeight: uint64(json.readUint(".initialTrustedHeight")),
            initialTrustedTimestamp: uint64(json.readUint(".initialTrustedTimestamp")),
            initialTrustedStateRoot: json.readBytes32(".initialTrustedStateRoot"),
            initialTrustedValidators: abi.decode(json.parseRaw(".initialTrustedValidators"), (address[])),
            trustingPeriod: uint64(json.readUint(".trustingPeriod")),
            maxClockDrift: uint64(json.readUint(".maxClockDrift")),
            adjacentUpdate: _readUpdate(json, ".adjacentUpdate"),
            nonAdjacentUpdate: _readUpdate(json, ".nonAdjacentUpdate"),
            lowQuorumUpdate: _readRejectionUpdate(json, ".lowQuorumUpdate"),
            conflictingUpdate: _readRejectionUpdate(json, ".conflictingUpdate"),
            lowOverlapUpdate: _readRejectionUpdate(json, ".lowOverlapUpdate"),
            membership: _readProof(json, ".membership"),
            nonMembership: _readProof(json, ".nonMembership")
        });
    }

    function _readUpdate(string memory json, string memory path) internal pure returns (BesuUpdateFixture memory) {
        return BesuUpdateFixture({
            height: uint64(json.readUint(string.concat(path, ".height"))),
            headerRlp: json.readBytes(string.concat(path, ".headerRlp")),
            commitSeals: json.readBytesArray(string.concat(path, ".commitSeals")),
            trustedHeight: uint64(json.readUint(string.concat(path, ".trustedHeight"))),
            expectedTimestamp: uint64(json.readUint(string.concat(path, ".expectedTimestamp"))),
            expectedStateRoot: json.readBytes32(string.concat(path, ".expectedStateRoot")),
            expectedValidators: abi.decode(json.parseRaw(string.concat(path, ".expectedValidators")), (address[]))
        });
    }

    function _readRejectionUpdate(
        string memory json,
        string memory path
    )
        internal
        pure
        returns (BesuRejectionUpdateFixture memory)
    {
        return BesuRejectionUpdateFixture({
            height: uint64(json.readUint(string.concat(path, ".height"))),
            headerRlp: json.readBytes(string.concat(path, ".headerRlp")),
            commitSeals: json.readBytesArray(string.concat(path, ".commitSeals")),
            trustedHeight: uint64(json.readUint(string.concat(path, ".trustedHeight")))
        });
    }

    function _readProof(string memory json, string memory path) internal view returns (BesuProofFixture memory) {
        return BesuProofFixture({
            proof: json.readBytes(string.concat(path, ".proof")),
            accountProof: json.readBytes(string.concat(path, ".accountProof")),
            proofHeight: uint64(json.readUint(string.concat(path, ".proofHeight"))),
            path: json.readBytes(string.concat(path, ".path")),
            value: json.keyExists(string.concat(path, ".value"))
                ? json.readBytes(string.concat(path, ".value"))
                : bytes(""),
            expectedTimestamp: uint64(json.readUint(string.concat(path, ".expectedTimestamp")))
        });
    }

    function _deployClient(
        IBesuLightClientMsgs.ClientState memory clientState,
        IBesuLightClientMsgs.ConsensusState memory consensusState
    )
        internal
        returns (IBesuLightClient)
    {
        return new BesuLightClient(clientState, consensusState, ics26Router);
    }

    function _fixtureFile() internal pure virtual returns (string memory);
}
