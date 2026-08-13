// SPDX-License-Identifier: Apache-2.0

//! This module implements the sync protocol helpers defined in [consensus-specs](https://github.com/ethereum/consensus-specs)

use alloy_primitives::B256;
use ethereum_types::consensus::{
    light_client_header::LightClientHeader,
    merkle::{
        floorlog2, CURRENT_SYNC_COMMITTEE_GINDEX_ELECTRA, EXECUTION_PAYLOAD_GINDEX,
        FINALIZED_ROOT_GINDEX_ELECTRA, NEXT_SYNC_COMMITTEE_GINDEX_ELECTRA,
    },
};
use tree_hash::TreeHash;

use crate::{client_state::ClientState, error::EthereumIBCError, trie::validate_merkle_branch};

// See spec: https://github.com/ethereum/consensus-specs/blob/dev/specs/electra/light-client/sync-protocol.md#modified-finalized_root_gindex_at_slot
/// Returns the finalized root gindex at the given slot.
/// # Errors
/// Returns an error if the epoch is in a non-supported fork.
pub fn finalized_root_gindex_at_slot(
    client_state: &ClientState,
    slot: u64,
) -> Result<u64, EthereumIBCError> {
    let epoch = client_state.compute_epoch_at_slot(slot);
    client_state.verify_supported_fork_at_epoch(epoch)?;

    Ok(FINALIZED_ROOT_GINDEX_ELECTRA)
}

// See spec: https://github.com/ethereum/consensus-specs/blob/dev/specs/electra/light-client/sync-protocol.md#modified-current_sync_committee_gindex_at_slot
/// Returns the current sync committee gindex at the given slot.
/// # Errors
/// Returns an error if the epoch is in a non-supported fork.
pub fn current_sync_committee_gindex_at_slot(
    client_state: &ClientState,
    slot: u64,
) -> Result<u64, EthereumIBCError> {
    let epoch = client_state.compute_epoch_at_slot(slot);
    client_state.verify_supported_fork_at_epoch(epoch)?;

    Ok(CURRENT_SYNC_COMMITTEE_GINDEX_ELECTRA)
}

// See spec: https://github.com/ethereum/consensus-specs/blob/dev/specs/electra/light-client/sync-protocol.md#modified-next_sync_committee_gindex_at_slot
/// Returns the next sync committee gindex at the given slot.
/// # Errors
/// Returns an error if the epoch is in a non-supported fork.
pub fn next_sync_committee_gindex_at_slot(
    client_state: &ClientState,
    slot: u64,
) -> Result<u64, EthereumIBCError> {
    let epoch = client_state.compute_epoch_at_slot(slot);
    client_state.verify_supported_fork_at_epoch(epoch)?;

    Ok(NEXT_SYNC_COMMITTEE_GINDEX_ELECTRA)
}

// See spec: https://github.com/ethereum/consensus-specs/blob/dev/specs/electra/light-client/sync-protocol.md#modified-get_lc_execution_root
/// Returns the execution root of the light client header.
/// # Errors
/// Returns an error if the epoch is in a non-supported fork.
pub fn get_lc_execution_root(
    client_state: &ClientState,
    header: &LightClientHeader,
) -> Result<B256, EthereumIBCError> {
    let epoch = client_state.compute_epoch_at_slot(header.beacon.slot);
    client_state.verify_supported_fork_at_epoch(epoch)?;

    Ok(header.execution.tree_hash_root())
}

// See spec: https://github.com/ethereum/consensus-specs/blob/dev/specs/deneb/light-client/sync-protocol.md#modified-is_valid_light_client_header
/// Validates a light client header.
/// # Errors
/// Returns an error if the header cannot be validated.
pub fn is_valid_light_client_header(
    client_state: &ClientState,
    header: &LightClientHeader,
) -> Result<(), EthereumIBCError> {
    let epoch = client_state.compute_epoch_at_slot(header.beacon.slot);

    client_state.verify_supported_fork_at_epoch(epoch)?;

    validate_merkle_branch(
        get_lc_execution_root(client_state, header)?,
        header.execution_branch.to_vec(),
        floorlog2(EXECUTION_PAYLOAD_GINDEX),
        get_subtree_index(EXECUTION_PAYLOAD_GINDEX),
        header.beacon.body_root,
    )
}

// See spec: <https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/light-client/sync-protocol.md#get_subtree_index>
/// Values that are constant across all configurations.
#[must_use]
pub const fn get_subtree_index(idx: u64) -> u64 {
    idx % 2_u64.pow(idx.ilog2())
}

// See spec: <https://github.com/ethereum/consensus-specs/blob/ff99bc03d6da29d9ef6e055bdb8500e1b2942f1e/specs/electra/light-client/fork.md#L26>
/// Normalize a merkle branch to a depth for given gindex.
/// # Panics
/// Panics if the merkle branch is larger than the calculated depth for the given gindex.
#[must_use]
pub fn normalize_merkle_branch(branch: &[B256], gindex: u64) -> Vec<B256> {
    let depth = floorlog2(gindex);
    let num_extra = depth - branch.len();

    // TODO: Switch to std::iter::repeat_n when cosmwasm supports rust 1.85 (https://github.com/CosmWasm/cosmwasm/issues/2292)
    vec![B256::default(); num_extra]
        .into_iter()
        .chain(branch.to_vec())
        .collect()
}

#[cfg(test)]
mod test {
    use alloy_primitives::{hex::FromHex, B256};

    use crate::sync_protocol_helpers::normalize_merkle_branch;

    #[test]
    fn test_nomralize_merkle_branch() {
        let branch = vec![B256::from_hex(
            "0x75d7411cb01daad167713b5a9b7219670f0e500653cbbcd45cfe1bfe04222459",
        )
        .unwrap()];
        let gindex = 4;
        let normalized = normalize_merkle_branch(&branch, gindex);

        let expected_branch = vec![B256::default(), branch[0]];
        assert_eq!(normalized, expected_branch);
    }

    #[test]
    fn test_normalize_merkle_branch_with_no_extra() {
        let branch = vec![B256::default(); 3];
        let gindex = 8;
        let normalized = normalize_merkle_branch(&branch, gindex);

        assert_eq!(normalized, branch);
    }

    #[test]
    #[should_panic(expected = "attempt to subtract with overflow")]
    fn test_normalize_merkle_branch_panics_on_invalid_branch() {
        // should panic if num_extra becomes negative (depth < branch.len())
        let _ = normalize_merkle_branch(&[B256::default(); 3], 2);
    }
}
