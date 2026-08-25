// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { BesuIBFT2LightClient } from "../../contracts/light-clients/besu/BesuIBFT2LightClient.sol";
import { BesuQBFTLightClient } from "../../contracts/light-clients/besu/BesuQBFTLightClient.sol";
import { IBesuLightClient } from "../../contracts/light-clients/besu/interfaces/IBesuLightClient.sol";
import { BesuLightClientFixtureTestBase } from "./BesuLightClientFixtureTestBase.sol";

contract BesuIBFT2LightClientTest is BesuLightClientFixtureTestBase {
    function _fixtureFile() internal pure override returns (string memory) {
        return "ibft2.json";
    }

    function _deployPrimaryClient() internal override returns (IBesuLightClient) {
        return IBesuLightClient(
            address(
                new BesuIBFT2LightClient(
                    fixture.routerAddress,
                    fixture.initialTrustedHeight,
                    fixture.initialTrustedTimestamp,
                    fixture.initialTrustedStorageRoot,
                    fixture.initialTrustedValidators,
                    fixture.trustingPeriod,
                    fixture.maxClockDrift,
                    address(0)
                )
            )
        );
    }

    function _deployWrongWrapper() internal override returns (IBesuLightClient) {
        return IBesuLightClient(
            address(
                new BesuQBFTLightClient(
                    fixture.routerAddress,
                    fixture.initialTrustedHeight,
                    fixture.initialTrustedTimestamp,
                    fixture.initialTrustedStorageRoot,
                    fixture.initialTrustedValidators,
                    fixture.trustingPeriod,
                    fixture.maxClockDrift,
                    address(0)
                )
            )
        );
    }
}
