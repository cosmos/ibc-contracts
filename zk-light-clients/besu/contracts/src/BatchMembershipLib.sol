// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

library BatchMembershipLib {
    error InvalidBatchEntryCount(uint32 entryCount);
    error InvalidLeafIndex(uint32 leafIndex, uint32 entryCount);
    error InvalidMerkleProofLength(uint256 expected, uint256 actual);
    error InvalidMembershipBranch();

    function leaf(bytes32 pathDigest, bytes32 expectedCommitment)
        internal
        pure
        returns (bytes32)
    {
        return keccak256(abi.encodePacked(bytes1(0x00), pathDigest, expectedCommitment));
    }

    function parent(bytes32 left, bytes32 right) internal pure returns (bytes32) {
        return keccak256(abi.encodePacked(bytes1(0x01), left, right));
    }

    function emptyLeaf() internal pure returns (bytes32) {
        return keccak256(hex"02");
    }

    function verify(
        bytes32 expectedRoot,
        bytes32 itemLeaf,
        uint32 leafIndex,
        uint32 entryCount,
        bytes32[] memory proof
    ) internal pure {
        if (entryCount == 0 || entryCount > 32) revert InvalidBatchEntryCount(entryCount);
        if (leafIndex >= entryCount) revert InvalidLeafIndex(leafIndex, entryCount);
        uint256 padded = 1;
        uint256 depth = 0;
        while (padded < entryCount) {
            padded <<= 1;
            ++depth;
        }
        if (proof.length != depth) revert InvalidMerkleProofLength(depth, proof.length);
        bytes32 current = itemLeaf;
        uint256 position = leafIndex;
        for (uint256 i = 0; i < proof.length; ++i) {
            current = (position & 1) == 0
                ? parent(current, proof[i])
                : parent(proof[i], current);
            position >>= 1;
        }
        if (current != expectedRoot) revert InvalidMembershipBranch();
    }
}
