// SPDX-License-Identifier: Apache-2.0

use crate::error::ErrorCode;
use crate::helpers::deserialize_misbehaviour;
use crate::state::{ConsensusStateStore, MisbehaviourChunk};
use crate::types::{AppState, ClientState};
use anchor_lang::prelude::*;
use ibc_client_tendermint::types::ConsensusState as IbcConsensusState;
use tendermint_light_client_update_client::ClientState as TmClientState;

/// Reassembles previously uploaded misbehaviour chunks, verifies the evidence
/// against two trusted consensus states and freezes the client on confirmed misbehaviour.
///
/// Remaining accounts must contain misbehaviour chunk PDAs in order, optionally
/// followed by `SignatureVerification` accounts for pre-verified Ed25519 signatures.
#[derive(Accounts)]
#[instruction(chunk_count: u8, trusted_height_1: u64, trusted_height_2: u64)]
pub struct AssembleAndSubmitMisbehaviour<'info> {
    /// PDA holding the light client configuration; frozen when misbehaviour is confirmed.
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

    /// First trusted consensus state referenced by the misbehaviour evidence.
    #[account(
        seeds = [ConsensusStateStore::SEED, &trusted_height_1.to_le_bytes()],
        bump
    )]
    pub trusted_consensus_state_1: Account<'info, ConsensusStateStore>,

    /// Second trusted consensus state referenced by the misbehaviour evidence.
    #[account(
        seeds = [ConsensusStateStore::SEED, &trusted_height_2.to_le_bytes()],
        bump
    )]
    pub trusted_consensus_state_2: Account<'info, ConsensusStateStore>,

    /// Relayer that uploaded the chunks, signs the assembly transaction and receives rent refunds.
    #[account(mut)]
    pub submitter: Signer<'info>,

    /// Instructions sysvar used by the access manager to inspect the transaction.
    /// CHECK: Address constraint verifies this is the instructions sysvar
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instructions_sysvar: AccountInfo<'info>,
    // Remaining accounts are the chunk accounts in order, followed by signature verification accounts.
}

pub fn assemble_and_submit_misbehaviour<'info>(
    mut ctx: Context<'_, '_, 'info, 'info, AssembleAndSubmitMisbehaviour<'info>>,
    chunk_count: u8,
    _trusted_height_1: u64,
    _trusted_height_2: u64,
) -> Result<()> {
    access_manager::require_role(
        &ctx.accounts.access_manager,
        solana_ibc_types::roles::RELAYER_ROLE,
        &ctx.accounts.submitter,
        &ctx.accounts.instructions_sysvar,
        &crate::ID,
    )?;

    require!(
        !ctx.accounts.client_state.is_frozen(),
        ErrorCode::ClientAlreadyFrozen
    );

    let chunk_count = chunk_count as usize;
    let submitter = ctx.accounts.submitter.key();

    let misbehaviour_bytes = assemble_chunks(&ctx, submitter, chunk_count)?;

    let signature_verification_accounts = &ctx.remaining_accounts[chunk_count..];

    process_misbehaviour(
        &mut ctx,
        misbehaviour_bytes,
        signature_verification_accounts,
    )?;

    cleanup_chunks(&ctx, submitter, chunk_count)?;

    Ok(())
}

fn assemble_chunks(
    ctx: &Context<AssembleAndSubmitMisbehaviour>,
    submitter: Pubkey,
    chunk_count: usize,
) -> Result<Vec<u8>> {
    let mut misbehaviour_bytes = Vec::new();

    for (index, chunk_account) in ctx.remaining_accounts[..chunk_count].iter().enumerate() {
        validate_and_load_chunk(
            chunk_account,
            submitter,
            index as u8,
            &mut misbehaviour_bytes,
        )?;
    }

    Ok(misbehaviour_bytes)
}

fn validate_and_load_chunk(
    chunk_account: &AccountInfo,
    submitter: Pubkey,
    index: u8,
    misbehaviour_bytes: &mut Vec<u8>,
) -> Result<()> {
    let expected_seeds = &[
        crate::state::MisbehaviourChunk::SEED,
        submitter.as_ref(),
        &[index],
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

    let chunk: MisbehaviourChunk = MisbehaviourChunk::try_deserialize(&mut &chunk_data[..])?;

    misbehaviour_bytes.extend_from_slice(&chunk.chunk_data);
    Ok(())
}

fn process_misbehaviour<'info>(
    ctx: &mut Context<'_, '_, 'info, 'info, AssembleAndSubmitMisbehaviour<'info>>,
    misbehaviour_bytes: Vec<u8>,
    signature_verification_accounts: &'info [AccountInfo<'info>],
) -> Result<()> {
    let client_state = &ctx.accounts.client_state;

    let misbehaviour = deserialize_misbehaviour(&misbehaviour_bytes)?;
    let tm_client_state: TmClientState = client_state.clone().into_inner().into();

    let trusted_consensus_state_1: IbcConsensusState = ctx
        .accounts
        .trusted_consensus_state_1
        .consensus_state
        .clone()
        .into();
    let trusted_consensus_state_2: IbcConsensusState = ctx
        .accounts
        .trusted_consensus_state_2
        .consensus_state
        .clone()
        .into();

    let current_time = crate::secs_to_nanos(Clock::get()?.unix_timestamp);

    let output = tendermint_light_client_misbehaviour::check_for_misbehaviour(
        &tm_client_state,
        &misbehaviour,
        trusted_consensus_state_1,
        trusted_consensus_state_2,
        current_time,
        signature_verification_accounts,
        &crate::ID,
    )
    .map_err(|_| error!(ErrorCode::MisbehaviourCheckFailed))?;

    require_eq!(
        ctx.accounts.trusted_consensus_state_1.height,
        output.trusted_height_1.revision_height(),
        ErrorCode::HeightMismatch
    );

    require_eq!(
        ctx.accounts.trusted_consensus_state_2.height,
        output.trusted_height_2.revision_height(),
        ErrorCode::HeightMismatch
    );

    // If we reach here, misbehaviour was detected
    ctx.accounts.client_state.freeze();

    msg!(
        "Misbehaviour detected at heights {:?} and {:?}",
        output.trusted_height_1,
        output.trusted_height_2
    );

    Ok(())
}

fn cleanup_chunks(
    ctx: &Context<AssembleAndSubmitMisbehaviour>,
    submitter: Pubkey,
    chunk_count: usize,
) -> Result<()> {
    for (index, chunk_account) in ctx.remaining_accounts[..chunk_count].iter().enumerate() {
        let expected_seeds = &[
            crate::state::MisbehaviourChunk::SEED,
            submitter.as_ref(),
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

        let mut data = chunk_account.try_borrow_mut_data()?;
        data.fill(0);

        let mut lamports = chunk_account.try_borrow_mut_lamports()?;
        let mut submitter_lamports = ctx.accounts.submitter.try_borrow_mut_lamports()?;

        **submitter_lamports = submitter_lamports
            .checked_add(**lamports)
            .ok_or(ErrorCode::ArithmeticOverflow)?;
        **lamports = 0;
    }
    Ok(())
}

#[cfg(test)]
mod tests;
