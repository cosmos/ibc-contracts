// SPDX-License-Identifier: Apache-2.0

use crate::errors::AccessManagerError;
use crate::events::ProgramUpgradedEvent;
use crate::helpers::require_admin;
use crate::state::AccessManager;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::bpf_loader_upgradeable;

/// Upgrades a target program's bytecode using the BPF Loader Upgradeable. Requires admin authorization.
#[derive(Accounts)]
#[instruction(target_program: Pubkey)]
pub struct UpgradeProgram<'info> {
    /// The access manager PDA for admin authorization.
    #[account(
        seeds = [AccessManager::SEED],
        bump
    )]
    pub access_manager: Account<'info, AccessManager>,

    /// The target program to upgrade.
    /// CHECK: Must be an upgradeable program matching `target_program`.
    /// Writable because BPF Loader Upgradeable requires it during upgrade.
    #[account(
        mut,
        executable,
        owner = bpf_loader_upgradeable::ID,
        constraint = program.key() == target_program @ AccessManagerError::ProgramMismatch
    )]
    pub program: AccountInfo<'info>,

    /// The target program's data account (BPF Loader Upgradeable PDA).
    /// CHECK: Validated via BPF Loader seeds derivation from program account
    #[account(
        mut,
        seeds = [program.key().as_ref()],
        bump,
        seeds::program = bpf_loader_upgradeable::ID
    )]
    pub program_data: AccountInfo<'info>,

    /// Buffer account containing the new program bytecode.
    /// CHECK: Must be a BPF Loader buffer containing the new bytecode
    #[account(
        mut,
        owner = bpf_loader_upgradeable::ID
    )]
    pub buffer: AccountInfo<'info>,

    /// `AccessManager`'s PDA that acts as the upgrade authority for the target program.
    /// CHECK: Validated via seeds constraint
    #[account(
        mut,
        seeds = [AccessManager::UPGRADE_AUTHORITY_SEED, target_program.as_ref()],
        bump
    )]
    pub upgrade_authority: AccountInfo<'info>,

    /// Account to receive refunded rent from the old buffer.
    /// CHECK: Can be any account to receive refunded rent
    #[account(mut)]
    pub spill: AccountInfo<'info>,

    /// The admin signer authorizing the upgrade.
    pub authority: Signer<'info>,

    /// CHECK: Address constraint verifies this is the instructions sysvar
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instructions_sysvar: AccountInfo<'info>,

    /// CHECK: Must be BPF Loader Upgradeable program ID
    #[account(address = bpf_loader_upgradeable::ID)]
    pub bpf_loader_upgradeable: AccountInfo<'info>,

    /// Rent sysvar required by BPF Loader Upgradeable.
    pub rent: Sysvar<'info, Rent>,

    /// Required by BPF Loader Upgradeable's upgrade instruction
    pub clock: Sysvar<'info, Clock>,
}

pub fn upgrade_program(ctx: Context<UpgradeProgram>, target_program: Pubkey) -> Result<()> {
    require_admin(
        &ctx.accounts.access_manager.to_account_info(),
        &ctx.accounts.authority.to_account_info(),
        &ctx.accounts.instructions_sysvar,
        &crate::ID,
    )?;

    let upgrade_authority_pda = ctx.accounts.upgrade_authority.key();
    let bump = ctx.bumps.upgrade_authority;

    let upgrade_ix = bpf_loader_upgradeable::upgrade(
        &ctx.accounts.program.key(),
        &ctx.accounts.buffer.key(),
        &upgrade_authority_pda,
        &ctx.accounts.spill.key(),
    );

    // Using invoke_signed because BPF Loader Upgradeable is a native Solana program
    // without Anchor CPI bindings (CpiContext requires typed Anchor accounts)
    anchor_lang::solana_program::program::invoke_signed(
        &upgrade_ix,
        &[
            ctx.accounts.program_data.to_account_info(),
            ctx.accounts.program.to_account_info(),
            ctx.accounts.buffer.to_account_info(),
            ctx.accounts.spill.to_account_info(),
            ctx.accounts.rent.to_account_info(),
            ctx.accounts.clock.to_account_info(),
            ctx.accounts.upgrade_authority.to_account_info(),
        ],
        &[&[
            AccessManager::UPGRADE_AUTHORITY_SEED,
            target_program.as_ref(),
            &[bump],
        ]],
    )?;

    emit!(ProgramUpgradedEvent {
        program: target_program,
        authority: ctx.accounts.authority.key(),
        timestamp: ctx.accounts.clock.unix_timestamp,
    });

    msg!(
        "Program {} upgraded by {}",
        target_program,
        ctx.accounts.authority.key()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;
    use mollusk_svm::result::Check;
    use solana_sdk::{account::Account, instruction::AccountMeta};

    fn derive_program_data(target_program: &Pubkey) -> Pubkey {
        Pubkey::find_program_address(&[target_program.as_ref()], &bpf_loader_upgradeable::ID).0
    }

    fn setup_upgrade_test(
        admin: Pubkey,
        target_program: Pubkey,
    ) -> (
        Pubkey,
        Account,
        Pubkey,
        Pubkey,
        Pubkey,
        Pubkey,
        Vec<AccountMeta>,
    ) {
        let (access_manager_pda, access_manager_account) = create_initialized_access_manager(admin);

        let (upgrade_authority_pda, _) =
            AccessManager::upgrade_authority_pda(&target_program, &crate::ID);

        let program_data_address = derive_program_data(&target_program);
        let buffer = Pubkey::new_unique();
        let spill = Pubkey::new_unique();

        let account_metas = build_upgrade_account_metas(
            access_manager_pda,
            target_program,
            program_data_address,
            buffer,
            upgrade_authority_pda,
            spill,
            admin,
        );

        (
            access_manager_pda,
            access_manager_account,
            upgrade_authority_pda,
            program_data_address,
            buffer,
            spill,
            account_metas,
        )
    }

    fn create_program_accounts(
        target_program: Pubkey,
        program_data_address: Pubkey,
        buffer: Pubkey,
        upgrade_authority_pda: Pubkey,
        spill: Pubkey,
        upgrader: Pubkey,
    ) -> Vec<(Pubkey, Account)> {
        vec![
            (
                target_program,
                Account {
                    lamports: 1_000_000,
                    data: vec![],
                    owner: bpf_loader_upgradeable::ID,
                    executable: true,
                    ..Default::default()
                },
            ),
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
                buffer,
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
            (
                spill,
                Account {
                    lamports: 1_000_000,
                    owner: solana_sdk::system_program::ID,
                    ..Default::default()
                },
            ),
            (upgrader, create_signer_account()),
            (
                bpf_loader_upgradeable::ID,
                Account {
                    lamports: 1_000_000,
                    executable: true,
                    owner: solana_sdk::native_loader::ID,
                    ..Default::default()
                },
            ),
            create_rent_sysvar_account(),
            create_clock_sysvar_account(),
        ]
    }

    fn build_upgrade_account_metas(
        access_manager_pda: Pubkey,
        target_program: Pubkey,
        program_data_address: Pubkey,
        buffer: Pubkey,
        upgrade_authority_pda: Pubkey,
        spill: Pubkey,
        authority: Pubkey,
    ) -> Vec<AccountMeta> {
        vec![
            AccountMeta::new_readonly(access_manager_pda, false),
            AccountMeta::new(target_program, false),
            AccountMeta::new(program_data_address, false),
            AccountMeta::new(buffer, false),
            AccountMeta::new(upgrade_authority_pda, false),
            AccountMeta::new(spill, false),
            AccountMeta::new_readonly(authority, true),
            AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
            AccountMeta::new_readonly(bpf_loader_upgradeable::ID, false),
            AccountMeta::new_readonly(solana_sdk::sysvar::rent::ID, false),
            AccountMeta::new_readonly(solana_sdk::sysvar::clock::ID, false),
        ]
    }

    #[allow(clippy::too_many_arguments)]
    fn build_upgrade_instruction_and_accounts(
        access_manager_pda: Pubkey,
        access_manager_account: Account,
        target_program: Pubkey,
        program_data_address: Pubkey,
        buffer: Pubkey,
        upgrade_authority_pda: Pubkey,
        spill: Pubkey,
        authority: Pubkey,
        sysvar_account: (Pubkey, Account),
    ) -> (solana_sdk::instruction::Instruction, Vec<(Pubkey, Account)>) {
        let account_metas = build_upgrade_account_metas(
            access_manager_pda,
            target_program,
            program_data_address,
            buffer,
            upgrade_authority_pda,
            spill,
            authority,
        );

        let instruction = build_instruction(
            crate::instruction::UpgradeProgram { target_program },
            account_metas,
        );

        let mut accounts = vec![(access_manager_pda, access_manager_account)];
        accounts.extend(create_program_accounts(
            target_program,
            program_data_address,
            buffer,
            upgrade_authority_pda,
            spill,
            authority,
        ));
        accounts.push(sysvar_account);

        (instruction, accounts)
    }

    // Note: This test cannot fully succeed in Mollusk because invoke_signed to bpf_loader_upgradeable
    // references additional accounts not available in the unit test environment.
    // The instruction logic is validated by the authorization tests.
    // Full upgrade testing should be done in integration tests.
    #[test]
    #[ignore = "Requires full integration test setup with BPF Loader"]
    fn test_upgrade_program_success() {
        let admin = Pubkey::new_unique();
        let target_program = Pubkey::new_unique();

        let (
            access_manager_pda,
            access_manager_account,
            upgrade_authority_pda,
            program_data_address,
            buffer,
            spill,
            account_metas,
        ) = setup_upgrade_test(admin, target_program);

        let instruction = build_instruction(
            crate::instruction::UpgradeProgram { target_program },
            account_metas,
        );

        let mut accounts = vec![(access_manager_pda, access_manager_account)];
        accounts.extend(create_program_accounts(
            target_program,
            program_data_address,
            buffer,
            upgrade_authority_pda,
            spill,
            admin,
        ));
        accounts.push(create_instructions_sysvar_account_with_caller(crate::ID));

        let mollusk = setup_mollusk();
        let checks = vec![Check::success()];
        mollusk.process_and_validate_instruction(&instruction, &accounts, &checks);
    }

    #[test]
    fn test_upgrade_program_not_admin() {
        let admin = Pubkey::new_unique();
        let non_admin = Pubkey::new_unique();
        let target_program = Pubkey::new_unique();

        let (access_manager_pda, access_manager_account) = create_initialized_access_manager(admin);
        let (upgrade_authority_pda, _) =
            AccessManager::upgrade_authority_pda(&target_program, &crate::ID);

        let program_data_address = derive_program_data(&target_program);
        let buffer = Pubkey::new_unique();
        let spill = Pubkey::new_unique();

        let (instruction, accounts) = build_upgrade_instruction_and_accounts(
            access_manager_pda,
            access_manager_account,
            target_program,
            program_data_address,
            buffer,
            upgrade_authority_pda,
            spill,
            non_admin,
            create_instructions_sysvar_account_with_caller(crate::ID),
        );

        let mollusk = setup_mollusk();
        let checks = vec![Check::err(solana_sdk::program_error::ProgramError::Custom(
            ANCHOR_ERROR_OFFSET + AccessManagerError::Unauthorized as u32,
        ))];

        mollusk.process_and_validate_instruction(&instruction, &accounts, &checks);
    }

    #[test]
    fn test_upgrade_program_cpi_rejection() {
        let admin = Pubkey::new_unique();
        let target_program = Pubkey::new_unique();

        let (
            access_manager_pda,
            access_manager_account,
            upgrade_authority_pda,
            program_data_address,
            buffer,
            spill,
            account_metas,
        ) = setup_upgrade_test(admin, target_program);

        let instruction = build_instruction(
            crate::instruction::UpgradeProgram { target_program },
            account_metas,
        );

        let malicious_program = Pubkey::new_unique();
        let (instruction, cpi_sysvar_account) = setup_cpi_call_test(instruction, malicious_program);

        let mut accounts = vec![(access_manager_pda, access_manager_account)];
        accounts.extend(create_program_accounts(
            target_program,
            program_data_address,
            buffer,
            upgrade_authority_pda,
            spill,
            admin,
        ));
        accounts.push(cpi_sysvar_account);

        let mollusk = setup_mollusk();
        mollusk.process_and_validate_instruction(
            &instruction,
            &accounts,
            &[expect_access_manager_cpi_rejection_error()],
        );
    }

    #[test]
    fn test_upgrade_program_fake_sysvar_wormhole_attack() {
        let admin = Pubkey::new_unique();
        let target_program = Pubkey::new_unique();

        let (
            access_manager_pda,
            access_manager_account,
            upgrade_authority_pda,
            program_data_address,
            buffer,
            spill,
            account_metas,
        ) = setup_upgrade_test(admin, target_program);

        let instruction = build_instruction(
            crate::instruction::UpgradeProgram { target_program },
            account_metas,
        );

        let (instruction, fake_sysvar_account) = setup_fake_sysvar_attack(instruction, crate::ID);

        let mut accounts = vec![(access_manager_pda, access_manager_account)];
        accounts.extend(create_program_accounts(
            target_program,
            program_data_address,
            buffer,
            upgrade_authority_pda,
            spill,
            admin,
        ));
        accounts.push(fake_sysvar_account);

        let mollusk = setup_mollusk();
        mollusk.process_and_validate_instruction(
            &instruction,
            &accounts,
            &[expect_sysvar_attack_error()],
        );
    }

    #[test]
    fn test_upgrade_program_wrong_pda() {
        let admin = Pubkey::new_unique();
        let target_program = Pubkey::new_unique();

        let (access_manager_pda, access_manager_account) = create_initialized_access_manager(admin);

        let wrong_upgrade_authority = Pubkey::new_unique();
        let program_data_address = Pubkey::new_unique();
        let buffer = Pubkey::new_unique();
        let spill = Pubkey::new_unique();

        let (instruction, accounts) = build_upgrade_instruction_and_accounts(
            access_manager_pda,
            access_manager_account,
            target_program,
            program_data_address,
            buffer,
            wrong_upgrade_authority,
            spill,
            admin,
            create_instructions_sysvar_account_with_caller(crate::ID),
        );

        let mollusk = setup_mollusk();
        let checks = vec![Check::err(solana_sdk::program_error::ProgramError::Custom(
            anchor_lang::error::ErrorCode::ConstraintSeeds as u32,
        ))];

        mollusk.process_and_validate_instruction(&instruction, &accounts, &checks);
    }
}

#[cfg(test)]
mod integration_tests {
    use crate::state::AccessManager;
    use crate::test_utils::*;
    use anchor_lang::prelude::bpf_loader_upgradeable;
    use anchor_lang::InstructionData;
    use solana_sdk::{
        account::Account,
        bpf_loader_upgradeable::UpgradeableLoaderState,
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        signature::Keypair,
        signer::Signer,
    };

    struct UpgradeTestAccounts {
        target_program: Pubkey,
        buffer: Pubkey,
    }

    fn read_program_elf() -> Vec<u8> {
        let deploy_dir =
            std::env::var("SBF_OUT_DIR").unwrap_or_else(|_| "../../target/deploy".to_string());
        std::fs::read(format!("{deploy_dir}/access_manager.so"))
            .expect("access_manager.so must be built before running integration tests")
    }

    /// Creates a `ProgramTest` with a fake upgradeable program suitable for upgrade tests.
    ///
    /// Sets up properly serialized BPF loader accounts using real ELF bytes
    /// so the BPF loader's upgrade handler accepts them:
    /// - Program account pointing to its `ProgramData` PDA
    /// - `ProgramData` with `upgrade_authority` set to `AccessManager`'s PDA + real ELF
    /// - Buffer with authority set to `AccessManager`'s PDA + real ELF
    /// - Upgrade authority PDA account
    fn setup_upgrade_program_test(
        admin: &Pubkey,
        whitelisted: &[Pubkey],
    ) -> (solana_program_test::ProgramTest, UpgradeTestAccounts) {
        let mut pt = setup_program_test_with_whitelist(admin, whitelisted);

        let elf_bytes = read_program_elf();
        let elf_len = elf_bytes.len();

        let target_program = Pubkey::new_unique();
        let (program_data_pda, _) =
            Pubkey::find_program_address(&[target_program.as_ref()], &bpf_loader_upgradeable::ID);
        let (upgrade_authority_pda, _) =
            AccessManager::upgrade_authority_pda(&target_program, &crate::ID);
        let buffer = Pubkey::new_unique();

        // Program account: UpgradeableLoaderState::Program { programdata_address }
        let mut program_account = Account::new_data_with_space(
            10_000_000_000,
            &UpgradeableLoaderState::Program {
                programdata_address: program_data_pda,
            },
            UpgradeableLoaderState::size_of_program(),
            &bpf_loader_upgradeable::ID,
        )
        .unwrap();
        program_account.executable = true;
        pt.add_account(target_program, program_account);

        // ProgramData: upgrade_authority set to our PDA + real ELF
        let pd_meta_len = UpgradeableLoaderState::size_of_programdata_metadata();
        let mut pd_account = Account::new_data_with_space(
            10_000_000_000,
            &UpgradeableLoaderState::ProgramData {
                slot: 0,
                upgrade_authority_address: Some(upgrade_authority_pda),
            },
            UpgradeableLoaderState::size_of_programdata(elf_len),
            &bpf_loader_upgradeable::ID,
        )
        .unwrap();
        pd_account.data[pd_meta_len..].copy_from_slice(&elf_bytes);
        pt.add_account(program_data_pda, pd_account);

        // Buffer: authority set to our PDA + real ELF
        let buf_meta_len = UpgradeableLoaderState::size_of_buffer_metadata();
        let mut buf_account = Account::new_data_with_space(
            10_000_000_000,
            &UpgradeableLoaderState::Buffer {
                authority_address: Some(upgrade_authority_pda),
            },
            UpgradeableLoaderState::size_of_buffer(elf_len),
            &bpf_loader_upgradeable::ID,
        )
        .unwrap();
        buf_account.data[buf_meta_len..].copy_from_slice(&elf_bytes);
        pt.add_account(buffer, buf_account);

        // Upgrade authority PDA (signs via invoke_signed)
        pt.add_account(
            upgrade_authority_pda,
            Account {
                lamports: 1_000_000,
                owner: solana_sdk::system_program::ID,
                ..Default::default()
            },
        );

        (
            pt,
            UpgradeTestAccounts {
                target_program,
                buffer,
            },
        )
    }

    fn build_upgrade_program_ix(
        authority: Pubkey,
        spill: Pubkey,
        target_program: Pubkey,
        buffer: Pubkey,
    ) -> Instruction {
        let (access_manager_pda, _) =
            Pubkey::find_program_address(&[AccessManager::SEED], &crate::ID);
        let (program_data, _) =
            Pubkey::find_program_address(&[target_program.as_ref()], &bpf_loader_upgradeable::ID);
        let (upgrade_authority_pda, _) =
            AccessManager::upgrade_authority_pda(&target_program, &crate::ID);

        Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new_readonly(access_manager_pda, false),
                AccountMeta::new(target_program, false),
                AccountMeta::new(program_data, false),
                AccountMeta::new(buffer, false),
                AccountMeta::new(upgrade_authority_pda, false),
                AccountMeta::new(spill, false),
                AccountMeta::new_readonly(authority, true),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
                AccountMeta::new_readonly(bpf_loader_upgradeable::ID, false),
                AccountMeta::new_readonly(solana_sdk::sysvar::rent::ID, false),
                AccountMeta::new_readonly(solana_sdk::sysvar::clock::ID, false),
            ],
            data: crate::instruction::UpgradeProgram { target_program }.data(),
        }
    }

    struct UpgradeTestContext {
        admin: Keypair,
        accs: UpgradeTestAccounts,
        banks_client: solana_program_test::BanksClient,
        payer: Keypair,
        recent_blockhash: solana_sdk::hash::Hash,
    }

    impl UpgradeTestContext {
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

        fn build_default_upgrade_ix(&self) -> Instruction {
            build_upgrade_program_ix(
                self.admin.pubkey(),
                self.payer.pubkey(),
                self.accs.target_program,
                self.accs.buffer,
            )
        }
    }

    async fn setup_upgrade_test() -> UpgradeTestContext {
        let admin = Keypair::new();
        let (pt, accs) = setup_upgrade_program_test(&admin.pubkey(), &[TEST_CPI_TARGET_ID]);
        let (banks_client, payer, recent_blockhash) = pt.start().await;
        UpgradeTestContext {
            admin,
            accs,
            banks_client,
            payer,
            recent_blockhash,
        }
    }

    #[tokio::test]
    async fn test_upgrade_direct_call_by_admin_succeeds() {
        let ctx = setup_upgrade_test().await;
        let ix = ctx.build_default_upgrade_ix();

        ctx.execute(&[ix], &[&ctx.admin])
            .await
            .expect("direct upgrade by admin should succeed");
    }

    #[tokio::test]
    async fn test_upgrade_whitelisted_cpi_succeeds() {
        let ctx = setup_upgrade_test().await;
        let inner_ix = ctx.build_default_upgrade_ix();
        let wrapped_ix = wrap_in_test_cpi_target_proxy(ctx.admin.pubkey(), &inner_ix);

        ctx.execute(&[wrapped_ix], &[&ctx.admin])
            .await
            .expect("whitelisted CPI upgrade should succeed");
    }

    #[tokio::test]
    async fn test_upgrade_non_admin_rejected() {
        let ctx = setup_upgrade_test().await;
        let non_admin = Keypair::new();

        let ix = build_upgrade_program_ix(
            non_admin.pubkey(),
            ctx.payer.pubkey(),
            ctx.accs.target_program,
            ctx.accs.buffer,
        );

        let err = ctx.execute(&[ix], &[&non_admin]).await.unwrap_err();
        assert_eq!(
            extract_custom_error(&err),
            Some(ANCHOR_ERROR_OFFSET + crate::AccessManagerError::Unauthorized as u32),
        );
    }

    #[tokio::test]
    async fn test_upgrade_unauthorized_cpi_rejected() {
        let ctx = setup_upgrade_test().await;
        let inner_ix = ctx.build_default_upgrade_ix();
        let wrapped_ix = wrap_in_test_cpi_proxy(ctx.admin.pubkey(), &inner_ix);

        let err = ctx.execute(&[wrapped_ix], &[&ctx.admin]).await.unwrap_err();
        assert_eq!(
            extract_custom_error(&err),
            Some(ANCHOR_ERROR_OFFSET + crate::AccessManagerError::CpiNotAllowed as u32),
        );
    }

    #[tokio::test]
    async fn test_upgrade_nested_cpi_rejected() {
        let ctx = setup_upgrade_test().await;

        // Use admin as both authority and spill to keep a single signer through the CPI chain
        let inner_ix = build_upgrade_program_ix(
            ctx.admin.pubkey(),
            ctx.admin.pubkey(),
            ctx.accs.target_program,
            ctx.accs.buffer,
        );
        let cpi_target_ix = wrap_in_test_cpi_target_proxy(ctx.admin.pubkey(), &inner_ix);
        let nested_ix = wrap_in_test_cpi_proxy(ctx.admin.pubkey(), &cpi_target_ix);

        let err = ctx.execute(&[nested_ix], &[&ctx.admin]).await.unwrap_err();
        assert_eq!(
            extract_custom_error(&err),
            Some(ANCHOR_ERROR_OFFSET + crate::AccessManagerError::CpiNotAllowed as u32),
        );
    }
}
