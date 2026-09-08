// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

/// @title IMintableAndBurnable
/// @notice Interface for ERC20 tokens to be minted and burned by the ICS20 contract.
interface IMintableAndBurnable {
    /// @notice Mint new tokens to the Escrow contract
    /// @dev This function can only be called by an authorized contract (e.g., ICS20)
    /// @dev This function needs to allow minting tokens to the Escrow contract
    /// @param mintAddress Address to mint tokens to
    /// @param amount Amount of tokens to mint
    function mint(address mintAddress, uint256 amount) external;

    /// @notice Burn tokens from the Escrow contract
    /// @dev This function can only be called by an authorized contract (e.g., ICS20)
    /// @dev This function needs to allow burning of tokens from the Escrow contract
    /// @param burnAddress Address to burn tokens from
    /// @param amount Amount of tokens to burn
    function burn(address burnAddress, uint256 amount) external;
}
