// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

// solhint-disable gas-struct-packing, named-parameters-mapping, gas-strict-inequalities, code-complexity

import { AccessControl } from "@openzeppelin-contracts/access/AccessControl.sol";
import { ECDSA } from "@openzeppelin-contracts/utils/cryptography/ECDSA.sol";
import { RLP } from "@openzeppelin-contracts/utils/RLP.sol";
import { TrieProof } from "../../utils/TrieProof.sol";
import { Memory } from "@openzeppelin-contracts/utils/Memory.sol";

import { ILightClient } from "../../interfaces/ILightClient.sol";
import { ILightClientMsgs } from "../../msgs/ILightClientMsgs.sol";
import { IICS02ClientMsgs } from "../../msgs/IICS02ClientMsgs.sol";
import { IBesuLightClientMsgs } from "./msgs/IBesuLightClientMsgs.sol";
import { IBesuLightClientErrors } from "./errors/IBesuLightClientErrors.sol";
import { IBesuLightClient } from "./interfaces/IBesuLightClient.sol";

/// @title Besu Light Client Base
/// @notice Shared implementation for Besu BFT light clients that verify headers and EVM storage proofs.
abstract contract BesuLightClientBase is IBesuLightClient, IBesuLightClientErrors, IBesuLightClientMsgs, AccessControl {
    using RLP for *;

    /// @notice Decoded fields from a submitted Besu header.
    /// @param headerItems Top-level RLP header fields.
    /// @param extraDataItems Decoded Besu BFT `extraData` fields.
    /// @param height Header block number.
    /// @param stateRoot Header state root.
    /// @param timestamp Header timestamp in seconds.
    /// @param validators Validator set from `extraData`.
    /// @param commitSeals Commit seals from `extraData`.
    struct ParsedHeader {
        Memory.Slice[] headerItems;
        Memory.Slice[] extraDataItems;
        uint64 height;
        bytes32 stateRoot;
        uint64 timestamp;
        address[] validators;
        bytes[] commitSeals;
    }

    /// @notice Role allowed to submit client updates and proof verifications.
    // natlint-disable-next-line MissingInheritdoc
    bytes32 public constant PROOF_SUBMITTER_ROLE = keccak256("PROOF_SUBMITTER_ROLE");

    /// @notice Besu BFT sentinel mix hash.
    bytes32 internal constant BESU_BFT_MIX_HASH = 0x63746963616c2062797a616e74696e65206661756c7420746f6c6572616e6365;
    /// @notice Empty ommers hash required by Besu BFT headers.
    bytes32 internal constant EMPTY_OMMERS_HASH = 0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347;
    /// @notice ERC-7201 storage slot used by `IBCStoreUpgradeable` commitments.
    bytes32 internal constant IBCSTORE_STORAGE_SLOT =
        0x1260944489272988d9df285149b5aa1b0f48f2136d6f416159f840a3e0747600;

    /// @notice Current client state.
    ClientState internal clientState;
    /// @notice Trusted consensus states by revision height.
    mapping(uint64 revisionHeight => ConsensusState) internal consensusStates;

    /// @notice Initializes shared Besu light client state.
    /// @param ibcRouter Counterparty ICS26 router address whose storage is proven.
    /// @param initialTrustedHeight Initial trusted Besu height.
    /// @param initialTrustedTimestamp Initial trusted header timestamp in seconds.
    /// @param initialTrustedStorageRoot Initial trusted storage root of `ibcRouter`.
    /// @param initialTrustedValidators Initial trusted validator set.
    /// @param trustingPeriod Maximum age in seconds for trusted consensus states.
    /// @param maxClockDrift Maximum allowed future drift in seconds for submitted headers.
    /// @param roleManager Address that administers proof submission; if zero, proof submission is open.
    constructor(
        address ibcRouter,
        uint64 initialTrustedHeight,
        uint64 initialTrustedTimestamp,
        bytes32 initialTrustedStorageRoot,
        address[] memory initialTrustedValidators,
        uint64 trustingPeriod,
        uint64 maxClockDrift,
        address roleManager
    ) {
        require(initialTrustedHeight != 0, InvalidHeaderHeight());
        require(initialTrustedTimestamp != 0, InvalidHeaderTimestamp());

        _validateValidators(initialTrustedValidators);

        clientState = ClientState({
            ibcRouter: ibcRouter,
            latestHeight: IICS02ClientMsgs.Height({ revisionNumber: 0, revisionHeight: initialTrustedHeight }),
            trustingPeriod: trustingPeriod,
            maxClockDrift: maxClockDrift
        });

        ConsensusState storage consensusState = consensusStates[initialTrustedHeight];
        consensusState.timestamp = initialTrustedTimestamp;
        consensusState.storageRoot = initialTrustedStorageRoot;
        consensusState.validators = initialTrustedValidators;

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
    function getConsensusState(uint64 revisionHeight) external view returns (bytes memory) {
        ConsensusState storage consensusState = _getConsensusState(revisionHeight);
        return abi.encode(consensusState.timestamp, consensusState.storageRoot, consensusState.validators);
    }

    /// @inheritdoc ILightClient
    function updateClient(bytes calldata updateMsg)
        external
        onlyProofSubmitter
        returns (ILightClientMsgs.UpdateResult)
    {
        MsgUpdateClient memory msg_ = abi.decode(updateMsg, (MsgUpdateClient));
        _requireZeroRevision(msg_.trustedHeight.revisionNumber);

        ParsedHeader memory header = _parseHeader(msg_.headerRlp);
        require(header.height != 0, InvalidHeaderHeight());
        require(header.timestamp != 0, InvalidHeaderTimestamp());
        require(
            block.timestamp + clientState.maxClockDrift >= header.timestamp,
            HeaderFromFuture(block.timestamp, header.timestamp, clientState.maxClockDrift)
        );

        ConsensusState storage trustedConsensusState = _getConsensusState(msg_.trustedHeight.revisionHeight);
        require(
            clientState.trustingPeriod == 0
                || uint256(trustedConsensusState.timestamp) + clientState.trustingPeriod > block.timestamp,
            ConsensusStateExpired(trustedConsensusState.timestamp, block.timestamp, clientState.trustingPeriod)
        );

        address[] memory signers = _recoverSigners(_commitSealDigest(header), header.commitSeals);
        _checkTrustedValidatorOverlap(signers, trustedConsensusState.validators);
        _checkValidatorQuorum(signers, header.validators);

        bytes32 storageRoot = _verifyAccountProof(clientState.ibcRouter, header.stateRoot, msg_.accountProof);

        ConsensusState storage existingConsensusState = consensusStates[header.height];
        if (existingConsensusState.timestamp != 0) {
            if (_isSameConsensusState(existingConsensusState, header.timestamp, storageRoot, header.validators)) {
                return ILightClientMsgs.UpdateResult.NoOp;
            }
            revert ConflictingConsensusState(header.height);
        }

        ConsensusState storage newConsensusState = consensusStates[header.height];
        newConsensusState.timestamp = header.timestamp;
        newConsensusState.storageRoot = storageRoot;
        newConsensusState.validators = header.validators;

        if (header.height > clientState.latestHeight.revisionHeight) {
            clientState.latestHeight.revisionHeight = header.height;
        }

        return ILightClientMsgs.UpdateResult.Update;
    }

    /// @inheritdoc ILightClient
    function verifyMembership(ILightClientMsgs.MsgVerifyMembership calldata msg_)
        external
        view
        onlyProofSubmitter
        returns (uint256)
    {
        _requireZeroRevision(msg_.proofHeight.revisionNumber);
        require(msg_.path.length == 1, InvalidPathLength(1, msg_.path.length));
        require(msg_.value.length == 32, InvalidValueLength(32, msg_.value.length));

        ConsensusState storage consensusState = _getConsensusState(msg_.proofHeight.revisionHeight);
        bytes32 storageSlot = _commitmentStorageSlot(msg_.path[0]);
        bytes[] memory proofNodes = abi.decode(msg_.proof, (bytes[]));
        bytes memory storageKey = abi.encodePacked(keccak256(abi.encodePacked(storageSlot)));

        bytes memory traversedValue = TrieProof.traverse(consensusState.storageRoot, storageKey, proofNodes);
        bytes32 actualValue = traversedValue.decodeBytes32();
        bytes32 expectedValue = bytes32(msg_.value);

        require(actualValue == expectedValue, InvalidCommitmentValue(expectedValue, actualValue));

        return consensusState.timestamp;
    }

    /// @inheritdoc ILightClient
    function verifyNonMembership(ILightClientMsgs.MsgVerifyNonMembership calldata msg_)
        external
        view
        onlyProofSubmitter
        returns (uint256)
    {
        _requireZeroRevision(msg_.proofHeight.revisionNumber);
        require(msg_.path.length == 1, InvalidPathLength(1, msg_.path.length));

        ConsensusState storage consensusState = _getConsensusState(msg_.proofHeight.revisionHeight);
        bytes32 storageSlot = _commitmentStorageSlot(msg_.path[0]);
        bytes[] memory proofNodes = abi.decode(msg_.proof, (bytes[]));
        bytes memory storageKey = abi.encodePacked(keccak256(abi.encodePacked(storageSlot)));

        require(TrieProof.verifyExclusion(consensusState.storageRoot, storageKey, proofNodes), InvalidExclusionProof());

        return consensusState.timestamp;
    }

    /// @inheritdoc ILightClient
    function misbehaviour(bytes calldata) external view onlyProofSubmitter {
        revert UnsupportedMisbehaviour();
    }

    /// @notice Computes the protocol-specific commit seal digest for a parsed header.
    /// @param header The parsed Besu header.
    /// @return The digest signed by commit seals.
    function _commitSealDigest(ParsedHeader memory header) internal pure virtual returns (bytes32);

    /// @notice Parses and validates common Besu BFT header fields.
    /// @param headerRlp RLP-encoded Besu block header.
    /// @return header The parsed header fields used by update and proof verification.
    function _parseHeader(bytes memory headerRlp) internal pure returns (ParsedHeader memory header) {
        header.headerItems = headerRlp.decodeList();
        require(header.headerItems.length >= 15, InvalidHeaderFormat(header.headerItems.length));

        require(
            header.headerItems[1].readBytes32() == EMPTY_OMMERS_HASH,
            InvalidOmmersHash(header.headerItems[1].readBytes32())
        );
        require(header.headerItems[7].readUint256() == 1, InvalidDifficulty(header.headerItems[7].readUint256()));
        require(
            header.headerItems[13].readBytes32() == BESU_BFT_MIX_HASH,
            InvalidMixHash(header.headerItems[13].readBytes32())
        );

        bytes memory nonce = header.headerItems[14].readBytes();
        require(nonce.length == 8 && keccak256(nonce) == keccak256(hex"0000000000000000"), InvalidNonce(nonce));

        header.height = uint64(header.headerItems[8].readUint256());
        header.stateRoot = header.headerItems[3].readBytes32();
        header.timestamp = uint64(header.headerItems[11].readUint256());

        bytes memory extraData = header.headerItems[12].readBytes();
        header.extraDataItems = extraData.decodeList();
        require(header.extraDataItems.length == 5, InvalidExtraDataFormat(header.extraDataItems.length));

        Memory.Slice[] memory validatorItems = header.extraDataItems[1].readList();
        require(validatorItems.length != 0, EmptyValidatorSet());

        header.validators = new address[](validatorItems.length);
        for (uint256 i = 0; i < validatorItems.length; ++i) {
            bytes memory validatorBytes = validatorItems[i].readBytes();
            require(validatorBytes.length == 20, InvalidValidatorAddressLength(validatorBytes.length));

            address validator = address(bytes20(validatorBytes));
            require(validator != address(0), InvalidValidatorAddress(address(0)));
            for (uint256 j = 0; j < i; ++j) {
                require(header.validators[j] != validator, DuplicateValidator(validator));
            }
            header.validators[i] = validator;
        }

        Memory.Slice[] memory sealItems = header.extraDataItems[4].readList();
        header.commitSeals = new bytes[](sealItems.length);
        for (uint256 i = 0; i < sealItems.length; ++i) {
            header.commitSeals[i] = sealItems[i].readBytes();
        }
    }

    /// @notice Verifies the tracked account proof against a header state root.
    /// @param account The account address being proven.
    /// @param stateRoot The header state root.
    /// @param accountProof ABI-encoded account proof nodes (`abi.encode(bytes[])`).
    /// @return The proven account storage root.
    function _verifyAccountProof(
        address account,
        bytes32 stateRoot,
        bytes memory accountProof
    )
        internal
        pure
        returns (bytes32)
    {
        bytes memory accountKey = abi.encodePacked(keccak256(abi.encodePacked(account)));
        bytes[] memory proofNodes = abi.decode(accountProof, (bytes[]));
        bytes memory accountRlp = TrieProof.traverse(stateRoot, accountKey, proofNodes);
        Memory.Slice[] memory accountItems = accountRlp.decodeList();
        return accountItems[2].readBytes32();
    }

    /// @notice Computes the storage slot used for an IBC commitment path.
    /// @param rawPath Raw commitment path bytes.
    /// @return The storage slot for the commitment.
    function _commitmentStorageSlot(bytes memory rawPath) internal pure returns (bytes32) {
        return keccak256(abi.encode(keccak256(rawPath), IBCSTORE_STORAGE_SLOT));
    }

    /// @notice Recovers unique commit seal signers for a digest.
    /// @param digest The commit seal digest.
    /// @param seals The commit seals to recover.
    /// @return signers The recovered signer addresses.
    function _recoverSigners(bytes32 digest, bytes[] memory seals) internal pure returns (address[] memory signers) {
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
    function _recoverSigner(bytes32 digest, bytes memory seal) internal pure returns (address) {
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
    function _checkTrustedValidatorOverlap(
        address[] memory signers,
        address[] storage trustedValidators
    )
        internal
        view
    {
        uint256 actual = 0;
        for (uint256 i = 0; i < signers.length; ++i) {
            if (_containsStorage(trustedValidators, signers[i])) {
                ++actual;
            }
        }

        uint256 required = trustedValidators.length / 3 + 1;
        require(actual >= required, InsufficientTrustedValidatorOverlap(actual, required));
    }

    /// @notice Checks that signers meet quorum for the submitted header validator set.
    /// @param signers The recovered commit seal signers.
    /// @param validators The validator set from the submitted header.
    function _checkValidatorQuorum(address[] memory signers, address[] memory validators) internal pure {
        uint256 actual = 0;
        for (uint256 i = 0; i < signers.length; ++i) {
            if (_containsMemory(validators, signers[i])) {
                ++actual;
            }
        }

        // Besu requires ceil(2n / 3), equivalently n - floor(n / 3).
        uint256 required = validators.length - validators.length / 3;
        require(actual >= required, InsufficientValidatorQuorum(actual, required));
    }

    /// @notice Validates that a validator set is non-empty and unique.
    /// @param validators The validator set to validate.
    function _validateValidators(address[] memory validators) internal pure {
        require(validators.length != 0, EmptyValidatorSet());
        for (uint256 i = 0; i < validators.length; ++i) {
            require(validators[i] != address(0), InvalidValidatorAddress(validators[i]));
            for (uint256 j = 0; j < i; ++j) {
                require(validators[j] != validators[i], DuplicateValidator(validators[i]));
            }
        }
    }

    /// @notice Compares a stored consensus state with proposed fields.
    /// @param consensusState The stored consensus state.
    /// @param timestamp The proposed timestamp.
    /// @param storageRoot The proposed storage root.
    /// @param validators The proposed validator set.
    /// @return True if all fields match.
    function _isSameConsensusState(
        ConsensusState storage consensusState,
        uint64 timestamp,
        bytes32 storageRoot,
        address[] memory validators
    )
        internal
        view
        returns (bool)
    {
        if (consensusState.timestamp != timestamp || consensusState.storageRoot != storageRoot) {
            return false;
        }
        return keccak256(abi.encode(consensusState.validators)) == keccak256(abi.encode(validators));
    }

    /// @notice Returns a stored consensus state or reverts if it is missing.
    /// @param revisionHeight The consensus state revision height.
    /// @return consensusState The stored consensus state.
    function _getConsensusState(uint64 revisionHeight) internal view returns (ConsensusState storage consensusState) {
        consensusState = consensusStates[revisionHeight];
        require(consensusState.timestamp != 0, ConsensusStateNotFound(revisionHeight));
    }

    /// @notice Reverts unless the revision number is zero.
    /// @param revisionNumber The revision number to validate.
    function _requireZeroRevision(uint64 revisionNumber) internal pure {
        require(revisionNumber == 0, InvalidRevisionNumber(revisionNumber));
    }

    /// @notice Checks whether a storage validator set contains a signer.
    /// @param validators The storage validator set.
    /// @param signer The signer to find.
    /// @return True if `signer` is present.
    function _containsStorage(address[] storage validators, address signer) internal view returns (bool) {
        for (uint256 i = 0; i < validators.length; ++i) {
            if (validators[i] == signer) {
                return true;
            }
        }
        return false;
    }

    /// @notice Checks whether a memory validator set contains a signer.
    /// @param validators The memory validator set.
    /// @param signer The signer to find.
    /// @return True if `signer` is present.
    function _containsMemory(address[] memory validators, address signer) internal pure returns (bool) {
        for (uint256 i = 0; i < validators.length; ++i) {
            if (validators[i] == signer) {
                return true;
            }
        }
        return false;
    }

    /// @notice Restricts access to proof submitters unless submission is open to anyone.
    modifier onlyProofSubmitter() {
        if (!hasRole(PROOF_SUBMITTER_ROLE, address(0))) {
            _checkRole(PROOF_SUBMITTER_ROLE);
        }
        _;
    }
}
