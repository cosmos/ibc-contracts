// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

/// @title IICS02PrecompileWrapper
/// @notice IICS02PrecompileWrapper is the interface for the ICS02 Precompile Wrapper methods not covered by
/// ILightClient
interface IICS02PrecompileWrapper {
    /// @notice The client identifier of the IBC-Go Light Client
    /// @dev The client-id associated to this light client in solidity-ibc may be different.
    /// @return The IBC-Go client identifier
    function ibcGoClientId() external view returns (string memory);
}
