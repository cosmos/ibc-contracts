// SPDX-License-Identifier: Apache-2.0

//! This module defines constants related to merkle trees in the Ethereum consensus.

// New constants
/// `get_generalized_index(BeaconState, 'finalized_checkpoint', 'root')` (= 169)
pub const FINALIZED_ROOT_GINDEX_ELECTRA: u64 = 169;
/// `get_generalized_index(BeaconState, 'current_sync_committee')` (= 86)
pub const CURRENT_SYNC_COMMITTEE_GINDEX_ELECTRA: u64 = 86;
/// `get_generalized_index(BeaconState, 'next_sync_committee')` (= 87)
pub const NEXT_SYNC_COMMITTEE_GINDEX_ELECTRA: u64 = 87;

// https://github.com/ethereum/consensus-specs/blob/dev/specs/capella/light-client/sync-protocol.md#constants
/// `get_generalized_index(BeaconBlockBody, 'execution_payload')` (= 25)
pub const EXECUTION_PAYLOAD_GINDEX: u64 = 25;

/// Convenience function safely to call [`u64::ilog2`] and convert the result into a usize.
#[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
#[must_use]
pub const fn floorlog2(n: u64) -> usize {
    // conversion is safe since usize is either 32 or 64 bits as per cfg above
    n.ilog2() as usize
}
