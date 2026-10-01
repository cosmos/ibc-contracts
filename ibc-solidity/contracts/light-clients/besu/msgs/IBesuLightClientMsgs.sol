// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { IICS02ClientMsgs } from "../../../msgs/IICS02ClientMsgs.sol";

/// @title Besu Light Client Messages
/// @notice Defines shared message and state types for Besu BFT light clients.
interface IBesuLightClientMsgs {
    /// @notice Fraction of the trusted validator set that must sign a new header.
    /// @param numerator Numerator of the fraction.
    /// @param denominator Denominator of the fraction.
    struct TrustThreshold {
        uint8 numerator;
        uint8 denominator;
    }

    /// @notice Client state for a Besu BFT light client.
    /// @param ibcRouter Counterparty ICS26 router address whose storage is proven.
    /// @param latestHeight Latest trusted Besu height.
    /// @param trustingPeriod Maximum age in seconds for a trusted consensus state.
    /// @param maxClockDrift Maximum allowed future drift in seconds for submitted headers.
    /// @param isFrozen Whether the client has been permanently frozen due to misbehaviour.
    /// @param trustLevel Minimum fraction of the trusted validator set that must sign a new header.
    struct ClientState {
        address ibcRouter;
        IICS02ClientMsgs.Height latestHeight;
        uint64 trustingPeriod;
        uint64 maxClockDrift;
        bool isFrozen;
        TrustThreshold trustLevel;
    }

    /// @notice Trusted consensus state for a Besu height.
    /// @param timestamp Header timestamp in seconds.
    /// @param stateRoot Root of the state trie at the given height.
    /// @param validators Validator set committed in the header.
    struct ConsensusState {
        uint64 timestamp;
        bytes32 stateRoot;
        address[] validators;
    }

    /// @notice Update message containing a Besu header and the trusted consensus state preimage.
    /// @param headerRlp RLP-encoded Besu block header.
    /// @param trustedHeight Previously trusted height used for weak-subjectivity checks.
    /// @param consensusStatePreimage Preimage of the trusted consensus state at `trustedHeight`.
    struct MsgUpdateClient {
        bytes headerRlp;
        IICS02ClientMsgs.Height trustedHeight;
        ConsensusState consensusStatePreimage;
    }

    /// @notice Storage proof against the tracked router account, used for membership and non-membership.
    /// @param consensusStatePreimage Preimage of the trusted consensus state at `msg_.proofHeight`.
    /// @param accountProofNodes Ordered, RLP-encoded MPT nodes from the state trie proving the tracked router account.
    /// @param proofNodes Ordered, RLP-encoded MPT nodes from the router account storage trie.
    struct MembershipProof {
        ConsensusState consensusStatePreimage;
        bytes[] accountProofNodes;
        bytes[] proofNodes;
    }

    /// @notice Misbehaviour message containing two conflicting trusted consensus states at different heights.
    /// @dev Time monotonicity check is done during updateClient, however, it is possible that the check is bypassed if
    /// a suitable trusted consensus state is provided. This misbehavior message is used to freeze the client by simply
    /// providing two consensus states that are trusted but have timestamps that are not monotonic. The states only need
    /// to match their stored hashes; they may be past their trusting period.
    /// @param height1 Height of the first trusted consensus state.
    /// @param height2 Height of the second trusted consensus state. Must differ from `height1`; order is irrelevant.
    /// @param consensusStatePreimage1 Preimage of the first trusted consensus state.
    /// @param consensusStatePreimage2 Preimage of the second trusted consensus state.
    struct MsgTimeNonMonotonicityMisbehaviour {
        IICS02ClientMsgs.Height height1;
        IICS02ClientMsgs.Height height2;
        ConsensusState consensusStatePreimage1;
        ConsensusState consensusStatePreimage2;
    }

    /// @notice Misbehaviour message containing two validly signed headers that conflict with each other.
    /// @dev Each header is verified against its own trusted consensus state exactly as in `updateClient`, except that
    /// the clock drift check is skipped. Headers at the same height with different consensus states prove a double
    /// sign. Headers at different heights where the lower header's timestamp is not less than the higher header's
    /// timestamp prove time non-monotonicity. The headers may be in either order and need not be stored.
    /// @param update1 The first header with its trusted consensus state.
    /// @param update2 The second header with its trusted consensus state.
    struct MsgHeadersMisbehaviour {
        MsgUpdateClient update1;
        MsgUpdateClient update2;
    }

    /// @notice The type of misbehaviour evidence in a `MsgSubmitMisbehaviour`.
    enum MisbehaviourType {
        /// The evidence is an `abi.encode(MsgTimeNonMonotonicityMisbehaviour)`.
        TimeNonMonotonicity,
        /// The evidence is an `abi.encode(MsgHeadersMisbehaviour)`.
        Headers
    }

    /// @notice Misbehaviour message accepted by `misbehaviour(bytes)`.
    /// @param misbehaviourType The type of the encoded evidence.
    /// @param misbehaviour The ABI-encoded evidence message for `misbehaviourType`.
    struct MsgSubmitMisbehaviour {
        MisbehaviourType misbehaviourType;
        bytes misbehaviour;
    }
}
