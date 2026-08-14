// SPDX-License-Identifier: Apache-2.0

use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token_interface::{Mint, TokenAccount, TokenInterface},
};
use solana_ibc_types::reject_cpi;

use crate::constants::*;
use crate::errors::IFTError;
use crate::events::{
    AdminAccepted, AdminMintExecuted, AdminProposalCancelled, AdminProposed, MintAuthorityRevoked,
};
use crate::helpers::mint_to_account;
use crate::state::{AdminMintMsg, IFTAppMintState, IFTAppState};

#[derive(Accounts)]
#[instruction(new_admin: Pubkey)]
pub struct ProposeAdmin<'info> {
    /// Global IFT app state (mut, `pending_admin` field will be updated)
    #[account(
        mut,
        seeds = [IFT_APP_STATE_SEED],
        bump = app_state.bump
    )]
    pub app_state: Account<'info, IFTAppState>,

    /// Current admin authority, must match `app_state.admin`
    #[account(
        constraint = admin.key() == app_state.admin @ IFTError::UnauthorizedAdmin
    )]
    pub admin: Signer<'info>,

    /// CHECK: Address constraint verifies this is the instructions sysvar
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instructions_sysvar: AccountInfo<'info>,
}

/// Propose a new admin. The proposed admin must call `accept_admin` to finalize.
pub fn propose_admin(ctx: Context<ProposeAdmin>, new_admin: Pubkey) -> Result<()> {
    reject_cpi(&ctx.accounts.instructions_sysvar, &crate::ID).map_err(IFTError::from)?;

    require!(
        new_admin != Pubkey::default(),
        IFTError::InvalidProposedAdmin
    );
    require!(
        new_admin != ctx.accounts.admin.key(),
        IFTError::SelfProposal
    );
    require!(
        ctx.accounts.app_state.pending_admin.is_none(),
        IFTError::PendingAdminAlreadyExists
    );

    ctx.accounts.app_state.pending_admin = Some(new_admin);

    let clock = Clock::get()?;
    emit!(AdminProposed {
        current_admin: ctx.accounts.admin.key(),
        proposed_admin: new_admin,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct AcceptAdmin<'info> {
    /// Global IFT app state (mut, `admin` and `pending_admin` will be updated)
    #[account(
        mut,
        seeds = [IFT_APP_STATE_SEED],
        bump = app_state.bump
    )]
    pub app_state: Account<'info, IFTAppState>,

    /// Pending admin who accepts the role
    #[account(
        constraint = app_state.pending_admin.is_some_and(|p| p == pending_admin.key()) @ IFTError::UnauthorizedPendingAdmin
    )]
    pub pending_admin: Signer<'info>,

    /// CHECK: Address constraint verifies this is the instructions sysvar
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instructions_sysvar: AccountInfo<'info>,
}

/// Accept a pending admin proposal. Must be called by the proposed admin.
pub fn accept_admin(ctx: Context<AcceptAdmin>) -> Result<()> {
    reject_cpi(&ctx.accounts.instructions_sysvar, &crate::ID).map_err(IFTError::from)?;

    let previous_admin = ctx.accounts.app_state.admin;
    ctx.accounts.app_state.admin = ctx.accounts.pending_admin.key();
    ctx.accounts.app_state.pending_admin = None;

    let clock = Clock::get()?;
    emit!(AdminAccepted {
        previous_admin,
        new_admin: ctx.accounts.pending_admin.key(),
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct CancelAdminProposal<'info> {
    /// Global IFT app state (mut, `pending_admin` will be cleared)
    #[account(
        mut,
        seeds = [IFT_APP_STATE_SEED],
        bump = app_state.bump
    )]
    pub app_state: Account<'info, IFTAppState>,

    /// Current admin authority, must match `app_state.admin`
    #[account(
        constraint = admin.key() == app_state.admin @ IFTError::UnauthorizedAdmin
    )]
    pub admin: Signer<'info>,

    /// CHECK: Address constraint verifies this is the instructions sysvar
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instructions_sysvar: AccountInfo<'info>,
}

/// Cancel a pending admin proposal. Must be called by the current admin.
pub fn cancel_admin_proposal(ctx: Context<CancelAdminProposal>) -> Result<()> {
    reject_cpi(&ctx.accounts.instructions_sysvar, &crate::ID).map_err(IFTError::from)?;

    let cancelled_admin = ctx
        .accounts
        .app_state
        .pending_admin
        .ok_or_else(|| error!(IFTError::NoPendingAdmin))?;

    ctx.accounts.app_state.pending_admin = None;

    let clock = Clock::get()?;
    emit!(AdminProposalCancelled {
        admin: ctx.accounts.admin.key(),
        cancelled_admin,
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

/// Revoke mint authority from IFT and transfer it to a new authority.
#[derive(Accounts)]
pub struct RevokeMintAuthority<'info> {
    /// Global IFT app state (read-only, for admin check)
    #[account(
        seeds = [IFT_APP_STATE_SEED],
        bump = app_state.bump
    )]
    pub app_state: Account<'info, IFTAppState>,

    /// Per-mint IFT app state (closed on revocation)
    #[account(
        mut,
        close = admin,
        seeds = [IFT_APP_MINT_STATE_SEED, mint.key().as_ref()],
        bump = app_mint_state.bump
    )]
    pub app_mint_state: Account<'info, IFTAppMintState>,

    /// SPL Token mint - authority will be transferred
    #[account(
        mut,
        address = app_mint_state.mint
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    /// Current mint authority PDA (IFT's)
    /// CHECK: Derived PDA verified by seeds
    #[account(
        seeds = [MINT_AUTHORITY_SEED, mint.key().as_ref()],
        bump = app_mint_state.mint_authority_bump
    )]
    pub mint_authority: AccountInfo<'info>,

    /// New mint authority to receive ownership
    /// CHECK: Can be any pubkey chosen by admin
    pub new_mint_authority: AccountInfo<'info>,

    /// Admin signer (receives rent from closed `app_mint_state`)
    #[account(
        mut,
        constraint = admin.key() == app_state.admin @ IFTError::UnauthorizedAdmin
    )]
    pub admin: Signer<'info>,

    /// SPL Token or Token 2022 program for the `set_authority` CPI
    pub token_program: Interface<'info, TokenInterface>,

    /// CHECK: Address constraint verifies this is the instructions sysvar
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instructions_sysvar: AccountInfo<'info>,
}

/// Revoke mint authority and transfer it to the specified new authority.
pub fn revoke_mint_authority(ctx: Context<RevokeMintAuthority>) -> Result<()> {
    reject_cpi(&ctx.accounts.instructions_sysvar, &crate::ID).map_err(IFTError::from)?;

    let mint_key = ctx.accounts.mint.key();
    let mint_authority_bump = ctx.accounts.app_mint_state.mint_authority_bump;

    let seeds = &[
        MINT_AUTHORITY_SEED,
        mint_key.as_ref(),
        &[mint_authority_bump],
    ];
    let signer_seeds = &[&seeds[..]];

    anchor_spl::token_interface::set_authority(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token_interface::SetAuthority {
                current_authority: ctx.accounts.mint_authority.to_account_info(),
                account_or_mint: ctx.accounts.mint.to_account_info(),
            },
            signer_seeds,
        ),
        anchor_spl::token_interface::spl_token_2022::instruction::AuthorityType::MintTokens,
        Some(ctx.accounts.new_mint_authority.key()),
    )?;

    let clock = Clock::get()?;
    emit!(MintAuthorityRevoked {
        mint: ctx.accounts.mint.key(),
        new_authority: ctx.accounts.new_mint_authority.key(),
        timestamp: clock.unix_timestamp,
    });

    Ok(())
}

/// Admin-callable mint instruction
#[derive(Accounts)]
#[instruction(msg: AdminMintMsg)]
pub struct AdminMint<'info> {
    /// Global IFT app state (read-only, for admin + pause check)
    #[account(
        seeds = [IFT_APP_STATE_SEED],
        bump = app_state.bump,
        constraint = !app_state.paused @ IFTError::AppPaused,
    )]
    pub app_state: Account<'info, IFTAppState>,

    /// Per-mint IFT app state (mut for rate limits)
    #[account(
        mut,
        seeds = [IFT_APP_MINT_STATE_SEED, mint.key().as_ref()],
        bump = app_mint_state.bump
    )]
    pub app_mint_state: Account<'info, IFTAppMintState>,

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
    #[account(
        constraint = receiver_owner.key() == msg.receiver @ IFTError::InvalidReceiver
    )]
    pub receiver_owner: AccountInfo<'info>,

    /// Admin signer
    #[account(
        constraint = admin.key() == app_state.admin @ IFTError::UnauthorizedAdmin
    )]
    pub admin: Signer<'info>,

    /// Pays for ATA creation (if needed) and transaction fees
    #[account(mut)]
    pub payer: Signer<'info>,

    /// SPL Token or Token 2022 program for the mint CPI
    pub token_program: Interface<'info, TokenInterface>,
    /// Creates the receiver's associated token account if it doesn't exist
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// Required for ATA creation
    pub system_program: Program<'info, System>,

    /// CHECK: Address constraint verifies this is the instructions sysvar
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instructions_sysvar: AccountInfo<'info>,
}

/// Mint tokens to any account (admin only). Respects rate limits and pause state.
pub fn admin_mint(ctx: Context<AdminMint>, msg: AdminMintMsg) -> Result<()> {
    reject_cpi(&ctx.accounts.instructions_sysvar, &crate::ID).map_err(IFTError::from)?;

    let clock = Clock::get()?;

    require!(msg.amount > 0, IFTError::ZeroAmount);

    mint_to_account(
        &mut ctx.accounts.app_mint_state,
        &clock,
        &mut ctx.accounts.mint,
        &mut ctx.accounts.receiver_token_account,
        &ctx.accounts.mint_authority,
        &ctx.accounts.token_program,
        msg.amount,
    )?;

    emit!(AdminMintExecuted {
        mint: ctx.accounts.mint.key(),
        receiver: msg.receiver,
        amount: msg.amount,
        admin: ctx.accounts.admin.key(),
        timestamp: clock.unix_timestamp,
    });

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
    use crate::state::AdminMintMsg;
    use crate::test_utils::*;

    #[test]
    fn test_propose_admin_success() {
        let mollusk = setup_mollusk();

        let admin = Pubkey::new_unique();
        let new_admin = Pubkey::new_unique();
        let (app_state_pda, app_state_bump) = get_app_state_pda();
        let (sysvar_id, sysvar_account) = create_instructions_sysvar_account();

        let app_state_account = create_ift_app_state_account(app_state_bump, admin);

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(app_state_pda, false),
                AccountMeta::new_readonly(admin, true),
                AccountMeta::new_readonly(sysvar_id, false),
            ],
            data: crate::instruction::ProposeAdmin { new_admin }.data(),
        };

        let accounts = vec![
            (app_state_pda, app_state_account),
            (admin, create_signer_account()),
            (sysvar_id, sysvar_account),
        ];

        let result = mollusk.process_instruction(&instruction, &accounts);
        assert!(
            !result.program_result.is_err(),
            "propose_admin should succeed: {:?}",
            result.program_result
        );

        let updated_account = result
            .resulting_accounts
            .iter()
            .find(|(k, _)| *k == app_state_pda)
            .expect("app state should exist")
            .1
            .clone();
        let updated_state = deserialize_app_state(&updated_account);
        assert_eq!(updated_state.admin, admin, "Admin should remain unchanged");
        assert_eq!(
            updated_state.pending_admin,
            Some(new_admin),
            "Pending admin should be set"
        );
    }

    #[test]
    fn test_propose_admin_unauthorized() {
        let mollusk = setup_mollusk();

        let admin = Pubkey::new_unique();
        let unauthorized = Pubkey::new_unique();
        let new_admin = Pubkey::new_unique();

        let (app_state_pda, app_state_bump) = get_app_state_pda();
        let (sysvar_id, sysvar_account) = create_instructions_sysvar_account();

        let app_state_account = create_ift_app_state_account(app_state_bump, admin);

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(app_state_pda, false),
                AccountMeta::new_readonly(unauthorized, true),
                AccountMeta::new_readonly(sysvar_id, false),
            ],
            data: crate::instruction::ProposeAdmin { new_admin }.data(),
        };

        let accounts = vec![
            (app_state_pda, app_state_account),
            (unauthorized, create_signer_account()),
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
    fn test_propose_admin_cpi_rejected() {
        let mollusk = setup_mollusk();

        let admin = Pubkey::new_unique();
        let new_admin = Pubkey::new_unique();
        let (app_state_pda, app_state_bump) = get_app_state_pda();
        let (sysvar_id, sysvar_account) =
            create_cpi_instructions_sysvar_account(Pubkey::new_unique());

        let app_state_account = create_ift_app_state_account(app_state_bump, admin);

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(app_state_pda, false),
                AccountMeta::new_readonly(admin, true),
                AccountMeta::new_readonly(sysvar_id, false),
            ],
            data: crate::instruction::ProposeAdmin { new_admin }.data(),
        };

        let accounts = vec![
            (app_state_pda, app_state_account),
            (admin, create_signer_account()),
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

    #[test]
    fn test_propose_admin_zero_address_rejected() {
        let mollusk = setup_mollusk();

        let admin = Pubkey::new_unique();
        let (app_state_pda, app_state_bump) = get_app_state_pda();
        let (sysvar_id, sysvar_account) = create_instructions_sysvar_account();

        let app_state_account = create_ift_app_state_account(app_state_bump, admin);

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(app_state_pda, false),
                AccountMeta::new_readonly(admin, true),
                AccountMeta::new_readonly(sysvar_id, false),
            ],
            data: crate::instruction::ProposeAdmin {
                new_admin: Pubkey::default(),
            }
            .data(),
        };

        let accounts = vec![
            (app_state_pda, app_state_account),
            (admin, create_signer_account()),
            (sysvar_id, sysvar_account),
        ];

        let result = mollusk.process_instruction(&instruction, &accounts);
        assert_eq!(
            result.program_result,
            Err(solana_sdk::instruction::InstructionError::Custom(
                ANCHOR_ERROR_OFFSET + IFTError::InvalidProposedAdmin as u32,
            ))
            .into(),
        );
    }

    #[test]
    fn test_propose_admin_self_proposal_rejected() {
        let mollusk = setup_mollusk();

        let admin = Pubkey::new_unique();
        let (app_state_pda, app_state_bump) = get_app_state_pda();
        let (sysvar_id, sysvar_account) = create_instructions_sysvar_account();

        let app_state_account = create_ift_app_state_account(app_state_bump, admin);

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(app_state_pda, false),
                AccountMeta::new_readonly(admin, true),
                AccountMeta::new_readonly(sysvar_id, false),
            ],
            data: crate::instruction::ProposeAdmin { new_admin: admin }.data(),
        };

        let accounts = vec![
            (app_state_pda, app_state_account),
            (admin, create_signer_account()),
            (sysvar_id, sysvar_account),
        ];

        let result = mollusk.process_instruction(&instruction, &accounts);
        assert_eq!(
            result.program_result,
            Err(solana_sdk::instruction::InstructionError::Custom(
                ANCHOR_ERROR_OFFSET + IFTError::SelfProposal as u32,
            ))
            .into(),
        );
    }

    #[test]
    fn test_propose_admin_pending_already_exists() {
        let mollusk = setup_mollusk();

        let admin = Pubkey::new_unique();
        let first_proposed = Pubkey::new_unique();
        let second_proposed = Pubkey::new_unique();
        let (app_state_pda, app_state_bump) = get_app_state_pda();
        let (sysvar_id, sysvar_account) = create_instructions_sysvar_account();

        let app_state_account =
            create_ift_app_state_account_full(app_state_bump, admin, false, Some(first_proposed));

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(app_state_pda, false),
                AccountMeta::new_readonly(admin, true),
                AccountMeta::new_readonly(sysvar_id, false),
            ],
            data: crate::instruction::ProposeAdmin {
                new_admin: second_proposed,
            }
            .data(),
        };

        let accounts = vec![
            (app_state_pda, app_state_account),
            (admin, create_signer_account()),
            (sysvar_id, sysvar_account),
        ];

        let result = mollusk.process_instruction(&instruction, &accounts);
        assert_eq!(
            result.program_result,
            Err(solana_sdk::instruction::InstructionError::Custom(
                ANCHOR_ERROR_OFFSET + IFTError::PendingAdminAlreadyExists as u32,
            ))
            .into(),
        );
    }

    #[test]
    fn test_accept_admin_success() {
        let mollusk = setup_mollusk();

        let admin = Pubkey::new_unique();
        let new_admin = Pubkey::new_unique();
        let (app_state_pda, app_state_bump) = get_app_state_pda();
        let (sysvar_id, sysvar_account) = create_instructions_sysvar_account();

        let app_state_account =
            create_ift_app_state_account_full(app_state_bump, admin, false, Some(new_admin));

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(app_state_pda, false),
                AccountMeta::new_readonly(new_admin, true),
                AccountMeta::new_readonly(sysvar_id, false),
            ],
            data: crate::instruction::AcceptAdmin {}.data(),
        };

        let accounts = vec![
            (app_state_pda, app_state_account),
            (new_admin, create_signer_account()),
            (sysvar_id, sysvar_account),
        ];

        let result = mollusk.process_instruction(&instruction, &accounts);
        assert!(
            !result.program_result.is_err(),
            "accept_admin should succeed: {:?}",
            result.program_result
        );

        let updated_account = result
            .resulting_accounts
            .iter()
            .find(|(k, _)| *k == app_state_pda)
            .expect("app state should exist")
            .1
            .clone();
        let updated_state = deserialize_app_state(&updated_account);
        assert_eq!(updated_state.admin, new_admin, "Admin should be updated");
        assert_eq!(
            updated_state.pending_admin, None,
            "Pending admin should be cleared"
        );
    }

    #[test]
    fn test_accept_admin_wrong_signer() {
        let mollusk = setup_mollusk();

        let admin = Pubkey::new_unique();
        let pending = Pubkey::new_unique();
        let wrong_signer = Pubkey::new_unique();
        let (app_state_pda, app_state_bump) = get_app_state_pda();
        let (sysvar_id, sysvar_account) = create_instructions_sysvar_account();

        let app_state_account =
            create_ift_app_state_account_full(app_state_bump, admin, false, Some(pending));

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(app_state_pda, false),
                AccountMeta::new_readonly(wrong_signer, true),
                AccountMeta::new_readonly(sysvar_id, false),
            ],
            data: crate::instruction::AcceptAdmin {}.data(),
        };

        let accounts = vec![
            (app_state_pda, app_state_account),
            (wrong_signer, create_signer_account()),
            (sysvar_id, sysvar_account),
        ];

        let result = mollusk.process_instruction(&instruction, &accounts);
        assert_eq!(
            result.program_result,
            Err(solana_sdk::instruction::InstructionError::Custom(
                ANCHOR_ERROR_OFFSET + IFTError::UnauthorizedPendingAdmin as u32,
            ))
            .into(),
        );
    }

    #[test]
    fn test_accept_admin_no_pending() {
        let mollusk = setup_mollusk();

        let admin = Pubkey::new_unique();
        let random_signer = Pubkey::new_unique();
        let (app_state_pda, app_state_bump) = get_app_state_pda();
        let (sysvar_id, sysvar_account) = create_instructions_sysvar_account();

        let app_state_account = create_ift_app_state_account(app_state_bump, admin);

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(app_state_pda, false),
                AccountMeta::new_readonly(random_signer, true),
                AccountMeta::new_readonly(sysvar_id, false),
            ],
            data: crate::instruction::AcceptAdmin {}.data(),
        };

        let accounts = vec![
            (app_state_pda, app_state_account),
            (random_signer, create_signer_account()),
            (sysvar_id, sysvar_account),
        ];

        let result = mollusk.process_instruction(&instruction, &accounts);
        assert_eq!(
            result.program_result,
            Err(solana_sdk::instruction::InstructionError::Custom(
                ANCHOR_ERROR_OFFSET + IFTError::UnauthorizedPendingAdmin as u32,
            ))
            .into(),
        );
    }

    #[test]
    fn test_accept_admin_cpi_rejected() {
        let mollusk = setup_mollusk();

        let admin = Pubkey::new_unique();
        let new_admin = Pubkey::new_unique();
        let (app_state_pda, app_state_bump) = get_app_state_pda();
        let (sysvar_id, sysvar_account) =
            create_cpi_instructions_sysvar_account(Pubkey::new_unique());

        let app_state_account =
            create_ift_app_state_account_full(app_state_bump, admin, false, Some(new_admin));

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(app_state_pda, false),
                AccountMeta::new_readonly(new_admin, true),
                AccountMeta::new_readonly(sysvar_id, false),
            ],
            data: crate::instruction::AcceptAdmin {}.data(),
        };

        let accounts = vec![
            (app_state_pda, app_state_account),
            (new_admin, create_signer_account()),
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

    #[test]
    fn test_cancel_admin_proposal_success() {
        let mollusk = setup_mollusk();

        let admin = Pubkey::new_unique();
        let proposed = Pubkey::new_unique();
        let (app_state_pda, app_state_bump) = get_app_state_pda();
        let (sysvar_id, sysvar_account) = create_instructions_sysvar_account();

        let app_state_account =
            create_ift_app_state_account_full(app_state_bump, admin, false, Some(proposed));

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(app_state_pda, false),
                AccountMeta::new_readonly(admin, true),
                AccountMeta::new_readonly(sysvar_id, false),
            ],
            data: crate::instruction::CancelAdminProposal {}.data(),
        };

        let accounts = vec![
            (app_state_pda, app_state_account),
            (admin, create_signer_account()),
            (sysvar_id, sysvar_account),
        ];

        let result = mollusk.process_instruction(&instruction, &accounts);
        assert!(
            !result.program_result.is_err(),
            "cancel_admin_proposal should succeed: {:?}",
            result.program_result
        );

        let updated_account = result
            .resulting_accounts
            .iter()
            .find(|(k, _)| *k == app_state_pda)
            .expect("app state should exist")
            .1
            .clone();
        let updated_state = deserialize_app_state(&updated_account);
        assert_eq!(updated_state.admin, admin, "Admin should remain unchanged");
        assert_eq!(
            updated_state.pending_admin, None,
            "Pending admin should be cleared"
        );
    }

    #[test]
    fn test_cancel_admin_proposal_no_pending() {
        let mollusk = setup_mollusk();

        let admin = Pubkey::new_unique();
        let (app_state_pda, app_state_bump) = get_app_state_pda();
        let (sysvar_id, sysvar_account) = create_instructions_sysvar_account();

        let app_state_account = create_ift_app_state_account(app_state_bump, admin);

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(app_state_pda, false),
                AccountMeta::new_readonly(admin, true),
                AccountMeta::new_readonly(sysvar_id, false),
            ],
            data: crate::instruction::CancelAdminProposal {}.data(),
        };

        let accounts = vec![
            (app_state_pda, app_state_account),
            (admin, create_signer_account()),
            (sysvar_id, sysvar_account),
        ];

        let result = mollusk.process_instruction(&instruction, &accounts);
        assert_eq!(
            result.program_result,
            Err(solana_sdk::instruction::InstructionError::Custom(
                ANCHOR_ERROR_OFFSET + IFTError::NoPendingAdmin as u32,
            ))
            .into(),
        );
    }

    #[test]
    fn test_cancel_admin_proposal_unauthorized() {
        let mollusk = setup_mollusk();

        let admin = Pubkey::new_unique();
        let unauthorized = Pubkey::new_unique();
        let proposed = Pubkey::new_unique();
        let (app_state_pda, app_state_bump) = get_app_state_pda();
        let (sysvar_id, sysvar_account) = create_instructions_sysvar_account();

        let app_state_account =
            create_ift_app_state_account_full(app_state_bump, admin, false, Some(proposed));

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(app_state_pda, false),
                AccountMeta::new_readonly(unauthorized, true),
                AccountMeta::new_readonly(sysvar_id, false),
            ],
            data: crate::instruction::CancelAdminProposal {}.data(),
        };

        let accounts = vec![
            (app_state_pda, app_state_account),
            (unauthorized, create_signer_account()),
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
    fn test_cancel_admin_proposal_cpi_rejected() {
        let mollusk = setup_mollusk();

        let admin = Pubkey::new_unique();
        let proposed = Pubkey::new_unique();
        let (app_state_pda, app_state_bump) = get_app_state_pda();
        let (sysvar_id, sysvar_account) =
            create_cpi_instructions_sysvar_account(Pubkey::new_unique());

        let app_state_account =
            create_ift_app_state_account_full(app_state_bump, admin, false, Some(proposed));

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(app_state_pda, false),
                AccountMeta::new_readonly(admin, true),
                AccountMeta::new_readonly(sysvar_id, false),
            ],
            data: crate::instruction::CancelAdminProposal {}.data(),
        };

        let accounts = vec![
            (app_state_pda, app_state_account),
            (admin, create_signer_account()),
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

    #[test]
    fn test_cancel_then_propose_and_accept_new_admin() {
        let mollusk = setup_mollusk();

        let admin = Pubkey::new_unique();
        let first_proposed = Pubkey::new_unique();
        let second_proposed = Pubkey::new_unique();
        let (app_state_pda, app_state_bump) = get_app_state_pda();

        // Start with a pending proposal for first_proposed
        let app_state_account =
            create_ift_app_state_account_full(app_state_bump, admin, false, Some(first_proposed));

        // Step 1: Cancel the proposal
        let (sysvar_id, sysvar_account) = create_instructions_sysvar_account();
        let cancel_ix = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(app_state_pda, false),
                AccountMeta::new_readonly(admin, true),
                AccountMeta::new_readonly(sysvar_id, false),
            ],
            data: crate::instruction::CancelAdminProposal {}.data(),
        };
        let cancel_result = mollusk.process_instruction(
            &cancel_ix,
            &[
                (app_state_pda, app_state_account),
                (admin, create_signer_account()),
                (sysvar_id, sysvar_account),
            ],
        );
        assert!(
            !cancel_result.program_result.is_err(),
            "cancel should succeed: {:?}",
            cancel_result.program_result
        );

        let cancelled_state = deserialize_app_state(
            &cancel_result
                .resulting_accounts
                .iter()
                .find(|(k, _)| *k == app_state_pda)
                .unwrap()
                .1,
        );
        assert_eq!(cancelled_state.admin, admin);
        assert_eq!(cancelled_state.pending_admin, None);

        // Step 2: Propose a different admin
        let post_cancel_account = cancel_result
            .resulting_accounts
            .into_iter()
            .find(|(k, _)| *k == app_state_pda)
            .unwrap()
            .1;

        let (sysvar_id, sysvar_account) = create_instructions_sysvar_account();
        let propose_ix = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(app_state_pda, false),
                AccountMeta::new_readonly(admin, true),
                AccountMeta::new_readonly(sysvar_id, false),
            ],
            data: crate::instruction::ProposeAdmin {
                new_admin: second_proposed,
            }
            .data(),
        };
        let propose_result = mollusk.process_instruction(
            &propose_ix,
            &[
                (app_state_pda, post_cancel_account),
                (admin, create_signer_account()),
                (sysvar_id, sysvar_account),
            ],
        );
        assert!(
            !propose_result.program_result.is_err(),
            "propose should succeed: {:?}",
            propose_result.program_result
        );

        let proposed_state = deserialize_app_state(
            &propose_result
                .resulting_accounts
                .iter()
                .find(|(k, _)| *k == app_state_pda)
                .unwrap()
                .1,
        );
        assert_eq!(proposed_state.admin, admin);
        assert_eq!(proposed_state.pending_admin, Some(second_proposed));

        // Step 3: Accept with the second proposed admin
        let post_propose_account = propose_result
            .resulting_accounts
            .into_iter()
            .find(|(k, _)| *k == app_state_pda)
            .unwrap()
            .1;

        let (sysvar_id, sysvar_account) = create_instructions_sysvar_account();
        let accept_ix = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(app_state_pda, false),
                AccountMeta::new_readonly(second_proposed, true),
                AccountMeta::new_readonly(sysvar_id, false),
            ],
            data: crate::instruction::AcceptAdmin {}.data(),
        };
        let accept_result = mollusk.process_instruction(
            &accept_ix,
            &[
                (app_state_pda, post_propose_account),
                (second_proposed, create_signer_account()),
                (sysvar_id, sysvar_account),
            ],
        );
        assert!(
            !accept_result.program_result.is_err(),
            "accept should succeed: {:?}",
            accept_result.program_result
        );

        let final_state = deserialize_app_state(
            &accept_result
                .resulting_accounts
                .iter()
                .find(|(k, _)| *k == app_state_pda)
                .unwrap()
                .1,
        );
        assert_eq!(
            final_state.admin, second_proposed,
            "Admin should be the second proposed admin"
        );
        assert_eq!(
            final_state.pending_admin, None,
            "Pending admin should be cleared"
        );
    }

    #[derive(Clone, Copy)]
    enum AdminMintErrorCase {
        Unauthorized,
        ZeroAmount,
        AppPaused,
        RateLimitExceeded,
        ReceiverMismatch,
    }

    #[allow(clippy::struct_excessive_bools)]
    struct AdminMintTestConfig {
        amount: u64,
        use_real_admin: bool,
        use_wrong_receiver: bool,
        paused: bool,
        daily_mint_limit: u64,
        rate_limit_daily_usage: u64,
        expected_error: u32,
    }

    impl From<AdminMintErrorCase> for AdminMintTestConfig {
        fn from(case: AdminMintErrorCase) -> Self {
            let default = Self {
                amount: 1000,
                use_real_admin: true,
                use_wrong_receiver: false,
                paused: false,
                daily_mint_limit: 0,
                rate_limit_daily_usage: 0,
                expected_error: 0,
            };

            match case {
                AdminMintErrorCase::Unauthorized => Self {
                    use_real_admin: false,
                    expected_error: ANCHOR_ERROR_OFFSET + IFTError::UnauthorizedAdmin as u32,
                    ..default
                },
                AdminMintErrorCase::ZeroAmount => Self {
                    amount: 0,
                    expected_error: ANCHOR_ERROR_OFFSET + IFTError::ZeroAmount as u32,
                    ..default
                },
                AdminMintErrorCase::AppPaused => Self {
                    paused: true,
                    expected_error: ANCHOR_ERROR_OFFSET + IFTError::AppPaused as u32,
                    ..default
                },
                AdminMintErrorCase::RateLimitExceeded => Self {
                    daily_mint_limit: 100,
                    rate_limit_daily_usage: 100,
                    expected_error: ANCHOR_ERROR_OFFSET + IFTError::MintRateLimitExceeded as u32,
                    ..default
                },
                AdminMintErrorCase::ReceiverMismatch => Self {
                    use_wrong_receiver: true,
                    expected_error: ANCHOR_ERROR_OFFSET + IFTError::InvalidReceiver as u32,
                    ..default
                },
            }
        }
    }

    fn run_admin_mint_error_test(case: AdminMintErrorCase) {
        let config = AdminMintTestConfig::from(case);
        let mollusk = setup_mollusk();

        let mint = Pubkey::new_unique();
        let admin = Pubkey::new_unique();
        let unauthorized = Pubkey::new_unique();
        let receiver = Pubkey::new_unique();
        let wrong_receiver = Pubkey::new_unique();
        let payer = Pubkey::new_unique();

        let (app_state_pda, app_state_bump) = get_app_state_pda();
        let (app_mint_state_pda, app_mint_state_bump) = get_app_mint_state_pda(&mint);
        let (mint_authority_pda, mint_authority_bump) = get_mint_authority_pda(&mint);
        let (system_program, system_account) = create_system_program_account();
        let (token_program_id, token_program_account) = create_token_program_account();
        let (sysvar_id, sysvar_account) = create_instructions_sysvar_account();

        let app_state_account =
            create_ift_app_state_account_with_options(app_state_bump, admin, config.paused);

        let app_mint_state_account =
            create_ift_app_mint_state_account_full(IftAppMintStateParams {
                mint,
                bump: app_mint_state_bump,
                mint_authority_bump,
                daily_mint_limit: config.daily_mint_limit,
                rate_limit_day: 0,
                rate_limit_daily_usage: config.rate_limit_daily_usage,
            });

        let mint_account = create_mint_account(mint_authority_pda, 6);

        let receiver_owner_key = if config.use_wrong_receiver {
            wrong_receiver
        } else {
            receiver
        };

        let receiver_token_pda =
            anchor_spl::associated_token::get_associated_token_address(&receiver_owner_key, &mint);
        let receiver_token_account = create_token_account(mint, receiver_owner_key, 0);

        let associated_token_program_account = solana_sdk::account::Account {
            lamports: 1,
            data: vec![],
            owner: solana_sdk::native_loader::ID,
            executable: true,
            rent_epoch: 0,
        };

        let signer_key = if config.use_real_admin {
            admin
        } else {
            unauthorized
        };

        let msg = crate::state::AdminMintMsg {
            receiver,
            amount: config.amount,
        };

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new_readonly(app_state_pda, false),
                AccountMeta::new(app_mint_state_pda, false),
                AccountMeta::new(mint, false),
                AccountMeta::new_readonly(mint_authority_pda, false),
                AccountMeta::new(receiver_token_pda, false),
                AccountMeta::new_readonly(receiver_owner_key, false),
                AccountMeta::new_readonly(signer_key, true),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(token_program_id, false),
                AccountMeta::new_readonly(anchor_spl::associated_token::ID, false),
                AccountMeta::new_readonly(system_program, false),
                AccountMeta::new_readonly(sysvar_id, false),
            ],
            data: crate::instruction::AdminMint { msg }.data(),
        };

        let accounts = vec![
            (app_state_pda, app_state_account),
            (app_mint_state_pda, app_mint_state_account),
            (mint, mint_account),
            (mint_authority_pda, create_signer_account()),
            (receiver_token_pda, receiver_token_account),
            (receiver_owner_key, create_signer_account()),
            (signer_key, create_signer_account()),
            (payer, create_signer_account()),
            (token_program_id, token_program_account),
            (
                anchor_spl::associated_token::ID,
                associated_token_program_account,
            ),
            (system_program, system_account),
            (sysvar_id, sysvar_account),
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
    #[case::unauthorized(AdminMintErrorCase::Unauthorized)]
    #[case::zero_amount(AdminMintErrorCase::ZeroAmount)]
    #[case::app_paused(AdminMintErrorCase::AppPaused)]
    #[case::rate_limit_exceeded(AdminMintErrorCase::RateLimitExceeded)]
    #[case::receiver_mismatch(AdminMintErrorCase::ReceiverMismatch)]
    fn test_admin_mint_validation(#[case] case: AdminMintErrorCase) {
        run_admin_mint_error_test(case);
    }

    #[test]
    fn test_revoke_mint_authority_unauthorized() {
        let mollusk = setup_mollusk();

        let mint = Pubkey::new_unique();
        let admin = Pubkey::new_unique();
        let unauthorized = Pubkey::new_unique();
        let new_mint_authority = Pubkey::new_unique();

        let (app_state_pda, app_state_bump) = get_app_state_pda();
        let (app_mint_state_pda, app_mint_state_bump) = get_app_mint_state_pda(&mint);
        let (mint_authority_pda, mint_authority_bump) = get_mint_authority_pda(&mint);
        let (token_program_id, token_program_account) = create_token_program_account();
        let (sysvar_id, sysvar_account) = create_instructions_sysvar_account();

        let app_state_account = create_ift_app_state_account(app_state_bump, admin);

        let app_mint_state_account =
            create_ift_app_mint_state_account(mint, app_mint_state_bump, mint_authority_bump);

        let mint_account = create_mint_account(mint_authority_pda, 6);

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new_readonly(app_state_pda, false),
                AccountMeta::new(app_mint_state_pda, false),
                AccountMeta::new(mint, false),
                AccountMeta::new_readonly(mint_authority_pda, false),
                AccountMeta::new_readonly(new_mint_authority, false),
                AccountMeta::new(unauthorized, true),
                AccountMeta::new_readonly(token_program_id, false),
                AccountMeta::new_readonly(sysvar_id, false),
            ],
            data: crate::instruction::RevokeMintAuthority {}.data(),
        };

        let accounts = vec![
            (app_state_pda, app_state_account),
            (app_mint_state_pda, app_mint_state_account),
            (mint, mint_account),
            (mint_authority_pda, create_signer_account()),
            (new_mint_authority, create_signer_account()),
            (unauthorized, create_signer_account()),
            (token_program_id, token_program_account),
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
    fn test_admin_mint_success() {
        let mollusk = setup_mollusk_with_token();

        let mint = Pubkey::new_unique();
        let admin = Pubkey::new_unique();
        let receiver = Pubkey::new_unique();
        let payer = Pubkey::new_unique();

        let (app_state_pda, app_state_bump) = get_app_state_pda();
        let (app_mint_state_pda, app_mint_state_bump) = get_app_mint_state_pda(&mint);
        let (mint_authority_pda, mint_authority_bump) = get_mint_authority_pda(&mint);
        let (system_program, system_account) = create_system_program_account();
        let (token_program_id, token_program_account) = token_program_keyed_account();
        let (sysvar_id, sysvar_account) = create_instructions_sysvar_account();

        let app_state_account = create_ift_app_state_account(app_state_bump, admin);
        let app_mint_state_account =
            create_ift_app_mint_state_account(mint, app_mint_state_bump, mint_authority_bump);
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

        let msg = AdminMintMsg {
            receiver,
            amount: 500,
        };

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new_readonly(app_state_pda, false),
                AccountMeta::new(app_mint_state_pda, false),
                AccountMeta::new(mint, false),
                AccountMeta::new_readonly(mint_authority_pda, false),
                AccountMeta::new(receiver_token_pda, false),
                AccountMeta::new_readonly(receiver, false),
                AccountMeta::new_readonly(admin, true),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(token_program_id, false),
                AccountMeta::new_readonly(anchor_spl::associated_token::ID, false),
                AccountMeta::new_readonly(system_program, false),
                AccountMeta::new_readonly(sysvar_id, false),
            ],
            data: crate::instruction::AdminMint { msg }.data(),
        };

        let accounts = vec![
            (app_state_pda, app_state_account),
            (app_mint_state_pda, app_mint_state_account),
            (mint, mint_account),
            (mint_authority_pda, create_uninitialized_pda()),
            (receiver_token_pda, receiver_token_account),
            (receiver, create_signer_account()),
            (admin, create_signer_account()),
            (payer, create_signer_account()),
            (token_program_id, token_program_account),
            (
                anchor_spl::associated_token::ID,
                associated_token_program_account,
            ),
            (system_program, system_account),
            (sysvar_id, sysvar_account),
        ];

        let result = mollusk.process_instruction(&instruction, &accounts);
        assert!(
            !result.program_result.is_err(),
            "admin_mint should succeed: {:?}",
            result.program_result
        );

        let (_, receiver_acc) = &result.resulting_accounts[4];
        let token = anchor_spl::token::spl_token::state::Account::unpack(&receiver_acc.data)
            .expect("valid token account");
        assert_eq!(token.amount, 500);
    }

    #[test]
    fn test_admin_mint_cpi_rejected() {
        let mollusk = setup_mollusk();

        let mint = Pubkey::new_unique();
        let admin = Pubkey::new_unique();
        let receiver = Pubkey::new_unique();
        let payer = Pubkey::new_unique();

        let (app_state_pda, app_state_bump) = get_app_state_pda();
        let (app_mint_state_pda, app_mint_state_bump) = get_app_mint_state_pda(&mint);
        let (mint_authority_pda, mint_authority_bump) = get_mint_authority_pda(&mint);
        let (system_program, system_account) = create_system_program_account();
        let (token_program_id, token_program_account) = create_token_program_account();
        let (sysvar_id, sysvar_account) =
            create_cpi_instructions_sysvar_account(Pubkey::new_unique());

        let app_state_account = create_ift_app_state_account(app_state_bump, admin);
        let app_mint_state_account =
            create_ift_app_mint_state_account(mint, app_mint_state_bump, mint_authority_bump);
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

        let msg = AdminMintMsg {
            receiver,
            amount: 500,
        };

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new_readonly(app_state_pda, false),
                AccountMeta::new(app_mint_state_pda, false),
                AccountMeta::new(mint, false),
                AccountMeta::new_readonly(mint_authority_pda, false),
                AccountMeta::new(receiver_token_pda, false),
                AccountMeta::new_readonly(receiver, false),
                AccountMeta::new_readonly(admin, true),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(token_program_id, false),
                AccountMeta::new_readonly(anchor_spl::associated_token::ID, false),
                AccountMeta::new_readonly(system_program, false),
                AccountMeta::new_readonly(sysvar_id, false),
            ],
            data: crate::instruction::AdminMint { msg }.data(),
        };

        let accounts = vec![
            (app_state_pda, app_state_account),
            (app_mint_state_pda, app_mint_state_account),
            (mint, mint_account),
            (mint_authority_pda, create_signer_account()),
            (receiver_token_pda, receiver_token_account),
            (receiver, create_signer_account()),
            (admin, create_signer_account()),
            (payer, create_signer_account()),
            (token_program_id, token_program_account),
            (
                anchor_spl::associated_token::ID,
                associated_token_program_account,
            ),
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

    #[test]
    fn test_revoke_mint_authority_success() {
        let mollusk = setup_mollusk_with_token();

        let mint = Pubkey::new_unique();
        let admin = Pubkey::new_unique();
        let new_mint_authority = Pubkey::new_unique();

        let (app_state_pda, app_state_bump) = get_app_state_pda();
        let (app_mint_state_pda, app_mint_state_bump) = get_app_mint_state_pda(&mint);
        let (mint_authority_pda, mint_authority_bump) = get_mint_authority_pda(&mint);
        let (token_program_id, token_program_account) = token_program_keyed_account();
        let (sysvar_id, sysvar_account) = create_instructions_sysvar_account();

        let app_state_account = create_ift_app_state_account(app_state_bump, admin);
        let app_mint_state_account =
            create_ift_app_mint_state_account(mint, app_mint_state_bump, mint_authority_bump);
        let mint_account = create_mint_account(mint_authority_pda, 6);

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new_readonly(app_state_pda, false),
                AccountMeta::new(app_mint_state_pda, false),
                AccountMeta::new(mint, false),
                AccountMeta::new_readonly(mint_authority_pda, false),
                AccountMeta::new_readonly(new_mint_authority, false),
                AccountMeta::new(admin, true),
                AccountMeta::new_readonly(token_program_id, false),
                AccountMeta::new_readonly(sysvar_id, false),
            ],
            data: crate::instruction::RevokeMintAuthority {}.data(),
        };

        let accounts = vec![
            (app_state_pda, app_state_account),
            (app_mint_state_pda, app_mint_state_account),
            (mint, mint_account),
            (mint_authority_pda, create_uninitialized_pda()),
            (new_mint_authority, create_signer_account()),
            (admin, create_signer_account()),
            (token_program_id, token_program_account),
            (sysvar_id, sysvar_account),
        ];

        let result = mollusk.process_instruction(&instruction, &accounts);
        assert!(
            !result.program_result.is_err(),
            "revoke_mint_authority should succeed: {:?}",
            result.program_result
        );

        let (_, mint_acc) = &result.resulting_accounts[2];
        let updated_mint = anchor_spl::token::spl_token::state::Mint::unpack(&mint_acc.data)
            .expect("valid mint account");
        assert_eq!(
            updated_mint.mint_authority,
            solana_sdk::program_option::COption::Some(new_mint_authority),
        );

        // Verify app_mint_state was closed (lamports zeroed, data cleared)
        let (_, closed_mint_state) = &result.resulting_accounts[1];
        assert_eq!(
            closed_mint_state.lamports, 0,
            "app_mint_state should be closed (zero lamports)"
        );
    }

    #[test]
    fn test_revoke_mint_authority_cpi_rejected() {
        let mollusk = setup_mollusk();

        let mint = Pubkey::new_unique();
        let admin = Pubkey::new_unique();
        let new_mint_authority = Pubkey::new_unique();

        let (app_state_pda, app_state_bump) = get_app_state_pda();
        let (app_mint_state_pda, app_mint_state_bump) = get_app_mint_state_pda(&mint);
        let (mint_authority_pda, mint_authority_bump) = get_mint_authority_pda(&mint);
        let (token_program_id, token_program_account) = create_token_program_account();
        let (sysvar_id, sysvar_account) =
            create_cpi_instructions_sysvar_account(Pubkey::new_unique());

        let app_state_account = create_ift_app_state_account(app_state_bump, admin);
        let app_mint_state_account =
            create_ift_app_mint_state_account(mint, app_mint_state_bump, mint_authority_bump);
        let mint_account = create_mint_account(mint_authority_pda, 6);

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new_readonly(app_state_pda, false),
                AccountMeta::new(app_mint_state_pda, false),
                AccountMeta::new(mint, false),
                AccountMeta::new_readonly(mint_authority_pda, false),
                AccountMeta::new_readonly(new_mint_authority, false),
                AccountMeta::new(admin, true),
                AccountMeta::new_readonly(token_program_id, false),
                AccountMeta::new_readonly(sysvar_id, false),
            ],
            data: crate::instruction::RevokeMintAuthority {}.data(),
        };

        let accounts = vec![
            (app_state_pda, app_state_account),
            (app_mint_state_pda, app_mint_state_account),
            (mint, mint_account),
            (mint_authority_pda, create_signer_account()),
            (new_mint_authority, create_signer_account()),
            (admin, create_signer_account()),
            (token_program_id, token_program_account),
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
}
