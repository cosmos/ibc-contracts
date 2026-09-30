// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable gas-small-strings,gas-calldata-parameters

import { Test } from "forge-std/Test.sol";
import { RLP } from "@openzeppelin-contracts/utils/RLP.sol";
import { TrieProof } from "../../../contracts/utils/TrieProof.sol";
import { MerklePatriciaTrie } from "./MerklePatriciaTrie.sol";

contract MerklePatriciaTrieTest is Test {
    /// @dev OpenZeppelin TrieProof vectors: three keys under a shared extension, two of the leaves inline.
    function test_openZeppelinVectors() public pure {
        bytes[] memory keys = new bytes[](3);
        bytes[] memory values = new bytes[](3);
        keys[0] = "key1aa";
        values[0] = "0123456789012345678901234567890123456789xx";
        keys[1] = "key2bb";
        values[1] = "aval2";
        keys[2] = "key3cc";
        values[2] = "aval3";

        (bytes32 root, bytes[] memory proof) = MerklePatriciaTrie.prove(keys, values, "key2bb");
        assertEq(root, 0xd582f99275e227a1cf4284899e5ff06ee56da8859be71b553397c69151bc942f);
        assertEq(proof.length, 2, "inline leaf is embedded in the branch");
        assertEq(proof[0], hex"e68416b65793a03101b4447781f1e6c51ce76c709274fc80bd064f3a58ff981b6015348a826386");
        assertEq(
            proof[1],
            hex"f84580a0582eed8dd051b823d13f8648cdcd08aa2d8dac239f458863c4620e8c4d605debca83206262856176616c32ca83206363856176616c3380808080808080808080808080"
        );
        assertEq(TrieProof.traverse(root, "key2bb", proof), "aval2");

        (root, proof) = MerklePatriciaTrie.prove(keys, values, "key1aa");
        assertEq(proof.length, 3);
        assertEq(
            proof[2],
            hex"ef83206161aa303132333435363738393031323334353637383930313233343536373839303132333435363738397878"
        );
        assertEq(TrieProof.traverse(root, "key1aa", proof), values[0]);

        bytes[] memory single = new bytes[](1);
        bytes[] memory singleValue = new bytes[](1);
        single[0] = "key1aa";
        singleValue[0] = values[0];
        assertEq(
            MerklePatriciaTrie.build(single, singleValue),
            0xf838216fa749aefa91e0b672a9c06d3e6e983f913d7107b5dab4af60b5f5abed
        );
        singleValue[0] = "01234";
        assertEq(
            MerklePatriciaTrie.build(single, singleValue),
            0x37956bab6bba472308146808d5311ac19cb4a7daae5df7efcc0f32badc97f55e
        );
    }

    function test_emptyTrie() public pure {
        (bytes32 root, bytes[] memory proof) = MerklePatriciaTrie.prove(new bytes[](0), new bytes[](0), "x");
        assertEq(root, MerklePatriciaTrie.EMPTY_ROOT_HASH);
        assertTrue(TrieProof.verifyExclusion(root, "x", proof));
    }

    function test_duplicateKeyReverts() public {
        bytes[] memory keys = new bytes[](2);
        bytes[] memory values = new bytes[](2);
        keys[0] = "a";
        keys[1] = "a";
        values[0] = "1";
        values[1] = "2";
        vm.expectRevert(MerklePatriciaTrie.DuplicateKey.selector);
        this.build(keys, values);
    }

    /// @dev Every present key verifies as a member and every absent key as excluded, for hashed and short keys.
    function testFuzz_roundTrip(uint8 count, bool shortKeys, bytes32 seed) public pure {
        count = uint8(bound(count, 1, 24));
        bytes[] memory keys = new bytes[](count);
        bytes[] memory values = new bytes[](count);
        for (uint256 i = 0; i < count; ++i) {
            bytes32 raw = keccak256(abi.encode(seed, i));
            keys[i] = shortKeys ? abi.encodePacked(bytes2(raw) & bytes2(0x0f0f), uint8(i)) : abi.encodePacked(raw);
            values[i] = RLP.encode(uint256(raw) | 1);
        }

        bytes32 root = MerklePatriciaTrie.build(keys, values);
        for (uint256 i = 0; i < count; ++i) {
            (bytes32 provenRoot, bytes[] memory proof) = MerklePatriciaTrie.prove(keys, values, keys[i]);
            assertEq(provenRoot, root);
            assertEq(keccak256(proof[0]), root);
            assertTrue(TrieProof.verify(values[i], root, keys[i], proof), "membership");
        }

        bytes memory absent = abi.encodePacked(keccak256(abi.encode(seed, "absent")));
        (, bytes[] memory exclusion) = MerklePatriciaTrie.prove(keys, values, absent);
        assertTrue(TrieProof.verifyExclusion(root, absent, exclusion), "exclusion");
        (, exclusion) = MerklePatriciaTrie.prove(keys, values, bytes.concat(keys[0], hex"00"));
        assertTrue(TrieProof.verifyExclusion(root, bytes.concat(keys[0], hex"00"), exclusion), "longer key exclusion");
    }

    function build(bytes[] memory keys, bytes[] memory values) external pure returns (bytes32) {
        return MerklePatriciaTrie.build(keys, values);
    }
}
