// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { ILightClient } from "../../../interfaces/ILightClient.sol";

/// @title IAttestationLightClient
/// @notice Interface for an attestation-based IBC light client as specified in `IBC_ATTESTATION_DESIGN.md`.
/// @dev Implementations MUST also implement `ILightClient` behaviors.
interface IAttestationLightClient is ILightClient {
    /// @notice Returns the attestation set configuration.
    /// @dev The attestation set is fixed for the initial scope; rotation is out of scope.
    /// @return attestorAddresses The configured attestor EOA addresses
    /// @return minRequiredSigs The minimum number of unique valid signatures required
    function getAttestationSet() external view returns (address[] memory attestorAddresses, uint8 minRequiredSigs);

    /// @notice Returns the trusted consensus timestamp (in seconds) at the given revision height.
    /// @param revisionHeight The height for which to query the timestamp
    /// @return timestampSeconds The trusted timestamp in unix seconds
    function getConsensusTimestamp(uint64 revisionHeight) external view returns (uint64 timestampSeconds);
}
