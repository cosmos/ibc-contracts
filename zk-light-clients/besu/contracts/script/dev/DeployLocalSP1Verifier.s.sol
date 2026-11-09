// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import { Script } from "forge-std/Script.sol";

import { SP1VerifierGateway } from "@sp1-contracts/SP1VerifierGateway.sol";
import { SP1Verifier } from "@sp1-contracts/v6.1.0/SP1VerifierGroth16.sol";

contract DeployLocalSP1Verifier is Script {
    function run() external returns (SP1VerifierGateway gateway, SP1Verifier verifier) {
        uint256 deployerKey = vm.envUint("DEPLOYER_PRIVATE_KEY");
        address deployer = vm.addr(deployerKey);
        vm.startBroadcast(deployerKey);
        verifier = new SP1Verifier();
        gateway = new SP1VerifierGateway(deployer);
        gateway.addRoute(address(verifier));
        vm.stopBroadcast();
    }
}
