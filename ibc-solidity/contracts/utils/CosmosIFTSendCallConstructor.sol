// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable gas-strict-inequalities,code-complexity

import { IIFTSendCallConstructor } from "../interfaces/IIFTSendCallConstructor.sol";

import { Strings } from "@openzeppelin-contracts/utils/Strings.sol";
import { ERC165 } from "@openzeppelin-contracts/utils/introspection/ERC165.sol";
import { IERC165 } from "@openzeppelin-contracts/utils/introspection/IERC165.sol";

/// @title Cosmos SDK IFT Send Call Constructor
/// @notice Constructs ICS27-GMP call data for minting IFT tokens on Cosmos SDK-based counterparty chains
/// @dev This implementation encodes mint calls in protojson format for Cosmos SDK IFT module
contract CosmosIFTSendCallConstructor is IIFTSendCallConstructor, ERC165 {
    /// @notice Error thrown when the receiver address is invalid
    /// @param receiver The invalid receiver string
    error CosmosIFTInvalidReceiver(string receiver);

    /// @notice The type URL for the MsgIFTMint message in the TokenFactory module
    // natlint-disable-next-line MissingInheritdoc
    string public bridgeReceiveTypeUrl;

    /// @notice The denomination of the counterparty token on the Cosmos SDK chain
    // natlint-disable-next-line MissingInheritdoc
    string public denom;

    /// @notice The interchain account address of the submitter account on the Cosmos SDK chain
    /// @dev Required to set the signer of the MsgIFTMint message (Cosmos SDK limitation)
    // natlint-disable-next-line MissingInheritdoc
    string public icaAddress;

    /// @notice Creates a new CosmosIFTSendCallConstructor
    /// @param bridgeReceiveTypeUrl_ The type URL for MsgIFTMint
    /// @param denom_ The denom of the token on the Cosmos SDK chain (e.g., "uatom", "ibc/...")
    /// @param icaAddress_ The interchain account address on the Cosmos SDK chain
    constructor(string memory bridgeReceiveTypeUrl_, string memory denom_, string memory icaAddress_) {
        bridgeReceiveTypeUrl = bridgeReceiveTypeUrl_;
        denom = denom_;
        icaAddress = icaAddress_;
    }

    /// @inheritdoc IIFTSendCallConstructor
    /// @dev Encodes the mint call as protojson CosmosTx for Cosmos SDK ICS27-GMP module
    function constructMintCall(string calldata receiver, uint256 amount) external view returns (bytes memory) {
        if (!_validateReceiver(receiver)) {
            revert CosmosIFTInvalidReceiver(receiver);
        }

        return abi.encodePacked(
            "{\"messages\":[{\"@type\":\"",
            bridgeReceiveTypeUrl,
            "\",\"signer\":\"",
            icaAddress,
            "\",\"denom\":\"",
            denom,
            "\",\"receiver\":\"",
            receiver,
            "\",\"amount\":\"",
            Strings.toString(amount),
            "\"}]}"
        );
    }

    /// @inheritdoc ERC165
    function supportsInterface(bytes4 interfaceId) public view virtual override(ERC165, IERC165) returns (bool) {
        return interfaceId == type(IIFTSendCallConstructor).interfaceId || super.supportsInterface(interfaceId);
    }

    /// @notice Validates the receiver string to ensure it is either a valid Ethereum address or a valid bech32 address
    /// with an appropriate HRP
    /// @param receiver The receiver string to validate
    /// @return True if the receiver is valid, false otherwise
    function _validateReceiver(string calldata receiver) internal pure returns (bool) {
        if (bytes(receiver).length == 0) {
            return false;
        }

        // We allow eth addresses as receivers due to cosmos/evm
        // NOTE: We ignore the return value of tryParseAddress because
        // we only want to check if it is a valid address format
        // slither-disable-next-line unused-return
        (bool isAddress,) = Strings.tryParseAddress(receiver);
        if (isAddress) {
            return true;
        }

        // We also allow bech32 addresses for cosmos chains
        // We will only allow HRPs which consist of [a-z0-9] and do not contain "1". If the HRP contains "1", then we
        // will reject the address. All known Cosmos SDK chains have HRPs that do not contain "1", so this is a
        // reasonable heuristic.
        // We only allow lowercase alphanumeric characters excluding "1", "b", "i", and "o" as per bech32 specification,
        // but we do not enforce the full bech32 checksum as it is not critical for our use case.
        bool isHrp = true;
        for (uint256 i = 0; i < bytes(receiver).length; ++i) {
            uint256 c = uint256(uint8(bytes(receiver)[i]));
            if (isHrp) {
                if (c == 0x31) {
                    // "1"
                    isHrp = false;
                    continue;
                }
                if ((c >= 0x61 && c <= 0x7A) || (c >= 0x30 && c <= 0x39)) {
                    // a-z, 0-9
                    continue;
                }
                return false;
            } else {
                if ((c == 0x31) || (c == 0x62) || (c == 0x69) || (c == 0x6F)) {
                    // "1", "b", "i", "o"
                    return false;
                }
                if ((c >= 0x61 && c <= 0x7A) || (c >= 0x30 && c <= 0x39)) {
                    // a-z, 0-9
                    continue;
                }
                return false;
            }
        }

        if (isHrp) {
            return false; // must contain "1" separator
        }

        return true;
    }
}
