// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import { Test } from "forge-std/Test.sol";
import { AccessManager } from "@openzeppelin-contracts/access/manager/AccessManager.sol";
import { DeployLightClient } from "../script/DeployLightClient.s.sol";
import { BesuQBFTZKLightClient } from "../src/BesuQBFTZKLightClient.sol";

contract DeployLightClientStandaloneTest is Test {
    function test_deploysWithoutRouterCodeOrRegistration() public {
        vm.warp(1_001);
        uint256 key = 0xA11CE;
        vm.setEnv("DEPLOYER_PRIVATE_KEY", vm.toString(key));
        address manager = address(new AccessManager(address(this)));
        address localRouter = address(0x1111);
        address sourceRouter = address(0x2222);
        assertEq(localRouter.code.length, 0);
        assertEq(sourceRouter.code.length, 0);
        // No registration object: only the light-client deployment inputs are required.
        string memory json = string.concat(
            '{"chainId":',
            vm.toString(block.chainid),
            ',"deployer":"',
            vm.toString(vm.addr(key)),
            '","upgradeAuthority":"',
            vm.toString(manager)
        );
        json = string.concat(
            json,
            '","localIbcRouter":"',
            vm.toString(localRouter),
            '","source":{"ibcRouter":"',
            vm.toString(sourceRouter),
            '"},"sp1VerifierGateway":"0x0000000000000000000000000000000000003333",'
        );
        json = string.concat(
            json,
            '"vkeys":{"updateClient":"',
            vm.toString(bytes32(uint256(1))),
            '","membership":"',
            vm.toString(bytes32(uint256(2))),
            '","nonMembership":"',
            vm.toString(bytes32(uint256(3)))
        );
        json = string.concat(
            json,
            '"},"bootstrap":{"height":7,"stateRoot":"',
            vm.toString(bytes32(uint256(4))),
            '","blockTimestamp":1000,"validatorSetHash":"',
            vm.toString(bytes32(uint256(5))),
            '"},"trustingPeriodSeconds":1000,"maxClockDriftSeconds":20}'
        );
        (BesuQBFTZKLightClient client, address implementation) =
            new DeployLightClient().deploy(json);
        assertGt(implementation.code.length, 0);
        assertEq(client.LOCAL_IBC_ROUTER(), localRouter);
        assertEq(client.SOURCE_IBC_ROUTER(), sourceRouter);
        assertEq(client.authority(), manager);
        assertEq(client.latestHeight(), 7);
        assertEq(localRouter.code.length, 0);
    }
}
