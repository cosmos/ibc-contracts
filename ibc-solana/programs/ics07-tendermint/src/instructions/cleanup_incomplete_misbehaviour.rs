// SPDX-License-Identifier: Apache-2.0

use crate::helpers::close_account;
use crate::state::MisbehaviourChunk;
use anchor_lang::prelude::*;

/// Closes incomplete `MisbehaviourChunk` PDAs, returning the rent to the original submitter.
///
/// Remaining accounts must be the misbehaviour chunk PDAs to close.
#[derive(Accounts)]
pub struct CleanupIncompleteMisbehaviour<'info> {
    /// Original submitter who uploaded the chunks; must sign to prove ownership and receives rent refunds.
    #[account(mut)]
    pub submitter: Signer<'info>,
}

pub fn cleanup_incomplete_misbehaviour(ctx: Context<CleanupIncompleteMisbehaviour>) -> Result<()> {
    let submitter_key = ctx.accounts.submitter.key();
    for (index, chunk_account) in ctx.remaining_accounts.iter().enumerate() {
        let expected_chunk_pda = MisbehaviourChunk::pda(&submitter_key, index as u8);

        if chunk_account.key() != expected_chunk_pda {
            continue;
        }

        if chunk_account.owner == &crate::ID && chunk_account.lamports() > 0 {
            close_account(chunk_account, &ctx.accounts.submitter)?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests;
