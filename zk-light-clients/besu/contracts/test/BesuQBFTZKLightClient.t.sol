// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import { Test } from "forge-std/Test.sol";
import { AccessManager } from "@openzeppelin-contracts/access/manager/AccessManager.sol";

import { ILightClientMsgs } from "@ibc-contracts/msgs/ILightClientMsgs.sol";
import { IICS02ClientMsgs } from "@ibc-contracts/msgs/IICS02ClientMsgs.sol";

import { BesuQBFTZKLightClient } from "../src/BesuQBFTZKLightClient.sol";
import { BesuZKTypes } from "../src/BesuZKTypes.sol";
import { PublicValuesCodec } from "../src/PublicValuesCodec.sol";
import { MockSP1Verifier } from "./mocks/MockSP1Verifier.sol";
import { LightClientProxy } from "./utils/LightClientProxy.sol";

contract BesuQBFTZKLightClientTest is Test {
    struct ClientConfig {
        address localRouter;
        address sourceRouter;
        address verifierAddress;
        bytes32 updateVKey;
        bytes32 membershipVKey;
        bytes32 nonMembershipVKey;
        uint64 height;
        bytes32 root;
        uint64 timestamp;
        bytes32 validatorHash;
        uint64 trustingPeriod;
        uint64 maxClockDrift;
    }

    uint64 private constant INITIAL_HEIGHT = 7;
    uint64 private constant INITIAL_TIMESTAMP = 1_000;
    bytes32 private constant INITIAL_ROOT = bytes32(uint256(0x1111));
    bytes32 private constant INITIAL_VALIDATORS = bytes32(uint256(0x2222));
    bytes32 private constant UPDATE_VKEY = bytes32(uint256(1));
    bytes32 private constant MEMBERSHIP_VKEY = bytes32(uint256(2));
    bytes32 private constant NON_MEMBERSHIP_VKEY = bytes32(uint256(3));
    address private constant SOURCE_ROUTER = address(0xBEEF);
    bytes private constant PROOF = hex"01020304";

    MockSP1Verifier private verifier;
    AccessManager private manager;
    BesuQBFTZKLightClient private client;

    function setUp() public {
        verifier = new MockSP1Verifier();
        manager = new AccessManager(address(this));
        client = _newClient(_clientConfig());
        vm.warp(1_100);
    }

    function testBootstrapAndCanonicalClientState() public view {
        assertEq(client.latestHeight(), INITIAL_HEIGHT);
        assertEq(client.trustingPeriodSeconds(), 1_000);
        assertEq(client.maxClockDriftSeconds(), 20);
        (bool found, BesuZKTypes.ConsensusState memory state) =
            client.getConsensusState(INITIAL_HEIGHT);
        assertTrue(found);
        assertEq(state.stateRoot, INITIAL_ROOT);
        assertEq(state.blockTimestamp, INITIAL_TIMESTAMP);
        assertEq(state.validatorSetHash, INITIAL_VALIDATORS);
        assertEq(
            client.getClientState(), abi.encode(BesuZKTypes.ClientState(INITIAL_HEIGHT, 1_000, 20))
        );
        assertEq(
            client.getClientState(),
            hex"0000000000000000000000000000000000000000000000000000000000000007"
            hex"00000000000000000000000000000000000000000000000000000000000003e8"
            hex"0000000000000000000000000000000000000000000000000000000000000014"
        );
    }

    function testValidUpdateStoresStateAdvancesAndIsIdempotent() public {
        BesuZKTypes.UpdatePublicValues memory values = _updateValues();
        bytes memory publicValues = abi.encode(values);
        bytes memory envelope = _acceptedEnvelope(UPDATE_VKEY, publicValues);

        assertEq(uint8(client.updateClient(envelope)), uint8(ILightClientMsgs.UpdateResult.Update));
        assertEq(client.latestHeight(), values.verifiedHeight);
        (bool found, BesuZKTypes.ConsensusState memory state) =
            client.getConsensusState(values.verifiedHeight);
        assertTrue(found);
        assertEq(state.stateRoot, values.stateRoot);
        assertEq(state.blockTimestamp, values.blockTimestamp);
        assertEq(state.validatorSetHash, values.newValidatorSetHash);

        assertEq(uint8(client.updateClient(envelope)), uint8(ILightClientMsgs.UpdateResult.NoOp));
    }

    function testUpdateRejectsUnknownTrustHashHeightTimeConflictAndInvalidProof() public {
        BesuZKTypes.UpdatePublicValues memory values = _updateValues();

        values.trustedHeight = 8;
        vm.expectRevert(
            abi.encodeWithSelector(BesuQBFTZKLightClient.ConsensusStateNotFound.selector, uint64(8))
        );
        client.updateClient(_envelope(abi.encode(values)));

        values = _updateValues();
        values.trustedValidatorSetHash = bytes32(uint256(9));
        vm.expectRevert(BesuQBFTZKLightClient.TrustedValidatorSetHashMismatch.selector);
        client.updateClient(_envelope(abi.encode(values)));

        values = _updateValues();
        values.verifiedHeight = INITIAL_HEIGHT;
        vm.expectRevert(
            abi.encodeWithSelector(
                BesuQBFTZKLightClient.InvalidHeightTransition.selector,
                INITIAL_HEIGHT,
                INITIAL_HEIGHT
            )
        );
        client.updateClient(_envelope(abi.encode(values)));

        values = _updateValues();
        values.blockTimestamp = INITIAL_TIMESTAMP;
        vm.expectRevert(
            abi.encodeWithSelector(
                BesuQBFTZKLightClient.NonIncreasingTimestamp.selector,
                INITIAL_TIMESTAMP,
                INITIAL_TIMESTAMP
            )
        );
        client.updateClient(_envelope(abi.encode(values)));

        values = _updateValues();
        values.blockTimestamp = 1_121;
        vm.expectRevert(
            abi.encodeWithSelector(
                BesuQBFTZKLightClient.TimestampTooFarInFuture.selector, uint64(1_121)
            )
        );
        client.updateClient(_envelope(abi.encode(values)));

        values = _updateValues();
        bytes memory publicValues = abi.encode(values);
        verifier.setRejectAll(true);
        vm.expectRevert(MockSP1Verifier.RejectedProof.selector);
        client.updateClient(_envelope(publicValues));

        verifier.setExpected(UPDATE_VKEY, publicValues, PROOF);
        client.updateClient(_envelope(publicValues));
        values.stateRoot = bytes32(uint256(0x9999));
        publicValues = abi.encode(values);
        verifier.setExpected(UPDATE_VKEY, publicValues, PROOF);
        vm.expectRevert(
            abi.encodeWithSelector(
                BesuQBFTZKLightClient.ConflictingConsensusState.selector, values.verifiedHeight
            )
        );
        client.updateClient(_envelope(publicValues));
    }

    function testUpdateRejectsExpiredTrustAndUnauthorizedCaller() public {
        vm.warp(2_000);
        vm.expectRevert(
            abi.encodeWithSelector(
                BesuQBFTZKLightClient.TrustedStateExpired.selector, INITIAL_HEIGHT
            )
        );
        client.updateClient(_envelope(abi.encode(_updateValues())));

        vm.prank(address(0xCAFE));
        vm.expectRevert(
            abi.encodeWithSelector(
                BesuQBFTZKLightClient.UnauthorizedCaller.selector, address(0xCAFE)
            )
        );
        client.updateClient(_envelope(abi.encode(_updateValues())));
    }

    function testTrustAndClockDriftBoundariesAreExact() public {
        BesuQBFTZKLightClient boundaryClient = _deploy(100, 20);
        BesuZKTypes.UpdatePublicValues memory values = _updateValues();
        values.blockTimestamp = 1_119;
        bytes memory publicValues = abi.encode(values);
        verifier.setExpected(UPDATE_VKEY, publicValues, PROOF);

        vm.warp(1_099);
        assertEq(
            uint8(boundaryClient.updateClient(_envelope(publicValues))),
            uint8(ILightClientMsgs.UpdateResult.Update)
        );

        boundaryClient = _deploy(100, 20);
        vm.warp(1_100);
        vm.expectRevert(
            abi.encodeWithSelector(
                BesuQBFTZKLightClient.TrustedStateExpired.selector, INITIAL_HEIGHT
            )
        );
        boundaryClient.updateClient(_envelope(publicValues));

        boundaryClient = _deploy(0, 20);
        vm.warp(1_000_000);
        values.blockTimestamp = 1_000_020;
        publicValues = abi.encode(values);
        verifier.setExpected(UPDATE_VKEY, publicValues, PROOF);
        assertEq(
            uint8(boundaryClient.updateClient(_envelope(publicValues))),
            uint8(ILightClientMsgs.UpdateResult.Update)
        );

        boundaryClient = _deploy(0, 20);
        values.blockTimestamp = 1_000_021;
        publicValues = abi.encode(values);
        vm.expectRevert(
            abi.encodeWithSelector(
                BesuQBFTZKLightClient.TimestampTooFarInFuture.selector, uint64(1_000_021)
            )
        );
        boundaryClient.updateClient(_envelope(publicValues));
    }

    function testMembershipBindsEveryContextFieldAndReturnsSameStateTimestamp() public {
        bytes[] memory path = _path(0x01);
        bytes32 commitment = bytes32(uint256(0xABCD));
        bytes32 batchRoot = _membershipLeaf(path, commitment);
        BesuZKTypes.MembershipPublicValues memory values = BesuZKTypes.MembershipPublicValues({
            schemaVersion: 2,
            proofHeight: INITIAL_HEIGHT,
            stateRoot: INITIAL_ROOT,
            sourceRouter: SOURCE_ROUTER,
            batchRoot: batchRoot,
            entryCount: 1
        });
        bytes memory publicValues = abi.encode(values);
        verifier.setExpected(MEMBERSHIP_VKEY, publicValues, PROOF);
        ILightClientMsgs.MsgVerifyMembership memory message = ILightClientMsgs.MsgVerifyMembership({
            proof: _membershipEnvelope(1, publicValues, PROOF, batchRoot, 1, 0, new bytes32[](0)),
            proofHeight: IICS02ClientMsgs.Height(0, INITIAL_HEIGHT),
            path: path,
            value: abi.encodePacked(commitment)
        });
        assertEq(client.verifyMembership(message), INITIAL_TIMESTAMP);
        bytes32 batchId = client.membershipBatchId(INITIAL_HEIGHT, batchRoot, 1);
        assertTrue(client.isMembershipBatchVerified(batchId));
        message.proof = _membershipEnvelope(2, "", "", batchRoot, 1, 0, new bytes32[](0));
        assertEq(client.verifyMembership(message), INITIAL_TIMESTAMP);

        message.proofHeight.revisionHeight = 8;
        vm.expectRevert(
            abi.encodeWithSelector(BesuQBFTZKLightClient.ConsensusStateNotFound.selector, uint64(8))
        );
        client.verifyMembership(message);
        message.proofHeight.revisionHeight = INITIAL_HEIGHT;

        values.stateRoot = bytes32(uint256(1));
        message.proof = _membershipEnvelope(1, abi.encode(values), PROOF, batchRoot, 1, 0, new bytes32[](0));
        vm.expectRevert(BesuQBFTZKLightClient.StateRootMismatch.selector);
        client.verifyMembership(message);
        values.stateRoot = INITIAL_ROOT;

        values.sourceRouter = address(1);
        message.proof = _membershipEnvelope(1, abi.encode(values), PROOF, batchRoot, 1, 0, new bytes32[](0));
        vm.expectRevert(BesuQBFTZKLightClient.SourceRouterMismatch.selector);
        client.verifyMembership(message);
        values.sourceRouter = SOURCE_ROUTER;

        message.path = _path(0x03);
        message.proof = _membershipEnvelope(2, "", "", batchRoot, 1, 0, new bytes32[](0));
        vm.expectRevert();
        client.verifyMembership(message);
        message.path = path;
        message.value = abi.encodePacked(bytes32(uint256(2)));
        vm.expectRevert();
        client.verifyMembership(message);
    }

    function testMembershipRejectsWrongVKeyProofAndMalformedContexts() public {
        bytes[] memory path = _path(0x01);
        bytes32 commitment = bytes32(uint256(0xABCD));
        bytes32 batchRoot = _membershipLeaf(path, commitment);
        bytes memory publicValues = abi.encode(
            BesuZKTypes.MembershipPublicValues(
                2,
                INITIAL_HEIGHT,
                INITIAL_ROOT,
                SOURCE_ROUTER,
                batchRoot,
                1
            )
        );
        verifier.setExpected(NON_MEMBERSHIP_VKEY, publicValues, PROOF);
        ILightClientMsgs.MsgVerifyMembership memory message = ILightClientMsgs.MsgVerifyMembership({
            proof: _membershipEnvelope(1, publicValues, PROOF, batchRoot, 1, 0, new bytes32[](0)),
            proofHeight: IICS02ClientMsgs.Height(0, INITIAL_HEIGHT),
            path: path,
            value: abi.encodePacked(commitment)
        });
        vm.expectRevert(MockSP1Verifier.RejectedProof.selector);
        client.verifyMembership(message);

        message.proofHeight.revisionNumber = 1;
        vm.expectRevert(
            abi.encodeWithSelector(
                BesuQBFTZKLightClient.UnsupportedRevisionNumber.selector, uint64(1)
            )
        );
        client.verifyMembership(message);
        message.proofHeight.revisionNumber = 0;
        message.path = new bytes[](0);
        vm.expectRevert(
            abi.encodeWithSelector(BesuQBFTZKLightClient.InvalidPathLength.selector, uint256(0))
        );
        client.verifyMembership(message);
        message.path = path;
        message.value = hex"00";
        vm.expectRevert(
            abi.encodeWithSelector(
                BesuQBFTZKLightClient.InvalidMembershipValueLength.selector, uint256(1)
            )
        );
        client.verifyMembership(message);
    }

    function testNonMembershipBindsContextAndSelectsNonMembershipVKey() public {
        bytes[] memory path = _path(0x02);
        BesuZKTypes.NonMembershipPublicValues memory values = BesuZKTypes.NonMembershipPublicValues({
            schemaVersion: 1,
            proofHeight: INITIAL_HEIGHT,
            stateRoot: INITIAL_ROOT,
            sourceRouter: SOURCE_ROUTER,
            pathDigest: keccak256(abi.encode(path))
        });
        bytes memory publicValues = abi.encode(values);
        verifier.setExpected(NON_MEMBERSHIP_VKEY, publicValues, PROOF);
        ILightClientMsgs.MsgVerifyNonMembership memory message = ILightClientMsgs
            .MsgVerifyNonMembership({
            proof: _envelope(publicValues),
            proofHeight: IICS02ClientMsgs.Height(0, INITIAL_HEIGHT),
            path: path
        });
        assertEq(client.verifyNonMembership(message), INITIAL_TIMESTAMP);

        values.pathDigest = bytes32(uint256(7));
        message.proof = _envelope(abi.encode(values));
        vm.expectRevert(BesuQBFTZKLightClient.PathDigestMismatch.selector);
        client.verifyNonMembership(message);

        values.pathDigest = keccak256(abi.encode(path));
        values.proofHeight = INITIAL_HEIGHT + 1;
        message.proof = _envelope(abi.encode(values));
        vm.expectRevert(
            abi.encodeWithSelector(
                BesuQBFTZKLightClient.InvalidHeightTransition.selector,
                INITIAL_HEIGHT,
                INITIAL_HEIGHT + 1
            )
        );
        client.verifyNonMembership(message);

        values.proofHeight = INITIAL_HEIGHT;
        values.stateRoot = bytes32(uint256(7));
        message.proof = _envelope(abi.encode(values));
        vm.expectRevert(BesuQBFTZKLightClient.StateRootMismatch.selector);
        client.verifyNonMembership(message);

        values.stateRoot = INITIAL_ROOT;
        values.sourceRouter = address(7);
        message.proof = _envelope(abi.encode(values));
        vm.expectRevert(BesuQBFTZKLightClient.SourceRouterMismatch.selector);
        client.verifyNonMembership(message);

        values.sourceRouter = SOURCE_ROUTER;
        publicValues = abi.encode(values);
        verifier.setExpected(MEMBERSHIP_VKEY, publicValues, PROOF);
        message.proof = _envelope(publicValues);
        vm.expectRevert(MockSP1Verifier.RejectedProof.selector);
        client.verifyNonMembership(message);
    }

    function testMisbehaviourAlwaysFailsClosed() public {
        vm.expectRevert(BesuQBFTZKLightClient.UnsupportedMisbehaviour.selector);
        client.misbehaviour(hex"00");
    }

    function testInitializeRejectsDuplicateVKeys() public {
        ClientConfig memory config = _clientConfig();
        config.membershipVKey = UPDATE_VKEY;
        vm.expectRevert(BesuQBFTZKLightClient.InvalidProgramVKey.selector);
        _newClient(config);
    }

    function testInitializeRejectsZeroAuthority() public {
        vm.expectRevert(BesuQBFTZKLightClient.ZeroAddress.selector);
        LightClientProxy.deploy(address(0), _initParams(_clientConfig()));
    }

    function testInitializeRejectsZeroAddressesVKeysAndBootstrapFields() public {
        vm.expectRevert(BesuQBFTZKLightClient.ZeroAddress.selector);
        _newClient(
            address(0),
            SOURCE_ROUTER,
            address(verifier),
            UPDATE_VKEY,
            MEMBERSHIP_VKEY,
            NON_MEMBERSHIP_VKEY,
            INITIAL_HEIGHT,
            INITIAL_ROOT,
            INITIAL_TIMESTAMP,
            INITIAL_VALIDATORS
        );

        vm.expectRevert(BesuQBFTZKLightClient.ZeroAddress.selector);
        _newClient(
            address(this),
            address(0),
            address(verifier),
            UPDATE_VKEY,
            MEMBERSHIP_VKEY,
            NON_MEMBERSHIP_VKEY,
            INITIAL_HEIGHT,
            INITIAL_ROOT,
            INITIAL_TIMESTAMP,
            INITIAL_VALIDATORS
        );

        vm.expectRevert(BesuQBFTZKLightClient.ZeroAddress.selector);
        _newClient(
            address(this),
            SOURCE_ROUTER,
            address(0),
            UPDATE_VKEY,
            MEMBERSHIP_VKEY,
            NON_MEMBERSHIP_VKEY,
            INITIAL_HEIGHT,
            INITIAL_ROOT,
            INITIAL_TIMESTAMP,
            INITIAL_VALIDATORS
        );

        vm.expectRevert(BesuQBFTZKLightClient.InvalidProgramVKey.selector);
        _newClient(
            address(this),
            SOURCE_ROUTER,
            address(verifier),
            bytes32(0),
            MEMBERSHIP_VKEY,
            NON_MEMBERSHIP_VKEY,
            INITIAL_HEIGHT,
            INITIAL_ROOT,
            INITIAL_TIMESTAMP,
            INITIAL_VALIDATORS
        );

        vm.expectRevert(BesuQBFTZKLightClient.InvalidBootstrapState.selector);
        _newClient(
            address(this),
            SOURCE_ROUTER,
            address(verifier),
            UPDATE_VKEY,
            MEMBERSHIP_VKEY,
            NON_MEMBERSHIP_VKEY,
            0,
            INITIAL_ROOT,
            INITIAL_TIMESTAMP,
            INITIAL_VALIDATORS
        );

        vm.expectRevert(BesuQBFTZKLightClient.InvalidBootstrapState.selector);
        _newClient(
            address(this),
            SOURCE_ROUTER,
            address(verifier),
            UPDATE_VKEY,
            MEMBERSHIP_VKEY,
            NON_MEMBERSHIP_VKEY,
            INITIAL_HEIGHT,
            bytes32(0),
            INITIAL_TIMESTAMP,
            INITIAL_VALIDATORS
        );

        vm.expectRevert(BesuQBFTZKLightClient.InvalidBootstrapState.selector);
        _newClient(
            address(this),
            SOURCE_ROUTER,
            address(verifier),
            UPDATE_VKEY,
            MEMBERSHIP_VKEY,
            NON_MEMBERSHIP_VKEY,
            INITIAL_HEIGHT,
            INITIAL_ROOT,
            0,
            INITIAL_VALIDATORS
        );

        vm.expectRevert(BesuQBFTZKLightClient.InvalidBootstrapState.selector);
        _newClient(
            address(this),
            SOURCE_ROUTER,
            address(verifier),
            UPDATE_VKEY,
            MEMBERSHIP_VKEY,
            NON_MEMBERSHIP_VKEY,
            INITIAL_HEIGHT,
            INITIAL_ROOT,
            INITIAL_TIMESTAMP,
            bytes32(0)
        );
    }

    function testAllEntryPointsRejectWrongSchemaVersion() public {
        BesuZKTypes.UpdatePublicValues memory updateValues = _updateValues();
        updateValues.schemaVersion = 2;
        vm.expectRevert(
            abi.encodeWithSelector(PublicValuesCodec.UnsupportedSchemaVersion.selector, uint32(2))
        );
        client.updateClient(_envelope(abi.encode(updateValues)));

        bytes[] memory path = _path(0x01);
        BesuZKTypes.MembershipPublicValues memory membershipValues = BesuZKTypes
            .MembershipPublicValues({
            schemaVersion: 1,
            proofHeight: INITIAL_HEIGHT,
            stateRoot: INITIAL_ROOT,
            sourceRouter: SOURCE_ROUTER,
            batchRoot: _membershipLeaf(path, bytes32(uint256(1))),
            entryCount: 1
        });
        ILightClientMsgs.MsgVerifyMembership memory membership = ILightClientMsgs
            .MsgVerifyMembership({
            proof: _membershipEnvelope(
                1,
                abi.encode(membershipValues),
                PROOF,
                membershipValues.batchRoot,
                1,
                0,
                new bytes32[](0)
            ),
            proofHeight: IICS02ClientMsgs.Height(0, INITIAL_HEIGHT),
            path: path,
            value: abi.encodePacked(bytes32(uint256(1)))
        });
        vm.expectRevert(
            abi.encodeWithSelector(PublicValuesCodec.UnsupportedSchemaVersion.selector, uint32(1))
        );
        client.verifyMembership(membership);

        path = _path(0x02);
        BesuZKTypes.NonMembershipPublicValues memory nonMembershipValues = BesuZKTypes
            .NonMembershipPublicValues({
            schemaVersion: 2,
            proofHeight: INITIAL_HEIGHT,
            stateRoot: INITIAL_ROOT,
            sourceRouter: SOURCE_ROUTER,
            pathDigest: keccak256(abi.encode(path))
        });
        ILightClientMsgs.MsgVerifyNonMembership memory nonMembership = ILightClientMsgs
            .MsgVerifyNonMembership({
            proof: _envelope(abi.encode(nonMembershipValues)),
            proofHeight: IICS02ClientMsgs.Height(0, INITIAL_HEIGHT),
            path: path
        });
        vm.expectRevert(
            abi.encodeWithSelector(PublicValuesCodec.UnsupportedSchemaVersion.selector, uint32(2))
        );
        client.verifyNonMembership(nonMembership);
    }

    function _updateValues() private pure returns (BesuZKTypes.UpdatePublicValues memory) {
        return BesuZKTypes.UpdatePublicValues({
            schemaVersion: 1,
            trustedHeight: INITIAL_HEIGHT,
            trustedValidatorSetHash: INITIAL_VALIDATORS,
            verifiedHeight: 9,
            stateRoot: bytes32(uint256(0x3333)),
            blockTimestamp: 1_050,
            newValidatorSetHash: bytes32(uint256(0x4444))
        });
    }

    function _path(uint8 pathType) private pure returns (bytes[] memory path) {
        path = new bytes[](1);
        path[0] = abi.encodePacked(bytes("client-0"), pathType, uint64(1));
    }

    function _acceptedEnvelope(bytes32 vkey, bytes memory publicValues)
        private
        returns (bytes memory)
    {
        verifier.setExpected(vkey, publicValues, PROOF);
        return _envelope(publicValues);
    }

    function _envelope(bytes memory publicValues) private pure returns (bytes memory) {
        return abi.encode(BesuZKTypes.SP1ProofEnvelope(publicValues, PROOF));
    }

    function _membershipLeaf(bytes[] memory path, bytes32 commitment)
        private
        pure
        returns (bytes32)
    {
        return keccak256(abi.encodePacked(bytes1(0), keccak256(abi.encode(path)), commitment));
    }

    function _membershipEnvelope(
        uint8 mode,
        bytes memory publicValues,
        bytes memory proofBytes,
        bytes32 batchRoot,
        uint32 entryCount,
        uint32 leafIndex,
        bytes32[] memory merkleProof
    ) private pure returns (bytes memory) {
        return abi.encode(
            BesuZKTypes.MembershipBatchProofEnvelope({
                mode: mode,
                publicValues: publicValues,
                proofBytes: proofBytes,
                batchRoot: batchRoot,
                entryCount: entryCount,
                leafIndex: leafIndex,
                merkleProof: merkleProof
            })
        );
    }

    function _deploy(uint64 trustingPeriod, uint64 maxClockDrift)
        private
        returns (BesuQBFTZKLightClient)
    {
        ClientConfig memory config = _clientConfig();
        config.trustingPeriod = trustingPeriod;
        config.maxClockDrift = maxClockDrift;
        return _newClient(config);
    }

    function _newClient(
        address localRouter,
        address sourceRouter,
        address verifierAddress,
        bytes32 updateVKey,
        bytes32 membershipVKey,
        bytes32 nonMembershipVKey,
        uint64 height,
        bytes32 root,
        uint64 timestamp,
        bytes32 validatorHash
    ) private returns (BesuQBFTZKLightClient) {
        ClientConfig memory config;
        config.localRouter = localRouter;
        config.sourceRouter = sourceRouter;
        config.verifierAddress = verifierAddress;
        config.updateVKey = updateVKey;
        config.membershipVKey = membershipVKey;
        config.nonMembershipVKey = nonMembershipVKey;
        config.height = height;
        config.root = root;
        config.timestamp = timestamp;
        config.validatorHash = validatorHash;
        config.trustingPeriod = 1_000;
        config.maxClockDrift = 20;
        return _newClient(config);
    }

    function _clientConfig() private view returns (ClientConfig memory config) {
        config.localRouter = address(this);
        config.sourceRouter = SOURCE_ROUTER;
        config.verifierAddress = address(verifier);
        config.updateVKey = UPDATE_VKEY;
        config.membershipVKey = MEMBERSHIP_VKEY;
        config.nonMembershipVKey = NON_MEMBERSHIP_VKEY;
        config.height = INITIAL_HEIGHT;
        config.root = INITIAL_ROOT;
        config.timestamp = INITIAL_TIMESTAMP;
        config.validatorHash = INITIAL_VALIDATORS;
        config.trustingPeriod = 1_000;
        config.maxClockDrift = 20;
    }

    function _newClient(ClientConfig memory config) private returns (BesuQBFTZKLightClient) {
        return LightClientProxy.deploy(address(manager), _initParams(config));
    }

    function _initParams(ClientConfig memory config)
        private
        pure
        returns (BesuQBFTZKLightClient.InitParams memory)
    {
        return BesuQBFTZKLightClient.InitParams({
            localIbcRouter: config.localRouter,
            sourceIbcRouter: config.sourceRouter,
            sp1Verifier: config.verifierAddress,
            updateClientVKey: config.updateVKey,
            membershipVKey: config.membershipVKey,
            nonMembershipVKey: config.nonMembershipVKey,
            initialHeight: config.height,
            initialStateRoot: config.root,
            initialBlockTimestamp: config.timestamp,
            initialValidatorSetHash: config.validatorHash,
            trustingPeriodSeconds: config.trustingPeriod,
            maxClockDriftSeconds: config.maxClockDrift
        });
    }
}
