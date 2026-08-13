// SPDX-License-Identifier: Apache-2.0

//! This module provides [`update_consensus_state`] function to update the consensus state

use crate::{
    client_state::ClientState, consensus_state::ConsensusState, error::EthereumIBCError,
    header::Header,
};

/// Takes in the current client and consensus state and a new header and returns the updated
/// consensus state and optionally the updated client state (if it needs to be updated)
///
/// # Returns
/// Returns the updated slot, the updated consensus state and optionally the updated client state.
///
/// Current implementation requires the client and consensus states to be updated since historical updates are not allowed.
///
/// # Errors
/// Returns an error if the store period is not equal to the finalized period
#[allow(clippy::module_name_repetitions, clippy::needless_pass_by_value)]
pub fn update_consensus_state(
    current_consensus_state: ConsensusState,
    current_client_state: ClientState,
    header: Header,
) -> Result<(u64, ConsensusState, Option<ClientState>), EthereumIBCError> {
    ensure!(
        current_client_state.latest_slot == current_consensus_state.slot,
        EthereumIBCError::ClientAndConsensusSlotMismatch {
            client_state_slot: current_client_state.latest_slot,
            consensus_state_slot: current_consensus_state.slot
        }
    );

    let store_slot = current_consensus_state.slot;
    let store_period = current_client_state.compute_sync_committee_period_at_slot(store_slot);

    let update_finalized_slot = header.consensus_update.finalized_header.beacon.slot;
    let update_finalized_period =
        current_client_state.compute_sync_committee_period_at_slot(update_finalized_slot);

    let mut new_consensus_state = current_consensus_state.clone();

    if let Some(next_sync_committee) = current_consensus_state.next_sync_committee {
        // sync committee only changes when the period change
        if update_finalized_period == store_period + 1 {
            new_consensus_state.current_sync_committee = next_sync_committee;
            new_consensus_state.next_sync_committee = header
                .consensus_update
                .next_sync_committee
                .map(|c| c.to_summarized_sync_committee());
        }
    } else {
        // if the finalized period is greater, we have to have a next sync committee
        ensure!(
            update_finalized_period == store_period,
            EthereumIBCError::StorePeriodMustBeEqualToFinalizedPeriod
        );
        new_consensus_state.next_sync_committee = header
            .consensus_update
            .next_sync_committee
            .map(|c| c.to_summarized_sync_committee());
    }

    new_consensus_state.slot = update_finalized_slot;
    new_consensus_state.state_root = header
        .consensus_update
        .finalized_header
        .execution
        .state_root;
    new_consensus_state.timestamp = header.consensus_update.finalized_header.execution.timestamp;

    let new_client_state =
        (update_finalized_slot > current_consensus_state.slot).then_some(ClientState {
            latest_slot: update_finalized_slot,
            latest_execution_block_number: header
                .consensus_update
                .finalized_header
                .execution
                .block_number,
            ..current_client_state
        });
    Ok((update_finalized_slot, new_consensus_state, new_client_state))
}
