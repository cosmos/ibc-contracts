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
    /// @param headerItems Top-level RLP header fields.
    /// @param extraDataItems Decoded Besu BFT `extraData` fields.
    /// @param height Header block number.
    /// @param stateRoot Header state root.
    /// @param timestamp Header timestamp in seconds.
    /// @param validators Validator set from `extraData`.
    /// @param commitSeals Commit seals from `extraData`.
    struct Data {
        Memory.Slice[] headerItems;
        Memory.Slice[] extraDataItems;
        uint64 height;
        bytes32 stateRoot;
        uint64 timestamp;
        address[] validators;
        bytes[] commitSeals;
    }

    function decodeRlp(bytes memory headerRlp) internal pure returns (Data memory header) {
        header.headerItems = headerRlp.decodeList();
        require(header.headerItems.length >= 15, IBesuLightClientErrors.InvalidHeaderFormat(header.headerItems.length));

        require(
            header.headerItems[1].readBytes32() == EMPTY_OMMERS_HASH,
            IBesuLightClientErrors.InvalidOmmersHash(header.headerItems[1].readBytes32())
        );
        require(
            header.headerItems[7].readUint256() == 1,
            IBesuLightClientErrors.InvalidDifficulty(header.headerItems[7].readUint256())
        );
        require(
            header.headerItems[13].readBytes32() == BESU_BFT_MIX_HASH,
            IBesuLightClientErrors.InvalidMixHash(header.headerItems[13].readBytes32())
        );

        bytes memory nonce = header.headerItems[14].readBytes();
        require(
            nonce.length == 8 && keccak256(nonce) == keccak256(hex"0000000000000000"),
            IBesuLightClientErrors.InvalidNonce(nonce)
        );

        header.height = SafeCast.toUint64(header.headerItems[8].readUint256());
        header.stateRoot = header.headerItems[3].readBytes32();
        header.timestamp = SafeCast.toUint64(header.headerItems[11].readUint256());

        bytes memory extraData = header.headerItems[12].readBytes();
        header.extraDataItems = extraData.decodeList();
        require(
            header.extraDataItems.length == 5,
            IBesuLightClientErrors.InvalidExtraDataFormat(header.extraDataItems.length)
        );

        Memory.Slice[] memory validatorItems = header.extraDataItems[1].readList();
        require(validatorItems.length != 0, IBesuLightClientErrors.EmptyValidatorSet());

        header.validators = new address[](validatorItems.length);
        for (uint256 i = 0; i < validatorItems.length; ++i) {
            header.validators[i] = validatorItems[i].readAddress();
        }

        Memory.Slice[] memory sealItems = header.extraDataItems[4].readList();
        header.commitSeals = new bytes[](sealItems.length);
        for (uint256 i = 0; i < sealItems.length; ++i) {
            header.commitSeals[i] = sealItems[i].readBytes();
        }
    }
}
