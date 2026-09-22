// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable gas-custom-errors,gas-calldata-parameters

import { Test } from "forge-std/Test.sol";

import { IICS02ClientMsgs } from "../../../contracts/msgs/IICS02ClientMsgs.sol";
import { IICS26RouterMsgs } from "../../../contracts/msgs/IICS26RouterMsgs.sol";
import { ILightClientMsgs } from "../../../contracts/msgs/ILightClientMsgs.sol";
import { IBesuLightClientMsgs } from "../../../contracts/light-clients/besu/msgs/IBesuLightClientMsgs.sol";
import { IBesuLightClient } from "../../../contracts/light-clients/besu/interfaces/IBesuLightClient.sol";

import { BesuQBFTLightClient } from "../../../contracts/light-clients/besu/BesuQBFTLightClient.sol";
import { BesuIBFT2LightClient } from "../../../contracts/light-clients/besu/BesuIBFT2LightClient.sol";
import { ICS24Host } from "../../../contracts/utils/ICS24Host.sol";
import { IBCStoreUpgradeable } from "../../../contracts/utils/IBCStoreUpgradeable.sol";
import { SimHeader } from "./SimHeader.sol";
import { SimWorldState } from "./SimWorldState.sol";

/// @title QBFT Sim Suite
/// @notice In-process Besu QBFT/IBFT2 chain simulator: manages validator keys, produces and seals RLP headers, and
/// keeps a world state whose roots and MPT proofs the Besu light clients accept.
/// @dev Storage writes made while the tip is height `h` land in block `h + 1`. `commit` warps the EVM clock forward
/// to the committed timestamp so light client clock-drift checks pass by default.
contract QBFTSimSuite is Test, SimWorldState {
    using SimHeader for SimHeader.Data;

    struct Block {
        uint64 timestamp;
        bytes32 stateRoot;
        bytes32 blockHash;
        address[] validators;
        bytes rlp;
    }

    SimHeader.Mode public immutable MODE;
    address public immutable IBC_ROUTER;
    uint64 public blockPeriod = 2;

    Block[] private _chain;
    address[] private _validators;
    mapping(address validator => uint256 key) private _keys;
    uint256 private _validatorCount;
    uint256 private _accountCount;

    constructor(SimHeader.Mode mode) {
        MODE = mode;
        IBC_ROUTER = makeAddr("sim-ibc-router");
        _createAccount(IBC_ROUTER, 1, keccak256("sim-ibc-router-code"), 0);
        _chain.push();
        _chain[0].timestamp = uint64(vm.getBlockTimestamp());
        _chain[0].stateRoot = stateRootAt(0);
    }

    // ---------------------------------------------------------------- validators

    /// @notice Creates a validator key; it seals blocks from the next produced block on.
    function addValidator() public returns (address validator) {
        uint256 key;
        (validator, key) = makeAddrAndKey(string.concat("validator-", vm.toString(++_validatorCount)));
        _keys[validator] = key;
        _validators.push(validator);
        for (uint256 i = _validators.length - 1; i > 0 && _validators[i - 1] > validator; --i) {
            (_validators[i], _validators[i - 1]) = (_validators[i - 1], _validators[i]);
        }
    }

    function addValidators(uint256 count) external returns (address[] memory added) {
        added = new address[](count);
        for (uint256 i = 0; i < count; ++i) {
            added[i] = addValidator();
        }
    }

    function removeValidator(address validator) external {
        uint256 n = _validators.length;
        for (uint256 i = 0; i < n; ++i) {
            if (_validators[i] == validator) {
                for (; i + 1 < n; ++i) {
                    _validators[i] = _validators[i + 1];
                }
                _validators.pop();
                return;
            }
        }
        revert("unknown validator");
    }

    /// @notice Sorted validator set that seals the next produced block.
    function validators() external view returns (address[] memory) {
        return _validators;
    }

    function validatorKey(address validator) external view returns (uint256) {
        return _keys[validator];
    }

    // ---------------------------------------------------------------- world state

    /// @notice Adds `count` empty accounts to the world state from the next block on, to deepen the account trie.
    function addAccounts(uint256 count) external {
        for (uint256 i = 0; i < count; ++i) {
            address account = address(uint160(uint256(keccak256(abi.encode("sim-account", ++_accountCount)))));
            _createAccount(account, 1, keccak256(""), tipHeight() + 1);
        }
    }

    function setStorage(address account, bytes32 slot, bytes32 value) external {
        _setStorage(account, slot, value, tipHeight() + 1);
    }

    /// @notice Writes an ICS24 commitment into the router's IBC store; zero deletes it.
    function setCommitment(bytes memory path, bytes32 value) public {
        _setStorage(IBC_ROUTER, commitmentSlot(path), value, tipHeight() + 1);
    }

    function commitPacket(IICS26RouterMsgs.Packet memory packet) external {
        setCommitment(
            ICS24Host.packetCommitmentPathCalldata(packet.sourceClient, packet.sequence),
            ICS24Host.packetCommitmentBytes32(packet)
        );
    }

    /// @notice Mirrors the commitment stored under `path` by a real router into the simulated state.
    function syncCommitment(IBCStoreUpgradeable router, bytes memory path) external {
        setCommitment(path, router.getCommitment(keccak256(path)));
    }

    // ---------------------------------------------------------------- blocks

    function tipHeight() public view returns (uint64) {
        return uint64(_chain.length - 1);
    }

    function blockAt(uint64 height) external view returns (Block memory) {
        return _chain[height];
    }

    /// @notice Honest unsealed draft of the block following the tip.
    function nextBlock() public view returns (SimHeader.Data memory h) {
        Block storage tip = _chain[tipHeight()];
        h.parentHash = tip.blockHash;
        h.ommersHash = SimHeader.EMPTY_OMMERS_HASH;
        h.beneficiary = _validators[_chain.length % _validators.length];
        h.stateRoot = stateRootAt(tipHeight() + 1);
        h.transactionsRoot = SimHeader.EMPTY_ROOT_HASH;
        h.receiptsRoot = SimHeader.EMPTY_ROOT_HASH;
        h.logsBloom = new bytes(256);
        h.difficulty = 1;
        h.number = tipHeight() + 1;
        h.gasLimit = 30_000_000;
        h.timestamp = tip.timestamp + blockPeriod;
        h.mixHash = SimHeader.BESU_BFT_MIX_HASH;
        h.withdrawalsRoot = SimHeader.EMPTY_ROOT_HASH;
        h.validators = _validators;
    }

    /// @notice Seals `h` with every validator listed in it.
    function seal(SimHeader.Data memory h) public view returns (SimHeader.Data memory) {
        return seal(h, h.validators);
    }

    function seal(SimHeader.Data memory h, address[] memory signers) public view returns (SimHeader.Data memory) {
        uint256[] memory keys = new uint256[](signers.length);
        for (uint256 i = 0; i < signers.length; ++i) {
            keys[i] = _keys[signers[i]];
            require(keys[i] != 0, "unknown signer");
        }
        return sealWithKeys(h, keys);
    }

    /// @notice Seals `h` with arbitrary private keys, e.g. to add unknown or duplicate signers.
    function sealWithKeys(SimHeader.Data memory h, uint256[] memory keys) public view returns (SimHeader.Data memory) {
        bytes32 digest = h.commitSealDigest(MODE);
        h.commitSeals = new bytes[](keys.length);
        for (uint256 i = 0; i < keys.length; ++i) {
            (uint8 v, bytes32 r, bytes32 s) = vm.sign(keys[i], digest);
            h.commitSeals[i] = abi.encodePacked(r, s, v);
        }
        return h;
    }

    /// @notice Appends a sealed header as the new tip without validating it, and warps the clock forward to it.
    function commit(SimHeader.Data memory h) public {
        require(h.number == _chain.length, "not next height");
        _chain.push(Block(h.timestamp, h.stateRoot, h.blockHash(MODE), h.validators, h.encode()));
        if (vm.getBlockTimestamp() < h.timestamp) {
            vm.warp(h.timestamp);
        }
    }

    function produceBlock() public returns (SimHeader.Data memory h) {
        h = seal(nextBlock());
        commit(h);
    }

    function produceBlocks(uint256 count) external {
        for (uint256 i = 0; i < count; ++i) {
            produceBlock();
        }
    }

    // ---------------------------------------------------------------- light client glue

    /// @notice Deploys the light client for `MODE`, trusting the current tip, with open proof submission.
    function deployLightClient(uint64 trustingPeriod, uint64 maxClockDrift) external returns (IBesuLightClient) {
        uint64 height = tipHeight();
        Block storage tip = _chain[height];
        if (MODE == SimHeader.Mode.QBFT) {
            return new BesuQBFTLightClient(
                IBC_ROUTER,
                height,
                tip.timestamp,
                tip.stateRoot,
                tip.validators,
                trustingPeriod,
                maxClockDrift,
                address(0)
            );
        }
        return new BesuIBFT2LightClient(
            IBC_ROUTER, height, tip.timestamp, tip.stateRoot, tip.validators, trustingPeriod, maxClockDrift, address(0)
        );
    }

    function consensusState(uint64 height) public view returns (IBesuLightClientMsgs.ConsensusState memory) {
        Block storage b = _chain[height];
        return IBesuLightClientMsgs.ConsensusState(b.timestamp, b.stateRoot, b.validators);
    }

    /// @notice `updateClient` payload submitting the committed header at `height` against `trustedHeight`.
    function updateMsg(uint64 trustedHeight, uint64 height) external view returns (bytes memory) {
        return _updateMsg(trustedHeight, _chain[height].rlp);
    }

    /// @notice `updateClient` payload submitting any sealed header (committed or not) against `trustedHeight`.
    function updateMsg(uint64 trustedHeight, SimHeader.Data memory header) external view returns (bytes memory) {
        return _updateMsg(trustedHeight, header.encode());
    }

    function membershipMsg(
        uint64 height,
        bytes memory path
    )
        external
        view
        returns (ILightClientMsgs.MsgVerifyMembership memory)
    {
        bytes32 value = _storageAt(IBC_ROUTER, commitmentSlot(path), height);
        require(value != bytes32(0), "no commitment at height");
        return ILightClientMsgs.MsgVerifyMembership(
            _proof(height, path), _height(height), _singlePath(path), abi.encodePacked(value)
        );
    }

    function nonMembershipMsg(
        uint64 height,
        bytes memory path
    )
        external
        view
        returns (ILightClientMsgs.MsgVerifyNonMembership memory)
    {
        require(_storageAt(IBC_ROUTER, commitmentSlot(path), height) == bytes32(0), "commitment exists at height");
        return ILightClientMsgs.MsgVerifyNonMembership(_proof(height, path), _height(height), _singlePath(path));
    }

    function _updateMsg(uint64 trustedHeight, bytes memory headerRlp) private view returns (bytes memory) {
        return abi.encode(
            IBesuLightClientMsgs.MsgUpdateClient(headerRlp, _height(trustedHeight), consensusState(trustedHeight))
        );
    }

    function _proof(uint64 height, bytes memory path) private view returns (bytes memory) {
        (bytes[] memory accountNodes, bytes[] memory storageNodes) = _proofs(IBC_ROUTER, commitmentSlot(path), height);
        return abi.encode(IBesuLightClientMsgs.MembershipProof(consensusState(height), accountNodes, storageNodes));
    }

    function _height(uint64 height) private pure returns (IICS02ClientMsgs.Height memory) {
        return IICS02ClientMsgs.Height(0, height);
    }

    function _singlePath(bytes memory path) private pure returns (bytes[] memory out) {
        out = new bytes[](1);
        out[0] = path;
    }
}
