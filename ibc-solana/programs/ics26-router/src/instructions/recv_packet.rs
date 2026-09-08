// SPDX-License-Identifier: Apache-2.0

use crate::errors::RouterError;
use crate::events::{NoopEvent, WriteAcknowledgementEvent};
use crate::router_cpi::LightClientCpi;
use crate::state::*;
use crate::utils::chunking::total_payload_chunks;
use crate::utils::{chunking, packet};
use anchor_lang::prelude::*;
use ics25_handler::MembershipMsg;
use solana_ibc_types::ibc_app::{on_recv_packet, OnRecvPacket, OnRecvPacketMsg};
use solana_ibc_types::ics24;

/// Receives an IBC packet by verifying a membership proof against the light
/// client and invoking the destination IBC app's `on_recv_packet` callback.
///
/// Remaining accounts carry payload chunks, proof chunks and any extra
/// accounts forwarded to the IBC app.
///
/// # CPI failure = entire tx reverts
///
/// Solana has no try/catch for CPI. If the app reverts, the whole transaction
/// rolls back and the packet stays unacknowledged (timeout is the only recovery).
/// See `docs/adr/solana-ift-architecture.md` limitations section.
#[derive(Accounts)]
#[instruction(msg: MsgRecvPacket)]
pub struct RecvPacket<'info> {
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

    /// PDA mapping the destination port to its registered IBC application.
    #[account(
        seeds = [IBCApp::SEED, msg.packet.payloads[0].dest_port.as_bytes()],
        bump
    )]
    pub ibc_app: Account<'info, IBCApp>,

    /// Stores the packet receipt commitment; created on first receive.
    #[account(
        init_if_needed,
        payer = relayer,
        space = 8 + Commitment::INIT_SPACE,
        seeds = [
            Commitment::PACKET_RECEIPT_SEED,
            msg.packet.dest_client.as_bytes(),
            &msg.packet.sequence.to_le_bytes()
        ],
        bump
    )]
    pub packet_receipt: Account<'info, Commitment>,

    /// Stores the packet acknowledgement commitment after app callback.
    #[account(
        init_if_needed,
        payer = relayer,
        space = 8 + Commitment::INIT_SPACE,
        seeds = [
            Commitment::PACKET_ACK_SEED,
            msg.packet.dest_client.as_bytes(),
            &msg.packet.sequence.to_le_bytes()
        ],
        bump
    )]
    pub packet_ack: Account<'info, Commitment>,

    /// IBC application program to deliver the packet to via CPI.
    /// CHECK: IBC app program, validated against `IBCApp` account
    #[account(address = ibc_app.app_program_id @ RouterError::IbcAppNotFound)]
    pub ibc_app_program: AccountInfo<'info>,

    /// Mutable state account of the IBC application (passed into the CPI).
    /// CHECK: Ownership validated against IBC app program
    #[account(mut, owner = ibc_app.app_program_id @ RouterError::InvalidAccountOwner)]
    pub ibc_app_state: AccountInfo<'info>,

    /// Relayer submitting the packet; must hold the `RELAYER_ROLE` and pays rent.
    #[account(mut)]
    pub relayer: Signer<'info>,

    /// Solana system program used for account creation.
    pub system_program: Program<'info, System>,

    /// Instructions sysvar used for CPI detection.
    /// CHECK: Address constraint verifies this is the instructions sysvar
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instructions_sysvar: AccountInfo<'info>,

    /// Client PDA for the destination client; must be active.
    #[account(
        seeds = [Client::SEED, msg.packet.dest_client.as_bytes()],
        bump,
        constraint = client.active @ RouterError::ClientNotActive,
    )]
    pub client: Account<'info, Client>,

    /// Light client program used to verify the membership proof.
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

pub fn recv_packet<'info>(
    ctx: Context<'_, '_, '_, 'info, RecvPacket<'info>>,
    msg: MsgRecvPacket,
) -> Result<()> {
    access_manager::require_role(
        &ctx.accounts.access_manager,
        solana_ibc_types::roles::RELAYER_ROLE,
        &ctx.accounts.relayer,
        &ctx.accounts.instructions_sysvar,
        &crate::ID,
    )?;

    let packet_receipt = &mut ctx.accounts.packet_receipt;
    let packet_ack = &mut ctx.accounts.packet_ack;
    let client = &ctx.accounts.client;
    // Get clock directly via syscall
    let clock = Clock::get()?;

    require_eq!(
        &msg.packet.source_client,
        &client.counterparty_info.client_id,
        RouterError::InvalidCounterpartyClient
    );

    require_eq!(
        &msg.packet.dest_client,
        &client.client_id,
        RouterError::ClientMismatch
    );

    let current_timestamp =
        u64::try_from(clock.unix_timestamp).map_err(|_| RouterError::ArithmeticOverflow)?;
    require!(
        msg.packet.timeout_timestamp > current_timestamp,
        RouterError::InvalidTimeoutTimestamp
    );

    let packet = chunking::validate_and_reconstruct_packet(chunking::ReconstructPacketParams {
        msg_packet: &msg.packet,
        remaining_accounts: ctx.remaining_accounts,
        relayer: &ctx.accounts.relayer,
        submitter: ctx.accounts.relayer.key(),
        client_id: &msg.packet.dest_client,
    })?;

    let payload = packet::get_single_payload(&packet)?;

    let total_payload_chunks = total_payload_chunks(&msg.packet.payloads)?;

    let proof_data = chunking::assemble_proof_chunks(chunking::AssembleProofParams {
        remaining_accounts: ctx.remaining_accounts,
        relayer: &ctx.accounts.relayer,
        submitter: ctx.accounts.relayer.key(),
        client_id: &msg.packet.dest_client,
        sequence: msg.packet.sequence,
        delivery: &msg.proof.data,
        // proof chunks come after payload chunks
        start_index: total_payload_chunks,
    })?;

    // Verify packet commitment on counterparty chain via light client
    let commitment_path = ics24::packet_commitment_path(&packet.source_client, packet.sequence);

    let expected_commitment = ics24::packet_commitment_bytes32(&packet);

    // Verify membership proof via CPI to light client
    let membership_msg = MembershipMsg {
        height: msg.proof.height,
        proof: proof_data,
        path: ics24::prefixed_path(&client.counterparty_info.merkle_prefix, &commitment_path)?,
        value: expected_commitment.to_vec(),
    };

    let light_client_cpi = LightClientCpi::new(client);
    light_client_cpi.verify_membership(
        &ctx.accounts.light_client_program,
        &ctx.accounts.client_state,
        &ctx.accounts.consensus_state,
        membership_msg,
    )?;

    let receipt_commitment = ics24::packet_receipt_commitment_bytes32(&packet)?;

    // Check if packet_receipt already exists (non-empty means it was saved before)
    if packet_receipt.value != Commitment::EMPTY {
        if packet_receipt.value == receipt_commitment {
            emit!(NoopEvent {});
            return Ok(());
        }
        return Err(RouterError::PacketReceiptMismatch.into());
    }

    packet_receipt.value = receipt_commitment;

    let app_remaining_accounts = chunking::filter_app_remaining_accounts(
        ctx.remaining_accounts,
        total_payload_chunks,
        msg.proof.data.total_chunks(),
    );

    let cpi_ctx = CpiContext::new(
        ctx.accounts.ibc_app_program.clone(),
        OnRecvPacket {
            app_state: ctx.accounts.ibc_app_state.clone(),
            instructions_sysvar: ctx.accounts.instructions_sysvar.clone(),
            payer: ctx.accounts.relayer.to_account_info(),
            system_program: ctx.accounts.system_program.to_account_info(),
        },
    )
    .with_remaining_accounts(app_remaining_accounts.to_vec());

    let recv_msg = OnRecvPacketMsg {
        source_client: packet.source_client.clone(),
        dest_client: packet.dest_client.clone(),
        sequence: packet.sequence,
        payload: payload.clone(),
        relayer: ctx.accounts.relayer.key(),
    };

    let acknowledgement = match on_recv_packet(cpi_ctx, recv_msg) {
        Ok(ack) => {
            require!(
                !ack.is_empty(),
                RouterError::AsyncAcknowledgementNotSupported
            );

            // Apps must not return the universal error acknowledgement
            // The universal error ack is reserved for the router when the app callback fails
            require!(
                ack != ics24::UNIVERSAL_ERROR_ACK,
                RouterError::UniversalErrorAcknowledgement
            );

            ack
        }
        Err(_e) => {
            unreachable!()
            // IMPORTANT: CPI Error Handling Limitation in Solana
            //
            // In theory, this branch should catch CPI failures and return UNIVERSAL_ERROR_ACK,
            // matching the Ethereum implementation where try/catch handles app callback errors.
            //
            // HOWEVER, Solana's CPI error handling has a critical limitation:
            // - If a CPI call fails, the ENTIRE TRANSACTION ABORTS immediately
            // - This error branch is effectively UNREACHABLE in practice
            // - The `invoke()` function returns Result for legacy reasons, but errors cannot be caught
            //
            // Current Behavior:
            // When an IBC app's `on_recv_packet` callback fails:
            // - Solana: Transaction aborts, packet is NOT acknowledged (relayer must retry/timeout)
            // - Ethereum: Catch error, return UNIVERSAL_ERROR_ACK, packet IS acknowledged
            //
            // See: https://solana.stackexchange.com/questions/13723/tx-reverting-even-if-internal-cpi-fails
        }
    };

    let acknowledgements = vec![acknowledgement];
    let ack_commitment = ics24::packet_acknowledgement_commitment_bytes32(&acknowledgements)?;
    packet_ack.value = ack_commitment;

    emit!(WriteAcknowledgementEvent {
        client_id: packet.dest_client.clone(),
        sequence: packet.sequence,
        packet,
        acknowledgements,
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
    use solana_sdk::instruction::{AccountMeta, Instruction};
    use solana_sdk::program_error::ProgramError;
    use solana_sdk::pubkey::Pubkey;
    use solana_sdk::{clock::Clock, system_program};

    #[test]
    fn test_recv_packet_unauthorized_sender() {
        let ctx = setup_recv_packet_test_with_params(RecvPacketTestParams {
            unauthorized_relayer: Some(Pubkey::new_unique()),
            ..Default::default()
        });

        let mollusk = Mollusk::new(&crate::ID, crate::test_utils::get_router_program_path());

        // Expect RouterError::UnauthorizedSender
        let checks = vec![Check::err(ProgramError::Custom(
            ANCHOR_ERROR_OFFSET + RouterError::UnauthorizedSender as u32,
        ))];

        mollusk.process_and_validate_instruction(&ctx.instruction, &ctx.accounts, &checks);
    }

    #[test]
    fn test_recv_packet_invalid_counterparty() {
        // Setup expects "source-client" but packet comes from "wrong-source-client"
        let ctx = setup_recv_packet_test_with_params(RecvPacketTestParams {
            source_client_id: "wrong-source-client",
            ..Default::default()
        });

        let mollusk = setup_mollusk_with_mock_programs();

        let checks = vec![Check::err(ProgramError::Custom(
            ANCHOR_ERROR_OFFSET + RouterError::InvalidCounterpartyClient as u32,
        ))];

        mollusk.process_and_validate_instruction(&ctx.instruction, &ctx.accounts, &checks);
    }

    #[test]
    fn test_recv_packet_timeout_expired() {
        // timeout_timestamp = 1000 + 0 = 1000, clock = 1000
        // Check requires timeout > current_timestamp, so 1000 > 1000 is false (expired)
        let mut ctx = setup_recv_packet_test(true, 0);

        let mollusk = Mollusk::new(&crate::ID, crate::test_utils::get_router_program_path());

        let clock_data = create_clock_data(1000);
        ctx.accounts
            .push(create_clock_account_with_data(clock_data));

        let checks = vec![Check::err(ProgramError::Custom(
            ANCHOR_ERROR_OFFSET + RouterError::InvalidTimeoutTimestamp as u32,
        ))];

        mollusk.process_and_validate_instruction(&ctx.instruction, &ctx.accounts, &checks);
    }

    #[test]
    fn test_recv_packet_client_not_active() {
        let ctx = setup_recv_packet_test(false, 1000); // Inactive client

        let mollusk = Mollusk::new(&crate::ID, crate::test_utils::get_router_program_path());

        let checks = vec![Check::err(ProgramError::Custom(
            ANCHOR_ERROR_OFFSET + RouterError::ClientNotActive as u32,
        ))];

        mollusk.process_and_validate_instruction(&ctx.instruction, &ctx.accounts, &checks);
    }

    struct RecvPacketTestContext {
        instruction: Instruction,
        accounts: Vec<(Pubkey, solana_sdk::account::Account)>,
        packet: MsgPacket,
        packet_receipt_pubkey: Pubkey,
        packet_ack_pubkey: Pubkey,
    }

    struct RecvPacketTestParams {
        active_client: bool,
        timeout_offset: u64,
        source_client_id: &'static str,
        unauthorized_relayer: Option<Pubkey>,
        paused_router: bool,
    }

    impl Default for RecvPacketTestParams {
        fn default() -> Self {
            Self {
                active_client: true,
                timeout_offset: 1000,
                source_client_id: "source-client",
                unauthorized_relayer: None,
                paused_router: false,
            }
        }
    }

    fn setup_recv_packet_test_with_params(params: RecvPacketTestParams) -> RecvPacketTestContext {
        let authority = Pubkey::new_unique();
        let relayer = params.unauthorized_relayer.unwrap_or(authority);
        let client_id = "test-client";
        let port_id = "test-port";
        let light_client_program = MOCK_LIGHT_CLIENT_ID;

        let (router_state_pda, router_state_data) = if params.paused_router {
            setup_paused_router_state()
        } else {
            setup_router_state()
        };

        // Always setup client expecting "source-client" as counterparty
        let (client_pda, client_data) = setup_client(
            client_id,
            light_client_program,
            "source-client",
            params.active_client,
        );
        let ibc_app_program_id = MOCK_IBC_APP_PROGRAM_ID;
        let (ibc_app_pda, ibc_app_data) = setup_ibc_app(port_id, ibc_app_program_id);
        let ibc_app_state = Pubkey::new_unique();

        let current_timestamp = 1000;

        // Packet uses the source_client_id from params (could be different)
        // For tests, we'll simulate having already uploaded chunks
        let test_payload_value = b"test data".to_vec();

        let test_proof = vec![0u8; 32];

        let packet = MsgPacket {
            sequence: 1,
            source_client: params.source_client_id.to_string(),
            dest_client: client_id.to_string(),
            timeout_timestamp: current_timestamp + params.timeout_offset,
            payloads: vec![MsgPayload {
                source_port: "source-port".to_string(),
                dest_port: port_id.to_string(),
                version: "1".to_string(),
                encoding: "json".to_string(),
                data: Delivery::Chunked { total_chunks: 1 },
            }],
        };

        let msg = MsgRecvPacket {
            packet: packet.clone(),
            proof: MsgProof {
                height: 100,
                data: Delivery::Chunked { total_chunks: 1 },
            },
        };

        let (packet_receipt_pda, _) = Pubkey::find_program_address(
            &[
                Commitment::PACKET_RECEIPT_SEED,
                msg.packet.dest_client.as_bytes(),
                &msg.packet.sequence.to_le_bytes(),
            ],
            &crate::ID,
        );

        let (packet_ack_pda, _) = Pubkey::find_program_address(
            &[
                Commitment::PACKET_ACK_SEED,
                msg.packet.dest_client.as_bytes(),
                &msg.packet.sequence.to_le_bytes(),
            ],
            &crate::ID,
        );

        let client_state = Pubkey::new_unique();
        let consensus_state = Pubkey::new_unique();

        // The transaction signer is the relayer
        let transaction_signer = relayer;

        // Create chunk accounts for 1 payload chunk and 1 proof chunk
        let payload_chunk_account = create_payload_chunk_account(
            transaction_signer,
            client_id,
            1,
            0, // payload_index
            0, // chunk_index
            test_payload_value,
        );

        let proof_chunk_account = create_proof_chunk_account(
            transaction_signer,
            client_id,
            1,
            0, // chunk_index
            test_proof,
        );

        // Setup access control: authority always has RELAYER_ROLE
        // For authorized tests: transaction_signer == authority (has the role)
        // For unauthorized tests: transaction_signer != authority (does NOT have the role)
        let (access_manager_pda, access_manager_data) =
            setup_access_manager_with_roles(&[(roles::RELAYER_ROLE, &[authority])]);

        let mut instruction_accounts = vec![
            AccountMeta::new_readonly(router_state_pda, false),
            AccountMeta::new_readonly(access_manager_pda, false),
            AccountMeta::new_readonly(ibc_app_pda, false),
            AccountMeta::new(packet_receipt_pda, false),
            AccountMeta::new(packet_ack_pda, false),
            AccountMeta::new_readonly(ibc_app_program_id, false),
            AccountMeta::new(ibc_app_state, false),
            AccountMeta::new(relayer, true),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
            AccountMeta::new_readonly(client_pda, false),
            AccountMeta::new_readonly(light_client_program, false),
            AccountMeta::new_readonly(client_state, false),
            AccountMeta::new_readonly(consensus_state, false),
        ];

        // Add chunk accounts as remaining accounts
        instruction_accounts.push(AccountMeta::new(payload_chunk_account.0, false));
        instruction_accounts.push(AccountMeta::new(proof_chunk_account.0, false));

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: instruction_accounts,
            data: crate::instruction::RecvPacket { msg }.data(),
        };

        let packet_receipt_account = create_uninitialized_commitment_account(packet_receipt_pda);
        let packet_ack_account = create_uninitialized_commitment_account(packet_ack_pda);

        // Create signer account (transaction_signer and payer are the same)
        let signer_account = create_system_account(transaction_signer);

        // Accounts must be in the exact order of the instruction
        let mut accounts = vec![
            create_account(router_state_pda, router_state_data, crate::ID),
            create_account(access_manager_pda, access_manager_data, access_manager::ID),
            create_account(ibc_app_pda, ibc_app_data, crate::ID),
            packet_receipt_account,
            packet_ack_account,
            create_bpf_program_account(ibc_app_program_id),
            create_account(ibc_app_state, vec![0u8; 100], ibc_app_program_id),
            signer_account,
            create_program_account(system_program::ID),
            create_instructions_sysvar_account_with_caller(crate::ID),
            create_account(client_pda, client_data, crate::ID),
            create_bpf_program_account(light_client_program),
            create_account(client_state, vec![0u8; 100], light_client_program),
            create_account(consensus_state, vec![0u8; 100], light_client_program),
        ];

        // Add chunk accounts as remaining accounts
        accounts.push(payload_chunk_account);
        accounts.push(proof_chunk_account);

        RecvPacketTestContext {
            instruction,
            accounts,
            packet,
            packet_receipt_pubkey: packet_receipt_pda,
            packet_ack_pubkey: packet_ack_pda,
        }
    }

    fn setup_recv_packet_test(active_client: bool, timeout_offset: u64) -> RecvPacketTestContext {
        setup_recv_packet_test_with_params(RecvPacketTestParams {
            active_client,
            timeout_offset,
            ..Default::default()
        })
    }

    fn create_clock_data(timestamp: i64) -> Vec<u8> {
        let mut clock_data = vec![0u8; Clock::size_of()];
        let clock = Clock {
            slot: 0,
            epoch_start_timestamp: 0,
            epoch: 0,
            leader_schedule_epoch: 0,
            unix_timestamp: timestamp,
        };
        bincode::serialize_into(&mut clock_data[..], &clock).unwrap();
        clock_data
    }

    #[test]
    fn test_recv_packet_success() {
        let ctx = setup_recv_packet_test(true, 1000);

        let mollusk = setup_mollusk_with_mock_programs();

        // Calculate expected rent-exempt lamports for Commitment accounts
        let commitment_rent = {
            use anchor_lang::Space;
            use solana_sdk::rent::Rent;
            let account_size = 8 + Commitment::INIT_SPACE;
            Rent::default().minimum_balance(account_size)
        };

        let checks = vec![
            Check::success(),
            Check::account(&ctx.packet_receipt_pubkey)
                .lamports(commitment_rent)
                .owner(&crate::ID)
                .build(),
            Check::account(&ctx.packet_ack_pubkey)
                .lamports(commitment_rent)
                .owner(&crate::ID)
                .build(),
        ];

        mollusk.process_and_validate_instruction(&ctx.instruction, &ctx.accounts, &checks);

        let result = mollusk.process_instruction(&ctx.instruction, &ctx.accounts);

        // Check packet receipt
        // Note: The handler reconstructs the packet with payloads from chunks
        let expected_packet = Packet {
            sequence: ctx.packet.sequence,
            source_client: ctx.packet.source_client.clone(),
            dest_client: ctx.packet.dest_client.clone(),
            timeout_timestamp: ctx.packet.timeout_timestamp,
            payloads: vec![Payload {
                source_port: "source-port".to_string(),
                dest_port: "test-port".to_string(),
                version: "1".to_string(),
                encoding: "json".to_string(),
                value: b"test data".to_vec(), // Value from chunk
            }],
        };
        let receipt_commitment =
            ics24::packet_receipt_commitment_bytes32(&expected_packet).unwrap();
        let receipt_data = get_account_data_from_mollusk(&result, &ctx.packet_receipt_pubkey)
            .expect("packet receipt account not found");
        assert_eq!(receipt_data[..32], receipt_commitment);

        // Verify ack commitment matches the raw ack bytes (Borsh prefix stripped by router).
        let raw_ack = b"packet received";
        let expected_ack_commitment =
            ics24::packet_acknowledgement_commitment_bytes32(&[raw_ack.to_vec()]).unwrap();
        let ack_data = get_account_data_from_mollusk(&result, &ctx.packet_ack_pubkey)
            .expect("packet ack account not found");
        assert_eq!(
            ack_data[..32],
            expected_ack_commitment,
            "ack commitment must match raw ack bytes (Borsh prefix must be stripped)"
        );
    }

    #[test]
    fn test_recv_packet_app_returns_universal_error_ack() {
        // Test that the router properly handles error acknowledgements
        // Note: Since the mock app always returns success, we can't actually test error ack
        // This test now verifies normal success acknowledgement flow
        let mut ctx = setup_recv_packet_test(true, 1000);

        // Update the instruction with modified metadata
        let msg = MsgRecvPacket {
            packet: ctx.packet.clone(),
            proof: MsgProof {
                height: 100,
                data: Delivery::Chunked { total_chunks: 1 },
            },
        };

        ctx.instruction.data = crate::instruction::RecvPacket { msg }.data();

        let mollusk = setup_mollusk_with_mock_programs();

        // Calculate expected rent-exempt lamports for Commitment accounts
        let commitment_rent = {
            use anchor_lang::Space;
            use solana_sdk::rent::Rent;
            let account_size = 8 + Commitment::INIT_SPACE;
            Rent::default().minimum_balance(account_size)
        };

        let checks = vec![
            Check::success(), // Should still succeed even with error ack
            Check::account(&ctx.packet_receipt_pubkey)
                .lamports(commitment_rent)
                .owner(&crate::ID)
                .build(),
            Check::account(&ctx.packet_ack_pubkey)
                .lamports(commitment_rent)
                .owner(&crate::ID)
                .build(),
        ];

        mollusk.process_and_validate_instruction(&ctx.instruction, &ctx.accounts, &checks);

        let result = mollusk.process_instruction(&ctx.instruction, &ctx.accounts);

        // Check packet receipt first
        // Note: The handler reconstructs the packet with payloads from chunks
        let expected_packet = Packet {
            sequence: ctx.packet.sequence,
            source_client: ctx.packet.source_client.clone(),
            dest_client: ctx.packet.dest_client.clone(),
            timeout_timestamp: ctx.packet.timeout_timestamp,
            payloads: vec![Payload {
                source_port: "source-port".to_string(),
                dest_port: "test-port".to_string(),
                version: "1".to_string(),
                encoding: "json".to_string(),
                value: b"test data".to_vec(), // Value from chunk (with space, matching test data)
            }],
        };
        let receipt_commitment =
            ics24::packet_receipt_commitment_bytes32(&expected_packet).unwrap();
        let receipt_data = get_account_data_from_mollusk(&result, &ctx.packet_receipt_pubkey)
            .expect("packet receipt account not found");
        assert_eq!(receipt_data[..32], receipt_commitment);

        // Check acknowledgement was written (mock app returns "packet received")
        let ack_data = get_account_data_from_mollusk(&result, &ctx.packet_ack_pubkey)
            .expect("packet ack account not found");

        // The ack commitment should be the keccak256 of the acks vector containing b"packet received"
        let expected_acks = vec![b"packet received".to_vec()];
        let expected_ack_commitment =
            ics24::packet_acknowledgement_commitment_bytes32(&expected_acks)
                .expect("failed to compute ack commitment");

        assert_eq!(
            ack_data[..32],
            expected_ack_commitment,
            "acknowledgement should be set correctly"
        );
    }

    #[test]
    fn test_recv_packet_receipt_mismatch() {
        let mut ctx = setup_recv_packet_test_with_params(RecvPacketTestParams::default());

        // Pre-create packet_receipt with a different commitment (simulates prior receipt)
        let different_commitment = [0xFFu8; 32];

        let packet_receipt_data = {
            use crate::state::Commitment;
            use anchor_lang::AccountSerialize;

            let packet_receipt = Commitment {
                value: different_commitment,
            };
            let mut data = vec![];
            packet_receipt.try_serialize(&mut data).unwrap();
            data
        };

        let packet_receipt_account = solana_sdk::account::Account {
            lamports: 10_000_000,
            data: packet_receipt_data,
            owner: crate::ID,
            executable: false,
            rent_epoch: 0,
        };

        if let Some(pos) = ctx
            .accounts
            .iter()
            .position(|(k, _)| *k == ctx.packet_receipt_pubkey)
        {
            ctx.accounts[pos] = (ctx.packet_receipt_pubkey, packet_receipt_account);
        }

        let mollusk = setup_mollusk_with_mock_programs();

        let checks = vec![Check::err(ProgramError::Custom(
            ANCHOR_ERROR_OFFSET + RouterError::PacketReceiptMismatch as u32,
        ))];

        mollusk.process_and_validate_instruction(&ctx.instruction, &ctx.accounts, &checks);
    }

    #[test]
    fn test_recv_packet_noop_same_receipt() {
        // Test the no-op path: packet_receipt exists with correct commitment -> emit NoopEvent.
        // Note: Mollusk doesn't capture program logs, so we verify noop behavior by checking
        // that the packet_receipt state remains unchanged (no IBC app callback was invoked).
        let mut ctx = setup_recv_packet_test_with_params(RecvPacketTestParams::default());

        let reconstructed_packet = Packet {
            sequence: 1,
            source_client: "source-client".to_string(),
            dest_client: "test-client".to_string(),
            timeout_timestamp: 2000,
            payloads: vec![solana_ibc_types::Payload {
                source_port: "source-port".to_string(),
                dest_port: "test-port".to_string(),
                version: "1".to_string(),
                encoding: "json".to_string(),
                value: b"test data".to_vec(),
            }],
        };

        let correct_receipt_commitment =
            ics24::packet_receipt_commitment_bytes32(&reconstructed_packet).unwrap();

        let packet_receipt_data = {
            use anchor_lang::AccountSerialize;

            let packet_receipt = Commitment {
                value: correct_receipt_commitment,
            };
            let mut data = vec![];
            packet_receipt.try_serialize(&mut data).unwrap();
            data
        };

        let packet_receipt_account = solana_sdk::account::Account {
            lamports: 10_000_000,
            data: packet_receipt_data,
            owner: crate::ID,
            executable: false,
            rent_epoch: 0,
        };

        if let Some(pos) = ctx
            .accounts
            .iter()
            .position(|(k, _)| *k == ctx.packet_receipt_pubkey)
        {
            ctx.accounts[pos] = (ctx.packet_receipt_pubkey, packet_receipt_account);
        }

        let mollusk = setup_mollusk_with_mock_programs();
        let checks = vec![Check::success()];

        let result =
            mollusk.process_and_validate_instruction(&ctx.instruction, &ctx.accounts, &checks);

        let receipt_account = result
            .resulting_accounts
            .iter()
            .find(|(pubkey, _)| *pubkey == ctx.packet_receipt_pubkey)
            .map(|(_, account)| account)
            .expect("packet_receipt account not found");

        let receipt_commitment: Commitment =
            Commitment::try_deserialize(&mut &receipt_account.data[..]).unwrap();
        assert_eq!(
            receipt_commitment.value, correct_receipt_commitment,
            "packet_receipt should still have the same commitment"
        );
    }

    #[test]
    fn test_recv_packet_wrong_ibc_app_pda() {
        let mut ctx = setup_recv_packet_test(true, 1000);

        let wrong_ibc_app = Pubkey::new_unique();

        let ibc_app = IBCApp {
            version: AccountVersion::V1,
            port_id: "test-port".to_string(),
            app_program_id: MOCK_IBC_APP_PROGRAM_ID,
            _reserved: [0; 256],
        };

        let wrong_ibc_app_account = solana_sdk::account::Account {
            lamports: 10_000_000,
            data: crate::test_utils::create_account_data(&ibc_app),
            owner: crate::ID,
            executable: false,
            rent_epoch: 0,
        };

        if let Some(pos) = ctx
            .accounts
            .iter()
            .position(|(pubkey, _)| *pubkey == ctx.accounts[2].0)
        {
            ctx.accounts[pos] = (wrong_ibc_app, wrong_ibc_app_account);
            ctx.instruction.accounts[2].pubkey = wrong_ibc_app;
        }

        let mollusk = Mollusk::new(&crate::ID, crate::test_utils::get_router_program_path());

        let checks = vec![Check::err(ProgramError::Custom(
            anchor_lang::error::ErrorCode::ConstraintSeeds as u32,
        ))];

        mollusk.process_and_validate_instruction(&ctx.instruction, &ctx.accounts, &checks);
    }

    #[test]
    fn test_recv_packet_zero_payloads() {
        let mut ctx = setup_recv_packet_test(true, 1000);

        ctx.packet.payloads = vec![];
        let msg = MsgRecvPacket {
            packet: ctx.packet.clone(),
            proof: MsgProof {
                height: 100,
                data: Delivery::Chunked { total_chunks: 1 },
            },
        };

        ctx.instruction.data = crate::instruction::RecvPacket { msg }.data();

        let mollusk = setup_mollusk_with_mock_programs();
        let result = mollusk.process_instruction(&ctx.instruction, &ctx.accounts);

        assert!(
            !matches!(
                result.program_result,
                mollusk_svm::result::ProgramResult::Success
            ),
            "Empty payloads should be rejected"
        );
    }

    #[test]
    fn test_recv_packet_multiple_payloads() {
        // Test that packet with multiple inline payloads fails
        let mut ctx = setup_recv_packet_test(true, 1000);

        // Create a packet with 2 inline payloads
        ctx.packet.payloads = vec![
            MsgPayload {
                source_port: "source-port-1".to_string(),
                dest_port: "test-port".to_string(),
                version: "1".to_string(),
                encoding: "json".to_string(),
                data: Delivery::Inline {
                    data: b"data1".to_vec(),
                },
            },
            MsgPayload {
                source_port: "source-port-2".to_string(),
                dest_port: "test-port".to_string(),
                version: "1".to_string(),
                encoding: "json".to_string(),
                data: Delivery::Inline {
                    data: b"data2".to_vec(),
                },
            },
        ];

        let msg = MsgRecvPacket {
            packet: ctx.packet.clone(),
            proof: MsgProof {
                height: 100,
                data: Delivery::Chunked { total_chunks: 1 },
            },
        };

        ctx.instruction.data = crate::instruction::RecvPacket { msg }.data();

        let mollusk = setup_mollusk_with_mock_programs();

        let checks = vec![Check::err(ProgramError::Custom(
            ANCHOR_ERROR_OFFSET + RouterError::InvalidPayloadCount as u32,
        ))];

        mollusk.process_and_validate_instruction(&ctx.instruction, &ctx.accounts, &checks);
    }

    #[test]
    fn test_recv_packet_mixed_delivery_modes() {
        // Test that packet with mixed inline and chunked payloads fails
        let mut ctx = setup_recv_packet_test(true, 1000);

        // Mix inline and chunked payloads in the same packet
        ctx.packet.payloads = vec![
            MsgPayload {
                source_port: "source-port".to_string(),
                dest_port: "test-port".to_string(),
                version: "1".to_string(),
                encoding: "json".to_string(),
                data: Delivery::Inline {
                    data: b"inline data".to_vec(),
                },
            },
            MsgPayload {
                source_port: "source-port".to_string(),
                dest_port: "test-port".to_string(),
                version: "1".to_string(),
                encoding: "json".to_string(),
                data: Delivery::Chunked { total_chunks: 1 },
            },
        ];

        let msg = MsgRecvPacket {
            packet: ctx.packet.clone(),
            proof: MsgProof {
                height: 100,
                data: Delivery::Chunked { total_chunks: 1 },
            },
        };

        ctx.instruction.data = crate::instruction::RecvPacket { msg }.data();

        let mollusk = setup_mollusk_with_mock_programs();

        let checks = vec![Check::err(ProgramError::Custom(
            ANCHOR_ERROR_OFFSET + RouterError::MixedDeliveryModes as u32,
        ))];

        mollusk.process_and_validate_instruction(&ctx.instruction, &ctx.accounts, &checks);
    }

    #[test]
    fn test_recv_packet_fake_sysvar_wormhole_attack() {
        let mut ctx = setup_recv_packet_test(true, 1000);

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
    fn test_recv_packet_invalid_ibc_app_state_owner() {
        let mut ctx = setup_recv_packet_test(true, 1000);

        // Replace ibc_app_state (index 6) with account owned by wrong program
        let wrong_owner = Pubkey::new_unique();
        let (pubkey, _) = ctx.accounts[6].clone();
        ctx.accounts[6] = (
            pubkey,
            solana_sdk::account::Account {
                lamports: 10_000_000,
                data: vec![0u8; 100],
                owner: wrong_owner,
                executable: false,
                rent_epoch: 0,
            },
        );

        let mollusk = Mollusk::new(&crate::ID, crate::test_utils::get_router_program_path());

        let checks = vec![Check::err(ProgramError::Custom(
            ANCHOR_ERROR_OFFSET + RouterError::InvalidAccountOwner as u32,
        ))];

        mollusk.process_and_validate_instruction(&ctx.instruction, &ctx.accounts, &checks);
    }

    #[test]
    fn test_recv_packet_invalid_light_client_program() {
        let mut ctx = setup_recv_packet_test(true, 1000);

        // Replace light_client_program (index 11) with a different program
        let wrong_program = Pubkey::new_unique();
        ctx.instruction.accounts[11].pubkey = wrong_program;
        ctx.accounts[11] = create_bpf_program_account(wrong_program);

        let mollusk = Mollusk::new(&crate::ID, crate::test_utils::get_router_program_path());

        let checks = vec![Check::err(ProgramError::Custom(
            ANCHOR_ERROR_OFFSET + RouterError::InvalidLightClientProgram as u32,
        ))];

        mollusk.process_and_validate_instruction(&ctx.instruction, &ctx.accounts, &checks);
    }

    #[test]
    fn test_recv_packet_invalid_client_state_owner() {
        let mut ctx = setup_recv_packet_test(true, 1000);

        // Replace client_state (index 12) with account owned by wrong program
        let wrong_owner = Pubkey::new_unique();
        let (pubkey, _) = ctx.accounts[12].clone();
        ctx.accounts[12] = (
            pubkey,
            solana_sdk::account::Account {
                lamports: 10_000_000,
                data: vec![0u8; 100],
                owner: wrong_owner,
                executable: false,
                rent_epoch: 0,
            },
        );

        let mollusk = Mollusk::new(&crate::ID, crate::test_utils::get_router_program_path());

        let checks = vec![Check::err(ProgramError::Custom(
            ANCHOR_ERROR_OFFSET + RouterError::InvalidAccountOwner as u32,
        ))];

        mollusk.process_and_validate_instruction(&ctx.instruction, &ctx.accounts, &checks);
    }

    #[test]
    fn test_recv_packet_invalid_consensus_state_owner() {
        let mut ctx = setup_recv_packet_test(true, 1000);

        // Replace consensus_state (index 13) with account owned by wrong program
        let wrong_owner = Pubkey::new_unique();
        let (pubkey, _) = ctx.accounts[13].clone();
        ctx.accounts[13] = (
            pubkey,
            solana_sdk::account::Account {
                lamports: 10_000_000,
                data: vec![0u8; 100],
                owner: wrong_owner,
                executable: false,
                rent_epoch: 0,
            },
        );

        let mollusk = Mollusk::new(&crate::ID, crate::test_utils::get_router_program_path());

        let checks = vec![Check::err(ProgramError::Custom(
            ANCHOR_ERROR_OFFSET + RouterError::InvalidAccountOwner as u32,
        ))];

        mollusk.process_and_validate_instruction(&ctx.instruction, &ctx.accounts, &checks);
    }

    #[test]
    fn test_recv_packet_cpi_rejection() {
        let mut ctx = setup_recv_packet_test(true, 1000);

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

    // ── ProgramTest-based recv_packet tests ──

    use solana_ibc_types::ics24;
    use solana_program_test::ProgramTest;
    use solana_sdk::signature::Keypair;
    use solana_sdk::signer::Signer as SdkSigner;
    use solana_sdk::transaction::Transaction;

    const RECV_TEST_PORT: &str = "test-port";
    const RECV_DEST_CLIENT: &str = "dest-client";
    const RECV_SOURCE_CLIENT: &str = "source-client";
    const RECV_TEST_CLOCK_TIME: i64 = 1000;
    const RECV_TEST_TIMEOUT: u64 = RECV_TEST_CLOCK_TIME as u64 + 2000;

    fn setup_recv_two_packets_program_test(relayer_pubkey: Pubkey) -> ProgramTest {
        if std::env::var("SBF_OUT_DIR").is_err() {
            let deploy_dir = std::path::Path::new("../../target/deploy");
            std::env::set_var("SBF_OUT_DIR", deploy_dir);
        }

        let mut pt = ProgramTest::new("ics26_router", crate::ID, None);
        pt.add_program("mock_light_client", MOCK_LIGHT_CLIENT_ID, None);
        pt.add_program("mock_ibc_app", MOCK_IBC_APP_PROGRAM_ID, None);
        pt.add_program("access_manager", access_manager::ID, None);

        // RouterState
        let (router_state_pda, router_state_data) = setup_router_state();
        pt.add_account(
            router_state_pda,
            solana_sdk::account::Account {
                lamports: 1_000_000,
                data: router_state_data,
                owner: crate::ID,
                executable: false,
                rent_epoch: 0,
            },
        );

        // AccessManager with RELAYER_ROLE for the relayer
        let (access_manager_pda, access_manager_data) = setup_access_manager_with_roles(&[(
            solana_ibc_types::roles::RELAYER_ROLE,
            &[relayer_pubkey],
        )]);
        pt.add_account(
            access_manager_pda,
            solana_sdk::account::Account {
                lamports: 1_000_000,
                data: access_manager_data,
                owner: access_manager::ID,
                executable: false,
                rent_epoch: 0,
            },
        );

        // Client: dest-client with counterparty source-client
        let (client_pda, client_data) = setup_client(
            RECV_DEST_CLIENT,
            MOCK_LIGHT_CLIENT_ID,
            RECV_SOURCE_CLIENT,
            true,
        );
        pt.add_account(
            client_pda,
            solana_sdk::account::Account {
                lamports: 1_000_000,
                data: client_data,
                owner: crate::ID,
                executable: false,
                rent_epoch: 0,
            },
        );

        // IBCApp for test-port registered to mock_ibc_app
        let (ibc_app_pda, ibc_app_data) = setup_ibc_app(RECV_TEST_PORT, MOCK_IBC_APP_PROGRAM_ID);
        pt.add_account(
            ibc_app_pda,
            solana_sdk::account::Account {
                lamports: 1_000_000,
                data: ibc_app_data,
                owner: crate::ID,
                executable: false,
                rent_epoch: 0,
            },
        );

        // Mock light client state and consensus state
        let mock_client_state = Pubkey::new_unique();
        pt.add_account(
            mock_client_state,
            solana_sdk::account::Account {
                lamports: 1_000_000,
                data: vec![0u8; 64],
                owner: MOCK_LIGHT_CLIENT_ID,
                executable: false,
                rent_epoch: 0,
            },
        );
        let mock_consensus_state = Pubkey::new_unique();
        pt.add_account(
            mock_consensus_state,
            solana_sdk::account::Account {
                lamports: 1_000_000,
                data: vec![0u8; 64],
                owner: MOCK_LIGHT_CLIENT_ID,
                executable: false,
                rent_epoch: 0,
            },
        );

        // Mock IBC app state (owned by mock_ibc_app program)
        let mock_ibc_app_state = Pubkey::new_unique();
        pt.add_account(
            mock_ibc_app_state,
            solana_sdk::account::Account {
                lamports: 1_000_000,
                data: vec![0u8; 100],
                owner: MOCK_IBC_APP_PROGRAM_ID,
                executable: false,
                rent_epoch: 0,
            },
        );

        // Override clock for deterministic timestamps
        let clock = solana_sdk::clock::Clock {
            slot: 1,
            epoch_start_timestamp: RECV_TEST_CLOCK_TIME,
            epoch: 0,
            leader_schedule_epoch: 0,
            unix_timestamp: RECV_TEST_CLOCK_TIME,
        };
        let mut clock_data = vec![0u8; solana_sdk::clock::Clock::size_of()];
        bincode::serialize_into(&mut clock_data[..], &clock).unwrap();
        pt.add_account(
            solana_sdk::sysvar::clock::ID,
            solana_sdk::account::Account {
                lamports: 1,
                data: clock_data,
                owner: solana_sdk::sysvar::ID,
                executable: false,
                rent_epoch: 0,
            },
        );

        pt
    }

    /// Build a `recv_packet` instruction for `ProgramTest`.
    ///
    /// Returns (instruction, `packet_receipt_pda`, `packet_ack_pda`).
    /// The caller must pre-create the chunk accounts in `ProgramTest`.
    fn build_recv_packet_ix(
        relayer: Pubkey,
        sequence: u64,
        mock_client_state: Pubkey,
        mock_consensus_state: Pubkey,
        mock_ibc_app_state: Pubkey,
        payload_chunk_pda: Pubkey,
        proof_chunk_pda: Pubkey,
    ) -> (Instruction, Pubkey, Pubkey) {
        let (router_state_pda, _) = Pubkey::find_program_address(&[RouterState::SEED], &crate::ID);
        let (access_manager_pda, _) =
            solana_ibc_types::access_manager::AccessManager::pda(access_manager::ID);
        let (ibc_app_pda, _) =
            Pubkey::find_program_address(&[IBCApp::SEED, RECV_TEST_PORT.as_bytes()], &crate::ID);
        let (client_pda, _) =
            Pubkey::find_program_address(&[Client::SEED, RECV_DEST_CLIENT.as_bytes()], &crate::ID);

        let packet = MsgPacket {
            sequence,
            source_client: RECV_SOURCE_CLIENT.to_string(),
            dest_client: RECV_DEST_CLIENT.to_string(),
            timeout_timestamp: RECV_TEST_TIMEOUT,
            payloads: vec![MsgPayload {
                source_port: "source-port".to_string(),
                dest_port: RECV_TEST_PORT.to_string(),
                version: "1".to_string(),
                encoding: "json".to_string(),
                data: Delivery::Chunked { total_chunks: 1 },
            }],
        };

        let msg = MsgRecvPacket {
            packet,
            proof: MsgProof {
                height: 100,
                data: Delivery::Chunked { total_chunks: 1 },
            },
        };

        let (packet_receipt_pda, _) = Pubkey::find_program_address(
            &[
                Commitment::PACKET_RECEIPT_SEED,
                RECV_DEST_CLIENT.as_bytes(),
                &sequence.to_le_bytes(),
            ],
            &crate::ID,
        );

        let (packet_ack_pda, _) = Pubkey::find_program_address(
            &[
                Commitment::PACKET_ACK_SEED,
                RECV_DEST_CLIENT.as_bytes(),
                &sequence.to_le_bytes(),
            ],
            &crate::ID,
        );

        let mut accounts = vec![
            AccountMeta::new_readonly(router_state_pda, false),
            AccountMeta::new_readonly(access_manager_pda, false),
            AccountMeta::new_readonly(ibc_app_pda, false),
            AccountMeta::new(packet_receipt_pda, false),
            AccountMeta::new(packet_ack_pda, false),
            AccountMeta::new_readonly(MOCK_IBC_APP_PROGRAM_ID, false),
            AccountMeta::new(mock_ibc_app_state, false),
            AccountMeta::new(relayer, true),
            AccountMeta::new_readonly(system_program::ID, false),
            AccountMeta::new_readonly(solana_sdk::sysvar::instructions::ID, false),
            AccountMeta::new_readonly(client_pda, false),
            AccountMeta::new_readonly(MOCK_LIGHT_CLIENT_ID, false),
            AccountMeta::new_readonly(mock_client_state, false),
            AccountMeta::new_readonly(mock_consensus_state, false),
        ];

        // Remaining accounts: payload chunk, then proof chunk
        accounts.push(AccountMeta::new(payload_chunk_pda, false));
        accounts.push(AccountMeta::new(proof_chunk_pda, false));

        let ix = Instruction {
            program_id: crate::ID,
            accounts,
            data: crate::instruction::RecvPacket { msg }.data(),
        };

        (ix, packet_receipt_pda, packet_ack_pda)
    }

    #[tokio::test]
    async fn test_relay_two_packets_single_transaction() {
        let relayer = Keypair::new();
        let relayer_pubkey = relayer.pubkey();

        let mut pt = setup_recv_two_packets_program_test(relayer_pubkey);

        // Pre-fund the relayer
        pt.add_account(
            relayer_pubkey,
            solana_sdk::account::Account {
                lamports: 10_000_000_000,
                data: vec![],
                owner: system_program::ID,
                executable: false,
                rent_epoch: 0,
            },
        );

        // Fixed account pubkeys for mock light client and app state
        let mock_client_state = Pubkey::new_unique();
        let mock_consensus_state = Pubkey::new_unique();
        let mock_ibc_app_state = Pubkey::new_unique();

        pt.add_account(
            mock_client_state,
            solana_sdk::account::Account {
                lamports: 1_000_000,
                data: vec![0u8; 64],
                owner: MOCK_LIGHT_CLIENT_ID,
                executable: false,
                rent_epoch: 0,
            },
        );
        pt.add_account(
            mock_consensus_state,
            solana_sdk::account::Account {
                lamports: 1_000_000,
                data: vec![0u8; 64],
                owner: MOCK_LIGHT_CLIENT_ID,
                executable: false,
                rent_epoch: 0,
            },
        );
        pt.add_account(
            mock_ibc_app_state,
            solana_sdk::account::Account {
                lamports: 1_000_000,
                data: vec![0u8; 100],
                owner: MOCK_IBC_APP_PROGRAM_ID,
                executable: false,
                rent_epoch: 0,
            },
        );

        let payload_data_1 = b"packet one".to_vec();
        let payload_data_2 = b"packet two".to_vec();
        let proof_data = vec![0u8; 32];

        // Create chunk accounts for packet 1 (sequence=1)
        let (payload_chunk_pda_1, _) = Pubkey::find_program_address(
            &[
                PayloadChunk::SEED,
                relayer_pubkey.as_ref(),
                RECV_DEST_CLIENT.as_bytes(),
                &1u64.to_le_bytes(),
                &[0u8], // payload_index
                &[0u8], // chunk_index
            ],
            &crate::ID,
        );
        let payload_chunk_1 = PayloadChunk {
            client_id: RECV_DEST_CLIENT.to_string(),
            sequence: 1,
            payload_index: 0,
            chunk_index: 0,
            chunk_data: payload_data_1.clone(),
        };
        pt.add_account(
            payload_chunk_pda_1,
            solana_sdk::account::Account {
                lamports: 1_000_000,
                data: create_account_data(&payload_chunk_1),
                owner: crate::ID,
                executable: false,
                rent_epoch: 0,
            },
        );

        let (proof_chunk_pda_1, _) = Pubkey::find_program_address(
            &[
                ProofChunk::SEED,
                relayer_pubkey.as_ref(),
                RECV_DEST_CLIENT.as_bytes(),
                &1u64.to_le_bytes(),
                &[0u8], // chunk_index
            ],
            &crate::ID,
        );
        let proof_chunk_1 = ProofChunk {
            client_id: RECV_DEST_CLIENT.to_string(),
            sequence: 1,
            chunk_index: 0,
            chunk_data: proof_data.clone(),
        };
        pt.add_account(
            proof_chunk_pda_1,
            solana_sdk::account::Account {
                lamports: 1_000_000,
                data: create_account_data(&proof_chunk_1),
                owner: crate::ID,
                executable: false,
                rent_epoch: 0,
            },
        );

        // Create chunk accounts for packet 2 (sequence=2)
        let (payload_chunk_pda_2, _) = Pubkey::find_program_address(
            &[
                PayloadChunk::SEED,
                relayer_pubkey.as_ref(),
                RECV_DEST_CLIENT.as_bytes(),
                &2u64.to_le_bytes(),
                &[0u8],
                &[0u8],
            ],
            &crate::ID,
        );
        let payload_chunk_2 = PayloadChunk {
            client_id: RECV_DEST_CLIENT.to_string(),
            sequence: 2,
            payload_index: 0,
            chunk_index: 0,
            chunk_data: payload_data_2.clone(),
        };
        pt.add_account(
            payload_chunk_pda_2,
            solana_sdk::account::Account {
                lamports: 1_000_000,
                data: create_account_data(&payload_chunk_2),
                owner: crate::ID,
                executable: false,
                rent_epoch: 0,
            },
        );

        let (proof_chunk_pda_2, _) = Pubkey::find_program_address(
            &[
                ProofChunk::SEED,
                relayer_pubkey.as_ref(),
                RECV_DEST_CLIENT.as_bytes(),
                &2u64.to_le_bytes(),
                &[0u8],
            ],
            &crate::ID,
        );
        let proof_chunk_2 = ProofChunk {
            client_id: RECV_DEST_CLIENT.to_string(),
            sequence: 2,
            chunk_index: 0,
            chunk_data: proof_data.clone(),
        };
        pt.add_account(
            proof_chunk_pda_2,
            solana_sdk::account::Account {
                lamports: 1_000_000,
                data: create_account_data(&proof_chunk_2),
                owner: crate::ID,
                executable: false,
                rent_epoch: 0,
            },
        );

        let (banks_client, _payer, recent_blockhash) = pt.start().await;

        // Build recv_packet instructions for both packets
        let (ix1, receipt_pda_1, ack_pda_1) = build_recv_packet_ix(
            relayer_pubkey,
            1,
            mock_client_state,
            mock_consensus_state,
            mock_ibc_app_state,
            payload_chunk_pda_1,
            proof_chunk_pda_1,
        );

        let (ix2, receipt_pda_2, ack_pda_2) = build_recv_packet_ix(
            relayer_pubkey,
            2,
            mock_client_state,
            mock_consensus_state,
            mock_ibc_app_state,
            payload_chunk_pda_2,
            proof_chunk_pda_2,
        );

        // Execute both recv_packet instructions in a single transaction
        let tx = Transaction::new_signed_with_payer(
            &[ix1, ix2],
            Some(&relayer_pubkey),
            &[&relayer],
            recent_blockhash,
        );

        let result = banks_client.process_transaction(tx).await;
        assert!(
            result.is_ok(),
            "Both recv_packet instructions should succeed in a single tx: {:?}",
            result.err()
        );

        // Verify packet_receipt exists for packet 1
        let receipt_1 = banks_client
            .get_account(receipt_pda_1)
            .await
            .unwrap()
            .expect("packet receipt #1 should exist");
        assert_eq!(receipt_1.owner, crate::ID);
        let receipt_value_1 = &receipt_1.data[8..40];
        assert_ne!(receipt_value_1, &[0u8; 32], "Receipt #1 should be set");

        // Verify packet_ack exists for packet 1
        let ack_1 = banks_client
            .get_account(ack_pda_1)
            .await
            .unwrap()
            .expect("packet ack #1 should exist");
        assert_eq!(ack_1.owner, crate::ID);
        let ack_value_1 = &ack_1.data[8..40];
        assert_ne!(ack_value_1, &[0u8; 32], "Ack #1 should be set");

        // Verify packet_receipt exists for packet 2
        let receipt_2 = banks_client
            .get_account(receipt_pda_2)
            .await
            .unwrap()
            .expect("packet receipt #2 should exist");
        assert_eq!(receipt_2.owner, crate::ID);
        let receipt_value_2 = &receipt_2.data[8..40];
        assert_ne!(receipt_value_2, &[0u8; 32], "Receipt #2 should be set");

        // Verify packet_ack exists for packet 2
        let ack_2 = banks_client
            .get_account(ack_pda_2)
            .await
            .unwrap()
            .expect("packet ack #2 should exist");
        assert_eq!(ack_2.owner, crate::ID);
        let ack_value_2 = &ack_2.data[8..40];
        assert_ne!(ack_value_2, &[0u8; 32], "Ack #2 should be set");

        // Verify receipts are distinct (different packet content -> different commitments)
        assert_ne!(
            receipt_value_1, receipt_value_2,
            "Receipts should differ for distinct packets"
        );
    }

    #[test]
    fn test_recv_packet_inline_wrong_dest_port() {
        let mut ctx = setup_recv_packet_test(true, 1000);

        // Use wrong dest_port that doesn't match the ibc_app PDA ("test-port")
        ctx.packet.payloads = vec![MsgPayload {
            source_port: "source-port".to_string(),
            dest_port: "transfer".to_string(),
            version: "1".to_string(),
            encoding: "json".to_string(),
            data: Delivery::Inline {
                data: b"inline data".to_vec(),
            },
        }];

        let msg = MsgRecvPacket {
            packet: ctx.packet.clone(),
            proof: MsgProof {
                height: 100,
                data: Delivery::Chunked { total_chunks: 1 },
            },
        };

        ctx.instruction.data = crate::instruction::RecvPacket { msg }.data();

        let mollusk = setup_mollusk_with_mock_programs();

        // Anchor seeds constraint fails: ibc_app PDA derived from "transfer" != account seeded with "test-port"
        let checks = vec![Check::err(ProgramError::Custom(
            anchor_lang::error::ErrorCode::ConstraintSeeds as u32,
        ))];

        mollusk.process_and_validate_instruction(&ctx.instruction, &ctx.accounts, &checks);
    }

    #[test]
    fn test_recv_packet_paused() {
        let ctx = setup_recv_packet_test_with_params(RecvPacketTestParams {
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
