// SPDX-License-Identifier: Apache-2.0
pragma solidity ^0.8.28;

/// @title IAttestationLightClientErrors
/// @notice Error interface for the attestation-based IBC light client
interface IAttestationLightClientErrors {
    /// @notice Reverts when the attestation set is empty during initialization.
    error NoAttestors();
    /// @notice Reverts when the quorum threshold is zero or exceeds the attestation count.
    /// @param minRequired The provided minimum required signatures.
    /// @param attestationCount The size of the configured attestation set.
    error BadQuorum(uint8 minRequired, uint256 attestationCount);
    /// @notice Reverts when an action is attempted on a frozen client state.
    error FrozenClientState();
    /// @notice Reverts for functions that are out of scope for this implementation.
    error FeatureNotSupported();
    /// @notice Missing trusted timestamp for the given height.
    /// @param height The height that has no associated timestamp.
    error ConsensusTimestampNotFound(uint64 height);
    /// @notice ECDSA signature has an invalid length.
    /// @param signature The invalid signature.
    error InvalidSignatureLength(bytes signature);
    /// @notice ECDSA signature failed to recover a valid signer.
    /// @param signature The invalid signature.
    error SignatureInvalid(bytes signature);
    /// @notice The recovered signer is not part of the attestation set.
    /// @param signer Address of the unknown signer.
    error UnknownSigner(address signer);
    /// @notice The recovered signer appears more than once among the signatures.
    /// @param signer Address of the duplicate signer.
    error DuplicateSigner(address signer);
    /// @notice The number of valid unique signatures is below the required threshold.
    /// @param validSigners Number of valid, unique attestation signatures.
    /// @param minRequired The minimum required signatures.
    error ThresholdNotMet(uint256 validSigners, uint8 minRequired);
    /// @notice The provided membership value was empty.
    error EmptyValue();
    /// @notice The provided packet attestations were empty.
    error EmptyPackets();
    /// @notice The provided proof path length is invalid.
    /// @param expectedLength The expected length of the path.
    /// @param providedLength The length of the provided path.
    error InvalidPathLength(uint256 expectedLength, uint256 providedLength);
    /// @notice The provided value is not a member of the attested set.
    error NotMember();
    /// @notice The attested commitment is not zero (receipt exists, so packet was received).
    error NotNonMember();
    /// @notice The attested height did not match the provided proof height.
    /// @param expected The expected height from the proofHeight field.
    /// @param provided The height that was present in the attested payload.
    error HeightMismatch(uint64 expected, uint64 provided);
    /// @notice The attested state is invalid.
    /// @param height The height of the attested state.
    /// @param timestamp The timestamp of the attested state.
    error InvalidState(uint64 height, uint64 timestamp);
    /// @notice The provided signatures were empty.
    error EmptySignatures();
}
