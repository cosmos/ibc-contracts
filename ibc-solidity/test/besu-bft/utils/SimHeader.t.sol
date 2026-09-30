// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable gas-small-strings

import { Test } from "forge-std/Test.sol";
import { stdJson } from "forge-std/StdJson.sol";
import { ECDSA } from "@openzeppelin-contracts/utils/cryptography/ECDSA.sol";
import { SimHeader } from "./SimHeader.sol";

/// @dev Checks the codec against live Besu QBFT headers captured in `fixtures/qbft.json`.
contract SimHeaderTest is Test {
    using stdJson for string;
    using SimHeader for SimHeader.Data;

    string internal json;

    function setUp() public {
        json = vm.readFile(string.concat(vm.projectRoot(), "/test/besu-bft/fixtures/qbft.json"));
    }

    function test_roundTripAndSeals() public view {
        string[3] memory updates = ["adjacentUpdate", "nonAdjacentUpdate", "lowOverlapUpdate"];
        for (uint256 i = 0; i < updates.length; ++i) {
            bytes memory rlp = json.readBytes(string.concat(".", updates[i], ".headerRlp"));
            SimHeader.Data memory h = SimHeader.decode(rlp);
            assertEq(h.encode(), rlp, updates[i]);

            bytes32 digest = h.commitSealDigest(SimHeader.Mode.QBFT);
            assertTrue(h.commitSeals.length != 0);
            for (uint256 j = 0; j < h.commitSeals.length; ++j) {
                bytes memory seal = h.commitSeals[j];
                seal[64] = bytes1(uint8(seal[64]) + 27);
                assertTrue(_contains(h.validators, ECDSA.recover(digest, seal)), "seal signer");
            }
        }
    }

    function test_blockHashChainsLiveHeaders() public view {
        SimHeader.Data memory parent = SimHeader.decode(json.readBytes(".adjacentUpdate.headerRlp"));
        SimHeader.Data memory child = SimHeader.decode(json.readBytes(".nonAdjacentUpdate.headerRlp"));
        assertEq(child.number, parent.number + 1);
        assertEq(child.parentHash, parent.blockHash(SimHeader.Mode.QBFT));
    }

    function _contains(address[] memory set, address a) private pure returns (bool) {
        for (uint256 i = 0; i < set.length; ++i) {
            if (set[i] == a) return true;
        }
        return false;
    }
}
