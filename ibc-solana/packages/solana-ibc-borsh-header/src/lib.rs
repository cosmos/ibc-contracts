// SPDX-License-Identifier: Apache-2.0

//! Borsh-serializable wrapper types for Tendermint Header
//!
//! These types mirror `ibc-client-tendermint::types::Header` and related types,
//! but use Borsh serialization for efficient memory usage on Solana.
//!
//! Memory comparison:
//! - Protobuf: 38KB serialized → ~300KB deserialized
//! - Borsh: ~38KB serialized → ~60-90KB deserialized
//!
//! This allows fitting mainnet-sized headers within Solana's 256KB heap limit.
//!
//! Note: Conversion functions (From/TryFrom implementations) are implemented
//! in the packages that have access to ibc-rs types (relayer and ics07-tendermint).
//!
//! ## Direct Deserialization Optimization
//!
//! The `HeaderWrapper` type provides direct `BorshDeserialize` implementation for
//! `ibc_client_tendermint::types::Header`, eliminating the costly conversion step
//! from `BorshHeader` → Header (saves ~300k compute units).

use borsh::{BorshDeserialize, BorshSerialize};

#[cfg(feature = "light-client")]
use std::io::{self, Read};

/// Borsh-serializable wrapper for `ibc_client_tendermint::types::Header`
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize)]
pub struct BorshHeader {
    pub signed_header: BorshSignedHeader,
    pub validator_set: BorshValidatorSet,
    pub trusted_height: BorshHeight,
    pub trusted_next_validator_set: BorshValidatorSet,
}

/// Borsh-serializable wrapper for `tendermint::block::signed_header::SignedHeader`
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize)]
pub struct BorshSignedHeader {
    pub header: BorshBlockHeader,
    pub commit: BorshCommit,
}

/// Borsh-serializable wrapper for `tendermint::block::Header`
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize)]
pub struct BorshBlockHeader {
    pub version: BorshConsensusVersion,
    pub chain_id: String,
    pub height: u64,
    pub time: BorshTimestamp,
    pub last_block_id: Option<BorshBlockId>,
    pub last_commit_hash: Option<Vec<u8>>,
    pub data_hash: Option<Vec<u8>>,
    pub validators_hash: Vec<u8>,
    pub next_validators_hash: Vec<u8>,
    pub consensus_hash: Vec<u8>,
    pub app_hash: Vec<u8>,
    pub last_results_hash: Option<Vec<u8>>,
    pub evidence_hash: Option<Vec<u8>>,
    pub proposer_address: Vec<u8>,
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize)]
pub struct BorshConsensusVersion {
    pub block: u64,
    pub app: u64,
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize)]
pub struct BorshTimestamp {
    pub secs: i64,
    pub nanos: i32,
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize)]
pub struct BorshBlockId {
    pub hash: Vec<u8>,
    pub part_set_header: BorshPartSetHeader,
}

#[derive(Clone, Debug, BorshSerialize, BorshDeserialize)]
pub struct BorshPartSetHeader {
    pub total: u32,
    pub hash: Vec<u8>,
}

/// Borsh-serializable wrapper for `tendermint::block::Commit`
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize)]
pub struct BorshCommit {
    pub height: u64,
    pub round: u16,
    pub block_id: BorshBlockId,
    pub signatures: Vec<BorshCommitSig>,
}

/// Borsh-serializable wrapper for `tendermint::block::CommitSig`
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize)]
pub enum BorshCommitSig {
    BlockIdFlagAbsent,
    BlockIdFlagCommit {
        validator_address: [u8; 20],
        timestamp: BorshTimestamp,
        signature: [u8; 64],
    },
    BlockIdFlagNil {
        validator_address: [u8; 20],
        timestamp: BorshTimestamp,
        signature: [u8; 64],
    },
}

/// Borsh-serializable wrapper for `tendermint::validator::Set`
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize)]
pub struct BorshValidatorSet {
    pub validators: Vec<BorshValidator>,
    pub proposer: Option<BorshValidator>,
    pub total_voting_power: u64,
}

/// Borsh-serializable wrapper for `tendermint::validator::Info`
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize)]
pub struct BorshValidator {
    pub address: [u8; 20],
    pub pub_key: BorshPublicKey,
    pub voting_power: u64,
    pub proposer_priority: i64,
}

/// Borsh-serializable wrapper for `tendermint::public_key::PublicKey`
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize)]
pub enum BorshPublicKey {
    Ed25519([u8; 32]),
    Secp256k1([u8; 33]),
}

/// Borsh-serializable wrapper for `ibc_core_client_types::Height`
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize)]
pub struct BorshHeight {
    pub revision_number: u64,
    pub revision_height: u64,
}

/// Borsh-serializable wrapper for `ibc_client_tendermint::types::Misbehaviour`
#[derive(Clone, Debug, BorshSerialize, BorshDeserialize)]
pub struct BorshMisbehaviour {
    pub client_id: String,
    pub header1: BorshHeader,
    pub header2: BorshHeader,
}

#[cfg(feature = "light-client")]
mod direct_deser {
    use super::*;
    use ibc_client_tendermint::types::{Header, Misbehaviour};
    use ibc_core_client_types::Height;
    use tendermint::account::Id as AccountId;
    use tendermint::block::parts::Header as PartSetHeader;
    use tendermint::block::signed_header::SignedHeader;
    use tendermint::block::Id as BlockId;
    use tendermint::block::{Commit, CommitSig, Header as TmHeader};
    use tendermint::validator::{Info as ValidatorInfo, Set as ValidatorSet};
    use tendermint::{Hash, PublicKey, Time};

    /// Zero-copy wrapper for direct deserialization of Header
    ///
    /// This wrapper implements `BorshDeserialize` to convert bytes directly into
    /// `ibc_client_tendermint::types::Header`, bypassing the intermediate `BorshHeader`
    /// representation. This saves ~300k compute units by eliminating redundant
    /// type conversions and allocations.
    #[repr(transparent)]
    pub struct HeaderWrapper(pub Header);

    impl std::ops::Deref for HeaderWrapper {
        type Target = Header;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl std::ops::DerefMut for HeaderWrapper {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }

    impl From<HeaderWrapper> for Header {
        fn from(wrapper: HeaderWrapper) -> Self {
            wrapper.0
        }
    }

    impl BorshDeserialize for HeaderWrapper {
        fn deserialize_reader<R: Read>(reader: &mut R) -> io::Result<Self> {
            let signed_header = deserialize_signed_header(reader)?;
            let validator_set = deserialize_validator_set(reader)?;
            let trusted_height = deserialize_height(reader)?;
            let trusted_next_validator_set = deserialize_validator_set(reader)?;

            Ok(Self(Header {
                signed_header,
                validator_set,
                trusted_height,
                trusted_next_validator_set,
            }))
        }
    }

    /// Zero-copy wrapper for direct deserialization of Misbehaviour
    ///
    /// This wrapper implements `BorshDeserialize` to convert bytes directly into
    /// `ibc_client_tendermint::types::Misbehaviour`, bypassing the intermediate `BorshMisbehaviour`
    /// representation. This saves ~300k compute units by eliminating redundant
    /// type conversions and allocations.
    #[repr(transparent)]
    pub struct MisbehaviourWrapper(pub Misbehaviour);

    impl std::ops::Deref for MisbehaviourWrapper {
        type Target = Misbehaviour;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    impl std::ops::DerefMut for MisbehaviourWrapper {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }

    impl From<MisbehaviourWrapper> for Misbehaviour {
        fn from(wrapper: MisbehaviourWrapper) -> Self {
            wrapper.0
        }
    }

    impl BorshDeserialize for MisbehaviourWrapper {
        fn deserialize_reader<R: Read>(reader: &mut R) -> io::Result<Self> {
            let client_id = String::deserialize_reader(reader)?;
            let client_id = client_id
                .parse()
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid client ID"))?;
            let header1 = deserialize_header(reader)?;
            let header2 = deserialize_header(reader)?;

            Ok(Self(Misbehaviour::new(client_id, header1, header2)))
        }
    }

    fn deserialize_header<R: Read>(reader: &mut R) -> io::Result<Header> {
        let signed_header = deserialize_signed_header(reader)?;
        let validator_set = deserialize_validator_set(reader)?;
        let trusted_height = deserialize_height(reader)?;
        let trusted_next_validator_set = deserialize_validator_set(reader)?;

        Ok(Header {
            signed_header,
            validator_set,
            trusted_height,
            trusted_next_validator_set,
        })
    }

    fn deserialize_height<R: Read>(reader: &mut R) -> io::Result<Height> {
        let revision_number = u64::deserialize_reader(reader)?;
        let revision_height = u64::deserialize_reader(reader)?;
        Height::new(revision_number, revision_height)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
    }

    fn deserialize_time<R: Read>(reader: &mut R) -> io::Result<Time> {
        let secs = i64::deserialize_reader(reader)?;
        let nanos = i32::deserialize_reader(reader)?;
        Time::from_unix_timestamp(secs, nanos as u32)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
    }

    fn deserialize_required_hash<R: Read>(reader: &mut R) -> io::Result<Hash> {
        let len = u32::deserialize_reader(reader)?;
        if len != 32 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid hash length: expected 32, got {len}"),
            ));
        }

        // Read directly into fixed-size array (avoids Vec allocation, saves ~3-5k CUs)
        let mut hash_bytes = [0u8; 32];
        reader.read_exact(&mut hash_bytes)?;
        Ok(Hash::Sha256(hash_bytes))
    }

    fn deserialize_optional_hash<R: Read>(reader: &mut R) -> io::Result<Option<Hash>> {
        let has_value = u8::deserialize_reader(reader)?;
        if has_value == 1 {
            Ok(Some(deserialize_required_hash(reader)?))
        } else {
            Ok(None)
        }
    }

    fn deserialize_public_key<R: Read>(reader: &mut R) -> io::Result<PublicKey> {
        let discriminant = u8::deserialize_reader(reader)?;
        match discriminant {
            0 => {
                // Ed25519
                let mut bytes = [0u8; 32];
                reader.read_exact(&mut bytes)?;
                PublicKey::from_raw_ed25519(&bytes).ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidData, "Invalid Ed25519 key")
                })
            }
            1 => {
                // Secp256k1 (not supported on Solana)
                Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Secp256k1 not supported on Solana",
                ))
            }
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Unknown public key type",
            )),
        }
    }

    fn deserialize_validator<R: Read>(reader: &mut R) -> io::Result<ValidatorInfo> {
        let mut address_bytes = [0u8; 20];
        reader.read_exact(&mut address_bytes)?;
        let address = AccountId::new(address_bytes);

        let pub_key = deserialize_public_key(reader)?;

        let voting_power = u64::deserialize_reader(reader)?;
        let power = tendermint::vote::Power::try_from(voting_power)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

        let proposer_priority_val = i64::deserialize_reader(reader)?;
        let proposer_priority =
            tendermint::validator::ProposerPriority::from(proposer_priority_val);

        Ok(ValidatorInfo {
            address,
            pub_key,
            power,
            proposer_priority,
            name: None,
        })
    }

    fn deserialize_validator_set<R: Read>(reader: &mut R) -> io::Result<ValidatorSet> {
        let validators_len = u32::deserialize_reader(reader)?;
        let mut validators = Vec::with_capacity(validators_len as usize);

        for _ in 0..validators_len {
            validators.push(deserialize_validator(reader)?);
        }

        let has_proposer = u8::deserialize_reader(reader)?;
        let proposer = if has_proposer == 1 {
            Some(deserialize_validator(reader)?)
        } else {
            None
        };

        // Use pre-calculated total_voting_power from relayer (validators are already sorted)
        let total_voting_power = u64::deserialize_reader(reader)?;
        let total_voting_power = tendermint::vote::Power::try_from(total_voting_power)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

        // Directly construct ValidatorSet - validators are pre-sorted by relayer
        // This saves ~50k CUs per validator set by skipping sort (tested with 100 validators)
        Ok(ValidatorSet {
            validators,
            proposer,
            total_voting_power,
        })
    }

    fn deserialize_commit_sig<R: Read>(reader: &mut R) -> io::Result<CommitSig> {
        let discriminant = u8::deserialize_reader(reader)?;
        match discriminant {
            0 => Ok(CommitSig::BlockIdFlagAbsent),
            1 => {
                // BlockIdFlagCommit
                let mut validator_address = [0u8; 20];
                reader.read_exact(&mut validator_address)?;

                let timestamp = deserialize_time(reader)?;

                let mut signature = [0u8; 64];
                reader.read_exact(&mut signature)?;
                let signature = tendermint::Signature::new(signature)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

                Ok(CommitSig::BlockIdFlagCommit {
                    validator_address: AccountId::new(validator_address),
                    timestamp,
                    signature,
                })
            }
            2 => {
                // BlockIdFlagNil
                let mut validator_address = [0u8; 20];
                reader.read_exact(&mut validator_address)?;

                let timestamp = deserialize_time(reader)?;

                let mut signature = [0u8; 64];
                reader.read_exact(&mut signature)?;
                let signature = tendermint::Signature::new(signature)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

                Ok(CommitSig::BlockIdFlagNil {
                    validator_address: AccountId::new(validator_address),
                    timestamp,
                    signature,
                })
            }
            _ => Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Unknown CommitSig variant",
            )),
        }
    }

    fn deserialize_part_set_header<R: Read>(reader: &mut R) -> io::Result<PartSetHeader> {
        let total = u32::deserialize_reader(reader)?;
        let hash = deserialize_required_hash(reader)?;
        PartSetHeader::new(total, hash)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
    }

    fn deserialize_block_id<R: Read>(reader: &mut R) -> io::Result<BlockId> {
        let hash = deserialize_required_hash(reader)?;
        let part_set_header = deserialize_part_set_header(reader)?;
        Ok(BlockId {
            hash,
            part_set_header,
        })
    }

    fn deserialize_commit<R: Read>(reader: &mut R) -> io::Result<Commit> {
        let height_val = u64::deserialize_reader(reader)?;
        let height = tendermint::block::Height::try_from(height_val)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

        let round_val = u16::deserialize_reader(reader)?;
        let round: tendermint::block::Round = round_val.into();

        let block_id = deserialize_block_id(reader)?;

        let sigs_len = u32::deserialize_reader(reader)?;
        let mut signatures = Vec::with_capacity(sigs_len as usize);
        for _ in 0..sigs_len {
            signatures.push(deserialize_commit_sig(reader)?);
        }

        Ok(Commit {
            height,
            round,
            block_id,
            signatures,
        })
    }

    fn deserialize_consensus_version<R: Read>(
        reader: &mut R,
    ) -> io::Result<tendermint::block::header::Version> {
        let block = u64::deserialize_reader(reader)?;
        let app = u64::deserialize_reader(reader)?;
        Ok(tendermint::block::header::Version { block, app })
    }

    fn deserialize_block_header<R: Read>(reader: &mut R) -> io::Result<TmHeader> {
        let version = deserialize_consensus_version(reader)?;

        let chain_id = String::deserialize_reader(reader)?;
        let chain_id = chain_id
            .try_into()
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Invalid chain ID"))?;

        let height_val = u64::deserialize_reader(reader)?;
        let height = tendermint::block::Height::try_from(height_val)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?;

        let time = deserialize_time(reader)?;

        let has_last_block_id = u8::deserialize_reader(reader)?;
        let last_block_id = if has_last_block_id == 1 {
            Some(deserialize_block_id(reader)?)
        } else {
            None
        };

        let last_commit_hash = deserialize_optional_hash(reader)?;
        let data_hash = deserialize_optional_hash(reader)?;

        let validators_hash = deserialize_required_hash(reader)?;
        let next_validators_hash = deserialize_required_hash(reader)?;
        let consensus_hash = deserialize_required_hash(reader)?;

        // Optimized app hash deserialization - fast path for common 32-byte case
        let app_hash_len = u32::deserialize_reader(reader)?;
        let app_hash = if app_hash_len == 32 {
            // Fast path: read directly into fixed array (avoids Vec allocation, saves ~3-5k CUs)
            let mut bytes = [0u8; 32];
            reader.read_exact(&mut bytes)?;
            tendermint::AppHash::try_from(bytes.to_vec())
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?
        } else {
            // Fallback for variable-length app hash
            let mut bytes = vec![0u8; app_hash_len as usize];
            reader.read_exact(&mut bytes)?;
            tendermint::AppHash::try_from(bytes)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))?
        };

        let last_results_hash = deserialize_optional_hash(reader)?;
        let evidence_hash = deserialize_optional_hash(reader)?;

        // Optimized proposer address deserialization (avoids Vec allocation, saves ~3-5k CUs)
        let proposer_len = u32::deserialize_reader(reader)?;
        if proposer_len != 20 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Invalid proposer address length: expected 20, got {proposer_len}"),
            ));
        }
        let mut proposer_bytes = [0u8; 20];
        reader.read_exact(&mut proposer_bytes)?;
        let proposer_address = AccountId::new(proposer_bytes);

        Ok(TmHeader {
            version,
            chain_id,
            height,
            time,
            last_block_id,
            last_commit_hash,
            data_hash,
            validators_hash,
            next_validators_hash,
            consensus_hash,
            app_hash,
            last_results_hash,
            evidence_hash,
            proposer_address,
        })
    }

    fn deserialize_signed_header<R: Read>(reader: &mut R) -> io::Result<SignedHeader> {
        let header = deserialize_block_header(reader)?;
        let commit = deserialize_commit(reader)?;

        SignedHeader::new(header, commit)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e.to_string()))
    }
}

/// Conversion functions from ibc-rs types to `BorshHeader` types
///
/// These conversions are used to convert Header to `BorshHeader` for efficient
/// serialization before uploading to Solana.
#[cfg(feature = "light-client")]
pub mod conversions {
    use super::*;
    use ibc_client_tendermint::types::{Header, Misbehaviour};
    use ibc_core_client_types::Height;
    use tendermint::block::parts::Header as PartSetHeader;
    use tendermint::block::Id as BlockId;
    use tendermint::block::{signed_header::SignedHeader, Commit, CommitSig, Header as TmHeader};
    use tendermint::validator::{Info as ValidatorInfo, Set as ValidatorSet};
    use tendermint::{PublicKey, Time};

    pub fn height_to_borsh(height: Height) -> BorshHeight {
        BorshHeight {
            revision_number: height.revision_number(),
            revision_height: height.revision_height(),
        }
    }

    pub fn public_key_to_borsh(pk: PublicKey) -> BorshPublicKey {
        match pk {
            PublicKey::Ed25519(bytes) => {
                let bytes_array: [u8; 32] = bytes
                    .as_bytes()
                    .try_into()
                    .expect("Ed25519 pubkey must be 32 bytes");
                BorshPublicKey::Ed25519(bytes_array)
            }
            _ => panic!("Only Ed25519 public keys are supported on Solana"),
        }
    }

    pub fn validator_to_borsh(v: ValidatorInfo) -> BorshValidator {
        let address_array: [u8; 20] = v
            .address
            .as_bytes()
            .try_into()
            .expect("Validator address must be 20 bytes");

        BorshValidator {
            address: address_array,
            pub_key: public_key_to_borsh(v.pub_key),
            voting_power: v.power.value(),
            proposer_priority: v.proposer_priority.value(),
        }
    }

    pub fn validator_set_to_borsh(vs: ValidatorSet) -> BorshValidatorSet {
        BorshValidatorSet {
            validators: vs
                .validators()
                .iter()
                .cloned()
                .map(validator_to_borsh)
                .collect(),
            proposer: vs.proposer().clone().map(validator_to_borsh),
            total_voting_power: vs.total_voting_power().value(),
        }
    }

    pub fn time_to_borsh(t: Time) -> BorshTimestamp {
        BorshTimestamp {
            secs: t.unix_timestamp(),
            nanos: (t.unix_timestamp_nanos() % 1_000_000_000) as i32,
        }
    }

    pub fn part_set_header_to_borsh(psh: PartSetHeader) -> BorshPartSetHeader {
        BorshPartSetHeader {
            total: psh.total,
            hash: psh.hash.as_bytes().to_vec(),
        }
    }

    pub fn block_id_to_borsh(bid: BlockId) -> BorshBlockId {
        BorshBlockId {
            hash: bid.hash.as_bytes().to_vec(),
            part_set_header: part_set_header_to_borsh(bid.part_set_header),
        }
    }

    pub fn commit_sig_to_borsh(cs: CommitSig) -> BorshCommitSig {
        match cs {
            CommitSig::BlockIdFlagAbsent => BorshCommitSig::BlockIdFlagAbsent,
            CommitSig::BlockIdFlagCommit {
                validator_address,
                timestamp,
                signature,
            } => {
                let address_array: [u8; 20] = validator_address
                    .as_bytes()
                    .try_into()
                    .expect("Validator address must be 20 bytes");
                let sig_array: [u8; 64] = signature.map_or([0u8; 64], |s| {
                    s.as_bytes().try_into().expect("Signature must be 64 bytes")
                });

                BorshCommitSig::BlockIdFlagCommit {
                    validator_address: address_array,
                    timestamp: time_to_borsh(timestamp),
                    signature: sig_array,
                }
            }
            CommitSig::BlockIdFlagNil {
                validator_address,
                timestamp,
                signature,
            } => {
                let address_array: [u8; 20] = validator_address
                    .as_bytes()
                    .try_into()
                    .expect("Validator address must be 20 bytes");
                let sig_array: [u8; 64] = signature.map_or([0u8; 64], |s| {
                    s.as_bytes().try_into().expect("Signature must be 64 bytes")
                });

                BorshCommitSig::BlockIdFlagNil {
                    validator_address: address_array,
                    timestamp: time_to_borsh(timestamp),
                    signature: sig_array,
                }
            }
        }
    }

    pub fn commit_to_borsh(c: Commit) -> BorshCommit {
        // Convert and sort signatures by validator address
        // This pre-sorting saves ~60-80k CUs during on-chain deserialization (tested with 100 validators)
        // The sort must match the order expected by the verifier's binary search
        let mut signatures: Vec<BorshCommitSig> =
            c.signatures.into_iter().map(commit_sig_to_borsh).collect();

        signatures.sort_unstable_by(|a, b| match (a, b) {
            (
                BorshCommitSig::BlockIdFlagCommit {
                    validator_address: addr_a,
                    ..
                }
                | BorshCommitSig::BlockIdFlagNil {
                    validator_address: addr_a,
                    ..
                },
                BorshCommitSig::BlockIdFlagCommit {
                    validator_address: addr_b,
                    ..
                }
                | BorshCommitSig::BlockIdFlagNil {
                    validator_address: addr_b,
                    ..
                },
            ) => addr_a.cmp(addr_b),
            (BorshCommitSig::BlockIdFlagAbsent, BorshCommitSig::BlockIdFlagAbsent) => {
                std::cmp::Ordering::Equal
            }
            (BorshCommitSig::BlockIdFlagAbsent, _) => std::cmp::Ordering::Less,
            (_, BorshCommitSig::BlockIdFlagAbsent) => std::cmp::Ordering::Greater,
        });

        BorshCommit {
            height: c.height.value(),
            round: c.round.value() as u16,
            block_id: block_id_to_borsh(c.block_id),
            signatures,
        }
    }

    pub const fn consensus_version_to_borsh(
        v: tendermint::block::header::Version,
    ) -> BorshConsensusVersion {
        BorshConsensusVersion {
            block: v.block,
            app: v.app,
        }
    }

    pub fn block_header_to_borsh(h: TmHeader) -> BorshBlockHeader {
        BorshBlockHeader {
            version: consensus_version_to_borsh(h.version),
            chain_id: h.chain_id.to_string(),
            height: h.height.value(),
            time: time_to_borsh(h.time),
            last_block_id: h.last_block_id.map(block_id_to_borsh),
            last_commit_hash: h.last_commit_hash.map(|h| h.as_bytes().to_vec()),
            data_hash: h.data_hash.map(|h| h.as_bytes().to_vec()),
            validators_hash: h.validators_hash.as_bytes().to_vec(),
            next_validators_hash: h.next_validators_hash.as_bytes().to_vec(),
            consensus_hash: h.consensus_hash.as_bytes().to_vec(),
            app_hash: h.app_hash.as_bytes().to_vec(),
            last_results_hash: h.last_results_hash.map(|h| h.as_bytes().to_vec()),
            evidence_hash: h.evidence_hash.map(|h| h.as_bytes().to_vec()),
            proposer_address: h.proposer_address.as_bytes().to_vec(),
        }
    }

    pub fn signed_header_to_borsh(sh: SignedHeader) -> BorshSignedHeader {
        BorshSignedHeader {
            header: block_header_to_borsh(sh.header),
            commit: commit_to_borsh(sh.commit),
        }
    }

    pub fn header_to_borsh(h: Header) -> BorshHeader {
        BorshHeader {
            signed_header: signed_header_to_borsh(h.signed_header),
            validator_set: validator_set_to_borsh(h.validator_set),
            trusted_height: height_to_borsh(h.trusted_height),
            trusted_next_validator_set: validator_set_to_borsh(h.trusted_next_validator_set),
        }
    }

    pub fn misbehaviour_to_borsh(m: &Misbehaviour) -> BorshMisbehaviour {
        BorshMisbehaviour {
            client_id: m.client_id().to_string(),
            header1: header_to_borsh(m.header1().clone()),
            header2: header_to_borsh(m.header2().clone()),
        }
    }
}

#[cfg(feature = "light-client")]
pub use direct_deser::{HeaderWrapper, MisbehaviourWrapper};
