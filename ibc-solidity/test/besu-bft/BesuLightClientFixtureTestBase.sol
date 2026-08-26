// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable gas-struct-packing

import { Test } from "forge-std/Test.sol";
import { stdJson } from "forge-std/StdJson.sol";

import { ILightClientMsgs } from "../../contracts/msgs/ILightClientMsgs.sol";
import { IICS02ClientMsgs } from "../../contracts/msgs/IICS02ClientMsgs.sol";
import { BesuIBFT2LightClient } from "../../contracts/light-clients/besu/BesuIBFT2LightClient.sol";
import { BesuQBFTLightClient } from "../../contracts/light-clients/besu/BesuQBFTLightClient.sol";
import { IBesuLightClient } from "../../contracts/light-clients/besu/interfaces/IBesuLightClient.sol";
import { IBesuLightClientMsgs } from "../../contracts/light-clients/besu/msgs/IBesuLightClientMsgs.sol";
import { IBesuLightClientErrors } from "../../contracts/light-clients/besu/errors/IBesuLightClientErrors.sol";
import { RLPReader } from "../../contracts/light-clients/besu/RLPReader.sol";

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

    function tableUpdateClientValid(BesuUpdateFixture memory update) public {
        vm.warp(fixture.initialTrustedTimestamp + 1);

        ILightClientMsgs.UpdateResult result = client.updateClient(_encodeUpdate(update));

        assertEq(uint8(result), uint8(ILightClientMsgs.UpdateResult.Update));
        _assertClientState(update);
    }

    function test_updateClient_revertZeroTimestampWithoutStateChange() public {
        BesuUpdateFixture memory update = fixture.nonAdjacentUpdate;
        RLPReader.RLPItem memory headerItem = update.headerRlp.toRlpItem();
        RLPReader.RLPItem[] memory headerItems = headerItem.toList();
        (uint256 timestampPtr, uint256 timestampLen) = headerItems[11].payloadLocation();
        for (uint256 i = 0; i < timestampLen; ++i) {
            update.headerRlp[timestampPtr - headerItem.memPtr + i] = 0;
        }

        bytes memory clientStateBefore = client.getClientState();
        bytes memory consensusStateBefore = client.getConsensusState(fixture.initialTrustedHeight);

        vm.expectRevert(IBesuLightClientErrors.InvalidHeaderTimestamp.selector);
        client.updateClient(_encodeUpdate(update));

        assertEq(client.getClientState(), clientStateBefore);
        assertEq(client.getConsensusState(fixture.initialTrustedHeight), consensusStateBefore);
    }

    function test_verifyMembership_returnsStoredTimestamp() public {
        vm.warp(fixture.initialTrustedTimestamp + 1);
        client.updateClient(_encodeUpdate(fixture.nonAdjacentUpdate));

        uint256 timestamp = client.verifyMembership(
            ILightClientMsgs.MsgVerifyMembership({
                proof: fixture.membership.proof,
                proofHeight: IICS02ClientMsgs.Height({
                    revisionNumber: 0, revisionHeight: fixture.membership.proofHeight
                }),
                path: _singlePath(fixture.membership.path),
                value: fixture.membership.value
            })
        );

        assertEq(timestamp, fixture.membership.expectedTimestamp);
    }

    function test_verifyNonMembership_returnsStoredTimestamp() public {
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

    function test_updateClient_revertExpiredTrustedState() public {
        vm.warp(fixture.initialTrustedTimestamp + fixture.trustingPeriod + 1);

        vm.expectRevert(
            abi.encodeWithSelector(
                IBesuLightClientErrors.ConsensusStateExpired.selector,
                fixture.initialTrustedTimestamp,
                fixture.initialTrustedTimestamp + fixture.trustingPeriod + 1,
                fixture.trustingPeriod
            )
        );
        client.updateClient(_encodeUpdate(fixture.nonAdjacentUpdate));
    }

    function test_updateClient_revertInsufficientTrustedOverlap() public {
        vm.warp(fixture.initialTrustedTimestamp + 1);

        vm.expectRevert(
            abi.encodeWithSelector(IBesuLightClientErrors.InsufficientTrustedValidatorOverlap.selector, 1, 2)
        );
        client.updateClient(_encodeUpdate(fixture.lowOverlapUpdate));
    }

    function test_updateClient_revertInsufficientNewValidatorQuorum() public {
        vm.warp(fixture.initialTrustedTimestamp + 1);

        vm.expectRevert(abi.encodeWithSelector(IBesuLightClientErrors.InsufficientValidatorQuorum.selector, 2, 3));
        client.updateClient(_encodeUpdate(fixture.lowQuorumUpdate));
    }

    function test_updateClient_revertWrongRevisionNumber() public {
        vm.warp(fixture.initialTrustedTimestamp + 1);

        IBesuLightClientMsgs.MsgUpdateClient memory update =
            abi.decode(_encodeUpdate(fixture.nonAdjacentUpdate), (IBesuLightClientMsgs.MsgUpdateClient));
        update.trustedHeight.revisionNumber = 1;

        vm.expectRevert(abi.encodeWithSelector(IBesuLightClientErrors.InvalidRevisionNumber.selector, 1));
        client.updateClient(abi.encode(update));
    }

    function test_verifyMembership_revertWrongRevisionNumber() public {
        vm.warp(fixture.initialTrustedTimestamp + 1);
        client.updateClient(_encodeUpdate(fixture.nonAdjacentUpdate));

        vm.expectRevert(abi.encodeWithSelector(IBesuLightClientErrors.InvalidRevisionNumber.selector, 1));
        client.verifyMembership(
            ILightClientMsgs.MsgVerifyMembership({
                proof: fixture.membership.proof,
                proofHeight: IICS02ClientMsgs.Height({
                    revisionNumber: 1, revisionHeight: fixture.membership.proofHeight
                }),
                path: _singlePath(fixture.membership.path),
                value: fixture.membership.value
            })
        );
    }

    function test_verifyMembership_revertWrongPathShape() public {
        vm.warp(fixture.initialTrustedTimestamp + 1);
        client.updateClient(_encodeUpdate(fixture.nonAdjacentUpdate));

        bytes[] memory path = new bytes[](2);
        path[0] = fixture.membership.path;
        path[1] = fixture.nonMembership.path;

        vm.expectRevert(abi.encodeWithSelector(IBesuLightClientErrors.InvalidPathLength.selector, 1, 2));
        client.verifyMembership(
            ILightClientMsgs.MsgVerifyMembership({
                proof: fixture.membership.proof,
                proofHeight: IICS02ClientMsgs.Height({
                    revisionNumber: 0, revisionHeight: fixture.membership.proofHeight
                }),
                path: path,
                value: fixture.membership.value
            })
        );
    }

    function test_verifyMembership_revertWrongCommitmentValue() public {
        vm.warp(fixture.initialTrustedTimestamp + 1);
        client.updateClient(_encodeUpdate(fixture.nonAdjacentUpdate));

        bytes memory wrongValue = abi.encodePacked(bytes32(uint256(1)));
        vm.expectRevert(
            abi.encodeWithSelector(
                IBesuLightClientErrors.InvalidCommitmentValue.selector,
                bytes32(uint256(1)),
                abi.decode(fixture.membership.value, (bytes32))
            )
        );
        client.verifyMembership(
            ILightClientMsgs.MsgVerifyMembership({
                proof: fixture.membership.proof,
                proofHeight: IICS02ClientMsgs.Height({
                    revisionNumber: 0, revisionHeight: fixture.membership.proofHeight
                }),
                path: _singlePath(fixture.membership.path),
                value: wrongValue
            })
        );
    }

    function test_updateClient_revertConflictingSameHeight() public {
        vm.warp(fixture.initialTrustedTimestamp + 1);
        client.updateClient(_encodeUpdate(fixture.nonAdjacentUpdate));

        vm.expectRevert(
            abi.encodeWithSelector(
                IBesuLightClientErrors.ConflictingConsensusState.selector, fixture.conflictingUpdate.height
            )
        );
        client.updateClient(_encodeUpdate(fixture.conflictingUpdate));
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
                trustedHeight: IICS02ClientMsgs.Height({ revisionNumber: 0, revisionHeight: update.trustedHeight }),
                accountProof: update.accountProof
            })
        );
    }

    function _singlePath(bytes memory path) internal pure returns (bytes[] memory out) {
        out = new bytes[](1);
        out[0] = path;
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

    function _readRejectionUpdate(
        string memory json,
        string memory path
    )
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
                trustedHeight: IICS02ClientMsgs.Height({ revisionNumber: 0, revisionHeight: update.trustedHeight }),
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
