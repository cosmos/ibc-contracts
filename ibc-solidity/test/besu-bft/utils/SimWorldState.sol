// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable gas-custom-errors,gas-strict-inequalities,gas-increment-by-one,no-inline-assembly

import { RLP } from "@openzeppelin-contracts/utils/RLP.sol";
import { MerklePatriciaTrie } from "./MerklePatriciaTrie.sol";

/// @title Sim World State
/// @notice Test-only versioned Ethereum world state: accounts, storage writes tagged with the height they become
/// visible at, and state/account/storage roots and proofs computed with `MerklePatriciaTrie` for any height.
abstract contract SimWorldState {
    using RLP for RLP.Encoder;

    struct Version {
        uint64 height;
        bytes32 value;
    }

    struct SimAccount {
        bool exists;
        uint64 nonce;
        uint256 balance;
        bytes32 codeHash;
        bytes32[] slots;
        mapping(bytes32 slot => Version[] history) history;
    }

    /// @dev ERC-7201 namespace of `IBCStoreUpgradeable`; the commitments mapping is its first field.
    bytes32 internal constant IBCSTORE_STORAGE_SLOT =
        0x1260944489272988d9df285149b5aa1b0f48f2136d6f416159f840a3e0747600;

    address[] private _accountList;
    mapping(address account => SimAccount data) private _accounts;

    /// @notice State root of the world state as visible at `height`.
    function stateRootAt(uint64 height) public view returns (bytes32) {
        (bytes[] memory keys, bytes[] memory values) = _stateEntries(height);
        return MerklePatriciaTrie.build(keys, values);
    }

    /// @notice Storage slot of an ICS24 commitment path in the router's `IBCStoreUpgradeable` mapping.
    function commitmentSlot(bytes memory path) public pure returns (bytes32) {
        return keccak256(abi.encode(keccak256(path), IBCSTORE_STORAGE_SLOT));
    }

    function _createAccount(address account, uint64 nonce, bytes32 codeHash) internal {
        SimAccount storage acc = _accounts[account];
        require(!acc.exists, "account exists");
        acc.exists = true;
        acc.nonce = nonce;
        acc.codeHash = codeHash;
        _accountList.push(account);
    }

    /// @dev Records `value` for `slot`, visible from `height` on. Zero deletes the slot.
    function _setStorage(address account, bytes32 slot, bytes32 value, uint64 height) internal {
        SimAccount storage acc = _accounts[account];
        require(acc.exists, "unknown account");
        if (acc.history[slot].length == 0) {
            acc.slots.push(slot);
        }
        acc.history[slot].push(Version(height, value));
    }

    function _storageAt(address account, bytes32 slot, uint64 height) internal view returns (bytes32) {
        Version[] storage versions = _accounts[account].history[slot];
        for (uint256 i = versions.length; i > 0; --i) {
            if (versions[i - 1].height <= height) {
                return versions[i - 1].value;
            }
        }
        return bytes32(0);
    }

    /// @dev Account proof from the state trie and storage proof from the account's storage trie at `height`.
    function _proofs(
        address account,
        bytes32 slot,
        uint64 height
    )
        internal
        view
        returns (bytes[] memory accountNodes, bytes[] memory storageNodes)
    {
        (bytes[] memory keys, bytes[] memory values) = _stateEntries(height);
        (, accountNodes) =
            MerklePatriciaTrie.prove(keys, values, abi.encodePacked(keccak256(abi.encodePacked(account))));
        (keys, values) = _storageEntries(account, height);
        (, storageNodes) = MerklePatriciaTrie.prove(keys, values, abi.encodePacked(keccak256(abi.encodePacked(slot))));
    }

    function _storageEntries(
        address account,
        uint64 height
    )
        private
        view
        returns (bytes[] memory keys, bytes[] memory values)
    {
        bytes32[] storage slots = _accounts[account].slots;
        keys = new bytes[](slots.length);
        values = new bytes[](slots.length);
        uint256 n;
        for (uint256 i = 0; i < slots.length; ++i) {
            bytes32 value = _storageAt(account, slots[i], height);
            if (value != bytes32(0)) {
                keys[n] = abi.encodePacked(keccak256(abi.encodePacked(slots[i])));
                values[n++] = RLP.encode(uint256(value));
            }
        }
        assembly ("memory-safe") {
            mstore(keys, n)
            mstore(values, n)
        }
    }

    function _stateEntries(uint64 height) private view returns (bytes[] memory keys, bytes[] memory values) {
        keys = new bytes[](_accountList.length);
        values = new bytes[](_accountList.length);
        for (uint256 i = 0; i < _accountList.length; ++i) {
            SimAccount storage acc = _accounts[_accountList[i]];
            (bytes[] memory storageKeys, bytes[] memory storageValues) = _storageEntries(_accountList[i], height);
            keys[i] = abi.encodePacked(keccak256(abi.encodePacked(_accountList[i])));
            values[i] = RLP.encode(
                RLP.encoder()
                    .push(uint256(acc.nonce))
                    .push(acc.balance)
                    .push(MerklePatriciaTrie.build(storageKeys, storageValues))
                    .push(acc.codeHash)
            );
        }
    }
}
