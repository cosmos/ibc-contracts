// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import { Script } from "forge-std/Script.sol";
import { IAccessManaged } from "@openzeppelin-contracts/access/manager/IAccessManaged.sol";
import { ERC1967Proxy } from "@openzeppelin-contracts/proxy/ERC1967/ERC1967Proxy.sol";

import { ICS26Router } from "@ibc-contracts/ICS26Router.sol";
import { IICS02ClientMsgs } from "@ibc-contracts/msgs/IICS02ClientMsgs.sol";

import { BesuQBFTZKLightClient } from "../../../contracts/src/BesuQBFTZKLightClient.sol";

/// @notice Deploys one directional light client and registers it through the pinned Router API.
contract DeployE2ELightClient is Script {
    error EnvironmentValueDoesNotFitUint64(string name, uint256 value);

    function run() external returns (BesuQBFTZKLightClient client, string memory clientId) {
        uint256 deployerKey = vm.envUint("DEPLOYER_PRIVATE_KEY");
        ICS26Router router = ICS26Router(vm.envAddress("LOCAL_IBC_ROUTER"));

        BesuQBFTZKLightClient.InitParams memory p = _initParams(address(router));
        // Upgrades are authorized by the AccessManager that governs the Router.
        address authority = IAccessManaged(address(router)).authority();

        vm.startBroadcast(deployerKey);
        client = BesuQBFTZKLightClient(
            address(
                new ERC1967Proxy(
                    address(new BesuQBFTZKLightClient()),
                    abi.encodeCall(BesuQBFTZKLightClient.initialize, (authority, p))
                )
            )
        );

        // The pinned Router requires a non-empty prefix and appends the canonical path to its
        // final element. One empty prefix element therefore produces the v1 one-element path.
        bytes[] memory merklePrefix = new bytes[](1);
        merklePrefix[0] = bytes("");
        clientId = router.addClient(
            IICS02ClientMsgs.CounterpartyInfo({
                clientId: vm.envString("COUNTERPARTY_CLIENT_ID"), merklePrefix: merklePrefix
            }),
            address(client)
        );
        vm.stopBroadcast();

        string memory object = "e2e-light-client";
        vm.serializeAddress(object, "lightClient", address(client));
        string memory json = vm.serializeString(object, "clientId", clientId);
        vm.writeJson(json, vm.envString("E2E_OUTPUT_PATH"));
    }

    function _initParams(address router)
        private
        view
        returns (BesuQBFTZKLightClient.InitParams memory p)
    {
        p.localIbcRouter = router;
        p.sourceIbcRouter = vm.envAddress("SOURCE_IBC_ROUTER");
        p.sp1Verifier = vm.envAddress("SP1_VERIFIER_GATEWAY");
        p.updateClientVKey = vm.envBytes32("UPDATE_CLIENT_VKEY");
        p.membershipVKey = vm.envBytes32("MEMBERSHIP_VKEY");
        p.nonMembershipVKey = vm.envBytes32("NON_MEMBERSHIP_VKEY");
        p.initialHeight = _envUint64("INITIAL_HEIGHT");
        p.initialStateRoot = vm.envBytes32("INITIAL_STATE_ROOT");
        p.initialBlockTimestamp = _envUint64("INITIAL_BLOCK_TIMESTAMP");
        p.initialValidatorSetHash = vm.envBytes32("INITIAL_VALIDATOR_SET_HASH");
        p.trustingPeriodSeconds = _envUint64("TRUSTING_PERIOD_SECONDS");
        p.maxClockDriftSeconds = _envUint64("MAX_CLOCK_DRIFT_SECONDS");
    }

    function _envUint64(string memory name) private view returns (uint64 value) {
        uint256 raw = vm.envUint(name);
        if (raw > type(uint64).max) revert EnvironmentValueDoesNotFitUint64(name, raw);
        value = uint64(raw);
    }
}
