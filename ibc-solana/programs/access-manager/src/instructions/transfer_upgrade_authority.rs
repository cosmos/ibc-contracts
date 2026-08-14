// SPDX-License-Identifier: Apache-2.0

use crate::errors::AccessManagerError;
use crate::events::{
    UpgradeAuthorityTransferCancelledEvent, UpgradeAuthorityTransferProposedEvent,
    UpgradeAuthorityTransferredEvent,
};
use crate::helpers::require_admin;
use crate::state::AccessManager;
use crate::types::PendingAuthorityTransfer;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::bpf_loader_upgradeable;

// ── Propose ──────────────────────────────────────────────────────────────────

/// Proposes transferring a target program's BPF Loader upgrade authority from
/// this access manager's PDA to a new authority. Requires admin authorization.
///
/// The proposed transfer must be accepted by the new authority via
/// `accept_upgrade_authority_transfer` before it takes effect.
#[derive(Accounts)]
pub struct ProposeUpgradeAuthorityTransfer<'info> {
    #[account(
        mut,
        seeds = [AccessManager::SEED],
        bump
    )]
    pub access_manager: Account<'info, AccessManager>,

    /// Must hold `ADMIN_ROLE` on the current access manager.
    pub admin: Signer<'info>,

    /// CHECK: Address constraint verifies this is the instructions sysvar
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instructions_sysvar: AccountInfo<'info>,
}

pub fn propose_upgrade_authority_transfer(
    ctx: Context<ProposeUpgradeAuthorityTransfer>,
    target_program: Pubkey,
    new_authority: Pubkey,
) -> Result<()> {
    require_admin(
        &ctx.accounts.access_manager.to_account_info(),
        &ctx.accounts.admin.to_account_info(),
        &ctx.accounts.instructions_sysvar,
        &crate::ID,
    )?;

    require!(
        new_authority != Pubkey::default(),
        AccessManagerError::ZeroAccount
    );

    let (upgrade_authority_pda, _) =
        AccessManager::upgrade_authority_pda(&target_program, &crate::ID);
    require!(
        new_authority != upgrade_authority_pda,
        AccessManagerError::SelfTransfer
    );

    require!(
        ctx.accounts
            .access_manager
            .pending_authority_transfers
            .len()
            < AccessManager::MAX_PENDING_TRANSFERS,
        AccessManagerError::TooManyPendingTransfers
    );

    require!(
        !ctx.accounts
            .access_manager
            .pending_authority_transfers
            .iter()
            .any(|p| p.target_program == target_program),
        AccessManagerError::PendingTransferAlreadyExists
    );

    ctx.accounts
        .access_manager
        .pending_authority_transfers
        .push(PendingAuthorityTransfer {
            target_program,
            new_authority,
        });

    emit!(UpgradeAuthorityTransferProposedEvent {
        program: target_program,
        current_authority: upgrade_authority_pda,
        proposed_authority: new_authority,
        proposed_by: ctx.accounts.admin.key(),
    });

    Ok(())
}

// ── Accept ───────────────────────────────────────────────────────────────────

/// Accepts a pending upgrade authority transfer by executing the BPF Loader
/// `SetAuthority` CPI. Must be signed by the proposed new authority.
///
/// No CPI restriction: supports both keypair signers and multisig/PDA callers.
#[derive(Accounts)]
#[instruction(target_program: Pubkey)]
pub struct AcceptUpgradeAuthorityTransfer<'info> {
    #[account(
        mut,
        seeds = [AccessManager::SEED],
        bump
    )]
    pub access_manager: Account<'info, AccessManager>,

    /// The target program's data account (BPF Loader Upgradeable PDA).
    /// CHECK: Validated via BPF Loader seeds derivation from `target_program`
    #[account(
        mut,
        seeds = [target_program.as_ref()],
        bump,
        seeds::program = bpf_loader_upgradeable::ID
    )]
    pub program_data: AccountInfo<'info>,

    /// `AccessManager`'s PDA that acts as the current upgrade authority.
    /// CHECK: Validated via seeds constraint
    #[account(
        seeds = [AccessManager::UPGRADE_AUTHORITY_SEED, target_program.as_ref()],
        bump
    )]
    pub upgrade_authority: AccountInfo<'info>,

    /// The proposed new upgrade authority. Must match the pending transfer.
    pub new_authority: Signer<'info>,

    /// CHECK: Must be BPF Loader Upgradeable program ID
    #[account(address = bpf_loader_upgradeable::ID)]
    pub bpf_loader_upgradeable: AccountInfo<'info>,
}

pub fn accept_upgrade_authority_transfer(
    ctx: Context<AcceptUpgradeAuthorityTransfer>,
    target_program: Pubkey,
) -> Result<()> {
    let idx = ctx
        .accounts
        .access_manager
        .pending_authority_transfers
        .iter()
        .position(|p| p.target_program == target_program)
        .ok_or_else(|| error!(AccessManagerError::NoPendingTransfer))?;

    require!(
        ctx.accounts.access_manager.pending_authority_transfers[idx].new_authority
            == ctx.accounts.new_authority.key(),
        AccessManagerError::AuthorityMismatch
    );

    let upgrade_authority_pda = ctx.accounts.upgrade_authority.key();
    let bump = ctx.bumps.upgrade_authority;

    let set_authority_ix = bpf_loader_upgradeable::set_upgrade_authority(
        &target_program,
        &upgrade_authority_pda,
        Some(&ctx.accounts.new_authority.key()),
    );

    // BPF Loader SetAuthority expects:
    //   [0] programdata_address  (writable, non-signer)
    //   [1] current_authority    (read-only, signer via PDA)
    //   [2] new_authority        (read-only, non-signer)
    anchor_lang::solana_program::program::invoke_signed(
        &set_authority_ix,
        &[
            ctx.accounts.program_data.to_account_info(),
            ctx.accounts.upgrade_authority.to_account_info(),
            ctx.accounts.new_authority.to_account_info(),
        ],
        &[&[
            AccessManager::UPGRADE_AUTHORITY_SEED,
            target_program.as_ref(),
            &[bump],
        ]],
    )?;

    let new_authority = ctx.accounts.new_authority.key();
    ctx.accounts
        .access_manager
        .pending_authority_transfers
        .swap_remove(idx);

    emit!(UpgradeAuthorityTransferredEvent {
        program: target_program,
        old_authority: upgrade_authority_pda,
        new_authority,
        accepted_by: new_authority,
    });

    Ok(())
}

// ── Cancel ───────────────────────────────────────────────────────────────────

/// Cancels a pending upgrade authority transfer. Requires admin authorization.
#[derive(Accounts)]
pub struct CancelUpgradeAuthorityTransfer<'info> {
    #[account(
        mut,
        seeds = [AccessManager::SEED],
        bump
    )]
    pub access_manager: Account<'info, AccessManager>,

    /// Must hold `ADMIN_ROLE` on the current access manager.
    pub admin: Signer<'info>,

    /// CHECK: Address constraint verifies this is the instructions sysvar
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instructions_sysvar: AccountInfo<'info>,
}

pub fn cancel_upgrade_authority_transfer(
    ctx: Context<CancelUpgradeAuthorityTransfer>,
    target_program: Pubkey,
) -> Result<()> {
    require_admin(
        &ctx.accounts.access_manager.to_account_info(),
        &ctx.accounts.admin.to_account_info(),
        &ctx.accounts.instructions_sysvar,
        &crate::ID,
    )?;

    let idx = ctx
        .accounts
        .access_manager
        .pending_authority_transfers
        .iter()
        .position(|p| p.target_program == target_program)
        .ok_or_else(|| error!(AccessManagerError::NoPendingTransfer))?;

    let cancelled_authority =
        ctx.accounts.access_manager.pending_authority_transfers[idx].new_authority;

    ctx.accounts
        .access_manager
        .pending_authority_transfers
        .swap_remove(idx);

    emit!(UpgradeAuthorityTransferCancelledEvent {
        program: target_program,
        cancelled_authority,
        cancelled_by: ctx.accounts.admin.key(),
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;
    use mollusk_svm::result::Check;
    use solana_sdk::{account::Account, instruction::AccountMeta};

    // ── Helpers ──────────────────────────────────────────────────────────────

    fn create_access_manager_with_pending(
        admin: Pubkey,
        target_program: Pubkey,
        new_authority: Pubkey,
    ) -> (Pubkey, Account) {
        create_access_manager_with_transfers(
            admin,
            vec![PendingAuthorityTransfer {
                target_program,
                new_authority,
            }],
        )
    }

    fn build_admin_account_metas(
        access_manager_pda: Pubkey,
        authority: Pubkey,
    ) -> Vec<AccountMeta> {
        vec![
            AccountMeta::new(access_manager_pda, false),
            AccountMeta::new_readonly(authority, true),
            AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
        ]
    }

    fn derive_program_data(target_program: &Pubkey) -> Pubkey {
        Pubkey::find_program_address(&[target_program.as_ref()], &bpf_loader_upgradeable::ID).0
    }

    fn build_accept_account_metas(
        access_manager_pda: Pubkey,
        program_data_address: Pubkey,
        upgrade_authority_pda: Pubkey,
        new_authority: Pubkey,
    ) -> Vec<AccountMeta> {
        vec![
            AccountMeta::new(access_manager_pda, false),
            AccountMeta::new(program_data_address, false),
            AccountMeta::new_readonly(upgrade_authority_pda, false),
            AccountMeta::new_readonly(new_authority, true),
            AccountMeta::new_readonly(bpf_loader_upgradeable::ID, false),
        ]
    }

    fn create_accept_accounts(
        program_data_address: Pubkey,
        upgrade_authority_pda: Pubkey,
        new_authority: Pubkey,
    ) -> Vec<(Pubkey, Account)> {
        vec![
            (
                program_data_address,
                Account {
                    lamports: 1_000_000,
                    data: vec![0; 100],
                    owner: bpf_loader_upgradeable::ID,
                    ..Default::default()
                },
            ),
            (
                upgrade_authority_pda,
                Account {
                    lamports: 1_000_000,
                    owner: crate::ID,
                    ..Default::default()
                },
            ),
            (new_authority, create_signer_account()),
            (
                bpf_loader_upgradeable::ID,
                Account {
                    lamports: 1_000_000,
                    executable: true,
                    owner: solana_sdk::native_loader::ID,
                    ..Default::default()
                },
            ),
        ]
    }

    // ── Propose tests ────────────────────────────────────────────────────────

    #[test]
    fn test_propose_success() {
        let admin = Pubkey::new_unique();
        let target_program = Pubkey::new_unique();
        let new_authority = Pubkey::new_unique();

        let (pda, am_account) = create_initialized_access_manager(admin);
        let sysvar_account = create_instructions_sysvar_account();

        let instruction = build_instruction(
            crate::instruction::ProposeUpgradeAuthorityTransfer {
                target_program,
                new_authority,
            },
            build_admin_account_metas(pda, admin),
        );

        let accounts = vec![
            (pda, am_account),
            (admin, create_signer_account()),
            (solana_sdk::sysvar::instructions::ID, sysvar_account),
        ];

        let mollusk = setup_mollusk();
        let result =
            mollusk.process_and_validate_instruction(&instruction, &accounts, &[Check::success()]);

        let am = get_access_manager_from_result(&result, &pda);
        assert!(am
            .pending_authority_transfers
            .contains(&PendingAuthorityTransfer {
                target_program,
                new_authority,
            }));
    }

    #[test]
    fn test_propose_not_admin() {
        let admin = Pubkey::new_unique();
        let non_admin = Pubkey::new_unique();
        let target_program = Pubkey::new_unique();
        let new_authority = Pubkey::new_unique();

        let (pda, am_account) = create_initialized_access_manager(admin);
        let sysvar_account = create_instructions_sysvar_account();

        let instruction = build_instruction(
            crate::instruction::ProposeUpgradeAuthorityTransfer {
                target_program,
                new_authority,
            },
            build_admin_account_metas(pda, non_admin),
        );

        let accounts = vec![
            (pda, am_account),
            (non_admin, create_signer_account()),
            (solana_sdk::sysvar::instructions::ID, sysvar_account),
        ];

        let mollusk = setup_mollusk();
        mollusk.process_and_validate_instruction(
            &instruction,
            &accounts,
            &[Check::err(solana_sdk::program_error::ProgramError::Custom(
                ANCHOR_ERROR_OFFSET + AccessManagerError::Unauthorized as u32,
            ))],
        );
    }

    #[test]
    fn test_propose_cpi_rejection() {
        let admin = Pubkey::new_unique();
        let target_program = Pubkey::new_unique();
        let new_authority = Pubkey::new_unique();

        let (pda, am_account) = create_initialized_access_manager(admin);

        let instruction = build_instruction(
            crate::instruction::ProposeUpgradeAuthorityTransfer {
                target_program,
                new_authority,
            },
            build_admin_account_metas(pda, admin),
        );

        let malicious_program = Pubkey::new_unique();
        let (instruction, cpi_sysvar_account) = setup_cpi_call_test(instruction, malicious_program);

        let accounts = vec![
            (pda, am_account),
            (admin, create_signer_account()),
            cpi_sysvar_account,
        ];

        let mollusk = setup_mollusk();
        mollusk.process_and_validate_instruction(
            &instruction,
            &accounts,
            &[expect_access_manager_cpi_rejection_error()],
        );
    }

    #[test]
    fn test_propose_zero_address() {
        let admin = Pubkey::new_unique();
        let target_program = Pubkey::new_unique();

        let (pda, am_account) = create_initialized_access_manager(admin);
        let sysvar_account = create_instructions_sysvar_account();

        let instruction = build_instruction(
            crate::instruction::ProposeUpgradeAuthorityTransfer {
                target_program,
                new_authority: Pubkey::default(),
            },
            build_admin_account_metas(pda, admin),
        );

        let accounts = vec![
            (pda, am_account),
            (admin, create_signer_account()),
            (solana_sdk::sysvar::instructions::ID, sysvar_account),
        ];

        let mollusk = setup_mollusk();
        mollusk.process_and_validate_instruction(
            &instruction,
            &accounts,
            &[Check::err(solana_sdk::program_error::ProgramError::Custom(
                ANCHOR_ERROR_OFFSET + AccessManagerError::ZeroAccount as u32,
            ))],
        );
    }

    #[test]
    fn test_propose_self_transfer() {
        let admin = Pubkey::new_unique();
        let target_program = Pubkey::new_unique();
        let (upgrade_authority_pda, _) =
            AccessManager::upgrade_authority_pda(&target_program, &crate::ID);

        let (pda, am_account) = create_initialized_access_manager(admin);
        let sysvar_account = create_instructions_sysvar_account();

        let instruction = build_instruction(
            crate::instruction::ProposeUpgradeAuthorityTransfer {
                target_program,
                new_authority: upgrade_authority_pda,
            },
            build_admin_account_metas(pda, admin),
        );

        let accounts = vec![
            (pda, am_account),
            (admin, create_signer_account()),
            (solana_sdk::sysvar::instructions::ID, sysvar_account),
        ];

        let mollusk = setup_mollusk();
        mollusk.process_and_validate_instruction(
            &instruction,
            &accounts,
            &[Check::err(solana_sdk::program_error::ProgramError::Custom(
                ANCHOR_ERROR_OFFSET + AccessManagerError::SelfTransfer as u32,
            ))],
        );
    }

    #[test]
    fn test_propose_already_pending() {
        let admin = Pubkey::new_unique();
        let target_program = Pubkey::new_unique();
        let existing_authority = Pubkey::new_unique();
        let new_authority = Pubkey::new_unique();

        let (pda, am_account) =
            create_access_manager_with_pending(admin, target_program, existing_authority);
        let sysvar_account = create_instructions_sysvar_account();

        let instruction = build_instruction(
            crate::instruction::ProposeUpgradeAuthorityTransfer {
                target_program,
                new_authority,
            },
            build_admin_account_metas(pda, admin),
        );

        let accounts = vec![
            (pda, am_account),
            (admin, create_signer_account()),
            (solana_sdk::sysvar::instructions::ID, sysvar_account),
        ];

        let mollusk = setup_mollusk();
        mollusk.process_and_validate_instruction(
            &instruction,
            &accounts,
            &[Check::err(solana_sdk::program_error::ProgramError::Custom(
                ANCHOR_ERROR_OFFSET + AccessManagerError::PendingTransferAlreadyExists as u32,
            ))],
        );
    }

    #[test]
    fn test_propose_different_targets_succeeds() {
        let admin = Pubkey::new_unique();
        let target_a = Pubkey::new_unique();
        let target_b = Pubkey::new_unique();
        let authority_a = Pubkey::new_unique();
        let authority_b = Pubkey::new_unique();

        let (pda, am_account) = create_access_manager_with_pending(admin, target_a, authority_a);
        let sysvar_account = create_instructions_sysvar_account();

        let instruction = build_instruction(
            crate::instruction::ProposeUpgradeAuthorityTransfer {
                target_program: target_b,
                new_authority: authority_b,
            },
            build_admin_account_metas(pda, admin),
        );

        let accounts = vec![
            (pda, am_account),
            (admin, create_signer_account()),
            (solana_sdk::sysvar::instructions::ID, sysvar_account),
        ];

        let mollusk = setup_mollusk();
        let result =
            mollusk.process_and_validate_instruction(&instruction, &accounts, &[Check::success()]);

        let am = get_access_manager_from_result(&result, &pda);
        assert_eq!(am.pending_authority_transfers.len(), 2);
        assert!(am
            .pending_authority_transfers
            .contains(&PendingAuthorityTransfer {
                target_program: target_a,
                new_authority: authority_a,
            }));
        assert!(am
            .pending_authority_transfers
            .contains(&PendingAuthorityTransfer {
                target_program: target_b,
                new_authority: authority_b,
            }));
    }

    #[test]
    fn test_propose_vec_overflow_rejected() {
        let admin = Pubkey::new_unique();

        let full_transfers: Vec<PendingAuthorityTransfer> = (0
            ..AccessManager::MAX_PENDING_TRANSFERS)
            .map(|_| PendingAuthorityTransfer {
                target_program: Pubkey::new_unique(),
                new_authority: Pubkey::new_unique(),
            })
            .collect();

        let (pda, am_account) = create_access_manager_with_transfers(admin, full_transfers);
        let sysvar_account = create_instructions_sysvar_account();

        let instruction = build_instruction(
            crate::instruction::ProposeUpgradeAuthorityTransfer {
                target_program: Pubkey::new_unique(),
                new_authority: Pubkey::new_unique(),
            },
            build_admin_account_metas(pda, admin),
        );

        let accounts = vec![
            (pda, am_account),
            (admin, create_signer_account()),
            (solana_sdk::sysvar::instructions::ID, sysvar_account),
        ];

        let mollusk = setup_mollusk();
        mollusk.process_and_validate_instruction(
            &instruction,
            &accounts,
            &[Check::err(solana_sdk::program_error::ProgramError::Custom(
                ANCHOR_ERROR_OFFSET + AccessManagerError::TooManyPendingTransfers as u32,
            ))],
        );
    }

    #[test]
    fn test_propose_fake_sysvar_wormhole_attack() {
        let admin = Pubkey::new_unique();
        let target_program = Pubkey::new_unique();
        let new_authority = Pubkey::new_unique();

        let (pda, am_account) = create_initialized_access_manager(admin);

        let instruction = build_instruction(
            crate::instruction::ProposeUpgradeAuthorityTransfer {
                target_program,
                new_authority,
            },
            build_admin_account_metas(pda, admin),
        );

        let (instruction, fake_sysvar_account) = setup_fake_sysvar_attack(instruction, crate::ID);

        let accounts = vec![
            (pda, am_account),
            (admin, create_signer_account()),
            fake_sysvar_account,
        ];

        let mollusk = setup_mollusk();
        mollusk.process_and_validate_instruction(
            &instruction,
            &accounts,
            &[expect_sysvar_attack_error()],
        );
    }

    // ── Accept tests ─────────────────────────────────────────────────────────

    #[test]
    #[ignore = "Requires full integration test setup with BPF Loader"]
    fn test_accept_success() {
        let admin = Pubkey::new_unique();
        let target_program = Pubkey::new_unique();
        let new_authority = Pubkey::new_unique();
        let (upgrade_authority_pda, _) =
            AccessManager::upgrade_authority_pda(&target_program, &crate::ID);
        let program_data_address = derive_program_data(&target_program);

        let (pda, am_account) =
            create_access_manager_with_pending(admin, target_program, new_authority);

        let instruction = build_instruction(
            crate::instruction::AcceptUpgradeAuthorityTransfer { target_program },
            build_accept_account_metas(
                pda,
                program_data_address,
                upgrade_authority_pda,
                new_authority,
            ),
        );

        let mut accounts = vec![(pda, am_account)];
        accounts.extend(create_accept_accounts(
            program_data_address,
            upgrade_authority_pda,
            new_authority,
        ));

        let mollusk = setup_mollusk();
        mollusk.process_and_validate_instruction(&instruction, &accounts, &[Check::success()]);
    }

    #[test]
    fn test_accept_wrong_signer() {
        let admin = Pubkey::new_unique();
        let target_program = Pubkey::new_unique();
        let pending_authority = Pubkey::new_unique();
        let wrong_signer = Pubkey::new_unique();
        let (upgrade_authority_pda, _) =
            AccessManager::upgrade_authority_pda(&target_program, &crate::ID);
        let program_data_address = derive_program_data(&target_program);

        let (pda, am_account) =
            create_access_manager_with_pending(admin, target_program, pending_authority);

        let instruction = build_instruction(
            crate::instruction::AcceptUpgradeAuthorityTransfer { target_program },
            build_accept_account_metas(
                pda,
                program_data_address,
                upgrade_authority_pda,
                wrong_signer,
            ),
        );

        let mut accounts = vec![(pda, am_account)];
        accounts.extend(create_accept_accounts(
            program_data_address,
            upgrade_authority_pda,
            wrong_signer,
        ));

        let mollusk = setup_mollusk();
        mollusk.process_and_validate_instruction(
            &instruction,
            &accounts,
            &[Check::err(solana_sdk::program_error::ProgramError::Custom(
                ANCHOR_ERROR_OFFSET + AccessManagerError::AuthorityMismatch as u32,
            ))],
        );
    }

    #[test]
    fn test_accept_no_pending() {
        let admin = Pubkey::new_unique();
        let target_program = Pubkey::new_unique();
        let new_authority = Pubkey::new_unique();
        let (upgrade_authority_pda, _) =
            AccessManager::upgrade_authority_pda(&target_program, &crate::ID);
        let program_data_address = derive_program_data(&target_program);

        let (pda, am_account) = create_initialized_access_manager(admin);

        let instruction = build_instruction(
            crate::instruction::AcceptUpgradeAuthorityTransfer { target_program },
            build_accept_account_metas(
                pda,
                program_data_address,
                upgrade_authority_pda,
                new_authority,
            ),
        );

        let mut accounts = vec![(pda, am_account)];
        accounts.extend(create_accept_accounts(
            program_data_address,
            upgrade_authority_pda,
            new_authority,
        ));

        let mollusk = setup_mollusk();
        mollusk.process_and_validate_instruction(
            &instruction,
            &accounts,
            &[Check::err(solana_sdk::program_error::ProgramError::Custom(
                ANCHOR_ERROR_OFFSET + AccessManagerError::NoPendingTransfer as u32,
            ))],
        );
    }

    #[test]
    fn test_accept_wrong_target_program() {
        let admin = Pubkey::new_unique();
        let pending_target = Pubkey::new_unique();
        let wrong_target = Pubkey::new_unique();
        let new_authority = Pubkey::new_unique();
        let (upgrade_authority_pda, _) =
            AccessManager::upgrade_authority_pda(&wrong_target, &crate::ID);
        let program_data_address = derive_program_data(&wrong_target);

        let (pda, am_account) =
            create_access_manager_with_pending(admin, pending_target, new_authority);

        let instruction = build_instruction(
            crate::instruction::AcceptUpgradeAuthorityTransfer {
                target_program: wrong_target,
            },
            build_accept_account_metas(
                pda,
                program_data_address,
                upgrade_authority_pda,
                new_authority,
            ),
        );

        let mut accounts = vec![(pda, am_account)];
        accounts.extend(create_accept_accounts(
            program_data_address,
            upgrade_authority_pda,
            new_authority,
        ));

        let mollusk = setup_mollusk();
        mollusk.process_and_validate_instruction(
            &instruction,
            &accounts,
            &[Check::err(solana_sdk::program_error::ProgramError::Custom(
                ANCHOR_ERROR_OFFSET + AccessManagerError::NoPendingTransfer as u32,
            ))],
        );
    }

    // ── Cancel tests ─────────────────────────────────────────────────────────

    #[test]
    fn test_cancel_success() {
        let admin = Pubkey::new_unique();
        let target_program = Pubkey::new_unique();
        let new_authority = Pubkey::new_unique();

        let (pda, am_account) =
            create_access_manager_with_pending(admin, target_program, new_authority);
        let sysvar_account = create_instructions_sysvar_account();

        let instruction = build_instruction(
            crate::instruction::CancelUpgradeAuthorityTransfer { target_program },
            build_admin_account_metas(pda, admin),
        );

        let accounts = vec![
            (pda, am_account),
            (admin, create_signer_account()),
            (solana_sdk::sysvar::instructions::ID, sysvar_account),
        ];

        let mollusk = setup_mollusk();
        let result =
            mollusk.process_and_validate_instruction(&instruction, &accounts, &[Check::success()]);

        let am = get_access_manager_from_result(&result, &pda);
        assert!(am.pending_authority_transfers.is_empty());
    }

    #[test]
    fn test_cancel_preserves_other_entries() {
        let admin = Pubkey::new_unique();
        let target_a = Pubkey::new_unique();
        let target_b = Pubkey::new_unique();
        let authority_a = Pubkey::new_unique();
        let authority_b = Pubkey::new_unique();

        let (pda, am_account) = create_access_manager_with_transfers(
            admin,
            vec![
                PendingAuthorityTransfer {
                    target_program: target_a,
                    new_authority: authority_a,
                },
                PendingAuthorityTransfer {
                    target_program: target_b,
                    new_authority: authority_b,
                },
            ],
        );
        let sysvar_account = create_instructions_sysvar_account();

        let instruction = build_instruction(
            crate::instruction::CancelUpgradeAuthorityTransfer {
                target_program: target_a,
            },
            build_admin_account_metas(pda, admin),
        );

        let accounts = vec![
            (pda, am_account),
            (admin, create_signer_account()),
            (solana_sdk::sysvar::instructions::ID, sysvar_account),
        ];

        let mollusk = setup_mollusk();
        let result =
            mollusk.process_and_validate_instruction(&instruction, &accounts, &[Check::success()]);

        let am = get_access_manager_from_result(&result, &pda);
        assert_eq!(am.pending_authority_transfers.len(), 1);
        assert!(am
            .pending_authority_transfers
            .contains(&PendingAuthorityTransfer {
                target_program: target_b,
                new_authority: authority_b,
            }));
    }

    #[test]
    fn test_cancel_not_admin() {
        let admin = Pubkey::new_unique();
        let non_admin = Pubkey::new_unique();
        let target_program = Pubkey::new_unique();
        let new_authority = Pubkey::new_unique();

        let (pda, am_account) =
            create_access_manager_with_pending(admin, target_program, new_authority);
        let sysvar_account = create_instructions_sysvar_account();

        let instruction = build_instruction(
            crate::instruction::CancelUpgradeAuthorityTransfer { target_program },
            build_admin_account_metas(pda, non_admin),
        );

        let accounts = vec![
            (pda, am_account),
            (non_admin, create_signer_account()),
            (solana_sdk::sysvar::instructions::ID, sysvar_account),
        ];

        let mollusk = setup_mollusk();
        mollusk.process_and_validate_instruction(
            &instruction,
            &accounts,
            &[Check::err(solana_sdk::program_error::ProgramError::Custom(
                ANCHOR_ERROR_OFFSET + AccessManagerError::Unauthorized as u32,
            ))],
        );
    }

    #[test]
    fn test_cancel_no_pending() {
        let admin = Pubkey::new_unique();
        let target_program = Pubkey::new_unique();

        let (pda, am_account) = create_initialized_access_manager(admin);
        let sysvar_account = create_instructions_sysvar_account();

        let instruction = build_instruction(
            crate::instruction::CancelUpgradeAuthorityTransfer { target_program },
            build_admin_account_metas(pda, admin),
        );

        let accounts = vec![
            (pda, am_account),
            (admin, create_signer_account()),
            (solana_sdk::sysvar::instructions::ID, sysvar_account),
        ];

        let mollusk = setup_mollusk();
        mollusk.process_and_validate_instruction(
            &instruction,
            &accounts,
            &[Check::err(solana_sdk::program_error::ProgramError::Custom(
                ANCHOR_ERROR_OFFSET + AccessManagerError::NoPendingTransfer as u32,
            ))],
        );
    }

    #[test]
    fn test_cancel_wrong_target_program() {
        let admin = Pubkey::new_unique();
        let pending_target = Pubkey::new_unique();
        let wrong_target = Pubkey::new_unique();
        let new_authority = Pubkey::new_unique();

        let (pda, am_account) =
            create_access_manager_with_pending(admin, pending_target, new_authority);
        let sysvar_account = create_instructions_sysvar_account();

        let instruction = build_instruction(
            crate::instruction::CancelUpgradeAuthorityTransfer {
                target_program: wrong_target,
            },
            build_admin_account_metas(pda, admin),
        );

        let accounts = vec![
            (pda, am_account),
            (admin, create_signer_account()),
            (solana_sdk::sysvar::instructions::ID, sysvar_account),
        ];

        let mollusk = setup_mollusk();
        mollusk.process_and_validate_instruction(
            &instruction,
            &accounts,
            &[Check::err(solana_sdk::program_error::ProgramError::Custom(
                ANCHOR_ERROR_OFFSET + AccessManagerError::NoPendingTransfer as u32,
            ))],
        );
    }

    #[test]
    fn test_cancel_cpi_rejection() {
        let admin = Pubkey::new_unique();
        let target_program = Pubkey::new_unique();
        let new_authority = Pubkey::new_unique();

        let (pda, am_account) =
            create_access_manager_with_pending(admin, target_program, new_authority);

        let instruction = build_instruction(
            crate::instruction::CancelUpgradeAuthorityTransfer { target_program },
            build_admin_account_metas(pda, admin),
        );

        let malicious_program = Pubkey::new_unique();
        let (instruction, cpi_sysvar_account) = setup_cpi_call_test(instruction, malicious_program);

        let accounts = vec![
            (pda, am_account),
            (admin, create_signer_account()),
            cpi_sysvar_account,
        ];

        let mollusk = setup_mollusk();
        mollusk.process_and_validate_instruction(
            &instruction,
            &accounts,
            &[expect_access_manager_cpi_rejection_error()],
        );
    }
}

#[cfg(test)]
mod integration_tests {
    use crate::test_utils::*;
    use solana_sdk::{
        instruction::Instruction, pubkey::Pubkey, signature::Keypair, signer::Signer,
    };

    struct TransferTestContext {
        admin: Keypair,
        target_program: Pubkey,
        banks_client: solana_program_test::BanksClient,
        payer: Keypair,
        recent_blockhash: solana_sdk::hash::Hash,
    }

    impl TransferTestContext {
        async fn execute(
            &self,
            ixs: &[Instruction],
            signers: &[&Keypair],
        ) -> Result<(), solana_program_test::BanksClientError> {
            let mut all_signers = vec![&self.payer];
            all_signers.extend(signers);
            let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
                ixs,
                Some(&self.payer.pubkey()),
                &all_signers,
                self.recent_blockhash,
            );
            self.banks_client.process_transaction(tx).await
        }

        /// Sends a propose transaction signed by admin and asserts success.
        async fn propose(&self, new_authority: Pubkey) {
            let ix = build_propose_ix(self.admin.pubkey(), self.target_program, new_authority);
            self.execute(&[ix], &[&self.admin])
                .await
                .expect("propose should succeed");
        }
    }

    async fn setup_transfer_test() -> TransferTestContext {
        let admin = Keypair::new();
        let mut pt = setup_program_test_with_whitelist(&admin.pubkey(), &[TEST_CPI_TARGET_ID]);
        let target_program = Pubkey::new_unique();
        add_target_program_accounts(&mut pt, &target_program);
        let (banks_client, payer, recent_blockhash) = pt.start().await;
        TransferTestContext {
            admin,
            target_program,
            banks_client,
            payer,
            recent_blockhash,
        }
    }

    #[tokio::test]
    async fn test_propose_and_accept_succeeds() {
        let ctx = setup_transfer_test().await;
        let new_authority = Keypair::new();

        ctx.propose(new_authority.pubkey()).await;

        let pending = get_pending_transfers(&ctx.banks_client).await;
        assert!(pending.contains(&crate::types::PendingAuthorityTransfer {
            target_program: ctx.target_program,
            new_authority: new_authority.pubkey(),
        }));

        // Accept
        let accept_ix = build_accept_ix(ctx.target_program, new_authority.pubkey());
        ctx.execute(&[accept_ix], &[&new_authority])
            .await
            .expect("accept should succeed");

        let authority = get_program_data_authority(&ctx.banks_client, ctx.target_program).await;
        assert_eq!(
            authority,
            Some(new_authority.pubkey()),
            "upgrade authority should be transferred"
        );

        let pending = get_pending_transfers(&ctx.banks_client).await;
        assert!(pending.is_empty(), "pending transfers should be cleared");
    }

    #[tokio::test]
    async fn test_propose_cancel_and_repropose() {
        let ctx = setup_transfer_test().await;
        let first_authority = Keypair::new();
        let second_authority = Keypair::new();

        ctx.propose(first_authority.pubkey()).await;

        // Cancel
        let ix = build_cancel_ix(ctx.admin.pubkey(), ctx.target_program);
        ctx.execute(&[ix], &[&ctx.admin])
            .await
            .expect("cancel should succeed");

        let pending = get_pending_transfers(&ctx.banks_client).await;
        assert!(pending.is_empty(), "pending should be cleared after cancel");

        // Re-propose with second authority
        ctx.propose(second_authority.pubkey()).await;

        // Accept with second authority
        let ix = build_accept_ix(ctx.target_program, second_authority.pubkey());
        ctx.execute(&[ix], &[&second_authority])
            .await
            .expect("accept should succeed");

        let authority = get_program_data_authority(&ctx.banks_client, ctx.target_program).await;
        assert_eq!(authority, Some(second_authority.pubkey()));
    }

    #[tokio::test]
    async fn test_propose_non_admin_rejected() {
        let ctx = setup_transfer_test().await;
        let non_admin = Keypair::new();

        let ix = build_propose_ix(non_admin.pubkey(), ctx.target_program, Pubkey::new_unique());
        let err = ctx.execute(&[ix], &[&non_admin]).await.unwrap_err();
        assert_eq!(
            extract_custom_error(&err),
            Some(ANCHOR_ERROR_OFFSET + crate::AccessManagerError::Unauthorized as u32),
        );
    }

    #[tokio::test]
    async fn test_accept_wrong_signer_rejected() {
        let ctx = setup_transfer_test().await;
        let wrong_signer = Keypair::new();

        ctx.propose(Pubkey::new_unique()).await;

        let ix = build_accept_ix(ctx.target_program, wrong_signer.pubkey());
        let err = ctx.execute(&[ix], &[&wrong_signer]).await.unwrap_err();
        assert_eq!(
            extract_custom_error(&err),
            Some(ANCHOR_ERROR_OFFSET + crate::AccessManagerError::AuthorityMismatch as u32),
        );
    }

    #[tokio::test]
    async fn test_propose_whitelisted_cpi_succeeds() {
        let ctx = setup_transfer_test().await;
        let new_authority = Keypair::new();

        let inner_ix = build_propose_ix(
            ctx.admin.pubkey(),
            ctx.target_program,
            new_authority.pubkey(),
        );
        let wrapped_ix = wrap_in_test_cpi_target_proxy(ctx.admin.pubkey(), &inner_ix);

        ctx.execute(&[wrapped_ix], &[&ctx.admin])
            .await
            .expect("whitelisted CPI propose should succeed");

        let pending = get_pending_transfers(&ctx.banks_client).await;
        assert!(pending.contains(&crate::types::PendingAuthorityTransfer {
            target_program: ctx.target_program,
            new_authority: new_authority.pubkey(),
        }));
    }

    #[tokio::test]
    async fn test_propose_unauthorized_cpi_rejected() {
        let ctx = setup_transfer_test().await;

        let inner_ix =
            build_propose_ix(ctx.admin.pubkey(), ctx.target_program, Pubkey::new_unique());
        let wrapped_ix = wrap_in_test_cpi_proxy(ctx.admin.pubkey(), &inner_ix);

        let err = ctx.execute(&[wrapped_ix], &[&ctx.admin]).await.unwrap_err();
        assert_eq!(
            extract_custom_error(&err),
            Some(ANCHOR_ERROR_OFFSET + crate::AccessManagerError::CpiNotAllowed as u32),
        );
    }

    /// Batch-proposes transfers for multiple target programs in a single
    /// transaction, then accepts each independently — validates the core
    /// benefit of `Vec<PendingAuthorityTransfer>` over `Option`.
    #[tokio::test]
    async fn test_batch_propose_then_accept_each() {
        let admin = Keypair::new();
        let new_authority = Keypair::new();

        let target_programs: Vec<Pubkey> = (0..3).map(|_| Pubkey::new_unique()).collect();

        let mut pt = setup_program_test_with_whitelist(&admin.pubkey(), &[TEST_CPI_TARGET_ID]);

        for target in &target_programs {
            add_target_program_accounts(&mut pt, target);
        }

        let (banks_client, payer, recent_blockhash) = pt.start().await;

        // Batch-propose all transfers in a single transaction
        let propose_ixs: Vec<Instruction> = target_programs
            .iter()
            .map(|target| build_propose_ix(admin.pubkey(), *target, new_authority.pubkey()))
            .collect();

        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &propose_ixs,
            Some(&payer.pubkey()),
            &[&payer, &admin],
            recent_blockhash,
        );
        banks_client
            .process_transaction(tx)
            .await
            .expect("batch propose should succeed");

        let pending = get_pending_transfers(&banks_client).await;
        assert_eq!(pending.len(), target_programs.len());
        for target in &target_programs {
            assert!(pending.iter().any(|p| p.target_program == *target));
        }

        // Accept each transfer independently
        for target in &target_programs {
            let accept_ix = build_accept_ix(*target, new_authority.pubkey());
            let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
                &[accept_ix],
                Some(&payer.pubkey()),
                &[&payer, &new_authority],
                recent_blockhash,
            );
            banks_client
                .process_transaction(tx)
                .await
                .unwrap_or_else(|e| panic!("accept for {target} should succeed: {e}"));

            let authority = get_program_data_authority(&banks_client, *target).await;
            assert_eq!(authority, Some(new_authority.pubkey()));
        }

        let pending = get_pending_transfers(&banks_client).await;
        assert!(
            pending.is_empty(),
            "all pending transfers should be cleared"
        );
    }

    #[tokio::test]
    async fn test_cancel_unauthorized_cpi_rejected() {
        let ctx = setup_transfer_test().await;
        let new_authority = Keypair::new();

        ctx.propose(new_authority.pubkey()).await;

        let inner_ix = build_cancel_ix(ctx.admin.pubkey(), ctx.target_program);
        let wrapped_ix = wrap_in_test_cpi_proxy(ctx.admin.pubkey(), &inner_ix);

        let err = ctx.execute(&[wrapped_ix], &[&ctx.admin]).await.unwrap_err();
        assert_eq!(
            extract_custom_error(&err),
            Some(ANCHOR_ERROR_OFFSET + crate::AccessManagerError::CpiNotAllowed as u32),
        );
    }

    /// Propose A and B, cancel A, then accept B — verifies `swap_remove`
    /// doesn't corrupt remaining entries when operations are interleaved.
    #[tokio::test]
    async fn test_cancel_one_then_accept_other() {
        let admin = Keypair::new();
        let authority_a = Keypair::new();
        let authority_b = Keypair::new();

        let mut pt = setup_program_test_with_whitelist(&admin.pubkey(), &[TEST_CPI_TARGET_ID]);

        let targets: Vec<Pubkey> = (0..2).map(|_| Pubkey::new_unique()).collect();
        for target in &targets {
            add_target_program_accounts(&mut pt, target);
        }

        let (banks_client, payer, recent_blockhash) = pt.start().await;

        // Propose both
        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[
                build_propose_ix(admin.pubkey(), targets[0], authority_a.pubkey()),
                build_propose_ix(admin.pubkey(), targets[1], authority_b.pubkey()),
            ],
            Some(&payer.pubkey()),
            &[&payer, &admin],
            recent_blockhash,
        );
        banks_client
            .process_transaction(tx)
            .await
            .expect("batch propose should succeed");

        // Cancel A
        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[build_cancel_ix(admin.pubkey(), targets[0])],
            Some(&payer.pubkey()),
            &[&payer, &admin],
            recent_blockhash,
        );
        banks_client
            .process_transaction(tx)
            .await
            .expect("cancel A should succeed");

        let pending = get_pending_transfers(&banks_client).await;
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].target_program, targets[1]);

        // Accept B
        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[build_accept_ix(targets[1], authority_b.pubkey())],
            Some(&payer.pubkey()),
            &[&payer, &authority_b],
            recent_blockhash,
        );
        banks_client
            .process_transaction(tx)
            .await
            .expect("accept B should succeed");

        let authority = get_program_data_authority(&banks_client, targets[1]).await;
        assert_eq!(authority, Some(authority_b.pubkey()));

        let pending = get_pending_transfers(&banks_client).await;
        assert!(pending.is_empty());
    }

    #[tokio::test]
    async fn test_cancel_whitelisted_cpi_succeeds() {
        let ctx = setup_transfer_test().await;
        let new_authority = Keypair::new();

        ctx.propose(new_authority.pubkey()).await;

        let pending = get_pending_transfers(&ctx.banks_client).await;
        assert_eq!(pending.len(), 1);

        // Cancel via whitelisted CPI proxy
        let inner_ix = build_cancel_ix(ctx.admin.pubkey(), ctx.target_program);
        let wrapped_ix = wrap_in_test_cpi_target_proxy(ctx.admin.pubkey(), &inner_ix);

        ctx.execute(&[wrapped_ix], &[&ctx.admin])
            .await
            .expect("whitelisted CPI cancel should succeed");

        let pending = get_pending_transfers(&ctx.banks_client).await;
        assert!(pending.is_empty(), "pending transfers should be cleared");
    }
}
