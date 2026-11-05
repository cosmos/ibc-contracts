// SPDX-License-Identifier: Apache-2.0

//! This module defines types related to the Header we use for the Ethereum light client

use alloy_primitives::{keccak256, Bytes, B256};
use alloy_rlp::{Decodable, Rlp};
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
    /// Required from Gloas onwards, where light client data only commits to the block hash.
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
        let finalized_header = &self.consensus_update.finalized_header;
        if let Some(execution) = &finalized_header.execution {
            return Ok(FinalizedExecution {
                state_root: execution.state_root,
                block_number: execution.block_number,
                timestamp: execution.timestamp,
            });
        }

        let expected = finalized_header.execution_block_hash.ok_or(
            EthereumIBCError::InvalidLightClientHeaderFormat {
                slot: finalized_header.beacon.slot,
            },
        )?;
        let execution_header = self
            .finalized_execution_header
            .as_ref()
            .ok_or(EthereumIBCError::MissingFinalizedExecutionHeader)?;
        let found = keccak256(execution_header);
        ensure!(
            found == expected,
            EthereumIBCError::FinalizedExecutionHeaderMismatch { expected, found }
        );

        decode_execution_header(execution_header)
            .map_err(|e| EthereumIBCError::InvalidFinalizedExecutionHeader(e.to_string()))
    }
}

/// Decodes the fields we need from an RLP encoded execution block header. These leading fields
/// have kept their positions through all forks, new fields are only appended to the header.
fn decode_execution_header(header: &[u8]) -> alloy_rlp::Result<FinalizedExecution> {
    fn next<T: Decodable>(fields: &mut Rlp<'_>) -> alloy_rlp::Result<T> {
        fields.get_next()?.ok_or(alloy_rlp::Error::InputTooShort)
    }

    let mut fields = Rlp::new(header)?;
    // parent_hash, ommers_hash, beneficiary
    for _ in 0..3 {
        next::<Bytes>(&mut fields)?;
    }
    let state_root = next(&mut fields)?;
    // transactions_root, receipts_root, logs_bloom, difficulty
    for _ in 0..4 {
        next::<Bytes>(&mut fields)?;
    }
    let block_number = next(&mut fields)?;
    // gas_limit, gas_used
    for _ in 0..2 {
        next::<Bytes>(&mut fields)?;
    }
    let timestamp = next(&mut fields)?;

    Ok(FinalizedExecution {
        state_root,
        block_number,
        timestamp,
    })
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
        ExecutionPayloadHeader, LightClientHeader,
    };

    use super::*;

    // An execution block header of an Amsterdam block, as returned by `debug_getRawHeader`.
    const EXECUTION_HEADER: &str = "f90299a07a802472dcd4464726be3bddc3a64f36f6410fa01b733823e3b17a874a09d9d7a01dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347948943545177806ed17b9f23f0a21ee5948ecaa776a06d4cdea27ed68f0d4cbd443db0efe584a253fccf445e280040c81ba6502cf977a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421b90100000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000008047840bebc20080846aec45c699d88301110a846765746888676f312e32372e33856c696e7578a08aa640c661e4971471d6485e6ba1981b44ae4e4cdb05a1c60857224e92cb42f588000000000000000083012a18a056e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b4218080a0ce756ce287e17db000f9527f2744cd689676f67bfd09b11a1a76c3493d9ef9a1a0e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855a072b02fe4071f2eeed18399668a0598758416b8d551a71b7dd81aa640309fb8f847";
    const EXECUTION_BLOCK_HASH: B256 =
        b256!("1d00d753e3964751cefe1cae2d81ad8af9b59a4868df2db5ea243346e67a6038");

    fn gloas_header(execution_block_hash: B256, execution_header: Option<&str>) -> Header {
        Header {
            active_sync_committee: ActiveSyncCommittee::Current(SyncCommittee::default()),
            consensus_update: LightClientUpdate {
                finalized_header: LightClientHeader {
                    execution_block_hash: Some(execution_block_hash),
                    ..Default::default()
                },
                ..Default::default()
            },
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
        let mut header = gloas_header(B256::default(), None);
        header.consensus_update.finalized_header = LightClientHeader {
            execution: Some(ExecutionPayloadHeader {
                state_root: EXECUTION_BLOCK_HASH,
                block_number: 1,
                timestamp: 2,
                ..Default::default()
            }),
            ..Default::default()
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
