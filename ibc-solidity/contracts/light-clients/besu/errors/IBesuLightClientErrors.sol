// SPDX-License-Identifier: MIT
pragma solidity ^0.8.28;

/// @title Besu Light Client Errors
/// @notice Error interface for Besu BFT light clients.
interface IBesuLightClientErrors {
    /// @notice The revision number must be zero for Besu light client heights.
    /// @param providedRevisionNumber The non-zero revision number.
    error InvalidRevisionNumber(uint64 providedRevisionNumber);
    /// @notice The submitted header has an invalid RLP item count.
    /// @param itemsLength The decoded header item count.
    error InvalidHeaderFormat(uint256 itemsLength);
    /// @notice The submitted `extraData` field has an invalid RLP item count.
    /// @param itemsLength The decoded `extraData` item count.
    error InvalidExtraDataFormat(uint256 itemsLength);
    /// @notice The submitted header height is zero.
    error InvalidHeaderHeight();
    /// @notice The submitted header timestamp is too far in the future.
    /// @param currentTimestamp The current block timestamp.
    /// @param headerTimestamp The submitted header timestamp.
    /// @param maxClockDrift The configured maximum clock drift.
    error HeaderFromFuture(uint256 currentTimestamp, uint256 headerTimestamp, uint256 maxClockDrift);
    /// @notice No consensus state exists for the requested height.
    /// @param revisionHeight The requested revision height.
    error ConsensusStateNotFound(uint64 revisionHeight);
    /// @notice The trusted consensus state is outside the trusting period.
    /// @param trustedTimestamp Timestamp of the trusted consensus state.
    /// @param currentTimestamp The current block timestamp.
    /// @param trustingPeriod The configured trusting period.
    error ConsensusStateExpired(uint64 trustedTimestamp, uint256 currentTimestamp, uint64 trustingPeriod);
    /// @notice The Besu BFT mix hash is invalid.
    /// @param actualMixHash The mix hash found in the header.
    error InvalidMixHash(bytes32 actualMixHash);
    /// @notice The Besu BFT difficulty is invalid.
    /// @param actualDifficulty The difficulty found in the header.
    error InvalidDifficulty(uint256 actualDifficulty);
    /// @notice The Besu BFT nonce is invalid.
    /// @param actualNonce The nonce found in the header.
    error InvalidNonce(bytes actualNonce);
    /// @notice The ommers hash is invalid.
    /// @param actualOmmersHash The ommers hash found in the header.
    error InvalidOmmersHash(bytes32 actualOmmersHash);
    /// @notice The validator set is empty.
    error EmptyValidatorSet();
    /// @notice A validator address has an invalid byte length.
    /// @param length The decoded validator address length.
    error InvalidValidatorAddressLength(uint256 length);
    /// @notice A validator address is invalid.
    /// @param validator The invalid validator address.
    error InvalidValidatorAddress(address validator);
    /// @notice A validator appears more than once.
    /// @param validator The duplicate validator address.
    error DuplicateValidator(address validator);
    /// @notice A commit seal signer appears more than once.
    /// @param signer The duplicate signer address.
    error DuplicateCommitSealSigner(address signer);
    /// @notice An ECDSA signature has an invalid length.
    /// @param length The invalid signature length.
    error InvalidECDSASignatureLength(uint256 length);
    /// @notice A commit seal did not recover a valid signer.
    error InvalidCommitSeal();
    /// @notice The submitted signers do not overlap enough with the trusted validator set.
    /// @param actual The number of trusted validators that signed.
    /// @param required The minimum required trusted signer count.
    error InsufficientTrustedValidatorOverlap(uint256 actual, uint256 required);
    /// @notice The submitted signers do not meet quorum for the new validator set.
    /// @param actual The number of new validators that signed.
    /// @param required The minimum required signer count.
    error InsufficientValidatorQuorum(uint256 actual, uint256 required);
    /// @notice A proof path has an invalid length.
    /// @param expectedLength The expected path length.
    /// @param actualLength The actual path length.
    error InvalidPathLength(uint256 expectedLength, uint256 actualLength);
    /// @notice A membership value has an invalid length.
    /// @param expectedLength The expected value length.
    /// @param actualLength The actual value length.
    error InvalidValueLength(uint256 expectedLength, uint256 actualLength);
    /// @notice A proven commitment value does not match the expected value.
    /// @param expectedValue The expected commitment value.
    /// @param actualValue The proven commitment value.
    error InvalidCommitmentValue(bytes32 expectedValue, bytes32 actualValue);
    /// @notice A non-membership proof found an existing value.
    /// @param actualValue The value found at the proven storage slot.
    error ValueExists(bytes32 actualValue);
    /// @notice A different consensus state already exists at the submitted height.
    /// @param revisionHeight The conflicting revision height.
    error ConflictingConsensusState(uint64 revisionHeight);
    /// @notice Misbehaviour handling is not supported by this client.
    error UnsupportedMisbehaviour();
}
