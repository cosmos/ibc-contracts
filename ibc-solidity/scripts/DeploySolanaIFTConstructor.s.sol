// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable custom-errors,gas-custom-errors

import { stdJson } from "forge-std/StdJson.sol";
import { Script } from "forge-std/Script.sol";
import { Strings } from "@openzeppelin-contracts/utils/Strings.sol";

import { SolanaIFTSendCallConstructor } from "../contracts/utils/SolanaIFTSendCallConstructor.sol";

/// @notice Deploys SolanaIFTSendCallConstructor with 6 static Solana PDAs passed as arguments.
/// @dev Called separately from the main E2E deploy script because the PDAs depend on the Solana IFT mint
///      (created during the test) and the EVM IFT contract address (created by the main deploy script).
contract DeploySolanaIFTConstructor is Script {
    using stdJson for string;

    function run(
        bytes32 appState,
        bytes32 appMintState,
        bytes32 iftBridge,
        bytes32 mint,
        bytes32 mintAuthority,
        bytes32 gmpAccount
    )
        public
        returns (string memory)
    {
        vm.startBroadcast();

        address deployed = address(
            new SolanaIFTSendCallConstructor(appState, appMintState, iftBridge, mint, mintAuthority, gmpAccount)
        );

        vm.stopBroadcast();

        string memory json = "json";
        json = json.serialize("solanaIftConstructor", Strings.toHexString(deployed));

        return json;
    }
}
