// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { BesuIBFT2LightClient } from "../../contracts/light-clients/besu/BesuIBFT2LightClient.sol";
import { IBesuLightClient } from "../../contracts/light-clients/besu/interfaces/IBesuLightClient.sol";
import { BesuLightClientFixtureTestBase, IBesuCommitSealDigestHarness } from "./BesuLightClientFixtureTestBase.sol";

contract BesuIBFT2CommitSealDigestHarness is BesuIBFT2LightClient, IBesuCommitSealDigestHarness {
    constructor(address[] memory validators)
        BesuIBFT2LightClient(address(1), 1, 1, bytes32(0), validators, 1, 0, address(0))
    { }

    function commitSealDigest(bytes calldata headerRlp) external pure returns (bytes32) {
        return _commitSealDigest(_parseHeader(headerRlp));
    }
}

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

    function _deployDigestHarness() internal override returns (IBesuCommitSealDigestHarness) {
        return new BesuIBFT2CommitSealDigestHarness(fixture.initialTrustedValidators);
    }
}
