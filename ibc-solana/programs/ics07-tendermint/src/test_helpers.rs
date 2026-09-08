// SPDX-License-Identifier: Apache-2.0

use std::sync::LazyLock;

use access_manager::AccessManagerState;
use mollusk_svm::result::Check;

pub const PROGRAM_BINARY_PATH: &str = "../../target/deploy/ics07_tendermint";

// Solana compute budget constants for tests
// Match production runtime configuration
pub const TEST_HEAP_SIZE: u32 = 256 * 1024; // 256KB heap for large header deserialization
pub const TEST_COMPUTE_UNIT_LIMIT: u64 = 1_400_000; // Solana's actual CU limit

pub static SUCCESS_CHECK: LazyLock<Vec<Check>> = LazyLock::new(|| vec![Check::success()]);

pub mod fixtures {
    use crate::types::{ClientState, ConsensusState, IbcHeight};
    use serde::Deserialize;

    #[derive(Deserialize)]
    pub struct UpdateClientFixture {
        pub client_state_hex: String,
        pub consensus_state_hex: String,
        pub update_client_message: UpdateClientMessage,
    }

    #[derive(Deserialize)]
    pub struct UpdateClientMessage {
        pub client_message_hex: String,
        pub trusted_height: u64,
        pub new_height: u64,
    }

    fn load_fixture(filename: &str) -> UpdateClientFixture {
        let fixture_path =
            format!("../../../packages/tendermint-light-client/fixtures/{filename}.json");
        let fixture_content = std::fs::read_to_string(&fixture_path)
            .unwrap_or_else(|_| panic!("Failed to read fixture: {fixture_path}"));

        serde_json::from_str(&fixture_content)
            .unwrap_or_else(|_| panic!("Failed to parse fixture: {fixture_path}"))
    }

    pub fn load_update_client_message(filename: &str) -> UpdateClientMessage {
        let fixture = load_fixture(filename);
        fixture.update_client_message
    }

    pub fn load_primary_fixtures() -> (ClientState, ConsensusState, UpdateClientMessage) {
        let fixture = load_fixture("update_client_happy_path");
        (
            decode_client_state(&fixture.client_state_hex),
            decode_consensus_state(&fixture.consensus_state_hex),
            fixture.update_client_message,
        )
    }

    fn decode_client_state(client_state_hex: &str) -> ClientState {
        use prost::Message;

        let bytes = hex_to_bytes(client_state_hex);
        let proto = ibc_client_tendermint::types::proto::v1::ClientState::decode(&bytes[..])
            .expect("Failed to decode client state");

        let trust_level = proto
            .trust_level
            .expect("Missing trust_level in client state");
        let trusting_period = proto
            .trusting_period
            .expect("Missing trusting_period in client state");
        let unbonding_period = proto
            .unbonding_period
            .expect("Missing unbonding_period in client state");
        let max_clock_drift = proto
            .max_clock_drift
            .expect("Missing max_clock_drift in client state");
        let latest_height = proto
            .latest_height
            .expect("Missing latest_height in client state");

        ClientState {
            chain_id: proto.chain_id,
            trust_level_numerator: trust_level.numerator,
            trust_level_denominator: trust_level.denominator,
            trusting_period: trusting_period.seconds as u64,
            unbonding_period: unbonding_period.seconds as u64,
            max_clock_drift: max_clock_drift.seconds as u64,
            frozen_height: proto
                .frozen_height
                .map_or_else(IbcHeight::default, |frozen_height| IbcHeight {
                    revision_number: frozen_height.revision_number,
                    revision_height: frozen_height.revision_height,
                }),
            latest_height: IbcHeight {
                revision_number: latest_height.revision_number,
                revision_height: latest_height.revision_height,
            },
        }
    }

    fn decode_consensus_state(consensus_state_hex: &str) -> ConsensusState {
        use prost::Message;

        let bytes = hex_to_bytes(consensus_state_hex);
        let proto = ibc_client_tendermint::types::proto::v1::ConsensusState::decode(&bytes[..])
            .expect("Failed to decode consensus state");

        let timestamp = proto
            .timestamp
            .expect("Missing timestamp in consensus state");
        let root = proto.root.expect("Missing root in consensus state");

        ConsensusState {
            timestamp: timestamp_to_nanoseconds(timestamp.seconds, timestamp.nanos),
            root: root
                .hash
                .as_slice()
                .try_into()
                .expect("Invalid root hash length"),
            next_validators_hash: proto
                .next_validators_hash
                .as_slice()
                .try_into()
                .expect("Invalid next_validators_hash length"),
        }
    }

    // Helper to check if a fixture file exists
    pub fn fixture_exists(filename: &str) -> bool {
        let fixture_path =
            format!("../../../packages/tendermint-light-client/fixtures/{filename}.json");
        std::path::Path::new(&fixture_path).exists()
    }

    pub fn hex_to_bytes(hex_str: &str) -> Vec<u8> {
        let hex_str = hex_str.trim_start_matches("0x");
        hex::decode(hex_str).expect("Invalid hex string")
    }

    /// Convert protobuf Timestamp to nanoseconds since Unix epoch
    pub fn timestamp_to_nanoseconds(seconds: i64, nanos: i32) -> u64 {
        crate::secs_to_nanos(seconds) as u64 + nanos as u64
    }

    /// Convert Protobuf header bytes to Borsh format (like the relayer does)
    /// Fixtures contain Protobuf-encoded headers, but our program expects Borsh
    pub fn protobuf_to_borsh_header(protobuf_bytes: &[u8]) -> Vec<u8> {
        use borsh::BorshSerialize;
        use ibc_client_tendermint::types::Header;
        use ibc_proto::ibc::lightclients::tendermint::v1::Header as RawHeader;
        use ibc_proto::Protobuf;
        use solana_ibc_borsh_header::conversions::header_to_borsh;

        // Decode from Protobuf (fixture format)
        let header = <Header as Protobuf<RawHeader>>::decode_vec(protobuf_bytes)
            .expect("Failed to decode protobuf header from fixture");

        // Convert to BorshHeader and serialize (exactly like the relayer does)
        let borsh_header = header_to_borsh(header);
        borsh_header
            .try_to_vec()
            .expect("Failed to encode header to Borsh")
    }

    /// Extract header timestamp from update client message
    /// Returns the header time as Unix timestamp in seconds (suitable for Clock sysvar)
    pub fn get_header_timestamp_from_message(message: &UpdateClientMessage) -> i64 {
        use ibc_client_tendermint::types::Header;
        use ibc_proto::ibc::lightclients::tendermint::v1::Header as RawHeader;
        use ibc_proto::Protobuf;

        // Decode from Protobuf fixture to get timestamp
        let client_message_proto = hex_to_bytes(&message.client_message_hex);
        let header = <Header as Protobuf<RawHeader>>::decode_vec(&client_message_proto)
            .expect("Failed to decode header from fixture");

        // Extract timestamp from header and convert to Unix seconds
        let header_time_nanos = header.signed_header.header.time.unix_timestamp_nanos() as u64;
        crate::nanos_to_secs(header_time_nanos) as i64
    }

    /// Create a clock timestamp valid for the given header
    pub fn get_valid_clock_timestamp_for_header(message: &UpdateClientMessage) -> i64 {
        let header_timestamp = get_header_timestamp_from_message(message);
        header_timestamp.saturating_add(5)
    }

    /// Create an expired clock timestamp for testing
    pub fn get_expired_clock_timestamp_for_header(message: &UpdateClientMessage) -> i64 {
        let header_timestamp = get_header_timestamp_from_message(message);
        let one_year_in_seconds: i64 = 86400 * 365;
        header_timestamp.saturating_add(one_year_in_seconds)
    }

    /// Corrupt the header signature in the client message bytes
    pub fn corrupt_header_signature(client_message_hex: &str) -> Vec<u8> {
        use borsh::BorshSerialize;
        use ibc_client_tendermint::types::Header;
        use ibc_proto::ibc::lightclients::tendermint::v1::Header as RawHeader;
        use ibc_proto::Protobuf;
        use prost::Message;
        use solana_ibc_borsh_header::conversions::header_to_borsh;

        let bytes = hex_to_bytes(client_message_hex);
        let mut header_proto = ibc_client_tendermint::types::proto::v1::Header::decode(&bytes[..])
            .expect("Failed to decode header");

        if let Some(signed_header) = &mut header_proto.signed_header {
            if let Some(commit) = &mut signed_header.commit {
                for sig in &mut commit.signatures {
                    if !sig.signature.is_empty() {
                        let mid_pos = sig.signature.len() / 2;
                        sig.signature[mid_pos] ^= 0x01;
                        break;
                    }
                }
            }
        }

        let mut buf = Vec::new();
        header_proto
            .encode(&mut buf)
            .expect("Failed to encode header");

        let header = <Header as Protobuf<RawHeader>>::decode_vec(&buf)
            .expect("Failed to decode corrupted protobuf header");
        let borsh_header = header_to_borsh(header);
        borsh_header
            .try_to_vec()
            .expect("Failed to encode corrupted header to Borsh")
    }

    /// Create client message bytes with wrong trusted height
    pub fn create_message_with_wrong_trusted_height(
        client_message_hex: &str,
        wrong_height: u64,
    ) -> Vec<u8> {
        use borsh::BorshSerialize;
        use ibc_client_tendermint::types::Header;
        use ibc_proto::ibc::lightclients::tendermint::v1::Header as RawHeader;
        use ibc_proto::Protobuf;
        use prost::Message;
        use solana_ibc_borsh_header::conversions::header_to_borsh;

        let bytes = hex_to_bytes(client_message_hex);

        let mut header_proto = ibc_client_tendermint::types::proto::v1::Header::decode(&bytes[..])
            .expect("Failed to decode header from test fixture");

        header_proto.trusted_height = Some(ibc_proto::ibc::core::client::v1::Height {
            revision_number: 0,
            revision_height: wrong_height,
        });

        let mut buf = Vec::new();
        header_proto
            .encode(&mut buf)
            .expect("Failed to encode header");

        let header = <Header as Protobuf<RawHeader>>::decode_vec(&buf)
            .expect("Failed to decode modified protobuf header");
        let borsh_header = header_to_borsh(header);
        borsh_header
            .try_to_vec()
            .expect("Failed to encode modified header to Borsh")
    }

    // Generic test helper functions
    pub fn get_error_code(error: &anchor_lang::prelude::ProgramError) -> Option<u32> {
        match error {
            anchor_lang::prelude::ProgramError::Custom(code) => Some(*code),
            _ => None,
        }
    }

    pub fn assert_error_code(
        result: mollusk_svm::result::InstructionResult,
        expected_error: crate::error::ErrorCode,
        test_name: &str,
    ) {
        match result.program_result {
            mollusk_svm::result::ProgramResult::Success => {
                panic!("Expected {test_name} to fail with {expected_error:?}, but it succeeded");
            }
            mollusk_svm::result::ProgramResult::Failure(error) => {
                if let Some(code) = get_error_code(&error) {
                    let expected_code = expected_error as u32 + 6000; // Anchor errors start at 6000
                    assert_eq!(
                        code, expected_code,
                        "Expected {expected_error:?} ({expected_code}), but got error code {code}"
                    );
                    println!(
                        "✅ {test_name} correctly failed with {expected_error:?} ({expected_code})"
                    );
                } else {
                    panic!("Expected custom error code for {test_name}, got: {error:?}");
                }
            }
            mollusk_svm::result::ProgramResult::UnknownError(_) => panic!(
                "Unexpected program result for {}: {:?}",
                test_name, result.program_result
            ),
        }
    }

    pub fn assert_instruction_failed(
        result: mollusk_svm::result::InstructionResult,
        test_name: &str,
    ) {
        match result.program_result {
            mollusk_svm::result::ProgramResult::Success => {
                panic!("Expected instruction to fail for {test_name}, but it succeeded");
            }
            _ => {
                println!(
                    "✅ {} correctly rejected: {:?}",
                    test_name, result.program_result
                );
            }
        }
    }

    /// Membership verification fixture structures
    #[derive(Debug, serde::Deserialize)]
    pub struct MembershipMsgFixture {
        pub path: Vec<String>,
        pub proof: String,
        pub value: String,
        pub height: u64,
        pub delay_time_period: u64,
        pub delay_block_period: u64,
    }

    #[derive(Debug, serde::Deserialize)]
    pub struct MembershipVerificationFixture {
        pub membership_msg: MembershipMsgFixture,
        pub consensus_state_hex: String,
        pub client_state_hex: String,
    }

    pub fn load_membership_verification_fixture(filename: &str) -> MembershipVerificationFixture {
        let fixture_path =
            format!("../../../packages/tendermint-light-client/fixtures/{filename}.json");
        let fixture_content = std::fs::read_to_string(&fixture_path)
            .unwrap_or_else(|_| panic!("Failed to read fixture: {fixture_path}"));

        serde_json::from_str(&fixture_content)
            .unwrap_or_else(|_| panic!("Failed to parse fixture: {fixture_path}"))
    }

    pub fn decode_client_state_from_hex(client_state_hex: &str) -> ClientState {
        use prost::Message;

        let bytes = hex_to_bytes(client_state_hex);
        let proto = ibc_client_tendermint::types::proto::v1::ClientState::decode(&bytes[..])
            .expect("Failed to decode client state");

        let trust_level = proto
            .trust_level
            .expect("Missing trust_level in client state");
        let trusting_period = proto
            .trusting_period
            .expect("Missing trusting_period in client state");
        let unbonding_period = proto
            .unbonding_period
            .expect("Missing unbonding_period in client state");
        let max_clock_drift = proto
            .max_clock_drift
            .expect("Missing max_clock_drift in client state");
        let latest_height = proto
            .latest_height
            .expect("Missing latest_height in client state");

        ClientState {
            chain_id: proto.chain_id,
            trust_level_numerator: trust_level.numerator,
            trust_level_denominator: trust_level.denominator,
            trusting_period: trusting_period.seconds as u64,
            unbonding_period: unbonding_period.seconds as u64,
            max_clock_drift: max_clock_drift.seconds as u64,
            frozen_height: proto
                .frozen_height
                .map_or_else(IbcHeight::default, |frozen_height| IbcHeight {
                    revision_number: frozen_height.revision_number,
                    revision_height: frozen_height.revision_height,
                }),
            latest_height: IbcHeight {
                revision_number: latest_height.revision_number,
                revision_height: latest_height.revision_height,
            },
        }
    }

    pub fn decode_consensus_state_from_hex(consensus_state_hex: &str) -> ConsensusState {
        use prost::Message;

        let bytes = hex_to_bytes(consensus_state_hex);
        let proto = ibc_client_tendermint::types::proto::v1::ConsensusState::decode(&bytes[..])
            .expect("Failed to decode consensus state");

        let timestamp = proto
            .timestamp
            .expect("Missing timestamp in consensus state");
        let root = proto.root.expect("Missing root in consensus state");

        let timestamp_nanos = timestamp_to_nanoseconds(timestamp.seconds, timestamp.nanos);

        ConsensusState {
            timestamp: timestamp_nanos,
            root: root
                .hash
                .as_slice()
                .try_into()
                .expect("Invalid root hash length"),
            next_validators_hash: proto
                .next_validators_hash
                .as_slice()
                .try_into()
                .expect("Invalid next_validators_hash length"),
        }
    }

    /// Helper functions for misbehaviour testing
    pub mod misbehaviour {
        use super::*;
        use solana_ibc_borsh_header::{
            BorshBlockHeader, BorshBlockId, BorshCommit, BorshCommitSig, BorshConsensusVersion,
            BorshHeader, BorshHeight, BorshMisbehaviour, BorshPartSetHeader, BorshPublicKey,
            BorshSignedHeader, BorshTimestamp, BorshValidator, BorshValidatorSet,
        };

        /// Creates a Borsh-serialized mock misbehaviour for testing
        pub fn create_mock_tendermint_misbehaviour(
            chain_id: &str,
            header1_height: u64,
            header2_height: u64,
            trusted_height_1: u64,
            trusted_height_2: u64,
            conflicting_app_hashes: bool,
        ) -> Vec<u8> {
            let misbehaviour = create_borsh_misbehaviour(
                chain_id,
                header1_height,
                header2_height,
                trusted_height_1,
                trusted_height_2,
                conflicting_app_hashes,
            );
            borsh::to_vec(&misbehaviour).expect("Failed to serialize misbehaviour")
        }

        fn create_borsh_misbehaviour(
            chain_id: &str,
            header1_height: u64,
            header2_height: u64,
            trusted_height_1: u64,
            trusted_height_2: u64,
            conflicting_app_hashes: bool,
        ) -> BorshMisbehaviour {
            // Create two headers with same height but different app hashes (double sign)
            let app_hash1 = vec![1u8; 32];
            let app_hash2 = if conflicting_app_hashes {
                vec![2u8; 32]
            } else {
                vec![1u8; 32]
            };

            BorshMisbehaviour {
                client_id: "07-tendermint-0".to_string(),
                header1: create_mock_borsh_header(
                    chain_id,
                    header1_height,
                    trusted_height_1,
                    app_hash1,
                ),
                header2: create_mock_borsh_header(
                    chain_id,
                    header2_height,
                    trusted_height_2,
                    app_hash2,
                ),
            }
        }

        fn create_mock_borsh_header(
            chain_id: &str,
            height: u64,
            trusted_height: u64,
            app_hash: Vec<u8>,
        ) -> BorshHeader {
            let validator = create_mock_validator();
            let validator_set = BorshValidatorSet {
                validators: vec![validator.clone()],
                proposer: Some(validator),
                total_voting_power: 100,
            };

            BorshHeader {
                signed_header: create_mock_signed_header(chain_id, height, app_hash),
                validator_set: validator_set.clone(),
                trusted_height: BorshHeight {
                    revision_number: 0,
                    revision_height: trusted_height,
                },
                trusted_next_validator_set: validator_set,
            }
        }

        fn create_mock_signed_header(
            chain_id: &str,
            height: u64,
            app_hash: Vec<u8>,
        ) -> BorshSignedHeader {
            BorshSignedHeader {
                header: create_mock_block_header(chain_id, height, app_hash),
                commit: create_mock_commit(height),
            }
        }

        fn create_mock_block_header(
            chain_id: &str,
            height: u64,
            app_hash: Vec<u8>,
        ) -> BorshBlockHeader {
            BorshBlockHeader {
                version: BorshConsensusVersion { block: 11, app: 0 },
                chain_id: chain_id.to_string(),
                height,
                time: BorshTimestamp {
                    secs: 1_700_000_000,
                    nanos: 0,
                },
                last_block_id: Some(create_mock_block_id()),
                last_commit_hash: Some(vec![3u8; 32]),
                data_hash: Some(vec![4u8; 32]),
                validators_hash: vec![5u8; 32],
                next_validators_hash: vec![6u8; 32],
                consensus_hash: vec![7u8; 32],
                app_hash,
                last_results_hash: Some(vec![8u8; 32]),
                evidence_hash: Some(vec![9u8; 32]),
                proposer_address: vec![10u8; 20],
            }
        }

        fn create_mock_block_id() -> BorshBlockId {
            BorshBlockId {
                hash: vec![11u8; 32],
                part_set_header: BorshPartSetHeader {
                    total: 1,
                    hash: vec![12u8; 32],
                },
            }
        }

        fn create_mock_commit(height: u64) -> BorshCommit {
            BorshCommit {
                height,
                round: 0,
                block_id: create_mock_block_id(),
                signatures: vec![create_mock_commit_sig()],
            }
        }

        fn create_mock_commit_sig() -> BorshCommitSig {
            BorshCommitSig::BlockIdFlagCommit {
                validator_address: [10u8; 20],
                timestamp: BorshTimestamp {
                    secs: 1_700_000_000,
                    nanos: 0,
                },
                signature: [0u8; 64],
            }
        }

        fn create_mock_validator() -> BorshValidator {
            BorshValidator {
                address: [10u8; 20],
                pub_key: BorshPublicKey::Ed25519([0u8; 32]),
                voting_power: 100,
                proposer_priority: 0,
            }
        }

        pub fn misbehaviour_fixture_exists(filename: &str) -> bool {
            fixture_exists(filename)
        }

        pub fn load_misbehaviour_fixture(_filename: &str) -> Vec<u8> {
            vec![0xDE, 0xAD, 0xBE, 0xEF]
        }
    }
}

#[cfg(test)]
pub mod chunk_test_utils {
    use crate::state::{HeaderChunk, CHUNK_DATA_SIZE};
    use crate::types::{ClientState, ConsensusState, IbcHeight, UploadChunkParams};
    use solana_sdk::account::Account;
    use solana_sdk::keccak;
    use solana_sdk::pubkey::Pubkey;
    use solana_sdk::system_program;

    pub struct ChunkTestData {
        pub chunk_data: Vec<u8>,
        pub chunk_hash: [u8; 32],
    }

    pub fn create_test_chunk_data(index: u8, size: usize) -> ChunkTestData {
        let chunk_data = vec![index + 1; size];
        let chunk_hash = keccak::hash(&chunk_data).0;
        ChunkTestData {
            chunk_data,
            chunk_hash,
        }
    }

    pub fn create_chunk_account(chunk_data: Vec<u8>) -> Account {
        use anchor_lang::AccountSerialize;

        let chunk = HeaderChunk {
            submitter: Pubkey::default(),
            chunk_data,
        };

        let mut data = vec![];
        chunk.try_serialize(&mut data).unwrap();

        Account {
            lamports: 1_500_000, // Rent
            data,
            owner: crate::ID,
            executable: false,
            rent_epoch: 0,
        }
    }

    pub fn create_client_state_account(chain_id: &str, latest_height: u64) -> Account {
        use anchor_lang::AccountSerialize;

        let client_state = ClientState {
            chain_id: chain_id.to_string(),
            trust_level_numerator: 2,
            trust_level_denominator: 3,
            trusting_period: 86400,
            unbonding_period: 172_800,
            max_clock_drift: 600,
            frozen_height: IbcHeight {
                revision_number: 0,
                revision_height: 0,
            },
            latest_height: IbcHeight {
                revision_number: 0,
                revision_height: latest_height,
            },
        };

        let mut data = vec![];
        client_state.try_serialize(&mut data).unwrap();

        Account {
            lamports: 1_000_000,
            data,
            owner: crate::ID,
            executable: false,
            rent_epoch: 0,
        }
    }

    pub fn create_consensus_state_account(
        root: [u8; 32],
        next_validators_hash: [u8; 32],
        timestamp: u64,
    ) -> Account {
        use crate::state::ConsensusStateStore;
        use anchor_lang::AccountSerialize;

        let consensus_state_store = ConsensusStateStore {
            height: 0, // Will be set by the actual instruction
            consensus_state: ConsensusState {
                timestamp,
                root,
                next_validators_hash,
            },
        };

        let mut data = vec![];
        consensus_state_store.try_serialize(&mut data).unwrap();

        Account {
            lamports: 1_000_000,
            data,
            owner: crate::ID,
            executable: false,
            rent_epoch: 0,
        }
    }

    pub fn create_submitter_account(lamports: u64) -> Account {
        Account {
            lamports,
            data: vec![],
            owner: system_program::ID,
            executable: false,
            rent_epoch: 0,
        }
    }

    pub fn create_upload_chunk_params(
        target_height: u64,
        chunk_index: u8,
        chunk_data: Vec<u8>,
    ) -> UploadChunkParams {
        UploadChunkParams {
            target_height,
            chunk_index,
            chunk_data,
        }
    }

    pub fn derive_chunk_pda(submitter: &Pubkey, target_height: u64, chunk_index: u8) -> Pubkey {
        Pubkey::find_program_address(
            &[
                crate::state::HeaderChunk::SEED,
                submitter.as_ref(),
                &target_height.to_le_bytes(),
                &[chunk_index],
            ],
            &crate::ID,
        )
        .0
    }

    pub fn derive_client_state_pda() -> Pubkey {
        Pubkey::find_program_address(&[crate::types::ClientState::SEED], &crate::ID).0
    }

    pub fn derive_consensus_state_pda(height: u64) -> Pubkey {
        Pubkey::find_program_address(
            &[
                crate::state::ConsensusStateStore::SEED,
                &height.to_le_bytes(),
            ],
            &crate::ID,
        )
        .0
    }

    pub fn create_valid_header_chunks(num_chunks: u8) -> (Vec<Vec<u8>>, [u8; 32]) {
        // Create realistic header data that can be reassembled
        let mut all_chunks = vec![];
        let mut full_header = vec![];

        for i in 0..num_chunks {
            let chunk_data = vec![i + 1; CHUNK_DATA_SIZE / 2]; // Half size for testing
            all_chunks.push(chunk_data.clone());
            full_header.extend(&chunk_data);
        }

        let header_commitment = keccak::hash(&full_header).0;

        (all_chunks, header_commitment)
    }
}

// ── ProgramTest (BPF runtime) integration test helpers ──

pub const TEST_CPI_PROXY_ID: solana_sdk::pubkey::Pubkey =
    solana_sdk::pubkey!("CtQLLKbDMt1XVNXtLKJEt1K8cstbckjqE6zyFqR37KTc");
pub const TEST_CPI_TARGET_ID: solana_sdk::pubkey::Pubkey =
    solana_sdk::pubkey!("GHB99UGVmKFeNrtSLsuzL2QhZZgaqcASvTjotQd2dZzu");
const DEPLOY_DIR: &str = "../../target/deploy";

pub const ANCHOR_ERROR_OFFSET: u32 = 6000;

pub fn anchor_discriminator(instruction_name: &str) -> [u8; 8] {
    let hash = solana_sdk::hash::hash(format!("global:{instruction_name}").as_bytes());
    let mut disc = [0u8; 8];
    disc.copy_from_slice(&hash.to_bytes()[..8]);
    disc
}

pub fn setup_program_test_with_whitelist(
    admin: &solana_sdk::pubkey::Pubkey,
    whitelisted_programs: &[solana_sdk::pubkey::Pubkey],
) -> solana_program_test::ProgramTest {
    use anchor_lang::{AccountSerialize, AnchorSerialize, Discriminator, Space};

    if std::env::var("SBF_OUT_DIR").is_err() {
        let deploy_dir = std::path::Path::new(DEPLOY_DIR);
        std::env::set_var("SBF_OUT_DIR", deploy_dir);
    }

    let mut pt = solana_program_test::ProgramTest::new("ics07_tendermint", crate::ID, None);
    pt.add_program("test_cpi_proxy", TEST_CPI_PROXY_ID, None);
    pt.add_program("test_cpi_target", TEST_CPI_TARGET_ID, None);
    pt.add_program("access_manager", access_manager::ID, None);

    // Pre-create AppState PDA
    let (app_state_pda, _) = solana_sdk::pubkey::Pubkey::find_program_address(
        &[crate::types::AppState::SEED],
        &crate::ID,
    );
    let app_state = crate::types::AppState {
        am_state: AccessManagerState::new(access_manager::ID),
        _reserved: [0; 256],
    };
    let mut app_data = vec![0u8; 8 + crate::types::AppState::INIT_SPACE];
    app_state.try_serialize(&mut &mut app_data[..]).unwrap();

    pt.add_account(
        app_state_pda,
        solana_sdk::account::Account {
            lamports: 1_000_000,
            data: app_data,
            owner: crate::ID,
            executable: false,
            rent_epoch: 0,
        },
    );

    // Pre-create AccessManager PDA with admin role and whitelist
    let (access_manager_pda, _) = solana_sdk::pubkey::Pubkey::find_program_address(
        &[access_manager::state::AccessManager::SEED],
        &access_manager::ID,
    );
    let am = access_manager::state::AccessManager {
        roles: vec![access_manager::RoleData {
            role_id: solana_ibc_types::roles::ADMIN_ROLE,
            members: vec![*admin],
        }],
        whitelisted_programs: whitelisted_programs.to_vec(),
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

    pt
}

pub fn setup_program_test_with_relayer(
    relayer: &solana_sdk::pubkey::Pubkey,
) -> solana_program_test::ProgramTest {
    use anchor_lang::{AccountSerialize, AnchorSerialize, Discriminator, Space};

    if std::env::var("SBF_OUT_DIR").is_err() {
        let deploy_dir = std::path::Path::new(DEPLOY_DIR);
        std::env::set_var("SBF_OUT_DIR", deploy_dir);
    }

    let mut pt = solana_program_test::ProgramTest::new("ics07_tendermint", crate::ID, None);
    pt.add_program("test_cpi_proxy", TEST_CPI_PROXY_ID, None);
    pt.add_program("test_cpi_target", TEST_CPI_TARGET_ID, None);
    pt.add_program("access_manager", access_manager::ID, None);

    let (app_state_pda, _) = solana_sdk::pubkey::Pubkey::find_program_address(
        &[crate::types::AppState::SEED],
        &crate::ID,
    );
    let app_state = crate::types::AppState {
        am_state: AccessManagerState::new(access_manager::ID),
        _reserved: [0; 256],
    };
    let mut app_data = vec![0u8; 8 + crate::types::AppState::INIT_SPACE];
    app_state.try_serialize(&mut &mut app_data[..]).unwrap();

    pt.add_account(
        app_state_pda,
        solana_sdk::account::Account {
            lamports: 1_000_000,
            data: app_data,
            owner: crate::ID,
            executable: false,
            rent_epoch: 0,
        },
    );

    let (access_manager_pda, _) = solana_sdk::pubkey::Pubkey::find_program_address(
        &[access_manager::state::AccessManager::SEED],
        &access_manager::ID,
    );
    let am = access_manager::state::AccessManager {
        roles: vec![
            access_manager::RoleData {
                role_id: solana_ibc_types::roles::ADMIN_ROLE,
                members: vec![*relayer],
            },
            access_manager::RoleData {
                role_id: solana_ibc_types::roles::RELAYER_ROLE,
                members: vec![*relayer],
            },
        ],
        whitelisted_programs: vec![TEST_CPI_TARGET_ID],
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

    pt.add_account(
        *relayer,
        solana_sdk::account::Account {
            lamports: 10_000_000_000,
            data: vec![],
            owner: solana_sdk::system_program::ID,
            executable: false,
            rent_epoch: 0,
        },
    );

    pt
}

pub fn fund_account(
    pt: &mut solana_program_test::ProgramTest,
    pubkey: &solana_sdk::pubkey::Pubkey,
) {
    pt.add_account(
        *pubkey,
        solana_sdk::account::Account {
            lamports: 10_000_000_000,
            data: vec![],
            owner: solana_sdk::system_program::ID,
            executable: false,
            rent_epoch: 0,
        },
    );
}

pub fn wrap_in_test_cpi_proxy(
    payer: solana_sdk::pubkey::Pubkey,
    inner_ix: &solana_sdk::instruction::Instruction,
) -> solana_sdk::instruction::Instruction {
    use anchor_lang::AnchorSerialize;
    use solana_sdk::instruction::{AccountMeta, Instruction};

    let mut data = Vec::new();
    data.extend_from_slice(&anchor_discriminator("proxy_cpi"));
    inner_ix.data.serialize(&mut data).unwrap();
    let meta_count = inner_ix.accounts.len() as u32;
    meta_count.serialize(&mut data).unwrap();
    for meta in &inner_ix.accounts {
        meta.is_signer.serialize(&mut data).unwrap();
        meta.is_writable.serialize(&mut data).unwrap();
    }

    let mut accounts = vec![
        AccountMeta::new_readonly(inner_ix.program_id, false),
        AccountMeta::new_readonly(payer, true),
    ];
    for meta in &inner_ix.accounts {
        accounts.push(if meta.is_writable {
            AccountMeta::new(meta.pubkey, false)
        } else {
            AccountMeta::new_readonly(meta.pubkey, false)
        });
    }

    Instruction {
        program_id: TEST_CPI_PROXY_ID,
        accounts,
        data,
    }
}

pub fn wrap_in_test_cpi_target_proxy(
    payer: solana_sdk::pubkey::Pubkey,
    inner_ix: &solana_sdk::instruction::Instruction,
) -> solana_sdk::instruction::Instruction {
    use anchor_lang::AnchorSerialize;
    use solana_sdk::instruction::{AccountMeta, Instruction};

    let mut data = Vec::new();
    data.extend_from_slice(&anchor_discriminator("proxy_cpi"));
    inner_ix.data.serialize(&mut data).unwrap();
    let meta_count = inner_ix.accounts.len() as u32;
    meta_count.serialize(&mut data).unwrap();
    for meta in &inner_ix.accounts {
        meta.is_signer.serialize(&mut data).unwrap();
        meta.is_writable.serialize(&mut data).unwrap();
    }

    let mut accounts = vec![
        AccountMeta::new_readonly(inner_ix.program_id, false),
        AccountMeta::new_readonly(payer, true),
    ];
    for meta in &inner_ix.accounts {
        accounts.push(if meta.is_writable {
            AccountMeta::new(meta.pubkey, false)
        } else {
            AccountMeta::new_readonly(meta.pubkey, false)
        });
    }

    Instruction {
        program_id: TEST_CPI_TARGET_ID,
        accounts,
        data,
    }
}

pub fn extract_custom_error(err: &solana_program_test::BanksClientError) -> Option<u32> {
    match err {
        solana_program_test::BanksClientError::TransactionError(
            solana_sdk::transaction::TransactionError::InstructionError(
                _,
                solana_sdk::instruction::InstructionError::Custom(code),
            ),
        ) => Some(*code),
        _ => None,
    }
}

/// Access control test utilities
pub mod access_control {
    use access_manager::RoleData;
    use anchor_lang::prelude::Pubkey;
    use anchor_lang::{AnchorSerialize, Discriminator};

    /// Setup access manager account for tests
    /// Returns (PDA, serialized account data)
    pub fn setup_access_manager(admin: Pubkey, relayers: Vec<Pubkey>) -> (Pubkey, Vec<u8>) {
        let (access_manager_pda, _) = Pubkey::find_program_address(
            &[access_manager::state::AccessManager::SEED],
            &access_manager::ID,
        );

        let mut roles = vec![RoleData {
            role_id: solana_ibc_types::roles::ADMIN_ROLE,
            members: vec![admin],
        }];

        if !relayers.is_empty() {
            roles.push(RoleData {
                role_id: solana_ibc_types::roles::RELAYER_ROLE,
                members: relayers,
            });
        }

        let access_manager = access_manager::state::AccessManager {
            roles,
            whitelisted_programs: vec![],
            pending_authority_transfers: vec![],
        };

        let mut data = access_manager::state::AccessManager::DISCRIMINATOR.to_vec();
        access_manager.serialize(&mut data).unwrap();

        (access_manager_pda, data)
    }

    /// Create access manager account for mollusk tests
    pub fn create_access_manager_account(
        admin: Pubkey,
        relayers: Vec<Pubkey>,
    ) -> (Pubkey, solana_sdk::account::Account) {
        let (pda, data) = setup_access_manager(admin, relayers);

        let account = solana_sdk::account::Account {
            lamports: 10_000_000,
            data,
            owner: access_manager::ID,
            executable: false,
            rent_epoch: 0,
        };

        (pda, account)
    }
}

/// Create a BPF Loader Upgradeable `ProgramData` account for testing.
pub fn create_program_data_account(
    program_id: &solana_sdk::pubkey::Pubkey,
    authority: Option<solana_sdk::pubkey::Pubkey>,
) -> (solana_sdk::pubkey::Pubkey, solana_sdk::account::Account) {
    use solana_sdk::bpf_loader_upgradeable::{self, UpgradeableLoaderState};

    let (program_data_pda, _) = solana_sdk::pubkey::Pubkey::find_program_address(
        &[program_id.as_ref()],
        &bpf_loader_upgradeable::ID,
    );

    let state = UpgradeableLoaderState::ProgramData {
        slot: 0,
        upgrade_authority_address: authority,
    };
    let data = bincode::serialize(&state).unwrap();

    (
        program_data_pda,
        solana_sdk::account::Account {
            lamports: 1_000_000,
            data,
            owner: bpf_loader_upgradeable::ID,
            executable: false,
            rent_epoch: 0,
        },
    )
}

/// Create instructions sysvar account for direct call (not CPI)
pub fn create_instructions_sysvar_account() -> solana_sdk::account::Account {
    use solana_sdk::pubkey::Pubkey;
    use solana_sdk::sysvar::instructions::{
        construct_instructions_data, BorrowedAccountMeta, BorrowedInstruction,
    };

    // Create minimal mock instruction to simulate direct call
    // Current instruction has this program as the program_id
    let account_pubkey = Pubkey::new_unique();
    let account = BorrowedAccountMeta {
        pubkey: &account_pubkey,
        is_signer: false,
        is_writable: true,
    };
    let mock_instruction = BorrowedInstruction {
        program_id: &crate::ID, // Direct call to our program
        accounts: vec![account],
        data: &[],
    };

    let ixs_data = construct_instructions_data(&[mock_instruction]);

    solana_sdk::account::Account {
        lamports: 1_000_000,
        data: ixs_data,
        owner: solana_sdk::sysvar::ID,
        executable: false,
        rent_epoch: 0,
    }
}
