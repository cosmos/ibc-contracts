// SPDX-License-Identifier: Apache-2.0

use anchor_lang::prelude::*;
use anchor_spl::token_interface::spl_token_2022::instruction::AuthorityType;
use anchor_spl::token_interface::{set_authority, Mint, SetAuthority, TokenInterface};

use crate::constants::*;
use crate::errors::IFTError;
use crate::events::ExistingTokenInitialized;
use crate::state::{AccountVersion, IFTAppMintState, IFTAppState};

#[derive(Accounts)]
pub struct InitializeExistingToken<'info> {
    /// Global IFT app state (must exist)
    #[account(
        seeds = [IFT_APP_STATE_SEED],
        bump = app_state.bump
    )]
    pub app_state: Account<'info, IFTAppState>,

    /// Per-mint IFT app state PDA (to be created)
    #[account(
        init,
        payer = payer,
        space = 8 + IFTAppMintState::INIT_SPACE,
        seeds = [IFT_APP_MINT_STATE_SEED, mint.key().as_ref()],
        bump
    )]
    pub app_mint_state: Account<'info, IFTAppMintState>,

    /// Existing SPL Token mint whose authority will be transferred to the IFT PDA
    #[account(
        mut,
        constraint = mint.mint_authority.is_some() @ IFTError::MintAuthorityNotSet
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    /// CHECK: PDA that will become the new mint authority
    #[account(
        seeds = [MINT_AUTHORITY_SEED, mint.key().as_ref()],
        bump
    )]
    pub mint_authority: AccountInfo<'info>,

    /// Current mint authority that must sign to transfer ownership to the IFT PDA
    #[account(
        constraint = mint.mint_authority.unwrap() == current_authority.key() @ IFTError::InvalidMintAuthority
    )]
    pub current_authority: Signer<'info>,

    /// Pays for account creation and transaction fees
    #[account(mut)]
    pub payer: Signer<'info>,

    /// SPL Token or Token 2022 program for the `set_authority` CPI
    pub token_program: Interface<'info, TokenInterface>,
    /// Required for PDA account creation
    pub system_program: Program<'info, System>,
}

pub fn initialize_existing_token(ctx: Context<InitializeExistingToken>) -> Result<()> {
    let cpi_accounts = SetAuthority {
        account_or_mint: ctx.accounts.mint.to_account_info(),
        current_authority: ctx.accounts.current_authority.to_account_info(),
    };
    let cpi_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), cpi_accounts);
    set_authority(
        cpi_ctx,
        AuthorityType::MintTokens,
        Some(ctx.accounts.mint_authority.key()),
    )?;

    let app_mint_state = &mut ctx.accounts.app_mint_state;
    app_mint_state.version = AccountVersion::V1;
    app_mint_state.bump = ctx.bumps.app_mint_state;
    app_mint_state.mint = ctx.accounts.mint.key();
    app_mint_state.mint_authority_bump = ctx.bumps.mint_authority;

    let clock = Clock::get()?;
    emit!(ExistingTokenInitialized {
        mint: ctx.accounts.mint.key(),
        decimals: ctx.accounts.mint.decimals,
        previous_authority: ctx.accounts.current_authority.key(),
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use anchor_lang::{InstructionData, Space};
    use solana_sdk::{
        instruction::{AccountMeta, Instruction},
        program_pack::Pack,
        pubkey::Pubkey,
        rent::Rent,
    };

    use crate::errors::IFTError;
    use crate::state::IFTAppMintState;
    use crate::test_utils::*;

    fn create_mint_account_no_authority(decimals: u8) -> solana_sdk::account::Account {
        use anchor_spl::token::spl_token;
        use solana_sdk::program_pack::Pack;

        let mint = spl_token::state::Mint {
            mint_authority: solana_sdk::program_option::COption::None,
            supply: 1_000_000_000,
            decimals,
            is_initialized: true,
            freeze_authority: solana_sdk::program_option::COption::None,
        };

        let mut data = vec![0u8; spl_token::state::Mint::LEN];
        mint.pack_into_slice(&mut data);

        solana_sdk::account::Account {
            lamports: 1_000_000,
            data,
            owner: spl_token::ID,
            executable: false,
            rent_epoch: 0,
        }
    }

    struct TestContext {
        mollusk: mollusk_svm::Mollusk,
        mint: Pubkey,
        current_authority: Pubkey,
        payer: Pubkey,
        app_state_pda: Pubkey,
        app_state_bump: u8,
        app_mint_state_pda: Pubkey,
        mint_authority_pda: Pubkey,
        admin: Pubkey,
    }

    fn setup_test() -> TestContext {
        let mollusk = setup_mollusk();
        let mint = Pubkey::new_unique();
        let current_authority = Pubkey::new_unique();
        let payer = Pubkey::new_unique();
        let (app_state_pda, app_state_bump) = get_app_state_pda();
        let (app_mint_state_pda, _) = get_app_mint_state_pda(&mint);
        let (mint_authority_pda, _) = get_mint_authority_pda(&mint);
        let admin = Pubkey::new_unique();

        TestContext {
            mollusk,
            mint,
            current_authority,
            payer,
            app_state_pda,
            app_state_bump,
            app_mint_state_pda,
            mint_authority_pda,
            admin,
        }
    }

    fn build_instruction(ctx: &TestContext) -> Instruction {
        Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new_readonly(ctx.app_state_pda, false),
                AccountMeta::new(ctx.app_mint_state_pda, false),
                AccountMeta::new(ctx.mint, false),
                AccountMeta::new_readonly(ctx.mint_authority_pda, false),
                AccountMeta::new_readonly(ctx.current_authority, true),
                AccountMeta::new(ctx.payer, true),
                AccountMeta::new_readonly(anchor_spl::token::ID, false),
                AccountMeta::new_readonly(solana_sdk::system_program::ID, false),
            ],
            data: crate::instruction::InitializeExistingToken {}.data(),
        }
    }

    #[test]
    fn test_mint_without_authority_fails() {
        let ctx = setup_test();
        let instruction = build_instruction(&ctx);

        let (token_program, token_program_account) = create_token_program_account();
        let (system_program, system_program_account) = create_system_program_account();
        let (clock_sysvar, clock_account) = create_clock_sysvar_account(1_700_000_000);

        let accounts = vec![
            (
                ctx.app_state_pda,
                create_ift_app_state_account(ctx.app_state_bump, ctx.admin),
            ),
            (ctx.app_mint_state_pda, create_uninitialized_pda()),
            (ctx.mint, create_mint_account_no_authority(6)),
            (
                ctx.mint_authority_pda,
                solana_sdk::account::Account {
                    lamports: 0,
                    data: vec![],
                    owner: solana_sdk::system_program::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
            (ctx.current_authority, create_signer_account()),
            (ctx.payer, create_signer_account()),
            (token_program, token_program_account),
            (system_program, system_program_account),
            (clock_sysvar, clock_account),
        ];

        let result = ctx.mollusk.process_instruction(&instruction, &accounts);
        assert_eq!(
            result.program_result,
            Err(solana_sdk::instruction::InstructionError::Custom(
                ANCHOR_ERROR_OFFSET + IFTError::MintAuthorityNotSet as u32,
            ))
            .into(),
        );
    }

    #[test]
    fn test_wrong_authority_signer_fails() {
        let ctx = setup_test();
        let instruction = build_instruction(&ctx);

        let actual_authority = Pubkey::new_unique();
        let (token_program, token_program_account) = create_token_program_account();
        let (system_program, system_program_account) = create_system_program_account();
        let (clock_sysvar, clock_account) = create_clock_sysvar_account(1_700_000_000);

        let accounts = vec![
            (
                ctx.app_state_pda,
                create_ift_app_state_account(ctx.app_state_bump, ctx.admin),
            ),
            (ctx.app_mint_state_pda, create_uninitialized_pda()),
            (ctx.mint, create_mint_account(actual_authority, 6)), // Different authority
            (
                ctx.mint_authority_pda,
                solana_sdk::account::Account {
                    lamports: 0,
                    data: vec![],
                    owner: solana_sdk::system_program::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
            (ctx.current_authority, create_signer_account()), // Signs but doesn't match mint authority
            (ctx.payer, create_signer_account()),
            (token_program, token_program_account),
            (system_program, system_program_account),
            (clock_sysvar, clock_account),
        ];

        let result = ctx.mollusk.process_instruction(&instruction, &accounts);
        assert_eq!(
            result.program_result,
            Err(solana_sdk::instruction::InstructionError::Custom(
                ANCHOR_ERROR_OFFSET + IFTError::InvalidMintAuthority as u32,
            ))
            .into(),
        );
    }

    #[test]
    fn test_app_mint_state_already_exists_fails() {
        let ctx = setup_test();
        let instruction = build_instruction(&ctx);

        let (token_program, token_program_account) = create_token_program_account();
        let (system_program, system_program_account) = create_system_program_account();
        let (clock_sysvar, clock_account) = create_clock_sysvar_account(1_700_000_000);
        let (_, app_mint_state_bump) = get_app_mint_state_pda(&ctx.mint);
        let (_, mint_authority_bump) = get_mint_authority_pda(&ctx.mint);

        let accounts = vec![
            (
                ctx.app_state_pda,
                create_ift_app_state_account(ctx.app_state_bump, ctx.admin),
            ),
            (
                ctx.app_mint_state_pda,
                create_ift_app_mint_state_account(
                    ctx.mint,
                    app_mint_state_bump,
                    mint_authority_bump,
                ),
            ),
            (ctx.mint, create_mint_account(ctx.current_authority, 6)),
            (
                ctx.mint_authority_pda,
                solana_sdk::account::Account {
                    lamports: 0,
                    data: vec![],
                    owner: solana_sdk::system_program::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
            (ctx.current_authority, create_signer_account()),
            (ctx.payer, create_signer_account()),
            (token_program, token_program_account),
            (system_program, system_program_account),
            (clock_sysvar, clock_account),
        ];

        let result = ctx.mollusk.process_instruction(&instruction, &accounts);
        // System program returns AccountAlreadyInUse (0) when init tries to
        // create_account for an account that already has data/lamports
        assert_eq!(
            result.program_result,
            Err(solana_sdk::instruction::InstructionError::Custom(0)).into(),
        );
    }

    #[test]
    fn test_initialize_existing_token_success() {
        let mollusk = setup_mollusk_with_token();

        let mint = Pubkey::new_unique();
        let current_authority = Pubkey::new_unique();
        let payer = Pubkey::new_unique();
        let admin = Pubkey::new_unique();

        let (app_state_pda, app_state_bump) = get_app_state_pda();
        let (app_mint_state_pda, _) = get_app_mint_state_pda(&mint);
        let (mint_authority_pda, _) = get_mint_authority_pda(&mint);
        let (system_program, system_program_account) = create_system_program_account();
        let (token_program, token_program_account) = token_program_keyed_account();

        let app_mint_state_account = solana_sdk::account::Account {
            lamports: Rent::default().minimum_balance(8 + IFTAppMintState::INIT_SPACE),
            data: vec![],
            owner: solana_sdk::system_program::ID,
            executable: false,
            rent_epoch: 0,
        };

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new_readonly(app_state_pda, false),
                AccountMeta::new(app_mint_state_pda, false),
                AccountMeta::new(mint, false),
                AccountMeta::new_readonly(mint_authority_pda, false),
                AccountMeta::new_readonly(current_authority, true),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(token_program, false),
                AccountMeta::new_readonly(system_program, false),
            ],
            data: crate::instruction::InitializeExistingToken {}.data(),
        };

        let accounts = vec![
            (
                app_state_pda,
                create_ift_app_state_account(app_state_bump, admin),
            ),
            (app_mint_state_pda, app_mint_state_account),
            (mint, create_mint_account(current_authority, 6)),
            (mint_authority_pda, create_uninitialized_pda()),
            (current_authority, create_signer_account()),
            (payer, create_signer_account()),
            (token_program, token_program_account),
            (system_program, system_program_account),
        ];

        let result = mollusk.process_instruction(&instruction, &accounts);
        assert!(
            !result.program_result.is_err(),
            "initialize_existing_token should succeed: {:?}",
            result.program_result
        );

        let (_, mint_acc) = &result.resulting_accounts[2];
        let updated_mint =
            anchor_spl::token::spl_token::state::Mint::unpack(&mint_acc.data).expect("valid mint");
        assert_eq!(
            updated_mint.mint_authority,
            solana_sdk::program_option::COption::Some(mint_authority_pda),
        );

        let (_, mint_state_acc) = &result.resulting_accounts[1];
        let mint_state = deserialize_app_mint_state(mint_state_acc);
        assert_eq!(mint_state.mint, mint);
    }
}
