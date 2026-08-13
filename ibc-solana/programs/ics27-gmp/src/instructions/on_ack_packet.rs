// SPDX-License-Identifier: Apache-2.0

use crate::errors::GMPError;
use crate::events::GMPCallAcknowledgment;
use crate::state::{GMPAppState, GMPCallResult, GMPCallResultAccount};
use anchor_lang::prelude::*;
use solana_ibc_proto::GmpPacketData;

/// Processes an IBC packet acknowledgement received from the router via CPI.
/// Creates a `GMPCallResultAccount` PDA to store the acknowledgement outcome.
#[derive(Accounts)]
#[instruction(msg: solana_ibc_types::OnAcknowledgementPacketMsg)]
pub struct OnAckPacket<'info> {
    /// GMP program's global configuration PDA. Must not be paused.
    #[account(
        seeds = [GMPAppState::SEED],
        bump = app_state.bump,
        constraint = !app_state.paused @ GMPError::AppPaused
    )]
    pub app_state: Account<'info, GMPAppState>,

    /// Instructions sysvar used to verify the CPI caller is the authorized router.
    /// CHECK: Address constraint verifies this is the instructions sysvar
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instruction_sysvar: AccountInfo<'info>,

    /// Fee payer that funds the `result_account` creation.
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Solana system program used for `result_account` allocation.
    pub system_program: Program<'info, System>,

    /// Newly created PDA that stores the acknowledgement commitment and metadata,
    /// keyed by `source_client` and `sequence`.
    #[account(
        init,
        payer = payer,
        space = 8 + GMPCallResultAccount::INIT_SPACE,
        seeds = [GMPCallResult::SEED, msg.source_client.as_bytes(), &msg.sequence.to_le_bytes()],
        bump,
    )]
    pub result_account: Account<'info, GMPCallResultAccount>,
}

pub fn on_acknowledgement_packet(
    ctx: Context<OnAckPacket>,
    msg: solana_ibc_types::OnAcknowledgementPacketMsg,
) -> Result<()> {
    solana_ibc_types::validate_cpi_caller(
        &ctx.accounts.instruction_sysvar,
        &ics26_router::ID,
        &crate::ID,
    )
    .map_err(GMPError::from)?;

    let raw_packet_data =
        crate::encoding::decode_gmp_packet(&msg.payload.value, &msg.payload.encoding)?;
    let packet_data = GmpPacketData::try_from(raw_packet_data).map_err(|e| {
        msg!("GMP packet validation failed: {}", e);
        GMPError::InvalidPacketData
    })?;

    let clock = Clock::get()?;
    let sender: Pubkey = packet_data
        .sender
        .as_ref()
        .parse()
        .map_err(|_| GMPError::InvalidSender)?;

    let result = &mut ctx.accounts.result_account;
    result.init_acknowledged(msg, sender, clock.unix_timestamp, ctx.bumps.result_account);

    emit!(GMPCallAcknowledgment {
        source_client: result.source_client.clone(),
        sequence: result.sequence,
        sender,
        result_pda: result.key(),
        timestamp: result.result_timestamp,
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::constants::{
        GMP_PORT_ID, ICS27_ENCODING_ABI, ICS27_ENCODING_PROTOBUF, ICS27_VERSION,
    };
    use crate::encoding::encode_gmp_packet;
    use crate::state::{GMPAppState, GMPCallResult, GMPCallResultAccount};
    use crate::test_utils::{
        create_fake_instructions_sysvar_account, create_gmp_app_state_account,
        create_gmp_packet_data, create_instructions_sysvar_account_with_caller,
        create_payer_account, create_system_program_account, create_uninitialized_account_for_pda,
        ANCHOR_ERROR_OFFSET,
    };
    use anchor_lang::{AnchorDeserialize, InstructionData};
    use mollusk_svm::result::Check;
    use mollusk_svm::Mollusk;
    use rstest::rstest;
    use solana_ibc_proto::ProstMessage;
    use solana_ibc_types::{CallResultStatus, GmpPacketData};
    use solana_sdk::program_error::ProgramError;
    use solana_sdk::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
    };

    const TEST_SOURCE_CLIENT: &str = "cosmoshub-1";
    const TEST_SEQUENCE: u64 = 1;

    fn create_test_ack_msg() -> solana_ibc_types::OnAcknowledgementPacketMsg {
        solana_ibc_types::OnAcknowledgementPacketMsg {
            source_client: TEST_SOURCE_CLIENT.to_string(),
            dest_client: "solana-1".to_string(),
            sequence: TEST_SEQUENCE,
            payload: solana_ibc_types::Payload {
                source_port: GMP_PORT_ID.to_string(),
                dest_port: GMP_PORT_ID.to_string(),
                version: ICS27_VERSION.to_string(),
                encoding: ICS27_ENCODING_PROTOBUF.to_string(),
                value: vec![],
            },
            acknowledgement: vec![1, 2, 3],
            relayer: Pubkey::new_unique(),
        }
    }

    fn derive_result_pda() -> (Pubkey, u8) {
        GMPCallResult::pda(TEST_SOURCE_CLIENT, TEST_SEQUENCE, &crate::ID)
    }

    fn create_ack_instruction(
        app_state_pda: Pubkey,
        result_account_pda: Pubkey,
        payer: Pubkey,
    ) -> Instruction {
        let instruction_data = crate::instruction::OnAcknowledgementPacket {
            msg: create_test_ack_msg(),
        };

        Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new_readonly(app_state_pda, false),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(solana_sdk::system_program::ID, false),
                AccountMeta::new(result_account_pda, false),
            ],
            data: instruction_data.data(),
        }
    }

    #[test]
    fn test_on_ack_packet_app_paused() {
        let mollusk = Mollusk::new(&crate::ID, crate::get_gmp_program_path());

        let router_program = ics26_router::ID;
        let payer = Pubkey::new_unique();
        let (app_state_pda, app_state_bump) =
            Pubkey::find_program_address(&[GMPAppState::SEED], &crate::ID);
        let (result_pda, _) = derive_result_pda();

        let instruction = create_ack_instruction(app_state_pda, result_pda, payer);

        let accounts = vec![
            create_gmp_app_state_account(app_state_pda, app_state_bump, true),
            create_instructions_sysvar_account_with_caller(router_program),
            create_payer_account(payer),
            create_system_program_account(),
            create_uninitialized_account_for_pda(result_pda),
        ];

        let checks = vec![Check::err(ProgramError::Custom(
            ANCHOR_ERROR_OFFSET + crate::errors::GMPError::AppPaused as u32,
        ))];

        mollusk.process_and_validate_instruction(&instruction, &accounts, &checks);
    }

    #[test]
    fn test_on_ack_packet_invalid_app_state_pda() {
        let mollusk = Mollusk::new(&crate::ID, crate::get_gmp_program_path());

        let router_program = ics26_router::ID;
        let payer = Pubkey::new_unique();
        let (result_pda, _) = derive_result_pda();

        let (_correct_app_state_pda, _correct_bump) =
            Pubkey::find_program_address(&[GMPAppState::SEED], &crate::ID);

        // Use wrong PDA
        let wrong_app_state_pda = Pubkey::new_unique();

        let ack_msg = solana_ibc_types::OnAcknowledgementPacketMsg {
            source_client: TEST_SOURCE_CLIENT.to_string(),
            dest_client: "solana-1".to_string(),
            sequence: TEST_SEQUENCE,
            payload: solana_ibc_types::Payload {
                source_port: GMP_PORT_ID.to_string(),
                dest_port: GMP_PORT_ID.to_string(),
                version: ICS27_VERSION.to_string(),
                encoding: ICS27_ENCODING_PROTOBUF.to_string(),
                value: vec![],
            },
            acknowledgement: vec![1, 2, 3],
            relayer: Pubkey::new_unique(),
        };

        let instruction_data = crate::instruction::OnAcknowledgementPacket { msg: ack_msg };

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new_readonly(wrong_app_state_pda, false), // Wrong PDA!
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(solana_sdk::system_program::ID, false),
                AccountMeta::new(result_pda, false),
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
            create_payer_account(payer),
            create_system_program_account(),
            create_uninitialized_account_for_pda(result_pda),
        ];

        let checks = vec![Check::err(ProgramError::Custom(
            anchor_lang::error::ErrorCode::ConstraintSeeds as u32,
        ))];

        mollusk.process_and_validate_instruction(&instruction, &accounts, &checks);
    }

    #[test]
    fn test_on_ack_packet_direct_call_rejected() {
        let mollusk = Mollusk::new(&crate::ID, crate::get_gmp_program_path());

        let payer = Pubkey::new_unique();
        let (app_state_pda, app_state_bump) =
            Pubkey::find_program_address(&[GMPAppState::SEED], &crate::ID);
        let (result_pda, _) = derive_result_pda();

        let instruction = create_ack_instruction(app_state_pda, result_pda, payer);

        let accounts = vec![
            create_gmp_app_state_account(app_state_pda, app_state_bump, false),
            create_instructions_sysvar_account_with_caller(crate::ID), // Direct call
            create_payer_account(payer),
            create_system_program_account(),
            create_uninitialized_account_for_pda(result_pda),
        ];

        let checks = vec![Check::err(ProgramError::Custom(
            ANCHOR_ERROR_OFFSET + crate::errors::GMPError::DirectCallNotAllowed as u32,
        ))];

        mollusk.process_and_validate_instruction(&instruction, &accounts, &checks);
    }

    #[test]
    fn test_on_ack_packet_unauthorized_router() {
        let mollusk = Mollusk::new(&crate::ID, crate::get_gmp_program_path());

        let payer = Pubkey::new_unique();
        let (app_state_pda, app_state_bump) =
            Pubkey::find_program_address(&[GMPAppState::SEED], &crate::ID);
        let (result_pda, _) = derive_result_pda();

        let instruction = create_ack_instruction(app_state_pda, result_pda, payer);

        let unauthorized_program = Pubkey::new_unique();
        let accounts = vec![
            create_gmp_app_state_account(app_state_pda, app_state_bump, false),
            create_instructions_sysvar_account_with_caller(unauthorized_program), // Unauthorized
            create_payer_account(payer),
            create_system_program_account(),
            create_uninitialized_account_for_pda(result_pda),
        ];

        let checks = vec![Check::err(ProgramError::Custom(
            ANCHOR_ERROR_OFFSET + crate::errors::GMPError::UnauthorizedRouter as u32,
        ))];

        mollusk.process_and_validate_instruction(&instruction, &accounts, &checks);
    }

    #[test]
    fn test_on_ack_packet_fake_sysvar_wormhole_attack() {
        let mollusk = Mollusk::new(&crate::ID, crate::get_gmp_program_path());

        let router_program = ics26_router::ID;
        let payer = Pubkey::new_unique();
        let (app_state_pda, app_state_bump) =
            Pubkey::find_program_address(&[GMPAppState::SEED], &crate::ID);
        let (result_pda, _) = derive_result_pda();

        let mut instruction = create_ack_instruction(app_state_pda, result_pda, payer);

        // Simulate Wormhole attack: pass a completely different account with fake sysvar data
        let (fake_sysvar_pubkey, fake_sysvar_account) =
            create_fake_instructions_sysvar_account(router_program);

        // Modify the instruction to reference the fake sysvar (simulating attacker control)
        instruction.accounts[1] = AccountMeta::new_readonly(fake_sysvar_pubkey, false);

        let accounts = vec![
            create_gmp_app_state_account(app_state_pda, app_state_bump, false),
            // Wormhole attack: provide a DIFFERENT account instead of the real sysvar
            (fake_sysvar_pubkey, fake_sysvar_account),
            create_payer_account(payer),
            create_system_program_account(),
            create_uninitialized_account_for_pda(result_pda),
        ];

        // Should be rejected by Anchor's address constraint check
        let checks = vec![Check::err(ProgramError::Custom(
            anchor_lang::error::ErrorCode::ConstraintAddress as u32,
        ))];

        mollusk.process_and_validate_instruction(&instruction, &accounts, &checks);
    }

    #[test]
    fn test_on_ack_packet_invalid_packet_data() {
        let mollusk = Mollusk::new(&crate::ID, crate::get_gmp_program_path());

        let router_program = ics26_router::ID;
        let payer = Pubkey::new_unique();
        let (app_state_pda, app_state_bump) =
            Pubkey::find_program_address(&[GMPAppState::SEED], &crate::ID);
        let (result_pda, _) = derive_result_pda();

        // Create msg with empty payload value (invalid packet data)
        let ack_msg = solana_ibc_types::OnAcknowledgementPacketMsg {
            source_client: TEST_SOURCE_CLIENT.to_string(),
            dest_client: "solana-1".to_string(),
            sequence: TEST_SEQUENCE,
            payload: solana_ibc_types::Payload {
                source_port: GMP_PORT_ID.to_string(),
                dest_port: GMP_PORT_ID.to_string(),
                version: ICS27_VERSION.to_string(),
                encoding: ICS27_ENCODING_PROTOBUF.to_string(),
                value: vec![], // Empty - will fail to decode
            },
            acknowledgement: vec![1, 2, 3],
            relayer: Pubkey::new_unique(),
        };

        let instruction_data = crate::instruction::OnAcknowledgementPacket { msg: ack_msg };

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new_readonly(app_state_pda, false),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(solana_sdk::system_program::ID, false),
                AccountMeta::new(result_pda, false),
            ],
            data: instruction_data.data(),
        };

        let accounts = vec![
            create_gmp_app_state_account(app_state_pda, app_state_bump, false),
            create_instructions_sysvar_account_with_caller(router_program),
            create_payer_account(payer),
            create_system_program_account(),
            create_uninitialized_account_for_pda(result_pda),
        ];

        let checks = vec![Check::err(ProgramError::Custom(
            ANCHOR_ERROR_OFFSET + crate::errors::GMPError::InvalidPacketData as u32,
        ))];

        mollusk.process_and_validate_instruction(&instruction, &accounts, &checks);
    }

    fn encode_test_packet(raw: solana_ibc_proto::RawGmpPacketData, encoding: &str) -> Vec<u8> {
        encode_gmp_packet(GmpPacketData::try_from(raw).unwrap(), encoding).unwrap()
    }

    fn run_ack_success_test(encoding: &str) {
        let mollusk = Mollusk::new(&crate::ID, crate::get_gmp_program_path());

        let router_program = ics26_router::ID;
        let payer = Pubkey::new_unique();
        let (app_state_pda, app_state_bump) =
            Pubkey::find_program_address(&[GMPAppState::SEED], &crate::ID);
        let (result_pda, _) = derive_result_pda();

        let raw_packet_data = create_gmp_packet_data(
            &payer.to_string(),
            "0x1234567890abcdef",
            vec![1, 2, 3],
            vec![4, 5, 6],
        );
        let packet_bytes = encode_test_packet(raw_packet_data, encoding);

        let ack_bytes = vec![1, 2, 3];
        let ack_msg = solana_ibc_types::OnAcknowledgementPacketMsg {
            source_client: TEST_SOURCE_CLIENT.to_string(),
            dest_client: "solana-1".to_string(),
            sequence: TEST_SEQUENCE,
            payload: solana_ibc_types::Payload {
                source_port: GMP_PORT_ID.to_string(),
                dest_port: GMP_PORT_ID.to_string(),
                version: ICS27_VERSION.to_string(),
                encoding: encoding.to_string(),
                value: packet_bytes,
            },
            acknowledgement: ack_bytes.clone(),
            relayer: Pubkey::new_unique(),
        };

        let instruction_data = crate::instruction::OnAcknowledgementPacket { msg: ack_msg };

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new_readonly(app_state_pda, false),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(solana_sdk::system_program::ID, false),
                AccountMeta::new(result_pda, false),
            ],
            data: instruction_data.data(),
        };

        let accounts = vec![
            create_gmp_app_state_account(app_state_pda, app_state_bump, false),
            create_instructions_sysvar_account_with_caller(router_program),
            create_payer_account(payer),
            create_system_program_account(),
            create_uninitialized_account_for_pda(result_pda),
        ];

        let result = mollusk.process_instruction(&instruction, &accounts);
        assert!(
            !result.program_result.is_err(),
            "on_ack_packet should succeed with {encoding} encoding: {:?}",
            result.program_result
        );

        let result_account = result.get_account(&result_pda).unwrap();
        let mut result_data = &result_account.data[crate::constants::DISCRIMINATOR_SIZE..];
        let result_state = GMPCallResultAccount::deserialize(&mut result_data).unwrap();

        assert_eq!(result_state.sender, payer);
        assert_eq!(result_state.sequence, TEST_SEQUENCE);
        assert_eq!(result_state.source_client, TEST_SOURCE_CLIENT);
        assert_eq!(result_state.dest_client, "solana-1");

        let expected_commitment = solana_ibc_types::packet_acknowledgement_commitment_bytes32(
            std::slice::from_ref(&ack_bytes),
        )
        .unwrap();
        assert_eq!(
            result_state.status,
            CallResultStatus::Acknowledgement(expected_commitment)
        );
    }

    #[rstest]
    #[case::protobuf(ICS27_ENCODING_PROTOBUF)]
    #[case::abi(ICS27_ENCODING_ABI)]
    fn test_on_ack_packet_success(#[case] encoding: &str) {
        run_ack_success_test(encoding);
    }

    /// Helper to test ack packet with invalid GMP packet data (expects `InvalidPacketData` error)
    fn assert_ack_packet_invalid_gmp_data(packet_data: solana_ibc_proto::RawGmpPacketData) {
        let mollusk = Mollusk::new(&crate::ID, crate::get_gmp_program_path());

        let router_program = ics26_router::ID;
        let payer = Pubkey::new_unique();
        let (app_state_pda, app_state_bump) =
            Pubkey::find_program_address(&[GMPAppState::SEED], &crate::ID);
        let (result_pda, _) = derive_result_pda();

        let ack_msg = solana_ibc_types::OnAcknowledgementPacketMsg {
            source_client: TEST_SOURCE_CLIENT.to_string(),
            dest_client: "solana-1".to_string(),
            sequence: TEST_SEQUENCE,
            payload: solana_ibc_types::Payload {
                source_port: GMP_PORT_ID.to_string(),
                dest_port: GMP_PORT_ID.to_string(),
                version: ICS27_VERSION.to_string(),
                encoding: ICS27_ENCODING_PROTOBUF.to_string(),
                value: packet_data.encode_to_vec(),
            },
            acknowledgement: vec![1, 2, 3],
            relayer: Pubkey::new_unique(),
        };

        let instruction_data = crate::instruction::OnAcknowledgementPacket { msg: ack_msg };

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new_readonly(app_state_pda, false),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(solana_sdk::system_program::ID, false),
                AccountMeta::new(result_pda, false),
            ],
            data: instruction_data.data(),
        };

        let accounts = vec![
            create_gmp_app_state_account(app_state_pda, app_state_bump, false),
            create_instructions_sysvar_account_with_caller(router_program),
            create_payer_account(payer),
            create_system_program_account(),
            create_uninitialized_account_for_pda(result_pda),
        ];

        let checks = vec![Check::err(ProgramError::Custom(
            ANCHOR_ERROR_OFFSET + crate::errors::GMPError::InvalidPacketData as u32,
        ))];

        mollusk.process_and_validate_instruction(&instruction, &accounts, &checks);
    }

    #[test]
    fn test_on_ack_packet_empty_sender() {
        assert_ack_packet_invalid_gmp_data(solana_ibc_proto::RawGmpPacketData {
            sender: String::new(),
            receiver: "0x1234567890abcdef".to_string(),
            salt: vec![1, 2, 3],
            payload: vec![4, 5, 6],
            memo: String::new(),
        });
    }

    #[test]
    fn test_on_ack_packet_empty_gmp_payload() {
        assert_ack_packet_invalid_gmp_data(solana_ibc_proto::RawGmpPacketData {
            sender: Pubkey::new_unique().to_string(),
            receiver: "0x1234567890abcdef".to_string(),
            salt: vec![1, 2, 3],
            payload: vec![],
            memo: String::new(),
        });
    }

    #[test]
    fn test_on_ack_packet_sender_too_long() {
        assert_ack_packet_invalid_gmp_data(solana_ibc_proto::RawGmpPacketData {
            sender: "x".repeat(solana_ibc_proto::MAX_SENDER_LENGTH + 1),
            receiver: "0x1234567890abcdef".to_string(),
            salt: vec![1, 2, 3],
            payload: vec![4, 5, 6],
            memo: String::new(),
        });
    }

    #[test]
    fn test_on_ack_packet_memo_too_long() {
        assert_ack_packet_invalid_gmp_data(solana_ibc_proto::RawGmpPacketData {
            sender: Pubkey::new_unique().to_string(),
            receiver: "0x1234567890abcdef".to_string(),
            salt: vec![1, 2, 3],
            payload: vec![4, 5, 6],
            memo: "x".repeat(solana_ibc_proto::MAX_MEMO_LENGTH + 1),
        });
    }

    #[test]
    fn test_on_ack_packet_receiver_too_long() {
        assert_ack_packet_invalid_gmp_data(solana_ibc_proto::RawGmpPacketData {
            sender: Pubkey::new_unique().to_string(),
            receiver: "x".repeat(solana_ibc_proto::MAX_RECEIVER_LENGTH + 1),
            salt: vec![1, 2, 3],
            payload: vec![4, 5, 6],
            memo: String::new(),
        });
    }

    #[test]
    fn test_on_ack_packet_invalid_encoding() {
        let mollusk = Mollusk::new(&crate::ID, crate::get_gmp_program_path());

        let router_program = ics26_router::ID;
        let payer = Pubkey::new_unique();
        let (app_state_pda, app_state_bump) =
            Pubkey::find_program_address(&[GMPAppState::SEED], &crate::ID);
        let (result_pda, _) = derive_result_pda();

        let raw_packet_data = create_gmp_packet_data(
            &payer.to_string(),
            "0x1234567890abcdef",
            vec![1, 2, 3],
            vec![4, 5, 6],
        );
        let packet_bytes = raw_packet_data.encode_to_vec();

        let ack_msg = solana_ibc_types::OnAcknowledgementPacketMsg {
            source_client: TEST_SOURCE_CLIENT.to_string(),
            dest_client: "solana-1".to_string(),
            sequence: TEST_SEQUENCE,
            payload: solana_ibc_types::Payload {
                source_port: GMP_PORT_ID.to_string(),
                dest_port: GMP_PORT_ID.to_string(),
                version: ICS27_VERSION.to_string(),
                encoding: "application/json".to_string(),
                value: packet_bytes,
            },
            acknowledgement: vec![1, 2, 3],
            relayer: Pubkey::new_unique(),
        };

        let instruction_data = crate::instruction::OnAcknowledgementPacket { msg: ack_msg };

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new_readonly(app_state_pda, false),
                AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(solana_sdk::system_program::ID, false),
                AccountMeta::new(result_pda, false),
            ],
            data: instruction_data.data(),
        };

        let accounts = vec![
            create_gmp_app_state_account(app_state_pda, app_state_bump, false),
            create_instructions_sysvar_account_with_caller(router_program),
            create_payer_account(payer),
            create_system_program_account(),
            create_uninitialized_account_for_pda(result_pda),
        ];

        let checks = vec![Check::err(ProgramError::Custom(
            ANCHOR_ERROR_OFFSET + crate::errors::GMPError::InvalidEncoding as u32,
        ))];

        mollusk.process_and_validate_instruction(&instruction, &accounts, &checks);
    }

    #[test]
    fn test_on_ack_packet_salt_too_long() {
        assert_ack_packet_invalid_gmp_data(solana_ibc_proto::RawGmpPacketData {
            sender: Pubkey::new_unique().to_string(),
            receiver: "0x1234567890abcdef".to_string(),
            salt: vec![0u8; solana_ibc_proto::MAX_SALT_LENGTH + 1],
            payload: vec![4, 5, 6],
            memo: String::new(),
        });
    }
}

/// Integration tests using `ProgramTest` with real BPF runtime.
///
/// These verify that `validate_cpi_caller()` rejects direct calls, unauthorized
/// CPI callers and nested CPI using real `get_stack_height()` behavior.
#[cfg(test)]
mod integration_tests {
    use crate::constants::*;
    use crate::state::{GMPAppState, GMPCallResult};
    use crate::test_utils::*;
    use anchor_lang::InstructionData;
    use solana_sdk::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
        signer::Signer,
    };

    const TEST_SOURCE_CLIENT: &str = "cosmoshub-1";
    const TEST_SEQUENCE: u64 = 1;

    fn build_ack_packet_ix(payer: Pubkey) -> Instruction {
        let (app_state_pda, _) = Pubkey::find_program_address(&[GMPAppState::SEED], &crate::ID);
        let (result_pda, _) = GMPCallResult::pda(TEST_SOURCE_CLIENT, TEST_SEQUENCE, &crate::ID);

        let msg = solana_ibc_types::OnAcknowledgementPacketMsg {
            source_client: TEST_SOURCE_CLIENT.to_string(),
            dest_client: "solana-1".to_string(),
            sequence: TEST_SEQUENCE,
            payload: solana_ibc_types::Payload {
                source_port: GMP_PORT_ID.to_string(),
                dest_port: GMP_PORT_ID.to_string(),
                version: ICS27_VERSION.to_string(),
                encoding: ICS27_ENCODING_PROTOBUF.to_string(),
                value: vec![],
            },
            acknowledgement: vec![1, 2, 3],
            relayer: Pubkey::new_unique(),
        };

        let ix_data = crate::instruction::OnAcknowledgementPacket { msg };

        Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new_readonly(app_state_pda, false),
                AccountMeta::new_readonly(
                    anchor_lang::solana_program::sysvar::instructions::ID,
                    false,
                ),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(solana_sdk::system_program::ID, false),
                AccountMeta::new(result_pda, false),
            ],
            data: ix_data.data(),
        }
    }

    #[tokio::test]
    async fn test_direct_call_rejected() {
        let pt = setup_program_test();
        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let ix = build_ack_packet_ix(payer.pubkey());

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

        let inner_ix = build_ack_packet_ix(payer.pubkey());
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

        let inner_ix = build_ack_packet_ix(payer.pubkey());
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

        let inner_ix = build_ack_packet_ix(payer.pubkey());
        let middle_ix = wrap_in_test_cpi_proxy(payer.pubkey(), &inner_ix);
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
    /// The instruction fails later (missing result account data),
    /// but the error is NOT a CPI validation error — proving the rejections
    /// in other tests are genuine access control, not false positives.
    #[tokio::test]
    async fn test_authorized_router_cpi_passes_validation() {
        let pt = setup_program_test_with_router_proxy();
        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let inner_ix = build_ack_packet_ix(payer.pubkey());
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
