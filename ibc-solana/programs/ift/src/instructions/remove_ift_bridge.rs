// SPDX-License-Identifier: Apache-2.0

use anchor_lang::prelude::*;
use solana_ibc_types::reject_cpi;

use crate::constants::*;
use crate::errors::IFTError;
use crate::events::IFTBridgeRemoved;
use crate::state::{IFTAppMintState, IFTAppState, IFTBridge};

#[derive(Accounts)]
#[instruction(client_id: String)]
pub struct RemoveIFTBridge<'info> {
    /// Global IFT app state (read-only, for admin check)
    #[account(
        seeds = [IFT_APP_STATE_SEED],
        bump = app_state.bump,
    )]
    pub app_state: Account<'info, IFTAppState>,

    /// Per-mint IFT app state (for mint reference)
    #[account(
        seeds = [IFT_APP_MINT_STATE_SEED, app_mint_state.mint.as_ref()],
        bump = app_mint_state.bump,
    )]
    pub app_mint_state: Account<'info, IFTAppMintState>,

    /// IFT bridge to remove (close and refund rent)
    #[account(
        mut,
        close = payer,
        seeds = [IFT_BRIDGE_SEED, app_mint_state.mint.as_ref(), client_id.as_bytes()],
        bump = ift_bridge.bump,
        constraint = ift_bridge.mint == app_mint_state.mint @ IFTError::BridgeNotFound
    )]
    pub ift_bridge: Account<'info, IFTBridge>,

    /// Admin authority
    #[account(
        constraint = admin.key() == app_state.admin @ IFTError::UnauthorizedAdmin
    )]
    pub admin: Signer<'info>,

    /// Receives rent refund from the closed bridge account
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Required by Anchor for the `close` constraint
    pub system_program: Program<'info, System>,

    /// CHECK: Address constraint verifies this is the instructions sysvar
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instructions_sysvar: AccountInfo<'info>,
}

pub fn remove_ift_bridge(ctx: Context<RemoveIFTBridge>, client_id: String) -> Result<()> {
    reject_cpi(&ctx.accounts.instructions_sysvar, &crate::ID).map_err(IFTError::from)?;

    let clock = Clock::get()?;
    emit!(IFTBridgeRemoved {
        mint: ctx.accounts.app_mint_state.mint,
        client_id,
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
        pubkey::Pubkey,
    };

    use crate::errors::IFTError;
    use crate::state::ChainOptions;
    use crate::test_utils::*;

    const TEST_CLIENT_ID: &str = "07-tendermint-0";

    #[test]
    fn test_remove_ift_bridge_success() {
        let mollusk = setup_mollusk();

        let mint = Pubkey::new_unique();
        let admin = Pubkey::new_unique();
        let payer = Pubkey::new_unique();

        let (app_state_pda, app_state_bump) = get_app_state_pda();
        let (app_mint_state_pda, app_mint_state_bump) = get_app_mint_state_pda(&mint);
        let (_, mint_authority_bump) = get_mint_authority_pda(&mint);
        let (bridge_pda, bridge_bump) = get_bridge_pda(&mint, TEST_CLIENT_ID);
        let (system_program, system_account) = create_system_program_account();
        let (sysvar_id, sysvar_account) = create_instructions_sysvar_account();

        let app_state_account = create_ift_app_state_account(app_state_bump, admin);

        let app_mint_state_account =
            create_ift_app_mint_state_account(mint, app_mint_state_bump, mint_authority_bump);

        let bridge_account = create_ift_bridge_account(
            mint,
            TEST_CLIENT_ID,
            "0x1234",
            ChainOptions::Evm,
            bridge_bump,
            true,
        );

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new_readonly(app_state_pda, false),
                AccountMeta::new_readonly(app_mint_state_pda, false),
                AccountMeta::new(bridge_pda, false),
                AccountMeta::new_readonly(admin, true),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(system_program, false),
                AccountMeta::new_readonly(sysvar_id, false),
            ],
            data: crate::instruction::RemoveIftBridge {
                client_id: TEST_CLIENT_ID.to_string(),
            }
            .data(),
        };

        let accounts = vec![
            (app_state_pda, app_state_account),
            (app_mint_state_pda, app_mint_state_account),
            (bridge_pda, bridge_account),
            (admin, create_signer_account()),
            (payer, create_signer_account()),
            (system_program, system_account),
            (sysvar_id, sysvar_account),
        ];

        let result = mollusk.process_instruction(&instruction, &accounts);
        assert!(
            !result.program_result.is_err(),
            "remove_ift_bridge should succeed: {:?}",
            result.program_result
        );

        let bridge_result = result
            .resulting_accounts
            .iter()
            .find(|(k, _)| *k == bridge_pda)
            .expect("bridge should exist")
            .1
            .clone();

        assert_eq!(
            bridge_result.lamports, 0,
            "Bridge lamports should be zero after close"
        );
    }

    #[derive(Clone, Copy)]
    enum RemoveBridgeErrorCase {
        Unauthorized,
        MintMismatch,
    }

    fn run_remove_bridge_error_test(case: RemoveBridgeErrorCase) {
        let mollusk = setup_mollusk();

        let mint = Pubkey::new_unique();
        let wrong_mint = Pubkey::new_unique();
        let admin = Pubkey::new_unique();
        let unauthorized = Pubkey::new_unique();
        let payer = Pubkey::new_unique();

        let (use_unauthorized, use_wrong_mint, expected_error) = match case {
            RemoveBridgeErrorCase::Unauthorized => (
                true,
                false,
                ANCHOR_ERROR_OFFSET + IFTError::UnauthorizedAdmin as u32,
            ),
            RemoveBridgeErrorCase::MintMismatch => (
                false,
                true,
                ANCHOR_ERROR_OFFSET + IFTError::BridgeNotFound as u32,
            ),
        };

        let (app_state_pda, app_state_bump) = get_app_state_pda();
        let (app_mint_state_pda, app_mint_state_bump) = get_app_mint_state_pda(&mint);
        let (_, mint_authority_bump) = get_mint_authority_pda(&mint);
        let (bridge_pda, bridge_bump) = get_bridge_pda(&mint, TEST_CLIENT_ID);
        let (system_program, system_account) = create_system_program_account();
        let (sysvar_id, sysvar_account) = create_instructions_sysvar_account();

        let app_state_account = create_ift_app_state_account(app_state_bump, admin);

        let app_mint_state_account =
            create_ift_app_mint_state_account(mint, app_mint_state_bump, mint_authority_bump);

        let bridge_mint = if use_wrong_mint { wrong_mint } else { mint };
        let bridge_account = create_ift_bridge_account(
            bridge_mint,
            TEST_CLIENT_ID,
            "0x1234",
            ChainOptions::Evm,
            bridge_bump,
            true,
        );

        let signer = if use_unauthorized {
            unauthorized
        } else {
            admin
        };

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new_readonly(app_state_pda, false),
                AccountMeta::new_readonly(app_mint_state_pda, false),
                AccountMeta::new(bridge_pda, false),
                AccountMeta::new_readonly(signer, true),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(system_program, false),
                AccountMeta::new_readonly(sysvar_id, false),
            ],
            data: crate::instruction::RemoveIftBridge {
                client_id: TEST_CLIENT_ID.to_string(),
            }
            .data(),
        };

        let accounts = vec![
            (app_state_pda, app_state_account),
            (app_mint_state_pda, app_mint_state_account),
            (bridge_pda, bridge_account),
            (signer, create_signer_account()),
            (payer, create_signer_account()),
            (system_program, system_account),
            (sysvar_id, sysvar_account),
        ];

        let result = mollusk.process_instruction(&instruction, &accounts);
        assert_eq!(
            result.program_result,
            Err(solana_sdk::instruction::InstructionError::Custom(
                expected_error
            ))
            .into(),
        );
    }

    #[rstest]
    #[case::unauthorized(RemoveBridgeErrorCase::Unauthorized)]
    #[case::mint_mismatch(RemoveBridgeErrorCase::MintMismatch)]
    fn test_remove_ift_bridge_validation(#[case] case: RemoveBridgeErrorCase) {
        run_remove_bridge_error_test(case);
    }

    #[test]
    fn test_remove_inactive_bridge_success() {
        let mollusk = setup_mollusk();

        let mint = Pubkey::new_unique();
        let admin = Pubkey::new_unique();
        let payer = Pubkey::new_unique();

        let (app_state_pda, app_state_bump) = get_app_state_pda();
        let (app_mint_state_pda, app_mint_state_bump) = get_app_mint_state_pda(&mint);
        let (_, mint_authority_bump) = get_mint_authority_pda(&mint);
        let (bridge_pda, bridge_bump) = get_bridge_pda(&mint, TEST_CLIENT_ID);
        let (system_program, system_account) = create_system_program_account();
        let (sysvar_id, sysvar_account) = create_instructions_sysvar_account();

        let app_state_account = create_ift_app_state_account(app_state_bump, admin);

        let app_mint_state_account =
            create_ift_app_mint_state_account(mint, app_mint_state_bump, mint_authority_bump);

        let bridge_account = create_ift_bridge_account(
            mint,
            TEST_CLIENT_ID,
            "0x1234",
            ChainOptions::Evm,
            bridge_bump,
            false, // inactive bridge
        );

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new_readonly(app_state_pda, false),
                AccountMeta::new_readonly(app_mint_state_pda, false),
                AccountMeta::new(bridge_pda, false),
                AccountMeta::new_readonly(admin, true),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(system_program, false),
                AccountMeta::new_readonly(sysvar_id, false),
            ],
            data: crate::instruction::RemoveIftBridge {
                client_id: TEST_CLIENT_ID.to_string(),
            }
            .data(),
        };

        let accounts = vec![
            (app_state_pda, app_state_account),
            (app_mint_state_pda, app_mint_state_account),
            (bridge_pda, bridge_account),
            (admin, create_signer_account()),
            (payer, create_signer_account()),
            (system_program, system_account),
            (sysvar_id, sysvar_account),
        ];

        let result = mollusk.process_instruction(&instruction, &accounts);
        assert!(
            !result.program_result.is_err(),
            "removing inactive bridge should succeed: {:?}",
            result.program_result
        );

        let bridge_result = result
            .resulting_accounts
            .iter()
            .find(|(k, _)| *k == bridge_pda)
            .expect("bridge should exist")
            .1
            .clone();

        assert_eq!(
            bridge_result.lamports, 0,
            "Bridge lamports should be zero after close"
        );
    }

    #[test]
    fn test_remove_ift_bridge_cpi_rejected() {
        let mollusk = setup_mollusk();

        let mint = Pubkey::new_unique();
        let admin = Pubkey::new_unique();
        let payer = Pubkey::new_unique();

        let (app_state_pda, app_state_bump) = get_app_state_pda();
        let (app_mint_state_pda, app_mint_state_bump) = get_app_mint_state_pda(&mint);
        let (_, mint_authority_bump) = get_mint_authority_pda(&mint);
        let (bridge_pda, bridge_bump) = get_bridge_pda(&mint, TEST_CLIENT_ID);
        let (system_program, system_account) = create_system_program_account();
        let (sysvar_id, sysvar_account) =
            create_cpi_instructions_sysvar_account(Pubkey::new_unique());

        let app_state_account = create_ift_app_state_account(app_state_bump, admin);

        let app_mint_state_account =
            create_ift_app_mint_state_account(mint, app_mint_state_bump, mint_authority_bump);

        let bridge_account = create_ift_bridge_account(
            mint,
            TEST_CLIENT_ID,
            "0x1234",
            ChainOptions::Evm,
            bridge_bump,
            true,
        );

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new_readonly(app_state_pda, false),
                AccountMeta::new_readonly(app_mint_state_pda, false),
                AccountMeta::new(bridge_pda, false),
                AccountMeta::new_readonly(admin, true),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(system_program, false),
                AccountMeta::new_readonly(sysvar_id, false),
            ],
            data: crate::instruction::RemoveIftBridge {
                client_id: TEST_CLIENT_ID.to_string(),
            }
            .data(),
        };

        let accounts = vec![
            (app_state_pda, app_state_account),
            (app_mint_state_pda, app_mint_state_account),
            (bridge_pda, bridge_account),
            (admin, create_signer_account()),
            (payer, create_signer_account()),
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
}
