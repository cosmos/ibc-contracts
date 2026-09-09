// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable gas-struct-packing,function-max-lines,gas-small-strings

import { Test } from "forge-std/Test.sol";
import { stdJson } from "forge-std/StdJson.sol";
import { RLP } from "@openzeppelin-contracts/utils/RLP.sol";
import { Memory } from "@openzeppelin-contracts/utils/Memory.sol";

import { ILightClient } from "../../contracts/interfaces/ILightClient.sol";
import { ILightClientMsgs } from "../../contracts/msgs/ILightClientMsgs.sol";
import { IICS02ClientMsgs } from "../../contracts/msgs/IICS02ClientMsgs.sol";
import { BesuIBFT2LightClient } from "../../contracts/light-clients/besu/BesuIBFT2LightClient.sol";
import { BesuQBFTLightClient } from "../../contracts/light-clients/besu/BesuQBFTLightClient.sol";
import { IBesuLightClientMsgs } from "../../contracts/light-clients/besu/msgs/IBesuLightClientMsgs.sol";
import { IBesuLightClientErrors } from "../../contracts/light-clients/besu/errors/IBesuLightClientErrors.sol";

/// @dev Light client surface used by the tests, extended with a consensus state hash getter.
interface IBesuLightClientHarness is ILightClient {
    /// @dev Returns the stored consensus state hash for a revision height, or zero if none is stored.
    function consensusStateHash(uint64 revisionHeight) external view returns (bytes32);
}

/// @dev IBFT2 client exposing stored consensus state hashes for assertions.
contract BesuIBFT2LightClientHarness is BesuIBFT2LightClient, IBesuLightClientHarness {
    constructor(
        address ibcRouter,
        uint64 initialTrustedHeight,
        uint64 initialTrustedTimestamp,
        bytes32 initialTrustedStorageRoot,
        address[] memory initialTrustedValidators,
        uint64 trustingPeriod,
        uint64 maxClockDrift,
        address roleManager
    )
        BesuIBFT2LightClient(
            ibcRouter,
            initialTrustedHeight,
            initialTrustedTimestamp,
            initialTrustedStorageRoot,
            initialTrustedValidators,
            trustingPeriod,
            maxClockDrift,
            roleManager
        )
    { }

    function consensusStateHash(uint64 revisionHeight) external view returns (bytes32) {
        return consensusStateHashes[revisionHeight];
    }
}

/// @dev QBFT client exposing stored consensus state hashes for assertions.
contract BesuQBFTLightClientHarness is BesuQBFTLightClient, IBesuLightClientHarness {
    constructor(
        address ibcRouter,
        uint64 initialTrustedHeight,
        uint64 initialTrustedTimestamp,
        bytes32 initialTrustedStorageRoot,
        address[] memory initialTrustedValidators,
        uint64 trustingPeriod,
        uint64 maxClockDrift,
        address roleManager
    )
        BesuQBFTLightClient(
            ibcRouter,
            initialTrustedHeight,
            initialTrustedTimestamp,
            initialTrustedStorageRoot,
            initialTrustedValidators,
            trustingPeriod,
            maxClockDrift,
            roleManager
        )
    { }

    function consensusStateHash(uint64 revisionHeight) external view returns (bytes32) {
        return consensusStateHashes[revisionHeight];
    }
}

/// @dev Successful update input and expected consensus state.
struct BesuUpdateFixture {
    uint64 height;
    bytes headerRlp;
    uint64 trustedHeight;
    bytes accountProof;
    uint64 expectedTimestamp;
    bytes32 expectedStorageRoot;
    address[] expectedValidators;
}

/// @dev Update input expected to be rejected.
struct BesuRejectionUpdateFixture {
    uint64 height;
    bytes headerRlp;
    uint64 trustedHeight;
    bytes accountProof;
}

/// @dev Membership or non-membership storage proof and expected timestamp.
/// @dev `proof` holds the raw storage proof nodes as `abi.encode(bytes[])`; the tests wrap them into
/// `IBesuLightClientMsgs.MembershipProof` together with the consensus state preimage for `proofHeight`.
struct BesuProofFixture {
    bytes proof;
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
struct BesuMembershipTestCase {
    string name;
    ILightClientMsgs.MsgVerifyMembership message;
    bytes expectedRevert;
    uint64 expectedTimestamp;
}

/// @dev Complete fixture shared by the Besu BFT light-client tests.
struct BesuFixture {
    address routerAddress;
    uint64 initialTrustedHeight;
    uint64 initialTrustedTimestamp;
    bytes32 initialTrustedStorageRoot;
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

    BesuFixture internal fixture;
    IBesuLightClientHarness internal client;
    IBesuLightClientHarness internal wrongWrapper;

    function setUp() public virtual {
        fixture = _loadFixture(_fixtureFile());
        client = _deployPrimaryClient();
        wrongWrapper = _deployWrongWrapper();
    }

    function test_constructor_storesInitialConsensusStateHash() public view {
        assertEq(client.consensusStateHash(fixture.initialTrustedHeight), _consensusStateHash(_initialConsensusState()));
    }

    function test_verifyNonMembership() public {
        vm.warp(fixture.initialTrustedTimestamp + 1);
        client.updateClient(_encodeUpdate(fixture.nonAdjacentUpdate));

        uint256 timestamp = client.verifyNonMembership(_nonMembershipMessage(fixture.nonMembership.proofHeight));

        assertEq(timestamp, fixture.nonMembership.expectedTimestamp);
    }

    function test_verifyNonMembership_revertWrongPreimage() public {
        vm.warp(fixture.initialTrustedTimestamp + 1);
        client.updateClient(_encodeUpdate(fixture.nonAdjacentUpdate));

        IBesuLightClientMsgs.ConsensusState memory tampered = _provenConsensusState(fixture.nonMembership.proofHeight);
        tampered.storageRoot = bytes32(uint256(tampered.storageRoot) ^ 1);
        ILightClientMsgs.MsgVerifyNonMembership memory message =
            _nonMembershipMessage(fixture.nonMembership.proofHeight);
        message.proof = _encodeProof(fixture.nonMembership, tampered);

        vm.expectRevert(
            abi.encodeWithSelector(
                IBesuLightClientErrors.ConsensusStatePreimageMismatch.selector,
                _consensusStateHash(_provenConsensusState(fixture.nonMembership.proofHeight)),
                _consensusStateHash(tampered)
            )
        );
        client.verifyNonMembership(message);
    }

    function test_verifyNonMembership_revertUnknownHeight() public {
        vm.warp(fixture.initialTrustedTimestamp + 1);
        client.updateClient(_encodeUpdate(fixture.nonAdjacentUpdate));

        uint64 unknownHeight = fixture.nonMembership.proofHeight + 1000;
        ILightClientMsgs.MsgVerifyNonMembership memory message =
            _nonMembershipMessage(fixture.nonMembership.proofHeight);
        message.proofHeight.revisionHeight = unknownHeight;

        vm.expectRevert(abi.encodeWithSelector(IBesuLightClientErrors.ConsensusStateNotFound.selector, unknownHeight));
        client.verifyNonMembership(message);
    }

    function test_updateClient_noOpOnSameState() public {
        vm.warp(fixture.initialTrustedTimestamp + 1);
        client.updateClient(_encodeUpdate(fixture.nonAdjacentUpdate));
        bytes32 storedHash = client.consensusStateHash(fixture.nonAdjacentUpdate.height);

        ILightClientMsgs.UpdateResult result = client.updateClient(_encodeUpdate(fixture.nonAdjacentUpdate));

        assertEq(uint8(result), uint8(ILightClientMsgs.UpdateResult.NoOp));
        assertEq(client.consensusStateHash(fixture.nonAdjacentUpdate.height), storedHash);
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
        bytes32 consensusStateHashBefore = client.consensusStateHash(fixture.initialTrustedHeight);

        vm.expectRevert(update.expectedRevert);
        client.updateClient(update.update);

        assertEq(client.getClientState(), clientStateBefore);
        assertEq(client.consensusStateHash(fixture.initialTrustedHeight), consensusStateHashBefore);
    }

    function tableVerifyMembershipTest(BesuMembershipTestCase memory membership) public {
        vm.warp(fixture.initialTrustedTimestamp + 1);
        client.updateClient(_encodeUpdate(fixture.nonAdjacentUpdate));

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
        testCases = new BesuMembershipTestCase[](6);
        testCases[0] = BesuMembershipTestCase({
            name: "success",
            message: _membershipMessage(0, _singlePath(fixture.membership.path), fixture.membership.value),
            expectedRevert: "",
            expectedTimestamp: fixture.membership.expectedTimestamp
        });
        testCases[1] = BesuMembershipTestCase({
            name: "failure: wrong revision number",
            message: _membershipMessage(1, _singlePath(fixture.membership.path), fixture.membership.value),
            expectedRevert: abi.encodeWithSelector(IBesuLightClientErrors.InvalidRevisionNumber.selector, 1),
            expectedTimestamp: 0
        });
        testCases[2] = BesuMembershipTestCase({
            name: "failure: wrong path shape",
            message: _membershipMessage(0, path, fixture.membership.value),
            expectedRevert: abi.encodeWithSelector(IBesuLightClientErrors.InvalidPathLength.selector, 1, 2),
            expectedTimestamp: 0
        });
        testCases[3] = BesuMembershipTestCase({
            name: "failure: wrong commitment value",
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
            message: unknownHeightMessage,
            expectedRevert: abi.encodeWithSelector(
                IBesuLightClientErrors.ConsensusStateNotFound.selector, unknownHeight
            ),
            expectedTimestamp: 0
        });
    }

    function fixtureUpdate() public view returns (BesuUpdateTestCase[] memory testCases) {
        BesuUpdateFixture memory emptyExpectedState;

        testCases = new BesuUpdateTestCase[](10);
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
            name: "failure: insufficient trusted overlap",
            timestamp: fixture.initialTrustedTimestamp + 1,
            update: _encodeUpdate(fixture.lowOverlapUpdate),
            preUpdate: "",
            expectedRevert: abi.encodeWithSelector(
                IBesuLightClientErrors.InsufficientTrustedValidatorOverlap.selector, 1, 2
            ),
            expectedState: emptyExpectedState
        });
        testCases[5] = BesuUpdateTestCase({
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

        testCases[6] = BesuUpdateTestCase({
            name: "failure: wrong revision number",
            timestamp: fixture.initialTrustedTimestamp + 1,
            update: abi.encode(wrongRevisionUpdate),
            preUpdate: "",
            expectedRevert: abi.encodeWithSelector(IBesuLightClientErrors.InvalidRevisionNumber.selector, 1),
            expectedState: emptyExpectedState
        });
        testCases[7] = BesuUpdateTestCase({
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

        testCases[8] = BesuUpdateTestCase({
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

        assertEq(client.consensusStateHash(update.height), _consensusStateHash(_expectedConsensusState(update)));
    }

    /// @dev Consensus state committed by the constructor at `fixture.initialTrustedHeight`.
    function _initialConsensusState() internal view returns (IBesuLightClientMsgs.ConsensusState memory) {
        return IBesuLightClientMsgs.ConsensusState({
            timestamp: fixture.initialTrustedTimestamp,
            storageRoot: fixture.initialTrustedStorageRoot,
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
            storageRoot: update.expectedStorageRoot,
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
        return _encodeUpdate(update.headerRlp, update.trustedHeight, update.accountProof);
    }

    function _encodeUpdate(BesuRejectionUpdateFixture memory update) internal view returns (bytes memory) {
        return _encodeUpdate(update.headerRlp, update.trustedHeight, update.accountProof);
    }

    function _encodeUpdate(
        bytes memory headerRlp,
        uint64 trustedHeight,
        bytes memory accountProof
    )
        internal
        view
        returns (bytes memory)
    {
        return abi.encode(
            IBesuLightClientMsgs.MsgUpdateClient({
                headerRlp: headerRlp,
                trustedHeight: IICS02ClientMsgs.Height({ revisionNumber: 0, revisionHeight: trustedHeight }),
                consensusStatePreimage: _trustedConsensusState(trustedHeight),
                accountProof: accountProof
            })
        );
    }

    /// @dev Wraps fixture storage proof nodes and a consensus state preimage into the client proof format.
    function _encodeProof(
        BesuProofFixture memory proofFixture,
        IBesuLightClientMsgs.ConsensusState memory preimage
    )
        internal
        pure
        returns (bytes memory)
    {
        return abi.encode(
            IBesuLightClientMsgs.MembershipProof({
                consensusStatePreimage: preimage, proofNodes: abi.decode(proofFixture.proof, (bytes[]))
            })
        );
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

    function _nonMembershipMessage(uint64 proofHeight)
        internal
        view
        returns (ILightClientMsgs.MsgVerifyNonMembership memory)
    {
        return ILightClientMsgs.MsgVerifyNonMembership({
            proof: _encodeProof(fixture.nonMembership, _provenConsensusState(proofHeight)),
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
            initialTrustedStorageRoot: json.readBytes32(".initialTrustedStorageRoot"),
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
            accountProof: json.readBytes(string.concat(path, ".accountProof")),
            expectedTimestamp: uint64(json.readUint(string.concat(path, ".expectedTimestamp"))),
            expectedStorageRoot: json.readBytes32(string.concat(path, ".expectedStorageRoot")),
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
            trustedHeight: uint64(json.readUint(string.concat(path, ".trustedHeight"))),
            accountProof: json.readBytes(string.concat(path, ".accountProof"))
        });
    }

    function _readProof(string memory json, string memory path) internal view returns (BesuProofFixture memory) {
        return BesuProofFixture({
            proof: json.readBytes(string.concat(path, ".proof")),
            proofHeight: uint64(json.readUint(string.concat(path, ".proofHeight"))),
            path: json.readBytes(string.concat(path, ".path")),
            value: json.keyExists(string.concat(path, ".value"))
                ? json.readBytes(string.concat(path, ".value"))
                : bytes(""),
            expectedTimestamp: uint64(json.readUint(string.concat(path, ".expectedTimestamp")))
        });
    }

    function _deployIBFT2() internal returns (IBesuLightClientHarness) {
        return new BesuIBFT2LightClientHarness(
            fixture.routerAddress,
            fixture.initialTrustedHeight,
            fixture.initialTrustedTimestamp,
            fixture.initialTrustedStorageRoot,
            fixture.initialTrustedValidators,
            fixture.trustingPeriod,
            fixture.maxClockDrift,
            address(0)
        );
    }

    function _deployQBFT() internal returns (IBesuLightClientHarness) {
        return new BesuQBFTLightClientHarness(
            fixture.routerAddress,
            fixture.initialTrustedHeight,
            fixture.initialTrustedTimestamp,
            fixture.initialTrustedStorageRoot,
            fixture.initialTrustedValidators,
            fixture.trustingPeriod,
            fixture.maxClockDrift,
            address(0)
        );
    }

    function _fixtureFile() internal pure virtual returns (string memory);
    function _deployPrimaryClient() internal virtual returns (IBesuLightClientHarness);
    function _deployWrongWrapper() internal virtual returns (IBesuLightClientHarness);
}
