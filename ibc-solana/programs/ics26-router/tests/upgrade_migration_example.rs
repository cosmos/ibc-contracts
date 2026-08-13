// SPDX-License-Identifier: Apache-2.0

/// Upgrade migration example. This example defines it's own V2 structs into
/// which the existing data is loaded. In reality new fields would be added to
/// the existing account structs. The existing data would then be loaded into
/// those structs.
///
use access_manager::AccessManagerState;
use anchor_lang::prelude::*;
use anchor_lang::{AnchorSerialize, Discriminator};
use ics26_router::state::{AccountVersion, Client, CounterpartyInfo, RouterState};
use solana_sdk::pubkey::Pubkey;

/// Serialize account data into vector of bytes. This bytes are stored on Solana
/// and deserialized before the instruction is executed by the SVM
fn create_account_data<T: Discriminator + AnchorSerialize>(account: &T) -> Vec<u8> {
    let mut data = T::DISCRIMINATOR.to_vec();
    account.serialize(&mut data).unwrap();
    data
}

fn setup_router_state() -> (Pubkey, Vec<u8>) {
    let (router_state_pda, _) =
        Pubkey::find_program_address(&[RouterState::SEED], &ics26_router::ID);
    let router_state = RouterState {
        version: AccountVersion::V1,
        am_state: AccessManagerState::new(access_manager::ID),
        paused: false,
        _reserved: [0; 256],
    };
    let router_state_data = create_account_data(&router_state);
    (router_state_pda, router_state_data)
}

fn setup_client_state(
    client_id: &str,
    light_client_program: Pubkey,
    counterparty_client_id: &str,
    active: bool,
) -> (Pubkey, Vec<u8>) {
    let (client_pda, _) =
        Pubkey::find_program_address(&[Client::SEED, client_id.as_bytes()], &ics26_router::ID);

    let client = Client {
        version: AccountVersion::V1,
        client_id: client_id.to_string(),
        client_program_id: light_client_program,
        counterparty_info: CounterpartyInfo {
            client_id: counterparty_client_id.to_string(),
            merkle_prefix: vec![vec![0x01, 0x02, 0x03]],
        },
        active,
        _reserved: [0; 256],
    };
    let client_data = create_account_data(&client);

    (client_pda, client_data)
}

#[derive(Debug, AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum AccountVersionExample {
    V1,
    V2, // New version added
}

/// Example V2 `RouterState` demonstrating data migration pattern.
///
/// NOTE: Authorization for upgrades is handled by `AccessManager` (`ADMIN_ROLE`).
/// This test focuses on data serialization/migration, not authorization.
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct RouterStateExample {
    /// Schema version for upgrades
    pub version: AccountVersionExample,
    /// Access manager transfer state (existing V1 field)
    pub am_state: AccessManagerState,

    // ========== NEW V2 FIELDS ==========
    /// Fee collector account
    pub fee_collector: Option<Pubkey>, // 1 + 32 = 33 bytes
    /// Global rate limit
    pub global_rate_limit: u64, // 8 bytes
    // Total new fields: 33 + 8 = 41 bytes
    /// Reserved space for future fields (reduced from 256 to 215)
    pub _reserved: [u8; 215],
}

/// Example V2 Client demonstrating data migration pattern.
/// NOTE: Authorization is handled by `AccessManager`, not stored per-client.
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ClientExample {
    /// Schema version for upgrades
    pub version: AccountVersionExample,
    /// The client identifier
    pub client_id: String, // max 64 bytes
    /// The program ID of the light client
    pub client_program_id: Pubkey,
    /// Counterparty chain information
    pub counterparty_info: CounterpartyInfo,
    /// Whether the client is active
    pub active: bool,

    // ========== NEW V2 FIELDS ==========
    /// Client-specific rate limit (NEW in V2)
    pub rate_limit_per_client: Option<u64>, // 1 + 8 = 9 bytes
    /// Number of packets processed (NEW in V2)
    pub packet_count: u64, // 8 bytes
    // Total new fields: 9 + 8 = 17 bytes
    /// Reserved space for future fields (reduced from 256 to 239)
    pub _reserved: [u8; 239],
}

#[test]
fn test_router_state_migration_v1_to_v2() {
    // Create V1 account
    let (_, v1_data) = setup_router_state();

    // Deserialize account into the struct with new added fields
    let mut cursor = &v1_data[8..]; // Skip discriminator
    let mut state: RouterStateExample = AnchorDeserialize::deserialize(&mut cursor).unwrap();

    // Verify it's V1
    assert_eq!(state.version, AccountVersionExample::V1);

    // Here the actual migration logic would be done
    state.version = AccountVersionExample::V2;
    state.fee_collector.replace(Pubkey::new_unique());
    state.global_rate_limit = 10;

    // Verify migration
    assert_eq!(state.version, AccountVersionExample::V2);
    assert!(state.fee_collector.is_some());
    assert_eq!(state.global_rate_limit, 10);
    assert_eq!(state._reserved.len(), 215);
}

#[test]
fn test_client_migration_v1_to_v2() {
    // Create V1 client
    let client_id = "07-tendermint-0";
    let light_client = Pubkey::new_unique();
    let counterparty = "07-tendermint-1";

    let (_, v1_data) = setup_client_state(client_id, light_client, counterparty, true);

    // Deserialize V1 account
    let mut cursor = &v1_data[8..]; // Skip discriminator
    let mut state: ClientExample = AnchorDeserialize::deserialize(&mut cursor).unwrap();

    // Verify it's V1
    assert_eq!(state.version, AccountVersionExample::V1);

    // Here the actual migration logic would be done
    state.version = AccountVersionExample::V2;
    state.rate_limit_per_client.replace(5);
    state.packet_count = 99;

    // Verify migration preserved V1 fields
    assert_eq!(state.version, AccountVersionExample::V2);
    assert_eq!(state.client_id, client_id);
    assert_eq!(state.client_program_id, light_client);
    assert!(state.active);
    assert_eq!(state.counterparty_info.client_id, counterparty);

    // Verify new V2 fields
    assert_eq!(state.rate_limit_per_client, Some(5));
    assert_eq!(state.packet_count, 99);
    assert_eq!(state._reserved.len(), 239);
}
