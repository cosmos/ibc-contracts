// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

import { BesuZKTypes } from "../BesuZKTypes.sol";

/// @notice Additional read-only surface used by proof construction and monitoring.
interface IBesuQBFTZKLightClient {
    function latestHeight() external view returns (uint64);

    function trustingPeriodSeconds() external view returns (uint64);

    function maxClockDriftSeconds() external view returns (uint64);

    function getConsensusState(uint64 height)
        external
        view
        returns (bool found, BesuZKTypes.ConsensusState memory state);

    function isMembershipBatchVerified(bytes32 batchId) external view returns (bool);
}
