// SPDX-License-Identifier: Apache-2.0

//! Contains types and traits for `verify_header` validation within the program.

use std::collections::HashMap;

use ibc_client_tendermint::{
    client_state::ClientState as ClientStateWrapper,
    consensus_state::ConsensusState as ConsensusStateWrapper, types::ConsensusState,
};
use ibc_core_client::context::{ClientValidationContext, ExtClientValidationContext};
use ibc_core_host_types::{
    error::HostError, identifiers::ClientId, path::ClientConsensusStatePath,
};
use ibc_primitives::Timestamp;

/// The client validation context.
pub struct ClientValidationCtx<'a> {
    /// Current time in nanoseconds.
    now: u128,
    trusted_consensus_states: HashMap<ClientConsensusStatePath, &'a ConsensusState>,
}

impl<'a> ClientValidationCtx<'a> {
    /// Create a new instance of the client validation context.
    #[must_use]
    pub fn new(now: u128) -> Self {
        Self {
            now,
            trusted_consensus_states: HashMap::new(),
        }
    }

    /// Insert a trusted consensus state into the context.
    pub fn insert_trusted_consensus_state(
        &mut self,
        client_id: ClientId,
        revision_number: u64,
        revision_height: u64,
        consensus_state: &'a ConsensusState,
    ) {
        self.trusted_consensus_states.insert(
            ClientConsensusStatePath::new(client_id, revision_number, revision_height),
            consensus_state,
        );
    }
}

impl ClientValidationContext for ClientValidationCtx<'_> {
    type ClientStateRef = ClientStateWrapper;
    type ConsensusStateRef = ConsensusStateWrapper;

    fn consensus_state(
        &self,
        path: &ClientConsensusStatePath,
    ) -> Result<Self::ConsensusStateRef, HostError> {
        Ok(self.trusted_consensus_states[path].clone().into())
    }

    fn client_state(
        &self,
        _client_id: &ibc_core_host_types::identifiers::ClientId,
    ) -> Result<Self::ClientStateRef, HostError> {
        // not needed by the `verify_header` function
        unimplemented!()
    }

    fn client_update_meta(
        &self,
        _client_id: &ibc_core_host_types::identifiers::ClientId,
        _height: &ibc_core_client::types::Height,
    ) -> Result<(Timestamp, ibc_core_client::types::Height), HostError> {
        // not needed by the `verify_header` function
        unimplemented!()
    }
}

impl ExtClientValidationContext for ClientValidationCtx<'_> {
    fn host_timestamp(&self) -> Result<Timestamp, HostError> {
        Ok(Timestamp::from_nanoseconds(self.now.try_into().unwrap()))
    }

    fn host_height(&self) -> Result<ibc_core_client::types::Height, HostError> {
        // not needed by the `verify_header` function
        unimplemented!()
    }

    fn consensus_state_heights(
        &self,
        _client_id: &ibc_core_host_types::identifiers::ClientId,
    ) -> Result<Vec<ibc_core_client::types::Height>, HostError> {
        // not needed by the `verify_header` function
        unimplemented!()
    }

    fn next_consensus_state(
        &self,
        _client_id: &ibc_core_host_types::identifiers::ClientId,
        _height: &ibc_core_client::types::Height,
    ) -> Result<Option<Self::ConsensusStateRef>, HostError> {
        // not needed by the `verify_header` function
        unimplemented!()
    }

    fn prev_consensus_state(
        &self,
        _client_id: &ibc_core_host_types::identifiers::ClientId,
        _height: &ibc_core_client::types::Height,
    ) -> Result<Option<Self::ConsensusStateRef>, HostError> {
        // not needed by the `verify_header` function
        unimplemented!()
    }
}
