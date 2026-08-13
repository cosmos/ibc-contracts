// SPDX-License-Identifier: Apache-2.0

//! Conversion functions between `BorshHeader` types and ibc-rs types
//!
//! These conversions are implemented here (instead of in solana-ibc-types)
//! because they require access to ibc-rs dependencies which are not available
//! in the lightweight solana-ibc-types package.

use ibc_client_tendermint::types::Header;
use ibc_core_client_types::Height;
use solana_ibc_borsh_header::*;
use tendermint::account::Id as AccountId;
use tendermint::block::parts::Header as PartSetHeader;
use tendermint::block::signed_header::SignedHeader;
use tendermint::block::Id as BlockId;
use tendermint::block::{Commit, CommitSig, Header as TmHeader};
use tendermint::validator::{Info as ValidatorInfo, Set as ValidatorSet};
use tendermint::{Hash, PublicKey, Time};

/// Errors that can occur during Borsh to Tendermint type conversions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConversionError {
    /// Invalid height value
    InvalidHeight,
    /// Invalid Ed25519 public key bytes
    InvalidEd25519Key,
    /// Secp256k1 keys are not supported on Solana
    Secp256k1NotSupported,
    /// Invalid voting power value
    InvalidVotingPower,
    /// Invalid timestamp value
    InvalidTimestamp,
    /// Invalid part set header hash length (expected 32 bytes)
    InvalidPartSetHashLength,
    /// Invalid part set header
    InvalidPartSetHeader,
    /// Invalid block ID hash length (expected 32 bytes)
    InvalidBlockIdHashLength,
    /// Invalid signature bytes
    InvalidSignature,
    /// Invalid commit height
    InvalidCommitHeight,
    /// Invalid chain ID
    InvalidChainId,
    /// Invalid header height
    InvalidHeaderHeight,
    /// Invalid last commit hash length (expected 32 bytes)
    InvalidLastCommitHashLength,
    /// Invalid data hash length (expected 32 bytes)
    InvalidDataHashLength,
    /// Invalid last results hash length (expected 32 bytes)
    InvalidLastResultsHashLength,
    /// Invalid evidence hash length (expected 32 bytes)
    InvalidEvidenceHashLength,
    /// Invalid proposer address length (expected 20 bytes)
    InvalidProposerAddressLength,
    /// Invalid validators hash length (expected 32 bytes)
    InvalidValidatorsHashLength,
    /// Invalid next validators hash length (expected 32 bytes)
    InvalidNextValidatorsHashLength,
    /// Invalid consensus hash length (expected 32 bytes)
    InvalidConsensusHashLength,
    /// Invalid app hash
    InvalidAppHash,
    /// Failed to create signed header
    FailedToCreateSignedHeader,
}

impl ConversionError {
    /// Returns a human-readable error message
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::InvalidHeight => "Invalid height",
            Self::InvalidEd25519Key => "Invalid Ed25519 key",
            Self::Secp256k1NotSupported => "Secp256k1 not supported on Solana",
            Self::InvalidVotingPower => "Invalid voting power",
            Self::InvalidTimestamp => "Invalid timestamp",
            Self::InvalidPartSetHashLength => "Invalid part set header hash length",
            Self::InvalidPartSetHeader => "Invalid part set header",
            Self::InvalidBlockIdHashLength => "Invalid block ID hash length",
            Self::InvalidSignature => "Invalid signature",
            Self::InvalidCommitHeight => "Invalid commit height",
            Self::InvalidChainId => "Invalid chain ID",
            Self::InvalidHeaderHeight => "Invalid header height",
            Self::InvalidLastCommitHashLength => "Invalid last commit hash length",
            Self::InvalidDataHashLength => "Invalid data hash length",
            Self::InvalidLastResultsHashLength => "Invalid last results hash length",
            Self::InvalidEvidenceHashLength => "Invalid evidence hash length",
            Self::InvalidProposerAddressLength => "Invalid proposer address length",
            Self::InvalidValidatorsHashLength => "Invalid validators hash length",
            Self::InvalidNextValidatorsHashLength => "Invalid next validators hash length",
            Self::InvalidConsensusHashLength => "Invalid consensus hash length",
            Self::InvalidAppHash => "Invalid app hash",
            Self::FailedToCreateSignedHeader => "Failed to create signed header",
        }
    }
}

pub fn borsh_to_height(bh: BorshHeight) -> Result<Height, ConversionError> {
    Height::new(bh.revision_number, bh.revision_height).map_err(|_| ConversionError::InvalidHeight)
}

pub fn borsh_to_public_key(bpk: BorshPublicKey) -> Result<PublicKey, ConversionError> {
    match bpk {
        BorshPublicKey::Ed25519(bytes) => {
            PublicKey::from_raw_ed25519(&bytes).ok_or(ConversionError::InvalidEd25519Key)
        }
        BorshPublicKey::Secp256k1(_) => Err(ConversionError::Secp256k1NotSupported),
    }
}

pub fn borsh_to_validator(bv: BorshValidator) -> Result<ValidatorInfo, ConversionError> {
    Ok(ValidatorInfo {
        address: AccountId::new(bv.address),
        pub_key: borsh_to_public_key(bv.pub_key)?,
        power: tendermint::vote::Power::try_from(bv.voting_power)
            .map_err(|_| ConversionError::InvalidVotingPower)?,
        proposer_priority: tendermint::validator::ProposerPriority::from(bv.proposer_priority),
        name: None,
    })
}

pub fn borsh_to_validator_set(bvs: BorshValidatorSet) -> Result<ValidatorSet, ConversionError> {
    let validators: Result<Vec<_>, _> =
        bvs.validators.into_iter().map(borsh_to_validator).collect();
    let validators = validators?;

    let proposer = if let Some(p) = bvs.proposer {
        Some(borsh_to_validator(p)?)
    } else {
        None
    };

    Ok(ValidatorSet::new(validators, proposer))
}

pub fn borsh_to_time(bt: BorshTimestamp) -> Result<Time, ConversionError> {
    Time::from_unix_timestamp(bt.secs, bt.nanos as u32)
        .map_err(|_| ConversionError::InvalidTimestamp)
}

pub fn borsh_to_part_set_header(
    bpsh: BorshPartSetHeader,
) -> Result<PartSetHeader, ConversionError> {
    let hash_bytes: [u8; 32] = bpsh
        .hash
        .try_into()
        .map_err(|_| ConversionError::InvalidPartSetHashLength)?;
    let header = PartSetHeader::new(bpsh.total, Hash::Sha256(hash_bytes))
        .map_err(|_| ConversionError::InvalidPartSetHeader)?;

    Ok(header)
}

pub fn borsh_to_block_id(bbid: BorshBlockId) -> Result<BlockId, ConversionError> {
    let hash_bytes: [u8; 32] = bbid
        .hash
        .try_into()
        .map_err(|_| ConversionError::InvalidBlockIdHashLength)?;
    Ok(BlockId {
        hash: Hash::Sha256(hash_bytes),
        part_set_header: borsh_to_part_set_header(bbid.part_set_header)?,
    })
}

pub fn borsh_to_commit_sig(bcs: BorshCommitSig) -> Result<CommitSig, ConversionError> {
    match bcs {
        BorshCommitSig::BlockIdFlagAbsent => Ok(CommitSig::BlockIdFlagAbsent),
        BorshCommitSig::BlockIdFlagCommit {
            validator_address,
            timestamp,
            signature,
        } => {
            let sig = tendermint::Signature::new(signature)
                .map_err(|_| ConversionError::InvalidSignature)?;

            Ok(CommitSig::BlockIdFlagCommit {
                validator_address: AccountId::new(validator_address),
                timestamp: borsh_to_time(timestamp)?,
                signature: sig,
            })
        }
        BorshCommitSig::BlockIdFlagNil {
            validator_address,
            timestamp,
            signature,
        } => {
            let sig = tendermint::Signature::new(signature)
                .map_err(|_| ConversionError::InvalidSignature)?;

            Ok(CommitSig::BlockIdFlagNil {
                validator_address: AccountId::new(validator_address),
                timestamp: borsh_to_time(timestamp)?,
                signature: sig,
            })
        }
    }
}

pub fn borsh_to_commit(bc: BorshCommit) -> Result<Commit, ConversionError> {
    let signatures: Result<Vec<_>, _> =
        bc.signatures.into_iter().map(borsh_to_commit_sig).collect();

    Ok(Commit {
        height: tendermint::block::Height::try_from(bc.height)
            .map_err(|_| ConversionError::InvalidCommitHeight)?,
        round: bc.round.into(),
        block_id: borsh_to_block_id(bc.block_id)?,
        signatures: signatures?,
    })
}

pub const fn borsh_to_consensus_version(
    bcv: BorshConsensusVersion,
) -> tendermint::block::header::Version {
    tendermint::block::header::Version {
        block: bcv.block,
        app: bcv.app,
    }
}

pub fn borsh_to_block_header(bbh: BorshBlockHeader) -> Result<TmHeader, ConversionError> {
    let last_block_id = if let Some(lbid) = bbh.last_block_id {
        Some(borsh_to_block_id(lbid)?)
    } else {
        None
    };

    let last_commit_hash = if let Some(h) = bbh.last_commit_hash {
        let hash_bytes: [u8; 32] = h
            .try_into()
            .map_err(|_| ConversionError::InvalidLastCommitHashLength)?;
        Some(Hash::Sha256(hash_bytes))
    } else {
        None
    };

    let data_hash = if let Some(h) = bbh.data_hash {
        let hash_bytes: [u8; 32] = h
            .try_into()
            .map_err(|_| ConversionError::InvalidDataHashLength)?;
        Some(Hash::Sha256(hash_bytes))
    } else {
        None
    };

    let last_results_hash = if let Some(h) = bbh.last_results_hash {
        let hash_bytes: [u8; 32] = h
            .try_into()
            .map_err(|_| ConversionError::InvalidLastResultsHashLength)?;
        Some(Hash::Sha256(hash_bytes))
    } else {
        None
    };

    let evidence_hash = if let Some(h) = bbh.evidence_hash {
        let hash_bytes: [u8; 32] = h
            .try_into()
            .map_err(|_| ConversionError::InvalidEvidenceHashLength)?;
        Some(Hash::Sha256(hash_bytes))
    } else {
        None
    };

    let address_bytes: [u8; 20] = bbh
        .proposer_address
        .try_into()
        .map_err(|_| ConversionError::InvalidProposerAddressLength)?;

    Ok(TmHeader {
        version: borsh_to_consensus_version(bbh.version),
        chain_id: bbh
            .chain_id
            .try_into()
            .map_err(|_| ConversionError::InvalidChainId)?,
        height: tendermint::block::Height::try_from(bbh.height)
            .map_err(|_| ConversionError::InvalidHeaderHeight)?,
        time: borsh_to_time(bbh.time)?,
        last_block_id,
        last_commit_hash,
        data_hash,
        validators_hash: Hash::Sha256(
            bbh.validators_hash
                .try_into()
                .map_err(|_| ConversionError::InvalidValidatorsHashLength)?,
        ),
        next_validators_hash: Hash::Sha256(
            bbh.next_validators_hash
                .try_into()
                .map_err(|_| ConversionError::InvalidNextValidatorsHashLength)?,
        ),
        consensus_hash: Hash::Sha256(
            bbh.consensus_hash
                .try_into()
                .map_err(|_| ConversionError::InvalidConsensusHashLength)?,
        ),
        app_hash: tendermint::AppHash::try_from(bbh.app_hash)
            .map_err(|_| ConversionError::InvalidAppHash)?,
        last_results_hash,
        evidence_hash,
        proposer_address: AccountId::new(address_bytes),
    })
}

pub fn borsh_to_signed_header(bsh: BorshSignedHeader) -> Result<SignedHeader, ConversionError> {
    let header = borsh_to_block_header(bsh.header)?;
    let commit = borsh_to_commit(bsh.commit)?;

    SignedHeader::new(header, commit).map_err(|_| ConversionError::FailedToCreateSignedHeader)
}

pub fn borsh_to_header(bh: BorshHeader) -> Result<Header, ConversionError> {
    Ok(Header {
        signed_header: borsh_to_signed_header(bh.signed_header)?,
        validator_set: borsh_to_validator_set(bh.validator_set)?,
        trusted_height: borsh_to_height(bh.trusted_height)?,
        trusted_next_validator_set: borsh_to_validator_set(bh.trusted_next_validator_set)?,
    })
}
