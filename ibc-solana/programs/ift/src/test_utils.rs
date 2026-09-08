// SPDX-License-Identifier: Apache-2.0

//! Test utilities for ICS27 IFT program
//!
//! Provides helper functions for mollusk-based unit tests.

use anchor_lang::prelude::*;
use mollusk_svm::Mollusk;
use solana_sdk::{account::Account as SolanaAccount, pubkey::Pubkey, system_program};

use crate::constants::*;
use crate::state::{
    AccountVersion, ChainOptions, IFTAppMintState, IFTAppState, IFTBridge, PendingTransfer,
};

/// Path to the compiled IFT program binary
pub const IFT_PROGRAM_PATH: &str = "../../target/deploy/ift";

/// Anchor error code offset
pub const ANCHOR_ERROR_OFFSET: u32 = 6000;

/// Generate a valid bech32 cosmos address for testing
pub fn valid_cosmos_address() -> String {
    let hrp = bech32::Hrp::parse("cosmos").expect("valid HRP");
    let mut addr = String::new();
    bech32::encode_to_fmt::<bech32::Bech32, _>(&mut addr, hrp, &[0u8; 20])
        .expect("valid bech32 encoding");
    addr
}

/// Setup mollusk with the IFT program
pub fn setup_mollusk() -> Mollusk {
    Mollusk::new(&crate::ID, IFT_PROGRAM_PATH)
}

/// Setup mollusk with the IFT program and SPL Token program (for CPI tests)
pub fn setup_mollusk_with_token() -> Mollusk {
    let mut mollusk = Mollusk::new(&crate::ID, IFT_PROGRAM_PATH);
    mollusk_svm_programs_token::token::add_program(&mut mollusk);
    mollusk
}

/// Get the SPL Token program keyed account for use in CPI tests.
/// Unlike `create_token_program_account()`, this includes the real program ELF.
pub fn token_program_keyed_account() -> (Pubkey, SolanaAccount) {
    mollusk_svm_programs_token::token::keyed_account()
}

/// Setup mollusk with the IFT program and Token 2022 program (for CPI tests)
pub fn setup_mollusk_with_token_2022() -> Mollusk {
    let mut mollusk = Mollusk::new(&crate::ID, IFT_PROGRAM_PATH);
    mollusk_svm_programs_token::token2022::add_program(&mut mollusk);
    mollusk
}

/// Get the Token 2022 program keyed account for use in CPI tests.
pub fn token_2022_keyed_account() -> (Pubkey, SolanaAccount) {
    mollusk_svm_programs_token::token2022::keyed_account()
}

/// Create a serialized global IFT app state account
pub fn create_ift_app_state_account(bump: u8, admin: Pubkey) -> SolanaAccount {
    create_ift_app_state_account_with_options(bump, admin, false)
}

/// Create a serialized global IFT app state account with configurable paused state
pub fn create_ift_app_state_account_with_options(
    bump: u8,
    admin: Pubkey,
    paused: bool,
) -> SolanaAccount {
    create_ift_app_state_account_full(bump, admin, paused, None)
}

/// Create a serialized global IFT app state account with all configurable fields.
///
/// The account buffer is always allocated to the maximum size
/// (discriminator + `INIT_SPACE`) so that Anchor can write back
/// `Option<Pubkey>` variants of any size without `AccountDidNotSerialize`.
pub fn create_ift_app_state_account_full(
    bump: u8,
    admin: Pubkey,
    paused: bool,
    pending_admin: Option<Pubkey>,
) -> SolanaAccount {
    let app_state = IFTAppState {
        version: AccountVersion::V1,
        bump,
        admin,
        paused,
        pending_admin,
        _reserved: [0; 128],
    };

    let mut data = vec![0u8; 8 + IFTAppState::INIT_SPACE];
    data[0..8].copy_from_slice(IFTAppState::DISCRIMINATOR);
    app_state.serialize(&mut &mut data[8..]).unwrap();

    SolanaAccount {
        lamports: 1_000_000,
        data,
        owner: crate::ID,
        executable: false,
        rent_epoch: 0,
    }
}

/// Parameters for creating a full IFT app mint state account
pub struct IftAppMintStateParams {
    pub mint: Pubkey,
    pub bump: u8,
    pub mint_authority_bump: u8,
    pub daily_mint_limit: u64,
    pub rate_limit_day: u64,
    pub rate_limit_daily_usage: u64,
}

/// Create a serialized per-mint IFT app state account
pub fn create_ift_app_mint_state_account(
    mint: Pubkey,
    bump: u8,
    mint_authority_bump: u8,
) -> SolanaAccount {
    create_ift_app_mint_state_account_full(IftAppMintStateParams {
        mint,
        bump,
        mint_authority_bump,
        daily_mint_limit: 0,
        rate_limit_day: 0,
        rate_limit_daily_usage: 0,
    })
}

/// Create a serialized per-mint IFT app state account with all configurable fields
pub fn create_ift_app_mint_state_account_full(params: IftAppMintStateParams) -> SolanaAccount {
    let mint_state = IFTAppMintState {
        version: AccountVersion::V1,
        bump: params.bump,
        mint: params.mint,
        mint_authority_bump: params.mint_authority_bump,
        daily_mint_limit: params.daily_mint_limit,
        rate_limit_day: params.rate_limit_day,
        rate_limit_daily_usage: params.rate_limit_daily_usage,
        _reserved: [0; 128],
    };

    let mut data = IFTAppMintState::DISCRIMINATOR.to_vec();
    mint_state.serialize(&mut data).unwrap();

    SolanaAccount {
        lamports: 1_000_000,
        data,
        owner: crate::ID,
        executable: false,
        rent_epoch: 0,
    }
}

/// Create a serialized IFT bridge account
pub fn create_ift_bridge_account(
    mint: Pubkey,
    client_id: &str,
    counterparty_ift_address: &str,
    chain_options: ChainOptions,
    bump: u8,
    active: bool,
) -> SolanaAccount {
    let bridge = IFTBridge {
        version: AccountVersion::V1,
        bump,
        mint,
        client_id: client_id.to_string(),
        counterparty_ift_address: counterparty_ift_address.to_string(),
        chain_options,
        active,
        _reserved: [0; 64],
    };

    let mut data = IFTBridge::DISCRIMINATOR.to_vec();
    bridge.serialize(&mut data).unwrap();

    SolanaAccount {
        lamports: 1_000_000,
        data,
        owner: crate::ID,
        executable: false,
        rent_epoch: 0,
    }
}

/// Create a serialized pending transfer account
pub fn create_pending_transfer_account(
    mint: Pubkey,
    client_id: &str,
    sequence: u64,
    sender: Pubkey,
    amount: u64,
    bump: u8,
) -> SolanaAccount {
    let pending = PendingTransfer {
        version: AccountVersion::V1,
        bump,
        mint,
        client_id: client_id.to_string(),
        sequence,
        sender,
        amount,
        timestamp: 1_700_000_000,
        _reserved: [0; 32],
    };

    let mut data = PendingTransfer::DISCRIMINATOR.to_vec();
    pending.serialize(&mut data).unwrap();

    SolanaAccount {
        lamports: 1_000_000,
        data,
        owner: crate::ID,
        executable: false,
        rent_epoch: 0,
    }
}

/// Create a signer account with lamports
pub fn create_signer_account() -> SolanaAccount {
    SolanaAccount {
        lamports: 1_000_000_000,
        data: vec![],
        owner: system_program::ID,
        executable: false,
        rent_epoch: 0,
    }
}

/// Create an uninitialized PDA account (for account init)
pub fn create_uninitialized_pda() -> SolanaAccount {
    SolanaAccount {
        lamports: 0,
        data: vec![],
        owner: system_program::ID,
        executable: false,
        rent_epoch: 0,
    }
}

/// Create system program account
pub fn create_system_program_account() -> (Pubkey, SolanaAccount) {
    (
        system_program::ID,
        SolanaAccount {
            lamports: 1,
            data: vec![],
            owner: solana_sdk::native_loader::ID,
            executable: true,
            rent_epoch: 0,
        },
    )
}

/// Create clock sysvar account
pub fn create_clock_sysvar_account(unix_timestamp: i64) -> (Pubkey, SolanaAccount) {
    let clock = solana_sdk::sysvar::clock::Clock {
        slot: 1000,
        epoch_start_timestamp: 0,
        epoch: 1,
        leader_schedule_epoch: 1,
        unix_timestamp,
    };

    (
        solana_sdk::sysvar::clock::ID,
        SolanaAccount {
            lamports: 1,
            data: bincode::serialize(&clock).expect("Failed to serialize Clock sysvar"),
            owner: solana_sdk::sysvar::ID,
            executable: false,
            rent_epoch: 0,
        },
    )
}

/// Create a BPF Loader Upgradeable `ProgramData` account for testing.
pub fn create_program_data_account(
    program_id: &Pubkey,
    authority: Option<Pubkey>,
) -> (Pubkey, SolanaAccount) {
    use solana_sdk::bpf_loader_upgradeable::{self, UpgradeableLoaderState};

    let (program_data_pda, _) =
        Pubkey::find_program_address(&[program_id.as_ref()], &bpf_loader_upgradeable::ID);

    let state = UpgradeableLoaderState::ProgramData {
        slot: 0,
        upgrade_authority_address: authority,
    };
    let data = bincode::serialize(&state).unwrap();

    (
        program_data_pda,
        SolanaAccount {
            lamports: 1_000_000,
            data,
            owner: bpf_loader_upgradeable::ID,
            executable: false,
            rent_epoch: 0,
        },
    )
}

/// Get the global IFT app state PDA (singleton, no mint param)
pub fn get_app_state_pda() -> (Pubkey, u8) {
    Pubkey::find_program_address(&[IFT_APP_STATE_SEED], &crate::ID)
}

/// Get the per-mint IFT app state PDA
pub fn get_app_mint_state_pda(mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[IFT_APP_MINT_STATE_SEED, mint.as_ref()], &crate::ID)
}

/// Get the IFT bridge PDA
pub fn get_bridge_pda(mint: &Pubkey, client_id: &str) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[IFT_BRIDGE_SEED, mint.as_ref(), client_id.as_bytes()],
        &crate::ID,
    )
}

/// Get the pending transfer PDA
pub fn get_pending_transfer_pda(mint: &Pubkey, client_id: &str, sequence: u64) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[
            PENDING_TRANSFER_SEED,
            mint.as_ref(),
            client_id.as_bytes(),
            &sequence.to_le_bytes(),
        ],
        &crate::ID,
    )
}

/// Get the mint authority PDA
pub fn get_mint_authority_pda(mint: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[MINT_AUTHORITY_SEED, mint.as_ref()], &crate::ID)
}

/// Deserialize global IFT app state from account
pub fn deserialize_app_state(account: &SolanaAccount) -> IFTAppState {
    anchor_lang::AccountDeserialize::try_deserialize(&mut &account.data[..])
        .expect("Failed to deserialize IFTAppState")
}

/// Deserialize per-mint IFT app state from account
pub fn deserialize_app_mint_state(account: &SolanaAccount) -> IFTAppMintState {
    anchor_lang::AccountDeserialize::try_deserialize(&mut &account.data[..])
        .expect("Failed to deserialize IFTAppMintState")
}

/// Deserialize IFT bridge from account
pub fn deserialize_bridge(account: &SolanaAccount) -> IFTBridge {
    anchor_lang::AccountDeserialize::try_deserialize(&mut &account.data[..])
        .expect("Failed to deserialize IFTBridge")
}

/// Create a mock SPL Token account with specified mint, owner, and amount
pub fn create_token_account(mint: Pubkey, owner: Pubkey, amount: u64) -> SolanaAccount {
    use anchor_spl::token::spl_token;
    use solana_sdk::{program_pack::Pack, rent::Rent};

    let account = spl_token::state::Account {
        mint,
        owner,
        amount,
        delegate: solana_sdk::program_option::COption::None,
        state: spl_token::state::AccountState::Initialized,
        is_native: solana_sdk::program_option::COption::None,
        delegated_amount: 0,
        close_authority: solana_sdk::program_option::COption::None,
    };

    let mut data = vec![0u8; spl_token::state::Account::LEN];
    account.pack_into_slice(&mut data);

    SolanaAccount {
        lamports: Rent::default().minimum_balance(spl_token::state::Account::LEN),
        data,
        owner: spl_token::ID,
        executable: false,
        rent_epoch: 0,
    }
}

pub fn get_gmp_account_pda(client_id: &str, counterparty_address: &str) -> (Pubkey, u8) {
    use solana_ibc_gmp_types::{GMPAccount, Salt};

    let gmp_account = GMPAccount::new(
        client_id.to_string().try_into().expect("valid client_id"),
        counterparty_address
            .to_string()
            .try_into()
            .expect("valid sender"),
        Salt::empty(),
        &ics27_gmp::ID,
    );
    gmp_account.pda()
}

/// Create a GMP program account
pub fn create_gmp_program_account() -> SolanaAccount {
    SolanaAccount {
        lamports: 1,
        data: vec![],
        owner: solana_sdk::native_loader::ID,
        executable: true,
        rent_epoch: 0,
    }
}

/// Create a mock SPL Token mint account with specified mint authority
pub fn create_mint_account(mint_authority: Pubkey, decimals: u8) -> SolanaAccount {
    use anchor_spl::token::spl_token;
    use solana_sdk::program_pack::Pack;

    let mint = spl_token::state::Mint {
        mint_authority: solana_sdk::program_option::COption::Some(mint_authority),
        supply: 1_000_000_000,
        decimals,
        is_initialized: true,
        freeze_authority: solana_sdk::program_option::COption::None,
    };

    let mut data = vec![0u8; spl_token::state::Mint::LEN];
    mint.pack_into_slice(&mut data);

    SolanaAccount {
        lamports: 1_000_000,
        data,
        owner: spl_token::ID,
        executable: false,
        rent_epoch: 0,
    }
}

/// Create token program account
pub fn create_token_program_account() -> (Pubkey, SolanaAccount) {
    use anchor_spl::token::spl_token;

    (
        spl_token::ID,
        SolanaAccount {
            lamports: 1,
            data: vec![],
            owner: solana_sdk::native_loader::ID,
            executable: true,
            rent_epoch: 0,
        },
    )
}

/// Create instructions sysvar account for direct call (not CPI).
/// The sysvar data indicates IFT program is the top-level caller.
pub fn create_instructions_sysvar_account() -> (Pubkey, SolanaAccount) {
    use solana_sdk::sysvar::instructions::{
        construct_instructions_data, BorrowedAccountMeta, BorrowedInstruction,
    };

    let account_pubkey = Pubkey::new_unique();
    let account = BorrowedAccountMeta {
        pubkey: &account_pubkey,
        is_signer: false,
        is_writable: true,
    };
    let mock_instruction = BorrowedInstruction {
        program_id: &crate::ID,
        accounts: vec![account],
        data: &[],
    };

    let ixs_data = construct_instructions_data(&[mock_instruction]);

    (
        solana_sdk::sysvar::instructions::ID,
        SolanaAccount {
            lamports: 1_000_000,
            data: ixs_data,
            owner: solana_sdk::sysvar::ID,
            executable: false,
            rent_epoch: 0,
        },
    )
}

/// Create instructions sysvar that simulates a CPI call from another program.
/// Uses the real sysvar address but with a different `program_id` to simulate CPI context.
pub fn create_cpi_instructions_sysvar_account(
    caller_program_id: Pubkey,
) -> (Pubkey, SolanaAccount) {
    use solana_sdk::sysvar::instructions::{
        construct_instructions_data, BorrowedAccountMeta, BorrowedInstruction,
    };

    let account_pubkey = Pubkey::new_unique();
    let account = BorrowedAccountMeta {
        pubkey: &account_pubkey,
        is_signer: false,
        is_writable: true,
    };
    let mock_instruction = BorrowedInstruction {
        program_id: &caller_program_id,
        accounts: vec![account],
        data: &[],
    };

    let ixs_data = construct_instructions_data(&[mock_instruction]);

    (
        solana_sdk::sysvar::instructions::ID,
        SolanaAccount {
            lamports: 1_000_000,
            data: ixs_data,
            owner: solana_sdk::sysvar::ID,
            executable: false,
            rent_epoch: 0,
        },
    )
}

pub fn get_gmp_result_pda(client_id: &str, sequence: u64) -> (Pubkey, u8) {
    use solana_ibc_types::GMPCallResult;
    Pubkey::find_program_address(
        &[
            GMPCallResult::SEED,
            client_id.as_bytes(),
            &sequence.to_le_bytes(),
        ],
        &ics27_gmp::ID,
    )
}

pub fn create_gmp_result_account(
    sender: Pubkey,
    sequence: u64,
    source_client: &str,
    dest_client: &str,
    status: solana_ibc_types::CallResultStatus,
    bump: u8,
) -> SolanaAccount {
    use ics27_gmp::state::{AccountVersion, GMPCallResultAccount};

    let gmp_result = GMPCallResultAccount {
        version: AccountVersion::V1,
        sender,
        sequence,
        source_client: source_client.to_string(),
        dest_client: dest_client.to_string(),
        status,
        result_timestamp: 1_700_000_000,
        bump,
    };

    let mut data = GMPCallResultAccount::DISCRIMINATOR.to_vec();
    gmp_result.serialize(&mut data).unwrap();

    SolanaAccount {
        lamports: 1_000_000,
        data,
        owner: ics27_gmp::ID,
        executable: false,
        rent_epoch: 0,
    }
}
