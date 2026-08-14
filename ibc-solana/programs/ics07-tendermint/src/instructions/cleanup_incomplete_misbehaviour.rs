// SPDX-License-Identifier: Apache-2.0

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
        let expected_seeds = &[
            crate::state::MisbehaviourChunk::SEED,
            submitter_key.as_ref(),
            &[index as u8],
        ];
        let (expected_chunk_pda, _) = Pubkey::find_program_address(expected_seeds, &crate::ID);

        if chunk_account.key() != expected_chunk_pda {
            continue;
        }

        if chunk_account.owner == &crate::ID && chunk_account.lamports() > 0 {
            {
                let mut data = chunk_account.try_borrow_mut_data()?;
                data.fill(0);
            }

            let mut lamports = chunk_account.try_borrow_mut_lamports()?;
            let mut submitter_lamports = ctx.accounts.submitter.try_borrow_mut_lamports()?;
            **submitter_lamports = submitter_lamports
                .checked_add(**lamports)
                .ok_or(crate::error::ErrorCode::ArithmeticOverflow)?;
            **lamports = 0;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests;
