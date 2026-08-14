// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

/// @title Paths Library
/// @notice A library defining helper functions for IBC paths
library Paths {
    /// @notice Compares two bytes arrays
    /// @param a The first bytes array
    /// @param b The second bytes array
    /// @return True if the two bytes arrays are equal, false otherwise
    function equal(bytes[] memory a, bytes[] memory b) internal pure returns (bool) {
        if (a.length != b.length) {
            return false;
        }
        for (uint256 i = 0; i < a.length; ++i) {
            if (a[i].length != b[i].length) {
                return false;
            }
            if (keccak256(a[i]) != keccak256(b[i])) {
                return false;
            }
        }
        return true;
    }
}
