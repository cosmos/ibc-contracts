// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { Memory } from "@openzeppelin-contracts/utils/Memory.sol";
import { RLP } from "@openzeppelin-contracts/utils/RLP.sol";

/// @title Sim Header
/// @notice Test-only Besu BFT header model with RLP encoding, commit seal digests and block hashes.
/// @dev Field layout matches Cancun-era Besu headers (20 items) and the BFT `extraData` list
/// `[vanity, validators, vote, round, commitSeals]`. The vote item is always encoded as an empty list.
library SimHeader {
    using RLP for *;
    using Memory for *;

    /// @notice Consensus flavour, which only changes how commit seals are hashed.
    enum Mode {
        QBFT,
        IBFT2
    }

    /// @notice Mutable header; tests may change any field before sealing.
    // solhint-disable-next-line gas-struct-packing
    struct Data {
        bytes32 parentHash;
        bytes32 ommersHash;
        address beneficiary;
        bytes32 stateRoot;
        bytes32 transactionsRoot;
        bytes32 receiptsRoot;
        bytes logsBloom;
        uint256 difficulty;
        uint64 number;
        uint64 gasLimit;
        uint64 gasUsed;
        uint64 timestamp;
        bytes32 mixHash;
        bytes8 nonce;
        uint256 baseFeePerGas;
        bytes32 withdrawalsRoot;
        uint64 blobGasUsed;
        uint64 excessBlobGas;
        bytes32 parentBeaconBlockRoot;
        bytes32 vanity;
        address[] validators;
        uint256 round;
        bytes[] commitSeals;
    }

    bytes32 internal constant BESU_BFT_MIX_HASH = 0x63746963616c2062797a616e74696e65206661756c7420746f6c6572616e6365;
    bytes32 internal constant EMPTY_OMMERS_HASH = 0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347;
    bytes32 internal constant EMPTY_ROOT_HASH = 0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421;

    /// @notice Full header RLP, commit seals included.
    function encode(Data memory h) internal pure returns (bytes memory) {
        return _encode(h, Mode.QBFT, true, false);
    }

    /// @notice Digest signed by commit seals: QBFT keeps the round and an empty seal list, IBFT2 drops the seal item.
    function commitSealDigest(Data memory h, Mode mode) internal pure returns (bytes32) {
        return keccak256(_encode(h, mode, false, false));
    }

    /// @notice Canonical Besu block hash, which excludes commit seals and zeroes the round number.
    function blockHash(Data memory h, Mode mode) internal pure returns (bytes32) {
        return keccak256(_encode(h, mode, false, true));
    }

    /// @notice Decodes a header RLP whose vote item is empty.
    function decode(bytes memory headerRlp) internal pure returns (Data memory h) {
        Memory.Slice[] memory it = headerRlp.decodeList();
        h.parentHash = it[0].readBytes32();
        h.ommersHash = it[1].readBytes32();
        h.beneficiary = it[2].readAddress();
        h.stateRoot = it[3].readBytes32();
        h.transactionsRoot = it[4].readBytes32();
        h.receiptsRoot = it[5].readBytes32();
        h.logsBloom = it[6].readBytes();
        h.difficulty = it[7].readUint256();
        h.number = uint64(it[8].readUint256());
        h.gasLimit = uint64(it[9].readUint256());
        h.gasUsed = uint64(it[10].readUint256());
        h.timestamp = uint64(it[11].readUint256());
        h.mixHash = it[13].readBytes32();
        h.nonce = bytes8(it[14].readBytes());
        h.baseFeePerGas = it[15].readUint256();
        h.withdrawalsRoot = it[16].readBytes32();
        h.blobGasUsed = uint64(it[17].readUint256());
        h.excessBlobGas = uint64(it[18].readUint256());
        h.parentBeaconBlockRoot = it[19].readBytes32();

        Memory.Slice[] memory extra = it[12].readBytes().decodeList();
        h.vanity = extra[0].readBytes32();
        Memory.Slice[] memory validators = extra[1].readList();
        h.validators = new address[](validators.length);
        for (uint256 i = 0; i < validators.length; ++i) {
            h.validators[i] = validators[i].readAddress();
        }
        h.round = extra[3].readUint256();
        Memory.Slice[] memory seals = extra[4].readList();
        h.commitSeals = new bytes[](seals.length);
        for (uint256 i = 0; i < seals.length; ++i) {
            h.commitSeals[i] = seals[i].readBytes();
        }
    }

    function _encode(Data memory h, Mode mode, bool withSeals, bool zeroRound) private pure returns (bytes memory) {
        RLP.Encoder memory validators = RLP.encoder();
        for (uint256 i = 0; i < h.validators.length; ++i) {
            validators.push(h.validators[i]);
        }
        RLP.Encoder memory extra =
            RLP.encoder().push(h.vanity).push(validators).push(RLP.encoder()).push(zeroRound ? 0 : h.round);
        if (withSeals) {
            RLP.Encoder memory seals = RLP.encoder();
            for (uint256 i = 0; i < h.commitSeals.length; ++i) {
                seals.push(h.commitSeals[i]);
            }
            extra.push(seals);
        } else if (mode == Mode.QBFT) {
            extra.push(RLP.encoder());
        }

        RLP.Encoder memory header = RLP.encoder()
            .push(h.parentHash)
            .push(h.ommersHash)
            .push(h.beneficiary)
            .push(h.stateRoot)
            .push(h.transactionsRoot)
            .push(h.receiptsRoot)
            .push(h.logsBloom)
            .push(h.difficulty)
            .push(uint256(h.number))
            .push(uint256(h.gasLimit))
            .push(uint256(h.gasUsed))
            .push(uint256(h.timestamp))
            .push(RLP.encode(extra))
            .push(h.mixHash)
            .push(abi.encodePacked(h.nonce))
            .push(h.baseFeePerGas)
            .push(h.withdrawalsRoot)
            .push(uint256(h.blobGasUsed))
            .push(uint256(h.excessBlobGas))
            .push(h.parentBeaconBlockRoot);
        return RLP.encode(header);
    }
}
