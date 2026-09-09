// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { BesuLightClientFixtureTestBase, IBesuLightClientHarness } from "./BesuLightClientFixtureTestBase.sol";

contract BesuQBFTLightClientTest is BesuLightClientFixtureTestBase {
    function _fixtureFile() internal pure override returns (string memory) {
        return "qbft.json";
    }

    function _deployPrimaryClient() internal override returns (IBesuLightClientHarness) {
        return _deployQBFT();
    }

    function _deployWrongWrapper() internal override returns (IBesuLightClientHarness) {
        return _deployIBFT2();
    }
}
