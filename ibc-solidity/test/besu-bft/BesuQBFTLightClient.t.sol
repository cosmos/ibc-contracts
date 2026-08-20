// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

import { BesuIBFT2LightClient } from "../../contracts/light-clients/besu/BesuIBFT2LightClient.sol";
import { BesuQBFTLightClient } from "../../contracts/light-clients/besu/BesuQBFTLightClient.sol";
import { BesuLightClientTestBase, IBesuTestLightClient } from "./BesuLightClientTestBase.sol";

contract BesuQBFTLightClientTest is BesuLightClientTestBase {
    function _fixtureFile() internal pure override returns (string memory) {
        return "qbft.json";
    }

    function _deployPrimaryClient() internal override returns (IBesuTestLightClient) {
        return IBesuTestLightClient(
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

    function _deployWrongWrapper() internal override returns (IBesuTestLightClient) {
        return IBesuTestLightClient(
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
}
