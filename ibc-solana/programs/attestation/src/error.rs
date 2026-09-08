// SPDX-License-Identifier: Apache-2.0

use anchor_lang::prelude::*;

#[error_code]
pub enum ErrorCode {
    #[msg("Client state is frozen")]
    FrozenClientState,

    #[msg("No attestors provided")]
    NoAttestors,

    #[msg("Min required signatures must be positive and not exceed attestor count")]
    BadQuorum,

    #[msg("Height cannot be zero")]
    InvalidHeight,

    #[msg("Timestamp cannot be zero")]
    InvalidTimestamp,

    #[msg("Height and timestamp must be non-zero")]
    InvalidState,

    #[msg("Expected height does not match proof height")]
    HeightMismatch,

    #[msg("Proof validation failed")]
    InvalidProof,

    #[msg("Signature recovery failed")]
    InvalidSignature,

    #[msg("Address not in trusted attestor set")]
    UnknownSigner,

    #[msg("Minimum required signatures not provided")]
    ThresholdNotMet,

    #[msg("Same signer provided multiple times")]
    DuplicateSigner,

    #[msg("Failed to decode attestation data")]
    InvalidAttestationData,

    #[msg("Commitment path not in attestation")]
    NotMember,

    #[msg("Value does not match attested commitment")]
    CommitmentMismatch,

    #[msg("Packet commitment is empty")]
    ZeroCommitment,

    #[msg("Expected zero commitment for non-membership")]
    NonZeroCommitment,

    #[msg("Membership proof value cannot be empty")]
    EmptyValue,

    #[msg("Expected path length of 1")]
    InvalidPathLength,

    #[msg("Caller does not have required role")]
    UnauthorizedRole,

    #[msg("Failed to serialize/deserialize data")]
    SerializationError,

    #[msg("Arithmetic overflow")]
    ArithmeticOverflow,

    #[msg("No packets in attestation data")]
    EmptyAttestation,

    #[msg("No signatures provided")]
    EmptySignatures,

    #[msg("Conflicting timestamp for existing height")]
    Misbehaviour,

    #[msg("No trusted timestamp at requested height")]
    ConsensusTimestampNotFound,

    #[msg("Invalid access manager")]
    InvalidAccessManager,

    #[msg("Only the program's upgrade authority can call initialize")]
    UnauthorizedDeployer,
}
