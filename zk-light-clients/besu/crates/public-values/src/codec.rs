use alloy_primitives::Bytes;
use alloy_sol_types::{sol, SolValue};
use besu_zk_types::{
    ClientState, MembershipBatchProofEnvelope, MembershipPublicValues, NonMembershipPublicValues,
    SP1ProofEnvelope, UpdatePublicValues, MEMBERSHIP_ENVELOPE_CACHED, MEMBERSHIP_ENVELOPE_FULL,
    MEMBERSHIP_SCHEMA_VERSION, SCHEMA_VERSION,
};
use core::fmt;

sol! {
    struct SolClientState {
        uint64 latestHeight;
        uint64 trustingPeriodSeconds;
        uint64 maxClockDriftSeconds;
    }

    struct SolUpdatePublicValues {
        uint32 schemaVersion;
        uint64 trustedHeight;
        bytes32 trustedValidatorSetHash;
        uint64 verifiedHeight;
        bytes32 stateRoot;
        uint64 blockTimestamp;
        bytes32 newValidatorSetHash;
    }

    struct SolMembershipPublicValues {
        uint32 schemaVersion;
        uint64 proofHeight;
        bytes32 stateRoot;
        address sourceRouter;
        bytes32 batchRoot;
        uint32 entryCount;
    }

    struct SolNonMembershipPublicValues {
        uint32 schemaVersion;
        uint64 proofHeight;
        bytes32 stateRoot;
        address sourceRouter;
        bytes32 pathDigest;
    }

    struct SolSP1ProofEnvelope {
        bytes publicValues;
        bytes proofBytes;
    }

    struct SolMembershipBatchProofEnvelope {
        uint8 mode;
        bytes publicValues;
        bytes proofBytes;
        bytes32 batchRoot;
        uint32 entryCount;
        uint32 leafIndex;
        bytes32[] merkleProof;
    }
}

#[must_use]
pub fn encode_client_state(value: &ClientState) -> Vec<u8> {
    SolClientState {
        latestHeight: value.latest_height,
        trustingPeriodSeconds: value.trusting_period_seconds,
        maxClockDriftSeconds: value.max_clock_drift_seconds,
    }
    .abi_encode()
}

pub fn decode_client_state(bytes: &[u8]) -> Result<ClientState, PublicValuesError> {
    let decoded = SolClientState::abi_decode(bytes).map_err(|_| PublicValuesError::MalformedAbi)?;
    require_canonical(bytes, &decoded.abi_encode())?;
    Ok(ClientState {
        latest_height: decoded.latestHeight,
        trusting_period_seconds: decoded.trustingPeriodSeconds,
        max_clock_drift_seconds: decoded.maxClockDriftSeconds,
    })
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PublicValuesError {
    MalformedAbi,
    NonCanonicalAbi,
    UnsupportedSchema(u32),
    EmptyPublicValues,
    EmptyProof,
    UnsupportedMembershipMode(u8),
    InvalidMembershipModePayload,
}

impl fmt::Display for PublicValuesError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MalformedAbi => formatter.write_str("malformed public-values ABI"),
            Self::NonCanonicalAbi => formatter.write_str("non-canonical public-values ABI"),
            Self::UnsupportedSchema(version) => {
                write!(formatter, "unsupported public-values schema {version}")
            }
            Self::EmptyPublicValues => formatter.write_str("empty public values"),
            Self::EmptyProof => formatter.write_str("empty proof"),
            Self::UnsupportedMembershipMode(mode) => write!(formatter, "unsupported membership envelope mode {mode}"),
            Self::InvalidMembershipModePayload => formatter.write_str("invalid payload for membership envelope mode"),
        }
    }
}

#[must_use]
pub fn encode_membership_envelope(value: &MembershipBatchProofEnvelope) -> Vec<u8> {
    SolMembershipBatchProofEnvelope {
        mode: value.mode,
        publicValues: Bytes::copy_from_slice(&value.public_values),
        proofBytes: Bytes::copy_from_slice(&value.proof_bytes),
        batchRoot: value.batch_root.into(),
        entryCount: value.entry_count,
        leafIndex: value.leaf_index,
        merkleProof: value.merkle_proof.iter().copied().map(Into::into).collect(),
    }.abi_encode()
}

pub fn decode_membership_envelope(bytes: &[u8]) -> Result<MembershipBatchProofEnvelope, PublicValuesError> {
    let decoded = SolMembershipBatchProofEnvelope::abi_decode(bytes)
        .map_err(|_| PublicValuesError::MalformedAbi)?;
    require_canonical(bytes, &decoded.abi_encode())?;
    match decoded.mode {
        MEMBERSHIP_ENVELOPE_FULL if decoded.publicValues.is_empty() => return Err(PublicValuesError::EmptyPublicValues),
        MEMBERSHIP_ENVELOPE_FULL if decoded.proofBytes.is_empty() => return Err(PublicValuesError::EmptyProof),
        MEMBERSHIP_ENVELOPE_FULL => {}
        MEMBERSHIP_ENVELOPE_CACHED if !decoded.publicValues.is_empty() || !decoded.proofBytes.is_empty() => {
            return Err(PublicValuesError::InvalidMembershipModePayload);
        }
        MEMBERSHIP_ENVELOPE_CACHED => {}
        mode => return Err(PublicValuesError::UnsupportedMembershipMode(mode)),
    }
    Ok(MembershipBatchProofEnvelope {
        mode: decoded.mode,
        public_values: decoded.publicValues.to_vec(),
        proof_bytes: decoded.proofBytes.to_vec(),
        batch_root: decoded.batchRoot.into(),
        entry_count: decoded.entryCount,
        leaf_index: decoded.leafIndex,
        merkle_proof: decoded.merkleProof.into_iter().map(Into::into).collect(),
    })
}

#[must_use]
pub fn encode_proof_envelope(value: &SP1ProofEnvelope) -> Vec<u8> {
    SolSP1ProofEnvelope {
        publicValues: Bytes::copy_from_slice(&value.public_values),
        proofBytes: Bytes::copy_from_slice(&value.proof_bytes),
    }
    .abi_encode()
}

pub fn decode_proof_envelope(bytes: &[u8]) -> Result<SP1ProofEnvelope, PublicValuesError> {
    let decoded =
        SolSP1ProofEnvelope::abi_decode(bytes).map_err(|_| PublicValuesError::MalformedAbi)?;
    require_canonical(bytes, &decoded.abi_encode())?;
    if decoded.publicValues.is_empty() {
        return Err(PublicValuesError::EmptyPublicValues);
    }
    if decoded.proofBytes.is_empty() {
        return Err(PublicValuesError::EmptyProof);
    }
    Ok(SP1ProofEnvelope {
        public_values: decoded.publicValues.to_vec(),
        proof_bytes: decoded.proofBytes.to_vec(),
    })
}

impl core::error::Error for PublicValuesError {}

#[must_use]
pub fn encode_update(value: &UpdatePublicValues) -> Vec<u8> {
    SolUpdatePublicValues {
        schemaVersion: value.schema_version,
        trustedHeight: value.trusted_height,
        trustedValidatorSetHash: value.trusted_validator_set_hash.into(),
        verifiedHeight: value.verified_height,
        stateRoot: value.state_root.into(),
        blockTimestamp: value.block_timestamp,
        newValidatorSetHash: value.new_validator_set_hash.into(),
    }
    .abi_encode()
}

pub fn decode_update(bytes: &[u8]) -> Result<UpdatePublicValues, PublicValuesError> {
    let decoded =
        SolUpdatePublicValues::abi_decode(bytes).map_err(|_| PublicValuesError::MalformedAbi)?;
    require_canonical(bytes, &decoded.abi_encode())?;
    require_schema(decoded.schemaVersion)?;
    Ok(UpdatePublicValues {
        schema_version: decoded.schemaVersion,
        trusted_height: decoded.trustedHeight,
        trusted_validator_set_hash: decoded.trustedValidatorSetHash.into(),
        verified_height: decoded.verifiedHeight,
        state_root: decoded.stateRoot.into(),
        block_timestamp: decoded.blockTimestamp,
        new_validator_set_hash: decoded.newValidatorSetHash.into(),
    })
}

#[must_use]
pub fn encode_membership(value: &MembershipPublicValues) -> Vec<u8> {
    let mut encoded = Vec::with_capacity(6 * 32);
    append_u32_word(&mut encoded, value.schema_version);
    append_u64_word(&mut encoded, value.proof_height);
    encoded.extend_from_slice(&value.state_root);
    append_address_word(&mut encoded, &value.source_router);
    encoded.extend_from_slice(&value.batch_root);
    append_u32_word(&mut encoded, value.entry_count);
    encoded
}

pub fn decode_membership(bytes: &[u8]) -> Result<MembershipPublicValues, PublicValuesError> {
    let decoded = SolMembershipPublicValues::abi_decode(bytes)
        .map_err(|_| PublicValuesError::MalformedAbi)?;
    require_canonical(bytes, &decoded.abi_encode())?;
    require_membership_schema(decoded.schemaVersion)?;
    Ok(MembershipPublicValues {
        schema_version: decoded.schemaVersion,
        proof_height: decoded.proofHeight,
        state_root: decoded.stateRoot.into(),
        source_router: decoded.sourceRouter.into_array(),
        batch_root: decoded.batchRoot.into(),
        entry_count: decoded.entryCount,
    })
}

#[must_use]
pub fn encode_non_membership(value: &NonMembershipPublicValues) -> Vec<u8> {
    let mut encoded = Vec::with_capacity(5 * 32);
    append_u32_word(&mut encoded, value.schema_version);
    append_u64_word(&mut encoded, value.proof_height);
    encoded.extend_from_slice(&value.state_root);
    append_address_word(&mut encoded, &value.source_router);
    encoded.extend_from_slice(&value.path_digest);
    encoded
}

pub fn decode_non_membership(bytes: &[u8]) -> Result<NonMembershipPublicValues, PublicValuesError> {
    let decoded = SolNonMembershipPublicValues::abi_decode(bytes)
        .map_err(|_| PublicValuesError::MalformedAbi)?;
    require_canonical(bytes, &decoded.abi_encode())?;
    require_schema(decoded.schemaVersion)?;
    Ok(NonMembershipPublicValues {
        schema_version: decoded.schemaVersion,
        proof_height: decoded.proofHeight,
        state_root: decoded.stateRoot.into(),
        source_router: decoded.sourceRouter.into_array(),
        path_digest: decoded.pathDigest.into(),
    })
}

fn require_schema(version: u32) -> Result<(), PublicValuesError> {
    if version == SCHEMA_VERSION {
        Ok(())
    } else {
        Err(PublicValuesError::UnsupportedSchema(version))
    }
}

fn require_membership_schema(version: u32) -> Result<(), PublicValuesError> {
    if version == MEMBERSHIP_SCHEMA_VERSION { Ok(()) } else { Err(PublicValuesError::UnsupportedSchema(version)) }
}

fn append_u32_word(encoded: &mut Vec<u8>, value: u32) {
    encoded.extend_from_slice(&[0_u8; 28]);
    encoded.extend_from_slice(&value.to_be_bytes());
}

fn append_u64_word(encoded: &mut Vec<u8>, value: u64) {
    encoded.extend_from_slice(&[0_u8; 24]);
    encoded.extend_from_slice(&value.to_be_bytes());
}

fn append_address_word(encoded: &mut Vec<u8>, value: &[u8; 20]) {
    encoded.extend_from_slice(&[0_u8; 12]);
    encoded.extend_from_slice(value);
}

fn require_canonical(original: &[u8], encoded: &[u8]) -> Result<(), PublicValuesError> {
    if original == encoded {
        Ok(())
    } else {
        Err(PublicValuesError::NonCanonicalAbi)
    }
}
