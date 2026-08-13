// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { IIFTSendCallConstructor } from "../interfaces/IIFTSendCallConstructor.sol";
import { IIFT } from "../interfaces/IIFT.sol";

import { Strings } from "@openzeppelin-contracts/utils/Strings.sol";
import { ERC165 } from "@openzeppelin-contracts/utils/introspection/ERC165.sol";
import { IERC165 } from "@openzeppelin-contracts/utils/introspection/IERC165.sol";

/// @title EVM IFT Send Call Constructor
/// @notice Constructs ICS27-GMP call data for minting IFT tokens on EVM-based counterparty chains
contract EVMIFTSendCallConstructor is IIFTSendCallConstructor, ERC165 {
    /// @notice Error thrown when the receiver address is invalid
    /// @param receiver The invalid receiver string
    error EVMIFTInvalidReceiver(string receiver);

    /// @inheritdoc IIFTSendCallConstructor
    function constructMintCall(string calldata receiver, uint256 amount) external pure returns (bytes memory) {
        (bool success, address receiverAddr) = Strings.tryParseAddress(receiver);
        require(success, EVMIFTInvalidReceiver(receiver));

        return abi.encodeCall(IIFT.iftMint, (receiverAddr, amount));
    }

    /// @inheritdoc ERC165
    function supportsInterface(bytes4 interfaceId) public view virtual override(ERC165, IERC165) returns (bool) {
        return interfaceId == type(IIFTSendCallConstructor).interfaceId || super.supportsInterface(interfaceId);
    }
}
