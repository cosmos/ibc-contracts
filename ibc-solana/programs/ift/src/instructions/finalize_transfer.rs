// SPDX-License-Identifier: Apache-2.0

//! Finalize transfer instruction for IFT.
//!
//! This instruction is **permissionless** — anyone can call it, no sender signature
//! is required. It is designed so relayers can finalize transfers without the
//! original sender's involvement. The `payer` (only required signer) receives
//! rent from the closed `PendingTransfer` PDA as an incentive.

use anchor_lang::prelude::*;
use anchor_spl::token_interface::{Mint, TokenAccount, TokenInterface};
use solana_ibc_types::{reject_cpi, CallResultStatus, GMPCallResult};

use crate::constants::{
    IFT_APP_MINT_STATE_SEED, IFT_APP_STATE_SEED, MINT_AUTHORITY_SEED, PENDING_TRANSFER_SEED,
};
use crate::errors::IFTError;
use crate::events::{IFTTransferCompleted, IFTTransferRefunded, RefundReason};
use crate::evm_selectors::ERROR_ACK_COMMITMENT;
use crate::helpers::mint_to_account;
use crate::state::{IFTAppMintState, IFTAppState, PendingTransfer};

/// Accounts for the `finalize_transfer` instruction.
///
/// This instruction is permissionless: the `payer` is the only required signer
/// and does not need to be the original sender. Refund safety is guaranteed by
/// on-chain constraints — the caller cannot redirect funds because
/// `sender_token_account.owner` must equal `pending_transfer.sender`.
#[derive(Accounts)]
#[instruction(client_id: String, sequence: u64)]
pub struct FinalizeTransfer<'info> {
    /// Global IFT app state (read-only)
    #[account(
        seeds = [IFT_APP_STATE_SEED],
        bump = app_state.bump,
        constraint = !app_state.paused @ IFTError::AppPaused
    )]
    pub app_state: Account<'info, IFTAppState>,

    /// Per-mint IFT app state (mut for rate limit updates)
    #[account(
        mut,
        seeds = [IFT_APP_MINT_STATE_SEED, mint.key().as_ref()],
        bump = app_mint_state.bump
    )]
    pub app_mint_state: Account<'info, IFTAppMintState>,

    /// Pending transfer to process
    #[account(
        mut,
        close = payer,
        seeds = [
            PENDING_TRANSFER_SEED,
            app_mint_state.mint.as_ref(),
            client_id.as_bytes(),
            &sequence.to_le_bytes()
        ],
        bump = pending_transfer.bump,
        constraint = pending_transfer.mint == app_mint_state.mint @ IFTError::PendingTransferNotFound,
        constraint = pending_transfer.client_id == client_id @ IFTError::GmpResultClientMismatch,
        constraint = pending_transfer.sequence == sequence @ IFTError::GmpResultSequenceMismatch,
    )]
    pub pending_transfer: Account<'info, PendingTransfer>,

    /// GMP result account - proves the ack/timeout happened
    /// This is a cross-program account owned by the GMP program
    #[account(
        seeds = [GMPCallResult::SEED, client_id.as_bytes(), &sequence.to_le_bytes()],
        seeds::program = ics27_gmp::ID,
        bump,
    )]
    pub gmp_result: Account<'info, ics27_gmp::state::GMPCallResultAccount>,

    /// SPL Token mint
    #[account(mut, address = app_mint_state.mint)]
    pub mint: InterfaceAccount<'info, Mint>,

    /// Mint authority PDA
    /// CHECK: Derived PDA verified by seeds constraint
    #[account(
        seeds = [MINT_AUTHORITY_SEED, mint.key().as_ref()],
        bump = app_mint_state.mint_authority_bump
    )]
    pub mint_authority: AccountInfo<'info>,

    /// Original sender's token account for refunds.
    ///
    /// The caller provides this account but cannot redirect funds — the owner
    /// constraint ensures refunds always go to the original sender. Owning the
    /// `sender_token_account` private key is NOT required; the refund is
    /// authorized by the program via the mint authority PDA.
    ///
    /// If the sender's token account no longer exists (e.g. was closed), the
    /// transaction will fail. A new ATA must be created for the sender before
    /// calling this instruction.
    #[account(
        mut,
        constraint = sender_token_account.mint == mint.key() @ IFTError::TokenAccountOwnerMismatch,
        constraint = sender_token_account.owner == pending_transfer.sender @ IFTError::TokenAccountOwnerMismatch
    )]
    pub sender_token_account: InterfaceAccount<'info, TokenAccount>,

    /// Only required signer. Receives rent from the closed `PendingTransfer`
    /// account as an incentive. Does not need to be the original sender.
    #[account(mut)]
    pub payer: Signer<'info>,

    /// SPL Token or Token 2022 program for refund minting
    pub token_program: Interface<'info, TokenInterface>,
    /// Required by Anchor for the `close` constraint
    pub system_program: Program<'info, System>,

    /// CHECK: Address constraint verifies this is the instructions sysvar
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instructions_sysvar: AccountInfo<'info>,
}

/// Finalize a pending transfer based on GMP result
pub fn finalize_transfer(
    ctx: Context<FinalizeTransfer>,
    client_id: String,
    sequence: u64,
) -> Result<()> {
    reject_cpi(&ctx.accounts.instructions_sysvar, &crate::ID).map_err(IFTError::from)?;

    let pending = &ctx.accounts.pending_transfer;
    let gmp_result = &ctx.accounts.gmp_result;
    let clock = Clock::get()?;

    // Verify the GMP result was initiated by this IFT program.
    // GMP's `send_call_cpi` extracts the calling program ID as the sender.
    require!(
        gmp_result.sender == crate::ID,
        IFTError::GmpResultSenderMismatch
    );
    require!(
        gmp_result.source_client == client_id,
        IFTError::GmpResultClientMismatch
    );
    require!(
        gmp_result.sequence == sequence,
        IFTError::GmpResultSequenceMismatch
    );

    match gmp_result.status {
        CallResultStatus::Timeout => {
            mint_to_account(
                &mut ctx.accounts.app_mint_state,
                &clock,
                &mut ctx.accounts.mint,
                &mut ctx.accounts.sender_token_account,
                &ctx.accounts.mint_authority,
                &ctx.accounts.token_program,
                pending.amount,
            )?;

            emit!(IFTTransferRefunded {
                mint: ctx.accounts.app_mint_state.mint,
                client_id: pending.client_id.clone(),
                sequence: pending.sequence,
                sender: pending.sender,
                amount: pending.amount,
                reason: RefundReason::Timeout,
                timestamp: clock.unix_timestamp,
            });
        }
        CallResultStatus::Acknowledgement(commitment) => {
            if commitment == ERROR_ACK_COMMITMENT {
                mint_to_account(
                    &mut ctx.accounts.app_mint_state,
                    &clock,
                    &mut ctx.accounts.mint,
                    &mut ctx.accounts.sender_token_account,
                    &ctx.accounts.mint_authority,
                    &ctx.accounts.token_program,
                    pending.amount,
                )?;

                emit!(IFTTransferRefunded {
                    mint: ctx.accounts.app_mint_state.mint,
                    client_id: pending.client_id.clone(),
                    sequence: pending.sequence,
                    sender: pending.sender,
                    amount: pending.amount,
                    reason: RefundReason::Failed,
                    timestamp: clock.unix_timestamp,
                });
            } else {
                crate::helpers::reduce_mint_rate_limit_usage(
                    &mut ctx.accounts.app_mint_state,
                    pending.amount,
                    &clock,
                );

                emit!(IFTTransferCompleted {
                    mint: ctx.accounts.app_mint_state.mint,
                    client_id: pending.client_id.clone(),
                    sequence: pending.sequence,
                    sender: pending.sender,
                    amount: pending.amount,
                    timestamp: clock.unix_timestamp,
                });
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use anchor_lang::InstructionData;
    use solana_ibc_types::{packet_acknowledgement_commitment_bytes32, CallResultStatus};
    use solana_sdk::{
        instruction::{AccountMeta, Instruction},
        program_pack::Pack,
        pubkey::Pubkey,
    };

    use crate::errors::IFTError;
    use crate::evm_selectors::ERROR_ACK_COMMITMENT;
    use crate::test_utils::*;

    const TEST_CLIENT_ID: &str = "07-tendermint-0";
    const TEST_SEQUENCE: u64 = 42;
    const TEST_AMOUNT: u64 = 1_000_000;
    const TOKEN_DECIMALS: u8 = 9;

    // Account indices in instruction's account list
    const APP_MINT_STATE_IDX: usize = 1;
    const PENDING_TRANSFER_IDX: usize = 2;
    const MINT_IDX: usize = 4;
    const SENDER_TOKEN_IDX: usize = 6;

    fn empty_pda_account() -> solana_sdk::account::Account {
        solana_sdk::account::Account {
            lamports: 0,
            data: vec![],
            owner: solana_sdk::system_program::ID,
            executable: false,
            rent_epoch: 0,
        }
    }

    struct FinalizeTransferTestSetup {
        instruction: Instruction,
        accounts: Vec<(Pubkey, solana_sdk::account::Account)>,
    }

    struct FinalizeTransferTestSetupParams<'a> {
        status: CallResultStatus,
        gmp_result_sender: Option<Pubkey>,
        gmp_result_client_id: &'a str,
        gmp_result_sequence: u64,
        app_mint_state_override: Option<IftAppMintStateParams>,
        token_owner_override: Option<Pubkey>,
        token_mint_override: Option<Pubkey>,
        app_paused: bool,
    }

    fn build_finalize_transfer_test_setup(
        status: CallResultStatus,
        gmp_result_sender: Option<Pubkey>,
        gmp_result_client_id: &str,
        gmp_result_sequence: u64,
    ) -> FinalizeTransferTestSetup {
        build_finalize_transfer_test_setup_from_params(FinalizeTransferTestSetupParams {
            status,
            gmp_result_sender,
            gmp_result_client_id,
            gmp_result_sequence,
            app_mint_state_override: None,
            token_owner_override: None,
            token_mint_override: None,
            app_paused: false,
        })
    }

    fn build_finalize_transfer_test_setup_from_params(
        params: FinalizeTransferTestSetupParams<'_>,
    ) -> FinalizeTransferTestSetup {
        let FinalizeTransferTestSetupParams {
            status,
            gmp_result_sender,
            gmp_result_client_id,
            gmp_result_sequence,
            app_mint_state_override,
            token_owner_override,
            token_mint_override,
            app_paused,
        } = params;
        let mint = Pubkey::new_unique();
        let sender = Pubkey::new_unique();
        let payer = Pubkey::new_unique();

        let (app_state_pda, app_state_bump) = get_app_state_pda();
        let gmp_result_sender = gmp_result_sender.unwrap_or(crate::ID);
        let (app_mint_state_pda, app_mint_state_bump) = get_app_mint_state_pda(&mint);
        let (mint_authority_pda, mint_authority_bump) = get_mint_authority_pda(&mint);
        let (pending_transfer_pda, pending_transfer_bump) =
            get_pending_transfer_pda(&mint, TEST_CLIENT_ID, TEST_SEQUENCE);
        let (gmp_result_pda, gmp_result_bump) = get_gmp_result_pda(TEST_CLIENT_ID, TEST_SEQUENCE);
        let (system_program, system_account) = create_system_program_account();
        let (token_program_id, token_program_account) = token_program_keyed_account();
        let (sysvar_id, sysvar_account) = create_instructions_sysvar_account();

        let app_state_account = create_ift_app_state_account_with_options(
            app_state_bump,
            Pubkey::new_unique(),
            app_paused,
        );

        let app_mint_state_account = app_mint_state_override.map_or_else(
            || create_ift_app_mint_state_account(mint, app_mint_state_bump, mint_authority_bump),
            |params| {
                create_ift_app_mint_state_account_full(IftAppMintStateParams {
                    mint,
                    bump: app_mint_state_bump,
                    mint_authority_bump,
                    ..params
                })
            },
        );

        let pending_transfer_account = create_pending_transfer_account(
            mint,
            TEST_CLIENT_ID,
            TEST_SEQUENCE,
            sender,
            TEST_AMOUNT,
            pending_transfer_bump,
        );

        let gmp_result_account = create_gmp_result_account(
            gmp_result_sender,
            gmp_result_sequence,
            gmp_result_client_id,
            "dest-client",
            status,
            gmp_result_bump,
        );

        let mint_account = create_mint_account(mint_authority_pda, TOKEN_DECIMALS);

        let token_account_mint = token_mint_override.unwrap_or(mint);
        let token_account_owner = token_owner_override.unwrap_or(sender);
        let sender_token_pda = Pubkey::new_unique();
        let sender_token_account = create_token_account(token_account_mint, token_account_owner, 0);

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new_readonly(app_state_pda, false),
                AccountMeta::new(app_mint_state_pda, false),
                AccountMeta::new(pending_transfer_pda, false),
                AccountMeta::new_readonly(gmp_result_pda, false),
                AccountMeta::new(mint, false),
                AccountMeta::new_readonly(mint_authority_pda, false),
                AccountMeta::new(sender_token_pda, false),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(token_program_id, false),
                AccountMeta::new_readonly(system_program, false),
                AccountMeta::new_readonly(sysvar_id, false),
            ],
            data: crate::instruction::FinalizeTransfer {
                client_id: TEST_CLIENT_ID.to_string(),
                sequence: TEST_SEQUENCE,
            }
            .data(),
        };

        let accounts = vec![
            (app_state_pda, app_state_account),
            (app_mint_state_pda, app_mint_state_account),
            (pending_transfer_pda, pending_transfer_account),
            (gmp_result_pda, gmp_result_account),
            (mint, mint_account),
            (mint_authority_pda, empty_pda_account()),
            (sender_token_pda, sender_token_account),
            (payer, create_signer_account()),
            (token_program_id, token_program_account),
            (system_program, system_account),
            (sysvar_id, sysvar_account),
        ];

        FinalizeTransferTestSetup {
            instruction,
            accounts,
        }
    }

    fn unpack_token_balance(result: &mollusk_svm::result::InstructionResult, index: usize) -> u64 {
        let (_, account) = &result.resulting_accounts[index];
        anchor_spl::token::spl_token::state::Account::unpack(&account.data)
            .expect("valid token account")
            .amount
    }

    fn unpack_mint_supply(result: &mollusk_svm::result::InstructionResult, index: usize) -> u64 {
        let (_, account) = &result.resulting_accounts[index];
        anchor_spl::token::spl_token::state::Mint::unpack(&account.data)
            .expect("valid mint account")
            .supply
    }

    // ─── Constant tests ─────────────────────────────────────────────

    #[test]
    fn test_error_ack_commitment_matches_runtime_computation() {
        let error_ack = solana_ibc_types::UNIVERSAL_ERROR_ACK;
        let computed =
            packet_acknowledgement_commitment_bytes32(std::slice::from_ref(&error_ack.to_vec()))
                .expect("single ack is never empty");

        assert_eq!(
            ERROR_ACK_COMMITMENT, computed,
            "Precomputed ERROR_ACK_COMMITMENT must match runtime computation"
        );
    }

    #[test]
    fn test_error_ack_commitment_is_valid() {
        assert_eq!(ERROR_ACK_COMMITMENT.len(), 32);
        assert!(ERROR_ACK_COMMITMENT.iter().any(|&b| b != 0));
    }

    // ─── Validation failure tests ───────────────────────────────────

    fn assert_finalize_error(
        mollusk: &mollusk_svm::Mollusk,
        setup: &FinalizeTransferTestSetup,
        expected_error: u32,
    ) {
        let result = mollusk.process_instruction(&setup.instruction, &setup.accounts);
        assert_eq!(
            result.program_result,
            Err(solana_sdk::instruction::InstructionError::Custom(
                expected_error
            ))
            .into(),
        );
    }

    #[test]
    fn test_finalize_transfer_wrong_gmp_sender_fails() {
        let mollusk = setup_mollusk();

        let setup = build_finalize_transfer_test_setup(
            CallResultStatus::Timeout,
            Some(Pubkey::new_unique()),
            TEST_CLIENT_ID,
            TEST_SEQUENCE,
        );

        assert_finalize_error(
            &mollusk,
            &setup,
            ANCHOR_ERROR_OFFSET + IFTError::GmpResultSenderMismatch as u32,
        );
    }

    #[test]
    fn test_finalize_transfer_wrong_client_id_fails() {
        let mollusk = setup_mollusk();

        let setup = build_finalize_transfer_test_setup(
            CallResultStatus::Timeout,
            None,
            "wrong-client-id",
            TEST_SEQUENCE,
        );

        assert_finalize_error(
            &mollusk,
            &setup,
            ANCHOR_ERROR_OFFSET + IFTError::GmpResultClientMismatch as u32,
        );
    }

    #[test]
    fn test_finalize_transfer_wrong_sequence_fails() {
        let mollusk = setup_mollusk();

        let setup = build_finalize_transfer_test_setup(
            CallResultStatus::Timeout,
            None,
            TEST_CLIENT_ID,
            999,
        );

        assert_finalize_error(
            &mollusk,
            &setup,
            ANCHOR_ERROR_OFFSET + IFTError::GmpResultSequenceMismatch as u32,
        );
    }

    #[test]
    fn test_finalize_transfer_token_account_wrong_owner_fails() {
        let mollusk = setup_mollusk();

        let setup =
            build_finalize_transfer_test_setup_from_params(FinalizeTransferTestSetupParams {
                status: CallResultStatus::Timeout,
                gmp_result_sender: None,
                gmp_result_client_id: TEST_CLIENT_ID,
                gmp_result_sequence: TEST_SEQUENCE,
                app_mint_state_override: None,
                token_owner_override: Some(Pubkey::new_unique()), // wrong owner
                token_mint_override: None,
                app_paused: false,
            });

        assert_finalize_error(
            &mollusk,
            &setup,
            ANCHOR_ERROR_OFFSET + IFTError::TokenAccountOwnerMismatch as u32,
        );
    }

    #[test]
    fn test_finalize_transfer_token_account_wrong_mint_fails() {
        let mollusk = setup_mollusk();

        let setup =
            build_finalize_transfer_test_setup_from_params(FinalizeTransferTestSetupParams {
                status: CallResultStatus::Timeout,
                gmp_result_sender: None,
                gmp_result_client_id: TEST_CLIENT_ID,
                gmp_result_sequence: TEST_SEQUENCE,
                app_mint_state_override: None,
                token_owner_override: None,
                token_mint_override: Some(Pubkey::new_unique()), // wrong mint
                app_paused: false,
            });

        assert_finalize_error(
            &mollusk,
            &setup,
            ANCHOR_ERROR_OFFSET + IFTError::TokenAccountOwnerMismatch as u32,
        );
    }

    #[test]
    fn test_finalize_transfer_app_paused_fails() {
        let mollusk = setup_mollusk();

        let setup =
            build_finalize_transfer_test_setup_from_params(FinalizeTransferTestSetupParams {
                status: CallResultStatus::Timeout,
                gmp_result_sender: None,
                gmp_result_client_id: TEST_CLIENT_ID,
                gmp_result_sequence: TEST_SEQUENCE,
                app_mint_state_override: None,
                token_owner_override: None,
                token_mint_override: None,
                app_paused: true,
            });

        assert_finalize_error(
            &mollusk,
            &setup,
            ANCHOR_ERROR_OFFSET + IFTError::AppPaused as u32,
        );
    }

    // ─── Happy path tests ───────────────────────────────────────────

    #[test]
    fn test_finalize_transfer_succeeds_after_bridge_removed() {
        let mollusk = setup_mollusk_with_token();

        let setup = build_finalize_transfer_test_setup(
            CallResultStatus::Timeout,
            None,
            TEST_CLIENT_ID,
            TEST_SEQUENCE,
        );

        let result = mollusk.process_instruction(&setup.instruction, &setup.accounts);
        assert!(
            !result.program_result.is_err(),
            "finalization should succeed even when bridge is removed: {:?}",
            result.program_result
        );

        assert_eq!(
            unpack_token_balance(&result, SENDER_TOKEN_IDX),
            TEST_AMOUNT,
            "sender should receive refund even without bridge"
        );

        let (_, pending) = &result.resulting_accounts[PENDING_TRANSFER_IDX];
        assert_eq!(pending.lamports, 0, "pending transfer should be closed");
    }

    #[test]
    fn test_finalize_transfer_timeout_refund_succeeds() {
        let mollusk = setup_mollusk_with_token();

        let setup = build_finalize_transfer_test_setup(
            CallResultStatus::Timeout,
            None,
            TEST_CLIENT_ID,
            TEST_SEQUENCE,
        );

        let result = mollusk.process_instruction(&setup.instruction, &setup.accounts);
        assert!(
            !result.program_result.is_err(),
            "timeout refund should succeed: {:?}",
            result.program_result
        );

        assert_eq!(
            unpack_token_balance(&result, SENDER_TOKEN_IDX),
            TEST_AMOUNT,
            "sender should receive refund"
        );
        assert_eq!(
            unpack_mint_supply(&result, MINT_IDX),
            1_000_000_000 + TEST_AMOUNT,
            "mint supply should increase by refund amount"
        );

        let (_, pending) = &result.resulting_accounts[PENDING_TRANSFER_IDX];
        assert_eq!(pending.lamports, 0, "pending transfer should be closed");
    }

    #[test]
    fn test_finalize_transfer_error_ack_refund_succeeds() {
        let mollusk = setup_mollusk_with_token();

        let setup = build_finalize_transfer_test_setup(
            CallResultStatus::Acknowledgement(ERROR_ACK_COMMITMENT),
            None,
            TEST_CLIENT_ID,
            TEST_SEQUENCE,
        );

        let result = mollusk.process_instruction(&setup.instruction, &setup.accounts);
        assert!(
            !result.program_result.is_err(),
            "error ack refund should succeed: {:?}",
            result.program_result
        );

        assert_eq!(
            unpack_token_balance(&result, SENDER_TOKEN_IDX),
            TEST_AMOUNT,
            "sender should receive refund on error ack"
        );
        assert_eq!(
            unpack_mint_supply(&result, MINT_IDX),
            1_000_000_000 + TEST_AMOUNT,
            "mint supply should increase by refund amount"
        );

        let (_, pending) = &result.resulting_accounts[PENDING_TRANSFER_IDX];
        assert_eq!(pending.lamports, 0, "pending transfer should be closed");
    }

    #[test]
    fn test_finalize_transfer_success_ack_succeeds() {
        let mollusk = setup_mollusk_with_token();

        let success_commitment = [42u8; 32];
        let setup = build_finalize_transfer_test_setup(
            CallResultStatus::Acknowledgement(success_commitment),
            None,
            TEST_CLIENT_ID,
            TEST_SEQUENCE,
        );

        let result = mollusk.process_instruction(&setup.instruction, &setup.accounts);
        assert!(
            !result.program_result.is_err(),
            "success ack should succeed: {:?}",
            result.program_result
        );

        assert_eq!(
            unpack_token_balance(&result, SENDER_TOKEN_IDX),
            0,
            "no tokens should be minted on success"
        );
        assert_eq!(
            unpack_mint_supply(&result, MINT_IDX),
            1_000_000_000,
            "mint supply should be unchanged on success"
        );

        let (_, pending) = &result.resulting_accounts[PENDING_TRANSFER_IDX];
        assert_eq!(pending.lamports, 0, "pending transfer should be closed");
    }

    #[test]
    fn test_finalize_transfer_success_ack_reduces_rate_limit() {
        const RATE_LIMIT_TIMESTAMP: i64 = 1_700_000_000;
        const RATE_LIMIT_DAY: u64 = RATE_LIMIT_TIMESTAMP as u64 / 86400;
        const DAILY_LIMIT: u64 = 10_000_000;
        const INITIAL_USAGE: u64 = 5_000_000;

        let mut mollusk = setup_mollusk_with_token();
        mollusk.sysvars.clock.unix_timestamp = RATE_LIMIT_TIMESTAMP;

        let success_commitment = [42u8; 32];
        let setup =
            build_finalize_transfer_test_setup_from_params(FinalizeTransferTestSetupParams {
                status: CallResultStatus::Acknowledgement(success_commitment),
                gmp_result_sender: None,
                gmp_result_client_id: TEST_CLIENT_ID,
                gmp_result_sequence: TEST_SEQUENCE,
                app_mint_state_override: Some(IftAppMintStateParams {
                    mint: Pubkey::default(),
                    bump: 0,
                    mint_authority_bump: 0,
                    daily_mint_limit: DAILY_LIMIT,
                    rate_limit_day: RATE_LIMIT_DAY,
                    rate_limit_daily_usage: INITIAL_USAGE,
                }),
                token_owner_override: None,
                token_mint_override: None,
                app_paused: false,
            });

        let result = mollusk.process_instruction(&setup.instruction, &setup.accounts);
        assert!(
            !result.program_result.is_err(),
            "success ack with rate limit should succeed: {:?}",
            result.program_result
        );

        let (_, mint_state_account) = &result.resulting_accounts[APP_MINT_STATE_IDX];
        let mint_state: crate::state::IFTAppMintState =
            anchor_lang::AccountDeserialize::try_deserialize(&mut &mint_state_account.data[..])
                .expect("valid IFTAppMintState");

        assert_eq!(
            mint_state.rate_limit_daily_usage,
            INITIAL_USAGE - TEST_AMOUNT,
            "rate limit usage should be reduced by transfer amount"
        );
        assert_eq!(mint_state.rate_limit_day, RATE_LIMIT_DAY);
    }

    #[test]
    fn test_finalize_transfer_timeout_refund_with_existing_balance() {
        const EXISTING_BALANCE: u64 = 500_000;

        let mollusk = setup_mollusk_with_token();
        let mint = Pubkey::new_unique();
        let sender = Pubkey::new_unique();
        let payer = Pubkey::new_unique();

        let (app_state_pda, app_state_bump) = get_app_state_pda();
        let (app_mint_state_pda, app_mint_state_bump) = get_app_mint_state_pda(&mint);
        let (mint_authority_pda, mint_authority_bump) = get_mint_authority_pda(&mint);
        let (pending_transfer_pda, pending_transfer_bump) =
            get_pending_transfer_pda(&mint, TEST_CLIENT_ID, TEST_SEQUENCE);
        let (gmp_result_pda, gmp_result_bump) = get_gmp_result_pda(TEST_CLIENT_ID, TEST_SEQUENCE);
        let (system_program, system_account) = create_system_program_account();
        let (token_program_id, token_program_account) = token_program_keyed_account();
        let (sysvar_id, sysvar_account) = create_instructions_sysvar_account();

        let sender_token_pda = Pubkey::new_unique();

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new_readonly(app_state_pda, false),
                AccountMeta::new(app_mint_state_pda, false),
                AccountMeta::new(pending_transfer_pda, false),
                AccountMeta::new_readonly(gmp_result_pda, false),
                AccountMeta::new(mint, false),
                AccountMeta::new_readonly(mint_authority_pda, false),
                AccountMeta::new(sender_token_pda, false),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(token_program_id, false),
                AccountMeta::new_readonly(system_program, false),
                AccountMeta::new_readonly(sysvar_id, false),
            ],
            data: crate::instruction::FinalizeTransfer {
                client_id: TEST_CLIENT_ID.to_string(),
                sequence: TEST_SEQUENCE,
            }
            .data(),
        };

        let accounts = vec![
            (
                app_state_pda,
                create_ift_app_state_account(app_state_bump, Pubkey::new_unique()),
            ),
            (
                app_mint_state_pda,
                create_ift_app_mint_state_account(mint, app_mint_state_bump, mint_authority_bump),
            ),
            (
                pending_transfer_pda,
                create_pending_transfer_account(
                    mint,
                    TEST_CLIENT_ID,
                    TEST_SEQUENCE,
                    sender,
                    TEST_AMOUNT,
                    pending_transfer_bump,
                ),
            ),
            (
                gmp_result_pda,
                create_gmp_result_account(
                    crate::ID,
                    TEST_SEQUENCE,
                    TEST_CLIENT_ID,
                    "dest-client",
                    CallResultStatus::Timeout,
                    gmp_result_bump,
                ),
            ),
            (
                mint,
                create_mint_account(mint_authority_pda, TOKEN_DECIMALS),
            ),
            (mint_authority_pda, empty_pda_account()),
            (
                sender_token_pda,
                create_token_account(mint, sender, EXISTING_BALANCE),
            ),
            (payer, create_signer_account()),
            (token_program_id, token_program_account),
            (system_program, system_account),
            (sysvar_id, sysvar_account),
        ];

        let result = mollusk.process_instruction(&instruction, &accounts);
        assert!(
            !result.program_result.is_err(),
            "timeout refund with existing balance should succeed: {:?}",
            result.program_result
        );

        assert_eq!(
            unpack_token_balance(&result, SENDER_TOKEN_IDX),
            EXISTING_BALANCE + TEST_AMOUNT,
            "sender should receive refund on top of existing balance"
        );
    }

    #[test]
    fn test_finalize_transfer_success_ack_saturating_rate_limit_reduction() {
        const RATE_LIMIT_TIMESTAMP: i64 = 1_700_000_000;
        const RATE_LIMIT_DAY: u64 = RATE_LIMIT_TIMESTAMP as u64 / 86400;
        const DAILY_LIMIT: u64 = 10_000_000;
        // Usage less than TEST_AMOUNT — reduction should saturate to 0
        const INITIAL_USAGE: u64 = 500_000;

        let mut mollusk = setup_mollusk_with_token();
        mollusk.sysvars.clock.unix_timestamp = RATE_LIMIT_TIMESTAMP;

        let success_commitment = [42u8; 32];
        let setup =
            build_finalize_transfer_test_setup_from_params(FinalizeTransferTestSetupParams {
                status: CallResultStatus::Acknowledgement(success_commitment),
                gmp_result_sender: None,
                gmp_result_client_id: TEST_CLIENT_ID,
                gmp_result_sequence: TEST_SEQUENCE,
                app_mint_state_override: Some(IftAppMintStateParams {
                    mint: Pubkey::default(),
                    bump: 0,
                    mint_authority_bump: 0,
                    daily_mint_limit: DAILY_LIMIT,
                    rate_limit_day: RATE_LIMIT_DAY,
                    rate_limit_daily_usage: INITIAL_USAGE,
                }),
                token_owner_override: None,
                token_mint_override: None,
                app_paused: false,
            });

        let result = mollusk.process_instruction(&setup.instruction, &setup.accounts);
        assert!(
            !result.program_result.is_err(),
            "success ack with saturating rate limit should succeed: {:?}",
            result.program_result
        );

        let (_, mint_state_account) = &result.resulting_accounts[APP_MINT_STATE_IDX];
        let mint_state: crate::state::IFTAppMintState =
            anchor_lang::AccountDeserialize::try_deserialize(&mut &mint_state_account.data[..])
                .expect("valid IFTAppMintState");

        assert_eq!(
            mint_state.rate_limit_daily_usage, 0,
            "rate limit usage should saturate to 0 when amount > usage"
        );
    }

    // ─── Refund rate limit tests ─────────────────────────────────────

    #[test]
    fn test_finalize_transfer_timeout_rate_limit_exceeded() {
        const RATE_LIMIT_TIMESTAMP: i64 = 1_700_000_000;
        const RATE_LIMIT_DAY: u64 = RATE_LIMIT_TIMESTAMP as u64 / 86400;
        const DAILY_LIMIT: u64 = 1_000_000;

        let mut mollusk = setup_mollusk_with_token();
        mollusk.sysvars.clock.unix_timestamp = RATE_LIMIT_TIMESTAMP;

        let setup =
            build_finalize_transfer_test_setup_from_params(FinalizeTransferTestSetupParams {
                status: CallResultStatus::Timeout,
                gmp_result_sender: None,
                gmp_result_client_id: TEST_CLIENT_ID,
                gmp_result_sequence: TEST_SEQUENCE,
                app_mint_state_override: Some(IftAppMintStateParams {
                    mint: Pubkey::default(),
                    bump: 0,
                    mint_authority_bump: 0,
                    daily_mint_limit: DAILY_LIMIT,
                    rate_limit_day: RATE_LIMIT_DAY,
                    rate_limit_daily_usage: DAILY_LIMIT, // already at limit
                }),
                token_owner_override: None,
                token_mint_override: None,
                app_paused: false,
            });

        assert_finalize_error(
            &mollusk,
            &setup,
            ANCHOR_ERROR_OFFSET + IFTError::MintRateLimitExceeded as u32,
        );
    }

    #[test]
    fn test_finalize_transfer_error_ack_rate_limit_exceeded() {
        const RATE_LIMIT_TIMESTAMP: i64 = 1_700_000_000;
        const RATE_LIMIT_DAY: u64 = RATE_LIMIT_TIMESTAMP as u64 / 86400;
        const DAILY_LIMIT: u64 = 1_000_000;

        let mut mollusk = setup_mollusk_with_token();
        mollusk.sysvars.clock.unix_timestamp = RATE_LIMIT_TIMESTAMP;

        let setup =
            build_finalize_transfer_test_setup_from_params(FinalizeTransferTestSetupParams {
                status: CallResultStatus::Acknowledgement(ERROR_ACK_COMMITMENT),
                gmp_result_sender: None,
                gmp_result_client_id: TEST_CLIENT_ID,
                gmp_result_sequence: TEST_SEQUENCE,
                app_mint_state_override: Some(IftAppMintStateParams {
                    mint: Pubkey::default(),
                    bump: 0,
                    mint_authority_bump: 0,
                    daily_mint_limit: DAILY_LIMIT,
                    rate_limit_day: RATE_LIMIT_DAY,
                    rate_limit_daily_usage: DAILY_LIMIT,
                }),
                token_owner_override: None,
                token_mint_override: None,
                app_paused: false,
            });

        assert_finalize_error(
            &mollusk,
            &setup,
            ANCHOR_ERROR_OFFSET + IFTError::MintRateLimitExceeded as u32,
        );
    }

    #[test]
    fn test_finalize_transfer_timeout_increases_rate_limit_usage() {
        const RATE_LIMIT_TIMESTAMP: i64 = 1_700_000_000;
        const RATE_LIMIT_DAY: u64 = RATE_LIMIT_TIMESTAMP as u64 / 86400;
        const DAILY_LIMIT: u64 = 10_000_000;
        const INITIAL_USAGE: u64 = 2_000_000;

        let mut mollusk = setup_mollusk_with_token();
        mollusk.sysvars.clock.unix_timestamp = RATE_LIMIT_TIMESTAMP;

        let setup =
            build_finalize_transfer_test_setup_from_params(FinalizeTransferTestSetupParams {
                status: CallResultStatus::Timeout,
                gmp_result_sender: None,
                gmp_result_client_id: TEST_CLIENT_ID,
                gmp_result_sequence: TEST_SEQUENCE,
                app_mint_state_override: Some(IftAppMintStateParams {
                    mint: Pubkey::default(),
                    bump: 0,
                    mint_authority_bump: 0,
                    daily_mint_limit: DAILY_LIMIT,
                    rate_limit_day: RATE_LIMIT_DAY,
                    rate_limit_daily_usage: INITIAL_USAGE,
                }),
                token_owner_override: None,
                token_mint_override: None,
                app_paused: false,
            });

        let result = mollusk.process_instruction(&setup.instruction, &setup.accounts);
        assert!(
            !result.program_result.is_err(),
            "timeout refund within limit should succeed: {:?}",
            result.program_result
        );

        let (_, mint_state_account) = &result.resulting_accounts[APP_MINT_STATE_IDX];
        let mint_state: crate::state::IFTAppMintState =
            anchor_lang::AccountDeserialize::try_deserialize(&mut &mint_state_account.data[..])
                .expect("valid IFTAppMintState");

        assert_eq!(
            mint_state.rate_limit_daily_usage,
            INITIAL_USAGE + TEST_AMOUNT,
            "timeout refund should increase rate limit usage"
        );
        assert_eq!(mint_state.rate_limit_day, RATE_LIMIT_DAY);
    }

    #[test]
    fn test_finalize_transfer_error_ack_increases_rate_limit_usage() {
        const RATE_LIMIT_TIMESTAMP: i64 = 1_700_000_000;
        const RATE_LIMIT_DAY: u64 = RATE_LIMIT_TIMESTAMP as u64 / 86400;
        const DAILY_LIMIT: u64 = 10_000_000;
        const INITIAL_USAGE: u64 = 2_000_000;

        let mut mollusk = setup_mollusk_with_token();
        mollusk.sysvars.clock.unix_timestamp = RATE_LIMIT_TIMESTAMP;

        let setup =
            build_finalize_transfer_test_setup_from_params(FinalizeTransferTestSetupParams {
                status: CallResultStatus::Acknowledgement(ERROR_ACK_COMMITMENT),
                gmp_result_sender: None,
                gmp_result_client_id: TEST_CLIENT_ID,
                gmp_result_sequence: TEST_SEQUENCE,
                app_mint_state_override: Some(IftAppMintStateParams {
                    mint: Pubkey::default(),
                    bump: 0,
                    mint_authority_bump: 0,
                    daily_mint_limit: DAILY_LIMIT,
                    rate_limit_day: RATE_LIMIT_DAY,
                    rate_limit_daily_usage: INITIAL_USAGE,
                }),
                token_owner_override: None,
                token_mint_override: None,
                app_paused: false,
            });

        let result = mollusk.process_instruction(&setup.instruction, &setup.accounts);
        assert!(
            !result.program_result.is_err(),
            "error ack refund within limit should succeed: {:?}",
            result.program_result
        );

        let (_, mint_state_account) = &result.resulting_accounts[APP_MINT_STATE_IDX];
        let mint_state: crate::state::IFTAppMintState =
            anchor_lang::AccountDeserialize::try_deserialize(&mut &mint_state_account.data[..])
                .expect("valid IFTAppMintState");

        assert_eq!(
            mint_state.rate_limit_daily_usage,
            INITIAL_USAGE + TEST_AMOUNT,
            "error ack refund should increase rate limit usage"
        );
        assert_eq!(mint_state.rate_limit_day, RATE_LIMIT_DAY);
    }

    #[test]
    fn test_finalize_transfer_timeout_no_rate_limit_succeeds() {
        let mollusk = setup_mollusk_with_token();

        let setup =
            build_finalize_transfer_test_setup_from_params(FinalizeTransferTestSetupParams {
                status: CallResultStatus::Timeout,
                gmp_result_sender: None,
                gmp_result_client_id: TEST_CLIENT_ID,
                gmp_result_sequence: TEST_SEQUENCE,
                app_mint_state_override: Some(IftAppMintStateParams {
                    mint: Pubkey::default(),
                    bump: 0,
                    mint_authority_bump: 0,
                    daily_mint_limit: 0, // no rate limit
                    rate_limit_day: 0,
                    rate_limit_daily_usage: 0,
                }),
                token_owner_override: None,
                token_mint_override: None,
                app_paused: false,
            });

        let result = mollusk.process_instruction(&setup.instruction, &setup.accounts);
        assert!(
            !result.program_result.is_err(),
            "timeout refund with no rate limit should succeed: {:?}",
            result.program_result
        );

        let (_, mint_state_account) = &result.resulting_accounts[APP_MINT_STATE_IDX];
        let mint_state: crate::state::IFTAppMintState =
            anchor_lang::AccountDeserialize::try_deserialize(&mut &mint_state_account.data[..])
                .expect("valid IFTAppMintState");

        assert_eq!(
            mint_state.rate_limit_daily_usage, 0,
            "usage should remain 0 when no rate limit configured"
        );
    }

    // ─── Multi-step attack simulation tests ─────────────────────────

    /// Shared context for multi-step `finalize_transfer` tests.
    /// All steps share the same mint, sender and static accounts;
    /// only the sequence (and thus `PendingTransfer` + `GMPCallResult` PDAs) varies.
    struct MultiStepContext {
        mint: Pubkey,
        sender: Pubkey,
        payer: Pubkey,
        app_state_pda: Pubkey,
        app_state_account: solana_sdk::account::Account,
        app_mint_state_pda: Pubkey,
        app_mint_state_bump: u8,
        mint_authority_pda: Pubkey,
        mint_authority_bump: u8,
        sender_token_pda: Pubkey,
        token_program_id: Pubkey,
        token_program_account: solana_sdk::account::Account,
        system_program: Pubkey,
        system_account: solana_sdk::account::Account,
        sysvar_id: Pubkey,
        sysvar_account: solana_sdk::account::Account,
    }

    impl MultiStepContext {
        fn new() -> Self {
            let mint = Pubkey::new_unique();
            let sender = Pubkey::new_unique();
            let payer = Pubkey::new_unique();
            let (app_state_pda, app_state_bump) = get_app_state_pda();
            let (app_mint_state_pda, app_mint_state_bump) = get_app_mint_state_pda(&mint);
            let (mint_authority_pda, mint_authority_bump) = get_mint_authority_pda(&mint);
            let (system_program, system_account) = create_system_program_account();
            let (token_program_id, token_program_account) = token_program_keyed_account();
            let (sysvar_id, sysvar_account) = create_instructions_sysvar_account();

            let app_state_account =
                create_ift_app_state_account(app_state_bump, Pubkey::new_unique());

            Self {
                mint,
                sender,
                payer,
                app_state_pda,
                app_state_account,
                app_mint_state_pda,
                app_mint_state_bump,
                mint_authority_pda,
                mint_authority_bump,
                sender_token_pda: Pubkey::new_unique(),
                token_program_id,
                token_program_account,
                system_program,
                system_account,
                sysvar_id,
                sysvar_account,
            }
        }

        fn initial_app_mint_state(
            &self,
            daily_mint_limit: u64,
            rate_limit_day: u64,
        ) -> solana_sdk::account::Account {
            create_ift_app_mint_state_account_full(IftAppMintStateParams {
                mint: self.mint,
                bump: self.app_mint_state_bump,
                mint_authority_bump: self.mint_authority_bump,
                daily_mint_limit,
                rate_limit_day,
                rate_limit_daily_usage: 0,
            })
        }

        /// Build instruction + accounts for one `finalize_transfer` step.
        /// Mutable accounts (`app_mint_state`, mint, `sender_token`) are passed in
        /// so callers can carry forward state from previous steps.
        fn build_step(
            &self,
            sequence: u64,
            status: CallResultStatus,
            app_mint_state_account: solana_sdk::account::Account,
            mint_account: solana_sdk::account::Account,
            sender_token_account: solana_sdk::account::Account,
        ) -> FinalizeTransferTestSetup {
            let (pending_transfer_pda, pending_transfer_bump) =
                get_pending_transfer_pda(&self.mint, TEST_CLIENT_ID, sequence);
            let (gmp_result_pda, gmp_result_bump) = get_gmp_result_pda(TEST_CLIENT_ID, sequence);

            let instruction = Instruction {
                program_id: crate::ID,
                accounts: vec![
                    AccountMeta::new_readonly(self.app_state_pda, false),
                    AccountMeta::new(self.app_mint_state_pda, false),
                    AccountMeta::new(pending_transfer_pda, false),
                    AccountMeta::new_readonly(gmp_result_pda, false),
                    AccountMeta::new(self.mint, false),
                    AccountMeta::new_readonly(self.mint_authority_pda, false),
                    AccountMeta::new(self.sender_token_pda, false),
                    AccountMeta::new(self.payer, true),
                    AccountMeta::new_readonly(self.token_program_id, false),
                    AccountMeta::new_readonly(self.system_program, false),
                    AccountMeta::new_readonly(self.sysvar_id, false),
                ],
                data: crate::instruction::FinalizeTransfer {
                    client_id: TEST_CLIENT_ID.to_string(),
                    sequence,
                }
                .data(),
            };

            let accounts = vec![
                (self.app_state_pda, self.app_state_account.clone()),
                (self.app_mint_state_pda, app_mint_state_account),
                (
                    pending_transfer_pda,
                    create_pending_transfer_account(
                        self.mint,
                        TEST_CLIENT_ID,
                        sequence,
                        self.sender,
                        TEST_AMOUNT,
                        pending_transfer_bump,
                    ),
                ),
                (
                    gmp_result_pda,
                    create_gmp_result_account(
                        crate::ID,
                        sequence,
                        TEST_CLIENT_ID,
                        "dest-client",
                        status,
                        gmp_result_bump,
                    ),
                ),
                (self.mint, mint_account),
                (self.mint_authority_pda, empty_pda_account()),
                (self.sender_token_pda, sender_token_account),
                (self.payer, create_signer_account()),
                (self.token_program_id, self.token_program_account.clone()),
                (self.system_program, self.system_account.clone()),
                (self.sysvar_id, self.sysvar_account.clone()),
            ];

            FinalizeTransferTestSetup {
                instruction,
                accounts,
            }
        }
    }

    /// Extract the three mutable accounts that carry state between steps.
    fn extract_carried_accounts(
        result: &mollusk_svm::result::InstructionResult,
    ) -> (
        solana_sdk::account::Account,
        solana_sdk::account::Account,
        solana_sdk::account::Account,
    ) {
        let app_mint_state = result.resulting_accounts[APP_MINT_STATE_IDX].1.clone();
        let mint = result.resulting_accounts[MINT_IDX].1.clone();
        let sender_token = result.resulting_accounts[SENDER_TOKEN_IDX].1.clone();
        (app_mint_state, mint, sender_token)
    }

    #[test]
    fn test_repeated_timeout_refunds_hit_rate_limit() {
        const RATE_LIMIT_TIMESTAMP: i64 = 1_700_000_000;
        const RATE_LIMIT_DAY: u64 = RATE_LIMIT_TIMESTAMP as u64 / 86400;
        const DAILY_LIMIT: u64 = 3 * TEST_AMOUNT;

        let mut mollusk = setup_mollusk_with_token();
        mollusk.sysvars.clock.unix_timestamp = RATE_LIMIT_TIMESTAMP;

        let ctx = MultiStepContext::new();
        let mut app_mint_state = ctx.initial_app_mint_state(DAILY_LIMIT, RATE_LIMIT_DAY);
        let mut mint_account = create_mint_account(ctx.mint_authority_pda, TOKEN_DECIMALS);
        let mut sender_token = create_token_account(ctx.mint, ctx.sender, 0);

        // 3 timeout refunds of TEST_AMOUNT each should succeed (3 * 1M = 3M = limit)
        for seq in 1..=3 {
            let setup = ctx.build_step(
                seq,
                CallResultStatus::Timeout,
                app_mint_state.clone(),
                mint_account.clone(),
                sender_token.clone(),
            );
            let result = mollusk.process_instruction(&setup.instruction, &setup.accounts);
            assert!(
                !result.program_result.is_err(),
                "timeout refund #{seq} should succeed: {:?}",
                result.program_result
            );
            (app_mint_state, mint_account, sender_token) = extract_carried_accounts(&result);
        }

        // Verify rate limit is exactly at cap
        let mint_state = deserialize_app_mint_state(&app_mint_state);
        assert_eq!(mint_state.rate_limit_daily_usage, DAILY_LIMIT);

        // 4th timeout refund should be blocked by rate limit
        let setup = ctx.build_step(
            4,
            CallResultStatus::Timeout,
            app_mint_state,
            mint_account,
            sender_token,
        );
        let result = mollusk.process_instruction(&setup.instruction, &setup.accounts);
        assert_eq!(
            result.program_result,
            Err(solana_sdk::instruction::InstructionError::Custom(
                ANCHOR_ERROR_OFFSET + IFTError::MintRateLimitExceeded as u32,
            ))
            .into(),
            "4th timeout refund should be blocked by rate limit"
        );
    }

    #[test]
    fn test_mixed_timeout_and_error_ack_refunds_cumulate() {
        const RATE_LIMIT_TIMESTAMP: i64 = 1_700_000_000;
        const RATE_LIMIT_DAY: u64 = RATE_LIMIT_TIMESTAMP as u64 / 86400;
        const DAILY_LIMIT: u64 = 2 * TEST_AMOUNT;

        let mut mollusk = setup_mollusk_with_token();
        mollusk.sysvars.clock.unix_timestamp = RATE_LIMIT_TIMESTAMP;

        let ctx = MultiStepContext::new();
        let mut app_mint_state = ctx.initial_app_mint_state(DAILY_LIMIT, RATE_LIMIT_DAY);
        let mut mint_account = create_mint_account(ctx.mint_authority_pda, TOKEN_DECIMALS);
        let mut sender_token = create_token_account(ctx.mint, ctx.sender, 0);

        // Step 1: timeout refund (usage: 0 → 1M)
        let setup = ctx.build_step(
            1,
            CallResultStatus::Timeout,
            app_mint_state,
            mint_account,
            sender_token,
        );
        let result = mollusk.process_instruction(&setup.instruction, &setup.accounts);
        assert!(
            !result.program_result.is_err(),
            "step 1 timeout: {:?}",
            result.program_result
        );
        (app_mint_state, mint_account, sender_token) = extract_carried_accounts(&result);

        let mint_state = deserialize_app_mint_state(&app_mint_state);
        assert_eq!(mint_state.rate_limit_daily_usage, TEST_AMOUNT);

        // Step 2: error ack refund (usage: 1M → 2M = limit)
        let setup = ctx.build_step(
            2,
            CallResultStatus::Acknowledgement(ERROR_ACK_COMMITMENT),
            app_mint_state,
            mint_account,
            sender_token,
        );
        let result = mollusk.process_instruction(&setup.instruction, &setup.accounts);
        assert!(
            !result.program_result.is_err(),
            "step 2 error ack: {:?}",
            result.program_result
        );
        (app_mint_state, mint_account, sender_token) = extract_carried_accounts(&result);

        let mint_state = deserialize_app_mint_state(&app_mint_state);
        assert_eq!(mint_state.rate_limit_daily_usage, DAILY_LIMIT);

        // Step 3: another timeout should be blocked
        let setup = ctx.build_step(
            3,
            CallResultStatus::Timeout,
            app_mint_state,
            mint_account,
            sender_token,
        );
        let result = mollusk.process_instruction(&setup.instruction, &setup.accounts);
        assert_eq!(
            result.program_result,
            Err(solana_sdk::instruction::InstructionError::Custom(
                ANCHOR_ERROR_OFFSET + IFTError::MintRateLimitExceeded as u32,
            ))
            .into(),
            "step 3 should be blocked after timeout + error ack filled budget"
        );
    }

    #[test]
    fn test_success_ack_frees_budget_for_subsequent_timeout_refund() {
        const RATE_LIMIT_TIMESTAMP: i64 = 1_700_000_000;
        const RATE_LIMIT_DAY: u64 = RATE_LIMIT_TIMESTAMP as u64 / 86400;
        const DAILY_LIMIT: u64 = TEST_AMOUNT;

        let mut mollusk = setup_mollusk_with_token();
        mollusk.sysvars.clock.unix_timestamp = RATE_LIMIT_TIMESTAMP;

        let ctx = MultiStepContext::new();
        let mut app_mint_state = ctx.initial_app_mint_state(DAILY_LIMIT, RATE_LIMIT_DAY);
        let mut mint_account = create_mint_account(ctx.mint_authority_pda, TOKEN_DECIMALS);
        let mut sender_token = create_token_account(ctx.mint, ctx.sender, 0);

        // Step 1: timeout refund fills limit (usage: 0 → 1M)
        let setup = ctx.build_step(
            1,
            CallResultStatus::Timeout,
            app_mint_state,
            mint_account,
            sender_token,
        );
        let result = mollusk.process_instruction(&setup.instruction, &setup.accounts);
        assert!(
            !result.program_result.is_err(),
            "step 1: {:?}",
            result.program_result
        );
        (app_mint_state, mint_account, sender_token) = extract_carried_accounts(&result);

        // Step 2: another timeout should fail (at limit)
        let setup = ctx.build_step(
            2,
            CallResultStatus::Timeout,
            app_mint_state.clone(),
            mint_account.clone(),
            sender_token.clone(),
        );
        let result = mollusk.process_instruction(&setup.instruction, &setup.accounts);
        assert_eq!(
            result.program_result,
            Err(solana_sdk::instruction::InstructionError::Custom(
                ANCHOR_ERROR_OFFSET + IFTError::MintRateLimitExceeded as u32,
            ))
            .into(),
            "step 2 should be blocked — budget full"
        );

        // Step 3: success ack frees the budget (usage: 1M → 0)
        let success_commitment = [42u8; 32];
        let setup = ctx.build_step(
            3,
            CallResultStatus::Acknowledgement(success_commitment),
            app_mint_state,
            mint_account,
            sender_token,
        );
        let result = mollusk.process_instruction(&setup.instruction, &setup.accounts);
        assert!(
            !result.program_result.is_err(),
            "step 3: {:?}",
            result.program_result
        );
        (app_mint_state, mint_account, sender_token) = extract_carried_accounts(&result);

        let mint_state = deserialize_app_mint_state(&app_mint_state);
        assert_eq!(mint_state.rate_limit_daily_usage, 0);

        // Step 4: timeout refund now succeeds (usage: 0 → 1M)
        let setup = ctx.build_step(
            4,
            CallResultStatus::Timeout,
            app_mint_state,
            mint_account,
            sender_token,
        );
        let result = mollusk.process_instruction(&setup.instruction, &setup.accounts);
        assert!(
            !result.program_result.is_err(),
            "step 4: {:?}",
            result.program_result
        );

        let (carried_mint_state, _, _) = extract_carried_accounts(&result);
        let mint_state = deserialize_app_mint_state(&carried_mint_state);
        assert_eq!(mint_state.rate_limit_daily_usage, DAILY_LIMIT);
    }

    #[test]
    fn test_success_ack_does_not_allow_unlimited_cycling() {
        const RATE_LIMIT_TIMESTAMP: i64 = 1_700_000_000;
        const RATE_LIMIT_DAY: u64 = RATE_LIMIT_TIMESTAMP as u64 / 86400;
        const DAILY_LIMIT: u64 = TEST_AMOUNT;

        let mut mollusk = setup_mollusk_with_token();
        mollusk.sysvars.clock.unix_timestamp = RATE_LIMIT_TIMESTAMP;

        let ctx = MultiStepContext::new();
        let mut app_mint_state = ctx.initial_app_mint_state(DAILY_LIMIT, RATE_LIMIT_DAY);
        let mut mint_account = create_mint_account(ctx.mint_authority_pda, TOKEN_DECIMALS);
        let mut sender_token = create_token_account(ctx.mint, ctx.sender, 0);

        let success_commitment = [42u8; 32];
        let mut seq = 0u64;

        // 5 rounds of: timeout refund (fills limit) → success ack (frees it)
        // This is legitimate — tokens genuinely left the system each round.
        // At no point can the usage exceed DAILY_LIMIT.
        for round in 0..5 {
            seq += 1;
            let setup = ctx.build_step(
                seq,
                CallResultStatus::Timeout,
                app_mint_state,
                mint_account,
                sender_token,
            );
            let result = mollusk.process_instruction(&setup.instruction, &setup.accounts);
            assert!(
                !result.program_result.is_err(),
                "round {round} timeout: {:?}",
                result.program_result
            );
            (app_mint_state, mint_account, sender_token) = extract_carried_accounts(&result);

            let state = deserialize_app_mint_state(&app_mint_state);
            assert_eq!(
                state.rate_limit_daily_usage, DAILY_LIMIT,
                "round {round}: usage should be at limit after timeout refund"
            );

            // Cannot mint more while at limit
            seq += 1;
            let setup_blocked = ctx.build_step(
                seq,
                CallResultStatus::Timeout,
                app_mint_state.clone(),
                mint_account.clone(),
                sender_token.clone(),
            );
            let result_blocked =
                mollusk.process_instruction(&setup_blocked.instruction, &setup_blocked.accounts);
            assert!(
                result_blocked.program_result.is_err(),
                "round {round}: should be blocked while at limit"
            );

            // Success ack frees budget
            seq += 1;
            let setup = ctx.build_step(
                seq,
                CallResultStatus::Acknowledgement(success_commitment),
                app_mint_state,
                mint_account,
                sender_token,
            );
            let result = mollusk.process_instruction(&setup.instruction, &setup.accounts);
            assert!(
                !result.program_result.is_err(),
                "round {round} success ack: {:?}",
                result.program_result
            );
            (app_mint_state, mint_account, sender_token) = extract_carried_accounts(&result);

            let state = deserialize_app_mint_state(&app_mint_state);
            assert_eq!(
                state.rate_limit_daily_usage, 0,
                "round {round}: usage should be 0 after success ack"
            );
        }
    }
}
