// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { IBesuLightClientErrors } from "../errors/IBesuLightClientErrors.sol";

import { Memory } from "@openzeppelin-contracts/utils/Memory.sol";
import { SafeCast } from "@openzeppelin-contracts/utils/math/SafeCast.sol";
import { RLP } from "@openzeppelin-contracts/utils/RLP.sol";

/// @title Header
/// @notice Header is a library that provides types and utilities for working with Besu headers and their fields.
library Header {
    using RLP for *;

    /// @notice Besu BFT sentinel mix hash.
    bytes32 private constant BESU_BFT_MIX_HASH = 0x63746963616c2062797a616e74696e65206661756c7420746f6c6572616e6365;
    /// @notice Empty ommers hash required by Besu BFT headers.
    bytes32 private constant EMPTY_OMMERS_HASH = 0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347;

    /// @notice Decoded fields from a submitted Besu header.
    /// @param height Header block number.
    /// @param stateRoot Header state root.
    /// @param timestamp Header timestamp in seconds.
    /// @param validators Validator set from `extraData`.
    // solhint-disable-next-line gas-struct-packing
    struct Data {
        uint64 height;
        bytes32 stateRoot;
        uint64 timestamp;
        address[] validators;
    }

    /// @notice Decodes a Besu header RLP in its commit seal signing form into its fields.
    /// @dev The commit seals must already be removed from `extraData`: IBFT 2.0 signs 4 `extraData` items, and QBFT
    /// signs 5 items whose last one is an empty seal list.
    /// @param headerRlp RLP-encoded Besu header in its commit seal signing form.
    /// @return header Decoded Besu header fields.
    function decodeRlp(bytes memory headerRlp) internal pure returns (Data memory header) {
        Memory.Slice[] memory headerItems = headerRlp.decodeList();
        require(headerItems.length > 14, IBesuLightClientErrors.InvalidHeaderFormat(headerItems.length));

        require(
            headerItems[1].readBytes32() == EMPTY_OMMERS_HASH,
            IBesuLightClientErrors.InvalidOmmersHash(headerItems[1].readBytes32())
        );
        require(
            headerItems[7].readUint256() == 1, IBesuLightClientErrors.InvalidDifficulty(headerItems[7].readUint256())
        );
        require(
            headerItems[13].readBytes32() == BESU_BFT_MIX_HASH,
            IBesuLightClientErrors.InvalidMixHash(headerItems[13].readBytes32())
        );

        bytes memory nonce = headerItems[14].readBytes();
        require(
            nonce.length == 8 && keccak256(nonce) == keccak256(hex"0000000000000000"),
            IBesuLightClientErrors.InvalidNonce(nonce)
        );

        header.height = SafeCast.toUint64(headerItems[8].readUint256());
        header.stateRoot = headerItems[3].readBytes32();
        header.timestamp = SafeCast.toUint64(headerItems[11].readUint256());

        bytes memory extraData = headerItems[12].readBytes();
        Memory.Slice[] memory extraDataItems = extraData.decodeList();
        require(
            extraDataItems.length == 4 || extraDataItems.length == 5,
            IBesuLightClientErrors.InvalidExtraDataFormat(extraDataItems.length)
        );
        if (extraDataItems.length == 5) {
            // An empty list encodes as the bare list prefix.
            Memory.Slice sealList = extraDataItems[4];
            require(
                Memory.length(sealList) == 1 && uint8(bytes1(Memory.load(sealList, 0))) == RLP.LONG_OFFSET,
                IBesuLightClientErrors.NonEmptyCommitSealList()
            );
        }

        Memory.Slice[] memory validatorItems = extraDataItems[1].readList();
        require(validatorItems.length != 0, IBesuLightClientErrors.EmptyValidatorSet());

        header.validators = new address[](validatorItems.length);
        for (uint256 i = 0; i < validatorItems.length; ++i) {
            header.validators[i] = validatorItems[i].readAddress();
        }
    }
}
