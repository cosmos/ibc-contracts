// SPDX-License-Identifier: Apache-2.0

use crate::error::ErrorCode;
use crate::helpers::deserialize_header;
use crate::state::{ConsensusStateStore, HeaderChunk, CHUNK_DATA_SIZE};
use crate::types::{AppState, ClientState, ConsensusState, UpdateResult};
use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::set_return_data;
use ibc_client_tendermint::types::{ConsensusState as IbcConsensusState, Header};
use tendermint_light_client_update_client::ClientState as UpdateClientState;

/// Reassembles previously uploaded header chunks, verifies the Tendermint header
/// and updates the light client state.
///
/// Remaining accounts must contain chunk PDAs in order, optionally followed by
/// `SignatureVerification` accounts for pre-verified Ed25519 signatures.
#[derive(Accounts)]
#[instruction(target_height: u64, chunk_count: u8, trusted_height: u64)]
pub struct AssembleAndUpdateClient<'info> {
    /// PDA holding the light client configuration; updated with the new latest height on success.
    #[account(
        mut,
        seeds = [ClientState::SEED],
        bump
    )]
    pub client_state: Account<'info, ClientState>,

    /// PDA holding program-level settings; provides the `access_manager` address for role checks.
    #[account(
        seeds = [AppState::SEED],
        bump
    )]
    pub app_state: Account<'info, AppState>,

    /// Access-manager PDA used to verify the submitter holds the relayer role.
    /// CHECK: Validated by seeds constraint using stored `access_manager` program ID
    #[account(
        seeds = [access_manager::state::AccessManager::SEED],
        bump,
        seeds::program = app_state.am_state.access_manager
    )]
    pub access_manager: AccountInfo<'info>,

    /// Consensus state the header declares as its trust anchor; validated against PDA seeds.
    #[account(
        seeds = [ConsensusStateStore::SEED, &trusted_height.to_le_bytes()],
        bump
    )]
    pub trusted_consensus_state: Account<'info, ConsensusStateStore>,

    /// Destination PDA for the newly derived consensus state; created if it does not already exist.
    #[account(
        init_if_needed,
        payer = submitter,
        space = 8 + ConsensusStateStore::INIT_SPACE,
        seeds = [ConsensusStateStore::SEED, &target_height.to_le_bytes()],
        bump
    )]
    pub new_consensus_state_store: Account<'info, ConsensusStateStore>,

    /// Relayer that uploaded the chunks, signs the assembly transaction and receives rent refunds.
    #[account(mut)]
    pub submitter: Signer<'info>,

    /// Required by Anchor for creating the new consensus-state PDA.
    pub system_program: Program<'info, System>,

    /// Instructions sysvar used by the access manager to inspect the transaction.
    /// CHECK: Address constraint verifies this is the instructions sysvar
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instructions_sysvar: AccountInfo<'info>,
    // Remaining accounts are the chunk accounts in order, followed by signature verification accounts.
    // They will be validated and closed in the instruction handler.
}

impl AssembleAndUpdateClient<'_> {
    /// Number of static accounts (excludes `remaining_accounts` for chunks/sigs)
    pub const STATIC_ACCOUNTS: usize = solana_ibc_constants::ASSEMBLE_UPDATE_CLIENT_STATIC_ACCOUNTS;
}

pub fn assemble_and_update_client<'info>(
    mut ctx: Context<'_, '_, 'info, 'info, AssembleAndUpdateClient<'info>>,
    target_height: u64,
    chunk_count: u8,
    trusted_height: u64,
) -> Result<UpdateResult> {
    access_manager::require_role(
        &ctx.accounts.access_manager,
        solana_ibc_types::roles::RELAYER_ROLE,
        &ctx.accounts.submitter,
        &ctx.accounts.instructions_sysvar,
        &crate::ID,
    )?;

    require!(
        !ctx.accounts.client_state.is_frozen(),
        ErrorCode::ClientFrozen
    );

    let chunk_count = chunk_count as usize;

    let header_bytes = assemble_chunks(&ctx, target_height, chunk_count)?;

    let result = process_header_update(
        &mut ctx,
        header_bytes,
        chunk_count,
        target_height,
        trusted_height,
    )?;

    // Return the UpdateResult as bytes for callers to verify
    set_return_data(&result.try_to_vec()?);

    Ok(result)
}

fn assemble_chunks(
    ctx: &Context<AssembleAndUpdateClient>,
    target_height: u64,
    chunk_count: usize,
) -> Result<Vec<u8>> {
    let submitter = ctx.accounts.submitter.key();
    let header_size = chunk_count * CHUNK_DATA_SIZE;
    let mut header_bytes = Vec::with_capacity(header_size);

    for (index, chunk_account) in ctx.remaining_accounts[..chunk_count].iter().enumerate() {
        let expected_seeds = &[
            crate::state::HeaderChunk::SEED,
            submitter.as_ref(),
            &target_height.to_le_bytes(),
            &[index as u8],
        ];
        let (expected_pda, _) = Pubkey::find_program_address(expected_seeds, &crate::ID);
        require_keys_eq!(
            chunk_account.key(),
            expected_pda,
            ErrorCode::InvalidChunkAccount
        );

        require_keys_eq!(
            *chunk_account.owner,
            crate::ID,
            ErrorCode::InvalidAccountOwner
        );

        let chunk_data = chunk_account.try_borrow_data()?;

        let chunk: HeaderChunk = HeaderChunk::try_deserialize(&mut &chunk_data[..])?;

        header_bytes.extend_from_slice(&chunk.chunk_data);
    }

    Ok(header_bytes)
}

fn process_header_update<'info>(
    ctx: &mut Context<'_, '_, 'info, 'info, AssembleAndUpdateClient<'info>>,
    header_bytes: Vec<u8>,
    chunk_count: usize,
    target_height: u64,
    trusted_height: u64,
) -> Result<UpdateResult> {
    let client_state = &mut ctx.accounts.client_state;

    let header = deserialize_header(&header_bytes)?;

    // Sanity check: Anchor seeds already enforce the PDA, but verify the
    // header's embedded trusted height matches the instruction argument.
    require!(
        header.trusted_height.revision_height() == trusted_height,
        ErrorCode::HeightMismatch
    );

    let trusted_consensus_state = &ctx.accounts.trusted_consensus_state;

    // Signature verification accounts come after chunk accounts
    let signature_verification_accounts = &ctx.remaining_accounts[chunk_count..];

    msg!(
        "Assembly: {} chunks, {} pre-verified sigs",
        chunk_count,
        signature_verification_accounts.len()
    );

    let (new_height, new_consensus_state) = verify_and_update_header(
        client_state,
        &trusted_consensus_state.consensus_state,
        header,
        signature_verification_accounts,
    )?;

    // Sanity check: the Anchor seeds constraint already validates that the
    // new_consensus_state_store PDA matches target_height, but if the header's
    // actual new_height differs from target_height this gives a clearer error.
    require!(
        new_height.revision_height() == target_height,
        ErrorCode::HeightMismatch
    );

    let result = store_consensus_state(StoreConsensusStateParams {
        account: &mut ctx.accounts.new_consensus_state_store,
        height: new_height.revision_height(),
        new_consensus_state: &new_consensus_state,
        trusted_consensus_state: &trusted_consensus_state.consensus_state,
        client_state,
    });

    // Update latest height only on successful update
    if result == UpdateResult::UpdateSuccess {
        client_state.latest_height = new_height.into();
    }

    Ok(result)
}

fn verify_and_update_header<'info>(
    client_state: &crate::types::ClientState,
    trusted_state: &ConsensusState,
    header: Header,
    signature_verification_accounts: &'info [anchor_lang::prelude::AccountInfo<'info>],
) -> Result<(ibc_core_client_types::Height, ConsensusState)> {
    let update_client_state: UpdateClientState = client_state.into();
    let trusted_ibc_state: IbcConsensusState = trusted_state.into();

    let current_time = crate::secs_to_nanos(Clock::get()?.unix_timestamp);

    let output = tendermint_light_client_update_client::update_client(
        &update_client_state,
        &trusted_ibc_state,
        header,
        current_time,
        signature_verification_accounts,
        &crate::ID,
    )
    .map_err(|e| {
        msg!("update_client FAILED: {:?}", e);
        ErrorCode::UpdateClientFailed
    })?;

    let new_consensus_state = output
        .new_consensus_state
        .try_into()
        .map_err(|_| ErrorCode::SerializationError)?;

    Ok((output.latest_height, new_consensus_state))
}

struct StoreConsensusStateParams<'a, 'info> {
    account: &'a mut Account<'info, ConsensusStateStore>,
    height: u64,
    new_consensus_state: &'a ConsensusState,
    trusted_consensus_state: &'a ConsensusState,
    client_state: &'a mut Account<'info, crate::types::ClientState>,
}

fn store_consensus_state(params: StoreConsensusStateParams) -> UpdateResult {
    if !params.account.is_uninitialized() {
        let state_mismatch = params.account.consensus_state != *params.new_consensus_state;
        let timestamp_not_increasing =
            params.trusted_consensus_state.timestamp >= params.new_consensus_state.timestamp;

        if state_mismatch || timestamp_not_increasing {
            params.client_state.freeze();
            return UpdateResult::Misbehaviour;
        }

        return UpdateResult::NoOp;
    }

    params.account.height = params.height;
    params.account.consensus_state = params.new_consensus_state.clone();

    UpdateResult::UpdateSuccess
}

#[cfg(test)]
mod tests;
