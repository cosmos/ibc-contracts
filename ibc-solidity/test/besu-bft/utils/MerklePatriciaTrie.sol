// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable gas-increment-by-one,gas-strict-inequalities,no-inline-assembly

import { Bytes } from "@openzeppelin-contracts/utils/Bytes.sol";
import { RLP } from "@openzeppelin-contracts/utils/RLP.sol";

/// @title Merkle Patricia Trie
/// @notice Test-only builder that computes the root and membership or exclusion proofs of a hexary MPT from a flat
/// key/value set. The trie is rebuilt from scratch on every call; proof nodes are collected in root-to-leaf order and
/// nodes shorter than 32 bytes are embedded inline like geth and Besu do.
library MerklePatriciaTrie {
    using Bytes for bytes;

    bytes32 internal constant EMPTY_ROOT_HASH = 0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421;
    uint256 private constant NONE = type(uint256).max;

    error DuplicateKey();

    struct Ctx {
        bytes[] nibbles;
        bytes[] values;
        bytes query;
        bytes[] proof;
        uint256 proofLength;
    }

    /// @notice Root of the trie holding `values` (already RLP-encoded) under `keys`.
    function build(bytes[] memory keys, bytes[] memory values) internal pure returns (bytes32 root) {
        (root,) = prove(keys, values, "");
    }

    /// @notice Root and proof nodes for `key`; the proof is a membership proof if `key` is present, otherwise an
    /// exclusion proof whose last node is the witness.
    function prove(
        bytes[] memory keys,
        bytes[] memory values,
        bytes memory key
    )
        internal
        pure
        returns (bytes32 root, bytes[] memory proof)
    {
        if (keys.length == 0) {
            return (EMPTY_ROOT_HASH, new bytes[](0));
        }
        Ctx memory c = Ctx(new bytes[](keys.length), values, key.toNibbles(), new bytes[](66), 0);
        uint256[] memory idx = new uint256[](keys.length);
        for (uint256 i = 0; i < keys.length; ++i) {
            c.nibbles[i] = keys[i].toNibbles();
            idx[i] = i;
        }
        root = keccak256(_node(c, idx, 0, true));

        uint256 count;
        for (uint256 i = 0; i < c.proofLength; ++i) {
            if (i == 0 || c.proof[i].length >= 32) {
                c.proof[count++] = c.proof[i];
            }
        }
        proof = c.proof;
        assembly ("memory-safe") {
            mstore(proof, count)
        }
    }

    function _node(Ctx memory c, uint256[] memory idx, uint256 depth, bool onPath) private pure returns (bytes memory) {
        if (idx.length == 1) {
            bytes memory nibbles = c.nibbles[idx[0]];
            return _record(
                c, onPath, _pair(_compact(_slice(nibbles, depth, nibbles.length), true), RLP.encode(c.values[idx[0]]))
            );
        }
        uint256 end = _commonPrefixEnd(c, idx, depth);
        if (end == depth) {
            return _branch(c, idx, depth, onPath);
        }
        uint256 slot = onPath ? c.proofLength++ : 0;
        bytes memory prefix = _slice(c.nibbles[idx[0]], depth, end);
        bool childOnPath =
            onPath && c.query.length >= end && keccak256(_slice(c.query, depth, end)) == keccak256(prefix);
        bytes memory node = _pair(_compact(prefix, false), _ref(_branch(c, idx, end, childOnPath)));
        if (onPath) {
            c.proof[slot] = node;
        }
        return node;
    }

    function _branch(
        Ctx memory c,
        uint256[] memory idx,
        uint256 depth,
        bool onPath
    )
        private
        pure
        returns (bytes memory)
    {
        uint256 slot = onPath ? c.proofLength++ : 0;
        uint256[16] memory counts;
        uint256 valueIdx = NONE;
        for (uint256 i = 0; i < idx.length; ++i) {
            bytes memory nibbles = c.nibbles[idx[i]];
            if (nibbles.length == depth) {
                require(valueIdx == NONE, DuplicateKey());
                valueIdx = idx[i];
            } else {
                ++counts[uint8(nibbles[depth])];
            }
        }

        bytes[] memory items = new bytes[](17);
        for (uint256 b = 0; b < 16; ++b) {
            if (counts[b] == 0) {
                items[b] = hex"80";
                continue;
            }
            uint256[] memory sub = new uint256[](counts[b]);
            uint256 n;
            for (uint256 i = 0; i < idx.length; ++i) {
                bytes memory nibbles = c.nibbles[idx[i]];
                if (nibbles.length > depth && uint8(nibbles[depth]) == b) {
                    sub[n++] = idx[i];
                }
            }
            bool childOnPath = onPath && c.query.length > depth && uint8(c.query[depth]) == b;
            items[b] = _ref(_node(c, sub, depth + 1, childOnPath));
        }
        items[16] = valueIdx == NONE ? bytes(hex"80") : RLP.encode(c.values[valueIdx]);
        return _record(c, onPath, RLP.encode(items), slot);
    }

    function _record(Ctx memory c, bool onPath, bytes memory node) private pure returns (bytes memory) {
        return _record(c, onPath, node, onPath ? c.proofLength++ : 0);
    }

    function _record(Ctx memory c, bool onPath, bytes memory node, uint256 slot) private pure returns (bytes memory) {
        if (onPath) {
            c.proof[slot] = node;
        }
        return node;
    }

    /// @dev Child reference: inline when shorter than 32 bytes, otherwise the RLP-encoded keccak hash.
    function _ref(bytes memory node) private pure returns (bytes memory) {
        return node.length < 32 ? node : RLP.encode(keccak256(node));
    }

    function _pair(bytes memory encodedPath, bytes memory second) private pure returns (bytes memory) {
        bytes[] memory items = new bytes[](2);
        items[0] = RLP.encode(encodedPath);
        items[1] = second;
        return RLP.encode(items);
    }

    /// @dev Hex-prefix encoding of a nibble path.
    function _compact(bytes memory nibbles, bool isLeaf) private pure returns (bytes memory out) {
        uint256 start = nibbles.length % 2;
        out = new bytes(1 + nibbles.length / 2);
        out[0] = bytes1((isLeaf ? 0x20 : 0x00) | (start == 1 ? 0x10 | uint8(nibbles[0]) : 0));
        for (uint256 i = start; i < nibbles.length; i += 2) {
            out[1 + (i - start) / 2] = bytes1((uint8(nibbles[i]) << 4) | uint8(nibbles[i + 1]));
        }
    }

    /// @dev First nibble position at or after `depth` where the keys in `idx` differ or one of them ends.
    function _commonPrefixEnd(Ctx memory c, uint256[] memory idx, uint256 depth) private pure returns (uint256 end) {
        bytes memory first = c.nibbles[idx[0]];
        for (end = depth; end < first.length; ++end) {
            for (uint256 i = 1; i < idx.length; ++i) {
                bytes memory other = c.nibbles[idx[i]];
                if (other.length == end || other[end] != first[end]) {
                    return end;
                }
            }
        }
    }

    function _slice(bytes memory data, uint256 from, uint256 to) private pure returns (bytes memory out) {
        out = new bytes(to - from);
        for (uint256 i = from; i < to; ++i) {
            out[i - from] = data[i];
        }
    }
}
