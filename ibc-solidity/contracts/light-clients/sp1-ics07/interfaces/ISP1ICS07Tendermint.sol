// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

import { ISP1Verifier } from "@sp1-contracts/ISP1Verifier.sol";

/// @title ISP1ICS07Tendermint
/// @notice ISP1ICS07Tendermint is the interface for the ICS07 Tendermint light client
interface ISP1ICS07Tendermint {
    /// @notice The role identifier for the proof submitter role
    /// @dev The proof submitter role is used to whitelist addresses that can submit proofs
    /// @dev If `address(0)` has this role, then anyone can submit proofs
    /// @dev If this client is hooked up to ICS26Router, the router must be given this role
    /// @return The role identifier
    function PROOF_SUBMITTER_ROLE() external view returns (bytes32);

    /// @notice Immutable update client program verification key.
    /// @return The verification key for the update client program.
    function UPDATE_CLIENT_PROGRAM_VKEY() external view returns (bytes32);

    /// @notice Immutable membership program verification key.
    /// @return The verification key for the membership program.
    function MEMBERSHIP_PROGRAM_VKEY() external view returns (bytes32);

    /// @notice Immutable update client and membership program verification key.
    /// @return The verification key for the update client and membership program.
    function UPDATE_CLIENT_AND_MEMBERSHIP_PROGRAM_VKEY() external view returns (bytes32);

    /// @notice Immutable misbehaviour program verification key.
    /// @return The verification key for the misbehaviour program.
    function MISBEHAVIOUR_PROGRAM_VKEY() external view returns (bytes32);

    /// @notice Immutable SP1 verifier contract address.
    /// @return The SP1 verifier contract.
    function VERIFIER() external view returns (ISP1Verifier);

    /// @notice Constant allowed prover clock drift in seconds.
    /// @return The allowed prover clock drift in seconds.
    function ALLOWED_SP1_CLOCK_DRIFT() external view returns (uint16);

    /// @notice Returns the consensus state keccak256 hash at the given revision height.
    /// @param revisionHeight The revision height.
    /// @return The consensus state at the given revision height.
    function getConsensusStateHash(uint64 revisionHeight) external view returns (bytes32);
}
