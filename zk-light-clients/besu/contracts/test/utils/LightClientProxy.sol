// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import { ERC1967Proxy } from "@openzeppelin-contracts/proxy/ERC1967/ERC1967Proxy.sol";

import { BesuQBFTZKLightClient } from "../../src/BesuQBFTZKLightClient.sol";

/// @notice Deploys a light client the way DeployLightClient does: a fresh implementation behind an
/// ERC1967Proxy that is initialized in its constructor.
library LightClientProxy {
    function deploy(address authority, BesuQBFTZKLightClient.InitParams memory p)
        internal
        returns (BesuQBFTZKLightClient)
    {
        BesuQBFTZKLightClient implementation = new BesuQBFTZKLightClient();
        return BesuQBFTZKLightClient(
            address(
                new ERC1967Proxy(
                    address(implementation),
                    abi.encodeCall(BesuQBFTZKLightClient.initialize, (authority, p))
                )
            )
        );
    }
}
