// SPDX-License-Identifier: Apache-2.0

use crate::errors::RouterError;
use crate::events::RouterPausedEvent;
use crate::state::RouterState;
use anchor_lang::prelude::*;

/// Pauses the ICS26 router, halting all IBC packet processing.
/// Requires `PAUSER_ROLE` and rejects CPI calls.
#[derive(Accounts)]
pub struct Pause<'info> {
    /// Mutable global router configuration PDA whose `paused` flag will be set.
    #[account(
        mut,
        seeds = [RouterState::SEED],
        bump,
        constraint = !router_state.paused @ RouterError::RouterPaused,
    )]
    pub router_state: Account<'info, RouterState>,

    /// Global access control state used for pauser role verification.
    /// CHECK: Validated via seeds constraint using the stored `access_manager` program ID
    #[account(
        seeds = [access_manager::state::AccessManager::SEED],
        bump,
        seeds::program = router_state.am_state.access_manager,
    )]
    pub access_manager: AccountInfo<'info>,

    /// Signer authorized to pause the router.
    pub pauser: Signer<'info>,

    /// Instructions sysvar used for CPI detection.
    /// CHECK: Address constraint verifies this is the instructions sysvar
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instructions_sysvar: AccountInfo<'info>,
}

pub fn pause(ctx: Context<Pause>) -> Result<()> {
    access_manager::require_role(
        &ctx.accounts.access_manager,
        solana_ibc_types::roles::PAUSER_ROLE,
        &ctx.accounts.pauser,
        &ctx.accounts.instructions_sysvar,
        &crate::ID,
    )?;

    ctx.accounts.router_state.paused = true;

    emit!(RouterPausedEvent {});

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::errors::RouterError;
    use crate::state::RouterState;
    use crate::test_utils::*;
    use access_manager::AccessManagerError;
    use mollusk_svm::result::Check;
    use rstest::rstest;
    use solana_ibc_types::roles;
    use solana_sdk::instruction::AccountMeta;
    use solana_sdk::program_error::ProgramError;
    use solana_sdk::pubkey::Pubkey;

    #[derive(Clone, Copy)]
    enum PauseTestCase {
        PauseSuccess,
        UnpauseSuccess,
        PauseUnauthorized,
        UnpauseUnauthorized,
        AlreadyPaused,
        NotPaused,
        PauseCpiRejected,
        UnpauseCpiRejected,
        PauseFakeSysvar,
        UnpauseFakeSysvar,
    }

    #[derive(Clone, Copy)]
    enum SysvarMode {
        Direct,
        Cpi,
        Fake,
    }

    struct PauseTestConfig {
        is_pause: bool,
        unauthorized_signer: bool,
        wrong_initial_state: bool,
        sysvar_mode: SysvarMode,
        expected_error: Option<ProgramError>,
    }

    impl Default for PauseTestConfig {
        fn default() -> Self {
            Self {
                is_pause: true,
                unauthorized_signer: false,
                wrong_initial_state: false,
                sysvar_mode: SysvarMode::Direct,
                expected_error: None,
            }
        }
    }

    fn custom_error(code: u32) -> ProgramError {
        ProgramError::Custom(code)
    }

    impl From<PauseTestCase> for PauseTestConfig {
        fn from(case: PauseTestCase) -> Self {
            use PauseTestCase::*;
            match case {
                PauseSuccess => Self::default(),
                UnpauseSuccess => Self {
                    is_pause: false,
                    ..Default::default()
                },
                PauseUnauthorized => Self {
                    unauthorized_signer: true,
                    expected_error: Some(custom_error(
                        ANCHOR_ERROR_OFFSET + AccessManagerError::Unauthorized as u32,
                    )),
                    ..Default::default()
                },
                UnpauseUnauthorized => Self {
                    is_pause: false,
                    unauthorized_signer: true,
                    expected_error: Some(custom_error(
                        ANCHOR_ERROR_OFFSET + AccessManagerError::Unauthorized as u32,
                    )),
                    ..Default::default()
                },
                AlreadyPaused => Self {
                    wrong_initial_state: true,
                    expected_error: Some(custom_error(
                        ANCHOR_ERROR_OFFSET + RouterError::RouterPaused as u32,
                    )),
                    ..Default::default()
                },
                NotPaused => Self {
                    is_pause: false,
                    wrong_initial_state: true,
                    expected_error: Some(custom_error(
                        ANCHOR_ERROR_OFFSET + RouterError::RouterNotPaused as u32,
                    )),
                    ..Default::default()
                },
                PauseCpiRejected => Self {
                    sysvar_mode: SysvarMode::Cpi,
                    expected_error: Some(custom_error(
                        ANCHOR_ERROR_OFFSET + AccessManagerError::CpiNotAllowed as u32,
                    )),
                    ..Default::default()
                },
                UnpauseCpiRejected => Self {
                    is_pause: false,
                    sysvar_mode: SysvarMode::Cpi,
                    expected_error: Some(custom_error(
                        ANCHOR_ERROR_OFFSET + AccessManagerError::CpiNotAllowed as u32,
                    )),
                    ..Default::default()
                },
                PauseFakeSysvar => Self {
                    sysvar_mode: SysvarMode::Fake,
                    expected_error: Some(custom_error(
                        anchor_lang::error::ErrorCode::ConstraintAddress as u32,
                    )),
                    ..Default::default()
                },
                UnpauseFakeSysvar => Self {
                    is_pause: false,
                    sysvar_mode: SysvarMode::Fake,
                    expected_error: Some(custom_error(
                        anchor_lang::error::ErrorCode::ConstraintAddress as u32,
                    )),
                    ..Default::default()
                },
            }
        }
    }

    fn build_pause_toggle_ix(
        signer: Pubkey,
        is_pause: bool,
    ) -> solana_sdk::instruction::Instruction {
        let (router_state_pda, _) = Pubkey::find_program_address(&[RouterState::SEED], &crate::ID);
        let (access_manager_pda, _) = Pubkey::find_program_address(
            &[access_manager::state::AccessManager::SEED],
            &access_manager::ID,
        );

        let accounts = vec![
            AccountMeta::new(router_state_pda, false),
            AccountMeta::new_readonly(access_manager_pda, false),
            AccountMeta::new_readonly(signer, true),
            AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
        ];

        if is_pause {
            build_instruction(crate::instruction::Pause {}, accounts)
        } else {
            build_instruction(crate::instruction::Unpause {}, accounts)
        }
    }

    fn run_pause_test(case: PauseTestCase) {
        let config = PauseTestConfig::from(case);

        let role = if config.is_pause {
            roles::PAUSER_ROLE
        } else {
            roles::UNPAUSER_ROLE
        };

        let authorized = Pubkey::new_unique();
        let signer = if config.unauthorized_signer {
            Pubkey::new_unique()
        } else {
            authorized
        };

        // Correct state: unpaused for pause, paused for unpause.
        // XOR flips when wrong_initial_state is set.
        let (router_state_pda, router_state_account) =
            if config.is_pause ^ config.wrong_initial_state {
                create_initialized_router_state()
            } else {
                create_initialized_paused_router_state()
            };

        let (access_manager_pda, access_manager_account) =
            create_access_manager_with_role(authorized, role, authorized);

        let mut instruction = build_pause_toggle_ix(signer, config.is_pause);

        let sysvar_account = match config.sysvar_mode {
            SysvarMode::Direct => create_instructions_sysvar_account_with_caller(crate::ID),
            SysvarMode::Cpi => {
                let (new_ix, account) = setup_cpi_call_test(instruction, Pubkey::new_unique());
                instruction = new_ix;
                account
            }
            SysvarMode::Fake => {
                let (new_ix, account) = setup_fake_sysvar_attack(instruction, crate::ID);
                instruction = new_ix;
                account
            }
        };

        let accounts = vec![
            (router_state_pda, router_state_account),
            (access_manager_pda, access_manager_account),
            (signer, create_signer_account()),
            sysvar_account,
        ];

        let expect_success = config.expected_error.is_none();

        let check = config
            .expected_error
            .map_or_else(Check::success, Check::err);

        let mollusk = setup_mollusk();
        let result = mollusk.process_and_validate_instruction(&instruction, &accounts, &[check]);

        if expect_success {
            let router_state = get_router_state_from_result(&result, &router_state_pda);
            assert_eq!(router_state.paused, config.is_pause);
        }
    }

    #[rstest]
    #[case::pause_success(PauseTestCase::PauseSuccess)]
    #[case::unpause_success(PauseTestCase::UnpauseSuccess)]
    #[case::pause_unauthorized(PauseTestCase::PauseUnauthorized)]
    #[case::unpause_unauthorized(PauseTestCase::UnpauseUnauthorized)]
    #[case::already_paused(PauseTestCase::AlreadyPaused)]
    #[case::not_paused(PauseTestCase::NotPaused)]
    #[case::pause_cpi_rejected(PauseTestCase::PauseCpiRejected)]
    #[case::unpause_cpi_rejected(PauseTestCase::UnpauseCpiRejected)]
    #[case::pause_fake_sysvar(PauseTestCase::PauseFakeSysvar)]
    #[case::unpause_fake_sysvar(PauseTestCase::UnpauseFakeSysvar)]
    fn test_pause_toggle(#[case] case: PauseTestCase) {
        run_pause_test(case);
    }
}
