// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import { Vm } from "forge-std/Vm.sol";

/// @notice Reads deployment inputs from a deployment profile, `config/<name>.json`, by JSON path.
/// @dev Nothing is defaulted: a field that is absent or `null` reverts `ProfileFieldMissing` with
/// its path, before anything is signed.
/// A profile keeps `null` wherever a value has not been supplied yet, and Forge's typed JSON parsers
/// would otherwise fail on it with a message that names no field. `docs/DEPLOYMENT.md#profile-fields` documents the
/// fields.
library ProfileJson {
    /// @dev The cheatcode address, as forge-std's `stdJson` binds it; a library has no `vm` of its own.
    Vm private constant VM = Vm(address(uint160(uint256(keccak256("hevm cheat code")))));

    /// @notice A profile field the deployment needs is absent or `null`. Named by its JSON path.
    error ProfileFieldMissing(string field);
    /// @notice The node `--rpc-url` points at is not the profile's destination chain. Raised before
    /// anything is signed.
    error ChainIdMismatch(uint256 profileChainId, uint256 liveChainId);
    /// @notice The signing key does not belong to the profile's `deployer`.
    error DeployerMismatch(address profileDeployer, address keyAddress);

    /// @notice The profile's JSON text. `foundry.toml` grants reads on `../config`, `../artifacts`
    /// `../deployments` and `../.e2e-state`; Forge refuses other paths before parsing.
    function load(string memory path) internal view returns (string memory json) {
        // forge-lint: disable-next-line(unsafe-cheatcode)
        json = VM.readFile(path);
    }

    /// @notice Refuses to go on unless the connected chain is the profile's `chainId`.
    function requireChainId(string memory json) internal view {
        uint256 expected = readUint(json, ".chainId");
        if (block.chainid != expected) revert ChainIdMismatch(expected, block.chainid);
    }

    /// @notice Refuses to go on unless `key` belongs to the profile's `deployer`.
    function requireDeployer(string memory json, uint256 key) internal view {
        address deployer = readAddress(json, ".deployer");
        address keyAddress = VM.addr(key);
        if (keyAddress != deployer) revert DeployerMismatch(deployer, keyAddress);
    }

    function readAddress(string memory json, string memory key) internal view returns (address) {
        requirePresent(json, key);
        return VM.parseJsonAddress(json, key);
    }

    function readUint(string memory json, string memory key) internal view returns (uint256) {
        requirePresent(json, key);
        return VM.parseJsonUint(json, key);
    }

    /// @dev An empty string reads as not supplied: no profile string is meaningful empty.
    function readString(string memory json, string memory key)
        internal
        view
        returns (string memory value)
    {
        requirePresent(json, key);
        value = VM.parseJsonString(json, key);
        if (bytes(value).length == 0) revert ProfileFieldMissing(key);
    }

    function readBytes32(string memory json, string memory key) internal view returns (bytes32) {
        requirePresent(json, key);
        return VM.parseJsonBytes32(json, key);
    }

    /// @dev The one check every typed reader shares: the key is present and not `null`. Forge
    /// renders a scalar's JSON text when asked for it as a string, so `null` reads back literally.
    function requirePresent(string memory json, string memory key) internal view {
        if (!VM.keyExistsJson(json, key)) revert ProfileFieldMissing(key);
        if (keccak256(bytes(VM.parseJsonString(json, key))) == keccak256("null")) {
            revert ProfileFieldMissing(key);
        }
    }
}
