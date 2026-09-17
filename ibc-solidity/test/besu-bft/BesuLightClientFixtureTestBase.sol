// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable gas-struct-packing,function-max-lines,gas-small-strings

import { Test } from "forge-std/Test.sol";
import { stdJson } from "forge-std/StdJson.sol";
import { RLP } from "@openzeppelin-contracts/utils/RLP.sol";
import { Memory } from "@openzeppelin-contracts/utils/Memory.sol";

import { ILightClientMsgs } from "../../contracts/msgs/ILightClientMsgs.sol";
import { IICS02ClientMsgs } from "../../contracts/msgs/IICS02ClientMsgs.sol";
import { BesuIBFT2LightClient } from "../../contracts/light-clients/besu/BesuIBFT2LightClient.sol";
import { BesuQBFTLightClient } from "../../contracts/light-clients/besu/BesuQBFTLightClient.sol";
import { IBesuLightClient } from "../../contracts/light-clients/besu/interfaces/IBesuLightClient.sol";
import { IBesuLightClientMsgs } from "../../contracts/light-clients/besu/msgs/IBesuLightClientMsgs.sol";
import { IBesuLightClientErrors } from "../../contracts/light-clients/besu/errors/IBesuLightClientErrors.sol";
import { TrieProof } from "../../contracts/utils/TrieProof.sol";

/// @dev Exposes the commit seal digest of a Besu light client so tests can sign synthetic seals.
interface IBesuCommitSealDigest {
    function commitSealDigest(bytes calldata headerRlp) external pure returns (bytes32);
}

contract BesuQBFTCommitSealDigest is BesuQBFTLightClient, IBesuCommitSealDigest {
    constructor(address[] memory validators) BesuQBFTLightClient(address(1), 1, 1, 0, validators, 1, 0, address(0)) { }

    function commitSealDigest(bytes calldata headerRlp) external pure returns (bytes32) {
        return _commitSealDigest(_parseHeader(headerRlp));
    }
}

contract BesuIBFT2CommitSealDigest is BesuIBFT2LightClient, IBesuCommitSealDigest {
    constructor(address[] memory validators) BesuIBFT2LightClient(address(1), 1, 1, 0, validators, 1, 0, address(0)) { }

    function commitSealDigest(bytes calldata headerRlp) external pure returns (bytes32) {
        return _commitSealDigest(_parseHeader(headerRlp));
    }
}

/// @dev Successful update input and expected consensus state.
struct BesuUpdateFixture {
    uint64 height;
    bytes headerRlp;
    uint64 trustedHeight;
    uint64 expectedTimestamp;
    bytes32 expectedStateRoot;
    address[] expectedValidators;
}

/// @dev Update input expected to be rejected.
struct BesuRejectionUpdateFixture {
    uint64 height;
    bytes headerRlp;
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

    string internal constant FIXTURE_DIR = "/test/besu-bft/fixtures/";
    /// @dev Private key of a signer that is not in any fixture validator set.
    uint256 internal constant UNKNOWN_SIGNER_KEY = 0xdead;

    BesuFixture internal fixture;
    IBesuLightClient internal client;
    IBesuLightClient internal wrongWrapper;
    IBesuCommitSealDigest internal commitSealDigest;

    function setUp() public virtual {
        fixture = _loadFixture(_fixtureFile());
        client = _deployPrimaryClient();
        wrongWrapper = _deployWrongWrapper();
        commitSealDigest = _deployCommitSealDigest();
    }

    function test_constructor_storesInitialConsensusStateHash() public view {
        assertEq(
            client.getConsensusStateHash(fixture.initialTrustedHeight), _consensusStateHash(_initialConsensusState())
        );
    }

    function test_getConsensusStateHash_revertUnknownHeight() public {
        uint64 unknownHeight = fixture.initialTrustedHeight + 1000;
        vm.expectRevert(abi.encodeWithSelector(IBesuLightClientErrors.ConsensusStateNotFound.selector, unknownHeight));
        client.getConsensusStateHash(unknownHeight);
    }

    function tableVerifyNonMembershipTest(BesuNonMembershipTestCase memory nonMembership) public {
        vm.warp(fixture.initialTrustedTimestamp + 1);
        client.updateClient(_encodeUpdate(fixture.nonAdjacentUpdate));
        vm.warp(nonMembership.timestamp);

        if (nonMembership.expectedRevert.length != 0) {
            vm.expectRevert(nonMembership.expectedRevert);
        }
        uint256 timestamp = client.verifyNonMembership(nonMembership.message);

        if (nonMembership.expectedRevert.length == 0) {
            assertEq(timestamp, nonMembership.expectedTimestamp);
        }
    }

    /// @dev Primes the transient storage root cache with one verified proof, then verifies a second proof that
    /// relies on the cache within the same transaction.
    function tableCachedStorageRootTest(BesuCachedStorageRootTestCase memory cachedStorageRoot) public {
        vm.warp(fixture.initialTrustedTimestamp + 1);
        client.updateClient(_encodeUpdate(fixture.adjacentUpdate));
        client.updateClient(_encodeUpdate(fixture.nonAdjacentUpdate));
        if (cachedStorageRoot.primeWithMembership) {
            client.verifyMembership(
                _membershipMessage(0, _singlePath(fixture.membership.path), fixture.membership.value)
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

        if (cachedStorageRoot.expectedRevert.length == 0) {
            assertEq(timestamp, cachedStorageRoot.expectedTimestamp);
        }
    }

    function test_updateClient_noOpOnSameState() public {
        vm.warp(fixture.initialTrustedTimestamp + 1);
        client.updateClient(_encodeUpdate(fixture.nonAdjacentUpdate));
        bytes32 storedHash = client.getConsensusStateHash(fixture.nonAdjacentUpdate.height);

        ILightClientMsgs.UpdateResult result = client.updateClient(_encodeUpdate(fixture.nonAdjacentUpdate));

        assertEq(uint8(result), uint8(ILightClientMsgs.UpdateResult.NoOp));
        assertEq(client.getConsensusStateHash(fixture.nonAdjacentUpdate.height), storedHash);
        _assertClientState(fixture.nonAdjacentUpdate);
    }

    function tableUpdateClientTest(BesuUpdateTestCase memory update) public {
        vm.warp(update.timestamp);
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
        client.updateClient(_encodeUpdate(fixture.nonAdjacentUpdate));
        vm.warp(membership.timestamp);

        if (membership.expectedRevert.length != 0) {
            vm.expectRevert(membership.expectedRevert);
        }
        uint256 timestamp = client.verifyMembership(membership.message);

        if (membership.expectedRevert.length == 0) {
            assertEq(timestamp, membership.expectedTimestamp);
        }
    }

    function test_misbehaviour_reverts() public {
        vm.expectRevert(abi.encodeWithSelector(IBesuLightClientErrors.UnsupportedMisbehaviour.selector));
        client.misbehaviour(bytes(""));
    }

    function test_updateClient_revertThroughWrongWrapper() public {
        vm.warp(fixture.initialTrustedTimestamp + 1);

        vm.expectRevert(
            abi.encodeWithSelector(IBesuLightClientErrors.InsufficientTrustedValidatorOverlap.selector, 0, 2)
        );
        wrongWrapper.updateClient(_encodeUpdate(fixture.nonAdjacentUpdate));
    }

    function fixtureMembership() public view returns (BesuMembershipTestCase[] memory testCases) {
        bytes[] memory path = new bytes[](2);
        path[0] = fixture.membership.path;
        path[1] = fixture.nonMembership.path;

        bytes memory wrongValue = abi.encodePacked(bytes32(uint256(1)));
        uint64 timestamp = fixture.initialTrustedTimestamp + 1;
        testCases = new BesuMembershipTestCase[](9);
        testCases[0] = BesuMembershipTestCase({
            name: "success",
            timestamp: timestamp,
            message: _membershipMessage(0, _singlePath(fixture.membership.path), fixture.membership.value),
            expectedRevert: "",
            expectedTimestamp: fixture.membership.expectedTimestamp
        });
        testCases[1] = BesuMembershipTestCase({
            name: "failure: wrong revision number",
            timestamp: timestamp,
            message: _membershipMessage(1, _singlePath(fixture.membership.path), fixture.membership.value),
            expectedRevert: abi.encodeWithSelector(IBesuLightClientErrors.InvalidRevisionNumber.selector, 1),
            expectedTimestamp: 0
        });
        testCases[2] = BesuMembershipTestCase({
            name: "failure: wrong path shape",
            timestamp: timestamp,
            message: _membershipMessage(0, path, fixture.membership.value),
            expectedRevert: abi.encodeWithSelector(IBesuLightClientErrors.InvalidPathLength.selector, 1, 2),
            expectedTimestamp: 0
        });
        testCases[3] = BesuMembershipTestCase({
            name: "failure: wrong commitment value",
            timestamp: timestamp,
            message: _membershipMessage(0, _singlePath(fixture.membership.path), wrongValue),
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
            _membershipMessage(0, _singlePath(fixture.membership.path), fixture.membership.value);
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
            _membershipMessage(0, _singlePath(fixture.membership.path), fixture.membership.value);
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
            message: _membershipMessage(0, _singlePath(fixture.membership.path), fixture.membership.value),
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
            _membershipMessage(0, _singlePath(fixture.membership.path), fixture.membership.value);
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
    }

    function fixtureNonMembership() public view returns (BesuNonMembershipTestCase[] memory testCases) {
        uint64 proofHeight = fixture.nonMembership.proofHeight;
        IBesuLightClientMsgs.ConsensusState memory expectedPreimage = _provenConsensusState(proofHeight);

        uint64 timestamp = fixture.initialTrustedTimestamp + 1;
        testCases = new BesuNonMembershipTestCase[](7);
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

        ILightClientMsgs.MsgVerifyNonMembership memory wrongPathMessage = _nonMembershipMessage(proofHeight);
        wrongPathMessage.path = new bytes[](2);
        wrongPathMessage.path[0] = fixture.nonMembership.path;
        wrongPathMessage.path[1] = fixture.membership.path;
        testCases[2] = BesuNonMembershipTestCase({
            name: "failure: wrong path shape",
            timestamp: timestamp,
            message: wrongPathMessage,
            expectedRevert: abi.encodeWithSelector(IBesuLightClientErrors.InvalidPathLength.selector, 1, 2),
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

        testCases = new BesuUpdateTestCase[](16);
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
            update: _zeroTimestampUpdate(),
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
            name: "failure: insufficient validator quorum",
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
        testCases[8] = BesuUpdateTestCase({
            name: "failure: conflicting same-height state",
            timestamp: fixture.initialTrustedTimestamp + 1,
            update: _encodeUpdate(fixture.conflictingUpdate),
            preUpdate: _encodeUpdate(fixture.nonAdjacentUpdate),
            expectedRevert: abi.encodeWithSelector(
                IBesuLightClientErrors.ConflictingConsensusState.selector, fixture.conflictingUpdate.height
            ),
            expectedState: emptyExpectedState
        });

        IBesuLightClientMsgs.MsgUpdateClient memory unknownTrustedHeightUpdate =
            abi.decode(_encodeUpdate(fixture.nonAdjacentUpdate), (IBesuLightClientMsgs.MsgUpdateClient));
        unknownTrustedHeightUpdate.trustedHeight.revisionHeight = fixture.initialTrustedHeight + 1000;

        testCases[9] = BesuUpdateTestCase({
            name: "failure: unknown trusted height",
            timestamp: fixture.initialTrustedTimestamp + 1,
            update: abi.encode(unknownTrustedHeightUpdate),
            preUpdate: "",
            expectedRevert: abi.encodeWithSelector(
                IBesuLightClientErrors.ConsensusStateNotFound.selector, fixture.initialTrustedHeight + 1000
            ),
            expectedState: emptyExpectedState
        });

        IBesuLightClientMsgs.MsgUpdateClient memory wrongPreimageUpdate =
            abi.decode(_encodeUpdate(fixture.nonAdjacentUpdate), (IBesuLightClientMsgs.MsgUpdateClient));
        wrongPreimageUpdate.consensusStatePreimage.validators[0] = address(0xdead);

        testCases[10] = BesuUpdateTestCase({
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

        testCases[11] = BesuUpdateTestCase({
            name: "failure: empty validators",
            timestamp: fixture.initialTrustedTimestamp + 1,
            update: _validatorsUpdate(new address[](0)),
            preUpdate: "",
            expectedRevert: abi.encodeWithSelector(IBesuLightClientErrors.EmptyValidatorSet.selector),
            expectedState: emptyExpectedState
        });

        address[] memory validators = fixture.nonAdjacentUpdate.expectedValidators;
        validators[0] = address(0);
        testCases[12] = BesuUpdateTestCase({
            name: "failure: zero validator",
            timestamp: fixture.initialTrustedTimestamp + 1,
            update: _validatorsUpdate(validators),
            preUpdate: "",
            expectedRevert: abi.encodeWithSelector(IBesuLightClientErrors.InvalidValidatorAddress.selector, address(0)),
            expectedState: emptyExpectedState
        });

        validators = fixture.nonAdjacentUpdate.expectedValidators;
        (validators[1], validators[2]) = (validators[2], validators[1]);
        testCases[13] = BesuUpdateTestCase({
            name: "failure: descending validators",
            timestamp: fixture.initialTrustedTimestamp + 1,
            update: _validatorsUpdate(validators),
            preUpdate: "",
            expectedRevert: abi.encodeWithSelector(IBesuLightClientErrors.UnsortedValidatorSet.selector, 1),
            expectedState: emptyExpectedState
        });

        validators = fixture.nonAdjacentUpdate.expectedValidators;
        validators[2] = validators[1];
        testCases[14] = BesuUpdateTestCase({
            name: "failure: duplicate validators",
            timestamp: fixture.initialTrustedTimestamp + 1,
            update: _validatorsUpdate(validators),
            preUpdate: "",
            expectedRevert: abi.encodeWithSelector(IBesuLightClientErrors.UnsortedValidatorSet.selector, 1),
            expectedState: emptyExpectedState
        });
        testCases[15] = BesuUpdateTestCase({
            name: "failure: unknown commit seal signer",
            timestamp: fixture.initialTrustedTimestamp + 1,
            update: _unknownSignerUpdate(),
            preUpdate: "",
            expectedRevert: abi.encodeWithSelector(
                IBesuLightClientErrors.UnknownCommitSealSigner.selector, vm.addr(UNKNOWN_SIGNER_KEY)
            ),
            expectedState: emptyExpectedState
        });
    }

    /// @dev Appends a commit seal from a non-validator key to `nonAdjacentUpdate`.
    /// The fixture's seals stay valid because the commit seal digest excludes the seal list.
    function _unknownSignerUpdate() internal view returns (bytes memory) {
        BesuUpdateFixture memory update = fixture.nonAdjacentUpdate;
        (uint8 v, bytes32 r, bytes32 s) =
            vm.sign(UNKNOWN_SIGNER_KEY, commitSealDigest.commitSealDigest(update.headerRlp));

        Memory.Slice[] memory headerItems = update.headerRlp.decodeList();
        Memory.Slice[] memory extraItems = RLP.readBytes(headerItems[12]).decodeList();
        Memory.Slice[] memory sealItems = RLP.readList(extraItems[4]);
        bytes[] memory encodedSeals = new bytes[](sealItems.length + 1);
        for (uint256 i = 0; i < sealItems.length; ++i) {
            encodedSeals[i] = sealItems[i].toBytes();
        }
        encodedSeals[sealItems.length] = RLP.encode(abi.encodePacked(r, s, v));
        bytes[] memory encodedExtraItems = new bytes[](extraItems.length);
        for (uint256 i = 0; i < extraItems.length; ++i) {
            encodedExtraItems[i] = i == 4 ? RLP.encode(encodedSeals) : extraItems[i].toBytes();
        }
        bytes[] memory encodedHeaderItems = new bytes[](headerItems.length);
        for (uint256 i = 0; i < headerItems.length; ++i) {
            encodedHeaderItems[i] = i == 12 ? RLP.encode(RLP.encode(encodedExtraItems)) : headerItems[i].toBytes();
        }
        update.headerRlp = RLP.encode(encodedHeaderItems);
        return _encodeUpdate(update);
    }

    function _validatorsUpdate(address[] memory validators) internal view returns (bytes memory) {
        BesuUpdateFixture memory update = fixture.nonAdjacentUpdate;
        Memory.Slice[] memory headerItems = update.headerRlp.decodeList();
        Memory.Slice[] memory extraItems = RLP.readBytes(headerItems[12]).decodeList();
        bytes[] memory encodedValidators = new bytes[](validators.length);
        for (uint256 i = 0; i < validators.length; ++i) {
            encodedValidators[i] = RLP.encode(abi.encodePacked(validators[i]));
        }
        bytes[] memory encodedExtraItems = new bytes[](extraItems.length);
        for (uint256 i = 0; i < extraItems.length; ++i) {
            encodedExtraItems[i] = i == 1 ? RLP.encode(encodedValidators) : extraItems[i].toBytes();
        }
        bytes[] memory encodedHeaderItems = new bytes[](headerItems.length);
        for (uint256 i = 0; i < headerItems.length; ++i) {
            encodedHeaderItems[i] = i == 12 ? RLP.encode(RLP.encode(encodedExtraItems)) : headerItems[i].toBytes();
        }
        update.headerRlp = RLP.encode(encodedHeaderItems);
        return _encodeUpdate(update);
    }

    function _zeroTimestampUpdate() internal view returns (bytes memory) {
        BesuUpdateFixture memory update = fixture.nonAdjacentUpdate;
        Memory.Slice[] memory headerItems = update.headerRlp.decodeList();
        bytes[] memory encodedHeaderItems = new bytes[](headerItems.length);
        for (uint256 i = 0; i < headerItems.length; ++i) {
            encodedHeaderItems[i] = i == 11 ? RLP.encode(uint256(0)) : headerItems[i].toBytes();
        }
        update.headerRlp = RLP.encode(encodedHeaderItems);
        return _encodeUpdate(update);
    }

    function _assertClientState(BesuUpdateFixture memory update) internal view {
        (address ibcRouter, IICS02ClientMsgs.Height memory latestHeight, uint64 trustingPeriod, uint64 maxClockDrift) =
            abi.decode(client.getClientState(), (address, IICS02ClientMsgs.Height, uint64, uint64));
        assertEq(ibcRouter, fixture.routerAddress);
        assertEq(latestHeight.revisionNumber, 0);
        assertEq(latestHeight.revisionHeight, update.height);
        assertEq(trustingPeriod, fixture.trustingPeriod);
        assertEq(maxClockDrift, fixture.maxClockDrift);

        assertEq(client.getConsensusStateHash(update.height), _consensusStateHash(_expectedConsensusState(update)));
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
        return _encodeUpdate(update.headerRlp, update.trustedHeight);
    }

    function _encodeUpdate(BesuRejectionUpdateFixture memory update) internal view returns (bytes memory) {
        return _encodeUpdate(update.headerRlp, update.trustedHeight);
    }

    function _encodeUpdate(bytes memory headerRlp, uint64 trustedHeight) internal view returns (bytes memory) {
        return abi.encode(
            IBesuLightClientMsgs.MsgUpdateClient({
                headerRlp: headerRlp,
                trustedHeight: IICS02ClientMsgs.Height({ revisionNumber: 0, revisionHeight: trustedHeight }),
                consensusStatePreimage: _trustedConsensusState(trustedHeight)
            })
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

    function _singlePath(bytes memory path) internal pure returns (bytes[] memory out) {
        out = new bytes[](1);
        out[0] = path;
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
            path: _singlePath(fixture.membership.path),
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
            path: _singlePath(fixture.nonMembership.path)
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

    function _deployIBFT2() internal returns (IBesuLightClient) {
        return new BesuIBFT2LightClient(
            fixture.routerAddress,
            fixture.initialTrustedHeight,
            fixture.initialTrustedTimestamp,
            fixture.initialTrustedStateRoot,
            fixture.initialTrustedValidators,
            fixture.trustingPeriod,
            fixture.maxClockDrift,
            address(0)
        );
    }

    function _deployQBFT() internal returns (IBesuLightClient) {
        return new BesuQBFTLightClient(
            fixture.routerAddress,
            fixture.initialTrustedHeight,
            fixture.initialTrustedTimestamp,
            fixture.initialTrustedStateRoot,
            fixture.initialTrustedValidators,
            fixture.trustingPeriod,
            fixture.maxClockDrift,
            address(0)
        );
    }

    function _fixtureFile() internal pure virtual returns (string memory);
    function _deployPrimaryClient() internal virtual returns (IBesuLightClient);
    function _deployWrongWrapper() internal virtual returns (IBesuLightClient);
    function _deployCommitSealDigest() internal virtual returns (IBesuCommitSealDigest);
}
