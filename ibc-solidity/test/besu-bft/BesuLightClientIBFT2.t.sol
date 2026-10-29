// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { BesuLightClientFixtureTestBase } from "./BesuLightClientFixtureTestBase.sol";

contract BesuLightClientIBFT2Test is BesuLightClientFixtureTestBase {
    function _fixtureFile() internal pure override returns (string memory) {
        return "ibft2.json";
    }
}
