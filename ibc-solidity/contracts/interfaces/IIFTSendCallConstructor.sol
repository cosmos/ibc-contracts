// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { IERC165 } from "@openzeppelin-contracts/utils/introspection/IERC165.sol";

/// @title IFT Send Call Constructor Interface
/// @notice Interface for constructing ICS27-GMP call data for minting IFT tokens on counterparty chains
/// @dev Implementations may be stateless (pure functions) or stateful (e.g., storing denom for Cosmos SDK chains)
interface IIFTSendCallConstructor is IERC165 {
    /// @notice Constructs the ICS27-GMP call data for minting IFT tokens on the counterparty chain
    /// @dev The constructed call data should conform to the expected format of the counterparty IFT
    /// contract's mint function
    /// @param receiver The address of the receiver on the counterparty chain
    /// @param amount The amount of tokens to mint
    /// @return The constructed call data for the ICS27-GMP message
    function constructMintCall(string calldata receiver, uint256 amount) external view returns (bytes memory);
}
