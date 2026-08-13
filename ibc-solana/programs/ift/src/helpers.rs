// SPDX-License-Identifier: Apache-2.0

//! Shared helper functions for IFT operations

use anchor_lang::prelude::*;
use anchor_spl::token_interface::{self, Mint, MintTo, TokenAccount, TokenInterface};

use anchor_spl::associated_token::get_associated_token_address;
use solana_ibc_gmp_types::{GMPAccount, Salt};
use solana_ibc_proto::{RawGmpSolanaPayload, RawSolanaAccountMeta};

use crate::constants::*;
use crate::errors::IFTError;
use crate::state::IFTAppMintState;

/// Rate-limited mint: checks the daily rate limit, updates usage, then mints
/// tokens via the IFT mint authority PDA.
///
/// Every token mint in the program goes through this function, ensuring no
/// unchecked mint paths exist.
pub fn mint_to_account<'info>(
    mint_state: &mut IFTAppMintState,
    clock: &Clock,
    mint: &mut InterfaceAccount<'info, Mint>,
    token_account: &mut InterfaceAccount<'info, TokenAccount>,
    mint_authority: &AccountInfo<'info>,
    token_program: &Interface<'info, TokenInterface>,
    amount: u64,
) -> Result<()> {
    check_and_update_mint_rate_limit(mint_state, amount, clock)?;

    let mint_key = mint.key();
    let seeds = &[
        MINT_AUTHORITY_SEED,
        mint_key.as_ref(),
        &[mint_state.mint_authority_bump],
    ];
    let signer_seeds = &[&seeds[..]];

    let mint_accounts = MintTo {
        mint: mint.to_account_info(),
        to: token_account.to_account_info(),
        authority: mint_authority.to_account_info(),
    };
    let mint_ctx =
        CpiContext::new_with_signer(token_program.to_account_info(), mint_accounts, signer_seeds);
    token_interface::mint_to(mint_ctx, amount)?;

    mint.reload()?;
    token_account.reload()?;
    Ok(())
}

const fn current_day(clock: &Clock) -> u64 {
    if clock.unix_timestamp < 0 {
        return 0;
    }
    clock.unix_timestamp as u64 / SECONDS_PER_DAY
}

#[allow(clippy::missing_const_for_fn)]
fn maybe_reset_day(mint_state: &mut IFTAppMintState, clock: &Clock) {
    let today = current_day(clock);
    if mint_state.rate_limit_day != today {
        mint_state.rate_limit_day = today;
        mint_state.rate_limit_daily_usage = 0;
    }
}

/// Update mint rate limit usage for `ift_mint`.
/// Resets usage on new day, adds amount, returns error if limit exceeded.
fn check_and_update_mint_rate_limit(
    mint_state: &mut IFTAppMintState,
    amount: u64,
    clock: &Clock,
) -> Result<()> {
    if mint_state.daily_mint_limit == 0 {
        return Ok(());
    }
    maybe_reset_day(mint_state, clock);
    let new_usage = mint_state
        .rate_limit_daily_usage
        .checked_add(amount)
        .ok_or_else(|| error!(IFTError::MintRateLimitExceeded))?;
    require!(
        new_usage <= mint_state.daily_mint_limit,
        IFTError::MintRateLimitExceeded
    );
    mint_state.rate_limit_daily_usage = new_usage;
    Ok(())
}

/// Reduce mint rate limit usage when a transfer completes successfully
/// (tokens permanently left the system).
pub(crate) fn reduce_mint_rate_limit_usage(
    mint_state: &mut IFTAppMintState,
    amount: u64,
    clock: &Clock,
) {
    if mint_state.daily_mint_limit == 0 {
        return;
    }
    maybe_reset_day(mint_state, clock);
    mint_state.rate_limit_daily_usage = mint_state.rate_limit_daily_usage.saturating_sub(amount);
}

// ── Solana mint account derivation ────────────────────────────────────────

/// All derived accounts needed for an `ift_mint` CPI on a counterparty
/// Solana chain.
///
/// Use [`derive`](Self::derive) to compute every PDA from the counterparty's
/// program ID, mint, client ID, source address and receiver. Then call
/// [`to_payload`](Self::to_payload) to build the `RawGmpSolanaPayload`
/// committed in the GMP packet.
///
/// The account order in [`to_payload`](Self::to_payload) matches
/// `instructions::ift_mint::IFTMint<'info>`.
pub struct IftMintAccounts {
    pub app_state: Pubkey,
    pub app_mint_state: Pubkey,
    pub ift_bridge: Pubkey,
    pub mint: Pubkey,
    pub mint_authority: Pubkey,
    pub receiver_ata: Pubkey,
    pub receiver: Pubkey,
    pub gmp_account: Pubkey,
}

impl IftMintAccounts {
    /// Derive all accounts for an `ift_mint` instruction on the counterparty.
    ///
    /// `source_address` is the sender string embedded in the GMP packet
    /// (typically the source IFT program ID as base58).
    pub fn derive(
        ift_program_id: &Pubkey,
        mint: &Pubkey,
        client_id: &str,
        source_address: &str,
        receiver: &Pubkey,
    ) -> Result<Self> {
        let (app_state, _) = Pubkey::find_program_address(&[IFT_APP_STATE_SEED], ift_program_id);
        let (app_mint_state, _) =
            Pubkey::find_program_address(&[IFT_APP_MINT_STATE_SEED, mint.as_ref()], ift_program_id);
        let (ift_bridge, _) = Pubkey::find_program_address(
            &[IFT_BRIDGE_SEED, mint.as_ref(), client_id.as_bytes()],
            ift_program_id,
        );
        let (mint_authority, _) =
            Pubkey::find_program_address(&[MINT_AUTHORITY_SEED, mint.as_ref()], ift_program_id);
        let receiver_ata = get_associated_token_address(receiver, mint);

        let gmp = GMPAccount::new(
            client_id
                .to_string()
                .try_into()
                .map_err(|_| error!(IFTError::InvalidClientIdLength))?,
            source_address
                .to_string()
                .try_into()
                .map_err(|_| error!(IFTError::EmptyCounterpartyAddress))?,
            Salt::empty(),
            &ics27_gmp::ID,
        );

        Ok(Self {
            app_state,
            app_mint_state,
            ift_bridge,
            mint: *mint,
            mint_authority,
            receiver_ata,
            receiver: *receiver,
            gmp_account: gmp.pda,
        })
    }

    /// Build the `RawGmpSolanaPayload` that the GMP program dispatches on
    /// the counterparty chain.
    ///
    /// `ix_data` is the Anchor instruction data for `ift_mint`.
    pub fn to_payload(&self, ix_data: Vec<u8>) -> RawGmpSolanaPayload {
        let meta = |pubkey: &Pubkey, is_signer: bool, is_writable: bool| RawSolanaAccountMeta {
            pubkey: pubkey.to_bytes().to_vec(),
            is_signer,
            is_writable,
        };

        RawGmpSolanaPayload {
            data: ix_data,
            accounts: vec![
                meta(&self.app_state, false, false),
                meta(&self.app_mint_state, false, true),
                meta(&self.ift_bridge, false, false),
                meta(&self.mint, false, true),
                meta(&self.mint_authority, false, false),
                meta(&self.receiver_ata, false, true),
                meta(&self.receiver, false, false),
                meta(&self.gmp_account, true, false),
                meta(&self.gmp_account, true, true),
                meta(&anchor_spl::token::ID, false, false),
                meta(&anchor_spl::associated_token::ID, false, false),
                meta(&anchor_lang::system_program::ID, false, false),
            ],
            prefund_lamports: SOLANA_MINT_PAYLOAD_PREFUND_LAMPORTS,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::AccountVersion;

    fn make_mint_state(limit: u64, day: u64, usage: u64) -> IFTAppMintState {
        IFTAppMintState {
            version: AccountVersion::V1,
            bump: 0,
            mint: Pubkey::new_unique(),
            mint_authority_bump: 0,
            daily_mint_limit: limit,
            rate_limit_day: day,
            rate_limit_daily_usage: usage,
            _reserved: [0; 128],
        }
    }

    fn make_clock(unix_timestamp: i64) -> Clock {
        Clock {
            slot: 0,
            epoch_start_timestamp: 0,
            epoch: 0,
            leader_schedule_epoch: 0,
            unix_timestamp,
        }
    }

    #[test]
    fn test_check_and_update_no_limit() {
        let mut state = make_mint_state(0, 0, 0);
        let clock = make_clock(100_000);
        assert!(check_and_update_mint_rate_limit(&mut state, 999_999, &clock).is_ok());
        assert_eq!(state.rate_limit_daily_usage, 0);
    }

    #[test]
    fn test_check_and_update_within_limit() {
        let day = 100_000i64 / SECONDS_PER_DAY as i64;
        let mut state = make_mint_state(1000, day as u64, 0);
        let clock = make_clock(100_000);
        assert!(check_and_update_mint_rate_limit(&mut state, 500, &clock).is_ok());
        assert_eq!(state.rate_limit_daily_usage, 500);
    }

    #[test]
    fn test_check_and_update_at_limit() {
        let day = 100_000i64 / SECONDS_PER_DAY as i64;
        let mut state = make_mint_state(1000, day as u64, 500);
        let clock = make_clock(100_000);
        assert!(check_and_update_mint_rate_limit(&mut state, 500, &clock).is_ok());
        assert_eq!(state.rate_limit_daily_usage, 1000);
    }

    #[test]
    fn test_check_and_update_exceeds_limit() {
        let day = 100_000i64 / SECONDS_PER_DAY as i64;
        let mut state = make_mint_state(1000, day as u64, 500);
        let clock = make_clock(100_000);
        assert!(check_and_update_mint_rate_limit(&mut state, 501, &clock).is_err());
    }

    #[test]
    fn test_check_and_update_resets_on_new_day() {
        let mut state = make_mint_state(1000, 0, 999);
        let clock = make_clock(SECONDS_PER_DAY as i64); // day 1
        assert!(check_and_update_mint_rate_limit(&mut state, 500, &clock).is_ok());
        assert_eq!(state.rate_limit_daily_usage, 500);
        assert_eq!(state.rate_limit_day, 1);
    }

    #[test]
    fn test_reduce_usage() {
        let mut state = make_mint_state(1000, 1, 800);
        let clock = make_clock(SECONDS_PER_DAY as i64);
        reduce_mint_rate_limit_usage(&mut state, 300, &clock);
        assert_eq!(state.rate_limit_daily_usage, 500);
    }

    #[test]
    fn test_reduce_usage_saturates_to_zero() {
        let mut state = make_mint_state(1000, 1, 100);
        let clock = make_clock(SECONDS_PER_DAY as i64);
        reduce_mint_rate_limit_usage(&mut state, 500, &clock);
        assert_eq!(state.rate_limit_daily_usage, 0);
    }

    #[test]
    fn test_reduce_usage_no_limit() {
        let mut state = make_mint_state(0, 0, 0);
        let clock = make_clock(100_000);
        reduce_mint_rate_limit_usage(&mut state, 500, &clock);
        assert_eq!(state.rate_limit_daily_usage, 0);
    }

    #[test]
    fn test_at_limit_then_one_more_rejected() {
        let day = 1u64;
        let mut state = make_mint_state(1000, day, 1000);
        let clock = make_clock(SECONDS_PER_DAY as i64);
        assert!(check_and_update_mint_rate_limit(&mut state, 1, &clock).is_err());
        assert_eq!(state.rate_limit_daily_usage, 1000);
    }

    #[test]
    fn test_mint_transfer_complete_mint_cycle() {
        let day = 1u64;
        let clock = make_clock(SECONDS_PER_DAY as i64);
        let mut state = make_mint_state(1000, day, 0);

        // Mint 800
        assert!(check_and_update_mint_rate_limit(&mut state, 800, &clock).is_ok());
        assert_eq!(state.rate_limit_daily_usage, 800);

        // Transfer completes successfully — reduce usage
        reduce_mint_rate_limit_usage(&mut state, 300, &clock);
        assert_eq!(state.rate_limit_daily_usage, 500);

        // Mint 500 more — should succeed (500 + 500 = 1000)
        assert!(check_and_update_mint_rate_limit(&mut state, 500, &clock).is_ok());
        assert_eq!(state.rate_limit_daily_usage, 1000);

        // Mint 1 more — should fail
        assert!(check_and_update_mint_rate_limit(&mut state, 1, &clock).is_err());
    }

    #[test]
    fn test_mint_transfer_refund_no_usage_change() {
        let day = 1u64;
        let clock = make_clock(SECONDS_PER_DAY as i64);
        let mut state = make_mint_state(1000, day, 0);

        // Mint to limit
        assert!(check_and_update_mint_rate_limit(&mut state, 1000, &clock).is_ok());

        // Transfer out (no rate limit change at burn time)
        // Refund (no rate limit change — net zero)
        // Usage stays at limit
        assert_eq!(state.rate_limit_daily_usage, 1000);

        // New mint should fail — usage unchanged
        assert!(check_and_update_mint_rate_limit(&mut state, 1, &clock).is_err());
    }

    #[test]
    fn test_reduce_resets_on_new_day() {
        let mut state = make_mint_state(1000, 0, 800);
        let clock = make_clock(SECONDS_PER_DAY as i64); // day 1
        reduce_mint_rate_limit_usage(&mut state, 300, &clock);
        // Day reset to 0, then saturating_sub(0, 300) = 0
        assert_eq!(state.rate_limit_day, 1);
        assert_eq!(state.rate_limit_daily_usage, 0);
    }

    #[test]
    fn test_negative_timestamp_clamps_to_day_zero() {
        let mut state = make_mint_state(1000, 0, 0);
        let clock = make_clock(-86400); // negative timestamp
        assert!(check_and_update_mint_rate_limit(&mut state, 500, &clock).is_ok());
        assert_eq!(state.rate_limit_day, 0);
        assert_eq!(state.rate_limit_daily_usage, 500);
    }

    #[test]
    fn test_checked_add_overflow_returns_error() {
        let day = 1u64;
        let mut state = make_mint_state(u64::MAX, day, u64::MAX - 100);
        let clock = make_clock(SECONDS_PER_DAY as i64);
        assert!(check_and_update_mint_rate_limit(&mut state, 200, &clock).is_err());
        assert_eq!(state.rate_limit_daily_usage, u64::MAX - 100); // unchanged
    }

    // ─── Attack simulation tests ─────────────────────────────────────

    #[test]
    fn test_cycling_attack_blocked_without_success_ack() {
        let day = 1u64;
        let clock = make_clock(SECONDS_PER_DAY as i64);
        let mut state = make_mint_state(1000, day, 0);

        // Mint 1000 (to limit)
        assert!(check_and_update_mint_rate_limit(&mut state, 1000, &clock).is_ok());

        // Attacker burns tokens (transfers out) — no rate limit change
        // Attacker triggers timeout/refund — now rate-limited
        // Simulate: refund mint should fail because usage is already at cap
        assert!(check_and_update_mint_rate_limit(&mut state, 1000, &clock).is_err());
        assert_eq!(state.rate_limit_daily_usage, 1000);

        // Even minting 1 token fails
        assert!(check_and_update_mint_rate_limit(&mut state, 1, &clock).is_err());
    }

    #[test]
    fn test_cycling_with_success_ack_respects_cap() {
        let day = 1u64;
        let clock = make_clock(SECONDS_PER_DAY as i64);
        let mut state = make_mint_state(1000, day, 0);

        // Simulate 5 rounds of mint → success_ack → mint ...
        // Each round: mint full limit, then success ack frees it
        for round in 0..5 {
            assert!(
                check_and_update_mint_rate_limit(&mut state, 1000, &clock).is_ok(),
                "round {round}: mint should succeed after previous ack freed budget"
            );
            assert_eq!(state.rate_limit_daily_usage, 1000);

            // Tokens left the system (success ack)
            reduce_mint_rate_limit_usage(&mut state, 1000, &clock);
            assert_eq!(state.rate_limit_daily_usage, 0);
        }

        // At any point during a round, exceeding the cap is blocked
        assert!(check_and_update_mint_rate_limit(&mut state, 1000, &clock).is_ok());
        assert!(check_and_update_mint_rate_limit(&mut state, 1, &clock).is_err());
    }

    #[test]
    fn test_partial_ack_then_refund_respects_cap() {
        let day = 1u64;
        let clock = make_clock(SECONDS_PER_DAY as i64);
        let mut state = make_mint_state(1000, day, 0);

        // Mint 600 (transfer A)
        assert!(check_and_update_mint_rate_limit(&mut state, 600, &clock).is_ok());
        // Mint 400 (transfer B)
        assert!(check_and_update_mint_rate_limit(&mut state, 400, &clock).is_ok());
        assert_eq!(state.rate_limit_daily_usage, 1000);

        // Transfer A succeeds — frees 600
        reduce_mint_rate_limit_usage(&mut state, 600, &clock);
        assert_eq!(state.rate_limit_daily_usage, 400);

        // Transfer B times out — refund mint consumes 400 more
        assert!(check_and_update_mint_rate_limit(&mut state, 400, &clock).is_ok());
        assert_eq!(state.rate_limit_daily_usage, 800);

        // 200 remaining budget
        assert!(check_and_update_mint_rate_limit(&mut state, 200, &clock).is_ok());
        assert!(check_and_update_mint_rate_limit(&mut state, 1, &clock).is_err());
    }

    #[test]
    fn test_repeated_refund_mints_cumulate_to_limit() {
        let day = 1u64;
        let clock = make_clock(SECONDS_PER_DAY as i64);
        let mut state = make_mint_state(1000, day, 0);

        // Simulate multiple timeouts, each minting a refund
        for i in 0..10 {
            let result = check_and_update_mint_rate_limit(&mut state, 100, &clock);
            assert!(
                result.is_ok(),
                "refund {i} of 100 should succeed (usage {})",
                state.rate_limit_daily_usage
            );
        }
        assert_eq!(state.rate_limit_daily_usage, 1000);

        // 11th refund is blocked
        assert!(check_and_update_mint_rate_limit(&mut state, 100, &clock).is_err());
        assert_eq!(state.rate_limit_daily_usage, 1000);
    }
}
