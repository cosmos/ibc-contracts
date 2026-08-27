// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable gas-struct-packing

import {Test} from "forge-std/Test.sol";
import {stdJson} from "forge-std/StdJson.sol";

import {ILightClientMsgs} from "../../contracts/msgs/ILightClientMsgs.sol";
import {IICS02ClientMsgs} from "../../contracts/msgs/IICS02ClientMsgs.sol";
import {BesuIBFT2LightClient} from "../../contracts/light-clients/besu/BesuIBFT2LightClient.sol";
import {BesuQBFTLightClient} from "../../contracts/light-clients/besu/BesuQBFTLightClient.sol";
import {IBesuLightClient} from "../../contracts/light-clients/besu/interfaces/IBesuLightClient.sol";
import {IBesuLightClientMsgs} from "../../contracts/light-clients/besu/msgs/IBesuLightClientMsgs.sol";
import {IBesuLightClientErrors} from "../../contracts/light-clients/besu/errors/IBesuLightClientErrors.sol";
import {RLPReader} from "../../contracts/light-clients/besu/RLPReader.sol";

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

/// @dev Membership or non-membership proof and expected timestamp.
struct BesuProofFixture {
    bytes proof;
    uint64 proofHeight;
    bytes path;
    bytes value;
    uint64 expectedTimestamp;
}

/// @dev Update expected to be rejected, including any scenario-specific setup.
struct BesuUpdateRejectionTestCase {
    string name;
    uint64 timestamp;
    bytes update;
    bytes preUpdate;
    bytes expectedRevert;
}

/// @dev Membership verification expected to be rejected.
struct BesuMembershipRejectionTestCase {
    string name;
    ILightClientMsgs.MsgVerifyMembership message;
    bytes expectedRevert;
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
    using RLPReader for bytes;
    using RLPReader for RLPReader.RLPItem;

    string internal constant FIXTURE_DIR = "/test/besu-bft/fixtures/";

    BesuFixture internal fixture;
    IBesuLightClient internal client;
    IBesuLightClient internal wrongWrapper;

    function setUp() public virtual {
        fixture = _loadFixture(_fixtureFile());
        client = _deployPrimaryClient();
        wrongWrapper = _deployWrongWrapper();
    }

    function fixtureUpdate() public view returns (BesuUpdateFixture[] memory updates) {
        updates = new BesuUpdateFixture[](2);
        updates[0] = fixture.adjacentUpdate;
        updates[1] = fixture.nonAdjacentUpdate;
    }

    function tableUpdateClientValidTest(BesuUpdateFixture memory update) public {
        vm.warp(fixture.initialTrustedTimestamp + 1);

        ILightClientMsgs.UpdateResult result = client.updateClient(_encodeUpdate(update));

        assertEq(uint8(result), uint8(ILightClientMsgs.UpdateResult.Update));
        _assertClientState(update);
    }

    function test_verifyMembership() public {
        vm.warp(fixture.initialTrustedTimestamp + 1);
        client.updateClient(_encodeUpdate(fixture.nonAdjacentUpdate));

        uint256 timestamp = client.verifyMembership(
            _membershipMessage(0, _singlePath(fixture.membership.path), fixture.membership.value)
        );

        assertEq(timestamp, fixture.membership.expectedTimestamp);
    }

    function test_verifyNonMembership() public {
        vm.warp(fixture.initialTrustedTimestamp + 1);
        client.updateClient(_encodeUpdate(fixture.nonAdjacentUpdate));

        uint256 timestamp = client.verifyNonMembership(
            ILightClientMsgs.MsgVerifyNonMembership({
                proof: fixture.nonMembership.proof,
                proofHeight: IICS02ClientMsgs.Height({
                    revisionNumber: 0, revisionHeight: fixture.nonMembership.proofHeight
                }),
                path: _singlePath(fixture.nonMembership.path)
            })
        );

        assertEq(timestamp, fixture.nonMembership.expectedTimestamp);
    }

    function tableUpdateClientRejectionTest(BesuUpdateRejectionTestCase memory updateRejection) public {
        vm.warp(updateRejection.timestamp);
        if (updateRejection.preUpdate.length != 0) {
            client.updateClient(updateRejection.preUpdate);
        }

        bytes memory clientStateBefore = client.getClientState();
        bytes memory consensusStateBefore = client.getConsensusState(fixture.initialTrustedHeight);

        vm.expectRevert(updateRejection.expectedRevert);
        client.updateClient(updateRejection.update);

        assertEq(client.getClientState(), clientStateBefore);
        assertEq(client.getConsensusState(fixture.initialTrustedHeight), consensusStateBefore);
    }

    function tableVerifyMembershipRejectionTest(BesuMembershipRejectionTestCase memory membershipRejection) public {
        vm.warp(fixture.initialTrustedTimestamp + 1);
        client.updateClient(_encodeUpdate(fixture.nonAdjacentUpdate));

        vm.expectRevert(membershipRejection.expectedRevert);
        client.verifyMembership(membershipRejection.message);
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

    function fixtureMembershipRejection() public view returns (BesuMembershipRejectionTestCase[] memory testCases) {
        bytes[] memory path = new bytes[](2);
        path[0] = fixture.membership.path;
        path[1] = fixture.nonMembership.path;

        bytes memory wrongValue = abi.encodePacked(bytes32(uint256(1)));
        testCases = new BesuMembershipRejectionTestCase[](3);
        testCases[0] = BesuMembershipRejectionTestCase({
            name: "wrong revision number",
            message: _membershipMessage(1, _singlePath(fixture.membership.path), fixture.membership.value),
            expectedRevert: abi.encodeWithSelector(IBesuLightClientErrors.InvalidRevisionNumber.selector, 1)
        });
        testCases[1] = BesuMembershipRejectionTestCase({
            name: "wrong path shape",
            message: _membershipMessage(0, path, fixture.membership.value),
            expectedRevert: abi.encodeWithSelector(IBesuLightClientErrors.InvalidPathLength.selector, 1, 2)
        });
        testCases[2] = BesuMembershipRejectionTestCase({
            name: "wrong commitment value",
            message: _membershipMessage(0, _singlePath(fixture.membership.path), wrongValue),
            expectedRevert: abi.encodeWithSelector(
                IBesuLightClientErrors.InvalidCommitmentValue.selector,
                bytes32(uint256(1)),
                abi.decode(fixture.membership.value, (bytes32))
            )
        });
    }

    function fixtureUpdateRejection() public view returns (BesuUpdateRejectionTestCase[] memory testCases) {
        testCases = new BesuUpdateRejectionTestCase[](6);

        BesuUpdateFixture memory update = fixture.nonAdjacentUpdate;
        RLPReader.RLPItem memory headerItem = update.headerRlp.toRlpItem();
        RLPReader.RLPItem[] memory headerItems = headerItem.toList();
        (uint256 timestampPtr, uint256 timestampLen) = headerItems[11].payloadLocation();
        for (uint256 i = 0; i < timestampLen; ++i) {
            update.headerRlp[timestampPtr - headerItem.memPtr + i] = 0;
        }

        testCases[0] = BesuUpdateRejectionTestCase({
            name: "zero timestamp",
            timestamp: fixture.initialTrustedTimestamp + 1,
            update: _encodeUpdate(update),
            preUpdate: "",
            expectedRevert: abi.encodeWithSelector(IBesuLightClientErrors.InvalidHeaderTimestamp.selector)
        });
        testCases[1] = BesuUpdateRejectionTestCase({
            name: "expired trusted state",
            timestamp: fixture.initialTrustedTimestamp + fixture.trustingPeriod + 1,
            update: _encodeUpdate(fixture.nonAdjacentUpdate),
            preUpdate: "",
            expectedRevert: abi.encodeWithSelector(
                IBesuLightClientErrors.ConsensusStateExpired.selector,
                fixture.initialTrustedTimestamp,
                fixture.initialTrustedTimestamp + fixture.trustingPeriod + 1,
                fixture.trustingPeriod
            )
        });
        testCases[2] = BesuUpdateRejectionTestCase({
            name: "insufficient trusted overlap",
            timestamp: fixture.initialTrustedTimestamp + 1,
            update: _encodeUpdate(fixture.lowOverlapUpdate),
            preUpdate: "",
            expectedRevert: abi.encodeWithSelector(
                IBesuLightClientErrors.InsufficientTrustedValidatorOverlap.selector, 1, 2
            )
        });
        testCases[3] = BesuUpdateRejectionTestCase({
            name: "insufficient validator quorum",
            timestamp: fixture.initialTrustedTimestamp + 1,
            update: _encodeUpdate(fixture.lowQuorumUpdate),
            preUpdate: "",
            expectedRevert: abi.encodeWithSelector(IBesuLightClientErrors.InsufficientValidatorQuorum.selector, 2, 3)
        });

        IBesuLightClientMsgs.MsgUpdateClient memory wrongRevisionUpdate =
            abi.decode(_encodeUpdate(fixture.nonAdjacentUpdate), (IBesuLightClientMsgs.MsgUpdateClient));
        wrongRevisionUpdate.trustedHeight.revisionNumber = 1;
        testCases[4] = BesuUpdateRejectionTestCase({
            name: "wrong revision number",
            timestamp: fixture.initialTrustedTimestamp + 1,
            update: abi.encode(wrongRevisionUpdate),
            preUpdate: "",
            expectedRevert: abi.encodeWithSelector(IBesuLightClientErrors.InvalidRevisionNumber.selector, 1)
        });
        testCases[5] = BesuUpdateRejectionTestCase({
            name: "conflicting same-height state",
            timestamp: fixture.initialTrustedTimestamp + 1,
            update: _encodeUpdate(fixture.conflictingUpdate),
            preUpdate: _encodeUpdate(fixture.nonAdjacentUpdate),
            expectedRevert: abi.encodeWithSelector(
                IBesuLightClientErrors.ConflictingConsensusState.selector, fixture.conflictingUpdate.height
            )
        });
    }

    function _assertClientState(BesuUpdateFixture memory update) internal view {
        (address ibcRouter, IICS02ClientMsgs.Height memory latestHeight, uint64 trustingPeriod, uint64 maxClockDrift) =
            abi.decode(client.getClientState(), (address, IICS02ClientMsgs.Height, uint64, uint64));
        assertEq(ibcRouter, fixture.routerAddress);
        assertEq(latestHeight.revisionNumber, 0);
        assertEq(latestHeight.revisionHeight, update.height);
        assertEq(trustingPeriod, fixture.trustingPeriod);
        assertEq(maxClockDrift, fixture.maxClockDrift);

        (uint64 timestamp, bytes32 storageRoot, address[] memory validators) =
            abi.decode(client.getConsensusState(update.height), (uint64, bytes32, address[]));
        assertEq(timestamp, update.expectedTimestamp);
        assertEq(storageRoot, update.expectedStorageRoot);

        assertEq(validators.length, update.expectedValidators.length);
        for (uint256 i = 0; i < update.expectedValidators.length; ++i) {
            assertEq(validators[i], update.expectedValidators[i]);
        }
    }

    function _encodeUpdate(BesuUpdateFixture memory update) internal pure returns (bytes memory) {
        return abi.encode(
            IBesuLightClientMsgs.MsgUpdateClient({
                headerRlp: update.headerRlp,
                trustedHeight: IICS02ClientMsgs.Height({revisionNumber: 0, revisionHeight: update.trustedHeight}),
                accountProof: update.accountProof
            })
        );
    }

    function _singlePath(bytes memory path) internal pure returns (bytes[] memory out) {
        out = new bytes[](1);
        out[0] = path;
    }

    function _membershipMessage(uint64 revisionNumber, bytes[] memory path, bytes memory value)
        internal
        view
        returns (ILightClientMsgs.MsgVerifyMembership memory)
    {
        return ILightClientMsgs.MsgVerifyMembership({
            proof: fixture.membership.proof,
            proofHeight: IICS02ClientMsgs.Height({
                revisionNumber: revisionNumber, revisionHeight: fixture.membership.proofHeight
            }),
            path: path,
            value: value
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

    function _readUpdate(string memory json, string memory path) internal view returns (BesuUpdateFixture memory) {
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

    function _readRejectionUpdate(string memory json, string memory path)
        internal
        view
        returns (BesuRejectionUpdateFixture memory)
    {
        return BesuRejectionUpdateFixture({
            height: uint64(json.readUint(string.concat(path, ".height"))),
            headerRlp: json.readBytes(string.concat(path, ".headerRlp")),
            trustedHeight: uint64(json.readUint(string.concat(path, ".trustedHeight"))),
            accountProof: json.readBytes(string.concat(path, ".accountProof"))
        });
    }

    function _encodeUpdate(BesuRejectionUpdateFixture memory update) internal pure returns (bytes memory) {
        return abi.encode(
            IBesuLightClientMsgs.MsgUpdateClient({
                headerRlp: update.headerRlp,
                trustedHeight: IICS02ClientMsgs.Height({revisionNumber: 0, revisionHeight: update.trustedHeight}),
                accountProof: update.accountProof
            })
        );
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

    function _deployIBFT2() internal returns (IBesuLightClient) {
        return new BesuIBFT2LightClient(
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

    function _deployQBFT() internal returns (IBesuLightClient) {
        return new BesuQBFTLightClient(
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
    function _deployPrimaryClient() internal virtual returns (IBesuLightClient);
    function _deployWrongWrapper() internal virtual returns (IBesuLightClient);
}
