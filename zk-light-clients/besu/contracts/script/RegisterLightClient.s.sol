// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import { Script } from "forge-std/Script.sol";
import { IICS02Client } from "@ibc-contracts/interfaces/IICS02Client.sol";
import { IICS02ClientMsgs } from "@ibc-contracts/msgs/IICS02ClientMsgs.sol";

import { BesuQBFTZKLightClient } from "../src/BesuQBFTZKLightClient.sol";
import { BesuZKTypes } from "../src/BesuZKTypes.sol";
import { ProfileJson } from "./ProfileJson.sol";

/// @notice Registers a light client deployed from a profile with the profile's local Router.
/// @dev `forge script script/RegisterLightClient.s.sol --sig 'run(string,address)'
/// ../config/<profile>.json <light client>`, after `DeployLightClient`. The address is that run's
/// output, not a profile field. `DEPLOYER_PRIVATE_KEY` must belong to the profile's `deployer`.
///
/// The registrant cannot change the counterparty client ID once the Router stores it; only the
/// Router's AccessManager can, through `migrateClient`. Each client of a pair names the other's ID,
/// so both IDs are fixed in the profiles in advance. IDs of the form `client-<n>` use the Router's
/// sequential registration and must be next in sequence. Other IDs use the Router's role-gated
/// custom-ID registration overload.
contract RegisterLightClient is Script {
    error NoCodeAtLightClient(address lightClient);
    /// @notice The light client at the given address was not deployed from this profile.
    error LightClientDoesNotMatchProfile(string field);
    /// @notice The Router would assign another ID than the profile expects.
    error ClientIdUnavailable(string profileClientId, string nextClientId);

    function run(string calldata path, address lightClient) external returns (string memory) {
        return register(ProfileJson.load(path), lightClient);
    }

    function register(string memory json, address lightClient)
        public
        returns (string memory clientId)
    {
        ProfileJson.requireChainId(json);
        IICS02Client router = IICS02Client(ProfileJson.readAddress(json, ".localIbcRouter"));
        string memory expectedId = ProfileJson.readString(json, ".registration.clientId");
        string memory counterpartyClientId =
            ProfileJson.readString(json, ".registration.counterpartyClientId");

        uint256 deployerKey = vm.envUint("DEPLOYER_PRIVATE_KEY");
        ProfileJson.requireDeployer(json, deployerKey);

        _requireFromProfile(json, lightClient, address(router));

        bool sequential = _isSequentialId(expectedId);
        string memory nextId = string.concat("client-", vm.toString(router.getNextClientSeq()));
        if (sequential && keccak256(bytes(nextId)) != keccak256(bytes(expectedId))) {
            revert ClientIdUnavailable(expectedId, nextId);
        }

        // The pinned Router appends the terminal path to the last prefix element. A single empty
        // element preserves the v1 one-element raw terminal path the light client expects.
        bytes[] memory merklePrefix = new bytes[](1);
        merklePrefix[0] = bytes("");

        vm.startBroadcast(deployerKey);
        IICS02ClientMsgs.CounterpartyInfo memory counterparty = IICS02ClientMsgs.CounterpartyInfo({
            clientId: counterpartyClientId, merklePrefix: merklePrefix
        });
        if (sequential) {
            clientId = router.addClient(counterparty, lightClient);
        } else {
            clientId = router.addClient(expectedId, counterparty, lightClient);
        }
        vm.stopBroadcast();
    }

    function _isSequentialId(string memory clientId) private pure returns (bool) {
        bytes memory value = bytes(clientId);
        bytes memory prefix = bytes("client-");
        if (value.length <= prefix.length) return false;
        for (uint256 i = 0; i < prefix.length; ++i) {
            if (value[i] != prefix[i]) return false;
        }
        for (uint256 i = prefix.length; i < value.length; ++i) {
            if (value[i] < 0x30 || value[i] > 0x39) return false;
        }
        return true;
    }

    /// @dev The initialized values: a client built for another Router, source, program set,
    /// upgrade authority, anchor or client parameters is refused before anything is signed. A bare
    /// implementation fails the first check, since its initializers are disabled.
    function _requireFromProfile(string memory json, address lightClient, address router)
        private
        view
    {
        if (lightClient.code.length == 0) revert NoCodeAtLightClient(lightClient);
        BesuQBFTZKLightClient client = BesuQBFTZKLightClient(lightClient);
        if (client.LOCAL_IBC_ROUTER() != router) {
            revert LightClientDoesNotMatchProfile(".localIbcRouter");
        }
        if (client.SOURCE_IBC_ROUTER() != ProfileJson.readAddress(json, ".source.ibcRouter")) {
            revert LightClientDoesNotMatchProfile(".source.ibcRouter");
        }
        if (client.SP1_VERIFIER() != ProfileJson.readAddress(json, ".sp1VerifierGateway")) {
            revert LightClientDoesNotMatchProfile(".sp1VerifierGateway");
        }
        if (client.UPDATE_CLIENT_VKEY() != ProfileJson.readBytes32(json, ".vkeys.updateClient")) {
            revert LightClientDoesNotMatchProfile(".vkeys.updateClient");
        }
        if (client.MEMBERSHIP_VKEY() != ProfileJson.readBytes32(json, ".vkeys.membership")) {
            revert LightClientDoesNotMatchProfile(".vkeys.membership");
        }
        if (client.NON_MEMBERSHIP_VKEY() != ProfileJson.readBytes32(json, ".vkeys.nonMembership")) {
            revert LightClientDoesNotMatchProfile(".vkeys.nonMembership");
        }
        if (client.authority() != ProfileJson.readAddress(json, ".upgradeAuthority")) {
            revert LightClientDoesNotMatchProfile(".upgradeAuthority");
        }
        if (client.trustingPeriodSeconds() != ProfileJson.readUint(json, ".trustingPeriodSeconds"))
        {
            revert LightClientDoesNotMatchProfile(".trustingPeriodSeconds");
        }
        if (client.maxClockDriftSeconds() != ProfileJson.readUint(json, ".maxClockDriftSeconds")) {
            revert LightClientDoesNotMatchProfile(".maxClockDriftSeconds");
        }
        _requireAnchorFromProfile(json, client);
    }

    /// @dev The anchor's consensus state, which no update rewrites.
    function _requireAnchorFromProfile(string memory json, BesuQBFTZKLightClient client)
        private
        view
    {
        uint256 height = ProfileJson.readUint(json, ".bootstrap.height");
        if (height > type(uint64).max) revert LightClientDoesNotMatchProfile(".bootstrap.height");
        // The range check above ensures the anchor height fits uint64.
        (bool found, BesuZKTypes.ConsensusState memory anchor) =
            // forge-lint: disable-next-line(unsafe-typecast)
            client.getConsensusState(uint64(height));
        if (!found) revert LightClientDoesNotMatchProfile(".bootstrap.height");
        if (anchor.stateRoot != ProfileJson.readBytes32(json, ".bootstrap.stateRoot")) {
            revert LightClientDoesNotMatchProfile(".bootstrap.stateRoot");
        }
        if (anchor.blockTimestamp != ProfileJson.readUint(json, ".bootstrap.blockTimestamp")) {
            revert LightClientDoesNotMatchProfile(".bootstrap.blockTimestamp");
        }
        if (anchor.validatorSetHash != ProfileJson.readBytes32(json, ".bootstrap.validatorSetHash"))
        {
            revert LightClientDoesNotMatchProfile(".bootstrap.validatorSetHash");
        }
    }
}
