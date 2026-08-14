// SPDX-License-Identifier: Apache-2.0

use crate::errors::RouterError;
use crate::events::AckPacketEvent;
use crate::router_cpi::LightClientCpi;
use crate::state::*;
use crate::utils::chunking::total_payload_chunks;
use crate::utils::{chunking, packet};
use anchor_lang::prelude::*;
use ics25_handler::MembershipMsg;
use solana_ibc_types::ibc_app::{
    on_acknowledgement_packet, OnAcknowledgementPacket, OnAcknowledgementPacketMsg,
};
use solana_ibc_types::ics24;
#[cfg(test)]
use solana_ibc_types::IBCAppState;

/// Processes an acknowledgement for a previously sent packet.
///
/// Verifies the ack proof via the light client, invokes the source IBC app's
/// `on_acknowledgement_packet` callback, and zeroes the packet commitment.
/// Remaining accounts carry payload chunks, proof chunks and any extra
/// accounts forwarded to the IBC app.
#[derive(Accounts)]
#[instruction(msg: MsgAckPacket)]
pub struct AckPacket<'info> {
    /// Global router configuration PDA.
    #[account(
        seeds = [RouterState::SEED],
        bump,
        constraint = !router_state.paused @ RouterError::RouterPaused,
    )]
    pub router_state: Account<'info, RouterState>,

    /// Global access control state used for relayer role verification.
    /// CHECK: Validated by seeds constraint using stored `access_manager` program ID
    #[account(
        seeds = [access_manager::state::AccessManager::SEED],
        bump,
        seeds::program = router_state.am_state.access_manager,
    )]
    pub access_manager: AccountInfo<'info>,

    /// PDA mapping the source port to its registered IBC application.
    #[account(
        seeds = [IBCApp::SEED, msg.packet.payloads[0].source_port.as_bytes()],
        bump
    )]
    pub ibc_app: Account<'info, IBCApp>,

    /// Packet commitment PDA; zeroed after successful acknowledgement.
    #[account(
        mut,
        seeds = [
            Commitment::PACKET_COMMITMENT_SEED,
            msg.packet.source_client.as_bytes(),
            &msg.packet.sequence.to_le_bytes()
        ],
        bump
    )]
    pub packet_commitment: Account<'info, Commitment>,

    /// IBC application program to notify via CPI.
    /// CHECK: IBC app program, validated against `IBCApp` account
    #[account(address = ibc_app.app_program_id @ RouterError::IbcAppNotFound)]
    pub ibc_app_program: AccountInfo<'info>,

    /// Mutable state account of the IBC application (passed into the CPI).
    /// CHECK: Ownership validated against IBC app program
    #[account(mut, owner = ibc_app.app_program_id @ RouterError::InvalidAccountOwner)]
    pub ibc_app_state: AccountInfo<'info>,

    /// Relayer submitting the acknowledgement; must hold the `RELAYER_ROLE`.
    /// Receives rent from the closed `packet_commitment` account.
    #[account(mut)]
    pub relayer: Signer<'info>,

    /// Solana system program required by Anchor.
    pub system_program: Program<'info, System>,

    /// Instructions sysvar used for CPI detection.
    /// CHECK: Address constraint verifies this is the instructions sysvar
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instructions_sysvar: AccountInfo<'info>,

    /// Client PDA for the source client; must be active.
    #[account(
        seeds = [Client::SEED, msg.packet.source_client.as_bytes()],
        bump,
        constraint = client.active @ RouterError::ClientNotActive,
    )]
    pub client: Account<'info, Client>,

    /// Light client program used to verify the acknowledgement proof.
    /// CHECK: Validated against client registry
    #[account(address = client.client_program_id @ RouterError::InvalidLightClientProgram)]
    pub light_client_program: AccountInfo<'info>,

    /// Client state account owned by the light client program.
    /// CHECK: Ownership validated against light client program
    #[account(owner = light_client_program.key() @ RouterError::InvalidAccountOwner)]
    pub client_state: AccountInfo<'info>,

    /// Consensus state account owned by the light client program.
    /// CHECK: Ownership validated against light client program
    #[account(owner = light_client_program.key() @ RouterError::InvalidAccountOwner)]
    pub consensus_state: AccountInfo<'info>,
}

pub fn ack_packet<'info>(
    ctx: Context<'_, '_, '_, 'info, AckPacket<'info>>,
    msg: MsgAckPacket,
) -> Result<()> {
    access_manager::require_role(
        &ctx.accounts.access_manager,
        solana_ibc_types::roles::RELAYER_ROLE,
        &ctx.accounts.relayer,
        &ctx.accounts.instructions_sysvar,
        &crate::ID,
    )?;

    // TODO: Support multi-payload packets #602
    let client = &ctx.accounts.client;

    require_eq!(
        &msg.packet.source_client,
        &client.client_id,
        RouterError::ClientMismatch
    );

    require_eq!(
        &msg.packet.dest_client,
        &client.counterparty_info.client_id,
        RouterError::InvalidCounterpartyClient
    );

    let packet = chunking::validate_and_reconstruct_packet(chunking::ReconstructPacketParams {
        msg_packet: &msg.packet,
        remaining_accounts: ctx.remaining_accounts,
        relayer: &ctx.accounts.relayer,
        submitter: ctx.accounts.relayer.key(),
        client_id: &msg.packet.source_client,
    })?;

    let payload = packet::get_single_payload(&packet)?;

    let total_payload_chunks = total_payload_chunks(&msg.packet.payloads)?;
    let proof_data = chunking::assemble_proof_chunks(chunking::AssembleProofParams {
        remaining_accounts: ctx.remaining_accounts,
        relayer: &ctx.accounts.relayer,
        submitter: ctx.accounts.relayer.key(),
        client_id: &msg.packet.source_client,
        sequence: msg.packet.sequence,
        delivery: &msg.proof.data,
        // proof chunks come after payload chunks
        start_index: total_payload_chunks,
    })?;

    // Verify acknowledgement proof on counterparty chain via light client
    let ack_path =
        ics24::packet_acknowledgement_commitment_path(&packet.dest_client, packet.sequence);

    let ack_commitment = ics24::packet_acknowledgement_commitment_bytes32(std::slice::from_ref(
        &msg.acknowledgement,
    ))?;

    let membership_msg = MembershipMsg {
        height: msg.proof.height,
        proof: proof_data,
        path: ics24::prefixed_path(&client.counterparty_info.merkle_prefix, &ack_path)?,
        value: ack_commitment.to_vec(),
    };

    let light_client_cpi = LightClientCpi::new(client);
    light_client_cpi.verify_membership(
        &ctx.accounts.light_client_program,
        &ctx.accounts.client_state,
        &ctx.accounts.consensus_state,
        membership_msg,
    )?;

    let expected_commitment = ics24::packet_commitment_bytes32(&packet);
    require!(
        ctx.accounts.packet_commitment.value == expected_commitment,
        RouterError::PacketCommitmentMismatch
    );

    ctx.accounts.packet_commitment.value = Commitment::EMPTY;

    let app_remaining_accounts = chunking::filter_app_remaining_accounts(
        ctx.remaining_accounts,
        total_payload_chunks,
        msg.proof.data.total_chunks(),
    );

    let cpi_ctx = CpiContext::new(
        ctx.accounts.ibc_app_program.clone(),
        OnAcknowledgementPacket {
            app_state: ctx.accounts.ibc_app_state.clone(),
            instructions_sysvar: ctx.accounts.instructions_sysvar.clone(),
            payer: ctx.accounts.relayer.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
        },
    )
    .with_remaining_accounts(app_remaining_accounts.to_vec());

    let ack_msg = OnAcknowledgementPacketMsg {
        source_client: packet.source_client.clone(),
        dest_client: packet.dest_client.clone(),
        sequence: packet.sequence,
        payload: payload.clone(),
        acknowledgement: msg.acknowledgement.clone(),
        relayer: ctx.accounts.relayer.key(),
    };

    on_acknowledgement_packet(cpi_ctx, ack_msg)?;

    emit!(AckPacketEvent {
        client_id: packet.source_client.clone(),
        sequence: packet.sequence,
        packet,
        acknowledgement: vec![msg.acknowledgement],
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;
    use anchor_lang::InstructionData;
    use mollusk_svm::result::Check;
    use mollusk_svm::Mollusk;
    use solana_ibc_types::{roles, Delivery, MsgPacket, MsgPayload, MsgProof, Payload};
    use solana_sdk::instruction::InstructionError;
    use solana_sdk::instruction::{AccountMeta, Instruction};
    use solana_sdk::program_error::ProgramError;
    use solana_sdk::pubkey::Pubkey;
    use solana_sdk::system_program;

    struct AckPacketTestContext {
        instruction: Instruction,
        accounts: Vec<(Pubkey, solana_sdk::account::Account)>,
        packet_commitment_pubkey: Pubkey,
        packet: MsgPacket,
        dummy_app_state_pubkey: Pubkey,
    }

    struct AckPacketTestParams {
        source_client_id: &'static str,
        dest_client_id: &'static str,
        port_id: &'static str,
        app_program_id: Option<Pubkey>,
        unauthorized_relayer: Option<Pubkey>,
        wrong_dest_client: Option<&'static str>,
        wrong_light_client_program: Option<Pubkey>,
        active_client: bool,
        initial_sequence: u64,
        acknowledgement: Vec<u8>,
        proof_height: u64,
        paused_router: bool,
    }

    impl Default for AckPacketTestParams {
        fn default() -> Self {
            Self {
                source_client_id: "source-client",
                dest_client_id: "dest-client",
                port_id: "test-port",
                app_program_id: None,
                unauthorized_relayer: None,
                wrong_dest_client: None,
                wrong_light_client_program: None,
                active_client: true,
                initial_sequence: 1,
                acknowledgement: vec![1, 2, 3, 4],
                proof_height: 100,
                paused_router: false,
            }
        }
    }

    fn setup_ack_packet_test_with_params(params: AckPacketTestParams) -> AckPacketTestContext {
        let authority = Pubkey::new_unique();
        let relayer = params.unauthorized_relayer.unwrap_or(authority);
        let app_program_id = params.app_program_id.unwrap_or(MOCK_IBC_APP_PROGRAM_ID);

        let client_light_client_program = MOCK_LIGHT_CLIENT_ID;
        let instruction_light_client_program = params
            .wrong_light_client_program
            .unwrap_or(MOCK_LIGHT_CLIENT_ID);

        let (router_state_pda, router_state_data) = if params.paused_router {
            setup_paused_router_state()
        } else {
            setup_router_state()
        };
        let (client_pda, client_data) = setup_client(
            params.source_client_id,
            client_light_client_program,
            params.dest_client_id,
            params.active_client,
        );
        let (ibc_app_pda, ibc_app_data) = setup_ibc_app(params.port_id, app_program_id);

        // Mock app state - just create a dummy account since mock app doesn't use it
        let (dummy_app_state_pda, _) =
            Pubkey::find_program_address(&[IBCAppState::SEED], &app_program_id);

        let packet_dest_client = params.wrong_dest_client.unwrap_or(params.dest_client_id);

        // For tests, we'll simulate having already uploaded chunks
        let test_payload_value = b"test data".to_vec();

        let test_proof = vec![0u8; 32];

        let packet = MsgPacket {
            sequence: params.initial_sequence,
            source_client: params.source_client_id.to_string(),
            dest_client: packet_dest_client.to_string(),
            timeout_timestamp: 1000,
            payloads: vec![MsgPayload {
                source_port: params.port_id.to_string(),
                dest_port: "dest-port".to_string(),
                version: "1".to_string(),
                encoding: "json".to_string(),
                data: Delivery::Chunked { total_chunks: 1 },
            }],
        };

        let (packet_commitment_pda, _) = Pubkey::find_program_address(
            &[
                Commitment::PACKET_COMMITMENT_SEED,
                packet.source_client.as_bytes(),
                &packet.sequence.to_le_bytes(),
            ],
            &crate::ID,
        );

        let msg = MsgAckPacket {
            packet: packet.clone(),
            acknowledgement: params.acknowledgement,
            proof: MsgProof {
                height: params.proof_height,
                data: Delivery::Chunked { total_chunks: 1 },
            },
        };

        let client_state = Pubkey::new_unique();
        let consensus_state = Pubkey::new_unique();

        // Create chunk accounts for 1 payload chunk and 1 proof chunk
        let payload_chunk_account = create_payload_chunk_account(
            relayer,
            params.source_client_id,
            params.initial_sequence,
            0, // payload_index
            0, // chunk_index
            test_payload_value.clone(),
        );

        let proof_chunk_account = create_proof_chunk_account(
            relayer,
            params.source_client_id,
            params.initial_sequence,
            0, // chunk_index
            test_proof,
        );

        // Setup access control with the authority having RELAYER_ROLE
        let (access_manager_pda, access_manager_data) =
            setup_access_manager_with_roles(&[(roles::RELAYER_ROLE, &[authority])]);

        let mut instruction_accounts = vec![
            AccountMeta::new_readonly(router_state_pda, false),
            AccountMeta::new_readonly(access_manager_pda, false),
            AccountMeta::new_readonly(ibc_app_pda, false),
            AccountMeta::new(packet_commitment_pda, false),
            AccountMeta::new_readonly(app_program_id, false),
            AccountMeta::new(dummy_app_state_pda, false),
            AccountMeta::new(relayer, true),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
            AccountMeta::new_readonly(client_pda, false),
            AccountMeta::new_readonly(instruction_light_client_program, false),
            AccountMeta::new_readonly(client_state, false),
            AccountMeta::new_readonly(consensus_state, false),
        ];

        // Add chunk accounts as remaining accounts
        instruction_accounts.push(AccountMeta::new(payload_chunk_account.0, false));
        instruction_accounts.push(AccountMeta::new(proof_chunk_account.0, false));

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: instruction_accounts,
            data: crate::instruction::AckPacket { msg }.data(),
        };

        // Reconstruct full packet with payload for commitment
        let full_packet = Packet {
            sequence: packet.sequence,
            source_client: packet.source_client.clone(),
            dest_client: packet.dest_client.clone(),
            timeout_timestamp: packet.timeout_timestamp,
            payloads: vec![Payload {
                source_port: params.port_id.to_string(),
                dest_port: "dest-port".to_string(),
                version: "1".to_string(),
                encoding: "json".to_string(),
                value: test_payload_value, // Value from chunk
            }],
        };
        let (_, commitment_data) =
            setup_packet_commitment(params.source_client_id, full_packet.sequence, &full_packet);
        let packet_commitment_account =
            create_account(packet_commitment_pda, commitment_data, crate::ID);

        let mut accounts = vec![
            create_account(router_state_pda, router_state_data, crate::ID),
            create_account(access_manager_pda, access_manager_data, access_manager::ID),
            create_account(ibc_app_pda, ibc_app_data, crate::ID),
            packet_commitment_account,
            create_bpf_program_account(app_program_id),
            create_account(dummy_app_state_pda, vec![0u8; 32], app_program_id), // Mock app state
            create_system_account(relayer), // relayer (also signer)
            create_program_account(system_program::ID),
            create_instructions_sysvar_account_with_caller(crate::ID),
            create_account(client_pda, client_data, crate::ID),
            create_bpf_program_account(instruction_light_client_program),
            create_account(client_state, vec![0u8; 100], client_light_client_program),
            create_account(consensus_state, vec![0u8; 100], client_light_client_program),
            create_program_account(access_manager::ID),
        ];

        // Add chunk accounts as remaining accounts
        accounts.push(payload_chunk_account);
        accounts.push(proof_chunk_account);

        AckPacketTestContext {
            instruction,
            accounts,
            packet_commitment_pubkey: packet_commitment_pda,
            packet,
            dummy_app_state_pubkey: dummy_app_state_pda,
        }
    }

    #[test]
    fn test_ack_packet_success() {
        let ctx = setup_ack_packet_test_with_params(AckPacketTestParams::default());

        let mollusk = setup_mollusk_with_mock_programs();

        let checks = vec![Check::success()];

        let result =
            mollusk.process_and_validate_instruction(&ctx.instruction, &ctx.accounts, &checks);

        // Verify packet commitment data was deleted
        let packet_commitment_account = result
            .get_account(&ctx.packet_commitment_pubkey)
            .expect("packet commitment account should exist");

        // Should be zeroed
        assert!(
            packet_commitment_account.data[8..].iter().all(|&b| b == 0),
            "Packet commitment value should be zeroed"
        );
    }

    #[test]
    fn test_ack_packet_unauthorized_sender() {
        let ctx = setup_ack_packet_test_with_params(AckPacketTestParams {
            unauthorized_relayer: Some(Pubkey::new_unique()),
            ..Default::default()
        });

        let mollusk = setup_mollusk_with_mock_programs();

        let checks = vec![Check::err(ProgramError::Custom(
            ANCHOR_ERROR_OFFSET + access_manager::AccessManagerError::Unauthorized as u32,
        ))];

        mollusk.process_and_validate_instruction(&ctx.instruction, &ctx.accounts, &checks);
    }

    #[test]
    fn test_ack_packet_invalid_counterparty() {
        let ctx = setup_ack_packet_test_with_params(AckPacketTestParams {
            wrong_dest_client: Some("wrong-dest-client"),
            ..Default::default()
        });

        let mollusk = Mollusk::new(&crate::ID, crate::test_utils::get_router_program_path());

        let checks = vec![Check::err(ProgramError::Custom(
            ANCHOR_ERROR_OFFSET + RouterError::InvalidCounterpartyClient as u32,
        ))];

        mollusk.process_and_validate_instruction(&ctx.instruction, &ctx.accounts, &checks);
    }

    #[test]
    fn test_ack_packet_client_not_active() {
        let ctx = setup_ack_packet_test_with_params(AckPacketTestParams {
            active_client: false,
            ..Default::default()
        });

        let mollusk = Mollusk::new(&crate::ID, crate::test_utils::get_router_program_path());

        let checks = vec![Check::err(ProgramError::Custom(
            ANCHOR_ERROR_OFFSET + RouterError::ClientNotActive as u32,
        ))];

        mollusk.process_and_validate_instruction(&ctx.instruction, &ctx.accounts, &checks);
    }

    #[test]
    fn test_ack_packet_invalid_light_client_program() {
        let ctx = setup_ack_packet_test_with_params(AckPacketTestParams {
            wrong_light_client_program: Some(Pubkey::new_unique()), // Wrong program
            ..Default::default()
        });

        let mollusk = setup_mollusk_with_mock_programs();

        let checks = vec![Check::err(ProgramError::Custom(
            ANCHOR_ERROR_OFFSET + RouterError::InvalidLightClientProgram as u32,
        ))];

        mollusk.process_and_validate_instruction(&ctx.instruction, &ctx.accounts, &checks);
    }

    // Note: IbcAppNotFound can't be tested here as Anchor's packet_commitment seeds
    // constraint fails first. The validation works in production.

    #[test]
    fn test_ack_packet_zero_payloads() {
        let mut ctx = setup_ack_packet_test_with_params(AckPacketTestParams::default());

        ctx.packet.payloads = vec![];
        let msg = MsgAckPacket {
            packet: ctx.packet.clone(),
            proof: MsgProof {
                height: 100,
                data: Delivery::Chunked { total_chunks: 1 },
            },
            acknowledgement: vec![1u8; 32],
        };

        ctx.instruction.data = crate::instruction::AckPacket { msg }.data();

        let mollusk = setup_mollusk_with_mock_programs();

        // Empty payloads vec causes index-out-of-bounds in Anchor seed resolution
        let checks = vec![Check::instruction_err(
            InstructionError::ProgramFailedToComplete,
        )];

        mollusk.process_and_validate_instruction(&ctx.instruction, &ctx.accounts, &checks);
    }

    #[test]
    fn test_ack_packet_multiple_inline_payloads() {
        let mut ctx = setup_ack_packet_test_with_params(AckPacketTestParams::default());

        ctx.packet.payloads = vec![
            MsgPayload {
                source_port: "test-port".to_string(),
                dest_port: "dest-port".to_string(),
                version: "1".to_string(),
                encoding: "json".to_string(),
                data: Delivery::Inline {
                    data: b"data1".to_vec(),
                },
            },
            MsgPayload {
                source_port: "test-port".to_string(),
                dest_port: "dest-port".to_string(),
                version: "1".to_string(),
                encoding: "json".to_string(),
                data: Delivery::Inline {
                    data: b"data2".to_vec(),
                },
            },
        ];

        let msg = MsgAckPacket {
            packet: ctx.packet.clone(),
            proof: MsgProof {
                height: 100,
                data: Delivery::Chunked { total_chunks: 1 },
            },
            acknowledgement: vec![1u8; 32],
        };

        ctx.instruction.data = crate::instruction::AckPacket { msg }.data();

        let mollusk = setup_mollusk_with_mock_programs();

        // 2 inline payloads reconstruct successfully, then get_single_payload
        // rejects multi-payload packets (only single-payload supported today)
        let checks = vec![Check::err(ProgramError::Custom(
            ANCHOR_ERROR_OFFSET + RouterError::InvalidPayloadCount as u32,
        ))];

        mollusk.process_and_validate_instruction(&ctx.instruction, &ctx.accounts, &checks);
    }

    #[test]
    fn test_ack_packet_mixed_delivery_modes() {
        let mut ctx = setup_ack_packet_test_with_params(AckPacketTestParams::default());

        // Mix inline and chunked payloads in the same packet
        ctx.packet.payloads = vec![
            MsgPayload {
                source_port: "test-port".to_string(),
                dest_port: "dest-port".to_string(),
                version: "1".to_string(),
                encoding: "json".to_string(),
                data: Delivery::Inline {
                    data: b"inline data".to_vec(),
                },
            },
            MsgPayload {
                source_port: "test-port".to_string(),
                dest_port: "dest-port".to_string(),
                version: "1".to_string(),
                encoding: "json".to_string(),
                data: Delivery::Chunked { total_chunks: 1 },
            },
        ];

        let msg = MsgAckPacket {
            packet: ctx.packet.clone(),
            proof: MsgProof {
                height: 100,
                data: Delivery::Chunked { total_chunks: 1 },
            },
            acknowledgement: vec![1u8; 32],
        };

        ctx.instruction.data = crate::instruction::AckPacket { msg }.data();

        let mollusk = setup_mollusk_with_mock_programs();

        let checks = vec![Check::err(ProgramError::Custom(
            ANCHOR_ERROR_OFFSET + RouterError::MixedDeliveryModes as u32,
        ))];

        mollusk.process_and_validate_instruction(&ctx.instruction, &ctx.accounts, &checks);
    }

    #[test]
    fn test_ack_packet_wrong_source_port_seeds_mismatch() {
        let mut ctx = setup_ack_packet_test_with_params(AckPacketTestParams::default());

        ctx.packet.payloads = vec![MsgPayload {
            source_port: "other-port".to_string(),
            dest_port: "dest-port".to_string(),
            version: "1".to_string(),
            encoding: "json".to_string(),
            data: Delivery::Inline {
                data: b"inline data".to_vec(),
            },
        }];

        // source_port doesn't match the ibc_app PDA ("test-port")
        let msg = MsgAckPacket {
            packet: ctx.packet.clone(),
            proof: MsgProof {
                height: 100,
                data: Delivery::Chunked { total_chunks: 1 },
            },
            acknowledgement: vec![1u8; 32],
        };

        ctx.instruction.data = crate::instruction::AckPacket { msg }.data();

        let mollusk = setup_mollusk_with_mock_programs();

        // Anchor seeds constraint fails: ibc_app PDA derived from "other-port" != account seeded with "test-port"
        let checks = vec![Check::err(ProgramError::Custom(
            anchor_lang::error::ErrorCode::ConstraintSeeds as u32,
        ))];

        mollusk.process_and_validate_instruction(&ctx.instruction, &ctx.accounts, &checks);
    }

    #[test]
    fn test_ack_packet_fake_sysvar_wormhole_attack() {
        let mut ctx = setup_ack_packet_test_with_params(AckPacketTestParams::default());

        // Replace real sysvar with fake one (Wormhole-style attack)
        let (instruction, fake_sysvar_account) =
            setup_fake_sysvar_attack(ctx.instruction, crate::ID);
        ctx.instruction = instruction;
        ctx.accounts.push(fake_sysvar_account);

        let mollusk = setup_mollusk_with_mock_programs();

        mollusk.process_and_validate_instruction(
            &ctx.instruction,
            &ctx.accounts,
            &[expect_sysvar_attack_error()],
        );
    }

    #[test]
    fn test_ack_packet_cpi_rejection() {
        let mut ctx = setup_ack_packet_test_with_params(AckPacketTestParams::default());

        // Simulate CPI call from unauthorized program
        let malicious_program = Pubkey::new_unique();
        let (instruction, cpi_sysvar_account) =
            setup_cpi_call_test(ctx.instruction, malicious_program);
        ctx.instruction = instruction;

        // Remove the existing direct-call sysvar and replace with CPI sysvar
        ctx.accounts
            .retain(|(pubkey, _)| *pubkey != solana_sdk::sysvar::instructions::ID);
        ctx.accounts.push(cpi_sysvar_account);

        let mollusk = setup_mollusk_with_mock_programs();

        // When CPI is detected by access_manager::require_role, it returns AccessManagerError::CpiNotAllowed (6005)
        let checks = vec![Check::err(ProgramError::Custom(
            ANCHOR_ERROR_OFFSET + access_manager::AccessManagerError::CpiNotAllowed as u32,
        ))];
        mollusk.process_and_validate_instruction(&ctx.instruction, &ctx.accounts, &checks);
    }

    #[test]
    fn test_ack_packet_wrong_source_port() {
        let mut ctx = setup_ack_packet_test_with_params(AckPacketTestParams::default());

        // Use wrong source_port that doesn't match the ibc_app PDA ("test-port")
        ctx.packet.payloads = vec![MsgPayload {
            source_port: "transfer".to_string(),
            dest_port: "dest-port".to_string(),
            version: "1".to_string(),
            encoding: "json".to_string(),
            data: Delivery::Inline {
                data: b"inline data".to_vec(),
            },
        }];

        let msg = MsgAckPacket {
            packet: ctx.packet.clone(),
            proof: MsgProof {
                height: 100,
                data: Delivery::Chunked { total_chunks: 1 },
            },
            acknowledgement: vec![1u8; 32],
        };

        ctx.instruction.data = crate::instruction::AckPacket { msg }.data();

        let mollusk = setup_mollusk_with_mock_programs();

        // Anchor seeds constraint fails: ibc_app PDA derived from "transfer" != account seeded with "test-port"
        let checks = vec![Check::err(ProgramError::Custom(
            anchor_lang::error::ErrorCode::ConstraintSeeds as u32,
        ))];

        mollusk.process_and_validate_instruction(&ctx.instruction, &ctx.accounts, &checks);
    }

    #[test]
    fn test_ack_packet_paused() {
        let ctx = setup_ack_packet_test_with_params(AckPacketTestParams {
            paused_router: true,
            ..Default::default()
        });

        let checks = vec![Check::err(ProgramError::Custom(
            ANCHOR_ERROR_OFFSET + RouterError::RouterPaused as u32,
        ))];
        let mollusk = setup_mollusk_with_mock_programs();
        mollusk.process_and_validate_instruction(&ctx.instruction, &ctx.accounts, &checks);
    }
}
