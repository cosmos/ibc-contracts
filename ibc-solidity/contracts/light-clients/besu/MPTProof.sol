// SPDX-License-Identifier: Apache-2.0
// OpenZeppelin Contracts (last updated v5.6.0) (utils/cryptography/TrieProof.sol)
// Derived from OpenZeppelin Contracts v5.6.1 TrieProof.sol:
// https://github.com/OpenZeppelin/openzeppelin-contracts/blob/v5.6.1/contracts/utils/cryptography/TrieProof.sol
pragma solidity ^0.8.28;

// Keep the upstream traversal structure easy to diff when upgrading OpenZeppelin.
// solhint-disable code-complexity, function-max-lines, gas-increment-by-one, gas-strict-inequalities,
// solhint-disable no-inline-assembly

import { Bytes } from "@openzeppelin-contracts/utils/Bytes.sol";
import { Memory } from "@openzeppelin-contracts/utils/Memory.sol";
import { RLP } from "@openzeppelin-contracts/utils/RLP.sol";

/**
 * @title Ethereum Merkle-Patricia trie proof verification
 * @notice Library for verifying Ethereum Merkle-Patricia trie inclusion and exclusion proofs.
 *
 * This is a narrow fork of OpenZeppelin's `TrieProof` library. OpenZeppelin v5.6.1 only accepts
 * inclusion proofs, while IBC non-membership verification also requires authenticated exclusion.
 * The traversal therefore returns an explicit `exists` flag and accepts these exclusion terminals:
 *
 * * an empty trie,
 * * an empty branch child or branch value, and
 * * a compact leaf or extension path that diverges from the requested key.
 *
 * Exclusion is accepted only at the final supplied proof element. The proof wire format remains
 * this light client's existing RLP-encoded list of RLP nodes.
 */
library MPTProof {
    using Bytes for *;
    using RLP for *;
    using Memory for *;

    /// @notice Hex-prefix encodings for extension and leaf paths.
    enum Prefix {
        EXTENSION_EVEN, // 0 - Extension node with even length path
        EXTENSION_ODD, // 1 - Extension node with odd length path
        LEAF_EVEN, // 2 - Leaf node with even length path
        LEAF_ODD // 3 - Leaf node with odd length path
    }

    /// @notice Errors that can occur while traversing a trie proof.
    enum ProofError {
        NO_ERROR, // No error occurred during proof traversal
        EMPTY_KEY, // The provided key is empty
        INVALID_ROOT, // The validation of the root node failed
        INVALID_LARGE_NODE, // The validation of a large node failed
        INVALID_SHORT_NODE, // The validation of a short node failed
        EMPTY_PATH, // The path in a leaf or extension node is empty
        INVALID_PATH_REMAINDER, // The path remainder in a leaf or extension node is invalid
        EMPTY_EXTENSION_PATH_REMAINDER, // The path remainder in an extension node is empty
        INVALID_EXTRA_PROOF_ELEMENT, // A leaf value should be the last proof element
        EMPTY_VALUE, // The leaf value is empty
        MISMATCH_LEAF_PATH_KEY_REMAINDER, // The path remainder in a leaf node doesn't match the key remainder
        UNKNOWN_NODE_PREFIX, // The node prefix is unknown
        UNPARSEABLE_NODE, // The node cannot be parsed from RLP encoding
        INVALID_PROOF // General failure during proof traversal
    }

    /// @notice Indicates that trie proof traversal failed.
    /// @param err The traversal error.
    error TrieProofTraversalError(ProofError err);

    /// @notice The radix of the Ethereum trie.
    uint256 internal constant EVM_TREE_RADIX = 16;

    /// @notice Number of items in a branch node (16 children and one value).
    uint256 internal constant BRANCH_NODE_LENGTH = EVM_TREE_RADIX + 1;

    /// @notice Number of items in leaf or extension nodes.
    uint256 internal constant LEAF_OR_EXTENSION_NODE_LENGTH = 2;

    /// @notice Root hash of the empty Ethereum Merkle-Patricia trie.
    bytes32 internal constant EMPTY_TRIE_ROOT = 0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421;

    /// @notice Verifies an RLP-encoded proof and returns whether `key` exists and its value.
    /// @dev Reverts with {TrieProofTraversalError} if the proof is malformed or not linked to `root`.
    /// @param rlpProof An RLP list containing the encoded trie nodes.
    /// @param root The expected trie root.
    /// @param key The trie key to verify.
    /// @return exists Whether the key is present in the trie.
    /// @return value The value associated with the key, or empty bytes when it is absent.
    function verifyRLPProof(
        bytes memory rlpProof,
        bytes32 root,
        bytes32 key
    )
        internal
        pure
        returns (bool exists, bytes memory value)
    {
        ProofError err;
        (value, err) = _tryTraverse(root, abi.encodePacked(key), _decodeProof(rlpProof));
        require(err == ProofError.NO_ERROR, TrieProofTraversalError(err));
        exists = value.length != 0;
    }

    /// @notice Traverses a proof and reports its value and error without reverting on traversal errors.
    /// @dev This function may still revert if malformed input leads to RLP decoding errors.
    /// @param root The expected trie root.
    /// @param key The trie key to verify.
    /// @param proof The encoded trie nodes, ordered from root to terminal node.
    /// @return value The value associated with the key, or empty bytes when it is absent.
    /// @return err The traversal error, or `NO_ERROR` for a valid inclusion or exclusion proof.
    function _tryTraverse(
        bytes32 root,
        bytes memory key,
        bytes[] memory proof
    )
        private
        pure
        returns (bytes memory, ProofError)
    {
        if (key.length == 0) return (_emptyBytesMemory(), ProofError.EMPTY_KEY);
        if (proof.length == 0) {
            return root == EMPTY_TRIE_ROOT
                ? (_emptyBytesMemory(), ProofError.NO_ERROR)
                : (_emptyBytesMemory(), ProofError.INVALID_PROOF);
        }

        // Expand the key
        bytes memory keyExpanded = key.toNibbles();

        // These are assigned before use whenever traversal advances beyond the root.
        // slither-disable-next-line uninitialized-local
        bytes32 currentNodeId;
        // slither-disable-next-line uninitialized-local
        uint256 currentNodeIdLength;

        // Free memory pointer cache
        Memory.Pointer fmp = Memory.getFreeMemoryPointer();

        // Traverse proof
        uint256 keyIndex = 0;
        for (uint256 i = 0; i < proof.length; ++i) {
            // validates the encoded node matches the expected node id
            bytes memory encoded = proof[i];
            if (keyIndex == 0) {
                // Root node must match root hash
                if (keccak256(encoded) != root) {
                    return (_emptyBytesMemory(), ProofError.INVALID_ROOT);
                }
            } else if (encoded.length >= 32) {
                // Large nodes are stored as hashes
                if (currentNodeIdLength != 32 || keccak256(encoded) != currentNodeId) {
                    return (_emptyBytesMemory(), ProofError.INVALID_LARGE_NODE);
                }
            } else {
                // Short nodes must match directly
                if (currentNodeIdLength != encoded.length || bytes32(encoded) != currentNodeId) {
                    return (_emptyBytesMemory(), ProofError.INVALID_SHORT_NODE);
                }
            }

            // decode the current node as an RLP list, and process it
            for (Memory.Slice[] memory decoded = encoded.decodeList();;) {
                if (decoded.length == BRANCH_NODE_LENGTH) {
                    // If we've consumed the entire key, the value must be in the last slot
                    // Otherwise, continue down the branch specified by the next nibble in the key
                    if (keyIndex == keyExpanded.length) {
                        return _validateLastItem(decoded[EVM_TREE_RADIX], proof.length, i, true);
                    } else {
                        bytes1 branchKey = keyExpanded[keyIndex];
                        Memory.Slice childNode = decoded[uint8(branchKey)];
                        if (_isEmpty(childNode)) {
                            return _validateExclusion(proof.length, i);
                        }

                        (currentNodeId, currentNodeIdLength) = _getNodeId(childNode);
                        keyIndex += 1;

                        if (currentNodeIdLength == 32 || _match(childNode, proof, i + 1)) {
                            break;
                        }
                        decoded = childNode.readList();
                    }
                } else if (decoded.length == LEAF_OR_EXTENSION_NODE_LENGTH) {
                    bytes[] memory proof_ = proof;

                    bytes memory path = decoded[0].readBytes().toNibbles(); // expanded path
                    // The following is equivalent to path.length < 2 because toNibbles can't return odd-length buffers
                    if (path.length == 0) {
                        return (_emptyBytesMemory(), ProofError.EMPTY_PATH);
                    }
                    uint8 prefix = uint8(path[0]); // path encoding nibble (node type + parity), see {Prefix}
                    if (prefix > uint8(Prefix.LEAF_ODD)) {
                        return (_emptyBytesMemory(), ProofError.UNKNOWN_NODE_PREFIX);
                    }

                    Memory.Slice keyRemainder = keyExpanded.asSlice().slice(keyIndex); // Remaining key to match
                    Memory.Slice pathRemainder = path.asSlice().slice(2 - (prefix % 2)); // Path after the prefix
                    uint256 pathRemainderLength = pathRemainder.length();

                    // pathRemainder must not be longer than keyRemainder and must match the start of keyRemainder
                    if (
                        pathRemainderLength > keyRemainder.length()
                            || !pathRemainder.equal(keyRemainder.slice(0, pathRemainderLength))
                    ) {
                        return _validateExclusion(proof_.length, i);
                    }

                    if (prefix <= uint8(Prefix.EXTENSION_ODD)) {
                        // Eq to: prefix == EXTENSION_EVEN || prefix == EXTENSION_ODD
                        if (pathRemainderLength == 0) {
                            return (_emptyBytesMemory(), ProofError.EMPTY_EXTENSION_PATH_REMAINDER);
                        }
                        // Increment keyIndex by the number of nibbles consumed and continue traversal
                        Memory.Slice childNode = decoded[1];
                        (currentNodeId, currentNodeIdLength) = _getNodeId(childNode);
                        keyIndex += pathRemainderLength;

                        if (currentNodeIdLength == 32 || _match(childNode, proof_, i + 1)) {
                            break;
                        }
                        decoded = childNode.readList();
                    } else if (prefix <= uint8(Prefix.LEAF_ODD)) {
                        // Eq to: prefix == LEAF_EVEN || prefix == LEAF_ODD
                        //
                        // Leaf node (terminal) - return its value if key matches completely
                        // we already know that pathRemainder is a prefix of keyRemainder, so checking the length
                        // sufficient
                        return pathRemainderLength == keyRemainder.length()
                            ? _validateLastItem(decoded[1], proof_.length, i, false)
                            : _validateExclusion(proof_.length, i);
                    }
                } else {
                    return (_emptyBytesMemory(), ProofError.UNPARSEABLE_NODE);
                }
            }
            // Reset memory before next iteration. Deallocates `decoded` and `path`.
            Memory.unsafeSetFreeMemoryPointer(fmp);
        }

        // If we've gone through all proof elements without finding a value, the proof is invalid
        return (_emptyBytesMemory(), ProofError.INVALID_PROOF);
    }

    /// @notice Validates that an exclusion terminal is the final proof element.
    /// @param trieProofLength The total number of proof elements.
    /// @param i The index of the current proof element.
    /// @return value Always empty bytes because the key is absent.
    /// @return err The validation error, or `NO_ERROR` for a valid exclusion terminal.
    function _validateExclusion(
        uint256 trieProofLength,
        uint256 i
    )
        private
        pure
        returns (bytes memory value, ProofError err)
    {
        return i == trieProofLength - 1
            ? (_emptyBytesMemory(), ProofError.NO_ERROR)
            : (_emptyBytesMemory(), ProofError.INVALID_EXTRA_PROOF_ELEMENT);
    }

    /// @notice Validates a terminal trie item and ensures it is the final proof element.
    /// @dev Branch values may be empty to prove exclusion; leaf values may not be empty.
    /// @param item The terminal branch or leaf item.
    /// @param trieProofLength The total number of proof elements.
    /// @param i The index of the current proof element.
    /// @param emptyMeansAbsent Whether an empty item is a valid exclusion proof.
    /// @return value The terminal value, or empty bytes when the key is absent.
    /// @return err The validation error, or `NO_ERROR` for a valid terminal item.
    function _validateLastItem(
        Memory.Slice item,
        uint256 trieProofLength,
        uint256 i,
        bool emptyMeansAbsent
    )
        private
        pure
        returns (bytes memory, ProofError)
    {
        if (i != trieProofLength - 1) {
            return (_emptyBytesMemory(), ProofError.INVALID_EXTRA_PROOF_ELEMENT);
        }
        bytes memory value = item.readBytes();
        if (value.length == 0) {
            return emptyMeansAbsent
                ? (_emptyBytesMemory(), ProofError.NO_ERROR)
                : (_emptyBytesMemory(), ProofError.EMPTY_VALUE);
        }
        return (value, ProofError.NO_ERROR);
    }

    /**
     * @notice Extracts the node ID as a hash or raw data based on its size.
     * @param node The encoded trie node.
     * @return nodeId The hashed or inline node identifier.
     * @return nodeIdLength The length of the inline identifier, or 32 for a hashed node.
     *
     * For short nodes (encoded length < 32 bytes) the node ID is the node content itself,
     * For larger nodes, the node ID is the hash of the encoded node data.
     *
     * [NOTE]
     * ====
     * If a 32-byte input is provided (can occur with inline child references), it is used directly (like short nodes).
     * When `nodeIdLength == 32`, inline processing is skipped. The next traversal step then checks whether the next
     * node is large and its hash matches those raw bytes. If that is not the case, it returns {INVALID_LARGE_NODE}.
     *
     * Empty branch children are handled as exclusion terminals before this function is called.
     * ====
     */
    function _getNodeId(Memory.Slice node) private pure returns (bytes32 nodeId, uint256 nodeIdLength) {
        uint256 nodeLength = node.length();
        return nodeLength < 33 ? (node.load(0), nodeLength) : (node.readBytes32(), 32);
    }

    /// @notice Decodes the light client's RLP list of already RLP-encoded proof nodes.
    /// @param rlpProof The RLP-encoded list of proof nodes.
    /// @return proof The individual encoded proof nodes.
    function _decodeProof(bytes memory rlpProof) private pure returns (bytes[] memory proof) {
        Memory.Slice[] memory items = rlpProof.decodeList();
        proof = new bytes[](items.length);
        for (uint256 i = 0; i < items.length; ++i) {
            proof[i] = items[i].toBytes();
        }
    }

    /// @notice Returns whether an RLP item is the canonical empty byte string.
    /// @param item The RLP item to inspect.
    /// @return Whether the item is the canonical empty byte string.
    function _isEmpty(Memory.Slice item) private pure returns (bool) {
        return item.length() == 1 && bytes1(item.load(0)) == 0x80;
    }

    /// @notice Returns an empty byte array without allocating memory.
    /// @return result The empty byte array.
    function _emptyBytesMemory() private pure returns (bytes memory result) {
        assembly ("memory-safe") {
            result := 0x60 // mload(0x60) is always 0
        }
    }

    /// @notice Returns whether a slice matches the array item at `index`.
    /// @param slice The memory slice to compare.
    /// @param array The byte-array collection containing the candidate item.
    /// @param index The candidate item index.
    /// @return Whether the index exists and the values match.
    function _match(Memory.Slice slice, bytes[] memory array, uint256 index) private pure returns (bool) {
        return index < array.length && slice.equal(array[index].asSlice());
    }
}
