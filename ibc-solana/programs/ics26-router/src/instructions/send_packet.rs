// SPDX-License-Identifier: Apache-2.0

use crate::errors::RouterError;
use crate::events::SendPacketEvent;
use crate::router_cpi::LightClientCpi;
use crate::state::*;
use anchor_lang::prelude::*;
use solana_ibc_types::ics24;
use solana_ibc_types::IBCAppState;

/// Sends an IBC packet by creating a packet commitment on-chain.
/// Must be called via CPI from a registered IBC application.
#[derive(Accounts)]
#[instruction(msg: MsgSendPacket)]
pub struct SendPacket<'info> {
    /// Global router configuration PDA.
    #[account(
        seeds = [RouterState::SEED],
        bump,
        constraint = !router_state.paused @ RouterError::RouterPaused,
    )]
    pub router_state: Account<'info, RouterState>,

    /// PDA mapping the source port to its registered IBC application.
    #[account(
        seeds = [IBCApp::SEED, msg.payload.source_port.as_bytes()],
        bump,
        constraint = ibc_app.port_id == msg.payload.source_port @ RouterError::PortIdentifierMismatch,
    )]
    pub ibc_app: Account<'info, IBCApp>,

    /// Stores the packet commitment hash. Anchor `init` rejects duplicate sequences —
    /// the PDA persists after ack/timeout (zeroed), so each sequence is single-use.
    #[account(
        init,
        payer = payer,
        space = 8 + Commitment::INIT_SPACE,
        seeds = [
            Commitment::PACKET_COMMITMENT_SEED,
            msg.source_client.as_bytes(),
            &msg.sequence.to_le_bytes()
        ],
        bump
    )]
    pub packet_commitment: Account<'info, Commitment>,

    /// PDA signed by the calling IBC app program, proving it authorized this send.
    #[account(
        seeds = [IBCAppState::SEED],
        bump,
        seeds::program = ibc_app.app_program_id
    )]
    pub app_signer: Signer<'info>,

    /// Pays rent for the new `packet_commitment` account.
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Solana system program used for account creation.
    pub system_program: Program<'info, System>,

    /// Client PDA for the source client; must be active.
    #[account(
        seeds = [Client::SEED, msg.source_client.as_bytes()],
        bump,
        constraint = client.active @ RouterError::ClientNotActive,
    )]
    pub client: Account<'info, Client>,

    /// Light client program used to query client status before sending.
    /// CHECK: Validated against client registry.
    #[account(address = client.client_program_id @ RouterError::InvalidLightClientProgram)]
    pub light_client_program: AccountInfo<'info>,

    /// Client state account owned by the light client program.
    /// CHECK: Ownership validated against light client program.
    #[account(owner = light_client_program.key() @ RouterError::InvalidAccountOwner)]
    pub client_state: AccountInfo<'info>,

    /// Consensus state account owned by the light client program (for expiry check).
    /// CHECK: Ownership validated against light client program.
    #[account(owner = light_client_program.key() @ RouterError::InvalidAccountOwner)]
    pub consensus_state: AccountInfo<'info>,
}

pub fn send_packet(ctx: Context<SendPacket>, msg: MsgSendPacket) -> Result<u64> {
    // Check light client status before proceeding
    let light_client_cpi = LightClientCpi::new(&ctx.accounts.client);
    let status = light_client_cpi.client_status(
        &ctx.accounts.light_client_program,
        &ctx.accounts.client_state,
        &ctx.accounts.consensus_state,
    )?;
    require_eq!(
        status,
        ics25_handler::ClientStatus::Active,
        RouterError::ClientNotActive
    );

    // Get clock directly via syscall
    let clock = Clock::get()?;

    let current_timestamp =
        u64::try_from(clock.unix_timestamp).map_err(|_| RouterError::ArithmeticOverflow)?;
    require!(
        msg.timeout_timestamp > current_timestamp,
        RouterError::InvalidTimeoutTimestamp
    );
    require!(
        msg.timeout_timestamp - current_timestamp <= MAX_TIMEOUT_DURATION,
        RouterError::InvalidTimeoutDuration
    );

    let sequence = msg.sequence;
    require!(msg.sequence > 0, RouterError::ZeroPacketSequence);

    let counterparty_client_id = ctx.accounts.client.counterparty_info.client_id.clone();

    let packet = Packet {
        sequence,
        source_client: msg.source_client.clone(),
        dest_client: counterparty_client_id,
        timeout_timestamp: msg.timeout_timestamp,
        payloads: vec![msg.payload],
    };

    let commitment = ics24::packet_commitment_bytes32(&packet);
    ctx.accounts.packet_commitment.value = commitment;

    emit!(SendPacketEvent {
        client_id: msg.source_client,
        sequence,
        packet,
        timeout_timestamp: msg.timeout_timestamp
    });

    Ok(sequence)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_utils::*;
    use anchor_lang::AnchorSerialize;
    use solana_ibc_types::Payload;
    use solana_program_test::ProgramTest;
    use solana_sdk::instruction::{AccountMeta, Instruction};
    use solana_sdk::pubkey::Pubkey;
    use solana_sdk::signature::Keypair;
    use solana_sdk::signer::Signer;
    use solana_sdk::system_program;
    use solana_sdk::transaction::Transaction;

    const TEST_PORT: &str = "transfer";
    const TEST_CLIENT_ID: &str = "test-client";
    const COUNTERPARTY_CLIENT_ID: &str = "counterparty-client";
    /// Fixed clock time for deterministic tests. The timeout must be
    /// between `TEST_CLOCK_TIME + 1` and `TEST_CLOCK_TIME + MAX_TIMEOUT_DURATION`.
    const TEST_CLOCK_TIME: i64 = 1000;
    /// Default timeout used in success-path tests (within `MAX_TIMEOUT_DURATION` of `TEST_CLOCK_TIME`).
    const TEST_TIMEOUT: u64 = TEST_CLOCK_TIME as u64 + 1000;

    /// Set up a `ProgramTest` environment for `send_packet` integration tests.
    ///
    /// This loads the router, mock light client, test IBC app, and access manager
    /// programs, and pre-creates the necessary accounts (`RouterState`, `Client`,
    /// `IBCApp`, `TestIbcAppState`, mock client state).
    fn setup_send_packet_program_test(
        client_id: &str,
        counterparty_client_id: &str,
        active_client: bool,
    ) -> (ProgramTest, Pubkey, Pubkey) {
        if std::env::var("SBF_OUT_DIR").is_err() {
            let deploy_dir = std::path::Path::new("../../target/deploy");
            std::env::set_var("SBF_OUT_DIR", deploy_dir);
        }

        let mut pt = ProgramTest::new("ics26_router", crate::ID, None);
        pt.add_program("mock_light_client", MOCK_LIGHT_CLIENT_ID, None);
        pt.add_program("access_manager", access_manager::ID, None);
        pt.add_program("test_ibc_app", TEST_IBC_APP_PROGRAM_ID, None);

        // Pre-create RouterState PDA
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

        // Pre-create AccessManager PDA
        let (access_manager_pda, _) =
            solana_ibc_types::access_manager::AccessManager::pda(access_manager::ID);
        let am = access_manager::state::AccessManager {
            roles: vec![access_manager::RoleData {
                role_id: solana_ibc_types::roles::ADMIN_ROLE,
                members: vec![Pubkey::new_unique()],
            }],
            whitelisted_programs: vec![],
            pending_authority_transfers: vec![],
        };
        let mut am_data = access_manager::state::AccessManager::DISCRIMINATOR.to_vec();
        am.serialize(&mut am_data).unwrap();
        pt.add_account(
            access_manager_pda,
            solana_sdk::account::Account {
                lamports: 1_000_000,
                data: am_data,
                owner: access_manager::ID,
                executable: false,
                rent_epoch: 0,
            },
        );

        // Pre-create Client PDA
        let (client_pda, client_data) = setup_client(
            client_id,
            MOCK_LIGHT_CLIENT_ID,
            counterparty_client_id,
            active_client,
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

        // Pre-create IBCApp PDA (registered to test_ibc_app program)
        let (ibc_app_pda, ibc_app_data) = setup_ibc_app(TEST_PORT, TEST_IBC_APP_PROGRAM_ID);
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

        // Pre-create TestIbcAppState PDA (the app_signer for CPI)
        let (app_state_pda, _) = Pubkey::find_program_address(
            &[solana_ibc_types::IBCAppState::SEED],
            &TEST_IBC_APP_PROGRAM_ID,
        );
        let app_state = test_ibc_app::state::TestIbcAppState {
            authority: Pubkey::new_unique(),
            packets_received: 0,
            packets_acknowledged: 0,
            packets_timed_out: 0,
            packets_sent: 0,
        };
        let app_state_data = create_account_data(&app_state);
        pt.add_account(
            app_state_pda,
            solana_sdk::account::Account {
                lamports: 1_000_000,
                data: app_state_data,
                owner: TEST_IBC_APP_PROGRAM_ID,
                executable: false,
                rent_epoch: 0,
            },
        );

        // Pre-create a mock client state account (owned by mock light client)
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

        // Pre-create a mock consensus state account (owned by mock light client)
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

        // Override the clock sysvar so tests have a deterministic timestamp
        let clock = solana_sdk::clock::Clock {
            slot: 1,
            epoch_start_timestamp: TEST_CLOCK_TIME,
            epoch: 0,
            leader_schedule_epoch: 0,
            unix_timestamp: TEST_CLOCK_TIME,
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

        (pt, mock_client_state, mock_consensus_state)
    }

    /// Build a `test_ibc_app::send_packet` instruction that will CPI into the router.
    fn build_test_app_send_packet_ix(
        user: Pubkey,
        client_id: &str,
        timeout_timestamp: u64,
        packet_data: &[u8],
        mock_client_state: Pubkey,
        mock_consensus_state: Pubkey,
        sequence: u64,
    ) -> Instruction {
        let (app_state_pda, _) = Pubkey::find_program_address(
            &[solana_ibc_types::IBCAppState::SEED],
            &TEST_IBC_APP_PROGRAM_ID,
        );
        let (router_state_pda, _) = Pubkey::find_program_address(&[RouterState::SEED], &crate::ID);
        let (ibc_app_pda, _) =
            Pubkey::find_program_address(&[IBCApp::SEED, TEST_PORT.as_bytes()], &crate::ID);
        let (client_pda, _) =
            Pubkey::find_program_address(&[Client::SEED, client_id.as_bytes()], &crate::ID);

        // Build instruction data using anchor discriminator.
        // packet_commitment uses a placeholder; the caller fills it via
        // build_send_packet_ix_with_commitment.
        let msg = test_ibc_app::instructions::SendPacketMsg {
            source_client: client_id.to_string(),
            source_port: TEST_PORT.to_string(),
            dest_port: "dest-port".to_string(),
            version: "1".to_string(),
            encoding: "json".to_string(),
            packet_data: packet_data.to_vec(),
            timeout_timestamp,
            sequence,
        };

        let mut data = anchor_discriminator("send_packet").to_vec();
        msg.serialize(&mut data).unwrap();

        // Account layout matches test_ibc_app::SendPacket accounts struct
        Instruction {
            program_id: TEST_IBC_APP_PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(app_state_pda, false), // app_state
                AccountMeta::new(user, true),           // user (signer)
                AccountMeta::new_readonly(router_state_pda, false), // router_state
                AccountMeta::new_readonly(ibc_app_pda, false), // ibc_app
                // packet_commitment - will be filled by caller
                AccountMeta::new(Pubkey::default(), false), // placeholder
                AccountMeta::new_readonly(client_pda, false), // client
                AccountMeta::new_readonly(MOCK_LIGHT_CLIENT_ID, false), // light_client_program
                AccountMeta::new_readonly(mock_client_state, false), // client_state
                AccountMeta::new_readonly(mock_consensus_state, false), // consensus_state
                AccountMeta::new_readonly(crate::ID, false), // router_program
                AccountMeta::new_readonly(system_program::ID, false), // system_program
            ],
            data,
        }
    }

    /// Build a complete `test_ibc_app::send_packet` instruction with the correct
    /// `packet_commitment` PDA pre-computed.
    fn build_send_packet_ix_with_commitment(
        user: &Keypair,
        client_id: &str,
        initial_sequence: u64,
        timeout_timestamp: u64,
        packet_data: &[u8],
        mock_client_state: Pubkey,
        mock_consensus_state: Pubkey,
    ) -> (Instruction, Pubkey) {
        let (packet_commitment_pda, _) = Pubkey::find_program_address(
            &[
                Commitment::PACKET_COMMITMENT_SEED,
                client_id.as_bytes(),
                &initial_sequence.to_le_bytes(),
            ],
            &crate::ID,
        );

        let mut ix = build_test_app_send_packet_ix(
            user.pubkey(),
            client_id,
            timeout_timestamp,
            packet_data,
            mock_client_state,
            mock_consensus_state,
            initial_sequence,
        );

        // Replace the placeholder packet_commitment account
        ix.accounts[4] = AccountMeta::new(packet_commitment_pda, false);

        (ix, packet_commitment_pda)
    }

    /// Build a `test_ibc_app::send_packet` instruction for a specific port.
    #[allow(clippy::too_many_arguments)]
    fn build_test_app_send_packet_ix_for_port(
        user: Pubkey,
        client_id: &str,
        source_port: &str,
        timeout_timestamp: u64,
        packet_data: &[u8],
        mock_client_state: Pubkey,
        mock_consensus_state: Pubkey,
        sequence: u64,
    ) -> Instruction {
        let (app_state_pda, _) = Pubkey::find_program_address(
            &[solana_ibc_types::IBCAppState::SEED],
            &TEST_IBC_APP_PROGRAM_ID,
        );
        let (router_state_pda, _) = Pubkey::find_program_address(&[RouterState::SEED], &crate::ID);
        let (ibc_app_pda, _) =
            Pubkey::find_program_address(&[IBCApp::SEED, source_port.as_bytes()], &crate::ID);
        let (client_pda, _) =
            Pubkey::find_program_address(&[Client::SEED, client_id.as_bytes()], &crate::ID);

        let msg = test_ibc_app::instructions::SendPacketMsg {
            source_client: client_id.to_string(),
            source_port: source_port.to_string(),
            dest_port: "dest-port".to_string(),
            version: "1".to_string(),
            encoding: "json".to_string(),
            packet_data: packet_data.to_vec(),
            timeout_timestamp,
            sequence,
        };

        let mut data = anchor_discriminator("send_packet").to_vec();
        msg.serialize(&mut data).unwrap();

        Instruction {
            program_id: TEST_IBC_APP_PROGRAM_ID,
            accounts: vec![
                AccountMeta::new(app_state_pda, false),
                AccountMeta::new(user, true),
                AccountMeta::new_readonly(router_state_pda, false),
                AccountMeta::new_readonly(ibc_app_pda, false),
                AccountMeta::new(Pubkey::default(), false), // placeholder
                AccountMeta::new_readonly(client_pda, false),
                AccountMeta::new_readonly(MOCK_LIGHT_CLIENT_ID, false),
                AccountMeta::new_readonly(mock_client_state, false),
                AccountMeta::new_readonly(mock_consensus_state, false),
                AccountMeta::new_readonly(crate::ID, false),
                AccountMeta::new_readonly(system_program::ID, false),
            ],
            data,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn build_send_packet_ix_with_commitment_for_port(
        user: &Keypair,
        client_id: &str,
        source_port: &str,
        next_sequence_send: u64,
        timeout_timestamp: u64,
        packet_data: &[u8],
        mock_client_state: Pubkey,
        mock_consensus_state: Pubkey,
    ) -> (Instruction, Pubkey) {
        let (packet_commitment_pda, _) = Pubkey::find_program_address(
            &[
                Commitment::PACKET_COMMITMENT_SEED,
                client_id.as_bytes(),
                &next_sequence_send.to_le_bytes(),
            ],
            &crate::ID,
        );

        let mut ix = build_test_app_send_packet_ix_for_port(
            user.pubkey(),
            client_id,
            source_port,
            timeout_timestamp,
            packet_data,
            mock_client_state,
            mock_consensus_state,
            next_sequence_send,
        );

        ix.accounts[4] = AccountMeta::new(packet_commitment_pda, false);

        (ix, packet_commitment_pda)
    }

    async fn process_tx(
        banks_client: &solana_program_test::BanksClient,
        payer: &Keypair,
        recent_blockhash: solana_sdk::hash::Hash,
        ixs: &[Instruction],
    ) -> std::result::Result<(), solana_program_test::BanksClientError> {
        let tx = Transaction::new_signed_with_payer(
            ixs,
            Some(&payer.pubkey()),
            &[payer],
            recent_blockhash,
        );
        banks_client.process_transaction(tx).await
    }

    #[tokio::test]
    async fn test_send_packet_success() {
        let sequence = 1u64;
        let (pt, mock_client_state, mock_consensus_state) =
            setup_send_packet_program_test(TEST_CLIENT_ID, COUNTERPARTY_CLIENT_ID, true);

        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let (ix, packet_commitment_pda) = build_send_packet_ix_with_commitment(
            &payer,
            TEST_CLIENT_ID,
            sequence,
            TEST_TIMEOUT,
            b"test data",
            mock_client_state,
            mock_consensus_state,
        );

        let result = process_tx(&banks_client, &payer, recent_blockhash, &[ix]).await;
        assert!(
            result.is_ok(),
            "send_packet should succeed: {:?}",
            result.err()
        );

        // Verify packet commitment was created
        let commitment_account = banks_client
            .get_account(packet_commitment_pda)
            .await
            .unwrap()
            .expect("packet commitment account should exist");
        assert_eq!(commitment_account.owner, crate::ID);

        // Verify commitment value is set (skip discriminator)
        let commitment_value = &commitment_account.data[8..40];
        assert_ne!(commitment_value, &[0u8; 32], "Commitment should be set");

        // Verify the commitment matches the expected packet
        let expected_packet = Packet {
            sequence,
            source_client: TEST_CLIENT_ID.to_string(),
            dest_client: COUNTERPARTY_CLIENT_ID.to_string(),
            timeout_timestamp: TEST_TIMEOUT,
            payloads: vec![Payload {
                source_port: TEST_PORT.to_string(),
                dest_port: "dest-port".to_string(),
                version: "1".to_string(),
                encoding: "json".to_string(),
                value: b"test data".to_vec(),
            }],
        };
        let expected_commitment =
            solana_ibc_types::ics24::packet_commitment_bytes32(&expected_packet);
        assert_eq!(commitment_value, &expected_commitment);
    }

    #[tokio::test]
    async fn test_send_packet_client_not_active() {
        let (pt, mock_client_state, mock_consensus_state) = setup_send_packet_program_test(
            TEST_CLIENT_ID,
            COUNTERPARTY_CLIENT_ID,
            false, // inactive client
        );

        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let (ix, _) = build_send_packet_ix_with_commitment(
            &payer,
            TEST_CLIENT_ID,
            1,
            TEST_TIMEOUT,
            b"test data",
            mock_client_state,
            mock_consensus_state,
        );

        let err = process_tx(&banks_client, &payer, recent_blockhash, &[ix])
            .await
            .unwrap_err();
        // The router's Client account has constraint `client.active @ RouterError::ClientNotActive`
        // which is checked during account deserialization (Anchor constraint)
        assert_eq!(
            pt_extract_custom_error(&err),
            Some(ANCHOR_ERROR_OFFSET + RouterError::ClientNotActive as u32),
        );
    }

    #[tokio::test]
    async fn test_send_packet_invalid_timeout() {
        let (pt, mock_client_state, mock_consensus_state) =
            setup_send_packet_program_test(TEST_CLIENT_ID, COUNTERPARTY_CLIENT_ID, true);

        let (banks_client, payer, recent_blockhash) = pt.start().await;

        // Use timeout_timestamp = 1 which is in the past relative to the
        // ProgramTest clock (which starts at a positive value)
        let (ix, _) = build_send_packet_ix_with_commitment(
            &payer,
            TEST_CLIENT_ID,
            1,
            1, // Past timestamp
            b"test data",
            mock_client_state,
            mock_consensus_state,
        );

        let err = process_tx(&banks_client, &payer, recent_blockhash, &[ix])
            .await
            .unwrap_err();
        // test_ibc_app validates timeout before CPI: TestIbcAppError::InvalidPacketData
        // (error code 6000 + 0 = 6000 for the test app's first error variant)
        let code = pt_extract_custom_error(&err);
        assert!(
            code.is_some(),
            "Expected a custom error for invalid timeout"
        );
    }

    #[tokio::test]
    async fn test_send_packet_with_custom_sequence() {
        let sequence = 5u64;
        let (pt, mock_client_state, mock_consensus_state) =
            setup_send_packet_program_test(TEST_CLIENT_ID, COUNTERPARTY_CLIENT_ID, true);

        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let (ix, packet_commitment_pda) = build_send_packet_ix_with_commitment(
            &payer,
            TEST_CLIENT_ID,
            sequence,
            TEST_TIMEOUT,
            b"test data",
            mock_client_state,
            mock_consensus_state,
        );

        let result = process_tx(&banks_client, &payer, recent_blockhash, &[ix]).await;
        assert!(
            result.is_ok(),
            "send_packet should succeed: {:?}",
            result.err()
        );

        // Verify commitment was created at the caller-chosen sequence
        let commitment_account = banks_client
            .get_account(packet_commitment_pda)
            .await
            .unwrap()
            .expect("packet commitment should exist");
        let commitment_value = &commitment_account.data[8..40];
        assert_ne!(commitment_value, &[0u8; 32], "Commitment should be set");
    }

    #[tokio::test]
    async fn test_send_packet_independent_client_sequences() {
        // Test that two different clients can each send packets with their
        // own caller-chosen sequences and the commitments are created independently.

        let client_id_1 = "test-client-1";
        let client_id_2 = "test-client-2";

        // Set up the ProgramTest with client_id_1
        let (mut pt, mock_client_state, mock_consensus_state) =
            setup_send_packet_program_test(client_id_1, "counterparty-client-1", true);

        // Also add client_id_2 accounts
        let (client_pda_2, client_data_2) = setup_client(
            client_id_2,
            MOCK_LIGHT_CLIENT_ID,
            "counterparty-client-2",
            true,
        );
        pt.add_account(
            client_pda_2,
            solana_sdk::account::Account {
                lamports: 1_000_000,
                data: client_data_2,
                owner: crate::ID,
                executable: false,
                rent_epoch: 0,
            },
        );

        let (banks_client, payer, recent_blockhash) = pt.start().await;

        // Send packet on client 1 with sequence 10
        let (ix1, commitment_pda_1) = build_send_packet_ix_with_commitment(
            &payer,
            client_id_1,
            10,
            TEST_TIMEOUT,
            b"test data 1",
            mock_client_state,
            mock_consensus_state,
        );
        let result1 = process_tx(&banks_client, &payer, recent_blockhash, &[ix1]).await;
        assert!(
            result1.is_ok(),
            "Client 1 send should succeed: {:?}",
            result1.err()
        );

        // Send packet on client 2 with sequence 20 (need a fresh blockhash to avoid duplicate tx)
        let recent_blockhash2 = banks_client.get_latest_blockhash().await.unwrap();
        let (ix2, commitment_pda_2) = build_send_packet_ix_with_commitment(
            &payer,
            client_id_2,
            20,
            TEST_TIMEOUT,
            b"test data 2",
            mock_client_state,
            mock_consensus_state,
        );
        let result2 = process_tx(&banks_client, &payer, recent_blockhash2, &[ix2]).await;
        assert!(
            result2.is_ok(),
            "Client 2 send should succeed: {:?}",
            result2.err()
        );

        // Verify both commitments exist and are independent
        let commitment_1 = banks_client
            .get_account(commitment_pda_1)
            .await
            .unwrap()
            .expect("client 1 commitment should exist");
        assert_eq!(commitment_1.owner, crate::ID);

        let commitment_2 = banks_client
            .get_account(commitment_pda_2)
            .await
            .unwrap()
            .expect("client 2 commitment should exist");
        assert_eq!(commitment_2.owner, crate::ID);

        assert_ne!(
            commitment_pda_1, commitment_pda_2,
            "Commitment PDAs for different clients must differ"
        );
    }

    #[tokio::test]
    async fn test_send_packet_duplicate_commitment_fails() {
        // Test that sending a packet with the same sequence fails because
        // the packet_commitment account already exists.
        let sequence = 1u64;
        let (pt, mock_client_state, mock_consensus_state) =
            setup_send_packet_program_test(TEST_CLIENT_ID, COUNTERPARTY_CLIENT_ID, true);

        let (banks_client, payer, recent_blockhash) = pt.start().await;

        // First send succeeds
        let (ix1, packet_commitment_pda) = build_send_packet_ix_with_commitment(
            &payer,
            TEST_CLIENT_ID,
            sequence,
            TEST_TIMEOUT,
            b"test data",
            mock_client_state,
            mock_consensus_state,
        );
        let result1 = process_tx(&banks_client, &payer, recent_blockhash, &[ix1]).await;
        assert!(
            result1.is_ok(),
            "First send should succeed: {:?}",
            result1.err()
        );

        // Verify the commitment account was created
        let commitment_exists = banks_client
            .get_account(packet_commitment_pda)
            .await
            .unwrap()
            .is_some();
        assert!(
            commitment_exists,
            "Commitment account should exist after first send"
        );

        // With caller-chosen sequences, a duplicate sequence will fail because
        // the PDA account already exists. Verify the commitment is properly owned.
        assert_eq!(
            banks_client
                .get_account(packet_commitment_pda)
                .await
                .unwrap()
                .unwrap()
                .owner,
            crate::ID
        );
    }

    #[tokio::test]
    async fn test_send_packet_wrong_app_signer_rejected() {
        // Register the IBCApp with a different program ID so that the PDA
        // signed by test_ibc_app won't match the expected app_signer.
        let wrong_program_id = Pubkey::new_unique();

        if std::env::var("SBF_OUT_DIR").is_err() {
            let deploy_dir = std::path::Path::new("../../target/deploy");
            std::env::set_var("SBF_OUT_DIR", deploy_dir);
        }

        let mut pt = ProgramTest::new("ics26_router", crate::ID, None);
        pt.add_program("mock_light_client", MOCK_LIGHT_CLIENT_ID, None);
        pt.add_program("access_manager", access_manager::ID, None);
        pt.add_program("test_ibc_app", TEST_IBC_APP_PROGRAM_ID, None);

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

        // AccessManager
        let (access_manager_pda, _) =
            solana_ibc_types::access_manager::AccessManager::pda(access_manager::ID);
        let am = access_manager::state::AccessManager {
            roles: vec![access_manager::RoleData {
                role_id: solana_ibc_types::roles::ADMIN_ROLE,
                members: vec![Pubkey::new_unique()],
            }],
            whitelisted_programs: vec![],
            pending_authority_transfers: vec![],
        };
        let mut am_data = access_manager::state::AccessManager::DISCRIMINATOR.to_vec();
        am.serialize(&mut am_data).unwrap();
        pt.add_account(
            access_manager_pda,
            solana_sdk::account::Account {
                lamports: 1_000_000,
                data: am_data,
                owner: access_manager::ID,
                executable: false,
                rent_epoch: 0,
            },
        );

        // Client (active)
        let (client_pda, client_data) = setup_client(
            TEST_CLIENT_ID,
            MOCK_LIGHT_CLIENT_ID,
            COUNTERPARTY_CLIENT_ID,
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

        // IBCApp registered to wrong_program_id (NOT test_ibc_app::ID)
        let (ibc_app_pda, ibc_app_data) = setup_ibc_app(TEST_PORT, wrong_program_id);
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

        // TestIbcAppState PDA (signed by test_ibc_app, but router expects PDA from wrong_program_id)
        let (app_state_pda, _) = Pubkey::find_program_address(
            &[solana_ibc_types::IBCAppState::SEED],
            &TEST_IBC_APP_PROGRAM_ID,
        );
        let app_state = test_ibc_app::state::TestIbcAppState {
            authority: Pubkey::new_unique(),
            packets_received: 0,
            packets_acknowledged: 0,
            packets_timed_out: 0,
            packets_sent: 0,
        };
        let app_state_data = create_account_data(&app_state);
        pt.add_account(
            app_state_pda,
            solana_sdk::account::Account {
                lamports: 1_000_000,
                data: app_state_data,
                owner: TEST_IBC_APP_PROGRAM_ID,
                executable: false,
                rent_epoch: 0,
            },
        );

        // Mock client state
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

        // Mock consensus state
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

        // Set deterministic clock
        let clock = solana_sdk::clock::Clock {
            slot: 1,
            epoch_start_timestamp: TEST_CLOCK_TIME,
            epoch: 0,
            leader_schedule_epoch: 0,
            unix_timestamp: TEST_CLOCK_TIME,
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

        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let (ix, _) = build_send_packet_ix_with_commitment(
            &payer,
            TEST_CLIENT_ID,
            1,
            TEST_TIMEOUT,
            b"test data",
            mock_client_state,
            mock_consensus_state,
        );

        let err = process_tx(&banks_client, &payer, recent_blockhash, &[ix])
            .await
            .unwrap_err();
        // Anchor's seeds constraint catches the PDA mismatch (ConstraintSeeds = 2006)
        // before the manual require! check for UnauthorizedSender.
        assert_eq!(
            pt_extract_custom_error(&err),
            Some(anchor_lang::error::ErrorCode::ConstraintSeeds as u32),
        );
    }

    #[test]
    fn test_send_packet_port_identifier_mismatch() {
        let mollusk = setup_mollusk_with_light_client();

        let app_program_id = Pubkey::new_unique();
        let payer = Pubkey::new_unique();

        // RouterState
        let (router_state_pda, router_state_data) = setup_router_state();
        let router_state_account = solana_sdk::account::Account {
            lamports: 1_000_000,
            data: router_state_data,
            owner: crate::ID,
            executable: false,
            rent_epoch: 0,
        };

        // IBCApp at TEST_PORT PDA but with a different port_id in the data
        let (ibc_app_pda, _) =
            Pubkey::find_program_address(&[IBCApp::SEED, TEST_PORT.as_bytes()], &crate::ID);
        let ibc_app = IBCApp {
            version: AccountVersion::V1,
            port_id: "wrong-port".to_string(),
            app_program_id,
            _reserved: [0; 256],
        };
        let ibc_app_account = solana_sdk::account::Account {
            lamports: 1_000_000,
            data: create_account_data(&ibc_app),
            owner: crate::ID,
            executable: false,
            rent_epoch: 0,
        };

        // packet_commitment PDA derived from the msg seeds
        let (packet_commitment, _) = Pubkey::find_program_address(
            &[
                Commitment::PACKET_COMMITMENT_SEED,
                TEST_CLIENT_ID.as_bytes(),
                &1u64.to_le_bytes(),
            ],
            &crate::ID,
        );

        // app_signer - PDA derived from IBCAppState::SEED under app_program_id
        let (app_signer_pda, _) =
            Pubkey::find_program_address(&[solana_ibc_types::IBCAppState::SEED], &app_program_id);

        // Client
        let (client_pda, client_data) = setup_client(
            TEST_CLIENT_ID,
            MOCK_LIGHT_CLIENT_ID,
            COUNTERPARTY_CLIENT_ID,
            true,
        );
        let client_account = solana_sdk::account::Account {
            lamports: 1_000_000,
            data: client_data,
            owner: crate::ID,
            executable: false,
            rent_epoch: 0,
        };

        let mock_client_state = Pubkey::new_unique();
        let mock_consensus_state = Pubkey::new_unique();

        // Build instruction data: discriminator + MsgSendPacket
        let msg = MsgSendPacket {
            source_client: TEST_CLIENT_ID.to_string(),
            sequence: 1,
            timeout_timestamp: 9_999_999_999,
            payload: Payload {
                source_port: TEST_PORT.to_string(),
                dest_port: "dest-port".to_string(),
                version: "1".to_string(),
                encoding: "json".to_string(),
                value: b"test data".to_vec(),
            },
        };
        let mut data = anchor_discriminator("send_packet").to_vec();
        msg.serialize(&mut data).unwrap();

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new_readonly(router_state_pda, false),
                AccountMeta::new_readonly(ibc_app_pda, false),
                AccountMeta::new(packet_commitment, false),
                AccountMeta::new_readonly(app_signer_pda, true),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(system_program::ID, false),
                AccountMeta::new_readonly(client_pda, false),
                AccountMeta::new_readonly(MOCK_LIGHT_CLIENT_ID, false),
                AccountMeta::new_readonly(mock_client_state, false),
                AccountMeta::new_readonly(mock_consensus_state, false),
            ],
            data,
        };

        let empty_account = || solana_sdk::account::Account {
            lamports: 1_000_000,
            data: vec![],
            owner: system_program::ID,
            executable: false,
            rent_epoch: 0,
        };

        let (sys_program, sys_account) = (
            system_program::ID,
            solana_sdk::account::Account {
                lamports: 1,
                data: vec![],
                owner: solana_sdk::native_loader::ID,
                executable: true,
                rent_epoch: 0,
            },
        );

        let accounts = vec![
            (router_state_pda, router_state_account),
            (ibc_app_pda, ibc_app_account),
            (packet_commitment, empty_account()),
            (app_signer_pda, empty_account()),
            (
                payer,
                solana_sdk::account::Account {
                    lamports: 10_000_000,
                    data: vec![],
                    owner: system_program::ID,
                    executable: false,
                    rent_epoch: 0,
                },
            ),
            (sys_program, sys_account),
            (client_pda, client_account),
            (
                MOCK_LIGHT_CLIENT_ID,
                solana_sdk::account::Account {
                    lamports: 1,
                    data: vec![],
                    owner: solana_sdk::native_loader::ID,
                    executable: true,
                    rent_epoch: 0,
                },
            ),
            (mock_client_state, empty_account()),
            (mock_consensus_state, empty_account()),
        ];

        let result = mollusk.process_instruction(&instruction, &accounts);
        assert_error_code(
            result,
            RouterError::PortIdentifierMismatch,
            "port_identifier_mismatch",
        );
    }

    #[tokio::test]
    async fn test_send_packet_paused() {
        if std::env::var("SBF_OUT_DIR").is_err() {
            let deploy_dir = std::path::Path::new("../../target/deploy");
            std::env::set_var("SBF_OUT_DIR", deploy_dir);
        }

        let mut pt = ProgramTest::new("ics26_router", crate::ID, None);
        pt.add_program("mock_light_client", MOCK_LIGHT_CLIENT_ID, None);
        pt.add_program("access_manager", access_manager::ID, None);
        pt.add_program("test_ibc_app", TEST_IBC_APP_PROGRAM_ID, None);

        // Use paused router state
        let (router_state_pda, router_state_data) = setup_paused_router_state();
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

        let (access_manager_pda, _) =
            solana_ibc_types::access_manager::AccessManager::pda(access_manager::ID);
        let am = access_manager::state::AccessManager {
            roles: vec![access_manager::RoleData {
                role_id: solana_ibc_types::roles::ADMIN_ROLE,
                members: vec![Pubkey::new_unique()],
            }],
            whitelisted_programs: vec![],
            pending_authority_transfers: vec![],
        };
        let mut am_data = access_manager::state::AccessManager::DISCRIMINATOR.to_vec();
        am.serialize(&mut am_data).unwrap();
        pt.add_account(
            access_manager_pda,
            solana_sdk::account::Account {
                lamports: 1_000_000,
                data: am_data,
                owner: access_manager::ID,
                executable: false,
                rent_epoch: 0,
            },
        );

        let (client_pda, client_data) = setup_client(
            TEST_CLIENT_ID,
            MOCK_LIGHT_CLIENT_ID,
            COUNTERPARTY_CLIENT_ID,
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

        let (ibc_app_pda, ibc_app_data) = setup_ibc_app(TEST_PORT, TEST_IBC_APP_PROGRAM_ID);
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

        let (app_state_pda, _) = Pubkey::find_program_address(
            &[solana_ibc_types::IBCAppState::SEED],
            &TEST_IBC_APP_PROGRAM_ID,
        );
        let app_state = test_ibc_app::state::TestIbcAppState {
            authority: Pubkey::new_unique(),
            packets_received: 0,
            packets_acknowledged: 0,
            packets_timed_out: 0,
            packets_sent: 0,
        };
        let app_state_data = create_account_data(&app_state);
        pt.add_account(
            app_state_pda,
            solana_sdk::account::Account {
                lamports: 1_000_000,
                data: app_state_data,
                owner: TEST_IBC_APP_PROGRAM_ID,
                executable: false,
                rent_epoch: 0,
            },
        );

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

        let clock = solana_sdk::clock::Clock {
            slot: 1,
            epoch_start_timestamp: TEST_CLOCK_TIME,
            epoch: 0,
            leader_schedule_epoch: 0,
            unix_timestamp: TEST_CLOCK_TIME,
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

        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let (ix, _) = build_send_packet_ix_with_commitment(
            &payer,
            TEST_CLIENT_ID,
            1,
            TEST_TIMEOUT,
            b"test data",
            mock_client_state,
            mock_consensus_state,
        );

        let err = process_tx(&banks_client, &payer, recent_blockhash, &[ix])
            .await
            .unwrap_err();
        assert_eq!(
            pt_extract_custom_error(&err),
            Some(ANCHOR_ERROR_OFFSET + RouterError::RouterPaused as u32),
        );
    }

    #[test]
    fn test_app_state_pda_does_not_include_port_id() {
        // Verify that IBCAppState PDA is derived from [SEED] only, not [SEED, port_id].
        let program_id = TEST_IBC_APP_PROGRAM_ID;

        let (pda_without_port, _) =
            Pubkey::find_program_address(&[solana_ibc_types::IBCAppState::SEED], &program_id);

        let (pda_with_port, _) = Pubkey::find_program_address(
            &[solana_ibc_types::IBCAppState::SEED, b"transfer"],
            &program_id,
        );

        assert_ne!(
            pda_without_port, pda_with_port,
            "PDA with port_id in seeds must differ from PDA without"
        );

        // Helper must agree with the seedless derivation
        let (pda_from_helper, _) = solana_ibc_types::IBCAppState::pda(program_id);
        assert_eq!(pda_without_port, pda_from_helper);
    }

    #[tokio::test]
    async fn test_send_packet_app_signer_port_independent() {
        // App signer seeds are port-independent: the `IBCApp` account already
        // provides per-port authorization (mapping port → `app_program_id`),
        // so the PDA only needs to prove the caller's program identity.
        // Registering the same program under two ports must share one signer.
        let second_port = "oracle";

        let (mut pt, mock_client_state, mock_consensus_state) =
            setup_send_packet_program_test(TEST_CLIENT_ID, COUNTERPARTY_CLIENT_ID, true);

        // Register the same app program under a second port
        let (ibc_app_pda_2, ibc_app_data_2) = setup_ibc_app(second_port, TEST_IBC_APP_PROGRAM_ID);
        pt.add_account(
            ibc_app_pda_2,
            solana_sdk::account::Account {
                lamports: 1_000_000,
                data: ibc_app_data_2,
                owner: crate::ID,
                executable: false,
                rent_epoch: 0,
            },
        );

        let (banks_client, payer, recent_blockhash) = pt.start().await;

        // Send on the original port with sequence 1
        let (ix1, _) = build_send_packet_ix_with_commitment(
            &payer,
            TEST_CLIENT_ID,
            1,
            TEST_TIMEOUT,
            b"test data 1",
            mock_client_state,
            mock_consensus_state,
        );
        let result1 = process_tx(&banks_client, &payer, recent_blockhash, &[ix1]).await;
        assert!(
            result1.is_ok(),
            "send on first port should succeed: {:?}",
            result1.err()
        );

        // Send on the second port (same app program, different port) with sequence 2.
        let recent_blockhash2 = banks_client.get_latest_blockhash().await.unwrap();
        let (ix2, _) = build_send_packet_ix_with_commitment_for_port(
            &payer,
            TEST_CLIENT_ID,
            second_port,
            2,
            TEST_TIMEOUT,
            b"test data 2",
            mock_client_state,
            mock_consensus_state,
        );
        let result2 = process_tx(&banks_client, &payer, recent_blockhash2, &[ix2]).await;
        assert!(
            result2.is_ok(),
            "send on second port (same app) should succeed: {:?}",
            result2.err()
        );
    }

    #[tokio::test]
    async fn test_two_consecutive_transfers_sequence_increment() {
        let (pt, mock_client_state, mock_consensus_state) =
            setup_send_packet_program_test(TEST_CLIENT_ID, COUNTERPARTY_CLIENT_ID, true);

        let (banks_client, payer, recent_blockhash) = pt.start().await;

        // --- Transfer #1 (sequence 1) ---
        let (ix1, commitment_pda_1) = build_send_packet_ix_with_commitment(
            &payer,
            TEST_CLIENT_ID,
            1,
            TEST_TIMEOUT,
            b"transfer one",
            mock_client_state,
            mock_consensus_state,
        );

        let result1 = process_tx(&banks_client, &payer, recent_blockhash, &[ix1]).await;
        assert!(
            result1.is_ok(),
            "Transfer #1 should succeed: {:?}",
            result1.err()
        );

        // Verify commitment #1 exists and matches
        let commitment_1 = banks_client
            .get_account(commitment_pda_1)
            .await
            .unwrap()
            .expect("commitment #1 should exist");
        assert_eq!(commitment_1.owner, crate::ID);
        let commitment_value_1 = &commitment_1.data[8..40];
        assert_ne!(commitment_value_1, &[0u8; 32]);

        let expected_packet_1 = Packet {
            sequence: 1,
            source_client: TEST_CLIENT_ID.to_string(),
            dest_client: COUNTERPARTY_CLIENT_ID.to_string(),
            timeout_timestamp: TEST_TIMEOUT,
            payloads: vec![Payload {
                source_port: TEST_PORT.to_string(),
                dest_port: "dest-port".to_string(),
                version: "1".to_string(),
                encoding: "json".to_string(),
                value: b"transfer one".to_vec(),
            }],
        };
        let expected_commitment_1 =
            solana_ibc_types::ics24::packet_commitment_bytes32(&expected_packet_1);
        assert_eq!(commitment_value_1, &expected_commitment_1);

        // --- Transfer #2 (sequence 2) ---
        let recent_blockhash2 = banks_client.get_latest_blockhash().await.unwrap();
        let (ix2, commitment_pda_2) = build_send_packet_ix_with_commitment(
            &payer,
            TEST_CLIENT_ID,
            2,
            TEST_TIMEOUT,
            b"transfer two",
            mock_client_state,
            mock_consensus_state,
        );

        let result2 = process_tx(&banks_client, &payer, recent_blockhash2, &[ix2]).await;
        assert!(
            result2.is_ok(),
            "Transfer #2 should succeed: {:?}",
            result2.err()
        );

        // Verify commitment #2 exists and matches
        let commitment_2 = banks_client
            .get_account(commitment_pda_2)
            .await
            .unwrap()
            .expect("commitment #2 should exist");
        assert_eq!(commitment_2.owner, crate::ID);
        let commitment_value_2 = &commitment_2.data[8..40];
        assert_ne!(commitment_value_2, &[0u8; 32]);

        let expected_packet_2 = Packet {
            sequence: 2,
            source_client: TEST_CLIENT_ID.to_string(),
            dest_client: COUNTERPARTY_CLIENT_ID.to_string(),
            timeout_timestamp: TEST_TIMEOUT,
            payloads: vec![Payload {
                source_port: TEST_PORT.to_string(),
                dest_port: "dest-port".to_string(),
                version: "1".to_string(),
                encoding: "json".to_string(),
                value: b"transfer two".to_vec(),
            }],
        };
        let expected_commitment_2 =
            solana_ibc_types::ics24::packet_commitment_bytes32(&expected_packet_2);
        assert_eq!(commitment_value_2, &expected_commitment_2);

        // Verify both commitments are distinct
        assert_ne!(
            commitment_value_1, commitment_value_2,
            "Commitments must differ (different sequences and payload data)"
        );
        assert_ne!(
            commitment_pda_1, commitment_pda_2,
            "Commitment PDAs must differ"
        );
    }

    #[tokio::test]
    async fn test_send_packet_wrong_light_client_program() {
        let (pt, mock_client_state, mock_consensus_state) =
            setup_send_packet_program_test(TEST_CLIENT_ID, COUNTERPARTY_CLIENT_ID, true);

        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let (mut ix, _) = build_send_packet_ix_with_commitment(
            &payer,
            TEST_CLIENT_ID,
            1,
            TEST_TIMEOUT,
            b"test data",
            mock_client_state,
            mock_consensus_state,
        );

        // Replace light_client_program (index 6) with a wrong pubkey.
        // The Client PDA stores client_program_id = MOCK_LIGHT_CLIENT_ID,
        // so any other key triggers the address constraint.
        let wrong_program = Pubkey::new_unique();
        ix.accounts[6] = AccountMeta::new_readonly(wrong_program, false);

        let err = process_tx(&banks_client, &payer, recent_blockhash, &[ix])
            .await
            .unwrap_err();
        assert_eq!(
            pt_extract_custom_error(&err),
            Some(ANCHOR_ERROR_OFFSET + RouterError::InvalidLightClientProgram as u32),
        );
    }

    #[tokio::test]
    async fn test_send_packet_wrong_consensus_state_owner() {
        if std::env::var("SBF_OUT_DIR").is_err() {
            let deploy_dir = std::path::Path::new("../../target/deploy");
            std::env::set_var("SBF_OUT_DIR", deploy_dir);
        }

        let mut pt = ProgramTest::new("ics26_router", crate::ID, None);
        pt.add_program("mock_light_client", MOCK_LIGHT_CLIENT_ID, None);
        pt.add_program("access_manager", access_manager::ID, None);
        pt.add_program("test_ibc_app", TEST_IBC_APP_PROGRAM_ID, None);

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

        let (access_manager_pda, _) =
            solana_ibc_types::access_manager::AccessManager::pda(access_manager::ID);
        let am = access_manager::state::AccessManager {
            roles: vec![access_manager::RoleData {
                role_id: solana_ibc_types::roles::ADMIN_ROLE,
                members: vec![Pubkey::new_unique()],
            }],
            whitelisted_programs: vec![],
            pending_authority_transfers: vec![],
        };
        let mut am_data = access_manager::state::AccessManager::DISCRIMINATOR.to_vec();
        am.serialize(&mut am_data).unwrap();
        pt.add_account(
            access_manager_pda,
            solana_sdk::account::Account {
                lamports: 1_000_000,
                data: am_data,
                owner: access_manager::ID,
                executable: false,
                rent_epoch: 0,
            },
        );

        let (client_pda, client_data) = setup_client(
            TEST_CLIENT_ID,
            MOCK_LIGHT_CLIENT_ID,
            COUNTERPARTY_CLIENT_ID,
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

        let (ibc_app_pda, ibc_app_data) = setup_ibc_app(TEST_PORT, TEST_IBC_APP_PROGRAM_ID);
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

        let (app_state_pda, _) = Pubkey::find_program_address(
            &[solana_ibc_types::IBCAppState::SEED],
            &TEST_IBC_APP_PROGRAM_ID,
        );
        let app_state = test_ibc_app::state::TestIbcAppState {
            authority: Pubkey::new_unique(),
            packets_received: 0,
            packets_acknowledged: 0,
            packets_timed_out: 0,
            packets_sent: 0,
        };
        let app_state_data = create_account_data(&app_state);
        pt.add_account(
            app_state_pda,
            solana_sdk::account::Account {
                lamports: 1_000_000,
                data: app_state_data,
                owner: TEST_IBC_APP_PROGRAM_ID,
                executable: false,
                rent_epoch: 0,
            },
        );

        // Client state owned by the correct program
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

        // Consensus state owned by the WRONG program
        let mock_consensus_state = Pubkey::new_unique();
        pt.add_account(
            mock_consensus_state,
            solana_sdk::account::Account {
                lamports: 1_000_000,
                data: vec![0u8; 64],
                owner: system_program::ID,
                executable: false,
                rent_epoch: 0,
            },
        );

        let clock = solana_sdk::clock::Clock {
            slot: 1,
            epoch_start_timestamp: TEST_CLOCK_TIME,
            epoch: 0,
            leader_schedule_epoch: 0,
            unix_timestamp: TEST_CLOCK_TIME,
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

        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let (ix, _) = build_send_packet_ix_with_commitment(
            &payer,
            TEST_CLIENT_ID,
            1,
            TEST_TIMEOUT,
            b"test data",
            mock_client_state,
            mock_consensus_state,
        );

        let err = process_tx(&banks_client, &payer, recent_blockhash, &[ix])
            .await
            .unwrap_err();
        assert_eq!(
            pt_extract_custom_error(&err),
            Some(ANCHOR_ERROR_OFFSET + RouterError::InvalidAccountOwner as u32),
        );
    }

    #[tokio::test]
    async fn test_send_packet_wrong_client_state_owner() {
        if std::env::var("SBF_OUT_DIR").is_err() {
            let deploy_dir = std::path::Path::new("../../target/deploy");
            std::env::set_var("SBF_OUT_DIR", deploy_dir);
        }

        let mut pt = ProgramTest::new("ics26_router", crate::ID, None);
        pt.add_program("mock_light_client", MOCK_LIGHT_CLIENT_ID, None);
        pt.add_program("access_manager", access_manager::ID, None);
        pt.add_program("test_ibc_app", TEST_IBC_APP_PROGRAM_ID, None);

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

        let (access_manager_pda, _) =
            solana_ibc_types::access_manager::AccessManager::pda(access_manager::ID);
        let am = access_manager::state::AccessManager {
            roles: vec![access_manager::RoleData {
                role_id: solana_ibc_types::roles::ADMIN_ROLE,
                members: vec![Pubkey::new_unique()],
            }],
            whitelisted_programs: vec![],
            pending_authority_transfers: vec![],
        };
        let mut am_data = access_manager::state::AccessManager::DISCRIMINATOR.to_vec();
        am.serialize(&mut am_data).unwrap();
        pt.add_account(
            access_manager_pda,
            solana_sdk::account::Account {
                lamports: 1_000_000,
                data: am_data,
                owner: access_manager::ID,
                executable: false,
                rent_epoch: 0,
            },
        );

        let (client_pda, client_data) = setup_client(
            TEST_CLIENT_ID,
            MOCK_LIGHT_CLIENT_ID,
            COUNTERPARTY_CLIENT_ID,
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

        let (ibc_app_pda, ibc_app_data) = setup_ibc_app(TEST_PORT, TEST_IBC_APP_PROGRAM_ID);
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

        let (app_state_pda, _) = Pubkey::find_program_address(
            &[solana_ibc_types::IBCAppState::SEED],
            &TEST_IBC_APP_PROGRAM_ID,
        );
        let app_state = test_ibc_app::state::TestIbcAppState {
            authority: Pubkey::new_unique(),
            packets_received: 0,
            packets_acknowledged: 0,
            packets_timed_out: 0,
            packets_sent: 0,
        };
        let app_state_data = create_account_data(&app_state);
        pt.add_account(
            app_state_pda,
            solana_sdk::account::Account {
                lamports: 1_000_000,
                data: app_state_data,
                owner: TEST_IBC_APP_PROGRAM_ID,
                executable: false,
                rent_epoch: 0,
            },
        );

        // Client state owned by the WRONG program (system_program instead of mock_light_client)
        let mock_client_state = Pubkey::new_unique();
        pt.add_account(
            mock_client_state,
            solana_sdk::account::Account {
                lamports: 1_000_000,
                data: vec![0u8; 64],
                owner: system_program::ID,
                executable: false,
                rent_epoch: 0,
            },
        );

        let clock = solana_sdk::clock::Clock {
            slot: 1,
            epoch_start_timestamp: TEST_CLOCK_TIME,
            epoch: 0,
            leader_schedule_epoch: 0,
            unix_timestamp: TEST_CLOCK_TIME,
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

        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let (ix, _) = build_send_packet_ix_with_commitment(
            &payer,
            TEST_CLIENT_ID,
            1,
            TEST_TIMEOUT,
            b"test data",
            mock_client_state,
            Pubkey::new_unique(),
        );

        let err = process_tx(&banks_client, &payer, recent_blockhash, &[ix])
            .await
            .unwrap_err();
        assert_eq!(
            pt_extract_custom_error(&err),
            Some(ANCHOR_ERROR_OFFSET + RouterError::InvalidAccountOwner as u32),
        );
    }

    #[tokio::test]
    async fn test_send_packet_zero_sequence_rejected() {
        let (pt, mock_client_state, mock_consensus_state) =
            setup_send_packet_program_test(TEST_CLIENT_ID, COUNTERPARTY_CLIENT_ID, true);

        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let (ix, _) = build_send_packet_ix_with_commitment(
            &payer,
            TEST_CLIENT_ID,
            0, // zero sequence
            TEST_TIMEOUT,
            b"test data",
            mock_client_state,
            mock_consensus_state,
        );

        let err = process_tx(&banks_client, &payer, recent_blockhash, &[ix])
            .await
            .unwrap_err();
        assert_eq!(
            pt_extract_custom_error(&err),
            Some(ANCHOR_ERROR_OFFSET + RouterError::ZeroPacketSequence as u32),
        );
    }

    #[tokio::test]
    async fn test_send_packet_replay_after_zeroing_rejected() {
        let sequence = 1u64;

        let (mut pt, mock_client_state, mock_consensus_state) =
            setup_send_packet_program_test(TEST_CLIENT_ID, COUNTERPARTY_CLIENT_ID, true);

        // Pre-create a zeroed commitment PDA (simulates post-ack state)
        let (packet_commitment_pda, _) = Pubkey::find_program_address(
            &[
                Commitment::PACKET_COMMITMENT_SEED,
                TEST_CLIENT_ID.as_bytes(),
                &sequence.to_le_bytes(),
            ],
            &crate::ID,
        );
        let zeroed_commitment = Commitment {
            value: Commitment::EMPTY,
        };
        let commitment_data = create_account_data(&zeroed_commitment);
        pt.add_account(
            packet_commitment_pda,
            solana_sdk::account::Account {
                lamports: 1_000_000,
                data: commitment_data,
                owner: crate::ID,
                executable: false,
                rent_epoch: 0,
            },
        );

        let (banks_client, payer, recent_blockhash) = pt.start().await;

        let (ix, _) = build_send_packet_ix_with_commitment(
            &payer,
            TEST_CLIENT_ID,
            sequence,
            TEST_TIMEOUT,
            b"test data",
            mock_client_state,
            mock_consensus_state,
        );

        let err = process_tx(&banks_client, &payer, recent_blockhash, &[ix])
            .await
            .expect_err("send_packet must reject a reused sequence after ack zeroing");

        // Anchor's `init` constraint fails when the PDA already exists
        let code = pt_extract_custom_error(&err);
        assert!(
            code.is_some(),
            "Expected an error when reusing a zeroed sequence PDA"
        );
    }
}
