// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import { Script } from "forge-std/Script.sol";
import { ERC1967Proxy } from "@openzeppelin-contracts/proxy/ERC1967/ERC1967Proxy.sol";

import { BesuQBFTZKLightClient } from "../src/BesuQBFTZKLightClient.sol";
import { ProfileJson } from "./ProfileJson.sol";

/// @notice Deploys one light client from a deployment profile: an implementation, and an
/// ERC1967Proxy initialized in its own constructor.
contract DeployLightClient is Script {
    error ProfileValueDoesNotFitUint64(string field, uint256 value);
    /// @notice An upgrade authority without code would make every upgrade revert.
    error NoCodeAtUpgradeAuthority(address authority);
    /// @notice The anchor is already outside the trusting period on the destination chain, so the
    /// client's first `updateClient` would revert `TrustedStateExpired`. Pick a newer height.
    error BootstrapExpired(uint256 expiresAt, uint256 destinationTimestamp);

    function run(string calldata path)
        external
        returns (BesuQBFTZKLightClient lightClient, address implementation)
    {
        return deploy(ProfileJson.load(path));
    }

    function deploy(string memory json)
        public
        returns (BesuQBFTZKLightClient lightClient, address implementation)
    {
        ProfileJson.requireChainId(json);
        BesuQBFTZKLightClient.InitParams memory p = parseParams(json);
        address authority = ProfileJson.readAddress(json, ".upgradeAuthority");

        uint256 deployerKey = vm.envUint("DEPLOYER_PRIVATE_KEY");
        ProfileJson.requireDeployer(json, deployerKey);

        // Mirrors the expiry test in `updateClient`, where a zero trusting period never expires.
        uint256 expiresAt = uint256(p.initialBlockTimestamp) + uint256(p.trustingPeriodSeconds);
        // Bootstrap expiry is defined relative to the destination chain's timestamp.
        // forge-lint: disable-next-line(block-timestamp)
        if (p.trustingPeriodSeconds != 0 && expiresAt <= block.timestamp) {
            revert BootstrapExpired(expiresAt, block.timestamp);
        }

        if (authority.code.length == 0) revert NoCodeAtUpgradeAuthority(authority);

        vm.startBroadcast(deployerKey);
        implementation = address(new BesuQBFTZKLightClient());
        lightClient = BesuQBFTZKLightClient(
            address(
                new ERC1967Proxy(
                    implementation, abi.encodeCall(BesuQBFTZKLightClient.initialize, (authority, p))
                )
            )
        );
        vm.stopBroadcast();
    }

    function parseParams(string memory json)
        public
        view
        returns (BesuQBFTZKLightClient.InitParams memory p)
    {
        p.localIbcRouter = ProfileJson.readAddress(json, ".localIbcRouter");
        p.sourceIbcRouter = ProfileJson.readAddress(json, ".source.ibcRouter");
        p.sp1Verifier = ProfileJson.readAddress(json, ".sp1VerifierGateway");
        p.updateClientVKey = ProfileJson.readBytes32(json, ".vkeys.updateClient");
        p.membershipVKey = ProfileJson.readBytes32(json, ".vkeys.membership");
        p.nonMembershipVKey = ProfileJson.readBytes32(json, ".vkeys.nonMembership");
        p.initialHeight = _readUint64(json, ".bootstrap.height");
        p.initialStateRoot = ProfileJson.readBytes32(json, ".bootstrap.stateRoot");
        p.initialBlockTimestamp = _readUint64(json, ".bootstrap.blockTimestamp");
        p.initialValidatorSetHash = ProfileJson.readBytes32(json, ".bootstrap.validatorSetHash");
        p.trustingPeriodSeconds = _readUint64(json, ".trustingPeriodSeconds");
        p.maxClockDriftSeconds = _readUint64(json, ".maxClockDriftSeconds");
    }

    function _readUint64(string memory json, string memory key) private view returns (uint64) {
        uint256 raw = ProfileJson.readUint(json, key);
        if (raw > type(uint64).max) revert ProfileValueDoesNotFitUint64(key, raw);
        // The range check above rejects all values that would be truncated.
        // forge-lint: disable-next-line(unsafe-typecast)
        return uint64(raw);
    }
}
