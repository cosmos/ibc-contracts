// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import { Script } from "forge-std/Script.sol";
import { AccessManager } from "@openzeppelin-contracts/access/manager/AccessManager.sol";
import { ERC1967Proxy } from "@openzeppelin-contracts/proxy/ERC1967/ERC1967Proxy.sol";
import { ICS26Router } from "@ibc-contracts/ICS26Router.sol";
import { IBCRolesLib } from "@ibc-contracts/utils/IBCRolesLib.sol";
import { SP1VerifierGateway } from "@sp1-contracts/SP1VerifierGateway.sol";
import { SP1Verifier as SP1VerifierGroth16 } from "@sp1-contracts/v6.1.0/SP1VerifierGroth16.sol";

import { TestIBCApp } from "e2e/contracts/TestIBCApp.sol";

contract DeployE2ECore is Script {
    function run()
        external
        returns (
            ICS26Router router,
            SP1VerifierGateway gateway,
            SP1VerifierGroth16 verifier,
            TestIBCApp app
        )
    {
        // Foundry 1.2.3 initializes standalone script simulation at timestamp zero even when an
        // RPC URL is supplied. AccessManager reserves zero as the "role absent" sentinel, so its
        // constructor grant is not observable until simulation time is nonzero. Broadcast
        // transactions still execute with the destination chain's real block timestamps.
        if (block.timestamp == 0) vm.warp(1);

        uint256 deployerKey = vm.envUint("DEPLOYER_PRIVATE_KEY");
        address deployer = vm.addr(deployerKey);
        vm.startBroadcast(deployerKey);

        AccessManager manager = new AccessManager(deployer);
        ICS26Router routerLogic = new ICS26Router();
        router = ICS26Router(
            address(
                new ERC1967Proxy(
                    address(routerLogic), abi.encodeCall(ICS26Router.initialize, (address(manager)))
                )
            )
        );
        manager.setTargetFunctionRole(
            address(router), IBCRolesLib.ics26RelayerSelectors(), IBCRolesLib.PUBLIC_ROLE
        );

        verifier = new SP1VerifierGroth16();
        gateway = new SP1VerifierGateway(deployer);
        gateway.addRoute(address(verifier));

        app = new TestIBCApp(address(router));
        router.addIBCApp(address(app));
        vm.stopBroadcast();

        string memory output = "e2e-core";
        vm.serializeAddress(output, "accessManager", address(manager));
        vm.serializeAddress(output, "groth16Verifier", address(verifier));
        vm.serializeAddress(output, "router", address(router));
        vm.serializeAddress(output, "testApp", address(app));
        string memory json = vm.serializeAddress(output, "verifierGateway", address(gateway));
        vm.writeJson(json, vm.envString("E2E_OUTPUT_PATH"));
    }
}
