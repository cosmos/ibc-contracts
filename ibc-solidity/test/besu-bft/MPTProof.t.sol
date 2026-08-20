// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { Test } from "forge-std/Test.sol";

import { RLP } from "@openzeppelin-contracts/utils/RLP.sol";

import { MPTProof } from "../../contracts/light-clients/besu/MPTProof.sol";

contract MPTProofHarness {
    function verify(bytes calldata proof, bytes32 root, bytes32 key) external pure returns (bool, bytes memory) {
        return MPTProof.verifyRLPProof(proof, root, key);
    }
}

contract MPTProofTest is Test {
    bytes32 private constant EMPTY_TRIE_ROOT = 0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421;

    MPTProofHarness private harness;

    function setUp() public {
        harness = new MPTProofHarness();
    }

    function test_inclusion() public view {
        bytes32 key = bytes32(uint256(1));
        bytes memory value = hex"05";
        bytes memory leaf = _leaf(key, value);

        (bool exists, bytes memory actualValue) = harness.verify(_proof(leaf), keccak256(leaf), key);

        assertTrue(exists);
        assertEq(actualValue, value);
    }

    function test_exclusionEmptyTrie() public view {
        (bool exists, bytes memory value) = harness.verify(RLP.encode(new bytes[](0)), EMPTY_TRIE_ROOT, bytes32(0));

        assertFalse(exists);
        assertEq(value, bytes(""));
    }

    function test_exclusionEmptyBranchChild() public view {
        bytes memory branch = _emptyBranch();

        (bool exists, bytes memory value) = harness.verify(_proof(branch), keccak256(branch), bytes32(0));

        assertFalse(exists);
        assertEq(value, bytes(""));
    }

    function test_exclusionEmptyBranchValue() public view {
        bytes32 key = bytes32(uint256(1));
        bytes memory branch = _emptyBranch();
        bytes memory extension = _extension(key, branch);

        (bool exists, bytes memory value) = harness.verify(_proof(extension), keccak256(extension), key);

        assertFalse(exists);
        assertEq(value, bytes(""));
    }

    function test_exclusionDivergentLeaf() public view {
        bytes32 targetKey;
        bytes memory leaf = _leaf(bytes32(uint256(1) << 252), hex"05");

        (bool exists, bytes memory value) = harness.verify(_proof(leaf), keccak256(leaf), targetKey);

        assertFalse(exists);
        assertEq(value, bytes(""));
    }

    function test_exclusionDivergentExtension() public view {
        bytes32 targetKey;
        bytes memory extension = _extension(bytes32(uint256(1) << 252), _emptyBranch());

        (bool exists, bytes memory value) = harness.verify(_proof(extension), keccak256(extension), targetKey);

        assertFalse(exists);
        assertEq(value, bytes(""));
    }

    function test_revertExtraNodeAfterExclusion() public {
        bytes memory branch = _emptyBranch();
        bytes[] memory nodes = new bytes[](2);
        nodes[0] = branch;
        nodes[1] = branch;

        vm.expectRevert(
            abi.encodeWithSelector(
                MPTProof.TrieProofTraversalError.selector, MPTProof.ProofError.INVALID_EXTRA_PROOF_ELEMENT
            )
        );
        harness.verify(RLP.encode(nodes), keccak256(branch), bytes32(0));
    }

    function _leaf(bytes32 key, bytes memory value) private pure returns (bytes memory) {
        bytes[] memory items = new bytes[](2);
        items[0] = RLP.encode(abi.encodePacked(bytes1(0x20), key));
        items[1] = RLP.encode(value);
        return RLP.encode(items);
    }

    function _extension(bytes32 key, bytes memory child) private pure returns (bytes memory) {
        bytes[] memory items = new bytes[](2);
        items[0] = RLP.encode(abi.encodePacked(bytes1(0), key));
        items[1] = child;
        return RLP.encode(items);
    }

    function _emptyBranch() private pure returns (bytes memory) {
        bytes[] memory items = new bytes[](17);
        bytes memory empty = RLP.encode(bytes(""));
        for (uint256 i = 0; i < items.length; ++i) {
            items[i] = empty;
        }
        return RLP.encode(items);
    }

    function _proof(bytes memory node) private pure returns (bytes memory) {
        bytes[] memory nodes = new bytes[](1);
        nodes[0] = node;
        return RLP.encode(nodes);
    }
}
