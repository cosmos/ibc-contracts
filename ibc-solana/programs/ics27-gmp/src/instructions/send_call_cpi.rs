// SPDX-License-Identifier: Apache-2.0

use crate::constants::*;
use crate::errors::GMPError;
use crate::instructions::send_call::send_call_inner;
use crate::state::{GMPAppState, SendCallMsg};
use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar::instructions as sysvar_instructions;
use ics26_router::state::{Client, IBCApp, RouterState};

/// Sends a GMP call packet via CPI from another program. Rejects direct calls
/// and nested CPI.
///
/// The calling program's ID is extracted from the instruction sysvar and used
/// as the sender. This ensures the sender matches the executable program,
/// satisfying the IFT spec requirement that `sender == executable target`.
#[derive(Accounts)]
#[instruction(msg: SendCallMsg)]
pub struct SendCallCpi<'info> {
    /// GMP program's global configuration PDA. Must not be paused.
    #[account(
        mut,
        seeds = [GMPAppState::SEED],
        bump = app_state.bump,
        constraint = !app_state.paused @ GMPError::AppPaused
    )]
    pub app_state: Account<'info, GMPAppState>,

    /// Fee payer for the packet commitment account creation.
    #[account(mut)]
    pub payer: Signer<'info>,

    /// ICS26 router program invoked via CPI to submit the packet.
    pub router_program: Program<'info, ics26_router::program::Ics26Router>,

    /// Router's global state PDA, forwarded to the router CPI.
    #[account(
        seeds = [RouterState::SEED],
        bump,
        seeds::program = router_program
    )]
    pub router_state: Box<Account<'info, RouterState>>,

    /// Stores the packet commitment hash; initialized by the router CPI.
    /// CHECK: PDA seeds verified against the router program.
    #[account(
        mut,
        seeds = [
            solana_ibc_types::Commitment::PACKET_COMMITMENT_SEED,
            msg.source_client.as_bytes(),
            &msg.sequence.to_le_bytes()
        ],
        bump,
        seeds::program = router_program
    )]
    pub packet_commitment: AccountInfo<'info>,

    /// Port-to-program mapping that authorizes this GMP program for the GMP port.
    #[account(
        seeds = [IBCApp::SEED, GMP_PORT_ID.as_bytes()],
        bump,
        seeds::program = router_program
    )]
    pub ibc_app: Box<Account<'info, IBCApp>>,

    /// IBC client account that identifies the destination chain for routing.
    #[account(
        seeds = [Client::SEED, msg.source_client.as_bytes()],
        bump,
        seeds::program = router_program
    )]
    pub client: Box<Account<'info, Client>>,

    /// Light client program used by the router to check client status.
    /// CHECK: Validated against client registry.
    #[account(address = client.client_program_id @ GMPError::InvalidLightClientProgram)]
    pub light_client_program: AccountInfo<'info>,

    /// Light client's state account, forwarded to the router for status verification.
    /// CHECK: Ownership validated against light client program.
    #[account(owner = light_client_program.key() @ GMPError::InvalidAccountOwner)]
    pub client_state: AccountInfo<'info>,

    /// Instructions sysvar used to enforce single-level CPI and extract the caller program ID.
    /// CHECK: Address constraint verifies this is the instructions sysvar
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instruction_sysvar: AccountInfo<'info>,

    /// Consensus state account, forwarded to the router for client expiry check.
    /// CHECK: Forwarded to router CPI; validated by the light client program.
    pub consensus_state: AccountInfo<'info>,

    /// Solana system program used for account allocation.
    pub system_program: Program<'info, System>,
}

pub fn send_call_cpi(ctx: Context<SendCallCpi>, msg: SendCallMsg) -> Result<u64> {
    solana_ibc_types::reject_direct_calls().map_err(GMPError::from)?;
    solana_ibc_types::reject_nested_cpi().map_err(GMPError::from)?;

    let ix_sysvar = ctx.accounts.instruction_sysvar.to_account_info();
    let sender_pubkey = sysvar_instructions::get_instruction_relative(0, &ix_sysvar)
        .map_err(|_| GMPError::UnauthorizedRouter)?
        .program_id;

    send_call_inner(
        &ctx.accounts.app_state,
        &ctx.accounts.router_program,
        &ctx.accounts.router_state,
        &ctx.accounts.packet_commitment,
        &ctx.accounts.payer,
        &ctx.accounts.ibc_app,
        &ctx.accounts.client,
        &ctx.accounts.light_client_program,
        &ctx.accounts.client_state,
        &ctx.accounts.consensus_state,
        &ctx.accounts.system_program,
        sender_pubkey,
        msg,
    )
}

#[cfg(test)]
mod tests {
    use crate::constants::GMP_PORT_ID;
    use crate::errors::GMPError;
    use crate::state::{GMPAppState, SendCallMsg};
    use crate::test_utils::*;
    use anchor_lang::InstructionData;
    use mollusk_svm::Mollusk;
    use rstest::rstest;
    use solana_sdk::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        system_program,
    };

    struct TestContext {
        mollusk: Mollusk,
        payer: Pubkey,
        router_program: Pubkey,
        router_state: Pubkey,
        packet_commitment: Pubkey,
        ibc_app: Pubkey,
        client: Pubkey,
        light_client_program: Pubkey,
        client_state: Pubkey,
        consensus_state: Pubkey,
        app_state_pda: Pubkey,
        app_state_bump: u8,
    }

    impl TestContext {
        fn new() -> Self {
            let payer = Pubkey::new_unique();
            let router_program = ics26_router::ID;
            let (router_state, _) = create_router_state_pda();
            let (packet_commitment, _) = Pubkey::find_program_address(
                &[
                    solana_ibc_types::Commitment::PACKET_COMMITMENT_SEED,
                    TEST_SOURCE_CLIENT.as_bytes(),
                    &1u64.to_le_bytes(),
                ],
                &ics26_router::ID,
            );
            let (ibc_app, _) = create_ibc_app_pda(GMP_PORT_ID);
            let light_client_program = Pubkey::new_unique();
            let (client, _) = create_client_pda(TEST_SOURCE_CLIENT, light_client_program);
            let client_state = Pubkey::new_unique();
            let consensus_state = Pubkey::new_unique();
            let (app_state_pda, app_state_bump) =
                Pubkey::find_program_address(&[GMPAppState::SEED], &crate::ID);

            Self {
                mollusk: Mollusk::new(&crate::ID, crate::get_gmp_program_path()),
                payer,
                router_program,
                router_state,
                packet_commitment,
                ibc_app,
                client,
                light_client_program,
                client_state,
                consensus_state,
                app_state_pda,
                app_state_bump,
            }
        }

        fn create_valid_msg() -> SendCallMsg {
            SendCallMsg {
                source_client: TEST_SOURCE_CLIENT.to_string(),
                sequence: 1,
                receiver: Pubkey::new_unique().to_string(),
                salt: vec![1, 2, 3],
                payload: vec![4, 5, 6],
                timeout_timestamp: 3600,
                memo: String::new(),
                encoding: crate::constants::ICS27_ENCODING_PROTOBUF.to_string(),
            }
        }

        fn build_instruction(&self, msg: SendCallMsg, instruction_sysvar: Pubkey) -> Instruction {
            let instruction_data = crate::instruction::SendCallCpi { msg };

            Instruction {
                program_id: crate::ID,
                accounts: vec![
                    AccountMeta::new(self.app_state_pda, false),
                    AccountMeta::new(self.payer, true),
                    AccountMeta::new_readonly(self.router_program, false),
                    AccountMeta::new_readonly(self.router_state, false),
                    AccountMeta::new(self.packet_commitment, false),
                    AccountMeta::new_readonly(self.ibc_app, false),
                    AccountMeta::new_readonly(self.client, false),
                    AccountMeta::new_readonly(self.light_client_program, false),
                    AccountMeta::new_readonly(self.client_state, false),
                    AccountMeta::new_readonly(instruction_sysvar, false),
                    AccountMeta::new_readonly(self.consensus_state, false),
                    AccountMeta::new_readonly(system_program::ID, false),
                ],
                data: instruction_data.data(),
            }
        }

        fn build_accounts(
            &self,
            paused: bool,
            sysvar_account: (Pubkey, solana_sdk::account::Account),
        ) -> Vec<(Pubkey, solana_sdk::account::Account)> {
            vec![
                create_gmp_app_state_account(self.app_state_pda, self.app_state_bump, paused),
                create_authority_account(self.payer),
                create_router_program_account(self.router_program),
                create_router_state_pda(),
                create_authority_account(self.packet_commitment),
                create_ibc_app_pda(GMP_PORT_ID),
                create_client_pda(TEST_SOURCE_CLIENT, self.light_client_program),
                create_authority_account(self.light_client_program),
                create_owned_account(self.client_state, self.light_client_program),
                sysvar_account,
                create_authority_account(self.consensus_state),
                create_system_program_account(),
            ]
        }

        fn build_instruction_with_wrong_pda(
            &self,
            msg: SendCallMsg,
            wrong_pda: Pubkey,
        ) -> Instruction {
            let instruction_data = crate::instruction::SendCallCpi { msg };

            Instruction {
                program_id: crate::ID,
                accounts: vec![
                    AccountMeta::new(wrong_pda, false),
                    AccountMeta::new(self.payer, true),
                    AccountMeta::new_readonly(self.router_program, false),
                    AccountMeta::new_readonly(self.router_state, false),
                    AccountMeta::new(self.packet_commitment, false),
                    AccountMeta::new_readonly(self.ibc_app, false),
                    AccountMeta::new_readonly(self.client, false),
                    AccountMeta::new_readonly(self.light_client_program, false),
                    AccountMeta::new_readonly(self.client_state, false),
                    AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
                    AccountMeta::new_readonly(self.consensus_state, false),
                    AccountMeta::new_readonly(system_program::ID, false),
                ],
                data: instruction_data.data(),
            }
        }

        fn build_accounts_with_wrong_pda(
            &self,
            wrong_pda: Pubkey,
        ) -> Vec<(Pubkey, solana_sdk::account::Account)> {
            vec![
                create_gmp_app_state_account(wrong_pda, self.app_state_bump, false),
                create_authority_account(self.payer),
                create_router_program_account(self.router_program),
                create_router_state_pda(),
                create_authority_account(self.packet_commitment),
                create_ibc_app_pda(GMP_PORT_ID),
                create_client_pda(TEST_SOURCE_CLIENT, self.light_client_program),
                create_authority_account(self.light_client_program),
                create_owned_account(self.client_state, self.light_client_program),
                create_instructions_sysvar_account(),
                create_authority_account(self.consensus_state),
                create_system_program_account(),
            ]
        }

        fn build_instruction_with_wrong_router(
            &self,
            msg: SendCallMsg,
            wrong_router: Pubkey,
        ) -> Instruction {
            let instruction_data = crate::instruction::SendCallCpi { msg };

            Instruction {
                program_id: crate::ID,
                accounts: vec![
                    AccountMeta::new(self.app_state_pda, false),
                    AccountMeta::new(self.payer, true),
                    AccountMeta::new_readonly(wrong_router, false),
                    AccountMeta::new_readonly(self.router_state, false),
                    AccountMeta::new(self.packet_commitment, false),
                    AccountMeta::new_readonly(self.ibc_app, false),
                    AccountMeta::new_readonly(self.client, false),
                    AccountMeta::new_readonly(self.light_client_program, false),
                    AccountMeta::new_readonly(self.client_state, false),
                    AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
                    AccountMeta::new_readonly(self.consensus_state, false),
                    AccountMeta::new_readonly(system_program::ID, false),
                ],
                data: instruction_data.data(),
            }
        }

        fn build_accounts_with_wrong_router(
            &self,
            wrong_router: Pubkey,
        ) -> Vec<(Pubkey, solana_sdk::account::Account)> {
            vec![
                create_gmp_app_state_account(self.app_state_pda, self.app_state_bump, false),
                create_authority_account(self.payer),
                create_router_program_account(wrong_router),
                create_router_state_pda(),
                create_authority_account(self.packet_commitment),
                create_ibc_app_pda(GMP_PORT_ID),
                create_client_pda(TEST_SOURCE_CLIENT, self.light_client_program),
                create_authority_account(self.light_client_program),
                create_owned_account(self.client_state, self.light_client_program),
                create_instructions_sysvar_account(),
                create_authority_account(self.consensus_state),
                create_system_program_account(),
            ]
        }
    }

    const ANCHOR_CONSTRAINT_ADDRESS: u32 = anchor_lang::error::ErrorCode::ConstraintAddress as u32;
    const ANCHOR_CONSTRAINT_SEEDS: u32 = 2006;
    const ANCHOR_INVALID_PROGRAM_ID: u32 = 3008;

    fn gmp_error(err: GMPError) -> u32 {
        anchor_lang::error::ERROR_CODE_OFFSET + err as u32
    }

    // Handler-level validations (EmptyPayload, SaltTooLong, MemoTooLong, etc.)
    // are not testable here because `reject_direct_calls()` fires first at
    // Mollusk's stack height 1. They share `send_call_inner` with `send_call`
    // and are covered by `send_call::tests`.
    #[derive(Clone, Copy)]
    enum SendCallCpiErrorCase {
        AppPaused,
        FakeSysvar,
        InvalidAppStatePda,
        WrongRouterProgram,
        WrongRouterStatePda,
        WrongIbcAppPda,
        WrongClientPda,
        WrongLightClientProgram,
        WrongClientStateOwner,
        EmptyClientId,
    }

    fn run_send_call_cpi_error_test(case: SendCallCpiErrorCase) {
        let ctx = TestContext::new();
        let mut msg = TestContext::create_valid_msg();
        let sysvar = create_instructions_sysvar_account();

        let (instruction, accounts, expected_error) = match case {
            SendCallCpiErrorCase::AppPaused => {
                let instruction = ctx.build_instruction(msg, sysvar.0);
                let accounts = ctx.build_accounts(true, sysvar);
                (instruction, accounts, gmp_error(GMPError::AppPaused))
            }
            SendCallCpiErrorCase::FakeSysvar => {
                let fake_sysvar = create_fake_instructions_sysvar_account(Pubkey::new_unique());
                let instruction = ctx.build_instruction(msg, fake_sysvar.0);
                let accounts = ctx.build_accounts(false, fake_sysvar);
                (instruction, accounts, ANCHOR_CONSTRAINT_ADDRESS)
            }
            SendCallCpiErrorCase::InvalidAppStatePda => {
                let wrong_pda = Pubkey::new_unique();
                let instruction = ctx.build_instruction_with_wrong_pda(msg, wrong_pda);
                let accounts = ctx.build_accounts_with_wrong_pda(wrong_pda);
                (instruction, accounts, ANCHOR_CONSTRAINT_SEEDS)
            }
            SendCallCpiErrorCase::WrongRouterProgram => {
                let wrong_router = Pubkey::new_unique();
                let instruction = ctx.build_instruction_with_wrong_router(msg, wrong_router);
                let accounts = ctx.build_accounts_with_wrong_router(wrong_router);
                (instruction, accounts, ANCHOR_INVALID_PROGRAM_ID)
            }
            SendCallCpiErrorCase::WrongRouterStatePda => {
                let wrong_pda = Pubkey::new_unique();
                let mut instruction = ctx.build_instruction(msg, sysvar.0);
                let mut accounts = ctx.build_accounts(false, sysvar);
                instruction.accounts[3] = AccountMeta::new_readonly(wrong_pda, false);
                accounts[3].0 = wrong_pda;
                (instruction, accounts, ANCHOR_CONSTRAINT_SEEDS)
            }
            SendCallCpiErrorCase::WrongIbcAppPda => {
                let wrong_pda = Pubkey::new_unique();
                let mut instruction = ctx.build_instruction(msg, sysvar.0);
                let mut accounts = ctx.build_accounts(false, sysvar);
                instruction.accounts[5] = AccountMeta::new_readonly(wrong_pda, false);
                accounts[5].0 = wrong_pda;
                (instruction, accounts, ANCHOR_CONSTRAINT_SEEDS)
            }
            SendCallCpiErrorCase::WrongClientPda => {
                let wrong_pda = Pubkey::new_unique();
                let mut instruction = ctx.build_instruction(msg, sysvar.0);
                let mut accounts = ctx.build_accounts(false, sysvar);
                instruction.accounts[6] = AccountMeta::new_readonly(wrong_pda, false);
                accounts[6].0 = wrong_pda;
                (instruction, accounts, ANCHOR_CONSTRAINT_SEEDS)
            }
            SendCallCpiErrorCase::WrongLightClientProgram => {
                let wrong_program = Pubkey::new_unique();
                let mut instruction = ctx.build_instruction(msg, sysvar.0);
                let mut accounts = ctx.build_accounts(false, sysvar);
                instruction.accounts[7] = AccountMeta::new_readonly(wrong_program, false);
                accounts[7] = create_authority_account(wrong_program);
                (
                    instruction,
                    accounts,
                    gmp_error(GMPError::InvalidLightClientProgram),
                )
            }
            SendCallCpiErrorCase::WrongClientStateOwner => {
                let wrong_owner = Pubkey::new_unique();
                let instruction = ctx.build_instruction(msg, sysvar.0);
                let mut accounts = ctx.build_accounts(false, sysvar);
                accounts[8] = create_owned_account(ctx.client_state, wrong_owner);
                (
                    instruction,
                    accounts,
                    gmp_error(GMPError::InvalidAccountOwner),
                )
            }
            SendCallCpiErrorCase::EmptyClientId => {
                msg.source_client = String::new();
                let instruction = ctx.build_instruction(msg, sysvar.0);
                let accounts = ctx.build_accounts(false, sysvar);
                (instruction, accounts, ANCHOR_CONSTRAINT_SEEDS)
            }
        };

        let result = ctx.mollusk.process_instruction(&instruction, &accounts);
        assert_eq!(
            result.program_result,
            Err(solana_sdk::instruction::InstructionError::Custom(
                expected_error
            ))
            .into(),
        );
    }

    #[rstest]
    #[case::app_paused(SendCallCpiErrorCase::AppPaused)]
    #[case::fake_sysvar(SendCallCpiErrorCase::FakeSysvar)]
    #[case::invalid_app_state_pda(SendCallCpiErrorCase::InvalidAppStatePda)]
    #[case::wrong_router_program(SendCallCpiErrorCase::WrongRouterProgram)]
    #[case::wrong_router_state_pda(SendCallCpiErrorCase::WrongRouterStatePda)]
    #[case::wrong_ibc_app_pda(SendCallCpiErrorCase::WrongIbcAppPda)]
    #[case::wrong_client_pda(SendCallCpiErrorCase::WrongClientPda)]
    #[case::wrong_light_client_program(SendCallCpiErrorCase::WrongLightClientProgram)]
    #[case::wrong_client_state_owner(SendCallCpiErrorCase::WrongClientStateOwner)]
    #[case::empty_client_id(SendCallCpiErrorCase::EmptyClientId)]
    fn test_send_call_cpi_validation(#[case] case: SendCallCpiErrorCase) {
        run_send_call_cpi_error_test(case);
    }
}

/// Integration tests using `ProgramTest` with real BPF runtime.
///
/// These test the CPI detection logic which depends on `get_stack_height()` —
/// a syscall that returns 0 in Mollusk but works correctly under `ProgramTest`.
#[cfg(test)]
mod integration_tests {
    use crate::state::{GMPAppState, SendCallMsg};
    use crate::test_utils::*;
    use anchor_lang::InstructionData;
    use solana_sdk::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        signer::Signer,
    };

    fn build_send_call_cpi_ix(payer: Pubkey) -> Instruction {
        let (app_state_pda, _) = Pubkey::find_program_address(&[GMPAppState::SEED], &crate::ID);

        let msg = SendCallMsg {
            source_client: TEST_SOURCE_CLIENT.to_string(),
            sequence: 1,
            receiver: Pubkey::new_unique().to_string(),
            salt: vec![1, 2, 3],
            payload: vec![4, 5, 6],
            timeout_timestamp: 3600,
            memo: String::new(),
            encoding: crate::constants::ICS27_ENCODING_PROTOBUF.to_string(),
        };

        let (router_state, _) = create_router_state_pda();
        let (ibc_app, _) = create_ibc_app_pda(crate::constants::GMP_PORT_ID);
        let (client, _) = create_client_pda(TEST_SOURCE_CLIENT, TEST_LIGHT_CLIENT_ID);

        let ix_data = crate::instruction::SendCallCpi { msg };

        Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(app_state_pda, false),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(ics26_router::ID, false),
                AccountMeta::new_readonly(router_state, false),
                AccountMeta::new(
                    Pubkey::find_program_address(
                        &[
                            solana_ibc_types::Commitment::PACKET_COMMITMENT_SEED,
                            TEST_SOURCE_CLIENT.as_bytes(),
                            &1u64.to_le_bytes(),
                        ],
                        &ics26_router::ID,
                    )
                    .0,
                    false,
                ), // packet_commitment
                AccountMeta::new_readonly(ibc_app, false),
                AccountMeta::new_readonly(client, false),
                AccountMeta::new_readonly(TEST_LIGHT_CLIENT_ID, false), // light_client_program
                AccountMeta::new_readonly(TEST_CLIENT_STATE_ID, false), // client_state
                AccountMeta::new_readonly(
                    anchor_lang::solana_program::sysvar::instructions::ID,
                    false,
                ),
                AccountMeta::new_readonly(Pubkey::new_unique(), false), // consensus_state
                AccountMeta::new_readonly(solana_sdk::system_program::ID, false),
            ],
            data: ix_data.data(),
        }
    }

    /// Direct call (Tx -> GMP `send_call_cpi`) must be rejected with
    /// `DirectCallNotAllowed` because this instruction requires CPI.
    #[tokio::test]
    async fn test_direct_call_rejected() {
        let pt = setup_program_test();
        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let ix = build_send_call_cpi_ix(payer.pubkey());

        let result = process_tx(&banks_client, &payer, recent_blockhash, &[ix]).await;
        let err = result.expect_err("direct call should be rejected");
        assert_eq!(
            extract_custom_error(&err),
            Some(
                anchor_lang::error::ERROR_CODE_OFFSET
                    + crate::errors::GMPError::DirectCallNotAllowed as u32
            ),
            "expected DirectCallNotAllowed, got: {err:?}"
        );
    }

    /// CPI call (Tx -> `test_cpi_proxy` -> GMP `send_call_cpi`) should
    /// extract the caller's program ID as sender. It will fail later at the
    /// router CPI (router is not initialized) but not with `DirectCallNotAllowed`
    /// or `NestedCpiNotAllowed`.
    #[tokio::test]
    async fn test_cpi_call_extracts_caller_program_id() {
        let pt = setup_program_test();
        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let inner_ix = build_send_call_cpi_ix(payer.pubkey());
        let ix = wrap_in_test_cpi_proxy(payer.pubkey(), &inner_ix);

        let result = process_tx(&banks_client, &payer, recent_blockhash, &[ix]).await;
        let err = result.expect_err("CPI call should fail (router not initialized)");
        let code = extract_custom_error(&err);
        assert_ne!(
            code,
            Some(
                anchor_lang::error::ERROR_CODE_OFFSET
                    + crate::errors::GMPError::DirectCallNotAllowed as u32
            ),
            "single-level CPI must NOT fail with DirectCallNotAllowed"
        );
        assert_ne!(
            code,
            Some(
                anchor_lang::error::ERROR_CODE_OFFSET
                    + crate::errors::GMPError::UnauthorizedRouter as u32
            ),
            "single-level CPI must NOT fail with NestedCpiNotAllowed"
        );
    }

    /// Nested CPI (Tx -> `test_cpi_proxy` -> `test_cpi_target` -> GMP)
    /// must be rejected with `UnauthorizedRouter` (mapped from `NestedCpiNotAllowed`).
    #[tokio::test]
    async fn test_nested_cpi_rejected() {
        let pt = setup_program_test();
        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let inner_ix = build_send_call_cpi_ix(payer.pubkey());
        let middle_ix = wrap_in_test_cpi_target_proxy(payer.pubkey(), &inner_ix);
        let ix = wrap_in_test_cpi_proxy(payer.pubkey(), &middle_ix);

        let result = process_tx(&banks_client, &payer, recent_blockhash, &[ix]).await;
        let err = result.expect_err("nested CPI should be rejected");
        assert_eq!(
            extract_custom_error(&err),
            Some(
                anchor_lang::error::ERROR_CODE_OFFSET
                    + crate::errors::GMPError::UnauthorizedRouter as u32
            ),
            "expected UnauthorizedRouter (from NestedCpiNotAllowed), got: {err:?}"
        );
    }
}
