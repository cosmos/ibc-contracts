// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { IBesuLightClientMsgs } from "../../contracts/light-clients/besu/msgs/IBesuLightClientMsgs.sol";
import { IBesuLightClient } from "../../contracts/light-clients/besu/interfaces/IBesuLightClient.sol";
import { BesuLightClientFixtureTestBase } from "./BesuLightClientFixtureTestBase.sol";

contract BesuQBFTLightClientTest is BesuLightClientFixtureTestBase {
    function _fixtureFile() internal pure override returns (string memory) {
        return "qbft.json";
    }

    function _deployPrimaryClient(
        IBesuLightClientMsgs.ClientState memory clientState,
        IBesuLightClientMsgs.ConsensusState memory consensusState
    )
        internal
        override
        returns (IBesuLightClient)
    {
        return _deployQBFT(clientState, consensusState);
    }

    function _deployWrongWrapper(
        IBesuLightClientMsgs.ClientState memory clientState,
        IBesuLightClientMsgs.ConsensusState memory consensusState
    )
        internal
        override
        returns (IBesuLightClient)
    {
        return _deployIBFT2(clientState, consensusState);
    }
}
