// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable gas-struct-packing

import { Test } from "forge-std/Test.sol";
import { stdJson } from "forge-std/StdJson.sol";

import { ILightClient } from "../../contracts/interfaces/ILightClient.sol";
import { ILightClientMsgs } from "../../contracts/msgs/ILightClientMsgs.sol";
import { IICS02ClientMsgs } from "../../contracts/msgs/IICS02ClientMsgs.sol";
import { BesuQBFTLightClient } from "../../contracts/light-clients/besu/BesuQBFTLightClient.sol";
import { IBesuLightClientMsgs } from "../../contracts/light-clients/besu/msgs/IBesuLightClientMsgs.sol";
import { IBesuLightClientErrors } from "../../contracts/light-clients/besu/errors/IBesuLightClientErrors.sol";

struct BesuUpdateFixture {
    uint64 height;
    bytes headerRlp;
    uint64 trustedHeight;
    bytes accountProof;
    uint64 expectedTimestamp;
    bytes32 expectedStorageRoot;
    address[] expectedValidators;
}

struct BesuRejectionUpdateFixture {
    uint64 height;
    bytes headerRlp;
    uint64 trustedHeight;
    bytes accountProof;
}

struct BesuProofFixture {
    bytes proof;
    uint64 proofHeight;
    bytes path;
    bytes value;
    uint64 expectedTimestamp;
}

struct BesuFixture {
    address routerAddress;
    uint64 initialTrustedHeight;
    uint64 initialTrustedTimestamp;
    bytes32 initialTrustedStorageRoot;
    address[] initialTrustedValidators;
    uint64 trustingPeriod;
    uint64 maxClockDrift;
    BesuUpdateFixture updateHeight11;
    BesuUpdateFixture updateHeight12;
    BesuRejectionUpdateFixture lowQuorumHeight12;
    BesuRejectionUpdateFixture conflictingHeight12;
    BesuRejectionUpdateFixture lowOverlapHeight13;
    BesuProofFixture membership;
    BesuProofFixture nonMembership;
}

interface IBesuTestLightClient is ILightClient {
    function getConsensusState(uint64 revisionHeight) external view returns (bytes memory);
}

contract BesuHeaderTestHarness is BesuQBFTLightClient {
    constructor(address[] memory validators)
        BesuQBFTLightClient(address(1), 1, 1, bytes32(0), validators, 1, 1, address(0))
    { }

    function validateHeader(bytes calldata headerRlp) external pure {
        _parseHeader(headerRlp);
    }

    function replaceHeaderBytes(
        bytes calldata headerRlp,
        uint256 index,
        bytes calldata rawValue
    )
        external
        pure
        returns (bytes memory)
    {
        ParsedHeader memory header = _parseHeader(headerRlp);
        bytes[] memory items = new bytes[](header.headerItems.length);
        for (uint256 i = 0; i < items.length; ++i) {
            items[i] = i == index ? _encodeRlpBytes(rawValue) : _rlpItemBytes(header.headerItems[i]);
        }
        return _encodeRlpList(items);
    }

    function replaceExtraDataBytes(
        bytes calldata headerRlp,
        uint256 index,
        bytes calldata rawValue
    )
        external
        pure
        returns (bytes memory)
    {
        ParsedHeader memory header = _parseHeader(headerRlp);
        bytes[] memory extraItems = new bytes[](header.extraDataItems.length);
        for (uint256 i = 0; i < extraItems.length; ++i) {
            extraItems[i] = i == index ? _encodeRlpBytes(rawValue) : _rlpItemBytes(header.extraDataItems[i]);
        }

        bytes[] memory headerItems = new bytes[](header.headerItems.length);
        for (uint256 i = 0; i < headerItems.length; ++i) {
            headerItems[i] =
                i == 12 ? _encodeRlpBytes(_encodeRlpList(extraItems)) : _rlpItemBytes(header.headerItems[i]);
        }
        return _encodeRlpList(headerItems);
    }
}

abstract contract BesuLightClientTestBase is Test {
    using stdJson for string;
    string internal constant FIXTURE_DIR = "/test/besu-bft/fixtures/";

    BesuFixture internal fixture;
    IBesuTestLightClient internal client;
    IBesuTestLightClient internal wrongWrapper;
    BesuHeaderTestHarness internal headerHarness;

    function setUp() public virtual {
        fixture = _loadFixture(_fixtureFile());
        client = _deployPrimaryClient();
        wrongWrapper = _deployWrongWrapper();
        headerHarness = new BesuHeaderTestHarness(fixture.initialTrustedValidators);
    }

    function test_updateClient_validAdjacentUpdate() public {
        vm.warp(fixture.initialTrustedTimestamp + 1);

        ILightClientMsgs.UpdateResult result = client.updateClient(_encodeUpdate(fixture.updateHeight11));

        assertEq(uint8(result), uint8(ILightClientMsgs.UpdateResult.Update));
        _assertClientState(
            fixture.updateHeight11.expectedTimestamp,
            fixture.updateHeight11.expectedStorageRoot,
            fixture.updateHeight11.height
        );
    }

    function test_updateClient_validNonAdjacentUpdate() public {
        vm.warp(fixture.initialTrustedTimestamp + 1);

        ILightClientMsgs.UpdateResult result = client.updateClient(_encodeUpdate(fixture.updateHeight12));

        assertEq(uint8(result), uint8(ILightClientMsgs.UpdateResult.Update));
        _assertClientState(
            fixture.updateHeight12.expectedTimestamp,
            fixture.updateHeight12.expectedStorageRoot,
            fixture.updateHeight12.height
        );
    }

    function test_updateClient_revertZeroTimestampWithoutStateChange() public {
        IBesuLightClientMsgs.MsgUpdateClient memory update =
            abi.decode(_encodeUpdate(fixture.updateHeight12), (IBesuLightClientMsgs.MsgUpdateClient));
        update.headerRlp = headerHarness.replaceHeaderBytes(update.headerRlp, 11, hex"00");

        bytes memory clientStateBefore = client.getClientState();
        bytes memory consensusStateBefore = client.getConsensusState(fixture.initialTrustedHeight);

        vm.expectRevert(abi.encodeWithSelector(IBesuLightClientErrors.InvalidHeaderTimestamp.selector, 0));
        client.updateClient(abi.encode(update));

        assertEq(client.getClientState(), clientStateBefore);
        assertEq(client.getConsensusState(fixture.initialTrustedHeight), consensusStateBefore);
    }

    function test_updateClient_revertSubmittedHeightBeforeTrustedHeight() public {
        vm.warp(fixture.initialTrustedTimestamp + 1);
        client.updateClient(_encodeUpdate(fixture.updateHeight12));

        IBesuLightClientMsgs.MsgUpdateClient memory update =
            abi.decode(_encodeUpdate(fixture.updateHeight11), (IBesuLightClientMsgs.MsgUpdateClient));
        update.trustedHeight.revisionHeight = fixture.updateHeight12.height;

        vm.expectRevert(IBesuLightClientErrors.InvalidHeaderHeight.selector);
        client.updateClient(abi.encode(update));
    }

    function test_updateClient_revertSubmittedHeightEqualsTrustedHeight() public {
        vm.warp(fixture.initialTrustedTimestamp + 1);
        client.updateClient(_encodeUpdate(fixture.updateHeight12));

        IBesuLightClientMsgs.MsgUpdateClient memory update =
            abi.decode(_encodeUpdate(fixture.updateHeight12), (IBesuLightClientMsgs.MsgUpdateClient));
        update.trustedHeight.revisionHeight = fixture.updateHeight12.height;

        vm.expectRevert(IBesuLightClientErrors.InvalidHeaderHeight.selector);
        client.updateClient(abi.encode(update));
    }

    function test_parseHeader_revertHeightOverflow() public {
        bytes memory headerRlp =
            headerHarness.replaceHeaderBytes(fixture.updateHeight12.headerRlp, 8, hex"010000000000000000");

        vm.expectRevert(IBesuLightClientErrors.InvalidHeaderHeight.selector);
        headerHarness.validateHeader(headerRlp);
    }

    function test_parseHeader_revertTimestampOverflow() public {
        bytes memory headerRlp =
            headerHarness.replaceHeaderBytes(fixture.updateHeight12.headerRlp, 11, hex"010000000000000000");

        vm.expectRevert(
            abi.encodeWithSelector(
                IBesuLightClientErrors.InvalidHeaderTimestamp.selector, uint256(type(uint64).max) + 1
            )
        );
        headerHarness.validateHeader(headerRlp);
    }

    function test_parseHeader_revertInvalidVanityDataLength() public {
        bytes memory headerRlp = headerHarness.replaceExtraDataBytes(fixture.updateHeight12.headerRlp, 0, new bytes(31));

        vm.expectRevert(abi.encodeWithSelector(IBesuLightClientErrors.InvalidVanityDataLength.selector, 31));
        headerHarness.validateHeader(headerRlp);
    }

    function test_verifyMembership_returnsStoredTimestamp() public {
        vm.warp(fixture.initialTrustedTimestamp + 1);
        client.updateClient(_encodeUpdate(fixture.updateHeight12));

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
        client.updateClient(_encodeUpdate(fixture.updateHeight12));

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
        client.updateClient(_encodeUpdate(fixture.updateHeight12));
    }

    function test_updateClient_revertInsufficientTrustedOverlap() public {
        vm.warp(fixture.initialTrustedTimestamp + 1);

        vm.expectRevert(
            abi.encodeWithSelector(IBesuLightClientErrors.InsufficientTrustedValidatorOverlap.selector, 1, 2)
        );
        client.updateClient(_encodeUpdate(fixture.lowOverlapHeight13));
    }

    function test_updateClient_revertInsufficientNewValidatorQuorum() public {
        vm.warp(fixture.initialTrustedTimestamp + 1);

        vm.expectRevert(abi.encodeWithSelector(IBesuLightClientErrors.InsufficientValidatorQuorum.selector, 2, 3));
        client.updateClient(_encodeUpdate(fixture.lowQuorumHeight12));
    }

    function test_updateClient_revertWrongRevisionNumber() public {
        vm.warp(fixture.initialTrustedTimestamp + 1);

        IBesuLightClientMsgs.MsgUpdateClient memory update =
            abi.decode(_encodeUpdate(fixture.updateHeight12), (IBesuLightClientMsgs.MsgUpdateClient));
        update.trustedHeight.revisionNumber = 1;

        vm.expectRevert(abi.encodeWithSelector(IBesuLightClientErrors.InvalidRevisionNumber.selector, 1));
        client.updateClient(abi.encode(update));
    }

    function test_verifyMembership_revertWrongRevisionNumber() public {
        vm.warp(fixture.initialTrustedTimestamp + 1);
        client.updateClient(_encodeUpdate(fixture.updateHeight12));

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
        client.updateClient(_encodeUpdate(fixture.updateHeight12));

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
        client.updateClient(_encodeUpdate(fixture.updateHeight12));

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
        client.updateClient(_encodeUpdate(fixture.updateHeight12));

        vm.expectRevert(
            abi.encodeWithSelector(
                IBesuLightClientErrors.ConflictingConsensusState.selector, fixture.conflictingHeight12.height
            )
        );
        client.updateClient(_encodeUpdate(fixture.conflictingHeight12));
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
        wrongWrapper.updateClient(_encodeUpdate(fixture.updateHeight12));
    }

    function _assertClientState(
        uint64 expectedTimestamp,
        bytes32 expectedStorageRoot,
        uint64 expectedLatestHeight
    )
        internal
        view
    {
        (address ibcRouter, IICS02ClientMsgs.Height memory latestHeight, uint64 trustingPeriod, uint64 maxClockDrift) =
            abi.decode(client.getClientState(), (address, IICS02ClientMsgs.Height, uint64, uint64));
        assertEq(ibcRouter, fixture.routerAddress);
        assertEq(latestHeight.revisionNumber, 0);
        assertEq(latestHeight.revisionHeight, expectedLatestHeight);
        assertEq(trustingPeriod, fixture.trustingPeriod);
        assertEq(maxClockDrift, fixture.maxClockDrift);

        (uint64 timestamp, bytes32 storageRoot, address[] memory validators) =
            abi.decode(client.getConsensusState(expectedLatestHeight), (uint64, bytes32, address[]));
        assertEq(timestamp, expectedTimestamp);
        assertEq(storageRoot, expectedStorageRoot);

        address[] memory expectedValidators = expectedLatestHeight == fixture.updateHeight11.height
            ? fixture.updateHeight11.expectedValidators
            : fixture.updateHeight12.expectedValidators;
        assertEq(validators.length, expectedValidators.length);
        for (uint256 i = 0; i < expectedValidators.length; ++i) {
            assertEq(validators[i], expectedValidators[i]);
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
            updateHeight11: _readUpdate(json, ".updateHeight11"),
            updateHeight12: _readUpdate(json, ".updateHeight12"),
            lowQuorumHeight12: _readRejectionUpdate(json, ".lowQuorumHeight12"),
            conflictingHeight12: _readRejectionUpdate(json, ".conflictingHeight12"),
            lowOverlapHeight13: _readRejectionUpdate(json, ".lowOverlapHeight13"),
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

    function _fixtureFile() internal pure virtual returns (string memory);
    function _deployPrimaryClient() internal virtual returns (IBesuTestLightClient);
    function _deployWrongWrapper() internal virtual returns (IBesuTestLightClient);
}
