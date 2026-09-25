// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable gas-strict-inequalities

import { Test } from "forge-std/Test.sol";
import { ILightClient } from "../../contracts/interfaces/ILightClient.sol";
import { ILightClientMsgs } from "../../contracts/msgs/ILightClientMsgs.sol";
import { IBesuLightClient } from "../../contracts/light-clients/besu/interfaces/IBesuLightClient.sol";
import { ICS24Host } from "../../contracts/utils/ICS24Host.sol";
import { QBFTSimSuite } from "../besu-bft/utils/QBFTSimSuite.sol";
import { SimHeader } from "../besu-bft/utils/SimHeader.sol";

/// @dev Gas of the QBFT light client against simulated chains of increasing validator count and store size.
contract BesuQBFTSimBenchmark is Test {
    string internal constant SNAPSHOT_GROUP = "BesuQBFTSim";

    function testBenchmark_UpdateClient_ValidatorCount() public {
        uint256[6] memory counts = [uint256(4), 7, 16, 32, 64, 100];
        for (uint256 i = 0; i < counts.length; ++i) {
            QBFTSimSuite sim = new QBFTSimSuite(SimHeader.Mode.QBFT);
            sim.addValidators(counts[i]);
            sim.produceBlocks(2);
            IBesuLightClient client = sim.deployLightClient(1 days, 10);
            sim.produceBlock();
            bytes memory update = sim.updateMsg(2, 3);
            string memory name = string.concat("update.validators_", vm.toString(counts[i]));

            client.updateClient(update);
            vm.snapshotGasLastFrame(SNAPSHOT_GROUP, string.concat(name, ".gas"));
            vm.snapshotValue(
                SNAPSHOT_GROUP,
                string.concat(name, ".calldata"),
                abi.encodeCall(ILightClient.updateClient, (update)).length
            );
        }
    }

    /// @dev Scales the world state trie (accounts, router included) and the router's storage trie (commitments).
    function testBenchmark_VerifyMembership_TrieSizes() public {
        uint256[4] memory sizes = [uint256(1), 16, 128, 1024];
        for (uint256 a = 0; a < sizes.length; ++a) {
            for (uint256 c = 0; c < sizes.length; ++c) {
                QBFTSimSuite sim = new QBFTSimSuite(SimHeader.Mode.QBFT);
                sim.addValidators(4);
                sim.produceBlocks(2);
                IBesuLightClient client = sim.deployLightClient(1 days, 10);
                sim.addAccounts(sizes[a] - 1);
                for (uint64 seq = 1; seq <= sizes[c]; ++seq) {
                    sim.setCommitment(
                        ICS24Host.packetCommitmentPathCalldata("client-0", seq), keccak256(abi.encode(seq))
                    );
                }
                sim.produceBlock();
                client.updateClient(sim.updateMsg(2, 3));
                ILightClientMsgs.MsgVerifyMembership memory message =
                    sim.membershipMsg(3, ICS24Host.packetCommitmentPathCalldata("client-0", 1));
                string memory name = string.concat(
                    "verify_membership.accounts_", vm.toString(sizes[a]), ".commitments_", vm.toString(sizes[c])
                );

                client.verifyMembership(message);
                vm.snapshotGasLastFrame(SNAPSHOT_GROUP, string.concat(name, ".gas"));
                vm.snapshotValue(
                    SNAPSHOT_GROUP,
                    string.concat(name, ".calldata"),
                    abi.encodeCall(ILightClient.verifyMembership, (message)).length
                );
            }
        }
    }
}
