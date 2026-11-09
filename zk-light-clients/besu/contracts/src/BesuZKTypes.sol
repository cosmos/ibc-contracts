// SPDX-License-Identifier: Apache-2.0
pragma solidity 0.8.28;

/// @notice Canonical on-chain and SP1 public-value schemas for the Besu QBFT light client.
library BesuZKTypes {
    uint32 internal constant SCHEMA_VERSION = 1;
    uint32 internal constant MEMBERSHIP_SCHEMA_VERSION = 2;
    uint32 internal constant MAX_BATCH_MESSAGES = 32;
    uint8 internal constant MEMBERSHIP_ENVELOPE_FULL = 1;
    uint8 internal constant MEMBERSHIP_ENVELOPE_CACHED = 2;

    /// @dev `abi.encode(ClientState)` is the canonical `getClientState()` representation.
    struct ClientState {
        uint64 latestHeight;
        uint64 trustingPeriodSeconds;
        uint64 maxClockDriftSeconds;
    }

    struct ConsensusState {
        bytes32 stateRoot;
        uint64 blockTimestamp;
        bytes32 validatorSetHash;
    }

    struct UpdatePublicValues {
        uint32 schemaVersion;
        uint64 trustedHeight;
        bytes32 trustedValidatorSetHash;
        uint64 verifiedHeight;
        bytes32 stateRoot;
        uint64 blockTimestamp;
        bytes32 newValidatorSetHash;
    }

    struct MembershipPublicValues {
        uint32 schemaVersion;
        uint64 proofHeight;
        bytes32 stateRoot;
        address sourceRouter;
        bytes32 batchRoot;
        uint32 entryCount;
    }

    struct NonMembershipPublicValues {
        uint32 schemaVersion;
        uint64 proofHeight;
        bytes32 stateRoot;
        address sourceRouter;
        bytes32 pathDigest;
    }

    struct SP1ProofEnvelope {
        bytes publicValues;
        bytes proofBytes;
    }

    struct MembershipBatchProofEnvelope {
        uint8 mode;
        bytes publicValues;
        bytes proofBytes;
        bytes32 batchRoot;
        uint32 entryCount;
        uint32 leafIndex;
        bytes32[] merkleProof;
    }
}
