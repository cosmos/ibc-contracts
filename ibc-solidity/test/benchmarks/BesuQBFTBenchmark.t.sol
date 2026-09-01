// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { ILightClientMsgs } from "../../contracts/msgs/ILightClientMsgs.sol";
import { ILightClient } from "../../contracts/interfaces/ILightClient.sol";
import { IBesuLightClient } from "../../contracts/light-clients/besu/interfaces/IBesuLightClient.sol";
import { BesuLightClientFixtureTestBase } from "../besu-bft/BesuLightClientFixtureTestBase.sol";

contract BesuQBFTBenchmark is BesuLightClientFixtureTestBase {
    string internal constant SNAPSHOT_GROUP = "BesuQBFT";

    function testBenchmark_UpdateClient_Adjacent() public {
        vm.warp(fixture.initialTrustedTimestamp + 1);
        bytes memory update = _encodeUpdate(fixture.adjacentUpdate);

        client.updateClient(update);
        vm.snapshotGasLastFrame(SNAPSHOT_GROUP, "update.adjacent.gas");
        vm.snapshotValue(
            SNAPSHOT_GROUP, "update.adjacent.calldata", abi.encodeCall(ILightClient.updateClient, (update)).length
        );
    }

    function testBenchmark_UpdateClient_NonAdjacent() public {
        vm.warp(fixture.initialTrustedTimestamp + 1);
        bytes memory update = _encodeUpdate(fixture.nonAdjacentUpdate);

        client.updateClient(update);
        vm.snapshotGasLastFrame(SNAPSHOT_GROUP, "update.non_adjacent.gas");
        vm.snapshotValue(
            SNAPSHOT_GROUP, "update.non_adjacent.calldata", abi.encodeCall(ILightClient.updateClient, (update)).length
        );
    }

    function testBenchmark_VerifyMembership() public {
        vm.warp(fixture.initialTrustedTimestamp + 1);
        client.updateClient(_encodeUpdate(fixture.nonAdjacentUpdate));
        ILightClientMsgs.MsgVerifyMembership memory message =
            _membershipMessage(0, _singlePath(fixture.membership.path), fixture.membership.value);

        uint256 timestamp = client.verifyMembership(message);
        vm.snapshotGasLastFrame(SNAPSHOT_GROUP, "verify_membership.gas");
        vm.snapshotValue(
            SNAPSHOT_GROUP,
            "verify_membership.calldata",
            abi.encodeCall(ILightClient.verifyMembership, (message)).length
        );

        assertEq(timestamp, fixture.membership.expectedTimestamp);
    }

    function _fixtureFile() internal pure override returns (string memory) {
        return "qbft.json";
    }

    function _deployPrimaryClient() internal override returns (IBesuLightClient) {
        return _deployQBFT();
    }

    function _deployWrongWrapper() internal override returns (IBesuLightClient) {
        return _deployIBFT2();
    }
}
