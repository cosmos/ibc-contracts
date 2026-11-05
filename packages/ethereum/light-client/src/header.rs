// SPDX-License-Identifier: Apache-2.0

//! This module defines types related to the Header we use for the Ethereum light client

use alloy_primitives::{aliases::B64, keccak256, Address, Bloom, Bytes, B256, U256};
use alloy_rlp::RlpDecodable;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use ethereum_types::consensus::{
    light_client_header::LightClientUpdate, sync_committee::SyncCommittee,
};

use crate::error::EthereumIBCError;

/// The header of a light client update
#[derive(Serialize, Deserialize, JsonSchema, PartialEq, Eq, Clone, Debug)]
pub struct Header {
    /// The active sync committee (untrusted)
    pub active_sync_committee: ActiveSyncCommittee,
    /// The consensus update
    pub consensus_update: LightClientUpdate,
    /// Trusted slot to verify the new update against
    // The client **must** have a consensus state for the provided slot
    pub trusted_slot: u64,
    /// The RLP encoded execution block header of the finalized header (untrusted).
    /// Required by Gloas light client data, which only commits to the block hash.
    // TODO: Make this required once mainnet has forked to Gloas (#1252)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(with = "Option<String>")]
    pub finalized_execution_header: Option<Bytes>,
}

/// The execution data of a finalized header
#[derive(PartialEq, Eq, Clone, Debug)]
pub struct FinalizedExecution {
    /// The state root of the execution block
    pub state_root: B256,
    /// The number of the execution block
    pub block_number: u64,
    /// The timestamp of the execution block
    pub timestamp: u64,
}

impl Header {
    /// Returns the execution data of the finalized header, which is either part of the consensus
    /// update or read from [`Self::finalized_execution_header`] after checking it against the
    /// block hash of the consensus update.
    /// # Errors
    /// Returns an error if the execution data is missing or doesn't match the finalized header.
    pub fn finalized_execution(&self) -> Result<FinalizedExecution, EthereumIBCError> {
        let expected = match &self.consensus_update {
            LightClientUpdate::Fulu(update) => {
                let execution = &update.finalized_header.execution;
                return Ok(FinalizedExecution {
                    state_root: execution.state_root,
                    block_number: execution.block_number,
                    timestamp: execution.timestamp,
                });
            }
            LightClientUpdate::Gloas(update) => update.finalized_header.execution_block_hash,
        };

        let execution_header = self
            .finalized_execution_header
            .as_ref()
            .ok_or(EthereumIBCError::MissingFinalizedExecutionHeader)?;
        let found = keccak256(execution_header);
        ensure!(
            found == expected,
            EthereumIBCError::FinalizedExecutionHeaderMismatch { expected, found }
        );

        let header = alloy_rlp::decode_exact::<ExecutionBlockHeader>(execution_header)
            .map_err(|e| EthereumIBCError::InvalidFinalizedExecutionHeader(e.to_string()))?;

        Ok(FinalizedExecution {
            state_root: header.state_root,
            block_number: header.number,
            timestamp: header.timestamp,
        })
    }
}

/// An execution block header as of the Amsterdam fork, in the order of its RLP encoding.
/// A later fork that adds fields needs them added here.
#[derive(RlpDecodable)]
#[rlp(trailing)]
#[allow(dead_code)]
struct ExecutionBlockHeader {
    parent_hash: B256,
    ommers_hash: B256,
    beneficiary: Address,
    state_root: B256,
    transactions_root: B256,
    receipts_root: B256,
    logs_bloom: Bloom,
    difficulty: U256,
    number: u64,
    gas_limit: u64,
    gas_used: u64,
    timestamp: u64,
    extra_data: Bytes,
    mix_hash: B256,
    nonce: B64,
    base_fee_per_gas: Option<u64>,
    withdrawals_root: Option<B256>,
    blob_gas_used: Option<u64>,
    excess_blob_gas: Option<u64>,
    parent_beacon_block_root: Option<B256>,
    requests_hash: Option<B256>,
    block_access_list_hash: Option<B256>,
    slot_number: Option<u64>,
}

/// The active sync committee
#[derive(Serialize, Deserialize, JsonSchema, PartialEq, Eq, Clone, Debug)]
pub enum ActiveSyncCommittee {
    /// The current sync committee
    Current(SyncCommittee),
    /// The next sync committee
    Next(SyncCommittee),
}

#[cfg(test)]
mod test {
    use alloy_primitives::{b256, hex};
    use ethereum_types::consensus::light_client_header::{
        ExecutionPayloadHeader, LightClientHeaderFulu, LightClientHeaderGloas,
        LightClientUpdateFulu, LightClientUpdateGloas, MerkleBranch,
    };
    use ethereum_types::consensus::sync_committee::SyncAggregate;

    use super::*;

    // An execution block header of an Amsterdam block, as returned by `debug_getRawHeader`.
    const EXECUTION_HEADER: &str = "f90299a07a802472dcd4464726be3bddc3a64f36f6410fa01b733823e3b17a874a09d9d7a01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347948943545177806ed17b9f23f0a21ee5948ecaa776a06d4cdea27ed68f0d4cbd443db0efe584a253fccf445e280040c81ba6502cf977a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421b90100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008047840bebc20080846aec45c699d88301110a846765746888676f312e32372e33856c696e7578a08aa640c661e4971471d6485e6ba1981b44ae4e4cdb05a1c60857224e92cb42f588000000000000000083012a18a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b4218080a0ce756ce287e17db000f9527f2744cd689676f67bfd09b11a1a76c3493d9ef9a1a0e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855a072b02fe4071f2eeed18399668a0598758416b8d551a71b7dd81aa640309fb8f847";
    const EXECUTION_BLOCK_HASH: B256 =
        b256!("1d00d753e3964751cefe1cae2d81ad8af9b59a4868df2db5ea243346e67a6038");

    fn gloas_header(execution_block_hash: B256, execution_header: Option<&str>) -> Header {
        Header {
            active_sync_committee: ActiveSyncCommittee::Current(SyncCommittee::default()),
            consensus_update: LightClientUpdate::Gloas(LightClientUpdateGloas {
                attested_header: LightClientHeaderGloas::default(),
                next_sync_committee: None,
                next_sync_committee_branch: None,
                finalized_header: LightClientHeaderGloas {
                    execution_block_hash,
                    ..Default::default()
                },
                finality_branch: MerkleBranch::default(),
                sync_aggregate: SyncAggregate::default(),
                signature_slot: 0,
            }),
            trusted_slot: 0,
            finalized_execution_header: execution_header
                .map(|header| hex::decode(header).unwrap().into()),
        }
    }

    #[test]
    fn test_finalized_execution_from_execution_header() {
        let header = gloas_header(EXECUTION_BLOCK_HASH, Some(EXECUTION_HEADER));

        assert_eq!(
            header.finalized_execution().unwrap(),
            FinalizedExecution {
                state_root: b256!(
                    "6d4cdea27ed68f0d4cbd443db0efe584a253fccf445e280040c81ba6502cf977"
                ),
                block_number: 71,
                timestamp: 1_793_869_254,
            }
        );
    }

    #[test]
    fn test_finalized_execution_from_execution_payload_header() {
        let header = Header {
            consensus_update: LightClientUpdate::Fulu(LightClientUpdateFulu {
                attested_header: LightClientHeaderFulu::default(),
                next_sync_committee: None,
                next_sync_committee_branch: None,
                finalized_header: LightClientHeaderFulu {
                    execution: ExecutionPayloadHeader {
                        state_root: EXECUTION_BLOCK_HASH,
                        block_number: 1,
                        timestamp: 2,
                        ..Default::default()
                    },
                    ..Default::default()
                },
                finality_branch: MerkleBranch::default(),
                sync_aggregate: SyncAggregate::default(),
                signature_slot: 0,
            }),
            ..gloas_header(B256::default(), None)
        };

        assert_eq!(
            header.finalized_execution().unwrap(),
            FinalizedExecution {
                state_root: EXECUTION_BLOCK_HASH,
                block_number: 1,
                timestamp: 2,
            }
        );
    }

    #[test]
    fn test_finalized_execution_rejects_missing_execution_header() {
        assert_eq!(
            gloas_header(EXECUTION_BLOCK_HASH, None).finalized_execution(),
            Err(EthereumIBCError::MissingFinalizedExecutionHeader)
        );
    }

    #[test]
    fn test_finalized_execution_rejects_wrong_execution_header() {
        assert_eq!(
            gloas_header(B256::default(), Some(EXECUTION_HEADER)).finalized_execution(),
            Err(EthereumIBCError::FinalizedExecutionHeaderMismatch {
                expected: B256::default(),
                found: EXECUTION_BLOCK_HASH,
            })
        );
    }

    #[test]
    fn test_finalized_execution_rejects_malformed_execution_header() {
        // RLP list of a single empty string, which is too short to be a block header
        let malformed = "c180";
        let block_hash = keccak256(hex::decode(malformed).unwrap());

        assert!(matches!(
            gloas_header(block_hash, Some(malformed)).finalized_execution(),
            Err(EthereumIBCError::InvalidFinalizedExecutionHeader(_))
        ));
    }
}
