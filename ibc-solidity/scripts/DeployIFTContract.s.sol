// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable custom-errors,gas-custom-errors

import { stdJson } from "forge-std/StdJson.sol";
import { Script } from "forge-std/Script.sol";
import { Strings } from "@openzeppelin-contracts/utils/Strings.sol";
import { ERC1967Proxy } from "@openzeppelin-contracts/proxy/ERC1967/ERC1967Proxy.sol";

import { IFTOwnable } from "../contracts/utils/IFTOwnable.sol";
import { IIFTMsgs } from "../contracts/msgs/IIFTMsgs.sol";

/// @notice Deploys a new IFTOwnable proxy for E2E testing.
/// @dev Required env vars: ICS27_GMP_ADDRESS, IFT_TOKEN_NAME, IFT_TOKEN_SYMBOL
contract DeployIFTContract is Script {
    using stdJson for string;

    /// @dev Rate limit capacity per direction, generous so that e2e flows never hit it
    uint208 internal constant IFT_RATE_LIMIT_CAPACITY = type(uint128).max;
    /// @dev Rate limit refill window
    uint48 internal constant IFT_RATE_LIMIT_WINDOW = 1 days;

    function run() public returns (string memory) {
        address ics27Gmp = vm.envAddress("ICS27_GMP_ADDRESS");
        string memory tokenName = vm.envString("IFT_TOKEN_NAME");
        string memory tokenSymbol = vm.envString("IFT_TOKEN_SYMBOL");

        vm.startBroadcast();

        address iftLogic = address(new IFTOwnable());
        address deployed = address(
            new ERC1967Proxy(
                iftLogic, abi.encodeCall(IFTOwnable.initialize, (msg.sender, tokenName, tokenSymbol, ics27Gmp))
            )
        );
        IFTOwnable(deployed)
            .setIFTRateLimit(IIFTMsgs.IFTRateLimitDirection.Inbound, IFT_RATE_LIMIT_CAPACITY, IFT_RATE_LIMIT_WINDOW);
        IFTOwnable(deployed)
            .setIFTRateLimit(IIFTMsgs.IFTRateLimitDirection.Outbound, IFT_RATE_LIMIT_CAPACITY, IFT_RATE_LIMIT_WINDOW);

        vm.stopBroadcast();

        string memory json = "json";
        json = json.serialize("ift", Strings.toHexString(deployed));

        return json;
    }
}
