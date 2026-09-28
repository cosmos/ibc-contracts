// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { AccessControl } from "@openzeppelin-contracts/access/AccessControl.sol";
import { ECDSA } from "@openzeppelin-contracts/utils/cryptography/ECDSA.sol";
import { RLP } from "@openzeppelin-contracts/utils/RLP.sol";
import { TrieProof } from "../../utils/TrieProof.sol";
import { Memory } from "@openzeppelin-contracts/utils/Memory.sol";
import { TransientSlot } from "@openzeppelin-contracts/utils/TransientSlot.sol";
import { Math } from "@openzeppelin-contracts/utils/math/Math.sol";

import { ILightClient } from "../../interfaces/ILightClient.sol";
import { ILightClientMsgs } from "../../msgs/ILightClientMsgs.sol";
import { IICS02ClientMsgs } from "../../msgs/IICS02ClientMsgs.sol";
import { IBesuLightClientMsgs } from "./msgs/IBesuLightClientMsgs.sol";
import { IBesuLightClientErrors } from "./errors/IBesuLightClientErrors.sol";
import { IBesuLightClient } from "./interfaces/IBesuLightClient.sol";

import { Header } from "./utils/Header.sol";

/// @title Besu Light Client Base
/// @notice Shared implementation for Besu BFT light clients that verify headers and EVM storage proofs.
abstract contract BesuLightClientBase is IBesuLightClient, IBesuLightClientErrors, AccessControl {
    using TransientSlot for TransientSlot.Bytes32Slot;

    /// @notice Role allowed to submit client updates and proof verifications.
    // natlint-disable-next-line MissingInheritdoc
    bytes32 public constant PROOF_SUBMITTER_ROLE = keccak256("PROOF_SUBMITTER_ROLE");

    /// @notice ERC-7201 storage slot used by `IBCStoreUpgradeable` commitments.
    /// @dev keccak256(abi.encode(uint256(keccak256("ibc.storage.IBCStore")) - 1)) & ~bytes32(uint256(0xff))
    bytes32 private constant IBCSTORE_STORAGE_SLOT = 0x1260944489272988d9df285149b5aa1b0f48f2136d6f416159f840a3e0747600;

    /// @notice Current client state.
    IBesuLightClientMsgs.ClientState private clientState;
    /// @notice Keccak256 hash of trusted consensus states by revision height.
    mapping(uint64 revisionHeight => bytes32 consensusStateHash) private consensusStateHashes;

    /// @notice Initializes shared Besu light client state.
    /// @param ibcRouter Counterparty ICS26 router address whose storage is proven.
    /// @param initialTrustedHeight Initial trusted Besu height.
    /// @param initialTrustedTimestamp Initial trusted header timestamp in seconds.
    /// @param initialTrustedStateRoot Initial trusted root of the state trie at `initialTrustedHeight`.
    /// @param initialTrustedValidators Initial trusted validator set.
    /// @param trustingPeriod Maximum age in seconds for trusted consensus states.
    /// @param maxClockDrift Maximum allowed future drift in seconds for submitted headers.
    /// @param roleManager Address that administers proof submission; if zero, proof submission is open.
    constructor(
        address ibcRouter,
        uint64 initialTrustedHeight,
        uint64 initialTrustedTimestamp,
        bytes32 initialTrustedStateRoot,
        address[] memory initialTrustedValidators,
        uint64 trustingPeriod,
        uint64 maxClockDrift,
        address roleManager
    ) {
        require(initialTrustedHeight != 0, InvalidHeaderHeight());
        require(initialTrustedTimestamp != 0, InvalidHeaderTimestamp());
        require(trustingPeriod != 0, InvalidTrustingPeriod());

        _validateValidators(initialTrustedValidators);

        clientState = IBesuLightClientMsgs.ClientState({
            ibcRouter: ibcRouter,
            latestHeight: IICS02ClientMsgs.Height({ revisionNumber: 0, revisionHeight: initialTrustedHeight }),
            trustingPeriod: trustingPeriod,
            maxClockDrift: maxClockDrift,
            isFrozen: false
        });

        IBesuLightClientMsgs.ConsensusState memory initialConsensusState = IBesuLightClientMsgs.ConsensusState({
            timestamp: initialTrustedTimestamp, stateRoot: initialTrustedStateRoot, validators: initialTrustedValidators
        });
        consensusStateHashes[initialTrustedHeight] = keccak256(abi.encode(initialConsensusState));

        if (roleManager == address(0)) {
            _grantRole(PROOF_SUBMITTER_ROLE, address(0));
        } else {
            _grantRole(DEFAULT_ADMIN_ROLE, roleManager);
            _grantRole(PROOF_SUBMITTER_ROLE, roleManager);
        }
    }

    /// @inheritdoc ILightClient
    function getClientState() external view returns (bytes memory) {
        return abi.encode(clientState);
    }

    /// @inheritdoc IBesuLightClient
    function getConsensusStateHash(uint64 revisionHeight) external view returns (bytes32) {
        bytes32 consensusStateHash = consensusStateHashes[revisionHeight];
        require(consensusStateHash != bytes32(0), ConsensusStateNotFound(revisionHeight));
        return consensusStateHash;
    }

    /// @inheritdoc ILightClient
    function updateClient(bytes calldata updateMsg)
        external
        notFrozen
        onlyProofSubmitter
        returns (ILightClientMsgs.UpdateResult)
    {
        IBesuLightClientMsgs.MsgUpdateClient memory msg_ = abi.decode(updateMsg, (IBesuLightClientMsgs.MsgUpdateClient));
        require(msg_.trustedHeight.revisionNumber == 0, InvalidRevisionNumber(msg_.trustedHeight.revisionNumber));

        Header.Data memory header = Header.decodeRlp(msg_.headerRlp);
        require(header.height != 0, InvalidHeaderHeight());
        require(header.timestamp != 0, InvalidHeaderTimestamp());
        _validateValidators(header.validators);
        require(
            // solhint-disable-next-line gas-strict-inequalities
            block.timestamp + clientState.maxClockDrift >= header.timestamp,
            HeaderFromFuture(block.timestamp, header.timestamp, clientState.maxClockDrift)
        );

        _requireTrustedConsensusState(msg_.trustedHeight.revisionHeight, msg_.consensusStatePreimage);

        address[] memory signers = _recoverSigners(_commitSealDigest(header), header.commitSeals);
        _checkTrustedValidatorOverlap(signers, msg_.consensusStatePreimage.validators);
        _checkValidatorQuorum(signers, header.validators);

        IBesuLightClientMsgs.ConsensusState memory newConsensusState = IBesuLightClientMsgs.ConsensusState({
            timestamp: header.timestamp, stateRoot: header.stateRoot, validators: header.validators
        });

        bytes32 newHash = keccak256(abi.encode(newConsensusState));
        bytes32 existingHash = consensusStateHashes[header.height];
        if (existingHash != bytes32(0)) {
            if (existingHash == newHash) {
                return ILightClientMsgs.UpdateResult.NoOp;
            }

            clientState.isFrozen = true;
            emit DoubleSign(header.height, existingHash, newHash);
            return ILightClientMsgs.UpdateResult.Misbehaviour;
        }

        consensusStateHashes[header.height] = newHash;

        if (header.height > clientState.latestHeight.revisionHeight) {
            clientState.latestHeight.revisionHeight = header.height;
        }

        return ILightClientMsgs.UpdateResult.Update;
    }

    /// @inheritdoc ILightClient
    function verifyMembership(ILightClientMsgs.MsgVerifyMembership calldata msg_)
        external
        notFrozen
        onlyProofSubmitter
        returns (uint256)
    {
        require(msg_.proofHeight.revisionNumber == 0, InvalidRevisionNumber(msg_.proofHeight.revisionNumber));
        require(msg_.path.length == 1, InvalidPathLength(1, msg_.path.length));
        require(msg_.value.length == 32, InvalidValueLength(32, msg_.value.length));

        IBesuLightClientMsgs.MembershipProof memory proof =
            abi.decode(msg_.proof, (IBesuLightClientMsgs.MembershipProof));
        _requireTrustedConsensusState(msg_.proofHeight.revisionHeight, proof.consensusStatePreimage);

        bytes32 storageRoot = _verifiedStorageRoot(
            msg_.proofHeight.revisionHeight, proof.consensusStatePreimage.stateRoot, proof.accountProofNodes
        );

        bytes32 storageSlot = _commitmentStorageSlot(msg_.path[0]);
        bytes memory storageKey = abi.encodePacked(keccak256(abi.encodePacked(storageSlot)));

        bytes memory traversedValue = TrieProof.traverse(storageRoot, storageKey, proof.proofNodes);

        bytes32 actualValue = RLP.decodeBytes32(traversedValue);
        bytes32 expectedValue = bytes32(msg_.value);
        require(actualValue == expectedValue, InvalidCommitmentValue(expectedValue, actualValue));

        return proof.consensusStatePreimage.timestamp;
    }

    /// @inheritdoc ILightClient
    function verifyNonMembership(ILightClientMsgs.MsgVerifyNonMembership calldata msg_)
        external
        notFrozen
        onlyProofSubmitter
        returns (uint256)
    {
        require(msg_.proofHeight.revisionNumber == 0, InvalidRevisionNumber(msg_.proofHeight.revisionNumber));
        require(msg_.path.length == 1, InvalidPathLength(1, msg_.path.length));

        IBesuLightClientMsgs.MembershipProof memory proof =
            abi.decode(msg_.proof, (IBesuLightClientMsgs.MembershipProof));
        _requireTrustedConsensusState(msg_.proofHeight.revisionHeight, proof.consensusStatePreimage);

        bytes32 storageRoot = _verifiedStorageRoot(
            msg_.proofHeight.revisionHeight, proof.consensusStatePreimage.stateRoot, proof.accountProofNodes
        );

        bytes32 storageSlot = _commitmentStorageSlot(msg_.path[0]);
        bytes memory storageKey = abi.encodePacked(keccak256(abi.encodePacked(storageSlot)));

        require(TrieProof.verifyExclusion(storageRoot, storageKey, proof.proofNodes), InvalidExclusionProof());

        return proof.consensusStatePreimage.timestamp;
    }

    /// @inheritdoc ILightClient
    function misbehaviour(bytes calldata) external view notFrozen onlyProofSubmitter {
        revert UnsupportedMisbehaviour();
    }

    /// @notice Computes the protocol-specific commit seal digest for a parsed header.
    /// @dev See `hashBlockForCommitSeal` in QBFT specification: https://entethalliance.org/specs/qbft/v1
    /// @param header The parsed Besu header.
    /// @return The digest signed by commit seals.
    function _commitSealDigest(Header.Data memory header) internal pure virtual returns (bytes32);

    /// @notice Returns the storage root for a revision height, verifying the account proof if provided.
    /// @dev If the account proof is empty, the storage root is retrieved from a transient cache
    /// which can only be populated by a previous call to this function with a non-empty account proof.
    /// @param revisionHeight The revision height for the storage root.
    /// @param stateRoot The header state root.
    /// @param accountProofNodes The ordered, RLP-encoded MPT nodes from the state trie proving the account.
    /// @return The storage root for the revision height.
    function _verifiedStorageRoot(
        uint64 revisionHeight,
        bytes32 stateRoot,
        bytes[] memory accountProofNodes
    )
        private
        returns (bytes32)
    {
        address counterpartyRouter = clientState.ibcRouter;
        if (accountProofNodes.length == 0) {
            return _getCachedStorageRoot(counterpartyRouter, revisionHeight);
        }

        bytes32 storageRoot = _verifyAccountProof(counterpartyRouter, stateRoot, accountProofNodes);
        _cacheStorageRoot(counterpartyRouter, revisionHeight, storageRoot);
        return storageRoot;
    }

    /// @notice Verifies the tracked account proof against a header state root.
    /// @param account The account address being proven.
    /// @param stateRoot The header state root.
    /// @param proofNodes The ordered, RLP-encoded MPT nodes from the state trie proving the account.
    /// @return The proven account storage root.
    function _verifyAccountProof(
        address account,
        bytes32 stateRoot,
        bytes[] memory proofNodes
    )
        private
        pure
        returns (bytes32)
    {
        bytes memory accountKey = abi.encodePacked(keccak256(abi.encodePacked(account)));
        bytes memory accountRlp = TrieProof.traverse(stateRoot, accountKey, proofNodes);
        Memory.Slice[] memory accountItems = RLP.decodeList(accountRlp);
        return RLP.readBytes32(accountItems[2]);
    }

    /// @notice Computes the storage slot used for an IBC commitment path.
    /// @param rawPath Raw commitment path bytes.
    /// @return The storage slot for the commitment.
    function _commitmentStorageSlot(bytes memory rawPath) private pure returns (bytes32) {
        return keccak256(abi.encode(keccak256(rawPath), IBCSTORE_STORAGE_SLOT));
    }

    /// @notice Recovers unique commit seal signers for a digest.
    /// @param digest The commit seal digest.
    /// @param seals The commit seals to recover.
    /// @return signers The recovered signer addresses.
    function _recoverSigners(bytes32 digest, bytes[] memory seals) private pure returns (address[] memory signers) {
        signers = new address[](seals.length);
        for (uint256 i = 0; i < seals.length; ++i) {
            address signer = _recoverSigner(digest, seals[i]);
            for (uint256 j = 0; j < i; ++j) {
                require(signers[j] != signer, DuplicateCommitSealSigner(signer));
            }
            signers[i] = signer;
        }
    }

    /// @notice Recovers one commit seal signer.
    /// @param digest The commit seal digest.
    /// @param seal The 65-byte ECDSA commit seal.
    /// @return The recovered signer address.
    function _recoverSigner(bytes32 digest, bytes memory seal) private pure returns (address) {
        require(seal.length == 65, InvalidECDSASignatureLength(seal.length));
        if (uint8(seal[64]) < 27) {
            seal[64] = bytes1(uint8(seal[64]) + 27);
        }
        (address signer, ECDSA.RecoverError err, bytes32 errorArgument) = ECDSA.tryRecover(digest, seal);
        require(
            err == ECDSA.RecoverError.NoError && errorArgument == bytes32(0) && signer != address(0),
            InvalidCommitSeal()
        );
        return signer;
    }

    /// @notice Checks that signers overlap enough with the trusted validator set.
    /// @param signers The recovered commit seal signers.
    /// @param trustedValidators The trusted validator set.
    function _checkTrustedValidatorOverlap(address[] memory signers, address[] memory trustedValidators) private pure {
        uint256 actual = 0;
        for (uint256 i = 0; i < signers.length; ++i) {
            if (_containsMemory(trustedValidators, signers[i])) {
                ++actual;
            }
        }

        uint256 required = _bftThreshold(trustedValidators.length);
        require(actual >= required, InsufficientTrustedValidatorOverlap(actual, required));
        // solhint-disable-previous-line gas-strict-inequalities
    }

    /// @notice Checks that signers meet quorum for the submitted header validator set.
    /// @dev Assumes that the signer set has no duplicates. Checked in `_recoverSigners`.
    /// @param signers The recovered commit seal signers.
    /// @param validators The validator set from the submitted header.
    function _checkValidatorQuorum(address[] memory signers, address[] memory validators) private pure {
        for (uint256 i = 0; i < signers.length; ++i) {
            require(_containsMemory(validators, signers[i]), UnknownCommitSealSigner(signers[i]));
        }

        uint256 required = _bftThreshold(validators.length);
        require(signers.length >= required, InsufficientValidatorQuorum(signers.length, required));
        // solhint-disable-previous-line gas-strict-inequalities
    }

    /// @notice Computes the BFT threshold `ceil(2n / 3)` used for trusted overlap and quorum checks.
    /// @dev Besu requires `ceil(2n / 3)` commit seals; the trusted overlap intentionally uses the same threshold.
    /// @param n The validator set size.
    /// @return The minimum number of matching signers.
    function _bftThreshold(uint256 n) private pure returns (uint256) {
        return Math.ceilDiv(2 * n, 3);
    }

    /// @notice Validates that a validator set is non-empty, unique, and sorted.
    /// @param validators The validator set to validate.
    function _validateValidators(address[] memory validators) private pure {
        require(validators.length != 0, EmptyValidatorSet());
        require(validators[0] != address(0), InvalidValidatorAddress(address(0)));
        for (uint256 i = 1; i < validators.length; ++i) {
            require(validators[i - 1] < validators[i], UnsortedValidatorSet(i - 1));
        }
    }

    /// @notice Reverts unless the given consensus state matches the stored hash and is within the trusting period.
    /// @param revisionHeight The consensus state revision height.
    /// @param preimage The consensus state preimage to check.
    function _requireTrustedConsensusState(
        uint64 revisionHeight,
        IBesuLightClientMsgs.ConsensusState memory preimage
    )
        private
        view
    {
        bytes32 consensusStateHash = consensusStateHashes[revisionHeight];
        require(consensusStateHash != bytes32(0), ConsensusStateNotFound(revisionHeight));

        bytes32 preimageHash = keccak256(abi.encode(preimage));
        require(consensusStateHash == preimageHash, ConsensusStatePreimageMismatch(consensusStateHash, preimageHash));

        require(
            uint256(preimage.timestamp) + clientState.trustingPeriod > block.timestamp,
            ConsensusStateExpired(preimage.timestamp, block.timestamp, clientState.trustingPeriod)
        );
    }

    /// @notice Checks whether a memory validator set contains a signer.
    /// @param validators The memory validator set.
    /// @param signer The signer to find.
    /// @return True if `signer` is present.
    function _containsMemory(address[] memory validators, address signer) private pure returns (bool) {
        for (uint256 i = 0; i < validators.length; ++i) {
            if (validators[i] == signer) {
                return true;
            }
        }
        return false;
    }

    /// @notice Caches a storage root for a revision height in a transient slot.
    /// @param ibcRouter The ICS26 router address whose storageRoot was proven.
    /// @param revisionHeight The revision height for the storage root.
    /// @param storageRoot The storage root to cache.
    function _cacheStorageRoot(address ibcRouter, uint64 revisionHeight, bytes32 storageRoot) private {
        _cacheKey(ibcRouter, revisionHeight).tstore(storageRoot);
    }

    /// @notice Retrieves a cached storage root for a revision height from a transient slot.
    /// @param ibcRouter The ICS26 router address whose storageRoot was proven.
    /// @param revisionHeight The revision height for the storage root.
    /// @return The cached storage root, reverting if not found.
    function _getCachedStorageRoot(address ibcRouter, uint64 revisionHeight) private view returns (bytes32) {
        bytes32 storageRoot = _cacheKey(ibcRouter, revisionHeight).tload();
        require(storageRoot != bytes32(0), StorageRootNotInCache(revisionHeight));
        return storageRoot;
    }

    /// @notice Computes a transient slot cache key for a revision height.
    /// @param ibcRouter The ICS26 router address whose storageRoot was proven.
    /// @param revisionHeight The revision height for the cache key.
    /// @return The cache key.
    function _cacheKey(address ibcRouter, uint64 revisionHeight) private pure returns (TransientSlot.Bytes32Slot) {
        return TransientSlot.asBytes32(keccak256(abi.encode(ibcRouter, revisionHeight)));
    }

    /// @notice Reverts if the client is frozen. A frozen client can never be unfrozen.
    modifier notFrozen() {
        require(!clientState.isFrozen, FrozenClientState());
        _;
    }

    /// @notice Restricts access to proof submitters unless submission is open to anyone.
    modifier onlyProofSubmitter() {
        if (!hasRole(PROOF_SUBMITTER_ROLE, address(0))) {
            _checkRole(PROOF_SUBMITTER_ROLE);
        }
        _;
    }
}
