// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { IICS02ClientMsgs } from "../../../msgs/IICS02ClientMsgs.sol";

/// @title Besu Light Client Messages
/// @notice Defines shared message and state types for Besu BFT light clients.
interface IBesuLightClientMsgs {
    /// @notice Client state for a Besu BFT light client.
    /// @param ibcRouter Counterparty ICS26 router address whose storage is proven.
    /// @param latestHeight Latest trusted Besu height.
    /// @param trustingPeriod Maximum age in seconds for a trusted consensus state.
    /// @param maxClockDrift Maximum allowed future drift in seconds for submitted headers.
    /// @param isFrozen Whether the client has been permanently frozen due to misbehaviour.
    struct ClientState {
        address ibcRouter;
        IICS02ClientMsgs.Height latestHeight;
        uint64 trustingPeriod;
        uint64 maxClockDrift;
        bool isFrozen;
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
}
