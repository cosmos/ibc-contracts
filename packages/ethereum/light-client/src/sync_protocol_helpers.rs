// SPDX-License-Identifier: Apache-2.0

//! This module implements the sync protocol helpers defined in [consensus-specs](https://github.com/ethereum/consensus-specs)

use alloy_primitives::B256;
use ethereum_types::consensus::{
    light_client_header::LightClientHeader,
    merkle::{
        floorlog2, CURRENT_SYNC_COMMITTEE_GINDEX_ELECTRA, CURRENT_SYNC_COMMITTEE_GINDEX_GLOAS,
        EXECUTION_BLOCK_HASH_GINDEX_DENEB, EXECUTION_BLOCK_HASH_GINDEX_GLOAS,
        EXECUTION_PAYLOAD_GINDEX, FINALIZED_ROOT_GINDEX_ELECTRA, FINALIZED_ROOT_GINDEX_GLOAS,
        NEXT_SYNC_COMMITTEE_GINDEX_ELECTRA, NEXT_SYNC_COMMITTEE_GINDEX_GLOAS,
    },
};
use tree_hash::TreeHash;

use crate::{
    client_state::ClientState,
    error::EthereumIBCError,
    trie::{is_valid_normalized_merkle_branch, validate_merkle_branch},
};

// See spec: https://github.com/ethereum/consensus-specs/blob/master/specs/gloas/light-client/sync-protocol.md#modified-finalized_root_gindex_at_slot
/// Returns the finalized root gindex at the given slot.
/// # Errors
/// Returns an error if the epoch is in a non-supported fork.
pub fn finalized_root_gindex_at_slot(
    client_state: &ClientState,
    slot: u64,
) -> Result<u64, EthereumIBCError> {
    let epoch = client_state.compute_epoch_at_slot(slot);
    client_state.verify_supported_fork_at_epoch(epoch)?;

    Ok(match epoch {
        _ if epoch >= client_state.fork_parameters.gloas.epoch => FINALIZED_ROOT_GINDEX_GLOAS,
        _ => FINALIZED_ROOT_GINDEX_ELECTRA,
    })
}

// See spec: https://github.com/ethereum/consensus-specs/blob/master/specs/gloas/light-client/sync-protocol.md#modified-current_sync_committee_gindex_at_slot
/// Returns the current sync committee gindex at the given slot.
/// # Errors
/// Returns an error if the epoch is in a non-supported fork.
pub fn current_sync_committee_gindex_at_slot(
    client_state: &ClientState,
    slot: u64,
) -> Result<u64, EthereumIBCError> {
    let epoch = client_state.compute_epoch_at_slot(slot);
    client_state.verify_supported_fork_at_epoch(epoch)?;

    Ok(match epoch {
        _ if epoch >= client_state.fork_parameters.gloas.epoch => {
            CURRENT_SYNC_COMMITTEE_GINDEX_GLOAS
        }
        _ => CURRENT_SYNC_COMMITTEE_GINDEX_ELECTRA,
    })
}

// See spec: https://github.com/ethereum/consensus-specs/blob/master/specs/gloas/light-client/sync-protocol.md#modified-next_sync_committee_gindex_at_slot
/// Returns the next sync committee gindex at the given slot.
/// # Errors
/// Returns an error if the epoch is in a non-supported fork.
pub fn next_sync_committee_gindex_at_slot(
    client_state: &ClientState,
    slot: u64,
) -> Result<u64, EthereumIBCError> {
    let epoch = client_state.compute_epoch_at_slot(slot);
    client_state.verify_supported_fork_at_epoch(epoch)?;

    Ok(match epoch {
        _ if epoch >= client_state.fork_parameters.gloas.epoch => NEXT_SYNC_COMMITTEE_GINDEX_GLOAS,
        _ => NEXT_SYNC_COMMITTEE_GINDEX_ELECTRA,
    })
}

// See spec: https://github.com/ethereum/consensus-specs/blob/master/specs/gloas/light-client/sync-protocol.md#modified-is_valid_light_client_header
/// Validates a light client header.
/// # Errors
/// Returns an error if the header cannot be validated.
pub fn is_valid_light_client_header(
    client_state: &ClientState,
    header: &LightClientHeader,
) -> Result<(), EthereumIBCError> {
    let epoch = client_state.compute_epoch_at_slot(header.beacon().slot);

    client_state.verify_supported_fork_at_epoch(epoch)?;

    let is_gloas = epoch >= client_state.fork_parameters.gloas.epoch;
    match header {
        LightClientHeader::Gloas(header) => is_valid_normalized_merkle_branch(
            header.execution_block_hash,
            &header.execution_branch,
            // A pre-Gloas block proves its block hash through the execution payload, in a branch
            // normalized to the Gloas depth
            if is_gloas {
                EXECUTION_BLOCK_HASH_GINDEX_GLOAS
            } else {
                EXECUTION_BLOCK_HASH_GINDEX_DENEB
            },
            header.beacon.body_root,
        ),
        LightClientHeader::Fulu(_) if is_gloas => {
            Err(EthereumIBCError::InvalidLightClientHeaderFormat {
                slot: header.beacon().slot,
            })
        }
        LightClientHeader::Fulu(header) => validate_merkle_branch(
            header.execution.tree_hash_root(),
            header.execution_branch.to_vec(),
            floorlog2(EXECUTION_PAYLOAD_GINDEX),
            get_subtree_index(EXECUTION_PAYLOAD_GINDEX),
            header.beacon.body_root,
        ),
    }
}

// See spec: <https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/light-client/sync-protocol.md#get_subtree_index>
/// Values that are constant across all configurations.
#[must_use]
pub const fn get_subtree_index(idx: u64) -> u64 {
    idx % 2_u64.pow(idx.ilog2())
}

// See spec: <https://github.com/ethereum/consensus-specs/blob/ff99bc03d6da29d9ef6e055bdb8500e1b2942f1e/specs/electra/light-client/fork.md#L26>
/// Normalize a merkle branch to a depth for given gindex.
/// # Errors
/// Returns an error if the merkle branch is larger than the calculated depth for the given gindex.
pub fn normalize_merkle_branch(
    branch: &[B256],
    gindex: u64,
) -> Result<Vec<B256>, EthereumIBCError> {
    let depth = floorlog2(gindex);
    let num_extra =
        depth
            .checked_sub(branch.len())
            .ok_or(EthereumIBCError::InvalidMerkleBranchLength {
                expected: depth,
                found: branch.len(),
            })?;

    // TODO: Switch to std::iter::repeat_n when cosmwasm supports rust 1.85 (https://github.com/CosmWasm/cosmwasm/issues/2292)
    Ok(vec![B256::default(); num_extra]
        .into_iter()
        .chain(branch.to_vec())
        .collect())
}

#[cfg(test)]
mod test {
    use alloy_primitives::{hex::FromHex, B256};
    use ethereum_types::consensus::{
        fork::{Fork, ForkParameters},
        light_client_header::{LightClientHeader, LightClientHeaderFulu, LightClientHeaderGloas},
    };

    use crate::{
        client_state::ClientState,
        error::EthereumIBCError,
        sync_protocol_helpers::{is_valid_light_client_header, normalize_merkle_branch},
    };

    // Headers from a light client update of a chain with the minimal preset that forked to Gloas
    // at epoch 3, which is slot 24. The finalized header is a Fulu block in the Gloas format.
    const GLOAS_FORK_EPOCH: u64 = 3;
    const ATTESTED_GLOAS_HEADER: &str = r#"{"beacon": {"slot": "32", "proposer_index": "17", "parent_root": "0x4f92999ca0642ba30087b5096be30881fceff2461b7eb2c224b08ab18b2f5fc4", "state_root": "0x4ac3b4a27dbb5c60e8fbfa702f86ba75f4dbf5219d18b8f68708aa91b7e83817", "body_root": "0x586c58a3ffa8e75f60d8c172d5c4a3de3c86aaba75be213ac499522cd2109c57"}, "execution_block_hash": "0x0be9940c62ab12d13c6daa9fe151026110fe1ebe2de9bd8c77537f06278134a1", "execution_branch": ["0x79fd9d960bf28d58302ce41ad53bc3e0318115555345e5f0f72eecfb30ea8127", "0xff0f000000000000000000000000000000000000000000000000000000000000", "0x8e742f5552fe005d58a4676d4c9709ac5cba3fc36cb99e387dcb2414edf53252", "0xf5a5fd42d16a20302798ef6ed309979b43003d2320d9f0e8ea9831a92759fb4b", "0xf9d178a27812e08cd36ac44c9b2801f9d97064f66bea11e5731d4ebd4cd89c36", "0xf8cc67b3c782cb3a627dba287302985e1a6e69c03f816c63d1ea77fca6b9e946", "0xc78009fdf07fc56a11f122370658a353aaa542ed63e44c4bc15ff4cd105ab33c", "0x0000000000000000000000000000000000000000000000000000000000000000", "0xcdddd46ec9010e1c3e9c9e24c760f3c3cb97a647c250c4b0e2ba792d27917b8f", "0x2c2ed5ddfb8b06fd1ccee426b4cd16e01506929587326d96bcd0716d62510882", "0xff1f000000000000000000000000000000000000000000000000000000000000"]}"#;
    const FINALIZED_FULU_HEADER: &str = r#"{"beacon": {"slot": "16", "proposer_index": "95", "parent_root": "0x12b36d2a636dc8645dde108490c78ce578c292bd0952fb737d4c5811c1c16787", "state_root": "0x5d8b723adced04424ea179a14bba9408335d5547477afea653f9cc32c1fa611d", "body_root": "0x9f4f1f3e406413a20e764e6203755b7f179eca0103f0c4b28b4a78bdb5a56707"}, "execution_block_hash": "0x6fc91d3fa0888b32e651413d932af84dd1e81f2e81a128545e18bec9e79c81d3", "execution_branch": ["0x0000000000000000000000000000000000000000000000000000000000000000", "0x0000000000000000000000000000000000000000000000000000000000000000", "0x7ffe241ea60187fdb0187bfa22de35d1f9bed7ab061d9401fd47e34a54fbede1", "0x7d00409ac38e3a99bc86c045f570dea66e115ba1d767d89c9e7f82901dfcfd88", "0x1944592964352f839fff971af59bf656dafcc09f5dea1b6751fadcc6ff4168f0", "0x8826489e7f0d94b182e7e1b71390ea8bf43e7ca33cd089cf61b4b064c04f1fbf", "0x536d98837f2dd165a55d5eeae91485954472d56f246df256bf3cae19352a123c", "0xc068bc9791786253883c462d3bfdf58013a9eb4aedee89d2c8643933695f34b0", "0xb46f0c01805fe212e15907981b757e6c496b0cb06664224655613dcec82505bb", "0x6dd3b9955d892d92338b19976fd07084bfe88a76c3063482b7f30ee60feb2a58", "0x6715f99cb8416aa06a201e55666754246210aceca72e6402ebff18a1489e04b0"]}"#;

    fn client_state() -> ClientState {
        ClientState {
            slots_per_epoch: 8,
            fork_parameters: ForkParameters {
                gloas: Fork {
                    epoch: GLOAS_FORK_EPOCH,
                    ..Default::default()
                },
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn header(json: &str) -> LightClientHeader {
        serde_json::from_str(json).unwrap()
    }

    #[test]
    fn test_valid_gloas_format_headers() {
        is_valid_light_client_header(&client_state(), &header(ATTESTED_GLOAS_HEADER)).unwrap();
        is_valid_light_client_header(&client_state(), &header(FINALIZED_FULU_HEADER)).unwrap();
    }

    #[test]
    fn test_header_is_verified_against_the_gindex_of_its_fork() {
        // The Fulu header is not a valid Gloas header if Gloas was active at its slot
        let mut client_state = client_state();
        client_state.fork_parameters.gloas.epoch = 0;
        assert!(matches!(
            is_valid_light_client_header(&client_state, &header(FINALIZED_FULU_HEADER)),
            Err(EthereumIBCError::InvalidMerkleBranch(_))
        ));

        // The Gloas header is not a valid Fulu header if Gloas is not scheduled (i.e. not migrated)
        client_state.fork_parameters.gloas.epoch = u64::MAX;
        assert!(matches!(
            is_valid_light_client_header(&client_state, &header(ATTESTED_GLOAS_HEADER)),
            Err(EthereumIBCError::InvalidNormalizedMerkleBranch { .. })
        ));
    }

    #[test]
    fn test_header_format_is_picked_when_decoding() {
        let gloas: LightClientHeader = serde_json::from_str(ATTESTED_GLOAS_HEADER).unwrap();
        assert!(matches!(gloas, LightClientHeader::Gloas(_)));

        let fulu = serde_json::to_string(&LightClientHeaderFulu::default()).unwrap();
        let fulu: LightClientHeader = serde_json::from_str(&fulu).unwrap();
        assert!(matches!(fulu, LightClientHeader::Fulu(_)));
    }

    #[test]
    fn test_fulu_format_header_is_rejected_at_gloas() {
        let fulu_header = LightClientHeader::Fulu(LightClientHeaderFulu {
            beacon: header(ATTESTED_GLOAS_HEADER).beacon().clone(),
            ..Default::default()
        });

        assert_eq!(
            is_valid_light_client_header(&client_state(), &fulu_header),
            Err(EthereumIBCError::InvalidLightClientHeaderFormat { slot: 32 })
        );
    }

    #[test]
    fn test_execution_branch_with_invalid_length_is_rejected_when_decoding() {
        let mut json: serde_json::Value = serde_json::from_str(ATTESTED_GLOAS_HEADER).unwrap();
        json["execution_branch"].as_array_mut().unwrap().pop();

        assert!(serde_json::from_value::<LightClientHeaderGloas>(json).is_err());
    }

    #[test]
    fn test_nomralize_merkle_branch() {
        let branch = vec![B256::from_hex(
            "0x75d7411cb01daad167713b5a9b7219670f0e500653cbbcd45cfe1bfe04222459",
        )
        .unwrap()];
        let gindex = 4;
        let normalized = normalize_merkle_branch(&branch, gindex).unwrap();

        let expected_branch = vec![B256::default(), branch[0]];
        assert_eq!(normalized, expected_branch);
    }

    #[test]
    fn test_normalize_merkle_branch_with_no_extra() {
        let branch = vec![B256::default(); 3];
        let gindex = 8;
        let normalized = normalize_merkle_branch(&branch, gindex).unwrap();

        assert_eq!(normalized, branch);
    }

    #[test]
    fn test_normalize_merkle_branch_rejects_invalid_branch() {
        // should fail if num_extra becomes negative (depth < branch.len())
        assert_eq!(
            normalize_merkle_branch(&[B256::default(); 3], 2),
            Err(EthereumIBCError::InvalidMerkleBranchLength {
                expected: 1,
                found: 3
            })
        );
    }
}
