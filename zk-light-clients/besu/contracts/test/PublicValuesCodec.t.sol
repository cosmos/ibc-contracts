// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import { Test } from "forge-std/Test.sol";

import { BesuZKTypes } from "../src/BesuZKTypes.sol";
import { PublicValuesCodec } from "../src/PublicValuesCodec.sol";

contract CodecHarness {
    function envelope(bytes calldata encoded)
        external
        pure
        returns (BesuZKTypes.SP1ProofEnvelope memory)
    {
        return PublicValuesCodec.decodeEnvelope(encoded);
    }

    function update(bytes calldata encoded)
        external
        pure
        returns (BesuZKTypes.UpdatePublicValues memory)
    {
        return PublicValuesCodec.decodeUpdate(encoded);
    }

    function membership(bytes calldata encoded)
        external
        pure
        returns (BesuZKTypes.MembershipPublicValues memory)
    {
        return PublicValuesCodec.decodeMembership(encoded);
    }

    function nonMembership(bytes calldata encoded)
        external
        pure
        returns (BesuZKTypes.NonMembershipPublicValues memory)
    {
        return PublicValuesCodec.decodeNonMembership(encoded);
    }
}

contract PublicValuesCodecTest is Test {
    CodecHarness private harness;

    function setUp() public {
        harness = new CodecHarness();
    }

    function testRustUpdateGoldenVector() public view {
        bytes memory golden =
            hex"00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000007111111111111111111111111111111111111111111111111111111111111111100000000000000000000000000000000000000000000000000000000000000092222222222222222222222222222222222222222222222222222222222222222000000000000000000000000000000000000000000000000000000006553f1013333333333333333333333333333333333333333333333333333333333333333";
        BesuZKTypes.UpdatePublicValues memory values = harness.update(golden);
        assertEq(values.schemaVersion, 1);
        assertEq(values.trustedHeight, 7);
        assertEq(values.verifiedHeight, 9);
        assertEq(values.blockTimestamp, 1_700_000_001);
        assertEq(abi.encode(values), golden);
    }

    function testRustMembershipGoldenVector() public view {
        bytes memory golden =
            hex"0000000000000000000000000000000000000000000000000000000000000002000000000000000000000000000000000000000000000000ffffffffffffffff1111111111111111111111111111111111111111111111111111111111111111000000000000000000000000222222222222222222222222222222222222222233333333333333333333333333333333333333333333333333333333333333330000000000000000000000000000000000000000000000000000000000000020";
        BesuZKTypes.MembershipPublicValues memory values = harness.membership(golden);
        assertEq(values.proofHeight, type(uint64).max);
        assertEq(values.sourceRouter, address(0x2222222222222222222222222222222222222222));
        assertEq(abi.encode(values), golden);
    }

    function testRustNonMembershipGoldenVector() public view {
        bytes memory golden =
            hex"00000000000000000000000000000000000000000000000000000000000000010000000000000000000000000000000000000000000000000000000000000001111111111111111111111111111111111111111111111111111111111111111100000000000000000000000022222222222222222222222222222222222222223333333333333333333333333333333333333333333333333333333333333333";
        BesuZKTypes.NonMembershipPublicValues memory values = harness.nonMembership(golden);
        assertEq(values.proofHeight, 1);
        assertEq(abi.encode(values), golden);
    }

    function testRejectsWrongSchemaTruncationTrailingAndWrongWidth() public {
        BesuZKTypes.UpdatePublicValues memory values = BesuZKTypes.UpdatePublicValues({
            schemaVersion: 2,
            trustedHeight: 7,
            trustedValidatorSetHash: bytes32(uint256(1)),
            verifiedHeight: 9,
            stateRoot: bytes32(uint256(2)),
            blockTimestamp: 10,
            newValidatorSetHash: bytes32(uint256(3))
        });
        vm.expectRevert(
            abi.encodeWithSelector(PublicValuesCodec.UnsupportedSchemaVersion.selector, uint32(2))
        );
        harness.update(abi.encode(values));

        values.schemaVersion = 1;
        bytes memory valid = abi.encode(values);
        bytes memory truncated = new bytes(valid.length - 1);
        vm.expectRevert(PublicValuesCodec.NonCanonicalEncoding.selector);
        harness.update(truncated);

        bytes memory trailing = bytes.concat(valid, hex"00");
        vm.expectRevert(PublicValuesCodec.NonCanonicalEncoding.selector);
        harness.update(trailing);

        values.schemaVersion = 1;
        bytes memory wrongWidth = abi.encode(values);
        wrongWidth[32] = 0x01;
        vm.expectRevert();
        harness.update(wrongWidth);
    }

    function testMembershipRejectsWrongSchemaTruncationTrailingAndWrongFieldWidths() public {
        BesuZKTypes.MembershipPublicValues memory values = BesuZKTypes.MembershipPublicValues({
            schemaVersion: 1,
            proofHeight: 7,
            stateRoot: bytes32(uint256(1)),
            sourceRouter: address(0x1111111111111111111111111111111111111111),
            batchRoot: bytes32(uint256(2)),
            entryCount: 3
        });
        bytes memory encoded = abi.encode(values);
        vm.expectRevert(
            abi.encodeWithSelector(PublicValuesCodec.UnsupportedSchemaVersion.selector, uint32(1))
        );
        harness.membership(encoded);

        values.schemaVersion = 2;
        encoded = abi.encode(values);
        bytes memory truncated = new bytes(encoded.length - 1);
        vm.expectRevert(PublicValuesCodec.NonCanonicalEncoding.selector);
        harness.membership(truncated);
        vm.expectRevert(PublicValuesCodec.NonCanonicalEncoding.selector);
        harness.membership(bytes.concat(encoded, hex"00"));

        encoded[32] = 0x01;
        vm.expectRevert();
        harness.membership(encoded);
        encoded = abi.encode(values);
        encoded[96] = 0x01;
        vm.expectRevert();
        harness.membership(encoded);
    }

    function testNonMembershipRejectsWrongSchemaTruncationTrailingAndWrongFieldWidths() public {
        BesuZKTypes.NonMembershipPublicValues memory values = BesuZKTypes.NonMembershipPublicValues({
            schemaVersion: 2,
            proofHeight: 7,
            stateRoot: bytes32(uint256(1)),
            sourceRouter: address(0x1111111111111111111111111111111111111111),
            pathDigest: bytes32(uint256(2))
        });
        bytes memory encoded = abi.encode(values);
        vm.expectRevert(
            abi.encodeWithSelector(PublicValuesCodec.UnsupportedSchemaVersion.selector, uint32(2))
        );
        harness.nonMembership(encoded);

        values.schemaVersion = 1;
        encoded = abi.encode(values);
        bytes memory truncated = new bytes(encoded.length - 1);
        vm.expectRevert(PublicValuesCodec.NonCanonicalEncoding.selector);
        harness.nonMembership(truncated);
        vm.expectRevert(PublicValuesCodec.NonCanonicalEncoding.selector);
        harness.nonMembership(bytes.concat(encoded, hex"00"));

        encoded[32] = 0x01;
        vm.expectRevert();
        harness.nonMembership(encoded);
        encoded = abi.encode(values);
        encoded[96] = 0x01;
        vm.expectRevert();
        harness.nonMembership(encoded);
    }

    function testEnvelopeRejectsTrailingAndEmptyFields() public {
        BesuZKTypes.SP1ProofEnvelope memory envelope =
            BesuZKTypes.SP1ProofEnvelope({ publicValues: hex"01", proofBytes: hex"02" });
        bytes memory valid = abi.encode(envelope);
        BesuZKTypes.SP1ProofEnvelope memory decoded = harness.envelope(valid);
        assertEq(decoded.publicValues, hex"01");

        vm.expectRevert(PublicValuesCodec.NonCanonicalEncoding.selector);
        harness.envelope(bytes.concat(valid, hex"00"));

        envelope.publicValues = "";
        vm.expectRevert(PublicValuesCodec.EmptyPublicValues.selector);
        harness.envelope(abi.encode(envelope));

        envelope.publicValues = hex"01";
        envelope.proofBytes = "";
        vm.expectRevert(PublicValuesCodec.EmptyProof.selector);
        harness.envelope(abi.encode(envelope));
    }

    function testRustPathDigestAndStorageSlotGoldenVectors() public pure {
        bytes[] memory path = new bytes[](1);
        path[0] = hex"636c69656e742d30010000000000000001";
        bytes32 digest = keccak256(abi.encode(path));
        assertEq(digest, 0x96f56efbd67677c590346e37da4be678ec7e2dde8597350558f7ff406e930598);

        bytes32 ibcStoreSlot = 0x1260944489272988d9df285149b5aa1b0f48f2136d6f416159f840a3e0747600;
        bytes32 logicalSlot = keccak256(abi.encode(keccak256(path[0]), ibcStoreSlot));
        assertEq(logicalSlot, 0x79a5307f4993e7eb3894d3aafa592f86bb1b605ca1a506e595f1874343b9a987);
    }
}
