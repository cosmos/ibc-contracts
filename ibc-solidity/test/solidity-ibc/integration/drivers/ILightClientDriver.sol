// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { IICS02ClientMsgs } from "../../../../contracts/msgs/IICS02ClientMsgs.sol";

import { ILightClient } from "../../../../contracts/interfaces/ILightClient.sol";

/// @title Light Client Driver
/// @notice Test-only adapter that owns a light client tracking one counterparty router and makes the router's state
/// provable to it, so integration suites can run unchanged on any light client.
/// @dev Drivers only read `getCommitment` from the counterparty router, so they also work against a router of a
/// forked deployment.
interface ILightClientDriver {
    /// @notice The light client to register for the counterparty.
    function lightClient() external view returns (ILightClient);

    /// @notice Makes the counterparty's current value at the raw ICS24 `path` provable; zero proves non-membership.
    /// @return updateMsg The `updateClient` message to submit before the proof, or empty if none is needed
    /// @return proof The membership or non-membership proof
    /// @return proofHeight The height of the proof
    function prove(bytes calldata path)
        external
        returns (bytes memory updateMsg, bytes memory proof, IICS02ClientMsgs.Height memory proofHeight);
}
