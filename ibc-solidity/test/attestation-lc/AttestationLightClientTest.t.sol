// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import {Test} from "forge-std/Test.sol";

import {AttestationLightClient} from "../../contracts/light-clients/attestation/AttestationLightClient.sol";
import {IAttestationMsgs as AM} from "../../contracts/light-clients/attestation/msgs/IAttestationMsgs.sol";
import {
    IAttestationLightClientMsgs
} from "../../contracts/light-clients/attestation/msgs/IAttestationLightClientMsgs.sol";
import {ILightClientMsgs} from "../../contracts/msgs/ILightClientMsgs.sol";
import {IICS02ClientMsgs} from "../../contracts/msgs/IICS02ClientMsgs.sol";
import {
    IAttestationLightClientErrors
} from "../../contracts/light-clients/attestation/errors/IAttestationLightClientErrors.sol";
import {IAccessControl} from "@openzeppelin/contracts/access/IAccessControl.sol";

contract AttestationLightClientTest is Test {
    AttestationLightClient internal client;

    uint64 internal constant INITIAL_HEIGHT = 100;
    uint64 internal constant INITIAL_TS = 1_700_000_000;

    uint256 internal attestorPrivKey1;
    uint256 internal attestorPrivKey2;
    uint256 internal attestorPrivKey3;
    address internal attestorAddr1;
    address internal attestorAddr2;
    address internal attestorAddr3;

    function setUp() public {
        attestorPrivKey1 = 0xA11CE;
        attestorPrivKey2 = 0xB0B;
        attestorPrivKey3 = 0xC0C;
        attestorAddr1 = vm.addr(attestorPrivKey1);
        attestorAddr2 = vm.addr(attestorPrivKey2);
        attestorAddr3 = vm.addr(attestorPrivKey3);

        address[] memory addrs = new address[](3);
        addrs[0] = attestorAddr1;
        addrs[1] = attestorAddr2;
        addrs[2] = attestorAddr3;

        client = new AttestationLightClient({
            attestorAddresses: addrs,
            minRequiredSigs: 2,
            initialHeight: INITIAL_HEIGHT,
            initialTimestampSeconds: INITIAL_TS,
            roleManager: address(0)
        });

        assertEq(client.getConsensusTimestamp(INITIAL_HEIGHT), INITIAL_TS);

        bytes memory cs = client.getClientState();
        IAttestationLightClientMsgs.ClientState memory expected = IAttestationLightClientMsgs.ClientState({
            attestorAddresses: addrs, minRequiredSigs: 2, latestHeight: INITIAL_HEIGHT, isFrozen: false
        });

        assertEq(abi.encode(expected), cs);

        (address[] memory attestationSet, uint8 minSigs) = client.getAttestationSet();
        assertEq(attestationSet, addrs);
        assertEq(minSigs, 2);
    }

    function test_updateClient_success_updates_height_and_ts() public {
        uint64 newHeight = INITIAL_HEIGHT + 1;
        uint64 newTs = INITIAL_TS + 100;

        uint256[] memory signers = new uint256[](2);
        signers[0] = attestorPrivKey1;
        signers[1] = attestorPrivKey2;

        (bytes memory attestationData, bytes[] memory signatures) = _stateAttestation(newHeight, newTs, signers);
        AM.AttestationProof memory proof =
            AM.AttestationProof({attestationData: attestationData, signatures: signatures});

        ILightClientMsgs.UpdateResult res = client.updateClient(abi.encode(proof));
        assertEq(uint8(res), uint8(ILightClientMsgs.UpdateResult.Update));
        assertEq(client.getConsensusTimestamp(newHeight), newTs);
    }

    function test_proofSubmitterRole() public {
        attestorPrivKey1 = 0xA11CE;
        attestorPrivKey2 = 0xB0B;
        attestorPrivKey3 = 0xC0C;
        attestorAddr1 = vm.addr(attestorPrivKey1);
        attestorAddr2 = vm.addr(attestorPrivKey2);
        attestorAddr3 = vm.addr(attestorPrivKey3);

        address[] memory addrs = new address[](3);
        addrs[0] = attestorAddr1;
        addrs[1] = attestorAddr2;
        addrs[2] = attestorAddr3;

        address roleManager = makeAddr("roleManager");

        client = new AttestationLightClient({
            attestorAddresses: addrs,
            minRequiredSigs: 2,
            initialHeight: INITIAL_HEIGHT,
            initialTimestampSeconds: INITIAL_TS,
            roleManager: roleManager
        });

        // Check that the deployer (this contract) has the PROOF_SUBMITTER_ROLE
        bytes32 proofSubmitterRole = keccak256("PROOF_SUBMITTER_ROLE");
        assertTrue(client.hasRole(proofSubmitterRole, address(roleManager)));

        // Check that an arbitrary address does not have the PROOF_SUBMITTER_ROLE
        address unauthorized = makeAddr("unauthorized");
        assertFalse(client.hasRole(proofSubmitterRole, unauthorized));

        vm.prank(unauthorized);
        vm.expectRevert(
            abi.encodeWithSelector(
                IAccessControl.AccessControlUnauthorizedAccount.selector, unauthorized, proofSubmitterRole
            )
        );
        client.updateClient(bytes(""));
    }

    function test_updateClient_noop_same_height_same_ts() public {
        uint256[] memory signers = new uint256[](2);
        signers[0] = attestorPrivKey1;
        signers[1] = attestorPrivKey2;
        (bytes memory attestationData, bytes[] memory signatures) =
            _stateAttestation(INITIAL_HEIGHT, INITIAL_TS, signers);

        AM.AttestationProof memory proof =
            AM.AttestationProof({attestationData: attestationData, signatures: signatures});

        ILightClientMsgs.UpdateResult res = client.updateClient(abi.encode(proof));
        assertEq(uint8(res), uint8(ILightClientMsgs.UpdateResult.NoOp));
    }

    function test_updateClient_noop_same_height_different_ts_returns_misbehaviour() public {
        uint256[] memory signers = new uint256[](2);
        signers[0] = attestorPrivKey1;
        signers[1] = attestorPrivKey2;
        uint64 conflictingTs = INITIAL_TS + 1;
        (bytes memory attestationData, bytes[] memory signatures) =
            _stateAttestation(INITIAL_HEIGHT, conflictingTs, signers);

        AM.AttestationProof memory proof =
            AM.AttestationProof({attestationData: attestationData, signatures: signatures});
        ILightClientMsgs.UpdateResult res = client.updateClient(abi.encode(proof));
        assertEq(uint8(res), uint8(ILightClientMsgs.UpdateResult.Misbehaviour));
    }

    function test_updateClient_out_of_order_update() public {
        uint64 lowerHeight = INITIAL_HEIGHT - 1;
        uint256[] memory signers = new uint256[](2);
        signers[0] = attestorPrivKey1;
        signers[1] = attestorPrivKey2;
        (bytes memory attestationData, bytes[] memory signatures) =
            _stateAttestation(lowerHeight, INITIAL_TS - 1, signers);

        AM.AttestationProof memory proof =
            AM.AttestationProof({attestationData: attestationData, signatures: signatures});

        ILightClientMsgs.UpdateResult res = client.updateClient(abi.encode(proof));
        assertEq(uint8(res), uint8(ILightClientMsgs.UpdateResult.Update));
    }

    function test_updateClient_revert_threshold_not_met() public {
        uint64 newHeight = INITIAL_HEIGHT + 10;
        uint256[] memory signers = new uint256[](1);
        signers[0] = attestorPrivKey1;
        (bytes memory attestationData, bytes[] memory signatures) =
            _stateAttestation(newHeight, INITIAL_TS + 10, signers);

        vm.expectRevert(abi.encodeWithSelector(IAttestationLightClientErrors.ThresholdNotMet.selector, 1, 2));
        AM.AttestationProof memory proof =
            AM.AttestationProof({attestationData: attestationData, signatures: signatures});
        client.updateClient(abi.encode(proof));
    }

    function test_updateClient_revert_duplicate_signer() public {
        bytes memory attestationData =
            abi.encode(AM.StateAttestation({height: INITIAL_HEIGHT + 1, timestamp: INITIAL_TS + 1}));
        bytes32 digest = _taggedDigest(attestationData, STATE_TAG);

        bytes[] memory signatures = new bytes[](2);
        signatures[0] = _sig(attestorPrivKey1, digest);
        signatures[1] = _sig(attestorPrivKey1, digest);

        vm.expectRevert(abi.encodeWithSelector(IAttestationLightClientErrors.DuplicateSigner.selector, attestorAddr1));
        AM.AttestationProof memory proof =
            AM.AttestationProof({attestationData: attestationData, signatures: signatures});
        client.updateClient(abi.encode(proof));
    }

    function test_updateClient_revert_unknown_signer() public {
        bytes memory attestationData =
            abi.encode(AM.StateAttestation({height: INITIAL_HEIGHT + 1, timestamp: INITIAL_TS + 1}));
        bytes32 digest = _taggedDigest(attestationData, STATE_TAG);

        uint256 badPriv = 0xDEADBEEF;
        bytes[] memory signatures = new bytes[](2);
        signatures[0] = _sig(attestorPrivKey1, digest);
        signatures[1] = _sig(badPriv, digest);

        vm.expectRevert(abi.encodeWithSelector(IAttestationLightClientErrors.UnknownSigner.selector, vm.addr(badPriv)));
        AM.AttestationProof memory proof =
            AM.AttestationProof({attestationData: attestationData, signatures: signatures});
        client.updateClient(abi.encode(proof));
    }

    function test_verifyMembership_success_returns_ts() public view {
        AM.PacketCompact[] memory packets = new AM.PacketCompact[](3);
        packets[0] = _packet("a");
        packets[1] = _packet("b");
        packets[2] = _packet("c");

        uint256[] memory signers = new uint256[](2);
        signers[0] = attestorPrivKey1;
        signers[1] = attestorPrivKey2;

        (bytes memory attestationData, bytes[] memory signatures) = _attestation(INITIAL_HEIGHT, packets, signers);
        AM.AttestationProof memory proof =
            AM.AttestationProof({attestationData: attestationData, signatures: signatures});

        ILightClientMsgs.MsgVerifyMembership memory msgVerify;
        msgVerify.proof = abi.encode(proof);
        msgVerify.proofHeight = IICS02ClientMsgs.Height({revisionNumber: 0, revisionHeight: INITIAL_HEIGHT});
        msgVerify.path = new bytes[](1);
        msgVerify.path[0] = "path-b";
        msgVerify.value = abi.encode(_packet("b").commitment);

        uint256 ts = client.verifyMembership(msgVerify);
        assertEq(ts, INITIAL_TS);
    }

    function test_verifyMembership_revert_empty_value() public {
        AM.PacketAttestation memory p =
            AM.PacketAttestation({height: INITIAL_HEIGHT, packets: new AM.PacketCompact[](0)});

        AM.AttestationProof memory proof =
            AM.AttestationProof({attestationData: abi.encode(p), signatures: new bytes[](0)});

        ILightClientMsgs.MsgVerifyMembership memory msgVerify;

        msgVerify.proof = abi.encode(proof);
        msgVerify.proofHeight = IICS02ClientMsgs.Height({revisionNumber: 0, revisionHeight: INITIAL_HEIGHT});
        msgVerify.path = new bytes[](1);
        msgVerify.path[0] = "path-b";
        msgVerify.value = bytes("");

        vm.expectRevert(abi.encodeWithSelector(IAttestationLightClientErrors.EmptyValue.selector));
        client.verifyMembership(msgVerify);
    }

    function test_verifyMembership_revert_not_member() public {
        AM.PacketCompact[] memory packets = new AM.PacketCompact[](2);
        packets[0] = _packet("x");
        packets[1] = _packet("y");

        uint256[] memory signers = new uint256[](2);
        signers[0] = attestorPrivKey2;
        signers[1] = attestorPrivKey3;
        (bytes memory attestationData, bytes[] memory signatures) = _attestation(INITIAL_HEIGHT, packets, signers);

        AM.AttestationProof memory proof =
            AM.AttestationProof({attestationData: attestationData, signatures: signatures});

        ILightClientMsgs.MsgVerifyMembership memory msgVerify;
        msgVerify.proof = abi.encode(proof);
        msgVerify.proofHeight = IICS02ClientMsgs.Height({revisionNumber: 0, revisionHeight: INITIAL_HEIGHT});
        msgVerify.path = new bytes[](1);
        msgVerify.path[0] = "path-z";
        msgVerify.value = abi.encode(_packet("z").commitment);

        vm.expectRevert(abi.encodeWithSelector(IAttestationLightClientErrors.NotMember.selector));
        client.verifyMembership(msgVerify);
    }

    function test_verifyMembership_revert_missing_timestamp_for_height() public {
        uint64 unknownHeight = INITIAL_HEIGHT + 50;

        AM.PacketCompact[] memory packets = new AM.PacketCompact[](1);
        packets[0] = _packet("a");
        uint256[] memory signers = new uint256[](2);
        signers[0] = attestorPrivKey1;
        signers[1] = attestorPrivKey2;
        (bytes memory attestationData, bytes[] memory signatures) = _attestation(unknownHeight, packets, signers);
        AM.AttestationProof memory proof =
            AM.AttestationProof({attestationData: attestationData, signatures: signatures});

        ILightClientMsgs.MsgVerifyMembership memory msgVerify;
        msgVerify.proof = abi.encode(proof);
        msgVerify.proofHeight = IICS02ClientMsgs.Height({revisionNumber: 0, revisionHeight: unknownHeight});
        msgVerify.path = new bytes[](1);
        msgVerify.path[0] = "path-a";
        msgVerify.value = abi.encode(_packet("a").commitment);

        vm.expectRevert(
            abi.encodeWithSelector(IAttestationLightClientErrors.ConsensusTimestampNotFound.selector, unknownHeight)
        );
        client.verifyMembership(msgVerify);
    }

    function test_verifyMembership_revert_height_mismatch() public {
        // Build an attestation with a different height than the proofHeight
        uint64 attestedHeight = INITIAL_HEIGHT + 5;
        AM.PacketCompact[] memory packets = new AM.PacketCompact[](2);
        packets[0] = _packet("x");
        packets[1] = _packet("y");

        uint256[] memory signers = new uint256[](2);
        signers[0] = attestorPrivKey1;
        signers[1] = attestorPrivKey2;

        (bytes memory attestationData, bytes[] memory signatures) = _attestation(attestedHeight, packets, signers);
        AM.AttestationProof memory proof =
            AM.AttestationProof({attestationData: attestationData, signatures: signatures});

        ILightClientMsgs.MsgVerifyMembership memory msgVerify;
        msgVerify.proof = abi.encode(proof);
        // Provide a different height here so the mismatch triggers
        msgVerify.proofHeight = IICS02ClientMsgs.Height({revisionNumber: 0, revisionHeight: INITIAL_HEIGHT});
        msgVerify.path = new bytes[](1);
        msgVerify.path[0] = "path-y";
        msgVerify.value = abi.encode(_packet("y").commitment);

        vm.expectRevert(
            abi.encodeWithSelector(
                IAttestationLightClientErrors.HeightMismatch.selector, INITIAL_HEIGHT, attestedHeight
            )
        );
        client.verifyMembership(msgVerify);
    }

    function test_verifyNonMembership_success() public view {
        uint256[] memory signers = new uint256[](2);
        signers[0] = attestorPrivKey1;
        signers[1] = attestorPrivKey2;

        AM.PacketCompact[] memory packets = new AM.PacketCompact[](1);
        packets[0] = _nonMemberPacket("receipt-path");

        (bytes memory attestationData, bytes[] memory signatures) = _attestation(INITIAL_HEIGHT, packets, signers);
        AM.AttestationProof memory proof =
            AM.AttestationProof({attestationData: attestationData, signatures: signatures});

        bytes[] memory path = new bytes[](1);
        path[0] = "path-receipt-path";

        ILightClientMsgs.MsgVerifyNonMembership memory msgVerify = ILightClientMsgs.MsgVerifyNonMembership({
            proof: abi.encode(proof),
            proofHeight: IICS02ClientMsgs.Height({revisionNumber: 0, revisionHeight: INITIAL_HEIGHT}),
            path: path
        });

        uint256 ts = client.verifyNonMembership(msgVerify);
        assertEq(ts, INITIAL_TS);
    }

    function test_verifyNonMembership_reverts_not_non_member() public {
        uint256[] memory signers = new uint256[](2);
        signers[0] = attestorPrivKey1;
        signers[1] = attestorPrivKey2;

        AM.PacketCompact[] memory packets = new AM.PacketCompact[](1);
        packets[0] = _packet("non-zero-commitment");

        (bytes memory attestationData, bytes[] memory signatures) = _attestation(INITIAL_HEIGHT, packets, signers);
        AM.AttestationProof memory proof =
            AM.AttestationProof({attestationData: attestationData, signatures: signatures});

        bytes[] memory path = new bytes[](1);
        path[0] = "path-non-zero-commitment";

        ILightClientMsgs.MsgVerifyNonMembership memory msgVerify = ILightClientMsgs.MsgVerifyNonMembership({
            proof: abi.encode(proof),
            proofHeight: IICS02ClientMsgs.Height({revisionNumber: 0, revisionHeight: INITIAL_HEIGHT}),
            path: path
        });

        vm.expectRevert(abi.encodeWithSelector(IAttestationLightClientErrors.NotNonMember.selector));
        client.verifyNonMembership(msgVerify);
    }

    function test_verifyNonMembership_reverts_path_not_found() public {
        uint256[] memory signers = new uint256[](2);
        signers[0] = attestorPrivKey1;
        signers[1] = attestorPrivKey2;

        AM.PacketCompact[] memory packets = new AM.PacketCompact[](1);
        packets[0] = _nonMemberPacket("some-path");

        (bytes memory attestationData, bytes[] memory signatures) = _attestation(INITIAL_HEIGHT, packets, signers);
        AM.AttestationProof memory proof =
            AM.AttestationProof({attestationData: attestationData, signatures: signatures});

        bytes[] memory path = new bytes[](1);
        path[0] = "different-path";

        ILightClientMsgs.MsgVerifyNonMembership memory msgVerify = ILightClientMsgs.MsgVerifyNonMembership({
            proof: abi.encode(proof),
            proofHeight: IICS02ClientMsgs.Height({revisionNumber: 0, revisionHeight: INITIAL_HEIGHT}),
            path: path
        });

        vm.expectRevert(abi.encodeWithSelector(IAttestationLightClientErrors.NotMember.selector));
        client.verifyNonMembership(msgVerify);
    }

    function test_verifyNonMembership_reverts_invalid_path_length() public {
        uint256[] memory signers = new uint256[](2);
        signers[0] = attestorPrivKey1;
        signers[1] = attestorPrivKey2;

        AM.PacketCompact[] memory packets = new AM.PacketCompact[](1);
        packets[0] = _nonMemberPacket("receipt-path");

        (bytes memory attestationData, bytes[] memory signatures) = _attestation(INITIAL_HEIGHT, packets, signers);
        AM.AttestationProof memory proof =
            AM.AttestationProof({attestationData: attestationData, signatures: signatures});

        ILightClientMsgs.MsgVerifyNonMembership memory msgVerify = ILightClientMsgs.MsgVerifyNonMembership({
            proof: abi.encode(proof),
            proofHeight: IICS02ClientMsgs.Height({revisionNumber: 0, revisionHeight: INITIAL_HEIGHT}),
            path: new bytes[](0)
        });

        vm.expectRevert(abi.encodeWithSelector(IAttestationLightClientErrors.InvalidPathLength.selector, 1, 0));
        client.verifyNonMembership(msgVerify);
    }

    function test_crossDomainReplay_packetSigRejectedAsState() public {
        uint64 newHeight = INITIAL_HEIGHT + 1;
        uint64 newTs = INITIAL_TS + 100;

        // Sign state attestation data with Packet tag
        bytes memory attestationData = abi.encode(AM.StateAttestation({height: newHeight, timestamp: newTs}));
        bytes32 wrongDigest = _taggedDigest(attestationData, PACKET_TAG);

        bytes[] memory signatures = new bytes[](2);
        signatures[0] = _sig(attestorPrivKey1, wrongDigest);
        signatures[1] = _sig(attestorPrivKey2, wrongDigest);

        AM.AttestationProof memory proof =
            AM.AttestationProof({attestationData: attestationData, signatures: signatures});

        vm.expectRevert();
        client.updateClient(abi.encode(proof));
    }

    function test_crossDomainReplay_stateSigRejectedAsPacket() public {
        AM.PacketCompact[] memory packets = new AM.PacketCompact[](1);
        packets[0] = _packet("a");

        // Sign packet attestation data with State tag
        AM.PacketAttestation memory p = AM.PacketAttestation({height: INITIAL_HEIGHT, packets: packets});
        bytes memory attestationData = abi.encode(p);
        bytes32 wrongDigest = _taggedDigest(attestationData, STATE_TAG);

        bytes[] memory signatures = new bytes[](2);
        signatures[0] = _sig(attestorPrivKey1, wrongDigest);
        signatures[1] = _sig(attestorPrivKey2, wrongDigest);

        AM.AttestationProof memory proof =
            AM.AttestationProof({attestationData: attestationData, signatures: signatures});

        ILightClientMsgs.MsgVerifyMembership memory msgVerify;
        msgVerify.proof = abi.encode(proof);
        msgVerify.proofHeight = IICS02ClientMsgs.Height({revisionNumber: 0, revisionHeight: INITIAL_HEIGHT});
        msgVerify.path = new bytes[](1);
        msgVerify.path[0] = "path-a";
        msgVerify.value = abi.encode(_packet("a").commitment);

        vm.expectRevert();
        client.verifyMembership(msgVerify);
    }

    function test_crossDomainReplay_stateSigRejectedAsNonMembership() public {
        AM.PacketCompact[] memory packets = new AM.PacketCompact[](1);
        packets[0] = _nonMemberPacket("timeout-path");

        // Sign packet attestation data with State tag
        AM.PacketAttestation memory p = AM.PacketAttestation({height: INITIAL_HEIGHT, packets: packets});
        bytes memory attestationData = abi.encode(p);
        bytes32 wrongDigest = _taggedDigest(attestationData, STATE_TAG);

        bytes[] memory signatures = new bytes[](2);
        signatures[0] = _sig(attestorPrivKey1, wrongDigest);
        signatures[1] = _sig(attestorPrivKey2, wrongDigest);

        AM.AttestationProof memory proof =
            AM.AttestationProof({attestationData: attestationData, signatures: signatures});

        ILightClientMsgs.MsgVerifyNonMembership memory msgVerify = ILightClientMsgs.MsgVerifyNonMembership({
            proof: abi.encode(proof),
            proofHeight: IICS02ClientMsgs.Height({revisionNumber: 0, revisionHeight: INITIAL_HEIGHT}),
            path: new bytes[](1)
        });
        msgVerify.path[0] = "path-timeout-path";

        vm.expectRevert();
        client.verifyNonMembership(msgVerify);
    }

    function test_misbehaviour_revert_feature_not_supported() public {
        vm.expectRevert(abi.encodeWithSelector(IAttestationLightClientErrors.FeatureNotSupported.selector));
        client.misbehaviour(bytes(""));
    }

    function _sig(uint256 privKey, bytes32 digest) internal pure returns (bytes memory) {
        (uint8 v, bytes32 r, bytes32 s) = vm.sign(privKey, digest);
        return abi.encodePacked(r, s, v);
    }

    function _packet(string memory value) internal pure returns (AM.PacketCompact memory) {
        return AM.PacketCompact({
            path: keccak256(abi.encodePacked("path-", value)), commitment: keccak256(abi.encodePacked(value))
        });
    }

    function _nonMemberPacket(string memory pathValue) internal pure returns (AM.PacketCompact memory) {
        return AM.PacketCompact({path: keccak256(abi.encodePacked("path-", pathValue)), commitment: bytes32(0)});
    }

    bytes1 private constant STATE_TAG = 0x01;
    bytes1 private constant PACKET_TAG = 0x02;

    function _taggedDigest(bytes memory data, bytes1 typeTag) internal pure returns (bytes32) {
        return sha256(abi.encodePacked(typeTag, sha256(data)));
    }

    function _attestation(uint64 height, AM.PacketCompact[] memory packets, uint256[] memory signers)
        internal
        pure
        returns (bytes memory attestationData, bytes[] memory signatures)
    {
        AM.PacketAttestation memory p = AM.PacketAttestation({height: height, packets: packets});
        attestationData = abi.encode(p);
        bytes32 digest = _taggedDigest(attestationData, PACKET_TAG);
        signatures = new bytes[](signers.length);
        for (uint256 i = 0; i < signers.length; ++i) {
            signatures[i] = _sig(signers[i], digest);
        }
    }

    function _stateAttestation(uint64 height, uint64 timestamp, uint256[] memory signers)
        internal
        pure
        returns (bytes memory attestationData, bytes[] memory signatures)
    {
        AM.StateAttestation memory s = AM.StateAttestation({height: height, timestamp: timestamp});
        attestationData = abi.encode(s);
        bytes32 digest = _taggedDigest(attestationData, STATE_TAG);
        signatures = new bytes[](signers.length);
        for (uint256 i = 0; i < signers.length; ++i) {
            signatures[i] = _sig(signers[i], digest);
        }
    }
}
