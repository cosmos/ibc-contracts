// SPDX-License-Identifier: Apache-2.0

//! This module provides [`verify_misbehaviour`] function to check for misbehaviour

use ethereum_types::consensus::light_client_header::LightClientUpdate;

use crate::{
    client_state::ClientState,
    consensus_state::{ConsensusState, TrustedConsensusState},
    error::EthereumIBCError,
    header::ActiveSyncCommittee,
    verify::{validate_light_client_update, BlsVerify},
};

/// Verifies if a consensus misbehaviour is valid by checking if the two conflicting light client updates are valid.
///
/// * `client_state`: The current client state.
/// * `consensus_state`: The current consensus state (previously verified and stored)
/// * `full_sync_committee`: The full sync committee data. (untrusted)
/// * `update_1`: The first light client update.
/// * `update_2`: The second light client update.
/// * `current_slot`: The slot number computed based on the current timestamp.
/// * `bls_verifier`: BLS verification implementation.
///
/// # Errors
/// Returns an error if the misbehaviour cannot be verified.
#[allow(clippy::module_name_repetitions, clippy::needless_pass_by_value)]
pub fn verify_misbehaviour<V: BlsVerify>(
    client_state: &ClientState,
    consensus_state: &ConsensusState,
    full_sync_committee: &ActiveSyncCommittee,
    update_1: &LightClientUpdate,
    update_2: &LightClientUpdate,
    current_timestamp: u64,
    bls_verifier: V,
) -> Result<(), EthereumIBCError> {
    let trusted_consensus_state = TrustedConsensusState::new(
        client_state,
        consensus_state.clone(),
        full_sync_committee.clone(),
        &bls_verifier,
    )?;

    // There is no point to check for misbehaviour when the headers are not for the same height
    let (slot_1, slot_2) = (
        update_1.attested_header.beacon.slot,
        update_2.attested_header.beacon.slot,
    );
    ensure!(
        slot_1 == slot_2,
        EthereumIBCError::MisbehaviourSlotMismatch(slot_1, slot_2)
    );

    let (state_root_1, state_root_2) = (
        update_1.attested_header.execution.state_root,
        update_2.attested_header.execution.state_root,
    );
    ensure!(
        state_root_1 != state_root_2,
        EthereumIBCError::MisbehaviourStorageRootsMatch(state_root_1)
    );

    let current_slot = client_state
        .compute_slot_at_timestamp(current_timestamp)
        .ok_or(EthereumIBCError::FailedToComputeSlotAtTimestamp {
            timestamp: current_timestamp,
            genesis: client_state.genesis_time,
            seconds_per_slot: client_state.seconds_per_slot,
            genesis_slot: client_state.genesis_slot,
        })?;

    validate_light_client_update::<V>(
        client_state,
        &trusted_consensus_state,
        update_1,
        current_slot,
        &bls_verifier,
    )?;

    validate_light_client_update::<V>(
        client_state,
        &trusted_consensus_state,
        update_2,
        current_slot,
        &bls_verifier,
    )?;

    Ok(())
}
