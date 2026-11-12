// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { IICS27GMPMsgs } from "../msgs/IICS27GMPMsgs.sol";
import { IICS27Account } from "../interfaces/IICS27Account.sol";
import { BeaconProxy } from "@openzeppelin-contracts/proxy/beacon/BeaconProxy.sol";

/// @title ICS27Lib
/// @notice This library provides utility functions for ICS27GMP, including versioning, encoding, acknowledgements and
/// account proxy bytecode.
library ICS27Lib {
    /// @notice ICS27_VERSION is the version string for ICS27 packet data.
    string internal constant ICS27_VERSION = "ics27-2";

    /// @notice ICS27_ENCODING is the encoding string for ICS27 packet data.
    string internal constant ICS27_ENCODING = "application/x-solidity-abi";

    /// @notice DEFAULT_PORT_ID is the default port id for ICS27.
    string internal constant DEFAULT_PORT_ID = "gmpport";

    /// @notice KECCAK256_ICS27_VERSION is the keccak256 hash of the ICS27_VERSION.
    bytes32 internal constant KECCAK256_ICS27_VERSION = keccak256(bytes(ICS27_VERSION));

    /// @notice KECCAK256_ICS27_ENCODING is the keccak256 hash of the ICS27_ENCODING.
    bytes32 internal constant KECCAK256_ICS27_ENCODING = keccak256(bytes(ICS27_ENCODING));

    /// @notice KECCAK256_DEFAULT_PORT_ID is the keccak256 hash of the DEFAULT_PORT_ID.
    bytes32 internal constant KECCAK256_DEFAULT_PORT_ID = keccak256(bytes(DEFAULT_PORT_ID));

    /// @notice Create a GMP acknowledgement from a call result.
    /// @param result The result of the call
    /// @return The GMP acknowledgement message
    function acknowledgement(bytes memory result) internal pure returns (bytes memory) {
        return abi.encode(IICS27GMPMsgs.GMPAcknowledgement({ result: result }));
    }

    /// @notice Retrieve the deployment bytecode of the BeaconProxy contract.
    /// @param beacon The address of the beacon contract.
    /// @param ics27 The address of the ICS27 contract.
    /// @return The deployment bytecode of the BeaconProxy contract.
    function getBeaconProxyBytecode(address beacon, address ics27) internal pure returns (bytes memory) {
        return abi.encodePacked(
            type(BeaconProxy).creationCode, abi.encode(beacon, abi.encodeCall(IICS27Account.initialize, (ics27)))
        );
    }

    /// @notice Compute the code hash of the BeaconProxy contract.
    /// @param beacon The address of the beacon contract.
    /// @param ics27 The address of the ICS27 contract.
    /// @return The code hash of the BeaconProxy contract.
    function getBeaconProxyCodeHash(address beacon, address ics27) internal pure returns (bytes32) {
        return keccak256(getBeaconProxyBytecode(beacon, ics27));
    }
}
