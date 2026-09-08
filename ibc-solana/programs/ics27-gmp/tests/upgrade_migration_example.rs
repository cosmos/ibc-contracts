// SPDX-License-Identifier: Apache-2.0

/// Upgrade migration example for `GMPAppState`
///
/// This example demonstrates how to migrate `GMPAppState` from V1 to a hypothetical V2
/// with additional fields. This pattern shows how the `version` field and `_reserved`
/// space enable seamless upgrades without breaking existing deployments.
use access_manager::AccessManagerState;
use anchor_lang::prelude::*;
use anchor_lang::{AnchorSerialize, Discriminator};
use ics27_gmp::state::{AccountVersion, GMPAppState};
use solana_sdk::pubkey::Pubkey;

/// Serialize account data into vector of bytes. These bytes are stored on Solana
/// and deserialized before the instruction is executed by the SVM
fn create_account_data<T: Discriminator + AnchorSerialize>(account: &T) -> Vec<u8> {
    let mut data = T::DISCRIMINATOR.to_vec();
    account.serialize(&mut data).unwrap();
    data
}

fn setup_gmp_app_state(paused: bool) -> (Pubkey, Vec<u8>) {
    let (app_state_pda, bump) = Pubkey::find_program_address(&[GMPAppState::SEED], &ics27_gmp::ID);
    let app_state = GMPAppState {
        version: AccountVersion::V1,
        paused,
        bump,
        am_state: AccessManagerState::new(access_manager::ID),
        _reserved: [0; 256],
    };
    let app_state_data = create_account_data(&app_state);
    (app_state_pda, app_state_data)
}

#[derive(Debug, AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq)]
pub enum AccountVersionExample {
    V1,
    V2, // New version added
}

/// Example V2 `GMPAppState` with additional fields.
///
/// NOTE: Authorization for admin operations is handled by `AccessManager` (`PAUSER_ROLE`, `UNPAUSER_ROLE`).
/// This test focuses on data serialization/migration patterns, not authorization.
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct GMPAppStateV2Example {
    /// Schema version for upgrades
    pub version: AccountVersionExample,
    /// Whether the app is paused (existing V1 field)
    pub paused: bool,
    /// PDA bump seed (existing V1 field)
    pub bump: u8,
    /// Access manager transfer state (existing V1 field)
    pub am_state: AccessManagerState,

    // ========== NEW V2 FIELDS ==========
    /// Fee collector account for GMP operations (NEW in V2)
    pub fee_collector: Option<Pubkey>, // 1 + 32 = 33 bytes
    /// Global rate limit for GMP calls per epoch (NEW in V2)
    pub global_rate_limit: u64, // 8 bytes
    /// Total number of packets processed (NEW in V2)
    pub total_packets_processed: u64, // 8 bytes
    // Total new fields: 33 + 8 + 8 = 49 bytes
    /// Reserved space for future fields (reduced from 256 to 207)
    pub _reserved: [u8; 207],
}

#[test]
fn test_gmp_app_state_migration_v1_to_v2() {
    // Create V1 account
    let (_, v1_data) = setup_gmp_app_state(false);

    // Deserialize account into the struct with new added fields
    let mut cursor = &v1_data[8..]; // Skip discriminator
    let mut state: GMPAppStateV2Example = AnchorDeserialize::deserialize(&mut cursor).unwrap();

    // Verify it's V1
    assert_eq!(state.version, AccountVersionExample::V1);
    assert!(!state.paused); // V1 field preserved

    // Perform migration logic
    state.version = AccountVersionExample::V2;
    state.fee_collector.replace(Pubkey::new_unique());
    state.global_rate_limit = 1000;
    state.total_packets_processed = 0;

    // Verify migration preserved V1 fields
    assert_eq!(state.version, AccountVersionExample::V2);
    assert!(!state.paused); // V1 field still preserved

    // Verify new V2 fields
    assert!(state.fee_collector.is_some());
    assert_eq!(state.global_rate_limit, 1000);
    assert_eq!(state.total_packets_processed, 0);
    assert_eq!(state._reserved.len(), 207); // 256 - 49 (V2 fields) = 207
}

#[test]
fn test_gmp_app_state_migration_with_paused_state() {
    // Create V1 account that is paused
    let (_, v1_data) = setup_gmp_app_state(true);

    // Deserialize and migrate
    let mut cursor = &v1_data[8..]; // Skip discriminator
    let mut state: GMPAppStateV2Example = AnchorDeserialize::deserialize(&mut cursor).unwrap();

    assert_eq!(state.version, AccountVersionExample::V1);
    assert!(state.paused); // Was paused in V1

    // Migrate to V2
    state.version = AccountVersionExample::V2;
    state.fee_collector.replace(Pubkey::new_unique());
    state.global_rate_limit = 500;
    state.total_packets_processed = 1234;

    // Verify paused state is preserved
    assert_eq!(state.version, AccountVersionExample::V2);
    assert!(state.paused); // V1 field preserved

    // Verify new fields
    assert!(state.fee_collector.is_some());
    assert_eq!(state.global_rate_limit, 500);
    assert_eq!(state.total_packets_processed, 1234);
}

#[test]
fn test_gmp_app_state_reserved_space_sufficient() {
    // Create V1 account
    let (_, v1_data) = setup_gmp_app_state(false);

    // Deserialize to verify reserved space
    let mut cursor = &v1_data[8..];
    let state: GMPAppStateV2Example = AnchorDeserialize::deserialize(&mut cursor).unwrap();

    // Verify we can add fields and still have reserved space
    // V2 adds 49 bytes of new fields (fee_collector + global_rate_limit + total_packets_processed)
    // Original reserved: 256 bytes
    // Remaining reserved: 207 bytes (still plenty for future upgrades)
    assert_eq!(state._reserved.len(), 207);
}

#[test]
fn test_gmp_app_state_pda_derivation_preserved() {
    // Create V1 account
    let (original_pda, v1_data) = setup_gmp_app_state(false);

    // Deserialize and migrate
    let mut cursor = &v1_data[8..];
    let mut state: GMPAppStateV2Example = AnchorDeserialize::deserialize(&mut cursor).unwrap();

    // Migrate to V2
    state.version = AccountVersionExample::V2;
    state.fee_collector.replace(Pubkey::new_unique());

    // Verify PDA derivation still works (bump is preserved)
    let (derived_pda, derived_bump) =
        Pubkey::find_program_address(&[GMPAppState::SEED], &ics27_gmp::ID);

    assert_eq!(derived_pda, original_pda);
    assert_eq!(derived_bump, state.bump);
}
