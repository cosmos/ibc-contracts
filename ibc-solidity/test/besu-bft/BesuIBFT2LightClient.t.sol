// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import {IBesuLightClient} from "../../contracts/light-clients/besu/interfaces/IBesuLightClient.sol";
import {BesuLightClientFixtureTestBase} from "./BesuLightClientFixtureTestBase.sol";

contract BesuIBFT2LightClientTest is BesuLightClientFixtureTestBase {
    function _fixtureFile() internal pure override returns (string memory) {
        return "ibft2.json";
    }

    function _deployPrimaryClient() internal override returns (IBesuLightClient) {
        return _deployIBFT2();
    }

    function _deployWrongWrapper() internal override returns (IBesuLightClient) {
        return _deployQBFT();
    }
}
