// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import { Test } from "forge-std/Test.sol";
import { AccessManager } from "@openzeppelin-contracts/access/manager/AccessManager.sol";

import { ILightClientMsgs } from "@ibc-contracts/msgs/ILightClientMsgs.sol";
import { IICS02ClientMsgs } from "@ibc-contracts/msgs/IICS02ClientMsgs.sol";
import { SP1VerifierGateway } from "@sp1-contracts/SP1VerifierGateway.sol";
import { ISP1VerifierGatewayErrors } from "@sp1-contracts/ISP1VerifierGateway.sol";
import { SP1Verifier } from "@sp1-contracts/v6.1.0/SP1VerifierGroth16.sol";

import { BesuQBFTZKLightClient } from "../src/BesuQBFTZKLightClient.sol";
import { BesuZKTypes } from "../src/BesuZKTypes.sol";
import { LightClientProxy } from "./utils/LightClientProxy.sol";

contract SP1VerifierIntegrationTest is Test {
    bytes32 private constant UPDATE_VKEY =
        0x00e61f2a40543dc03ea299df98663b69027201d24b118976a56f62caf72c181e;
    bytes32 private constant MEMBERSHIP_VKEY =
        0x0084c197721bc0916f482267dc40279aeb87c4e7f1ed4cd2696c1378a6b3242d;
    bytes32 private constant NON_MEMBERSHIP_VKEY =
        0x003a984fc87ad1c0dbe45d1bbd9d53ebb24d4d48a546f4ee3826763e8a16b88f;

    SP1VerifierGateway private gateway;

    function setUp() public {
        vm.skip(!vm.envOr("RUN_REAL_SP1", false));
        SP1Verifier verifier = new SP1Verifier();
        gateway = new SP1VerifierGateway(address(this));
        gateway.addRoute(address(verifier));
    }

    function testAllThreeRealGroth16ProofsPassThroughGatewayAndLightClient() public {
        (bytes memory updatePublic, bytes memory updateProof) = _artifact("update-client");
        BesuZKTypes.UpdatePublicValues memory updateValues =
            abi.decode(updatePublic, (BesuZKTypes.UpdatePublicValues));
        BesuQBFTZKLightClient updateClient = _client(
            address(0x1111111111111111111111111111111111111111),
            updateValues.trustedHeight,
            bytes32(uint256(1)),
            updateValues.blockTimestamp - 1,
            updateValues.trustedValidatorSetHash
        );
        vm.warp(updateValues.blockTimestamp);
        assertEq(
            uint8(updateClient.updateClient(_envelope(updatePublic, updateProof))),
            uint8(ILightClientMsgs.UpdateResult.Update)
        );
        (bool found, BesuZKTypes.ConsensusState memory stored) =
            updateClient.getConsensusState(updateValues.verifiedHeight);
        assertTrue(found);
        assertEq(stored.stateRoot, updateValues.stateRoot);

        (bytes memory membershipPublic, bytes memory membershipProof) = _artifact("membership");
        BesuZKTypes.MembershipPublicValues memory membershipValues =
            abi.decode(membershipPublic, (BesuZKTypes.MembershipPublicValues));
        BesuQBFTZKLightClient membershipClient = _client(
            membershipValues.sourceRouter,
            membershipValues.proofHeight,
            membershipValues.stateRoot,
            123,
            bytes32(uint256(1))
        );
        bytes[] memory membershipPath = new bytes[](1);
        membershipPath[0] = abi.encodePacked(bytes("client-1"), uint8(3), _bigEndian(7));
        bytes32 expectedCommitment =
            0x4444444444444444444444444444444444444444444444444444444444444444;
        uint256 membershipTimestamp = membershipClient.verifyMembership(
            ILightClientMsgs.MsgVerifyMembership({
                proof: abi.encode(
                    BesuZKTypes.MembershipBatchProofEnvelope({
                        mode: 1,
                        publicValues: membershipPublic,
                        proofBytes: membershipProof,
                        batchRoot: membershipValues.batchRoot,
                        entryCount: membershipValues.entryCount,
                        leafIndex: 0,
                        merkleProof: new bytes32[](0)
                    })
                ),
                proofHeight: IICS02ClientMsgs.Height(0, membershipValues.proofHeight),
                path: membershipPath,
                value: abi.encodePacked(expectedCommitment)
            })
        );
        assertEq(membershipTimestamp, 123);

        (bytes memory nonMembershipPublic, bytes memory nonMembershipProof) =
            _artifact("non-membership");
        BesuZKTypes.NonMembershipPublicValues memory nonMembershipValues =
            abi.decode(nonMembershipPublic, (BesuZKTypes.NonMembershipPublicValues));
        BesuQBFTZKLightClient nonMembershipClient = _client(
            nonMembershipValues.sourceRouter,
            nonMembershipValues.proofHeight,
            nonMembershipValues.stateRoot,
            456,
            bytes32(uint256(1))
        );
        bytes[] memory receiptPath = new bytes[](1);
        receiptPath[0] = abi.encodePacked(bytes("client-2"), uint8(2), _bigEndian(8));
        uint256 nonMembershipTimestamp = nonMembershipClient.verifyNonMembership(
            ILightClientMsgs.MsgVerifyNonMembership({
                proof: _envelope(nonMembershipPublic, nonMembershipProof),
                proofHeight: IICS02ClientMsgs.Height(0, nonMembershipValues.proofHeight),
                path: receiptPath
            })
        );
        assertEq(nonMembershipTimestamp, 456);
    }

    function testGatewayRejectsTamperingWrongVKeyMissingRouteAndFrozenRoute() public {
        (bytes memory publicValues, bytes memory proofBytes) = _artifact("membership");
        gateway.verifyProof(MEMBERSHIP_VKEY, publicValues, proofBytes);

        bytes memory tamperedPublic = bytes.concat(publicValues);
        tamperedPublic[0] = bytes1(uint8(tamperedPublic[0]) ^ 1);
        vm.expectRevert();
        gateway.verifyProof(MEMBERSHIP_VKEY, tamperedPublic, proofBytes);

        vm.expectRevert();
        gateway.verifyProof(UPDATE_VKEY, publicValues, proofBytes);

        SP1VerifierGateway missingRoute = new SP1VerifierGateway(address(this));
        vm.expectRevert();
        missingRoute.verifyProof(MEMBERSHIP_VKEY, publicValues, proofBytes);

        bytes4 selector = bytes4(proofBytes);
        gateway.freezeRoute(selector);
        vm.expectRevert(
            abi.encodeWithSelector(ISP1VerifierGatewayErrors.RouteIsFrozen.selector, selector)
        );
        gateway.verifyProof(MEMBERSHIP_VKEY, publicValues, proofBytes);
    }

    function testAllProgramVKeysAreIsolatedAndArtifactsRejectEveryTamperClass() public {
        _assertArtifactIsolationAndTampering(
            "update-client", UPDATE_VKEY, MEMBERSHIP_VKEY, NON_MEMBERSHIP_VKEY
        );
        _assertArtifactIsolationAndTampering(
            "membership", MEMBERSHIP_VKEY, UPDATE_VKEY, NON_MEMBERSHIP_VKEY
        );
        _assertArtifactIsolationAndTampering(
            "non-membership", NON_MEMBERSHIP_VKEY, UPDATE_VKEY, MEMBERSHIP_VKEY
        );

        (bytes memory updatePublic,) = _artifact("update-client");
        (, bytes memory membershipProof) = _artifact("membership");
        vm.expectRevert();
        gateway.verifyProof(MEMBERSHIP_VKEY, updatePublic, membershipProof);
    }

    function _assertArtifactIsolationAndTampering(
        string memory program,
        bytes32 correctVKey,
        bytes32 wrongVKeyOne,
        bytes32 wrongVKeyTwo
    ) private {
        (bytes memory publicValues, bytes memory proofBytes) = _artifact(program);
        gateway.verifyProof(correctVKey, publicValues, proofBytes);

        vm.expectRevert();
        gateway.verifyProof(wrongVKeyOne, publicValues, proofBytes);
        vm.expectRevert();
        gateway.verifyProof(wrongVKeyTwo, publicValues, proofBytes);

        bytes memory mutatedPublic = bytes.concat(publicValues);
        mutatedPublic[mutatedPublic.length - 1] =
            bytes1(uint8(mutatedPublic[mutatedPublic.length - 1]) ^ 1);
        vm.expectRevert();
        gateway.verifyProof(correctVKey, mutatedPublic, proofBytes);
        vm.expectRevert();
        gateway.verifyProof(correctVKey, _truncate(publicValues), proofBytes);
        vm.expectRevert();
        gateway.verifyProof(correctVKey, bytes.concat(publicValues, hex"00"), proofBytes);

        bytes memory mutatedProof = bytes.concat(proofBytes);
        mutatedProof[mutatedProof.length - 1] =
            bytes1(uint8(mutatedProof[mutatedProof.length - 1]) ^ 1);
        vm.expectRevert();
        gateway.verifyProof(correctVKey, publicValues, mutatedProof);
        bytes memory wrongSelector = bytes.concat(proofBytes);
        wrongSelector[0] = bytes1(uint8(wrongSelector[0]) ^ 1);
        vm.expectRevert();
        gateway.verifyProof(correctVKey, publicValues, wrongSelector);
        vm.expectRevert();
        gateway.verifyProof(correctVKey, publicValues, _truncate(proofBytes));
    }

    function _truncate(bytes memory value) private pure returns (bytes memory result) {
        require(value.length > 0, "cannot truncate empty fixture");
        result = new bytes(value.length - 1);
        for (uint256 i = 0; i < result.length; ++i) {
            result[i] = value[i];
        }
    }

    function _client(
        address sourceRouter,
        uint64 height,
        bytes32 root,
        uint64 timestamp,
        bytes32 validatorHash
    ) private returns (BesuQBFTZKLightClient) {
        return LightClientProxy.deploy(
            address(new AccessManager(address(this))),
            BesuQBFTZKLightClient.InitParams({
                localIbcRouter: address(this),
                sourceIbcRouter: sourceRouter,
                sp1Verifier: address(gateway),
                updateClientVKey: UPDATE_VKEY,
                membershipVKey: MEMBERSHIP_VKEY,
                nonMembershipVKey: NON_MEMBERSHIP_VKEY,
                initialHeight: height,
                initialStateRoot: root,
                initialBlockTimestamp: timestamp,
                initialValidatorSetHash: validatorHash,
                trustingPeriodSeconds: 0,
                maxClockDriftSeconds: 0
            })
        );
    }

    function _artifact(string memory program)
        private
        view
        returns (bytes memory publicValues, bytes memory proofBytes)
    {
        string memory path = string.concat("../artifacts/proofs/", program, ".json");
        string memory json = vm.readFile(path);
        publicValues = vm.parseJsonBytes(json, ".publicValues");
        proofBytes = vm.parseJsonBytes(json, ".proofBytes");
        bytes32 vkey = vm.parseJsonBytes32(json, ".programVKey");
        if (keccak256(bytes(program)) == keccak256("update-client")) assertEq(vkey, UPDATE_VKEY);
        if (keccak256(bytes(program)) == keccak256("membership")) assertEq(vkey, MEMBERSHIP_VKEY);
        if (keccak256(bytes(program)) == keccak256("non-membership")) {
            assertEq(vkey, NON_MEMBERSHIP_VKEY);
        }
    }

    function _envelope(bytes memory publicValues, bytes memory proofBytes)
        private
        pure
        returns (bytes memory)
    {
        return abi.encode(BesuZKTypes.SP1ProofEnvelope(publicValues, proofBytes));
    }

    function _bigEndian(uint64 value) private pure returns (bytes8 result) {
        result = bytes8(value);
    }
}
