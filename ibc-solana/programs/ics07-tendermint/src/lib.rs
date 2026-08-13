// SPDX-License-Identifier: Apache-2.0

use anchor_lang::prelude::*;

pub mod conversions;
pub mod error;
pub mod events;
pub mod helpers;
pub mod instructions;
pub mod state;
#[cfg(test)]
pub mod test_helpers;
pub mod types;

use instructions::*;

declare_id!("HqPcGpVHxNNFfVatjhG78dFVMwjyZixoKPdZSt3d3TdD");
solana_allocator::custom_heap!();

pub use types::{
    AppState, ClientState, ConsensusState, IbcHeight, UpdateResult, UploadChunkParams,
    UploadMisbehaviourChunkParams,
};

pub use ics25_handler::{MembershipMsg, NonMembershipMsg};

/// Convert nanosecond timestamp to seconds
pub const fn nanos_to_secs(nanos: u64) -> u64 {
    nanos / 1_000_000_000
}

/// Convert seconds to nanoseconds (u128 to avoid overflow)
pub const fn secs_to_nanos(secs: i64) -> u128 {
    secs as u128 * 1_000_000_000
}

#[program]
pub mod ics07_tendermint {
    use super::*;
    use crate::types::{
        ClientState, ConsensusState, UploadChunkParams, UploadMisbehaviourChunkParams,
    };

    pub fn initialize(
        ctx: Context<Initialize>,
        client_state: ClientState,
        consensus_state: ConsensusState,
        access_manager: Pubkey,
    ) -> Result<()> {
        instructions::initialize::initialize(ctx, client_state, consensus_state, access_manager)
    }

    pub fn propose_access_manager_transfer(
        ctx: Context<ProposeAccessManagerTransfer>,
        new_access_manager: Pubkey,
    ) -> Result<()> {
        instructions::access_manager_transfer::propose_access_manager_transfer(
            ctx,
            new_access_manager,
        )
    }

    pub fn accept_access_manager_transfer(ctx: Context<AcceptAccessManagerTransfer>) -> Result<()> {
        instructions::access_manager_transfer::accept_access_manager_transfer(ctx)
    }

    pub fn cancel_access_manager_transfer(ctx: Context<CancelAccessManagerTransfer>) -> Result<()> {
        instructions::access_manager_transfer::cancel_access_manager_transfer(ctx)
    }

    pub fn verify_membership(ctx: Context<VerifyMembership>, msg: MembershipMsg) -> Result<()> {
        instructions::verify_membership::verify_membership(ctx, msg)
    }

    /// Verifies the absence of a value at a given path in the counterparty chain state.
    /// Returns the timestamp of the consensus state at the proof height in unix seconds.
    pub fn verify_non_membership(
        ctx: Context<VerifyNonMembership>,
        msg: NonMembershipMsg,
    ) -> Result<()> {
        instructions::verify_non_membership::verify_non_membership(ctx, msg)
    }

    /// Upload a chunk of header data for multi-transaction updates
    /// Fails if a chunk already exists at this position (no overwrites allowed)
    pub fn upload_header_chunk(
        ctx: Context<UploadHeaderChunk>,
        params: UploadChunkParams,
    ) -> Result<()> {
        instructions::upload_header_chunk::upload_header_chunk(ctx, params)
    }

    /// Assemble chunks and update the client
    /// Automatically cleans up all chunks after successful update
    pub fn assemble_and_update_client<'info>(
        ctx: Context<'_, '_, 'info, 'info, AssembleAndUpdateClient<'info>>,
        target_height: u64,
        chunk_count: u8,
        trusted_height: u64,
    ) -> Result<UpdateResult> {
        instructions::assemble_and_update_client::assemble_and_update_client(
            ctx,
            target_height,
            chunk_count,
            trusted_height,
        )
    }

    /// Clean up incomplete header uploads
    /// This can be called to reclaim rent from failed or abandoned uploads
    /// Closes both `HeaderChunk` and `SignatureVerification` PDAs owned by the submitter
    pub fn cleanup_incomplete_upload(ctx: Context<CleanupIncompleteUpload>) -> Result<()> {
        instructions::cleanup_incomplete_upload::cleanup_incomplete_upload(ctx)
    }

    /// Upload a chunk of misbehaviour data for multi-transaction submission
    pub fn upload_misbehaviour_chunk(
        ctx: Context<UploadMisbehaviourChunk>,
        params: UploadMisbehaviourChunkParams,
    ) -> Result<()> {
        instructions::upload_misbehaviour_chunk::upload_misbehaviour_chunk(ctx, params)
    }

    /// Assemble chunks and submit misbehaviour
    /// Automatically freezes the client and cleans up all chunks
    pub fn assemble_and_submit_misbehaviour<'info>(
        ctx: Context<'_, '_, 'info, 'info, AssembleAndSubmitMisbehaviour<'info>>,
        chunk_count: u8,
        trusted_height_1: u64,
        trusted_height_2: u64,
    ) -> Result<()> {
        instructions::assemble_and_submit_misbehaviour::assemble_and_submit_misbehaviour(
            ctx,
            chunk_count,
            trusted_height_1,
            trusted_height_2,
        )
    }

    /// Clean up incomplete misbehaviour uploads
    /// This can be called to reclaim rent from failed or abandoned misbehaviour submissions
    pub fn cleanup_incomplete_misbehaviour(
        ctx: Context<CleanupIncompleteMisbehaviour>,
    ) -> Result<()> {
        instructions::cleanup_incomplete_misbehaviour::cleanup_incomplete_misbehaviour(ctx)
    }

    pub fn pre_verify_signature<'info>(
        ctx: Context<'_, '_, '_, 'info, PreVerifySignature<'info>>,
        signature: solana_ibc_types::ics07::SignatureData,
    ) -> Result<()> {
        instructions::pre_verify_signatures::pre_verify_signature(ctx, signature)
    }

    pub fn client_status(ctx: Context<ClientStatus>) -> Result<()> {
        instructions::client_status::client_status(ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_static_accounts_matches_idl() {
        let idl_path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../../target/idl/ics07_tendermint.json"
        );
        let Ok(idl) = std::fs::read_to_string(idl_path) else {
            eprintln!("Skipping test: IDL file not found at {idl_path}");
            return;
        };
        let parsed: serde_json::Value = serde_json::from_str(&idl).unwrap();
        let accounts = parsed["instructions"]
            .as_array()
            .unwrap()
            .iter()
            .find(|i| i["name"] == "assemble_and_update_client")
            .unwrap()["accounts"]
            .as_array()
            .unwrap()
            .len();
        assert_eq!(accounts, AssembleAndUpdateClient::STATIC_ACCOUNTS);
    }
}
