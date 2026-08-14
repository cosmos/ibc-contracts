// SPDX-License-Identifier: Apache-2.0

use anchor_lang::prelude::*;
use anchor_lang::solana_program::{program::invoke, system_instruction};
use anchor_spl::token_2022_extensions::{
    metadata_pointer_initialize, token_metadata_initialize, MetadataPointerInitialize,
    TokenMetadataInitialize,
};
use anchor_spl::token_interface::spl_token_2022::extension::ExtensionType;
use anchor_spl::token_interface::TokenInterface;
use solana_ibc_types::reject_cpi;

use crate::constants::*;
use crate::errors::IFTError;
use crate::events::SplTokenCreated;
use crate::state::{AccountVersion, CreateTokenParams, IFTAppMintState, IFTAppState};

#[derive(Accounts)]
#[instruction(params: CreateTokenParams)]
pub struct CreateAndInitializeSplToken<'info> {
    /// Global IFT app state
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

    /// SPL Token mint keypair. Must sign so `create_account` can allocate it.
    /// Initialized manually to support Token 2022 extensions.
    #[account(mut)]
    pub mint: Signer<'info>,

    /// Mint authority PDA
    /// CHECK: Derived PDA set as mint authority
    #[account(
        seeds = [MINT_AUTHORITY_SEED, mint.key().as_ref()],
        bump
    )]
    pub mint_authority: AccountInfo<'info>,

    /// Admin authority
    #[account(
        constraint = admin.key() == app_state.admin @ IFTError::UnauthorizedAdmin
    )]
    pub admin: Signer<'info>,

    /// Pays for account creation and transaction fees
    #[account(mut)]
    pub payer: Signer<'info>,

    /// SPL Token or Token 2022 program, must match the `CreateTokenParams` variant
    pub token_program: Interface<'info, TokenInterface>,
    /// Required for mint and PDA account creation
    pub system_program: Program<'info, System>,

    /// CHECK: Address constraint verifies this is the instructions sysvar
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instructions_sysvar: AccountInfo<'info>,
}

const TOKEN_2022_PROGRAM_ID: Pubkey = anchor_spl::token_interface::spl_token_2022::ID;
const SPL_TOKEN_PROGRAM_ID: Pubkey = anchor_spl::token::spl_token::ID;

pub fn create_and_initialize_spl_token(
    ctx: Context<CreateAndInitializeSplToken>,
    params: CreateTokenParams,
) -> Result<()> {
    reject_cpi(&ctx.accounts.instructions_sysvar, &crate::ID).map_err(IFTError::from)?;

    let token_program_id = ctx.accounts.token_program.key();
    let mint_key = ctx.accounts.mint.key();
    let mint_authority_key = ctx.accounts.mint_authority.key();

    match &params {
        CreateTokenParams::SplToken { decimals } => {
            require_keys_eq!(
                token_program_id,
                SPL_TOKEN_PROGRAM_ID,
                IFTError::TokenProgramMismatch
            );
            create_legacy_mint(&ctx, *decimals, &mint_key, &mint_authority_key)?;
        }
        CreateTokenParams::Token2022 {
            decimals,
            name,
            symbol,
            uri,
        } => {
            require_keys_eq!(
                token_program_id,
                TOKEN_2022_PROGRAM_ID,
                IFTError::TokenProgramMismatch
            );
            create_token_2022_mint(
                &ctx,
                *decimals,
                &mint_key,
                &mint_authority_key,
                name,
                symbol,
                uri,
            )?;
        }
    }

    let app_mint_state = &mut ctx.accounts.app_mint_state;
    app_mint_state.version = AccountVersion::V1;
    app_mint_state.bump = ctx.bumps.app_mint_state;
    app_mint_state.mint = mint_key;
    app_mint_state.mint_authority_bump = ctx.bumps.mint_authority;

    let clock = Clock::get()?;
    emit!(SplTokenCreated {
        mint: mint_key,
        params,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

/// Defensive account creation that tolerates pre-funded accounts.
///
/// Unlike `system_program::create_account`, which fails with `AccountAlreadyInUse`
/// when the target already holds lamports, this function uses transfer + allocate +
/// assign to safely initialize the account regardless of existing balance.
fn create_account_defensively<'info>(
    payer: &AccountInfo<'info>,
    target: &AccountInfo<'info>,
    system_program: &AccountInfo<'info>,
    required_lamports: u64,
    space: u64,
    owner: &Pubkey,
) -> Result<()> {
    let transfer_amount = required_lamports.saturating_sub(target.lamports());

    if transfer_amount > 0 {
        invoke(
            &system_instruction::transfer(payer.key, target.key, transfer_amount),
            &[payer.clone(), target.clone(), system_program.clone()],
        )?;
    }

    invoke(
        &system_instruction::allocate(target.key, space),
        &[target.clone(), system_program.clone()],
    )?;

    invoke(
        &system_instruction::assign(target.key, owner),
        &[target.clone(), system_program.clone()],
    )?;

    Ok(())
}

/// Create a Token 2022 mint with `MetadataPointer` extension and on-chain metadata.
fn create_token_2022_mint(
    ctx: &Context<CreateAndInitializeSplToken>,
    decimals: u8,
    mint_key: &Pubkey,
    mint_authority_key: &Pubkey,
    name: &str,
    symbol: &str,
    uri: &str,
) -> Result<()> {
    use anchor_spl::token_2022_extensions::spl_token_metadata_interface::state::TokenMetadata;
    use anchor_spl::token_interface::spl_token_2022::state::Mint as Token2022Mint;

    let metadata = TokenMetadata {
        update_authority: anchor_spl::token_2022_extensions::spl_pod::optional_keys::OptionalNonZeroPubkey::try_from(
            Some(*mint_authority_key),
        )
        .unwrap(),
        mint: *mint_key,
        name: name.to_string(),
        symbol: symbol.to_string(),
        uri: uri.to_string(),
        additional_metadata: vec![],
    };

    // Allocate space for base mint + fixed extensions only. Token 2022's
    // `InitializeMint2` validates that account size matches exactly.
    // The variable-length metadata is written later by `token_metadata_initialize`,
    // which reallocates the account internally.
    let extension_space = ExtensionType::try_calculate_account_len::<Token2022Mint>(&[
        ExtensionType::MetadataPointer,
    ])?;

    // Pre-fund with enough lamports for the final size (including metadata)
    // so Token 2022 can grow the account without additional transfers.
    let metadata_space = metadata.tlv_size_of()?;
    let total_space = extension_space.saturating_add(metadata_space);

    let rent = Rent::get()?;
    let lamports = rent.minimum_balance(total_space);

    create_account_defensively(
        &ctx.accounts.payer.to_account_info(),
        &ctx.accounts.mint.to_account_info(),
        &ctx.accounts.system_program.to_account_info(),
        lamports,
        extension_space as u64,
        &TOKEN_2022_PROGRAM_ID,
    )?;

    // Initialize MetadataPointer (must be done before InitializeMint2)
    metadata_pointer_initialize(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            MetadataPointerInitialize {
                token_program_id: ctx.accounts.token_program.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
            },
        ),
        Some(*mint_authority_key),
        Some(*mint_key),
    )?;

    // Initialize the mint
    let ix = anchor_spl::token_interface::spl_token_2022::instruction::initialize_mint2(
        &TOKEN_2022_PROGRAM_ID,
        mint_key,
        mint_authority_key,
        None,
        decimals,
    )?;
    anchor_lang::solana_program::program::invoke(
        &ix,
        &[
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.mint.to_account_info(),
        ],
    )?;

    // Initialize token metadata (must be done after InitializeMint2)
    let mint_authority_bump = ctx.bumps.mint_authority;
    let seeds = &[
        MINT_AUTHORITY_SEED,
        mint_key.as_ref(),
        &[mint_authority_bump],
    ];
    let signer_seeds = &[&seeds[..]];

    token_metadata_initialize(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            TokenMetadataInitialize {
                program_id: ctx.accounts.token_program.to_account_info(),
                metadata: ctx.accounts.mint.to_account_info(),
                update_authority: ctx.accounts.mint_authority.to_account_info(),
                mint_authority: ctx.accounts.mint_authority.to_account_info(),
                mint: ctx.accounts.mint.to_account_info(),
            },
            signer_seeds,
        ),
        name.to_string(),
        symbol.to_string(),
        uri.to_string(),
    )?;

    Ok(())
}

/// Create a legacy SPL Token mint (no extensions).
fn create_legacy_mint(
    ctx: &Context<CreateAndInitializeSplToken>,
    decimals: u8,
    mint_key: &Pubkey,
    mint_authority_key: &Pubkey,
) -> Result<()> {
    use anchor_lang::solana_program::program_pack::Pack as _;

    let token_program_id = ctx.accounts.token_program.key();
    let space = anchor_spl::token::spl_token::state::Mint::LEN;
    let rent = Rent::get()?;
    let lamports = rent.minimum_balance(space);

    create_account_defensively(
        &ctx.accounts.payer.to_account_info(),
        &ctx.accounts.mint.to_account_info(),
        &ctx.accounts.system_program.to_account_info(),
        lamports,
        space as u64,
        &token_program_id,
    )?;

    let ix = anchor_spl::token_interface::spl_token_2022::instruction::initialize_mint2(
        &token_program_id,
        mint_key,
        mint_authority_key,
        None,
        decimals,
    )?;
    anchor_lang::solana_program::program::invoke(
        &ix,
        &[
            ctx.accounts.token_program.to_account_info(),
            ctx.accounts.mint.to_account_info(),
        ],
    )?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use anchor_lang::{InstructionData, Space};
    use anchor_spl::token_2022_extensions::spl_token_metadata_interface::state::TokenMetadata;
    use anchor_spl::token_interface::spl_token_2022::{
        extension::{
            metadata_pointer::MetadataPointer, BaseStateWithExtensions as _, ExtensionType,
            StateWithExtensions,
        },
        state::Mint as Token2022Mint,
    };
    use solana_sdk::{
        instruction::{AccountMeta, Instruction},
        program_pack::Pack,
        pubkey::Pubkey,
        rent::Rent,
    };

    use crate::errors::IFTError;
    use crate::state::{CreateTokenParams, IFTAppMintState};
    use crate::test_utils::*;

    fn create_empty_mint_account() -> solana_sdk::account::Account {
        solana_sdk::account::Account {
            lamports: 0,
            data: vec![],
            owner: solana_sdk::system_program::ID,
            executable: false,
            rent_epoch: 0,
        }
    }

    fn spl_token_instruction_data(decimals: u8) -> Vec<u8> {
        crate::instruction::CreateAndInitializeSplToken {
            params: CreateTokenParams::SplToken { decimals },
        }
        .data()
    }

    fn token_2022_instruction_data(decimals: u8, name: &str, symbol: &str, uri: &str) -> Vec<u8> {
        crate::instruction::CreateAndInitializeSplToken {
            params: CreateTokenParams::Token2022 {
                decimals,
                name: name.to_string(),
                symbol: symbol.to_string(),
                uri: uri.to_string(),
            },
        }
        .data()
    }

    struct CreateTokenTestSetup {
        instruction: Instruction,
        accounts: Vec<(Pubkey, solana_sdk::account::Account)>,
    }

    fn build_spl_token_setup(
        token_program_id: Pubkey,
        token_program_account: solana_sdk::account::Account,
        decimals: u8,
    ) -> CreateTokenTestSetup {
        build_create_token_setup(
            token_program_id,
            token_program_account,
            spl_token_instruction_data(decimals),
        )
    }

    fn build_token_2022_setup(
        token_program_id: Pubkey,
        token_program_account: solana_sdk::account::Account,
        decimals: u8,
        name: &str,
        symbol: &str,
        uri: &str,
    ) -> CreateTokenTestSetup {
        build_create_token_setup(
            token_program_id,
            token_program_account,
            token_2022_instruction_data(decimals, name, symbol, uri),
        )
    }

    fn build_create_token_setup(
        token_program_id: Pubkey,
        token_program_account: solana_sdk::account::Account,
        instruction_data: Vec<u8>,
    ) -> CreateTokenTestSetup {
        let mint = Pubkey::new_unique();
        let payer = Pubkey::new_unique();
        let admin = Pubkey::new_unique();

        let (app_state_pda, app_state_bump) = get_app_state_pda();
        let (app_mint_state_pda, _) = get_app_mint_state_pda(&mint);
        let (mint_authority_pda, _) = get_mint_authority_pda(&mint);
        let (system_program, system_account) = create_system_program_account();
        let (sysvar_id, sysvar_account) = create_instructions_sysvar_account();

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
                AccountMeta::new(mint, true),
                AccountMeta::new_readonly(mint_authority_pda, false),
                AccountMeta::new_readonly(admin, true),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(token_program_id, false),
                AccountMeta::new_readonly(system_program, false),
                AccountMeta::new_readonly(sysvar_id, false),
            ],
            data: instruction_data,
        };

        let accounts = vec![
            (
                app_state_pda,
                create_ift_app_state_account(app_state_bump, admin),
            ),
            (app_mint_state_pda, app_mint_state_account),
            (mint, create_empty_mint_account()),
            (mint_authority_pda, create_uninitialized_pda()),
            (admin, create_signer_account()),
            (payer, create_signer_account()),
            (token_program_id, token_program_account),
            (system_program, system_account),
            (sysvar_id, sysvar_account),
        ];

        CreateTokenTestSetup {
            instruction,
            accounts,
        }
    }

    #[test]
    fn test_create_and_initialize_spl_token_wrong_pda_fails() {
        let mollusk = setup_mollusk();

        let mint = Pubkey::new_unique();
        let wrong_mint = Pubkey::new_unique();
        let payer = Pubkey::new_unique();
        let admin = Pubkey::new_unique();

        let (app_state_pda, app_state_bump) = get_app_state_pda();
        // Use wrong mint for per-mint PDA derivation
        let (wrong_app_mint_state_pda, _) = get_app_mint_state_pda(&wrong_mint);
        let (mint_authority_pda, _) = get_mint_authority_pda(&mint);
        let (system_program, system_account) = create_system_program_account();
        let (sysvar_id, sysvar_account) = create_instructions_sysvar_account();

        let app_mint_state_account = solana_sdk::account::Account {
            lamports: Rent::default().minimum_balance(8 + IFTAppMintState::INIT_SPACE),
            data: vec![],
            owner: solana_sdk::system_program::ID,
            executable: false,
            rent_epoch: 0,
        };

        let mint_authority_account = solana_sdk::account::Account {
            lamports: 0,
            data: vec![],
            owner: solana_sdk::system_program::ID,
            executable: false,
            rent_epoch: 0,
        };

        let token_program_account = solana_sdk::account::Account {
            lamports: 1,
            data: vec![],
            owner: solana_sdk::native_loader::ID,
            executable: true,
            rent_epoch: 0,
        };

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new_readonly(app_state_pda, false),
                AccountMeta::new(wrong_app_mint_state_pda, false),
                AccountMeta::new(mint, true), // mint must sign for init
                AccountMeta::new_readonly(mint_authority_pda, false),
                AccountMeta::new_readonly(admin, true),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(anchor_spl::token::ID, false),
                AccountMeta::new_readonly(system_program, false),
                AccountMeta::new_readonly(sysvar_id, false),
            ],
            data: spl_token_instruction_data(6),
        };

        let accounts = vec![
            (
                app_state_pda,
                create_ift_app_state_account(app_state_bump, admin),
            ),
            (wrong_app_mint_state_pda, app_mint_state_account),
            (mint, create_empty_mint_account()),
            (mint_authority_pda, mint_authority_account),
            (admin, create_signer_account()),
            (payer, create_signer_account()),
            (anchor_spl::token::ID, token_program_account),
            (system_program, system_account),
            (sysvar_id, sysvar_account),
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

    #[test]
    fn test_create_and_initialize_spl_token_success() {
        let mollusk = setup_mollusk_with_token();

        let mint = Pubkey::new_unique();
        let payer = Pubkey::new_unique();
        let admin = Pubkey::new_unique();

        let (app_state_pda, app_state_bump) = get_app_state_pda();
        let (app_mint_state_pda, _) = get_app_mint_state_pda(&mint);
        let (mint_authority_pda, _) = get_mint_authority_pda(&mint);
        let (system_program, system_account) = create_system_program_account();
        let (token_program_id, token_program_account) = token_program_keyed_account();
        let (sysvar_id, sysvar_account) = create_instructions_sysvar_account();

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
                AccountMeta::new(mint, true),
                AccountMeta::new_readonly(mint_authority_pda, false),
                AccountMeta::new_readonly(admin, true),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(token_program_id, false),
                AccountMeta::new_readonly(system_program, false),
                AccountMeta::new_readonly(sysvar_id, false),
            ],
            data: spl_token_instruction_data(6),
        };

        let accounts = vec![
            (
                app_state_pda,
                create_ift_app_state_account(app_state_bump, admin),
            ),
            (app_mint_state_pda, app_mint_state_account),
            (mint, create_empty_mint_account()),
            (mint_authority_pda, create_uninitialized_pda()),
            (admin, create_signer_account()),
            (payer, create_signer_account()),
            (token_program_id, token_program_account),
            (system_program, system_account),
            (sysvar_id, sysvar_account),
        ];

        let result = mollusk.process_instruction(&instruction, &accounts);
        assert!(
            !result.program_result.is_err(),
            "create_and_initialize_spl_token should succeed: {:?}",
            result.program_result
        );

        let (_, mint_acc) = &result.resulting_accounts[2];
        let created_mint =
            anchor_spl::token::spl_token::state::Mint::unpack(&mint_acc.data).expect("valid mint");
        assert_eq!(created_mint.decimals, 6);
        assert_eq!(
            created_mint.mint_authority,
            solana_sdk::program_option::COption::Some(mint_authority_pda),
        );

        let (_, mint_state_acc) = &result.resulting_accounts[1];
        let mint_state = deserialize_app_mint_state(mint_state_acc);
        assert_eq!(mint_state.mint, mint);
    }

    #[test]
    fn test_create_and_initialize_spl_token_zero_decimals_success() {
        let mollusk = setup_mollusk_with_token();

        let mint = Pubkey::new_unique();
        let payer = Pubkey::new_unique();
        let admin = Pubkey::new_unique();

        let (app_state_pda, app_state_bump) = get_app_state_pda();
        let (app_mint_state_pda, _) = get_app_mint_state_pda(&mint);
        let (mint_authority_pda, _) = get_mint_authority_pda(&mint);
        let (system_program, system_account) = create_system_program_account();
        let (token_program_id, token_program_account) = token_program_keyed_account();
        let (sysvar_id, sysvar_account) = create_instructions_sysvar_account();

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
                AccountMeta::new(mint, true),
                AccountMeta::new_readonly(mint_authority_pda, false),
                AccountMeta::new_readonly(admin, true),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(token_program_id, false),
                AccountMeta::new_readonly(system_program, false),
                AccountMeta::new_readonly(sysvar_id, false),
            ],
            data: spl_token_instruction_data(0),
        };

        let accounts = vec![
            (
                app_state_pda,
                create_ift_app_state_account(app_state_bump, admin),
            ),
            (app_mint_state_pda, app_mint_state_account),
            (mint, create_empty_mint_account()),
            (mint_authority_pda, create_uninitialized_pda()),
            (admin, create_signer_account()),
            (payer, create_signer_account()),
            (token_program_id, token_program_account),
            (system_program, system_account),
            (sysvar_id, sysvar_account),
        ];

        let result = mollusk.process_instruction(&instruction, &accounts);
        assert!(
            !result.program_result.is_err(),
            "create_and_initialize_spl_token with 0 decimals should succeed: {:?}",
            result.program_result
        );

        let (_, mint_acc) = &result.resulting_accounts[2];
        let created_mint =
            anchor_spl::token::spl_token::state::Mint::unpack(&mint_acc.data).expect("valid mint");
        assert_eq!(created_mint.decimals, 0);
    }

    #[test]
    fn test_create_and_initialize_spl_token_unauthorized_admin() {
        let mollusk = setup_mollusk_with_token();

        let mint = Pubkey::new_unique();
        let payer = Pubkey::new_unique();
        let admin = Pubkey::new_unique();
        let unauthorized = Pubkey::new_unique();

        let (app_state_pda, app_state_bump) = get_app_state_pda();
        let (app_mint_state_pda, _) = get_app_mint_state_pda(&mint);
        let (mint_authority_pda, _) = get_mint_authority_pda(&mint);
        let (system_program, system_account) = create_system_program_account();
        let (token_program_id, token_program_account) = token_program_keyed_account();
        let (sysvar_id, sysvar_account) = create_instructions_sysvar_account();

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
                AccountMeta::new(mint, true),
                AccountMeta::new_readonly(mint_authority_pda, false),
                AccountMeta::new_readonly(unauthorized, true),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(token_program_id, false),
                AccountMeta::new_readonly(system_program, false),
                AccountMeta::new_readonly(sysvar_id, false),
            ],
            data: spl_token_instruction_data(6),
        };

        let accounts = vec![
            (
                app_state_pda,
                create_ift_app_state_account(app_state_bump, admin),
            ),
            (app_mint_state_pda, app_mint_state_account),
            (mint, create_empty_mint_account()),
            (mint_authority_pda, create_uninitialized_pda()),
            (unauthorized, create_signer_account()),
            (payer, create_signer_account()),
            (token_program_id, token_program_account),
            (system_program, system_account),
            (sysvar_id, sysvar_account),
        ];

        let result = mollusk.process_instruction(&instruction, &accounts);
        assert_eq!(
            result.program_result,
            Err(solana_sdk::instruction::InstructionError::Custom(
                ANCHOR_ERROR_OFFSET + IFTError::UnauthorizedAdmin as u32,
            ))
            .into(),
        );
    }

    #[test]
    fn test_create_and_initialize_spl_token_cpi_rejected() {
        let mollusk = setup_mollusk_with_token();

        let mint = Pubkey::new_unique();
        let payer = Pubkey::new_unique();
        let admin = Pubkey::new_unique();

        let (app_state_pda, app_state_bump) = get_app_state_pda();
        let (app_mint_state_pda, _) = get_app_mint_state_pda(&mint);
        let (mint_authority_pda, _) = get_mint_authority_pda(&mint);
        let (system_program, system_account) = create_system_program_account();
        let (token_program_id, token_program_account) = token_program_keyed_account();
        let (sysvar_id, sysvar_account) =
            create_cpi_instructions_sysvar_account(Pubkey::new_unique());

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
                AccountMeta::new(mint, true),
                AccountMeta::new_readonly(mint_authority_pda, false),
                AccountMeta::new_readonly(admin, true),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(token_program_id, false),
                AccountMeta::new_readonly(system_program, false),
                AccountMeta::new_readonly(sysvar_id, false),
            ],
            data: spl_token_instruction_data(6),
        };

        let accounts = vec![
            (
                app_state_pda,
                create_ift_app_state_account(app_state_bump, admin),
            ),
            (app_mint_state_pda, app_mint_state_account),
            (mint, create_empty_mint_account()),
            (mint_authority_pda, create_uninitialized_pda()),
            (admin, create_signer_account()),
            (payer, create_signer_account()),
            (token_program_id, token_program_account),
            (system_program, system_account),
            (sysvar_id, sysvar_account),
        ];

        let result = mollusk.process_instruction(&instruction, &accounts);
        assert_eq!(
            result.program_result,
            Err(solana_sdk::instruction::InstructionError::Custom(
                ANCHOR_ERROR_OFFSET + IFTError::CpiNotAllowed as u32,
            ))
            .into(),
        );
    }

    // ─── Token 2022 tests ──────────────────────────────────────────

    #[test]
    fn test_token_2022_creates_mint_with_metadata() {
        let mollusk = setup_mollusk_with_token_2022();
        let (token_program_id, token_program_account) = token_2022_keyed_account();

        let setup = build_token_2022_setup(
            token_program_id,
            token_program_account,
            9,
            "Test Token",
            "TST",
            "https://example.com/metadata.json",
        );

        let result = mollusk.process_instruction(&setup.instruction, &setup.accounts);
        assert!(
            !result.program_result.is_err(),
            "Token 2022 create should succeed: {:?}",
            result.program_result
        );

        let (_, mint_acc) = &result.resulting_accounts[2];
        assert_eq!(
            mint_acc.owner,
            anchor_spl::token_interface::spl_token_2022::ID,
            "mint should be owned by Token 2022"
        );

        let state = StateWithExtensions::<Token2022Mint>::unpack(&mint_acc.data)
            .expect("valid Token 2022 mint");
        assert_eq!(state.base.decimals, 9);

        let mp = state
            .get_extension::<MetadataPointer>()
            .expect("MetadataPointer should be present");
        let mint_key = setup.instruction.accounts[2].pubkey;
        assert_eq!(
            Option::<Pubkey>::from(mp.metadata_address),
            Some(mint_key),
            "metadata pointer should point to the mint itself"
        );

        let metadata = state
            .get_variable_len_extension::<TokenMetadata>()
            .expect("TokenMetadata should be present");
        assert_eq!(metadata.name, "Test Token");
        assert_eq!(metadata.symbol, "TST");
        assert_eq!(metadata.uri, "https://example.com/metadata.json");

        // Verify IFTAppMintState
        let (_, mint_state_acc) = &result.resulting_accounts[1];
        let mint_state = deserialize_app_mint_state(mint_state_acc);
        assert_eq!(mint_state.mint, mint_key);
    }

    #[test]
    fn test_token_2022_zero_decimals_success() {
        let mollusk = setup_mollusk_with_token_2022();
        let (token_program_id, token_program_account) = token_2022_keyed_account();

        let setup = build_token_2022_setup(
            token_program_id,
            token_program_account,
            0,
            "Zero Dec",
            "ZD",
            "",
        );

        let result = mollusk.process_instruction(&setup.instruction, &setup.accounts);
        assert!(
            !result.program_result.is_err(),
            "Token 2022 with 0 decimals should succeed: {:?}",
            result.program_result
        );

        let (_, mint_acc) = &result.resulting_accounts[2];
        let state =
            StateWithExtensions::<Token2022Mint>::unpack(&mint_acc.data).expect("valid mint");
        assert_eq!(state.base.decimals, 0);
    }

    #[test]
    fn test_token_2022_empty_metadata_strings() {
        let mollusk = setup_mollusk_with_token_2022();
        let (token_program_id, token_program_account) = token_2022_keyed_account();

        let setup = build_token_2022_setup(token_program_id, token_program_account, 6, "", "", "");

        let result = mollusk.process_instruction(&setup.instruction, &setup.accounts);
        assert!(
            !result.program_result.is_err(),
            "Token 2022 with empty metadata should succeed: {:?}",
            result.program_result
        );

        let (_, mint_acc) = &result.resulting_accounts[2];
        let state =
            StateWithExtensions::<Token2022Mint>::unpack(&mint_acc.data).expect("valid mint");
        assert_eq!(state.base.decimals, 6);

        let metadata = state
            .get_variable_len_extension::<TokenMetadata>()
            .expect("TokenMetadata should exist even with empty strings");
        assert!(metadata.name.is_empty());
        assert!(metadata.symbol.is_empty());
        assert!(metadata.uri.is_empty());
    }

    #[test]
    fn test_token_2022_long_metadata_strings() {
        let mollusk = setup_mollusk_with_token_2022();
        let (token_program_id, token_program_account) = token_2022_keyed_account();

        let long_name = "A".repeat(64);
        let long_symbol = "B".repeat(16);
        let long_uri = "https://example.com/".to_string() + &"x".repeat(128);

        let setup = build_token_2022_setup(
            token_program_id,
            token_program_account,
            6,
            &long_name,
            &long_symbol,
            &long_uri,
        );

        let result = mollusk.process_instruction(&setup.instruction, &setup.accounts);
        assert!(
            !result.program_result.is_err(),
            "Token 2022 with long metadata should succeed: {:?}",
            result.program_result
        );

        let (_, mint_acc) = &result.resulting_accounts[2];
        let state =
            StateWithExtensions::<Token2022Mint>::unpack(&mint_acc.data).expect("valid mint");

        let metadata = state
            .get_variable_len_extension::<TokenMetadata>()
            .expect("TokenMetadata should be present");
        assert_eq!(metadata.name, long_name);
        assert_eq!(metadata.symbol, long_symbol);
        assert_eq!(metadata.uri, long_uri);
    }

    #[test]
    fn test_spl_token_with_token_2022_program_fails() {
        let mollusk = setup_mollusk_with_token_2022();
        let (token_program_id, token_program_account) = token_2022_keyed_account();

        let setup = build_spl_token_setup(token_program_id, token_program_account, 6);

        let result = mollusk.process_instruction(&setup.instruction, &setup.accounts);
        assert_eq!(
            result.program_result,
            Err(solana_sdk::instruction::InstructionError::Custom(
                ANCHOR_ERROR_OFFSET + IFTError::TokenProgramMismatch as u32,
            ))
            .into(),
            "SplToken variant with Token 2022 program should fail"
        );
    }

    #[test]
    fn test_token_2022_with_spl_token_program_fails() {
        let mollusk = setup_mollusk_with_token();
        let (token_program_id, token_program_account) = token_program_keyed_account();

        let setup =
            build_token_2022_setup(token_program_id, token_program_account, 6, "T", "T", "");

        let result = mollusk.process_instruction(&setup.instruction, &setup.accounts);
        assert_eq!(
            result.program_result,
            Err(solana_sdk::instruction::InstructionError::Custom(
                ANCHOR_ERROR_OFFSET + IFTError::TokenProgramMismatch as u32,
            ))
            .into(),
            "Token2022 variant with SPL Token program should fail"
        );
    }

    // ─── Pre-funding griefing resistance tests ────────────────────

    fn adversary_account(lamports: u64) -> solana_sdk::account::Account {
        solana_sdk::account::Account {
            lamports,
            data: vec![],
            owner: solana_sdk::system_program::ID,
            executable: false,
            rent_epoch: 0,
        }
    }

    #[test]
    fn test_prefunded_spl_token_mint_succeeds() {
        let mollusk = setup_mollusk_with_token();
        let (token_program_id, token_program_account) = token_program_keyed_account();

        let setup = build_spl_token_setup(token_program_id, token_program_account, 6);
        let mint_key = setup.instruction.accounts[2].pubkey;

        let adversary = Pubkey::new_unique();
        let prefund_amount: u64 = 500_000;

        let prefund_ix =
            solana_sdk::system_instruction::transfer(&adversary, &mint_key, prefund_amount);

        let mut accounts = setup.accounts;
        accounts.push((
            adversary,
            adversary_account(prefund_amount.saturating_add(1_000_000)),
        ));

        let result = mollusk.process_instruction_chain(&[prefund_ix, setup.instruction], &accounts);
        assert!(
            !result.program_result.is_err(),
            "Partially pre-funded SPL Token mint must not block creation: {:?}",
            result.program_result
        );

        let (_, mint_acc) = &result.resulting_accounts[2];
        let created_mint =
            anchor_spl::token::spl_token::state::Mint::unpack(&mint_acc.data).expect("valid mint");
        assert_eq!(created_mint.decimals, 6);
    }

    #[test]
    fn test_prefunded_token_2022_mint_succeeds() {
        let mollusk = setup_mollusk_with_token_2022();
        let (token_program_id, token_program_account) = token_2022_keyed_account();

        let setup = build_token_2022_setup(
            token_program_id,
            token_program_account,
            9,
            "Test Token",
            "TST",
            "https://example.com/metadata.json",
        );
        let mint_key = setup.instruction.accounts[2].pubkey;

        let adversary = Pubkey::new_unique();
        let prefund_amount: u64 = 500_000;

        let prefund_ix =
            solana_sdk::system_instruction::transfer(&adversary, &mint_key, prefund_amount);

        let mut accounts = setup.accounts;
        accounts.push((
            adversary,
            adversary_account(prefund_amount.saturating_add(1_000_000)),
        ));

        let result = mollusk.process_instruction_chain(&[prefund_ix, setup.instruction], &accounts);
        assert!(
            !result.program_result.is_err(),
            "Partially pre-funded Token 2022 mint must not block creation: {:?}",
            result.program_result
        );

        let (_, mint_acc) = &result.resulting_accounts[2];
        let state = StateWithExtensions::<Token2022Mint>::unpack(&mint_acc.data)
            .expect("valid Token 2022 mint");
        assert_eq!(state.base.decimals, 9);

        let metadata = state
            .get_variable_len_extension::<TokenMetadata>()
            .expect("TokenMetadata should be present");
        assert_eq!(metadata.name, "Test Token");
        assert_eq!(metadata.symbol, "TST");
    }

    /// Compute the total lamports Token 2022 needs for a mint with metadata.
    ///
    /// Mirrors the production calculation: `rent.minimum_balance(extension_space + metadata_space)`.
    fn token_2022_required_lamports(name: &str, symbol: &str, uri: &str) -> u64 {
        use anchor_spl::token_2022_extensions::spl_pod::optional_keys::OptionalNonZeroPubkey;
        use anchor_spl::token_2022_extensions::spl_token_metadata_interface::state::TokenMetadata;
        use anchor_spl::token_interface::spl_token_2022::state::Mint as T22Mint;

        let metadata = TokenMetadata {
            update_authority: OptionalNonZeroPubkey::default(),
            mint: Pubkey::default(),
            name: name.to_string(),
            symbol: symbol.to_string(),
            uri: uri.to_string(),
            additional_metadata: vec![],
        };

        let extension_space =
            ExtensionType::try_calculate_account_len::<T22Mint>(&[ExtensionType::MetadataPointer])
                .unwrap();
        let metadata_space = metadata.tlv_size_of().unwrap();
        let total_space = extension_space.saturating_add(metadata_space);
        Rent::default().minimum_balance(total_space)
    }

    #[test]
    fn test_exact_prefunded_token_2022_mint_succeeds() {
        let mollusk = setup_mollusk_with_token_2022();
        let (token_program_id, token_program_account) = token_2022_keyed_account();

        let name = "Exact Token";
        let symbol = "EXT";
        let uri = "https://example.com/exact.json";

        let setup = build_token_2022_setup(
            token_program_id,
            token_program_account,
            6,
            name,
            symbol,
            uri,
        );
        let mint_key = setup.instruction.accounts[2].pubkey;

        let exact_rent = token_2022_required_lamports(name, symbol, uri);

        let adversary = Pubkey::new_unique();
        let prefund_ix =
            solana_sdk::system_instruction::transfer(&adversary, &mint_key, exact_rent);

        let mut accounts = setup.accounts;
        accounts.push((
            adversary,
            adversary_account(exact_rent.saturating_add(1_000_000)),
        ));

        let result = mollusk.process_instruction_chain(&[prefund_ix, setup.instruction], &accounts);
        assert!(
            !result.program_result.is_err(),
            "Exactly pre-funded Token 2022 mint must not block creation: {:?}",
            result.program_result
        );

        let (_, mint_acc) = &result.resulting_accounts[2];
        let state = StateWithExtensions::<Token2022Mint>::unpack(&mint_acc.data)
            .expect("valid Token 2022 mint");
        assert_eq!(state.base.decimals, 6);

        let metadata = state
            .get_variable_len_extension::<TokenMetadata>()
            .expect("TokenMetadata should be present");
        assert_eq!(metadata.name, name);
    }

    #[test]
    fn test_overfunded_token_2022_mint_succeeds() {
        let mollusk = setup_mollusk_with_token_2022();
        let (token_program_id, token_program_account) = token_2022_keyed_account();

        let name = "Over Token";
        let symbol = "OVR";
        let uri = "";

        let setup = build_token_2022_setup(
            token_program_id,
            token_program_account,
            6,
            name,
            symbol,
            uri,
        );
        let mint_key = setup.instruction.accounts[2].pubkey;

        let overfund_amount =
            token_2022_required_lamports(name, symbol, uri).saturating_add(5_000_000);

        let adversary = Pubkey::new_unique();
        let prefund_ix =
            solana_sdk::system_instruction::transfer(&adversary, &mint_key, overfund_amount);

        let mut accounts = setup.accounts;
        accounts.push((
            adversary,
            adversary_account(overfund_amount.saturating_add(1_000_000)),
        ));

        let result = mollusk.process_instruction_chain(&[prefund_ix, setup.instruction], &accounts);
        assert!(
            !result.program_result.is_err(),
            "Over-funded Token 2022 mint must not block creation: {:?}",
            result.program_result
        );

        let (_, mint_acc) = &result.resulting_accounts[2];
        let state = StateWithExtensions::<Token2022Mint>::unpack(&mint_acc.data)
            .expect("valid Token 2022 mint");
        assert_eq!(state.base.decimals, 6);

        let metadata = state
            .get_variable_len_extension::<TokenMetadata>()
            .expect("TokenMetadata should be present");
        assert_eq!(metadata.name, name);
    }

    #[test]
    fn test_exact_prefunded_spl_token_mint_succeeds() {
        let mollusk = setup_mollusk_with_token();
        let (token_program_id, token_program_account) = token_program_keyed_account();

        let setup = build_spl_token_setup(token_program_id, token_program_account, 6);
        let mint_key = setup.instruction.accounts[2].pubkey;

        let space = anchor_spl::token::spl_token::state::Mint::LEN;
        let exact_rent = Rent::default().minimum_balance(space);

        let adversary = Pubkey::new_unique();
        let prefund_ix =
            solana_sdk::system_instruction::transfer(&adversary, &mint_key, exact_rent);

        let mut accounts = setup.accounts;
        accounts.push((
            adversary,
            adversary_account(exact_rent.saturating_add(1_000_000)),
        ));

        let result = mollusk.process_instruction_chain(&[prefund_ix, setup.instruction], &accounts);
        assert!(
            !result.program_result.is_err(),
            "Exactly pre-funded SPL Token mint must not block creation: {:?}",
            result.program_result
        );

        let (_, mint_acc) = &result.resulting_accounts[2];
        let created_mint =
            anchor_spl::token::spl_token::state::Mint::unpack(&mint_acc.data).expect("valid mint");
        assert_eq!(created_mint.decimals, 6);
    }

    #[test]
    fn test_overfunded_spl_token_mint_succeeds() {
        let mollusk = setup_mollusk_with_token();
        let (token_program_id, token_program_account) = token_program_keyed_account();

        let setup = build_spl_token_setup(token_program_id, token_program_account, 6);
        let mint_key = setup.instruction.accounts[2].pubkey;

        let space = anchor_spl::token::spl_token::state::Mint::LEN;
        let overfund_amount = Rent::default()
            .minimum_balance(space)
            .saturating_add(5_000_000);

        let adversary = Pubkey::new_unique();
        let prefund_ix =
            solana_sdk::system_instruction::transfer(&adversary, &mint_key, overfund_amount);

        let mut accounts = setup.accounts;
        accounts.push((
            adversary,
            adversary_account(overfund_amount.saturating_add(1_000_000)),
        ));

        let result = mollusk.process_instruction_chain(&[prefund_ix, setup.instruction], &accounts);
        assert!(
            !result.program_result.is_err(),
            "Over-funded SPL Token mint must not block creation: {:?}",
            result.program_result
        );

        let (_, mint_acc) = &result.resulting_accounts[2];
        let created_mint =
            anchor_spl::token::spl_token::state::Mint::unpack(&mint_acc.data).expect("valid mint");
        assert_eq!(created_mint.decimals, 6);
    }
}
