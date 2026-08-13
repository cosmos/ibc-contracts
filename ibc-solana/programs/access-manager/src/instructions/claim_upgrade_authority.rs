// SPDX-License-Identifier: Apache-2.0

use crate::events::UpgradeAuthorityClaimedEvent;
use crate::helpers::{cpi, require_admin};
use crate::state::AccessManager;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::bpf_loader_upgradeable;

/// Claims upgrade authority from a source access manager that has proposed
/// a transfer to this access manager's upgrade authority PDA.
///
/// This enables AM-to-AM migration: the source AM proposes, and the
/// destination AM claims by calling (via CPI) the source's
/// `accept_upgrade_authority_transfer` with its own PDA as signer.
///
/// Requires `ADMIN_ROLE` because the CPI propagates PDA signer privileges
/// to the source program. Without admin gating, a malicious program passed
/// as `source_access_manager_program` could misuse the signed PDA to steal
/// upgrade authority from programs this access manager already controls.
#[derive(Accounts)]
#[instruction(target_program: Pubkey)]
pub struct ClaimUpgradeAuthority<'info> {
    /// This access manager's upgrade authority PDA for the target program.
    /// Signs the CPI as the new authority.
    /// CHECK: Validated via seeds constraint
    #[account(
        seeds = [AccessManager::UPGRADE_AUTHORITY_SEED, target_program.as_ref()],
        bump
    )]
    pub new_upgrade_authority: AccountInfo<'info>,

    /// The source access manager's state PDA.
    /// CHECK: Validated via seeds constraint against `source_access_manager_program`
    #[account(
        mut,
        seeds = [AccessManager::SEED],
        bump,
        seeds::program = source_access_manager_program.key()
    )]
    pub source_access_manager_state: AccountInfo<'info>,

    /// The target program's data account (BPF Loader Upgradeable PDA).
    /// CHECK: Validated via seeds constraint against BPF Loader Upgradeable
    #[account(
        mut,
        seeds = [target_program.as_ref()],
        bump,
        seeds::program = bpf_loader_upgradeable::ID
    )]
    pub target_program_data: AccountInfo<'info>,

    /// The source access manager's upgrade authority PDA for the target program.
    /// CHECK: Validated via seeds constraint against `source_access_manager_program`
    #[account(
        seeds = [AccessManager::UPGRADE_AUTHORITY_SEED, target_program.as_ref()],
        bump,
        seeds::program = source_access_manager_program.key()
    )]
    pub source_upgrade_authority: AccountInfo<'info>,

    /// The source access manager program (CPI target).
    /// CHECK: Must be executable
    #[account(executable)]
    pub source_access_manager_program: AccountInfo<'info>,

    /// CHECK: Must be BPF Loader Upgradeable program ID
    #[account(address = bpf_loader_upgradeable::ID)]
    pub bpf_loader_upgradeable: AccountInfo<'info>,

    /// Access manager state PDA used to verify the caller holds the admin role.
    #[account(seeds = [AccessManager::SEED], bump)]
    pub am_state: Account<'info, AccessManager>,

    /// Must hold `ADMIN_ROLE` on the current access manager.
    pub admin: Signer<'info>,

    /// CHECK: Address constraint verifies this is the instructions sysvar
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instructions_sysvar: AccountInfo<'info>,
}

pub fn claim_upgrade_authority(
    ctx: Context<ClaimUpgradeAuthority>,
    target_program: Pubkey,
) -> Result<()> {
    require_admin(
        &ctx.accounts.am_state.to_account_info(),
        &ctx.accounts.admin.to_account_info(),
        &ctx.accounts.instructions_sysvar,
        &crate::ID,
    )?;

    let cpi_accounts = cpi::AcceptUpgradeAuthorityTransferCpi {
        access_manager: ctx.accounts.source_access_manager_state.to_account_info(),
        program_data: ctx.accounts.target_program_data.to_account_info(),
        upgrade_authority: ctx.accounts.source_upgrade_authority.to_account_info(),
        new_authority: ctx.accounts.new_upgrade_authority.to_account_info(),
        bpf_loader_upgradeable: ctx.accounts.bpf_loader_upgradeable.to_account_info(),
    };
    let bump = [ctx.bumps.new_upgrade_authority];
    let signer_seeds = &[&[
        AccessManager::UPGRADE_AUTHORITY_SEED,
        target_program.as_ref(),
        &bump,
    ][..]];
    let cpi_ctx = CpiContext::new_with_signer(
        ctx.accounts.source_access_manager_program.to_account_info(),
        cpi_accounts,
        signer_seeds,
    );
    cpi::accept_upgrade_authority_transfer(cpi_ctx, target_program)?;

    emit!(UpgradeAuthorityClaimedEvent {
        program: target_program,
        source_access_manager: ctx.accounts.source_access_manager_program.key(),
        new_authority: ctx.accounts.new_upgrade_authority.key(),
    });

    Ok(())
}

#[cfg(test)]
mod integration_tests {
    use crate::state::AccessManager;
    use crate::test_utils::*;
    use crate::types::{PendingAuthorityTransfer, RoleData};
    use anchor_lang::prelude::bpf_loader_upgradeable;
    use anchor_lang::{AnchorSerialize, Discriminator, InstructionData, Space};
    use solana_ibc_types::roles;
    use solana_sdk::{
        account::Account,
        bpf_loader_upgradeable::UpgradeableLoaderState,
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        signature::Keypair,
        signer::Signer,
    };

    /// Set up `ProgramTest` with both AM binaries loaded.
    ///
    /// - `crate::ID` is the claimer (destination AM)
    /// - `crate::test_config::OTHER_AM_ID` is the source AM
    ///
    /// `admin` is the admin on the claimer AM (required for the admin gate).
    /// `target_program` identifies the program whose upgrade authority is being migrated.
    /// `pending_transfers` sets up the source AM's state with pending transfers.
    fn setup_claim_test(
        admin: &Pubkey,
        target_program: Pubkey,
        pending_transfers: Vec<PendingAuthorityTransfer>,
        whitelisted_programs: &[Pubkey],
    ) -> solana_program_test::ProgramTest {
        if std::env::var("SBF_OUT_DIR").is_err() {
            std::env::set_var("SBF_OUT_DIR", std::path::Path::new("../../target/deploy"));
        }

        let mut pt = solana_program_test::ProgramTest::new(
            crate::test_config::PROGRAM_BINARY_NAME,
            crate::ID,
            None,
        );
        pt.add_program(
            crate::test_config::OTHER_AM_BINARY_NAME,
            crate::test_config::OTHER_AM_ID,
            None,
        );
        if !whitelisted_programs.is_empty() {
            pt.add_program("test_cpi_proxy", TEST_CPI_PROXY_ID, None);
            pt.add_program("test_cpi_target", TEST_CPI_TARGET_ID, None);
        }

        // Our (claimer's) access manager state PDA with admin role
        let (our_am_pda, _) = Pubkey::find_program_address(&[AccessManager::SEED], &crate::ID);
        let our_am = AccessManager {
            roles: vec![RoleData {
                role_id: roles::ADMIN_ROLE,
                members: vec![*admin],
            }],
            whitelisted_programs: whitelisted_programs.to_vec(),
            pending_authority_transfers: vec![],
        };
        let mut our_am_data = vec![0u8; 8 + AccessManager::INIT_SPACE];
        our_am_data[0..8].copy_from_slice(AccessManager::DISCRIMINATOR);
        our_am.serialize(&mut &mut our_am_data[8..]).unwrap();
        pt.add_account(
            our_am_pda,
            Account {
                lamports: 10_000_000,
                data: our_am_data,
                owner: crate::ID,
                executable: false,
                rent_epoch: 0,
            },
        );

        // Source AM's upgrade authority PDA (current authority of target program)
        let (source_upgrade_authority, _) =
            AccessManager::upgrade_authority_pda(&target_program, &crate::test_config::OTHER_AM_ID);

        // Target program's program_data with source AM as authority
        let (program_data_pda, _) =
            Pubkey::find_program_address(&[target_program.as_ref()], &bpf_loader_upgradeable::ID);
        let pd_account = Account::new_data_with_space(
            10_000_000_000,
            &UpgradeableLoaderState::ProgramData {
                slot: 0,
                upgrade_authority_address: Some(source_upgrade_authority),
            },
            UpgradeableLoaderState::size_of_programdata_metadata(),
            &bpf_loader_upgradeable::ID,
        )
        .unwrap();
        pt.add_account(program_data_pda, pd_account);

        // Source AM's upgrade authority PDA account
        pt.add_account(
            source_upgrade_authority,
            Account {
                lamports: 1_000_000,
                owner: solana_sdk::system_program::ID,
                ..Default::default()
            },
        );

        // Source AM's state PDA
        let (source_am_pda, _) =
            Pubkey::find_program_address(&[AccessManager::SEED], &crate::test_config::OTHER_AM_ID);
        let source_am = AccessManager {
            roles: vec![RoleData {
                role_id: roles::ADMIN_ROLE,
                members: vec![Pubkey::new_unique()],
            }],
            whitelisted_programs: vec![],
            pending_authority_transfers: pending_transfers,
        };
        let mut am_data = vec![0u8; 8 + AccessManager::INIT_SPACE];
        am_data[0..8].copy_from_slice(AccessManager::DISCRIMINATOR);
        source_am.serialize(&mut &mut am_data[8..]).unwrap();
        pt.add_account(
            source_am_pda,
            Account {
                lamports: 10_000_000,
                data: am_data,
                owner: crate::test_config::OTHER_AM_ID,
                executable: false,
                rent_epoch: 0,
            },
        );

        // Our (claimer's) upgrade authority PDA account
        let (new_upgrade_authority, _) =
            AccessManager::upgrade_authority_pda(&target_program, &crate::ID);
        pt.add_account(
            new_upgrade_authority,
            Account {
                lamports: 1_000_000,
                owner: solana_sdk::system_program::ID,
                ..Default::default()
            },
        );

        pt
    }

    fn build_claim_ix(target_program: Pubkey, admin: Pubkey) -> Instruction {
        let (new_upgrade_authority, _) =
            AccessManager::upgrade_authority_pda(&target_program, &crate::ID);
        let (source_am_pda, _) =
            Pubkey::find_program_address(&[AccessManager::SEED], &crate::test_config::OTHER_AM_ID);
        let (program_data_pda, _) =
            Pubkey::find_program_address(&[target_program.as_ref()], &bpf_loader_upgradeable::ID);
        let (source_upgrade_authority, _) =
            AccessManager::upgrade_authority_pda(&target_program, &crate::test_config::OTHER_AM_ID);
        let (our_am_pda, _) = Pubkey::find_program_address(&[AccessManager::SEED], &crate::ID);

        Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new_readonly(new_upgrade_authority, false),
                AccountMeta::new(source_am_pda, false),
                AccountMeta::new(program_data_pda, false),
                AccountMeta::new_readonly(source_upgrade_authority, false),
                AccountMeta::new_readonly(crate::test_config::OTHER_AM_ID, false),
                AccountMeta::new_readonly(bpf_loader_upgradeable::ID, false),
                AccountMeta::new_readonly(our_am_pda, false),
                AccountMeta::new_readonly(admin, true),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
            ],
            data: crate::instruction::ClaimUpgradeAuthority { target_program }.data(),
        }
    }

    async fn get_source_pending_transfers(
        banks_client: &solana_program_test::BanksClient,
    ) -> Vec<PendingAuthorityTransfer> {
        let (pda, _) =
            Pubkey::find_program_address(&[AccessManager::SEED], &crate::test_config::OTHER_AM_ID);
        let account = banks_client.get_account(pda).await.unwrap().unwrap();
        let am: AccessManager =
            anchor_lang::AccountDeserialize::try_deserialize(&mut &account.data[..]).unwrap();
        am.pending_authority_transfers
    }

    struct ClaimTestContext {
        admin: Keypair,
        target_program: Pubkey,
        claimer_pda: Pubkey,
        banks_client: solana_program_test::BanksClient,
        payer: Keypair,
        recent_blockhash: solana_sdk::hash::Hash,
    }

    impl ClaimTestContext {
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
    }

    /// Sets up a claim test with a valid pending transfer pointing to the
    /// claimer's upgrade authority PDA.
    async fn setup_valid_claim_test(whitelisted_programs: &[Pubkey]) -> ClaimTestContext {
        let admin = Keypair::new();
        let target_program = Pubkey::new_unique();
        let (claimer_pda, _) = AccessManager::upgrade_authority_pda(&target_program, &crate::ID);

        let pt = setup_claim_test(
            &admin.pubkey(),
            target_program,
            vec![PendingAuthorityTransfer {
                target_program,
                new_authority: claimer_pda,
            }],
            whitelisted_programs,
        );
        let (banks_client, payer, recent_blockhash) = pt.start().await;

        ClaimTestContext {
            admin,
            target_program,
            claimer_pda,
            banks_client,
            payer,
            recent_blockhash,
        }
    }

    async fn assert_claim_completed(
        banks_client: &solana_program_test::BanksClient,
        target_program: Pubkey,
        expected_authority: Pubkey,
    ) {
        let authority = get_program_data_authority(banks_client, target_program).await;
        assert_eq!(
            authority,
            Some(expected_authority),
            "upgrade authority should transfer to claimer's PDA"
        );

        let pending = get_source_pending_transfers(banks_client).await;
        assert!(pending.is_empty(), "pending transfers should be cleared");
    }

    #[tokio::test]
    async fn test_claim_succeeds() {
        let ctx = setup_valid_claim_test(&[]).await;
        let ix = build_claim_ix(ctx.target_program, ctx.admin.pubkey());

        ctx.execute(&[ix], &[&ctx.admin])
            .await
            .expect("claim should succeed");

        assert_claim_completed(&ctx.banks_client, ctx.target_program, ctx.claimer_pda).await;
    }

    #[tokio::test]
    async fn test_claim_non_admin_rejected() {
        let ctx = setup_valid_claim_test(&[]).await;
        let non_admin = Keypair::new();

        let ix = build_claim_ix(ctx.target_program, non_admin.pubkey());
        let err = ctx.execute(&[ix], &[&non_admin]).await.unwrap_err();
        assert_eq!(
            extract_custom_error(&err),
            Some(ANCHOR_ERROR_OFFSET + crate::AccessManagerError::Unauthorized as u32),
        );
    }

    #[tokio::test]
    async fn test_claim_no_pending_fails() {
        let admin = Keypair::new();
        let target_program = Pubkey::new_unique();

        let pt = setup_claim_test(&admin.pubkey(), target_program, vec![], &[]);
        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let ix = build_claim_ix(target_program, admin.pubkey());
        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[ix],
            Some(&payer.pubkey()),
            &[&payer, &admin],
            recent_blockhash,
        );
        let err = banks_client.process_transaction(tx).await.unwrap_err();
        assert_eq!(
            extract_custom_error(&err),
            Some(ANCHOR_ERROR_OFFSET + crate::AccessManagerError::NoPendingTransfer as u32),
        );
    }

    #[tokio::test]
    async fn test_claim_authority_mismatch_fails() {
        let admin = Keypair::new();
        let target_program = Pubkey::new_unique();
        let wrong_authority = Pubkey::new_unique();

        let pt = setup_claim_test(
            &admin.pubkey(),
            target_program,
            vec![PendingAuthorityTransfer {
                target_program,
                new_authority: wrong_authority,
            }],
            &[],
        );
        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let ix = build_claim_ix(target_program, admin.pubkey());
        let tx = solana_sdk::transaction::Transaction::new_signed_with_payer(
            &[ix],
            Some(&payer.pubkey()),
            &[&payer, &admin],
            recent_blockhash,
        );
        let err = banks_client.process_transaction(tx).await.unwrap_err();
        assert_eq!(
            extract_custom_error(&err),
            Some(ANCHOR_ERROR_OFFSET + crate::AccessManagerError::AuthorityMismatch as u32),
        );
    }

    /// Proves `claim_upgrade_authority` works at CPI depth 4 via a whitelisted
    /// proxy — the deepest chain in a multisig scenario:
    /// multisig (1) → AM-B claim (2) → AM-A accept (3) → BPF Loader `set_authority` (4)
    #[tokio::test]
    async fn test_claim_whitelisted_cpi_succeeds() {
        let ctx = setup_valid_claim_test(&[TEST_CPI_TARGET_ID]).await;

        let inner_ix = build_claim_ix(ctx.target_program, ctx.admin.pubkey());
        let wrapped_ix = wrap_in_test_cpi_target_proxy(ctx.admin.pubkey(), &inner_ix);

        ctx.execute(&[wrapped_ix], &[&ctx.admin])
            .await
            .expect("whitelisted CPI claim should succeed");

        assert_claim_completed(&ctx.banks_client, ctx.target_program, ctx.claimer_pda).await;
    }

    #[tokio::test]
    async fn test_claim_unauthorized_cpi_rejected() {
        let ctx = setup_valid_claim_test(&[TEST_CPI_TARGET_ID]).await;

        let inner_ix = build_claim_ix(ctx.target_program, ctx.admin.pubkey());
        let wrapped_ix = wrap_in_test_cpi_proxy(ctx.admin.pubkey(), &inner_ix);

        let err = ctx.execute(&[wrapped_ix], &[&ctx.admin]).await.unwrap_err();
        assert_eq!(
            extract_custom_error(&err),
            Some(ANCHOR_ERROR_OFFSET + crate::AccessManagerError::CpiNotAllowed as u32),
        );
    }
}
