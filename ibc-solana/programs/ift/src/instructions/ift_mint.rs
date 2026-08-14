// SPDX-License-Identifier: Apache-2.0

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};

use crate::constants::*;
use crate::errors::IFTError;
use crate::events::IFTMintReceived;
use crate::helpers::mint_to_account;
use crate::state::{IFTAppMintState, IFTAppState, IFTBridge, IFTMintMsg};

/// IFT Mint instruction - called by GMP via CPI when receiving a cross-chain mint request.
#[derive(Accounts)]
#[instruction(msg: IFTMintMsg)]
pub struct IFTMint<'info> {
    /// Global IFT app state (read-only, for paused check)
    #[account(
        seeds = [IFT_APP_STATE_SEED],
        bump = app_state.bump,
        constraint = !app_state.paused @ IFTError::AppPaused,
    )]
    pub app_state: Account<'info, IFTAppState>,

    /// Per-mint IFT app state (mut, for rate limits)
    #[account(
        mut,
        seeds = [IFT_APP_MINT_STATE_SEED, mint.key().as_ref()],
        bump = app_mint_state.bump
    )]
    pub app_mint_state: Account<'info, IFTAppMintState>,

    /// IFT bridge - provides counterparty info for GMP account validation.
    /// Seeds use self-referencing `ift_bridge.client_id` (Anchor deserializes before checking seeds).
    #[account(
        seeds = [IFT_BRIDGE_SEED, app_mint_state.mint.as_ref(), ift_bridge.client_id.as_bytes()],
        bump = ift_bridge.bump,
        constraint = ift_bridge.mint == app_mint_state.mint @ IFTError::InvalidBridge,
        constraint = ift_bridge.active @ IFTError::BridgeNotActive
    )]
    pub ift_bridge: Account<'info, IFTBridge>,

    /// SPL Token mint
    #[account(
        mut,
        address = app_mint_state.mint
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    /// Mint authority PDA
    /// CHECK: Derived PDA that signs for minting
    #[account(
        seeds = [MINT_AUTHORITY_SEED, mint.key().as_ref()],
        // Use stored bump to avoid expensive `find_program_address` computation.
        // The bump is needed for PDA signing during the mint CPI call.
        bump = app_mint_state.mint_authority_bump
    )]
    pub mint_authority: AccountInfo<'info>,

    /// Receiver's token account (will be created if needed)
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = mint,
        associated_token::authority = receiver_owner,
        associated_token::token_program = token_program,
    )]
    pub receiver_token_account: InterfaceAccount<'info, TokenAccount>,

    /// CHECK: Receiver who will own the minted tokens.
    /// Constraint prevents relayer from substituting a different receiver than specified in cross-chain message.
    #[account(
        constraint = receiver_owner.key() == msg.receiver @ IFTError::InvalidReceiver
    )]
    pub receiver_owner: AccountInfo<'info>,

    /// GMP account PDA - validated to match counterparty bridge
    pub gmp_account: Signer<'info>,

    /// Pays for ATA creation (if needed) and transaction fees
    #[account(mut)]
    pub payer: Signer<'info>,

    /// SPL Token or Token 2022 program for the mint CPI
    pub token_program: Interface<'info, TokenInterface>,
    /// Creates the receiver's associated token account if it doesn't exist
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// Required for ATA creation
    pub system_program: Program<'info, System>,
}

pub fn ift_mint(ctx: Context<IFTMint>, msg: IFTMintMsg) -> Result<()> {
    let clock = Clock::get()?;
    let bridge = &ctx.accounts.ift_bridge;

    require!(msg.amount > 0, IFTError::ZeroAmount);

    // Validate GMP account matches the bridge's (client_id, counterparty_ift_address).
    // This ensures the relayer passed the correct bridge for this GMP call.
    validate_gmp_account(
        &ctx.accounts.gmp_account.key(),
        &bridge.client_id,
        &bridge.counterparty_ift_address,
    )?;

    mint_to_account(
        &mut ctx.accounts.app_mint_state,
        &clock,
        &mut ctx.accounts.mint,
        &mut ctx.accounts.receiver_token_account,
        &ctx.accounts.mint_authority,
        &ctx.accounts.token_program,
        msg.amount,
    )?;

    emit!(IFTMintReceived {
        mint: ctx.accounts.mint.key(),
        client_id: bridge.client_id.clone(),
        receiver: msg.receiver,
        amount: msg.amount,
        gmp_account: ctx.accounts.gmp_account.key(),
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

fn validate_gmp_account(
    gmp_account: &Pubkey,
    client_id: &str,
    counterparty_address: &str,
) -> Result<()> {
    use solana_ibc_gmp_types::{GMPAccount, Salt};

    let gmp = GMPAccount::new(
        client_id
            .to_string()
            .try_into()
            .map_err(|_| IFTError::InvalidGmpAccount)?,
        counterparty_address
            .to_string()
            .try_into()
            .map_err(|_| IFTError::InvalidGmpAccount)?,
        Salt::empty(),
        &ics27_gmp::ID,
    );

    require!(gmp.pda == *gmp_account, IFTError::InvalidGmpAccount);
    Ok(())
}

#[cfg(test)]
mod tests {
    use anchor_lang::InstructionData;
    use rstest::rstest;
    use solana_sdk::{
        instruction::{AccountMeta, Instruction},
        program_pack::Pack,
        pubkey::Pubkey,
    };

    use crate::errors::IFTError;
    use crate::state::{ChainOptions, IFTMintMsg};
    use crate::test_utils::*;

    const TEST_CLIENT_ID: &str = "07-tendermint-0";
    const TEST_COUNTERPARTY_ADDRESS: &str = "0x1234567890abcdef1234567890abcdef12345678";

    #[derive(Clone, Copy)]
    enum MintErrorCase {
        ZeroAmount,
        ReceiverMismatch,
        GmpNotSigner,
        BridgeNotActive,
        InvalidBridge,
        InvalidGmpAccount,
        AppPaused,
        MintRateLimitExceeded,
    }

    #[allow(clippy::struct_excessive_bools)]
    struct MintTestConfig {
        amount: u64,
        use_wrong_receiver: bool,
        gmp_is_signer: bool,
        bridge_active: bool,
        use_wrong_bridge_mint: bool,
        use_wrong_gmp_account: bool,
        token_paused: bool,
        rate_limit_exceeded: bool,
        expected_error: u32,
    }

    impl From<MintErrorCase> for MintTestConfig {
        fn from(case: MintErrorCase) -> Self {
            let default = Self {
                amount: 1000,
                use_wrong_receiver: false,
                gmp_is_signer: true,
                bridge_active: true,
                use_wrong_bridge_mint: false,
                use_wrong_gmp_account: false,
                token_paused: false,
                rate_limit_exceeded: false,
                expected_error: 0,
            };

            match case {
                MintErrorCase::ZeroAmount => Self {
                    amount: 0,
                    expected_error: ANCHOR_ERROR_OFFSET + IFTError::ZeroAmount as u32,
                    ..default
                },
                MintErrorCase::ReceiverMismatch => Self {
                    use_wrong_receiver: true,
                    expected_error: ANCHOR_ERROR_OFFSET + IFTError::InvalidReceiver as u32,
                    ..default
                },
                MintErrorCase::GmpNotSigner => Self {
                    gmp_is_signer: false,
                    expected_error: anchor_lang::error::ErrorCode::AccountNotSigner as u32,
                    ..default
                },
                MintErrorCase::BridgeNotActive => Self {
                    bridge_active: false,
                    expected_error: ANCHOR_ERROR_OFFSET + IFTError::BridgeNotActive as u32,
                    ..default
                },
                MintErrorCase::InvalidBridge => Self {
                    use_wrong_bridge_mint: true,
                    expected_error: ANCHOR_ERROR_OFFSET + IFTError::InvalidBridge as u32,
                    ..default
                },
                MintErrorCase::InvalidGmpAccount => Self {
                    use_wrong_gmp_account: true,
                    expected_error: ANCHOR_ERROR_OFFSET + IFTError::InvalidGmpAccount as u32,
                    ..default
                },
                MintErrorCase::AppPaused => Self {
                    token_paused: true,
                    expected_error: ANCHOR_ERROR_OFFSET + IFTError::AppPaused as u32,
                    ..default
                },
                MintErrorCase::MintRateLimitExceeded => Self {
                    rate_limit_exceeded: true,
                    expected_error: ANCHOR_ERROR_OFFSET + IFTError::MintRateLimitExceeded as u32,
                    ..default
                },
            }
        }
    }

    fn run_mint_error_test(case: MintErrorCase) {
        let config = MintTestConfig::from(case);
        let mollusk = setup_mollusk();

        let mint = Pubkey::new_unique();
        let receiver = Pubkey::new_unique();
        let wrong_receiver = Pubkey::new_unique();
        let payer = Pubkey::new_unique();

        let (app_state_pda, app_state_bump) = get_app_state_pda();
        let (app_mint_state_pda, app_mint_state_bump) = get_app_mint_state_pda(&mint);
        let (mint_authority_pda, mint_authority_bump) = get_mint_authority_pda(&mint);
        let (ift_bridge_pda, ift_bridge_bump) = get_bridge_pda(&mint, TEST_CLIENT_ID);
        let (gmp_account_pda, _) = get_gmp_account_pda(TEST_CLIENT_ID, TEST_COUNTERPARTY_ADDRESS);
        let wrong_gmp_account = Pubkey::new_unique();
        let (system_program, system_account) = create_system_program_account();

        let app_state_account = create_ift_app_state_account_with_options(
            app_state_bump,
            Pubkey::new_unique(),
            config.token_paused,
        );
        let app_mint_state_account = if config.rate_limit_exceeded {
            create_ift_app_mint_state_account_full(IftAppMintStateParams {
                mint,
                bump: app_mint_state_bump,
                mint_authority_bump,
                daily_mint_limit: 100,
                rate_limit_day: 0,
                rate_limit_daily_usage: 100,
            })
        } else {
            create_ift_app_mint_state_account(mint, app_mint_state_bump, mint_authority_bump)
        };

        let bridge_mint = if config.use_wrong_bridge_mint {
            Pubkey::new_unique()
        } else {
            mint
        };
        let ift_bridge_account = create_ift_bridge_account(
            bridge_mint,
            TEST_CLIENT_ID,
            TEST_COUNTERPARTY_ADDRESS,
            ChainOptions::Evm,
            ift_bridge_bump,
            config.bridge_active,
        );

        let mint_account = create_mint_account(mint_authority_pda, 6);

        let token_account_owner = if config.use_wrong_receiver {
            wrong_receiver
        } else {
            receiver
        };

        let receiver_owner_key = if config.use_wrong_receiver {
            wrong_receiver
        } else {
            receiver
        };

        let receiver_token_pda =
            anchor_spl::associated_token::get_associated_token_address(&receiver_owner_key, &mint);
        let receiver_token_account = create_token_account(mint, token_account_owner, 0);

        let token_program_account = solana_sdk::account::Account {
            lamports: 1,
            data: vec![],
            owner: solana_sdk::native_loader::ID,
            executable: true,
            rent_epoch: 0,
        };

        let associated_token_program_account = solana_sdk::account::Account {
            lamports: 1,
            data: vec![],
            owner: solana_sdk::native_loader::ID,
            executable: true,
            rent_epoch: 0,
        };

        let gmp_account_key = if config.use_wrong_gmp_account {
            wrong_gmp_account
        } else {
            gmp_account_pda
        };

        let msg = IFTMintMsg {
            receiver,
            amount: config.amount,
        };

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new_readonly(app_state_pda, false),
                AccountMeta::new(app_mint_state_pda, false),
                AccountMeta::new_readonly(ift_bridge_pda, false),
                AccountMeta::new(mint, false),
                AccountMeta::new_readonly(mint_authority_pda, false),
                AccountMeta::new(receiver_token_pda, false),
                AccountMeta::new_readonly(receiver_owner_key, false),
                AccountMeta::new_readonly(gmp_account_key, config.gmp_is_signer),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(anchor_spl::token::ID, false),
                AccountMeta::new_readonly(anchor_spl::associated_token::ID, false),
                AccountMeta::new_readonly(system_program, false),
            ],
            data: crate::instruction::IftMint { msg }.data(),
        };

        let accounts = vec![
            (app_state_pda, app_state_account),
            (app_mint_state_pda, app_mint_state_account),
            (ift_bridge_pda, ift_bridge_account),
            (mint, mint_account),
            (mint_authority_pda, create_signer_account()),
            (receiver_token_pda, receiver_token_account),
            (receiver_owner_key, create_signer_account()),
            (gmp_account_key, create_signer_account()),
            (payer, create_signer_account()),
            (anchor_spl::token::ID, token_program_account),
            (
                anchor_spl::associated_token::ID,
                associated_token_program_account,
            ),
            (system_program, system_account),
        ];

        let result = mollusk.process_instruction(&instruction, &accounts);
        assert_eq!(
            result.program_result,
            Err(solana_sdk::instruction::InstructionError::Custom(
                config.expected_error,
            ))
            .into(),
        );
    }

    #[rstest]
    #[case::zero_amount(MintErrorCase::ZeroAmount)]
    #[case::receiver_mismatch(MintErrorCase::ReceiverMismatch)]
    #[case::gmp_not_signer(MintErrorCase::GmpNotSigner)]
    #[case::bridge_not_active(MintErrorCase::BridgeNotActive)]
    #[case::invalid_bridge(MintErrorCase::InvalidBridge)]
    #[case::invalid_gmp_account(MintErrorCase::InvalidGmpAccount)]
    #[case::app_paused(MintErrorCase::AppPaused)]
    #[case::mint_rate_limit_exceeded(MintErrorCase::MintRateLimitExceeded)]
    fn test_ift_mint_validation(#[case] case: MintErrorCase) {
        run_mint_error_test(case);
    }

    #[test]
    fn test_ift_mint_success() {
        let mollusk = setup_mollusk_with_token();

        let mint = Pubkey::new_unique();
        let receiver = Pubkey::new_unique();
        let payer = Pubkey::new_unique();

        let (app_state_pda, app_state_bump) = get_app_state_pda();
        let (app_mint_state_pda, app_mint_state_bump) = get_app_mint_state_pda(&mint);
        let (mint_authority_pda, mint_authority_bump) = get_mint_authority_pda(&mint);
        let (ift_bridge_pda, ift_bridge_bump) = get_bridge_pda(&mint, TEST_CLIENT_ID);
        let (gmp_account_pda, _) = get_gmp_account_pda(TEST_CLIENT_ID, TEST_COUNTERPARTY_ADDRESS);
        let (system_program, system_account) = create_system_program_account();
        let (token_program_id, token_program_account) = token_program_keyed_account();

        let app_state_account = create_ift_app_state_account(app_state_bump, Pubkey::new_unique());
        let app_mint_state_account =
            create_ift_app_mint_state_account(mint, app_mint_state_bump, mint_authority_bump);
        let ift_bridge_account = create_ift_bridge_account(
            mint,
            TEST_CLIENT_ID,
            TEST_COUNTERPARTY_ADDRESS,
            ChainOptions::Evm,
            ift_bridge_bump,
            true,
        );
        let mint_account = create_mint_account(mint_authority_pda, 6);

        let receiver_token_pda =
            anchor_spl::associated_token::get_associated_token_address(&receiver, &mint);
        let receiver_token_account = create_token_account(mint, receiver, 0);

        let associated_token_program_account = solana_sdk::account::Account {
            lamports: 1,
            data: vec![],
            owner: solana_sdk::native_loader::ID,
            executable: true,
            rent_epoch: 0,
        };

        let msg = IFTMintMsg {
            receiver,
            amount: 1000,
        };

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new_readonly(app_state_pda, false),
                AccountMeta::new(app_mint_state_pda, false),
                AccountMeta::new_readonly(ift_bridge_pda, false),
                AccountMeta::new(mint, false),
                AccountMeta::new_readonly(mint_authority_pda, false),
                AccountMeta::new(receiver_token_pda, false),
                AccountMeta::new_readonly(receiver, false),
                AccountMeta::new_readonly(gmp_account_pda, true),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(token_program_id, false),
                AccountMeta::new_readonly(anchor_spl::associated_token::ID, false),
                AccountMeta::new_readonly(system_program, false),
            ],
            data: crate::instruction::IftMint { msg }.data(),
        };

        let accounts = vec![
            (app_state_pda, app_state_account),
            (app_mint_state_pda, app_mint_state_account),
            (ift_bridge_pda, ift_bridge_account),
            (mint, mint_account),
            (mint_authority_pda, create_uninitialized_pda()),
            (receiver_token_pda, receiver_token_account),
            (receiver, create_signer_account()),
            (gmp_account_pda, create_signer_account()),
            (payer, create_signer_account()),
            (token_program_id, token_program_account),
            (
                anchor_spl::associated_token::ID,
                associated_token_program_account,
            ),
            (system_program, system_account),
        ];

        let result = mollusk.process_instruction(&instruction, &accounts);
        assert!(
            !result.program_result.is_err(),
            "ift_mint should succeed: {:?}",
            result.program_result
        );

        let (_, receiver_acc) = &result.resulting_accounts[5];
        let token = anchor_spl::token::spl_token::state::Account::unpack(&receiver_acc.data)
            .expect("valid token account");
        assert_eq!(token.amount, 1000);
    }

    /// Verifies that passing a valid bridge at the wrong PDA address is rejected.
    /// The bridge has matching mint and is active, but its `client_id` doesn't match
    /// the PDA derivation address — the seeds constraint catches this.
    #[test]
    fn test_ift_mint_bridge_pda_mismatch_rejected() {
        let mollusk = setup_mollusk();

        let mint = Pubkey::new_unique();
        let receiver = Pubkey::new_unique();
        let payer = Pubkey::new_unique();

        let (app_state_pda, app_state_bump) = get_app_state_pda();
        let (app_mint_state_pda, app_mint_state_bump) = get_app_mint_state_pda(&mint);
        let (mint_authority_pda, mint_authority_bump) = get_mint_authority_pda(&mint);
        let (system_program, system_account) = create_system_program_account();

        // Derive PDA for TEST_CLIENT_ID but create bridge data with a different client_id.
        // The seeds constraint will recompute PDA from the deserialized client_id and reject.
        let (bridge_pda_wrong_addr, _) = get_bridge_pda(&mint, TEST_CLIENT_ID);
        let different_client_id = "07-tendermint-999";
        let (_, bridge_bump) = get_bridge_pda(&mint, different_client_id);

        let bridge_account = create_ift_bridge_account(
            mint,
            different_client_id,
            TEST_COUNTERPARTY_ADDRESS,
            ChainOptions::Evm,
            bridge_bump,
            true,
        );

        let (gmp_account_pda, _) =
            get_gmp_account_pda(different_client_id, TEST_COUNTERPARTY_ADDRESS);

        let app_state_account = create_ift_app_state_account(app_state_bump, Pubkey::new_unique());
        let app_mint_state_account =
            create_ift_app_mint_state_account(mint, app_mint_state_bump, mint_authority_bump);
        let mint_account = create_mint_account(mint_authority_pda, 6);

        let receiver_token_pda =
            anchor_spl::associated_token::get_associated_token_address(&receiver, &mint);
        let receiver_token_account = create_token_account(mint, receiver, 0);

        let token_program_account = solana_sdk::account::Account {
            lamports: 1,
            data: vec![],
            owner: solana_sdk::native_loader::ID,
            executable: true,
            rent_epoch: 0,
        };
        let associated_token_program_account = solana_sdk::account::Account {
            lamports: 1,
            data: vec![],
            owner: solana_sdk::native_loader::ID,
            executable: true,
            rent_epoch: 0,
        };

        let msg = IFTMintMsg {
            receiver,
            amount: 1000,
        };

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new_readonly(app_state_pda, false),
                AccountMeta::new(app_mint_state_pda, false),
                AccountMeta::new_readonly(bridge_pda_wrong_addr, false), // wrong PDA for this client_id
                AccountMeta::new(mint, false),
                AccountMeta::new_readonly(mint_authority_pda, false),
                AccountMeta::new(receiver_token_pda, false),
                AccountMeta::new_readonly(receiver, false),
                AccountMeta::new_readonly(gmp_account_pda, true),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(anchor_spl::token::ID, false),
                AccountMeta::new_readonly(anchor_spl::associated_token::ID, false),
                AccountMeta::new_readonly(system_program, false),
            ],
            data: crate::instruction::IftMint { msg }.data(),
        };

        let accounts = vec![
            (app_state_pda, app_state_account),
            (app_mint_state_pda, app_mint_state_account),
            (bridge_pda_wrong_addr, bridge_account),
            (mint, mint_account),
            (mint_authority_pda, create_signer_account()),
            (receiver_token_pda, receiver_token_account),
            (receiver, create_signer_account()),
            (gmp_account_pda, create_signer_account()),
            (payer, create_signer_account()),
            (anchor_spl::token::ID, token_program_account),
            (
                anchor_spl::associated_token::ID,
                associated_token_program_account,
            ),
            (system_program, system_account),
        ];

        let result = mollusk.process_instruction(&instruction, &accounts);
        assert_eq!(
            result.program_result,
            Err(solana_sdk::instruction::InstructionError::Custom(
                anchor_lang::error::ErrorCode::ConstraintSeeds as u32,
            ))
            .into(),
        );
    }
}
