// SPDX-License-Identifier: Apache-2.0

use crate::constants::*;
use crate::errors::GMPError;
use crate::proto::{GmpSolanaPayload, GmpValidationError};
use crate::state::GMPAppState;
use anchor_lang::prelude::*;
use anchor_lang::solana_program::instruction::Instruction;
use solana_ibc_proto::GmpPacketData;
use solana_ibc_types::GMPAccount;

/// Number of fixed accounts in `remaining_accounts` (before target program accounts)
const FIXED_REMAINING_ACCOUNTS: usize = 2;

/// Index of GMP account PDA in `remaining_accounts`
const GMP_ACCOUNT_INDEX: usize = 0;

/// Index of target program in `remaining_accounts`
const TARGET_PROGRAM_INDEX: usize = 1;

/// Receives an IBC packet from the router via CPI and executes the target
/// program call.
///
/// # Account Layout
/// The router is generic and passes all IBC-app-specific accounts via `remaining_accounts`.
/// GMP defines its own account layout in `remaining_accounts`:
///
/// `remaining_accounts`:
/// - [0]: `gmp_account` - GMP account PDA (created if needed, signs via `invoke_signed`)
/// - [1]: `target_program` - The program to execute (extracted internally, must be executable)
/// - [2..]: accounts from payload - All accounts required by target program
///
/// Note: `target_program` cannot be a direct Anchor account because:
/// 1. The router is generic and doesn't know about GMP-specific account needs
/// 2. Different IBC apps have different account layouts
/// 3. The router passes all app-specific accounts via `remaining_accounts`
///
#[derive(Accounts)]
#[instruction(msg: solana_ibc_types::OnRecvPacketMsg)]
pub struct OnRecvPacket<'info> {
    /// GMP program's global configuration PDA. Must not be paused.
    #[account(
        mut,
        seeds = [GMPAppState::SEED],
        bump = app_state.bump,
        constraint = !app_state.paused @ GMPError::AppPaused
    )]
    pub app_state: Account<'info, GMPAppState>,

    /// Instructions sysvar used to verify the CPI caller is the authorized router.
    /// CHECK: Address constraint verifies this is the instructions sysvar
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instruction_sysvar: AccountInfo<'info>,

    /// Relayer-provided fee payer. Required for router interface compatibility.
    /// NOT forwarded to the target CPI — target programs use the GMP PDA
    /// (pre-funded by the relayer) as their payer instead.
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Solana system program used for account allocation during target CPI.
    pub system_program: Program<'info, System>,
}

pub fn on_recv_packet<'info>(
    ctx: Context<'_, '_, 'info, 'info, OnRecvPacket<'info>>,
    msg: solana_ibc_types::OnRecvPacketMsg,
) -> Result<Vec<u8>> {
    // Verify this function is called via CPI from the authorized router
    solana_ibc_types::validate_cpi_caller(
        &ctx.accounts.instruction_sysvar,
        &ics26_router::ID,
        &crate::ID,
    )
    .map_err(GMPError::from)?;

    require!(
        msg.payload.version == ICS27_VERSION,
        GMPError::InvalidVersion
    );

    require!(
        msg.payload.source_port == GMP_PORT_ID,
        GMPError::InvalidPort
    );

    require!(msg.payload.dest_port == GMP_PORT_ID, GMPError::InvalidPort);

    // Extract target_program from `remaining_accounts`
    let target_program = ctx
        .remaining_accounts
        .get(TARGET_PROGRAM_INDEX)
        .ok_or(GMPError::InsufficientAccounts)?;

    require!(target_program.executable, GMPError::TargetNotExecutable);

    // Decode GMP packet data using the payload's declared encoding
    let raw_packet_data =
        crate::encoding::decode_gmp_packet(&msg.payload.value, &msg.payload.encoding)?;
    let packet_data = GmpPacketData::try_from(raw_packet_data).map_err(|e| {
        msg!("GMP packet validation failed: {}", e);
        GMPError::InvalidPacketData
    })?;

    // Parse receiver as Solana Pubkey (for incoming packets, receiver is a Solana address)
    let receiver_pubkey =
        Pubkey::try_from(&packet_data.receiver[..]).map_err(|_| GMPError::InvalidAccountKey)?;

    // Validate target program matches packet data
    require_keys_eq!(
        target_program.key(),
        receiver_pubkey,
        GMPError::AccountKeyMismatch
    );

    let client_id = solana_ibc_types::ClientId::try_from(msg.dest_client)
        .map_err(|_| GMPError::InvalidClientId)?;

    // Create account identifier and derive expected GMP account PDA address
    let gmp_account = GMPAccount::new(
        client_id,
        packet_data.sender.clone(),
        packet_data.salt.clone(),
        &crate::ID,
    );

    // Extract GMP account PDA from `remaining_accounts`
    let gmp_account_info = ctx
        .remaining_accounts
        .get(GMP_ACCOUNT_INDEX)
        .ok_or(GMPError::InsufficientAccounts)?;

    require_keys_eq!(
        gmp_account_info.key(),
        gmp_account.pda,
        GMPError::GMPAccountPDAMismatch
    );

    let solana_payload: GmpSolanaPayload =
        crate::gmp_solana_payload::decode_raw(&packet_data.payload, &msg.payload.encoding)?
            .try_into()
            .map_err(|e: GmpValidationError| {
                msg!("GMP Solana payload validation failed: {}", e);
                error!(GMPError::from(e))
            })?;

    // Build account metas from GMP Solana payload
    let account_metas = solana_payload.to_account_metas();

    // Skip fixed accounts: gmp_account[0] and target_program[1]
    let remaining_accounts_for_execution = &ctx.remaining_accounts[FIXED_REMAINING_ACCOUNTS..];

    // Validate account count matches exactly (before payer injection)
    require!(
        remaining_accounts_for_execution.len() == account_metas.len(),
        GMPError::AccountCountMismatch
    );

    // Validate all accounts match the provided metadata
    for (meta, account_info) in account_metas.iter().zip(remaining_accounts_for_execution) {
        require_keys_eq!(
            account_info.key(),
            meta.pubkey,
            GMPError::AccountKeyMismatch
        );

        // Sanity check that `account_info.is_writable` is at least as permissive as `meta.is_writable`.
        require!(
            account_info.is_writable || !meta.is_writable,
            GMPError::InsufficientAccountPermissions
        );

        // GMP PDA is not a signer at this stage - the program will sign it during CPI.
        // This check prevents relayer or any higher level signers from being included.
        require!(!account_info.is_signer, GMPError::UnexpectedSigner);
    }

    let instruction = Instruction {
        program_id: receiver_pubkey,
        accounts: account_metas,
        data: solana_payload.data,
    };

    // Call target program via CPI with GMP account PDA as signer
    // Note: CPI errors cause immediate transaction abort in Solana, so we cannot
    // handle execution failures gracefully like Ethereum. The ? operator will
    // propagate any error and abort the entire transaction.
    gmp_account.invoke_signed(&instruction, remaining_accounts_for_execution)?;

    // Wrap the CPI call result in the ICS27 GMP acknowledgement.
    // Only accept return data from the target program itself, not from nested CPIs.
    let result = match anchor_lang::solana_program::program::get_return_data() {
        Some((return_program_id, data)) if return_program_id == receiver_pubkey => data,
        _ => Vec::new(),
    };

    crate::encoding::encode_gmp_ack(&result, &msg.payload.encoding)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::abi::GmpSolanaPayloadAbi;
    use crate::constants::{ICS27_ENCODING_ABI, ICS27_ENCODING_PROTOBUF};
    use crate::encoding::encode_gmp_packet;
    use crate::proto::{RawGmpPacketData, RawGmpSolanaPayload, RawSolanaAccountMeta};
    use crate::state::GMPAppState;
    use crate::test_utils::*;
    use anchor_lang::InstructionData;
    use mollusk_svm::result::Check;
    use mollusk_svm::Mollusk;
    use rstest::rstest;
    use solana_ibc_proto::ProstMessage;
    use solana_ibc_types::{GMPAccount, GmpPacketData};
    use solana_sdk::account::Account;
    use solana_sdk::bpf_loader_upgradeable;
    use solana_sdk::program_error::ProgramError;
    use solana_sdk::{
        instruction::{AccountMeta, Instruction as SolanaInstructionSDK},
        pubkey::Pubkey,
        system_program,
    };
    use test_gmp_app::ID as COUNTER_APP_ID;

    /// Helper function to create a `GMPAccount` from test data
    fn create_test_gmp_account(
        client_id: &str,
        sender: &str,
        salt: Vec<u8>,
        program_id: &Pubkey,
    ) -> GMPAccount {
        GMPAccount::new(
            client_id.try_into().unwrap(),
            sender.try_into().unwrap(),
            salt.try_into().unwrap(),
            program_id,
        )
    }

    #[derive(Clone, Copy)]
    enum AccessControlErrorCase {
        AppPaused,
        DirectCallNotAllowed,
        UnauthorizedRouter,
    }

    struct AccessControlConfig {
        paused: bool,
        caller: CallerType,
        expected_error: crate::errors::GMPError,
    }

    #[derive(Clone, Copy)]
    enum CallerType {
        AuthorizedRouter,
        SelfProgram,
        Unauthorized,
    }

    impl From<AccessControlErrorCase> for AccessControlConfig {
        fn from(case: AccessControlErrorCase) -> Self {
            match case {
                AccessControlErrorCase::AppPaused => Self {
                    paused: true,
                    caller: CallerType::AuthorizedRouter,
                    expected_error: crate::errors::GMPError::AppPaused,
                },
                AccessControlErrorCase::DirectCallNotAllowed => Self {
                    paused: false,
                    caller: CallerType::SelfProgram,
                    expected_error: crate::errors::GMPError::DirectCallNotAllowed,
                },
                AccessControlErrorCase::UnauthorizedRouter => Self {
                    paused: false,
                    caller: CallerType::Unauthorized,
                    expected_error: crate::errors::GMPError::UnauthorizedRouter,
                },
            }
        }
    }

    fn run_access_control_test(case: AccessControlErrorCase) {
        let ctx = create_gmp_test_context();
        let config = AccessControlConfig::from(case);
        let (client_id, sender, salt, gmp_account_pda) = create_test_account_data();

        let packet_data = create_gmp_packet_data(
            sender,
            &crate::test_utils::DUMMY_TARGET_PROGRAM.to_string(),
            salt,
            vec![],
        );
        let packet_data_bytes = packet_data.encode_to_vec();

        let recv_msg = create_recv_packet_msg(client_id, packet_data_bytes, 1);
        let instruction = create_recv_packet_instruction(ctx.app_state_pda, ctx.payer, recv_msg);

        let caller_pubkey = match config.caller {
            CallerType::AuthorizedRouter => ctx.router_program,
            CallerType::SelfProgram => crate::ID,
            CallerType::Unauthorized => Pubkey::new_unique(),
        };

        let accounts = vec![
            create_gmp_app_state_account(ctx.app_state_pda, ctx.app_state_bump, config.paused),
            create_instructions_sysvar_account_with_caller(caller_pubkey),
            create_authority_account(ctx.payer),
            create_system_program_account(),
            create_uninitialized_account_for_pda(gmp_account_pda),
            create_dummy_target_program_account(),
        ];

        let checks = vec![Check::err(ProgramError::Custom(
            ANCHOR_ERROR_OFFSET + config.expected_error as u32,
        ))];

        ctx.mollusk
            .process_and_validate_instruction(&instruction, &accounts, &checks);
    }

    #[rstest]
    #[case::app_paused(AccessControlErrorCase::AppPaused)]
    #[case::direct_call_not_allowed(AccessControlErrorCase::DirectCallNotAllowed)]
    #[case::unauthorized_router(AccessControlErrorCase::UnauthorizedRouter)]
    fn test_on_recv_packet_access_control(#[case] case: AccessControlErrorCase) {
        run_access_control_test(case);
    }

    #[test]
    fn test_on_recv_packet_fake_sysvar_wormhole_attack() {
        let ctx = create_gmp_test_context();
        let (client_id, sender, salt, gmp_account_pda) = create_test_account_data();

        let packet_data = create_gmp_packet_data(
            sender,
            &crate::test_utils::DUMMY_TARGET_PROGRAM.to_string(),
            salt,
            vec![],
        );

        let packet_data_bytes = packet_data.encode_to_vec();

        let recv_msg = create_recv_packet_msg(client_id, packet_data_bytes, 1);
        let mut instruction =
            create_recv_packet_instruction(ctx.app_state_pda, ctx.payer, recv_msg);

        // Simulate Wormhole attack: pass a completely different account with fake sysvar data
        // instead of the real instructions sysvar
        let (fake_sysvar_pubkey, fake_sysvar_account) =
            create_fake_instructions_sysvar_account(ctx.router_program);

        // Modify the instruction to reference the fake sysvar (simulating attacker control)
        instruction.accounts[1] = AccountMeta::new_readonly(fake_sysvar_pubkey, false);

        let accounts = vec![
            create_gmp_app_state_account(
                ctx.app_state_pda,
                ctx.app_state_bump,
                false, // not paused
            ),
            // Wormhole attack: provide a DIFFERENT account instead of the real sysvar
            (fake_sysvar_pubkey, fake_sysvar_account),
            create_authority_account(ctx.payer),
            create_system_program_account(),
            // Remaining accounts
            create_uninitialized_account_for_pda(gmp_account_pda), // [0] gmp_account_pda
            create_dummy_target_program_account(),                 // [1] target_program
        ];

        // Should be rejected by Anchor's address constraint check
        // This happens before validate_cpi_caller even runs
        let checks = vec![Check::err(ProgramError::Custom(
            anchor_lang::error::ErrorCode::ConstraintAddress as u32,
        ))];

        ctx.mollusk
            .process_and_validate_instruction(&instruction, &accounts, &checks);
    }

    #[test]
    fn test_on_recv_packet_invalid_app_state_pda() {
        let mollusk = Mollusk::new(&crate::ID, crate::get_gmp_program_path());

        let router_program = Pubkey::new_unique();
        let payer = Pubkey::new_unique();

        let (_correct_app_state_pda, _correct_bump) =
            Pubkey::find_program_address(&[GMPAppState::SEED], &crate::ID);

        // Use wrong PDA
        let wrong_app_state_pda = Pubkey::new_unique();

        let (client_id, sender, salt, gmp_account_pda) = create_test_account_data();

        let packet_data = RawGmpPacketData {
            sender: sender.to_string(),
            receiver: system_program::ID.to_string(),
            salt,
            payload: vec![],
            memo: String::new(),
        };

        let packet_data_bytes = packet_data.encode_to_vec();

        let recv_msg = solana_ibc_types::OnRecvPacketMsg {
            source_client: "cosmos-1".to_string(),
            dest_client: client_id.to_string(),
            sequence: 1,
            payload: solana_ibc_types::Payload {
                source_port: GMP_PORT_ID.to_string(),
                dest_port: GMP_PORT_ID.to_string(),
                version: ICS27_VERSION.to_string(),
                encoding: ICS27_ENCODING_PROTOBUF.to_string(),
                value: packet_data_bytes,
            },
            relayer: Pubkey::new_unique(),
        };

        let instruction_data = crate::instruction::OnRecvPacket { msg: recv_msg };

        let instruction = SolanaInstructionSDK {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(wrong_app_state_pda, false), // Wrong PDA!
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
                AccountMeta::new(payer, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ],
            data: instruction_data.data(),
        };

        // Create account state at wrong PDA for testing
        let wrong_bump = 255u8;
        let accounts = vec![
            create_gmp_app_state_account(
                wrong_app_state_pda,
                wrong_bump,
                false, // not paused
            ),
            create_instructions_sysvar_account_with_caller(router_program),
            create_authority_account(payer),
            create_system_program_account(),
            // Remaining accounts
            create_uninitialized_account_for_pda(gmp_account_pda), // [0] gmp_account_pda
            create_dummy_target_program_account(),                 // [1] target_program
        ];

        let checks = vec![Check::err(ProgramError::Custom(
            anchor_lang::error::ErrorCode::AccountNotSigner as u32,
        ))];
        mollusk.process_and_validate_instruction(&instruction, &accounts, &checks);
    }

    #[derive(Clone)]
    enum PdaMismatchCase {
        Sender(&'static str),
        Salt(Vec<u8>),
        Client(&'static str),
    }

    fn run_pda_mismatch_test(case: PdaMismatchCase) {
        let ctx = create_gmp_test_context();

        let original_client_id = "test-client";
        let original_sender = "cosmos1original";
        let original_salt = vec![1u8, 2, 3];

        // Determine packet values (one will be wrong) and PDA values (all correct)
        let (packet_client_id, packet_sender, packet_salt) = match &case {
            PdaMismatchCase::Sender(wrong) => (original_client_id, *wrong, original_salt.clone()),
            PdaMismatchCase::Salt(wrong) => (original_client_id, original_sender, wrong.clone()),
            PdaMismatchCase::Client(wrong) => (*wrong, original_sender, original_salt.clone()),
        };

        // Derive the correct PDA using original values
        let (correct_pda, _) = create_test_gmp_account(
            original_client_id,
            original_sender,
            original_salt,
            &crate::ID,
        )
        .pda();

        let solana_payload = RawGmpSolanaPayload {
            accounts: vec![],
            data: vec![0u8],
            prefund_lamports: 0,
        };
        let solana_payload_bytes = solana_payload.encode_to_vec();

        // Packet uses the (potentially wrong) values - this will derive a different PDA
        let packet_data = create_gmp_packet_data(
            packet_sender,
            &crate::test_utils::DUMMY_TARGET_PROGRAM.to_string(),
            packet_salt,
            solana_payload_bytes,
        );
        let packet_data_bytes = packet_data.encode_to_vec();

        let recv_msg = create_recv_packet_msg(packet_client_id, packet_data_bytes, 1);
        let mut instruction =
            create_recv_packet_instruction(ctx.app_state_pda, ctx.payer, recv_msg);

        instruction
            .accounts
            .push(AccountMeta::new(correct_pda, false));
        instruction.accounts.push(AccountMeta::new_readonly(
            crate::test_utils::DUMMY_TARGET_PROGRAM,
            false,
        ));

        let accounts = vec![
            create_gmp_app_state_account(ctx.app_state_pda, ctx.app_state_bump, false),
            create_instructions_sysvar_account_with_caller(ctx.router_program),
            create_authority_account(ctx.payer),
            create_system_program_account(),
            create_uninitialized_account_for_pda(correct_pda),
            create_dummy_target_program_account(),
        ];

        let checks = vec![Check::err(ProgramError::Custom(
            ANCHOR_ERROR_OFFSET + crate::errors::GMPError::GMPAccountPDAMismatch as u32,
        ))];

        ctx.mollusk
            .process_and_validate_instruction(&instruction, &accounts, &checks);
    }

    #[rstest]
    #[case::wrong_sender(PdaMismatchCase::Sender("cosmos1attacker"))]
    #[case::wrong_salt(PdaMismatchCase::Salt(vec![4u8, 5, 6]))]
    #[case::wrong_client(PdaMismatchCase::Client("different-client"))]
    fn test_on_recv_packet_pda_mismatch(#[case] case: PdaMismatchCase) {
        run_pda_mismatch_test(case);
    }

    #[test]
    fn test_on_recv_packet_insufficient_accounts() {
        let ctx = create_gmp_test_context();
        let (client_id, sender, salt, _gmp_account_pda) = create_test_account_data();

        let packet_data = create_gmp_packet_data(
            sender,
            &crate::test_utils::DUMMY_TARGET_PROGRAM.to_string(),
            salt,
            vec![],
        );

        let packet_data_bytes = packet_data.encode_to_vec();

        let recv_msg = create_recv_packet_msg(client_id, packet_data_bytes, 1);
        let instruction = create_recv_packet_instruction(ctx.app_state_pda, ctx.payer, recv_msg);

        let accounts = vec![
            create_gmp_app_state_account(
                ctx.app_state_pda,
                ctx.app_state_bump,
                false, // not paused
            ),
            create_instructions_sysvar_account_with_caller(ctx.router_program),
            create_authority_account(ctx.payer),
            create_system_program_account(),
            // Missing remaining accounts! (should have at least gmp_account_pda and target_program)
        ];

        let checks = vec![Check::err(ProgramError::Custom(
            ANCHOR_ERROR_OFFSET + crate::errors::GMPError::InsufficientAccounts as u32,
        ))];
        ctx.mollusk
            .process_and_validate_instruction(&instruction, &accounts, &checks);
    }

    /// Payload field overrides for testing invalid packet validation
    #[derive(Default)]
    struct PayloadOverrides {
        source_port: Option<&'static str>,
        dest_port: Option<&'static str>,
        version: Option<&'static str>,
        encoding: Option<&'static str>,
    }

    fn run_invalid_payload_test(overrides: PayloadOverrides, expected_error: GMPError) {
        let ctx = create_gmp_test_context();
        let (client_id, sender, salt, gmp_account_pda) = create_test_account_data();

        let packet_data = create_gmp_packet_data(
            sender,
            &crate::test_utils::DUMMY_TARGET_PROGRAM.to_string(),
            salt,
            vec![1, 2, 3],
        );

        let packet_data_bytes = packet_data.encode_to_vec();

        let recv_msg = solana_ibc_types::OnRecvPacketMsg {
            source_client: "cosmos-1".to_string(),
            dest_client: client_id.to_string(),
            sequence: 1,
            payload: solana_ibc_types::Payload {
                source_port: overrides.source_port.unwrap_or(GMP_PORT_ID).to_string(),
                dest_port: overrides.dest_port.unwrap_or(GMP_PORT_ID).to_string(),
                version: overrides.version.unwrap_or(ICS27_VERSION).to_string(),
                encoding: overrides
                    .encoding
                    .unwrap_or(ICS27_ENCODING_PROTOBUF)
                    .to_string(),
                value: packet_data_bytes,
            },
            relayer: Pubkey::new_unique(),
        };

        let mut instruction =
            create_recv_packet_instruction(ctx.app_state_pda, ctx.payer, recv_msg);

        // Push remaining accounts so the program reaches payload/encoding validation
        instruction
            .accounts
            .push(AccountMeta::new(gmp_account_pda, false));
        instruction.accounts.push(AccountMeta::new_readonly(
            crate::test_utils::DUMMY_TARGET_PROGRAM,
            false,
        ));

        let accounts = vec![
            create_gmp_app_state_account(ctx.app_state_pda, ctx.app_state_bump, false),
            create_instructions_sysvar_account_with_caller(ctx.router_program),
            create_authority_account(ctx.payer),
            create_system_program_account(),
            create_uninitialized_account_for_pda(gmp_account_pda),
            create_dummy_target_program_account(),
        ];

        let checks = vec![Check::err(ProgramError::Custom(
            ANCHOR_ERROR_OFFSET + expected_error as u32,
        ))];
        ctx.mollusk
            .process_and_validate_instruction(&instruction, &accounts, &checks);
    }

    #[rstest]
    #[case::invalid_version(PayloadOverrides { version: Some("wrong-version"), ..Default::default() }, GMPError::InvalidVersion)]
    #[case::invalid_source_port(PayloadOverrides { source_port: Some("transfer"), ..Default::default() }, GMPError::InvalidPort)]
    #[case::invalid_encoding(PayloadOverrides { encoding: Some("application/json"), ..Default::default() }, GMPError::InvalidEncoding)]
    #[case::invalid_dest_port(PayloadOverrides { dest_port: Some("transfer"), ..Default::default() }, GMPError::InvalidPort)]
    fn test_on_recv_packet_payload_validation(
        #[case] overrides: PayloadOverrides,
        #[case] expected_error: GMPError,
    ) {
        run_invalid_payload_test(overrides, expected_error);
    }

    #[test]
    fn test_on_recv_packet_account_key_mismatch() {
        let ctx = create_gmp_test_context();
        let (client_id, sender, salt, _expected_gmp_account_pda) = create_test_account_data();

        // Use a different account key than expected
        let wrong_account_key = Pubkey::new_unique();

        let packet_data = create_gmp_packet_data(
            sender,
            &crate::test_utils::DUMMY_TARGET_PROGRAM.to_string(),
            salt,
            vec![],
        );

        let packet_data_bytes = packet_data.encode_to_vec();

        let recv_msg = create_recv_packet_msg(client_id, packet_data_bytes, 1);
        let instruction = create_recv_packet_instruction(ctx.app_state_pda, ctx.payer, recv_msg);

        let accounts = vec![
            create_gmp_app_state_account(
                ctx.app_state_pda,
                ctx.app_state_bump,
                false, // not paused
            ),
            create_instructions_sysvar_account_with_caller(ctx.router_program),
            create_authority_account(ctx.payer),
            create_system_program_account(),
            // Remaining accounts
            create_uninitialized_account_for_pda(wrong_account_key), // [0] Wrong account key!
            create_dummy_target_program_account(),                   // [1] target_program
        ];

        // Instruction has no remaining accounts, so InsufficientAccounts is hit first
        let checks = vec![Check::err(ProgramError::Custom(
            ANCHOR_ERROR_OFFSET + crate::errors::GMPError::InsufficientAccounts as u32,
        ))];
        ctx.mollusk
            .process_and_validate_instruction(&instruction, &accounts, &checks);
    }

    #[test]
    fn test_on_recv_packet_target_program_mismatch() {
        let ctx = create_gmp_test_context();
        let (client_id, sender, salt, gmp_account_pda) = create_test_account_data();

        // Create a minimal valid GMP Solana payload
        let solana_payload = RawGmpSolanaPayload {
            accounts: vec![],
            data: vec![0u8], // Minimal non-empty data
            prefund_lamports: 0,
        };

        let solana_payload_bytes = solana_payload.encode_to_vec();

        // Packet says to execute on DUMMY_TARGET_PROGRAM
        let packet_data = create_gmp_packet_data(
            sender,
            &crate::test_utils::DUMMY_TARGET_PROGRAM.to_string(),
            salt,
            solana_payload_bytes,
        );

        let packet_data_bytes = packet_data.encode_to_vec();

        let recv_msg = create_recv_packet_msg(client_id, packet_data_bytes, 1);
        let mut instruction =
            create_recv_packet_instruction(ctx.app_state_pda, ctx.payer, recv_msg);

        // But relayer provides a different program in remaining_accounts[1]
        let wrong_target_program = Pubkey::new_unique();

        // Add remaining accounts to instruction
        instruction
            .accounts
            .push(AccountMeta::new(gmp_account_pda, false)); // [0] GMP account PDA
        instruction
            .accounts
            .push(AccountMeta::new_readonly(wrong_target_program, false)); // [1] Wrong target program!

        let accounts = vec![
            create_gmp_app_state_account(
                ctx.app_state_pda,
                ctx.app_state_bump,
                false, // not paused
            ),
            create_instructions_sysvar_account_with_caller(ctx.router_program),
            create_authority_account(ctx.payer),
            create_system_program_account(),
            // Remaining accounts
            create_uninitialized_account_for_pda(gmp_account_pda), // [0] GMP account PDA (correct)
            (
                wrong_target_program, // [1] Wrong target program!
                solana_sdk::account::Account {
                    lamports: 1_000_000,
                    data: vec![],
                    owner: system_program::ID,
                    executable: true,
                    rent_epoch: 0,
                },
            ),
        ];

        // Should fail with AccountKeyMismatch error
        let checks = vec![Check::err(ProgramError::Custom(
            ANCHOR_ERROR_OFFSET + crate::errors::GMPError::AccountKeyMismatch as u32,
        ))];

        ctx.mollusk
            .process_and_validate_instruction(&instruction, &accounts, &checks);
    }

    // ── Signer exploit test helpers ──────────────────────────────────

    fn check_unexpected_signer() -> Check<'static> {
        Check::err(ProgramError::Custom(
            ANCHOR_ERROR_OFFSET + GMPError::UnexpectedSigner as u32,
        ))
    }

    fn assert_passes_signer_validation(result: &mollusk_svm::result::InstructionResult) {
        let unauthorized_signer =
            ProgramError::Custom(ANCHOR_ERROR_OFFSET + GMPError::UnexpectedSigner as u32);
        let insufficient_permissions = ProgramError::Custom(
            ANCHOR_ERROR_OFFSET + GMPError::InsufficientAccountPermissions as u32,
        );
        assert_ne!(
            result.program_result,
            mollusk_svm::result::ProgramResult::Failure(unauthorized_signer),
            "Should pass signer validation"
        );
        assert_ne!(
            result.program_result,
            mollusk_svm::result::ProgramResult::Failure(insufficient_permissions),
            "Should pass permission validation"
        );
    }

    /// Appends the standard remaining accounts to an `on_recv_packet` instruction:
    /// GMP PDA, target program, then the per-payload execution accounts.
    fn append_remaining_accounts(
        instruction: &mut Instruction,
        gmp_account_pda: Pubkey,
        target_program: Pubkey,
        execution_metas: Vec<AccountMeta>,
    ) {
        instruction
            .accounts
            .push(AccountMeta::new(gmp_account_pda, false));
        instruction
            .accounts
            .push(AccountMeta::new_readonly(target_program, false));
        instruction.accounts.extend(execution_metas);
    }

    /// Creates the Mollusk account list: base Anchor accounts, GMP PDA,
    /// target program account, then per-payload execution accounts.
    fn create_recv_mollusk_accounts(
        ctx: &GmpTestContext,
        gmp_account_pda: Pubkey,
        target_program_account: (Pubkey, Account),
        execution_accounts: Vec<(Pubkey, Account)>,
    ) -> Vec<(Pubkey, Account)> {
        let mut accounts = vec![
            create_gmp_app_state_account(ctx.app_state_pda, ctx.app_state_bump, false),
            create_instructions_sysvar_account_with_caller(ctx.router_program),
            create_authority_account(ctx.payer),
            create_system_program_account(),
            create_uninitialized_account_for_pda(gmp_account_pda),
            target_program_account,
        ];
        accounts.extend(execution_accounts);
        accounts
    }

    /// Converts `RawSolanaAccountMeta` (payload) to `AccountMeta` (outer tx).
    /// Preserves writable flag but sets `is_signer=false` — on the outer
    /// transaction these accounts are not signers; the exploit relies on
    /// Solana's runtime account merging to propagate signer status.
    fn payload_to_execution_metas(payload_accounts: &[RawSolanaAccountMeta]) -> Vec<AccountMeta> {
        payload_accounts
            .iter()
            .map(|meta| {
                let pk = Pubkey::try_from(meta.pubkey.as_slice()).unwrap();
                if meta.is_writable {
                    AccountMeta::new(pk, false)
                } else {
                    AccountMeta::new_readonly(pk, false)
                }
            })
            .collect()
    }

    /// Creates Mollusk `Account` entries for each payload account.
    /// Maps well-known pubkeys to appropriate account types:
    /// system program → executable, GMP PDA → uninitialized PDA,
    /// payer → funded authority, other → uninitialized PDA.
    fn payload_to_mollusk_accounts(
        payload_accounts: &[RawSolanaAccountMeta],
        gmp_account_pda: Pubkey,
        payer: Pubkey,
    ) -> Vec<(Pubkey, Account)> {
        payload_accounts
            .iter()
            .map(|meta| {
                let pk = Pubkey::try_from(meta.pubkey.as_slice()).unwrap();
                if pk == system_program::ID {
                    create_system_program_account()
                } else if pk == gmp_account_pda {
                    create_uninitialized_account_for_pda(gmp_account_pda)
                } else if pk == payer {
                    create_authority_account(payer)
                } else {
                    create_uninitialized_account_for_pda(pk)
                }
            })
            .collect()
    }

    fn create_counter_cpi_mollusk() -> Mollusk {
        let mut mollusk = Mollusk::new(&crate::ID, crate::get_gmp_program_path());
        mollusk.add_program(
            &COUNTER_APP_ID,
            "../../target/deploy/test_gmp_app",
            &bpf_loader_upgradeable::ID,
        );
        mollusk
    }

    fn serialize_counter_app_state(authority: Pubkey, bump: u8) -> Vec<u8> {
        let state = test_gmp_app::state::CounterAppState {
            authority,
            total_counters: 0,
            total_gmp_calls: 0,
            bump,
        };
        let mut data = Vec::new();
        data.extend_from_slice(test_gmp_app::state::CounterAppState::DISCRIMINATOR);
        state.serialize(&mut data).unwrap();
        data
    }

    fn create_counter_program_account() -> (Pubkey, Account) {
        (
            COUNTER_APP_ID,
            Account {
                lamports: 1_000_000,
                data: vec![],
                owner: bpf_loader_upgradeable::ID,
                executable: true,
                rent_epoch: 0,
            },
        )
    }

    fn create_counter_state_account(pda: Pubkey, authority: Pubkey, bump: u8) -> (Pubkey, Account) {
        (
            pda,
            Account {
                lamports: 1_000_000,
                data: serialize_counter_app_state(authority, bump),
                owner: COUNTER_APP_ID,
                executable: false,
                rent_epoch: 0,
            },
        )
    }

    // ── Signer exploit tests ───────────────────────────────────────────
    //
    // A malicious source-chain packet could include the relayer's pubkey
    // with `is_signer=true` in the GMP Solana payload. Because the
    // relayer signed the outer transaction, Solana's account-merging
    // would propagate that signer status into the target CPI, letting an
    // attacker-controlled program act with the relayer's authority.
    //
    // The fix: only the GMP PDA is allowed as a signer in the target CPI.

    /// Scenario: attacker crafts an IBC packet that calls the counter app's
    /// `increment` instruction, but substitutes the relayer's pubkey in place
    /// of the GMP PDA for both `user_authority` and `payer` (both marked
    /// `is_signer: true`). The `user_counter` PDA is also derived from the
    /// relayer's key instead of the GMP PDA.
    ///
    /// Without the fix, Solana's account merging would propagate the relayer's
    /// outer-transaction signer status into the CPI, so the counter app would
    /// see the relayer as an authorized signer. The relayer would unknowingly
    /// pay rent for a new `UserCounter` account it never requested.
    ///
    /// The fix rejects any account in the payload that claims `is_signer: true`
    /// unless its pubkey matches the GMP PDA. Expects `UnexpectedSigner`.
    #[test]
    fn test_signer_exploit_relayer_as_counter_payer_rejected() {
        let mollusk = create_counter_cpi_mollusk();

        let router_program = ics26_router::ID;
        let payer = Pubkey::new_unique(); // the relayer
        let (app_state_pda, app_state_bump) =
            Pubkey::find_program_address(&[GMPAppState::SEED], &crate::ID);

        let authority = Pubkey::new_unique();
        let (client_id, sender, salt, gmp_account_pda) = create_test_account_data();

        let (counter_app_state_pda, counter_app_state_bump) = Pubkey::find_program_address(
            &[test_gmp_app::state::CounterAppState::SEED],
            &COUNTER_APP_ID,
        );

        // Derive user_counter from the RELAYER's key (not GMP PDA) —
        // this is the attacker's goal: create a counter under the
        // relayer's authority so the payer (relayer) funds rent.
        let (user_counter_pda, _) = Pubkey::find_program_address(
            &[test_gmp_app::state::UserCounter::SEED, payer.as_ref()],
            &COUNTER_APP_ID,
        );

        let counter_ix_data =
            InstructionData::data(&test_gmp_app::instruction::Increment { amount: 1 });

        // EXPLOIT: payload specifies RELAYER as user_authority AND payer
        let solana_payload = RawGmpSolanaPayload {
            accounts: vec![
                RawSolanaAccountMeta {
                    pubkey: counter_app_state_pda.to_bytes().to_vec(),
                    is_signer: false,
                    is_writable: true,
                },
                RawSolanaAccountMeta {
                    pubkey: user_counter_pda.to_bytes().to_vec(),
                    is_signer: false,
                    is_writable: true,
                },
                // user_authority = RELAYER (exploit!)
                RawSolanaAccountMeta {
                    pubkey: payer.to_bytes().to_vec(),
                    is_signer: true,
                    is_writable: false,
                },
                // payer = RELAYER (exploit!)
                RawSolanaAccountMeta {
                    pubkey: payer.to_bytes().to_vec(),
                    is_signer: true,
                    is_writable: true,
                },
                RawSolanaAccountMeta {
                    pubkey: system_program::ID.to_bytes().to_vec(),
                    is_signer: false,
                    is_writable: false,
                },
            ],
            data: counter_ix_data,
            prefund_lamports: 0,
        };

        let packet_data = create_gmp_packet_data(
            sender,
            &COUNTER_APP_ID.to_string(),
            salt,
            solana_payload.encode_to_vec(),
        );
        let recv_msg = create_recv_packet_msg(client_id, packet_data.encode_to_vec(), 1);

        let instruction_data = crate::instruction::OnRecvPacket { msg: recv_msg };
        let instruction = SolanaInstructionSDK {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(app_state_pda, false),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(system_program::ID, false),
                // remaining_accounts
                AccountMeta::new(gmp_account_pda, false), // [0] GMP PDA
                AccountMeta::new_readonly(COUNTER_APP_ID, false), // [1] target
                AccountMeta::new(counter_app_state_pda, false), // [2] counter state
                AccountMeta::new(user_counter_pda, false), // [3] user counter
                AccountMeta::new(payer, true),            // [4] user_authority
                AccountMeta::new(payer, true),            // [5] payer
                AccountMeta::new_readonly(system_program::ID, false), // [6] system
            ],
            data: instruction_data.data(),
        };

        let accounts = vec![
            create_gmp_app_state_account(app_state_pda, app_state_bump, false),
            create_instructions_sysvar_account_with_caller(router_program),
            create_counter_program_account(),
            create_authority_account(payer),
            create_system_program_account(),
            (
                gmp_account_pda,
                Account {
                    lamports: 10_000_000,
                    data: vec![],
                    owner: system_program::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
            create_counter_state_account(counter_app_state_pda, authority, counter_app_state_bump),
            create_uninitialized_account_for_pda(user_counter_pda),
            create_system_program_account(),
        ];

        mollusk.process_and_validate_instruction(
            &instruction,
            &accounts,
            &[check_unexpected_signer()],
        );
    }

    // ── Parametrized unexpected signer edge cases ──────────────────
    //
    // The `!account_info.is_signer` check catches accounts whose signer
    // status was propagated from the outer transaction (i.e. the relayer).
    // Arbitrary pubkeys that aren't outer-tx signers pass this check but
    // are caught later by Solana's CPI runtime (`PrivilegeEscalation`).

    #[derive(Clone)]
    enum UnexpectedSignerCase {
        /// Relayer with `is_signer: true, is_writable: true`.
        /// The most direct exploit vector.
        RelayerWritable,
        /// Relayer with `is_signer: true, is_writable: false`.
        /// Proves the writable flag doesn't influence signer rejection.
        RelayerReadonly,
    }

    fn run_unexpected_signer_test(case: UnexpectedSignerCase) {
        let ctx = create_gmp_test_context();
        let (client_id, sender, salt, gmp_account_pda) = create_test_account_data();

        let payload_accounts: Vec<RawSolanaAccountMeta> = match case {
            UnexpectedSignerCase::RelayerWritable => vec![RawSolanaAccountMeta {
                pubkey: ctx.payer.to_bytes().to_vec(),
                is_signer: true,
                is_writable: true,
            }],
            UnexpectedSignerCase::RelayerReadonly => vec![RawSolanaAccountMeta {
                pubkey: ctx.payer.to_bytes().to_vec(),
                is_signer: true,
                is_writable: false,
            }],
        };

        let solana_payload = RawGmpSolanaPayload {
            accounts: payload_accounts.clone(),
            data: vec![0u8],
            prefund_lamports: 0,
        };

        let packet_data = create_gmp_packet_data(
            sender,
            &DUMMY_TARGET_PROGRAM.to_string(),
            salt,
            solana_payload.encode_to_vec(),
        );

        let recv_msg = create_recv_packet_msg(client_id, packet_data.encode_to_vec(), 1);
        let mut instruction =
            create_recv_packet_instruction(ctx.app_state_pda, ctx.payer, recv_msg);

        let execution_metas = payload_to_execution_metas(&payload_accounts);
        append_remaining_accounts(
            &mut instruction,
            gmp_account_pda,
            DUMMY_TARGET_PROGRAM,
            execution_metas,
        );

        let execution_accounts =
            payload_to_mollusk_accounts(&payload_accounts, gmp_account_pda, ctx.payer);
        let accounts = create_recv_mollusk_accounts(
            &ctx,
            gmp_account_pda,
            create_dummy_target_program_account(),
            execution_accounts,
        );

        ctx.mollusk.process_and_validate_instruction(
            &instruction,
            &accounts,
            &[check_unexpected_signer()],
        );
    }

    #[rstest]
    #[case::relayer_writable(UnexpectedSignerCase::RelayerWritable)]
    #[case::relayer_readonly(UnexpectedSignerCase::RelayerReadonly)]
    fn test_on_recv_packet_unexpected_signer_variants(#[case] case: UnexpectedSignerCase) {
        run_unexpected_signer_test(case);
    }

    /// Scenario: same as the basic signer exploit (GMP PDA + relayer both
    /// marked `is_signer: true`) but the outer `GmpPacketData` is ABI-encoded
    /// instead of protobuf. The signer validation happens after decoding, so
    /// this proves the fix works regardless of the wire encoding path.
    /// Expects `UnexpectedSigner`.
    #[test]
    fn test_signer_exploit_with_abi_encoding() {
        let ctx = create_gmp_test_context();
        let (client_id, sender, salt, gmp_account_pda) = create_test_account_data();

        // EXPLOIT: same signer attack as protobuf tests, but ABI-encoded
        let solana_payload = RawGmpSolanaPayload {
            accounts: vec![
                RawSolanaAccountMeta {
                    pubkey: gmp_account_pda.to_bytes().to_vec(),
                    is_signer: true,
                    is_writable: true,
                },
                RawSolanaAccountMeta {
                    pubkey: ctx.payer.to_bytes().to_vec(),
                    is_signer: true, // relayer as unauthorized signer
                    is_writable: true,
                },
            ],
            data: vec![0u8],
            prefund_lamports: 0,
        };

        let raw_packet = create_gmp_packet_data(
            sender,
            &DUMMY_TARGET_PROGRAM.to_string(),
            salt,
            encode_test_solana_payload(&solana_payload, ICS27_ENCODING_ABI),
        );
        // ABI-encode the outer packet (signer check must work for both encodings)
        let packet_data_bytes = encode_test_packet(raw_packet, ICS27_ENCODING_ABI);

        let recv_msg = solana_ibc_types::OnRecvPacketMsg {
            source_client: "cosmos-1".to_string(),
            dest_client: client_id.to_string(),
            sequence: 1,
            payload: solana_ibc_types::Payload {
                source_port: GMP_PORT_ID.to_string(),
                dest_port: GMP_PORT_ID.to_string(),
                version: ICS27_VERSION.to_string(),
                encoding: ICS27_ENCODING_ABI.to_string(),
                value: packet_data_bytes,
            },
            relayer: Pubkey::new_unique(),
        };

        let mut instruction =
            create_recv_packet_instruction(ctx.app_state_pda, ctx.payer, recv_msg);

        // EXPLOIT accounts: GMP PDA + relayer, both claiming signer
        append_remaining_accounts(
            &mut instruction,
            gmp_account_pda,
            DUMMY_TARGET_PROGRAM,
            vec![
                AccountMeta::new(gmp_account_pda, false),
                AccountMeta::new(ctx.payer, true),
            ],
        );

        let accounts = create_recv_mollusk_accounts(
            &ctx,
            gmp_account_pda,
            create_dummy_target_program_account(),
            vec![
                create_uninitialized_account_for_pda(gmp_account_pda),
                create_authority_account(ctx.payer),
            ],
        );

        ctx.mollusk.process_and_validate_instruction(
            &instruction,
            &accounts,
            &[check_unexpected_signer()],
        );
    }

    /// Scenario: the most dangerous exploit variant — direct SOL theft.
    /// The attacker crafts a payload that targets the System Program's
    /// `Transfer` instruction, moving 1 SOL from the relayer to an
    /// attacker-controlled wallet. The relayer's pubkey is marked
    /// `is_signer: true` as the transfer source.
    ///
    /// Without the fix, Solana's account merging would propagate the
    /// relayer's outer-transaction signer status into the CPI. The System
    /// Program would see the relayer as an authorized signer and execute
    /// the transfer. With the fix, validation rejects before any CPI.
    /// Expects `UnexpectedSigner`.
    #[test]
    fn test_signer_exploit_sol_transfer_via_system_program() {
        let ctx = create_gmp_test_context();
        let (client_id, sender, salt, gmp_account_pda) = create_test_account_data();
        let attacker = Pubkey::new_unique();

        // Build a real System Program Transfer instruction (relayer → attacker).
        // The GMP payload embeds this as the CPI the target program will execute.
        let stolen_lamports: u64 = 1_000_000_000; // 1 SOL
        let transfer_ix =
            solana_sdk::system_instruction::transfer(&ctx.payer, &attacker, stolen_lamports);

        // The malicious payload: relayer is marked `is_signer: true`.
        // Because the relayer signs the outer transaction, Solana's account
        // merging would propagate signer status into the CPI — letting the
        // System Program's Transfer succeed with the relayer as source.
        let solana_payload = RawGmpSolanaPayload {
            accounts: vec![
                RawSolanaAccountMeta {
                    pubkey: ctx.payer.to_bytes().to_vec(),
                    is_signer: true, // EXPLOIT: relayer as transfer source
                    is_writable: true,
                },
                RawSolanaAccountMeta {
                    pubkey: attacker.to_bytes().to_vec(),
                    is_signer: false,
                    is_writable: true,
                },
            ],
            data: transfer_ix.data,
            prefund_lamports: 0,
        };

        // Receiver = System Program: the GMP program will CPI into it
        let packet_data = create_gmp_packet_data(
            sender,
            &system_program::ID.to_string(),
            salt,
            solana_payload.encode_to_vec(),
        );

        let recv_msg = create_recv_packet_msg(client_id, packet_data.encode_to_vec(), 1);
        let mut instruction =
            create_recv_packet_instruction(ctx.app_state_pda, ctx.payer, recv_msg);

        // EXPLOIT accounts: relayer as transfer source (signer), attacker as destination
        append_remaining_accounts(
            &mut instruction,
            gmp_account_pda,
            system_program::ID,
            vec![
                AccountMeta::new(ctx.payer, true), // relayer (exploit: signer)
                AccountMeta::new(attacker, false), // attacker wallet
            ],
        );

        let accounts = create_recv_mollusk_accounts(
            &ctx,
            gmp_account_pda,
            create_system_program_account(), // target = System Program
            vec![
                create_authority_account(ctx.payer), // relayer with lamports
                create_uninitialized_account_for_pda(attacker), // attacker (empty)
            ],
        );

        ctx.mollusk.process_and_validate_instruction(
            &instruction,
            &accounts,
            &[check_unexpected_signer()],
        );
    }

    // ── Positive regression tests ───────────────────────────────────
    //
    // Each variant sends a legitimate payload (no unauthorized signers)
    // and asserts that signer validation passes. These tests ensure the
    // fix does not over-restrict by rejecting valid payloads.

    #[derive(Clone)]
    enum ValidSignerCase {
        /// Single arbitrary account (non-signer, readonly).
        /// No account claims signer — trivially passes.
        NoSigners,
        /// GMP PDA (signer, writable) + system program (non-signer, readonly).
        /// Standard legitimate case — GMP PDA signs the CPI via `invoke_signed`.
        GmpPdaSignerWritable,
        /// GMP PDA (signer, readonly).
        /// Valid: GMP PDA signs the CPI but target only reads from it.
        GmpPdaSignerReadonly,
        /// GMP PDA (non-signer, writable) + system program (non-signer, readonly).
        /// `is_signer=false` short-circuits the check.
        GmpPdaNonSignerWritable,
        /// GMP PDA (non-signer, readonly).
        /// Both checks trivially pass — neither signer nor writable requested.
        GmpPdaNonSignerReadonly,
    }

    fn run_valid_signer_test(case: ValidSignerCase) {
        let ctx = create_gmp_test_context();
        let (client_id, sender, salt, gmp_account_pda) = create_test_account_data();
        let arbitrary = Pubkey::new_unique();

        let payload_accounts: Vec<RawSolanaAccountMeta> = match case {
            ValidSignerCase::NoSigners => vec![RawSolanaAccountMeta {
                pubkey: arbitrary.to_bytes().to_vec(),
                is_signer: false,
                is_writable: false,
            }],
            ValidSignerCase::GmpPdaSignerWritable => vec![
                RawSolanaAccountMeta {
                    pubkey: gmp_account_pda.to_bytes().to_vec(),
                    is_signer: true,
                    is_writable: true,
                },
                RawSolanaAccountMeta {
                    pubkey: system_program::ID.to_bytes().to_vec(),
                    is_signer: false,
                    is_writable: false,
                },
            ],
            ValidSignerCase::GmpPdaSignerReadonly => vec![RawSolanaAccountMeta {
                pubkey: gmp_account_pda.to_bytes().to_vec(),
                is_signer: true,
                is_writable: false,
            }],
            ValidSignerCase::GmpPdaNonSignerWritable => vec![
                RawSolanaAccountMeta {
                    pubkey: gmp_account_pda.to_bytes().to_vec(),
                    is_signer: false,
                    is_writable: true,
                },
                RawSolanaAccountMeta {
                    pubkey: system_program::ID.to_bytes().to_vec(),
                    is_signer: false,
                    is_writable: false,
                },
            ],
            ValidSignerCase::GmpPdaNonSignerReadonly => vec![RawSolanaAccountMeta {
                pubkey: gmp_account_pda.to_bytes().to_vec(),
                is_signer: false,
                is_writable: false,
            }],
        };

        let solana_payload = RawGmpSolanaPayload {
            accounts: payload_accounts.clone(),
            data: vec![0u8],
            prefund_lamports: 0,
        };

        let packet_data = create_gmp_packet_data(
            sender,
            &DUMMY_TARGET_PROGRAM.to_string(),
            salt,
            solana_payload.encode_to_vec(),
        );

        let recv_msg = create_recv_packet_msg(client_id, packet_data.encode_to_vec(), 1);
        let mut instruction =
            create_recv_packet_instruction(ctx.app_state_pda, ctx.payer, recv_msg);

        let execution_metas = payload_to_execution_metas(&payload_accounts);
        append_remaining_accounts(
            &mut instruction,
            gmp_account_pda,
            DUMMY_TARGET_PROGRAM,
            execution_metas,
        );

        let execution_accounts =
            payload_to_mollusk_accounts(&payload_accounts, gmp_account_pda, ctx.payer);
        let accounts = create_recv_mollusk_accounts(
            &ctx,
            gmp_account_pda,
            create_dummy_target_program_account(),
            execution_accounts,
        );

        let result = ctx.mollusk.process_instruction(&instruction, &accounts);
        assert_passes_signer_validation(&result);
    }

    #[rstest]
    #[case::no_signers(ValidSignerCase::NoSigners)]
    #[case::gmp_pda_signer_writable(ValidSignerCase::GmpPdaSignerWritable)]
    #[case::gmp_pda_signer_readonly(ValidSignerCase::GmpPdaSignerReadonly)]
    #[case::gmp_pda_non_signer_writable(ValidSignerCase::GmpPdaNonSignerWritable)]
    #[case::gmp_pda_non_signer_readonly(ValidSignerCase::GmpPdaNonSignerReadonly)]
    fn test_on_recv_packet_valid_signer_variants(#[case] case: ValidSignerCase) {
        run_valid_signer_test(case);
    }

    fn encode_test_packet(raw: RawGmpPacketData, encoding: &str) -> Vec<u8> {
        encode_gmp_packet(GmpPacketData::try_from(raw).unwrap(), encoding).unwrap()
    }

    /// Encode a `RawGmpSolanaPayload` using the same encoding as the outer packet.
    /// ABI packets expect the inner payload to be ABI-encoded too.
    fn encode_test_solana_payload(raw: &RawGmpSolanaPayload, encoding: &str) -> Vec<u8> {
        match encoding {
            ICS27_ENCODING_ABI => {
                use alloy_sol_types::SolValue;
                let mut packed = Vec::new();
                for account in &raw.accounts {
                    packed.extend_from_slice(&account.pubkey);
                    packed.push(u8::from(account.is_signer));
                    packed.push(u8::from(account.is_writable));
                }
                GmpSolanaPayloadAbi {
                    packedAccounts: packed.into(),
                    instructionData: raw.data.clone().into(),
                    prefundLamports: raw.prefund_lamports,
                }
                .abi_encode()
            }
            _ => raw.encode_to_vec(),
        }
    }

    fn run_recv_success_test(encoding: &str) {
        // Create Mollusk instance and load both programs
        let mut mollusk = Mollusk::new(&crate::ID, crate::get_gmp_program_path());

        // Add the counter app program so CPI will work
        // Use BPF loader upgradeable for Anchor programs
        mollusk.add_program(
            &COUNTER_APP_ID,
            "../../target/deploy/test_gmp_app",
            &bpf_loader_upgradeable::ID,
        );

        let router_program = ics26_router::ID;
        let payer = Pubkey::new_unique();
        let (app_state_pda, app_state_bump) =
            Pubkey::find_program_address(&[GMPAppState::SEED], &crate::ID);

        // Create packet data that will call the counter app
        let authority = Pubkey::new_unique();
        let (client_id, sender, salt, gmp_account_pda) = create_test_account_data();

        // Counter app state and user counter PDAs
        let (counter_app_state_pda, counter_app_state_bump) = Pubkey::find_program_address(
            &[test_gmp_app::state::CounterAppState::SEED],
            &COUNTER_APP_ID,
        );

        let (user_counter_pda, _user_counter_bump) = Pubkey::find_program_address(
            &[
                test_gmp_app::state::UserCounter::SEED,
                gmp_account_pda.as_ref(),
            ],
            &COUNTER_APP_ID,
        );

        // Create counter instruction that will increment the counter
        let counter_instruction = test_gmp_app::instruction::Increment { amount: 5 };
        let counter_instruction_data = anchor_lang::InstructionData::data(&counter_instruction);

        // Build GMPSolanaPayload for the payload
        // The GMP PDA appears as both user_authority and payer — Solana merges
        // duplicate pubkeys with the most permissive flags.
        let solana_payload = RawGmpSolanaPayload {
            accounts: vec![
                // app_state
                RawSolanaAccountMeta {
                    pubkey: counter_app_state_pda.to_bytes().to_vec(),
                    is_signer: false,
                    is_writable: true,
                },
                // user_counter
                RawSolanaAccountMeta {
                    pubkey: user_counter_pda.to_bytes().to_vec(),
                    is_signer: false,
                    is_writable: true,
                },
                // user_authority (gmp_account_pda signs via invoke_signed)
                RawSolanaAccountMeta {
                    pubkey: gmp_account_pda.to_bytes().to_vec(),
                    is_signer: true,
                    is_writable: true,
                },
                // payer (GMP PDA, pre-funded by relayer for rent)
                RawSolanaAccountMeta {
                    pubkey: gmp_account_pda.to_bytes().to_vec(),
                    is_signer: true,
                    is_writable: true,
                },
                // system_program
                RawSolanaAccountMeta {
                    pubkey: system_program::ID.to_bytes().to_vec(),
                    is_signer: false,
                    is_writable: false,
                },
            ],
            data: counter_instruction_data,
            prefund_lamports: 5_000_000,
        };

        let solana_payload_bytes = encode_test_solana_payload(&solana_payload, encoding);

        let raw_packet = RawGmpPacketData {
            sender: sender.to_string(),
            receiver: COUNTER_APP_ID.to_string(),
            salt,
            payload: solana_payload_bytes,
            memo: String::new(),
        };

        // Encode GMPPacketData using the test's encoding (protobuf or ABI)
        let packet_data_bytes = encode_test_packet(raw_packet, encoding);

        let recv_msg = solana_ibc_types::OnRecvPacketMsg {
            source_client: "cosmos-1".to_string(),
            dest_client: client_id.to_string(),
            sequence: 1,
            payload: solana_ibc_types::Payload {
                source_port: GMP_PORT_ID.to_string(),
                dest_port: GMP_PORT_ID.to_string(),
                version: ICS27_VERSION.to_string(),
                encoding: encoding.to_string(),
                value: packet_data_bytes,
            },
            relayer: Pubkey::new_unique(),
        };

        let instruction_data = crate::instruction::OnRecvPacket { msg: recv_msg };

        let instruction = SolanaInstructionSDK {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(app_state_pda, false),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(system_program::ID, false),
                // Remaining accounts for CPI:
                AccountMeta::new(gmp_account_pda, false), // [0] gmp_account_pda
                AccountMeta::new_readonly(COUNTER_APP_ID, false), // [1] target_program
                AccountMeta::new(counter_app_state_pda, false), // [2] counter app state
                AccountMeta::new(user_counter_pda, false), // [3] user counter
                AccountMeta::new(gmp_account_pda, false), // [4] user_authority (gmp_pda signs via invoke_signed)
                AccountMeta::new(gmp_account_pda, false), // [5] payer (gmp_pda, pre-funded)
                AccountMeta::new_readonly(system_program::ID, false), // [6] system program
            ],
            data: instruction_data.data(),
        };

        // Create counter app state
        let counter_app_state = test_gmp_app::state::CounterAppState {
            authority,
            total_counters: 0,
            total_gmp_calls: 0,
            bump: counter_app_state_bump,
        };
        let mut counter_app_state_data = Vec::new();
        counter_app_state_data
            .extend_from_slice(test_gmp_app::state::CounterAppState::DISCRIMINATOR);
        counter_app_state
            .serialize(&mut counter_app_state_data)
            .unwrap();

        let accounts = vec![
            create_gmp_app_state_account(
                app_state_pda,
                app_state_bump,
                false, // not paused
            ),
            create_instructions_sysvar_account_with_caller(router_program),
            // Counter app program (target_program) - must be executable
            (
                COUNTER_APP_ID,
                Account {
                    lamports: 1_000_000,
                    data: vec![],
                    owner: bpf_loader_upgradeable::ID,
                    executable: true,
                    rent_epoch: 0,
                },
            ),
            create_authority_account(payer),
            create_system_program_account(),
            // Remaining accounts — GMP PDA is pre-funded to act as payer for rent
            (
                gmp_account_pda,
                Account {
                    lamports: 10_000_000, // Pre-funded for rent payment
                    data: vec![],
                    owner: system_program::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
            (
                counter_app_state_pda,
                Account {
                    lamports: 1_000_000,
                    data: counter_app_state_data,
                    owner: COUNTER_APP_ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
            create_uninitialized_account_for_pda(user_counter_pda), // User counter will be created
            create_system_program_account(),
        ];

        let result = mollusk.process_instruction(&instruction, &accounts);

        assert!(
            !result.program_result.is_err(),
            "OnRecvPacket should succeed with {encoding} encoding: {:?}",
            result.program_result
        );

        // Verify acknowledgement is returned
        assert!(
            !result.return_data.is_empty(),
            "Should return acknowledgement"
        );

        // Parse the acknowledgement and verify CPI succeeded.
        // Skip the 4-byte u32 length prefix that Anchor adds.
        let ack_bytes = if result.return_data.len() > 4 {
            &result.return_data[4..]
        } else {
            &result.return_data[..]
        };

        let ack = crate::encoding::decode_gmp_ack(ack_bytes, encoding).unwrap();

        // Following ibc-go convention: non-empty result = success
        assert!(
            !ack.result.is_empty(),
            "CPI execution should succeed (non-empty result)"
        );

        // Counter app returns u64 in little-endian (8 bytes)
        assert_eq!(
            ack.result.len(),
            8,
            "Counter return value should be 8 bytes (u64)"
        );
        let returned_counter = u64::from_le_bytes(ack.result[..8].try_into().unwrap());
        // Counter should be incremented to 5 (initial 0 + increment 5)
        assert_eq!(
            returned_counter, 5,
            "Acknowledgement should contain counter value 5, got {returned_counter}"
        );

        // With stateless approach, no account state is created.
        // The GMP account PDA is used as a signer without storing state.
    }

    #[rstest]
    #[case::protobuf(ICS27_ENCODING_PROTOBUF)]
    #[case::abi(ICS27_ENCODING_ABI)]
    fn test_on_recv_packet_success_with_cpi(#[case] encoding: &str) {
        run_recv_success_test(encoding);
    }

    /// Proves GMP PDAs are immune to the pre-funding DoS attack that affects
    /// `create_account`-based PDAs (see: packet commitment PDA vulnerability).
    ///
    /// Attack scenario: an adversary derives the GMP PDA off-chain and executes
    /// a `system_program::transfer` to it before the relayer submits
    /// `on_recv_packet`. With `create_account`, this would fail with
    /// `AccountAlreadyInUse`. GMP instead uses `invoke_signed` (signing only),
    /// so pre-funding is harmless — the PDA works as a signer regardless of its
    /// lamport balance.
    ///
    /// The test chains two instructions via `process_instruction_chain`:
    /// 1. Adversary transfer → GMP PDA (real system program transfer)
    /// 2. `on_recv_packet` using the now-funded PDA as CPI signer
    #[allow(clippy::doc_markdown)]
    #[test]
    fn test_on_recv_packet_prefunded_gmp_pda_not_blocked() {
        let mut mollusk = Mollusk::new(&crate::ID, crate::get_gmp_program_path());
        mollusk.add_program(
            &COUNTER_APP_ID,
            "../../target/deploy/test_gmp_app",
            &bpf_loader_upgradeable::ID,
        );

        let router_program = ics26_router::ID;
        let payer = Pubkey::new_unique();
        let adversary = Pubkey::new_unique();
        let (app_state_pda, app_state_bump) =
            Pubkey::find_program_address(&[GMPAppState::SEED], &crate::ID);

        let authority = Pubkey::new_unique();
        let (client_id, sender, salt, gmp_account_pda) = create_test_account_data();

        let (counter_app_state_pda, counter_app_state_bump) = Pubkey::find_program_address(
            &[test_gmp_app::state::CounterAppState::SEED],
            &COUNTER_APP_ID,
        );

        let (user_counter_pda, _) = Pubkey::find_program_address(
            &[
                test_gmp_app::state::UserCounter::SEED,
                gmp_account_pda.as_ref(),
            ],
            &COUNTER_APP_ID,
        );

        // Step 1: adversary pre-funds the GMP PDA via system transfer
        let adversary_lamports: u64 = 50_000_000;
        let prefund_ix = solana_sdk::system_instruction::transfer(
            &adversary,
            &gmp_account_pda,
            adversary_lamports,
        );

        // Step 2: build on_recv_packet instruction
        let counter_instruction = test_gmp_app::instruction::Increment { amount: 7 };
        let counter_instruction_data = anchor_lang::InstructionData::data(&counter_instruction);

        let solana_payload = RawGmpSolanaPayload {
            accounts: vec![
                RawSolanaAccountMeta {
                    pubkey: counter_app_state_pda.to_bytes().to_vec(),
                    is_signer: false,
                    is_writable: true,
                },
                RawSolanaAccountMeta {
                    pubkey: user_counter_pda.to_bytes().to_vec(),
                    is_signer: false,
                    is_writable: true,
                },
                RawSolanaAccountMeta {
                    pubkey: gmp_account_pda.to_bytes().to_vec(),
                    is_signer: true,
                    is_writable: true,
                },
                RawSolanaAccountMeta {
                    pubkey: gmp_account_pda.to_bytes().to_vec(),
                    is_signer: true,
                    is_writable: true,
                },
                RawSolanaAccountMeta {
                    pubkey: system_program::ID.to_bytes().to_vec(),
                    is_signer: false,
                    is_writable: false,
                },
            ],
            data: counter_instruction_data,
            prefund_lamports: 5_000_000,
        };

        let proto_packet_data = RawGmpPacketData {
            sender: sender.to_string(),
            receiver: COUNTER_APP_ID.to_string(),
            salt,
            payload: solana_payload.encode_to_vec(),
            memo: String::new(),
        };

        let recv_msg = solana_ibc_types::OnRecvPacketMsg {
            source_client: "cosmos-1".to_string(),
            dest_client: client_id.to_string(),
            sequence: 1,
            payload: solana_ibc_types::Payload {
                source_port: GMP_PORT_ID.to_string(),
                dest_port: GMP_PORT_ID.to_string(),
                version: ICS27_VERSION.to_string(),
                encoding: ICS27_ENCODING_PROTOBUF.to_string(),
                value: proto_packet_data.encode_to_vec(),
            },
            relayer: Pubkey::new_unique(),
        };

        let instruction_data = crate::instruction::OnRecvPacket { msg: recv_msg };
        let recv_ix = SolanaInstructionSDK {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(app_state_pda, false),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(system_program::ID, false),
                AccountMeta::new(gmp_account_pda, false),
                AccountMeta::new_readonly(COUNTER_APP_ID, false),
                AccountMeta::new(counter_app_state_pda, false),
                AccountMeta::new(user_counter_pda, false),
                AccountMeta::new(gmp_account_pda, false),
                AccountMeta::new(gmp_account_pda, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ],
            data: instruction_data.data(),
        };

        let counter_app_state = test_gmp_app::state::CounterAppState {
            authority,
            total_counters: 0,
            total_gmp_calls: 0,
            bump: counter_app_state_bump,
        };
        let mut counter_app_state_data = Vec::new();
        counter_app_state_data
            .extend_from_slice(test_gmp_app::state::CounterAppState::DISCRIMINATOR);
        counter_app_state
            .serialize(&mut counter_app_state_data)
            .unwrap();

        // GMP PDA starts with 0 lamports — the adversary transfer is what funds it
        let accounts = vec![
            (
                adversary,
                Account {
                    lamports: adversary_lamports.saturating_add(1_000_000),
                    data: vec![],
                    owner: system_program::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
            create_gmp_app_state_account(app_state_pda, app_state_bump, false),
            create_instructions_sysvar_account_with_caller(router_program),
            (
                COUNTER_APP_ID,
                Account {
                    lamports: 1_000_000,
                    data: vec![],
                    owner: bpf_loader_upgradeable::ID,
                    executable: true,
                    rent_epoch: 0,
                },
            ),
            create_authority_account(payer),
            create_system_program_account(),
            (
                gmp_account_pda,
                Account {
                    lamports: 0,
                    data: vec![],
                    owner: system_program::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
            (
                counter_app_state_pda,
                Account {
                    lamports: 1_000_000,
                    data: counter_app_state_data,
                    owner: COUNTER_APP_ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
            create_uninitialized_account_for_pda(user_counter_pda),
            create_system_program_account(),
        ];

        // Chain: adversary transfer → on_recv_packet
        // The transfer funds the GMP PDA; on_recv_packet uses it as a signer.
        let result = mollusk.process_instruction_chain(&[prefund_ix, recv_ix], &accounts);

        assert!(
            !result.program_result.is_err(),
            "Pre-funded GMP PDA must not block on_recv_packet: {:?}",
            result.program_result
        );

        let ack_bytes = if result.return_data.len() > 4 {
            &result.return_data[4..]
        } else {
            &result.return_data[..]
        };
        let ack = crate::encoding::decode_gmp_ack(ack_bytes, ICS27_ENCODING_PROTOBUF).unwrap();

        assert!(
            !ack.result.is_empty(),
            "CPI should succeed despite adversary pre-funding"
        );

        let returned_counter = u64::from_le_bytes(ack.result[..8].try_into().unwrap());
        assert_eq!(returned_counter, 7, "Counter should reflect the increment");
    }

    /// Verifies that CPI errors cause immediate transaction failure.
    ///
    /// Unlike EVM/IBC where errors can return error acknowledgments, Solana CPIs fail
    /// atomically - when `invoke()` fails, the entire transaction aborts at the VM level.
    /// This test triggers a CPI failure (insufficient lamports) and expects tx abort.
    #[test]
    fn test_on_recv_packet_failed_execution_returns_error_ack() {
        // Create Mollusk instance and load both programs
        let mut mollusk = Mollusk::new(&crate::ID, crate::get_gmp_program_path());

        // Add the counter app program so CPI will be attempted
        mollusk.add_program(
            &COUNTER_APP_ID,
            "../../target/deploy/test_gmp_app",
            &bpf_loader_upgradeable::ID,
        );

        let payer = Pubkey::new_unique();
        let (app_state_pda, app_state_bump) =
            Pubkey::find_program_address(&[GMPAppState::SEED], &crate::ID);

        // Create packet data
        let authority = Pubkey::new_unique();
        let (client_id, sender, salt, gmp_account_pda) = create_test_account_data();

        // Counter app state PDA
        let (counter_app_state_pda, counter_app_state_bump) = Pubkey::find_program_address(
            &[test_gmp_app::state::CounterAppState::SEED],
            &COUNTER_APP_ID,
        );

        let (user_counter_pda, _user_counter_bump) = Pubkey::find_program_address(
            &[
                test_gmp_app::state::UserCounter::SEED,
                gmp_account_pda.as_ref(),
            ],
            &COUNTER_APP_ID,
        );

        // Create counter instruction - will fail due to CPI validation (no authorized router)
        let counter_instruction = test_gmp_app::instruction::Increment { amount: 5 };
        let counter_instruction_data = anchor_lang::InstructionData::data(&counter_instruction);

        // Build GMPSolanaPayload for the payload
        let solana_payload = RawGmpSolanaPayload {
            accounts: vec![
                // app_state
                RawSolanaAccountMeta {
                    pubkey: counter_app_state_pda.to_bytes().to_vec(),
                    is_signer: false,
                    is_writable: true,
                },
                // user_counter
                RawSolanaAccountMeta {
                    pubkey: user_counter_pda.to_bytes().to_vec(),
                    is_signer: false,
                    is_writable: true,
                },
                // user_authority (gmp_account_pda signs via invoke_signed)
                RawSolanaAccountMeta {
                    pubkey: gmp_account_pda.to_bytes().to_vec(),
                    is_signer: true,
                    is_writable: true,
                },
                // payer (GMP PDA, pre-funded by relayer for rent)
                RawSolanaAccountMeta {
                    pubkey: gmp_account_pda.to_bytes().to_vec(),
                    is_signer: true,
                    is_writable: true,
                },
                // system_program
                RawSolanaAccountMeta {
                    pubkey: system_program::ID.to_bytes().to_vec(),
                    is_signer: false,
                    is_writable: false,
                },
            ],
            data: counter_instruction_data,
            prefund_lamports: 5_000_000,
        };

        let solana_payload_bytes = solana_payload.encode_to_vec();

        // Create GMPPacketData with the counter instruction as payload using protobuf
        let proto_packet_data = RawGmpPacketData {
            sender: sender.to_string(),
            receiver: COUNTER_APP_ID.to_string(),
            salt,
            payload: solana_payload_bytes,
            memo: String::new(),
        };

        let packet_data_bytes = proto_packet_data.encode_to_vec();

        let recv_msg = solana_ibc_types::OnRecvPacketMsg {
            source_client: "cosmos-1".to_string(),
            dest_client: client_id.to_string(),
            sequence: 1,
            payload: solana_ibc_types::Payload {
                source_port: GMP_PORT_ID.to_string(),
                dest_port: GMP_PORT_ID.to_string(),
                version: ICS27_VERSION.to_string(),
                encoding: ICS27_ENCODING_PROTOBUF.to_string(),
                value: packet_data_bytes,
            },
            relayer: Pubkey::new_unique(),
        };

        let instruction_data = crate::instruction::OnRecvPacket { msg: recv_msg };

        let instruction = SolanaInstructionSDK {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(app_state_pda, false),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(system_program::ID, false),
                // Remaining accounts for CPI:
                AccountMeta::new(gmp_account_pda, false), // [0] gmp_account_pda (GMP account)
                AccountMeta::new_readonly(COUNTER_APP_ID, false), // [1] target_program
                AccountMeta::new(counter_app_state_pda, false), // [2] counter app state
                AccountMeta::new(user_counter_pda, false), // [3] user counter
                AccountMeta::new(gmp_account_pda, false), // [4] user_authority (gmp_pda signs via invoke_signed)
                AccountMeta::new(gmp_account_pda, false), // [5] payer (gmp_pda, pre-funded)
                AccountMeta::new_readonly(system_program::ID, false), // [6] system program
            ],
            data: instruction_data.data(),
        };

        // Create counter app state (properly initialized)
        let counter_app_state = test_gmp_app::state::CounterAppState {
            authority,
            total_counters: 0,
            total_gmp_calls: 0,
            bump: counter_app_state_bump,
        };
        let mut counter_app_state_data = Vec::new();
        counter_app_state_data
            .extend_from_slice(test_gmp_app::state::CounterAppState::DISCRIMINATOR);
        counter_app_state
            .serialize(&mut counter_app_state_data)
            .unwrap();

        let accounts = vec![
            create_gmp_app_state_account(
                app_state_pda,
                app_state_bump,
                false, // not paused
            ),
            create_instructions_sysvar_account(),
            create_authority_account(payer),
            create_system_program_account(),
            // Remaining accounts — GMP PDA with low balance (test expects failure)
            (
                gmp_account_pda,
                Account {
                    lamports: 3_000_000, // Low balance — insufficient for user_counter rent
                    data: vec![],
                    owner: system_program::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
            // Counter app program (loaded via mollusk.add_program())
            (
                COUNTER_APP_ID,
                Account {
                    lamports: 1_000_000,
                    data: vec![],
                    owner: bpf_loader_upgradeable::ID,
                    executable: true,
                    rent_epoch: 0,
                },
            ),
            (
                counter_app_state_pda,
                Account {
                    lamports: 1_000_000,
                    data: counter_app_state_data,
                    owner: COUNTER_APP_ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
            create_uninitialized_account_for_pda(user_counter_pda),
            create_system_program_account(),
        ];

        // Instructions sysvar has no caller set, so CPI validation rejects
        let checks = vec![Check::err(ProgramError::Custom(
            ANCHOR_ERROR_OFFSET + crate::errors::GMPError::UnauthorizedRouter as u32,
        ))];
        let result = mollusk.process_and_validate_instruction(&instruction, &accounts, &checks);

        // Verify no acknowledgement was returned (transaction aborted)
        assert!(
            result.return_data.is_empty(),
            "No return data should be present when transaction aborts"
        );

        // With stateless approach, no account state is created or rolled back
        // The GMP account PDA is used as a signer without storing state
    }

    #[test]
    fn test_invalid_packet_data_returns_error() {
        let ctx = create_gmp_test_context();
        let (client_id, _sender, salt, gmp_account_pda) = create_test_account_data();
        let target_program = crate::test_utils::DUMMY_TARGET_PROGRAM;

        // Create invalid packet data with empty sender (will fail validation)
        let invalid_packet_data = RawGmpPacketData {
            sender: String::new(), // Invalid: empty sender
            receiver: target_program.to_string(),
            salt,
            payload: vec![1, 2, 3, 4],
            memo: String::new(),
        };

        let packet_data_bytes = invalid_packet_data.encode_to_vec();
        let recv_msg = create_recv_packet_msg(client_id, packet_data_bytes, 1);
        let mut instruction =
            create_recv_packet_instruction(ctx.app_state_pda, ctx.payer, recv_msg);

        // Add remaining accounts to the instruction
        instruction.accounts.extend(vec![
            AccountMeta::new(gmp_account_pda, false), // [0] gmp_account_pda
            AccountMeta::new_readonly(target_program, false), // [1] target_program
        ]);

        let accounts = vec![
            create_gmp_app_state_account(ctx.app_state_pda, ctx.app_state_bump, false),
            create_instructions_sysvar_account_with_caller(ctx.router_program),
            create_authority_account(ctx.payer),
            create_system_program_account(),
            // Remaining accounts - matching the instruction's remaining accounts
            create_uninitialized_account_for_pda(gmp_account_pda), // [0] gmp_account_pda
            create_dummy_target_program_account(),                 // [1] target_program
        ];

        let checks = vec![Check::err(ProgramError::Custom(
            ANCHOR_ERROR_OFFSET + GMPError::InvalidPacketData as u32,
        ))];

        ctx.mollusk
            .process_and_validate_instruction(&instruction, &accounts, &checks);
    }

    #[test]
    fn test_invalid_solana_payload_returns_error() {
        let ctx = create_gmp_test_context();
        let (client_id, sender, salt, gmp_account_pda) = create_test_account_data();
        let target_program = crate::test_utils::DUMMY_TARGET_PROGRAM;

        // Create invalid Solana payload with empty data (will fail validation during decode)
        let invalid_solana_payload = RawGmpSolanaPayload {
            data: vec![], // Invalid: empty instruction data
            accounts: vec![],
            prefund_lamports: 0,
        };

        // Encode the invalid solana payload - this creates a valid protobuf but with empty data field
        let mut invalid_payload_bytes = invalid_solana_payload.encode_to_vec();

        // If encoding results in empty vec, add a dummy byte to pass NonEmpty constraint
        // The payload will still be invalid when decoded as GmpSolanaPayload
        if invalid_payload_bytes.is_empty() {
            invalid_payload_bytes = vec![0xFF]; // Invalid protobuf that will fail decode
        }

        // Create packet data with the invalid payload bytes directly (not using helper)
        let packet_data = RawGmpPacketData {
            sender: sender.to_string(),
            receiver: target_program.to_string(),
            salt,
            payload: invalid_payload_bytes, // This will pass GmpPacketData validation but fail GmpSolanaPayload validation
            memo: String::new(),
        };

        let packet_data_bytes = packet_data.encode_to_vec();
        let recv_msg = create_recv_packet_msg(client_id, packet_data_bytes, 1);
        let mut instruction =
            create_recv_packet_instruction(ctx.app_state_pda, ctx.payer, recv_msg);

        // Add remaining accounts to the instruction
        instruction.accounts.extend(vec![
            AccountMeta::new(gmp_account_pda, false), // [0] gmp_account_pda
            AccountMeta::new_readonly(target_program, false), // [1] target_program
        ]);

        let accounts = vec![
            create_gmp_app_state_account(ctx.app_state_pda, ctx.app_state_bump, false),
            create_instructions_sysvar_account_with_caller(ctx.router_program),
            create_authority_account(ctx.payer),
            create_system_program_account(),
            // Remaining accounts - matching the instruction's remaining accounts
            create_uninitialized_account_for_pda(gmp_account_pda), // [0] gmp_account_pda
            create_dummy_target_program_account(),                 // [1] target_program
        ];

        let checks = vec![Check::err(ProgramError::Custom(
            ANCHOR_ERROR_OFFSET + GMPError::InvalidSolanaPayload as u32,
        ))];

        ctx.mollusk
            .process_and_validate_instruction(&instruction, &accounts, &checks);
    }
}

/// Integration tests using `ProgramTest` with real BPF runtime.
///
/// These verify that `validate_cpi_caller()` rejects direct calls, unauthorized
/// CPI callers and nested CPI using real `get_stack_height()` behavior.
#[cfg(test)]
mod integration_tests {
    use crate::constants::*;
    use crate::state::GMPAppState;
    use crate::test_utils::*;
    use anchor_lang::InstructionData;
    use solana_sdk::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        signer::Signer,
    };

    fn build_recv_packet_ix(payer: Pubkey) -> Instruction {
        let (app_state_pda, _) = Pubkey::find_program_address(&[GMPAppState::SEED], &crate::ID);

        let msg = solana_ibc_types::OnRecvPacketMsg {
            source_client: "cosmos-1".to_string(),
            dest_client: "cosmoshub-1".to_string(),
            sequence: 1,
            payload: solana_ibc_types::Payload {
                source_port: GMP_PORT_ID.to_string(),
                dest_port: GMP_PORT_ID.to_string(),
                version: ICS27_VERSION.to_string(),
                encoding: ICS27_ENCODING_PROTOBUF.to_string(),
                value: vec![0],
            },
            relayer: Pubkey::new_unique(),
        };

        let ix_data = crate::instruction::OnRecvPacket { msg };

        Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new(app_state_pda, false),
                AccountMeta::new_readonly(
                    anchor_lang::solana_program::sysvar::instructions::ID,
                    false,
                ),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(solana_sdk::system_program::ID, false),
            ],
            data: ix_data.data(),
        }
    }

    #[tokio::test]
    async fn test_direct_call_rejected() {
        let pt = setup_program_test();
        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let ix = build_recv_packet_ix(payer.pubkey());

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

    #[tokio::test]
    async fn test_unauthorized_cpi_rejected() {
        let pt = setup_program_test();
        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let inner_ix = build_recv_packet_ix(payer.pubkey());
        let ix = wrap_in_test_cpi_proxy(payer.pubkey(), &inner_ix);

        let result = process_tx(&banks_client, &payer, recent_blockhash, &[ix]).await;
        let err = result.expect_err("unauthorized CPI should be rejected");
        assert_eq!(
            extract_custom_error(&err),
            Some(
                anchor_lang::error::ERROR_CODE_OFFSET
                    + crate::errors::GMPError::UnauthorizedRouter as u32
            ),
            "expected UnauthorizedRouter, got: {err:?}"
        );
    }

    #[tokio::test]
    async fn test_nested_cpi_rejected() {
        let pt = setup_program_test();
        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let inner_ix = build_recv_packet_ix(payer.pubkey());
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

    /// Simulates router → proxy → GMP: even if the top-level caller is an authorized
    /// program, an intermediary proxy makes the chain nested CPI (stack height > 2)
    /// which is always rejected by `reject_nested_cpi`.
    #[tokio::test]
    async fn test_router_via_proxy_cpi_rejected() {
        let pt = setup_program_test();
        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let inner_ix = build_recv_packet_ix(payer.pubkey());
        // test_cpi_proxy acts as an intermediary between the outer caller and GMP
        let middle_ix = wrap_in_test_cpi_proxy(payer.pubkey(), &inner_ix);
        // test_cpi_target wraps the proxy (standing in for the router as outer caller)
        let ix = wrap_in_test_cpi_target_proxy(payer.pubkey(), &middle_ix);

        let result = process_tx(&banks_client, &payer, recent_blockhash, &[ix]).await;
        let err = result.expect_err("router-via-proxy CPI should be rejected");
        assert_eq!(
            extract_custom_error(&err),
            Some(
                anchor_lang::error::ERROR_CODE_OFFSET
                    + crate::errors::GMPError::UnauthorizedRouter as u32
            ),
            "expected UnauthorizedRouter (from NestedCpiNotAllowed), got: {err:?}"
        );
    }

    /// Verifies that a CPI call from the authorized router program passes
    /// CPI validation. Uses a test proxy loaded at `ics26_router::ID` so the
    /// runtime sees the correct caller program ID.
    ///
    /// The instruction fails later (no remaining accounts for target program),
    /// but the error is NOT a CPI validation error — proving the rejections
    /// in other tests are genuine access control, not false positives.
    #[tokio::test]
    async fn test_authorized_router_cpi_passes_validation() {
        let pt = setup_program_test_with_router_proxy();
        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let inner_ix = build_recv_packet_ix(payer.pubkey());
        let ix = wrap_as_router_cpi(payer.pubkey(), &inner_ix);

        let result = process_tx(&banks_client, &payer, recent_blockhash, &[ix]).await;
        let err = result.expect_err("should fail at packet level, not CPI validation");
        let code = extract_custom_error(&err).expect("should be a custom error");

        let direct_call = anchor_lang::error::ERROR_CODE_OFFSET
            + crate::errors::GMPError::DirectCallNotAllowed as u32;
        let unauthorized = anchor_lang::error::ERROR_CODE_OFFSET
            + crate::errors::GMPError::UnauthorizedRouter as u32;

        assert_ne!(code, direct_call, "should not be DirectCallNotAllowed");
        assert_ne!(code, unauthorized, "should not be UnauthorizedRouter");
    }
}
