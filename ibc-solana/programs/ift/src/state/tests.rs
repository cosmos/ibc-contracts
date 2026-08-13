// SPDX-License-Identifier: Apache-2.0

//! Tests for state module

use super::*;
use solana_sdk::pubkey::Pubkey;

#[test]
fn test_ift_app_state_seeds() {
    let seeds = IFTAppState::seeds();

    assert_eq!(seeds.len(), 1);
    assert_eq!(seeds[0], IFT_APP_STATE_SEED.to_vec());
}

#[test]
fn test_ift_app_mint_state_seeds() {
    let mint = Pubkey::new_unique();
    let seeds = IFTAppMintState::seeds(&mint);

    assert_eq!(seeds.len(), 2);
    assert_eq!(seeds[0], IFT_APP_MINT_STATE_SEED.to_vec());
    assert_eq!(seeds[1], mint.as_ref().to_vec());
}

#[test]
fn test_ift_bridge_seeds() {
    let mint = Pubkey::new_unique();
    let client_id = "07-tendermint-0";
    let seeds = IFTBridge::seeds(&mint, client_id);

    assert_eq!(seeds.len(), 3);
    assert_eq!(seeds[0], IFT_BRIDGE_SEED.to_vec());
    assert_eq!(seeds[1], mint.as_ref().to_vec());
    assert_eq!(seeds[2], client_id.as_bytes().to_vec());
}

#[test]
fn test_ift_bridge_seeds_empty_client_id() {
    let mint = Pubkey::new_unique();
    let seeds = IFTBridge::seeds(&mint, "");

    assert_eq!(seeds.len(), 3);
    assert_eq!(seeds[2], Vec::<u8>::new());
}

#[test]
fn test_pending_transfer_seeds() {
    let mint = Pubkey::new_unique();
    let client_id = "07-tendermint-0";
    let sequence = 42u64;
    let seeds = PendingTransfer::seeds(&mint, client_id, sequence);

    assert_eq!(seeds.len(), 4);
    assert_eq!(seeds[0], PENDING_TRANSFER_SEED.to_vec());
    assert_eq!(seeds[1], mint.as_ref().to_vec());
    assert_eq!(seeds[2], client_id.as_bytes().to_vec());
    assert_eq!(seeds[3], sequence.to_le_bytes().to_vec());
}

#[test]
fn test_pending_transfer_seeds_sequence_zero() {
    let mint = Pubkey::new_unique();
    let seeds = PendingTransfer::seeds(&mint, "client", 0);

    assert_eq!(seeds[3], 0u64.to_le_bytes().to_vec());
}

#[test]
fn test_pending_transfer_seeds_sequence_max() {
    let mint = Pubkey::new_unique();
    let seeds = PendingTransfer::seeds(&mint, "client", u64::MAX);

    assert_eq!(seeds[3], u64::MAX.to_le_bytes().to_vec());
}

#[test]
fn test_account_version_default() {
    let version = AccountVersion::default();
    assert_eq!(version, AccountVersion::V1);
}

#[test]
fn test_ift_bridge_serialization_roundtrip() {
    use anchor_lang::AccountDeserialize;

    let mint = Pubkey::new_unique();
    let bridge = IFTBridge {
        version: AccountVersion::V1,
        bump: 42,
        mint,
        client_id: "07-tendermint-0".to_string(),
        counterparty_ift_address: "0x1234567890abcdef".to_string(),
        chain_options: ChainOptions::Evm,
        active: true,
        _reserved: [0; 64],
    };

    // Serialize with discriminator
    let mut data = IFTBridge::DISCRIMINATOR.to_vec();
    bridge.serialize(&mut data).unwrap();

    // Deserialize
    let deserialized: IFTBridge =
        IFTBridge::try_deserialize(&mut &data[..]).expect("Failed to deserialize");

    assert_eq!(deserialized.version, AccountVersion::V1);
    assert_eq!(deserialized.bump, 42);
    assert_eq!(deserialized.mint, mint);
    assert_eq!(deserialized.client_id, "07-tendermint-0");
    assert_eq!(deserialized.counterparty_ift_address, "0x1234567890abcdef");
    assert!(matches!(deserialized.chain_options, ChainOptions::Evm));
    assert!(deserialized.active);
}

#[test]
fn test_chain_options_evm_validate() {
    assert!(ChainOptions::Evm.validate().is_ok());
}

#[test]
fn test_chain_options_cosmos_validate_success() {
    let hrp = bech32::Hrp::parse("cosmos").expect("valid HRP");
    let mut addr = String::new();
    bech32::encode_to_fmt::<bech32::Bech32, _>(&mut addr, hrp, &[0u8; 20])
        .expect("valid bech32 encoding");

    let options = ChainOptions::Cosmos {
        denom: "uatom".to_string(),
        type_url: "/cosmos.ift.v1.MsgIFTMint".to_string(),
        ica_address: addr,
    };
    assert!(options.validate().is_ok());
}

#[test]
fn test_chain_options_cosmos_validate_invalid_bech32() {
    let options = ChainOptions::Cosmos {
        denom: "uatom".to_string(),
        type_url: "/cosmos.ift.v1.MsgIFTMint".to_string(),
        ica_address: "not-valid-bech32".to_string(),
    };
    let err = options.validate().unwrap_err();
    assert_eq!(
        err,
        anchor_lang::error!(crate::errors::IFTError::InvalidCosmosIcaAddress)
    );
}

#[test]
fn test_chain_options_cosmos_validate_empty_denom() {
    let hrp = bech32::Hrp::parse("cosmos").expect("valid HRP");
    let mut addr = String::new();
    bech32::encode_to_fmt::<bech32::Bech32, _>(&mut addr, hrp, &[0u8; 20])
        .expect("valid bech32 encoding");

    let options = ChainOptions::Cosmos {
        denom: String::new(),
        type_url: "/cosmos.ift.v1.MsgIFTMint".to_string(),
        ica_address: addr,
    };
    let err = options.validate().unwrap_err();
    assert_eq!(
        err,
        anchor_lang::error!(crate::errors::IFTError::CosmosEmptyCounterpartyDenom)
    );
}

#[test]
fn test_chain_options_cosmos_validate_empty_type_url() {
    let hrp = bech32::Hrp::parse("cosmos").expect("valid HRP");
    let mut addr = String::new();
    bech32::encode_to_fmt::<bech32::Bech32, _>(&mut addr, hrp, &[0u8; 20])
        .expect("valid bech32 encoding");

    let options = ChainOptions::Cosmos {
        denom: "uatom".to_string(),
        type_url: String::new(),
        ica_address: addr,
    };
    let err = options.validate().unwrap_err();
    assert_eq!(
        err,
        anchor_lang::error!(crate::errors::IFTError::CosmosEmptyTypeUrl)
    );
}

#[test]
fn test_chain_options_cosmos_validate_empty_ica_address() {
    let options = ChainOptions::Cosmos {
        denom: "uatom".to_string(),
        type_url: "/cosmos.ift.v1.MsgIFTMint".to_string(),
        ica_address: String::new(),
    };
    let err = options.validate().unwrap_err();
    assert_eq!(
        err,
        anchor_lang::error!(crate::errors::IFTError::CosmosEmptyIcaAddress)
    );
}

#[test]
fn test_chain_options_cosmos_validate_denom_too_long() {
    let hrp = bech32::Hrp::parse("cosmos").expect("valid HRP");
    let mut addr = String::new();
    bech32::encode_to_fmt::<bech32::Bech32, _>(&mut addr, hrp, &[0u8; 20])
        .expect("valid bech32 encoding");

    let options = ChainOptions::Cosmos {
        denom: "x".repeat(crate::constants::MAX_COUNTERPARTY_ADDRESS_LENGTH + 1),
        type_url: "/cosmos.ift.v1.MsgIFTMint".to_string(),
        ica_address: addr,
    };
    let err = options.validate().unwrap_err();
    assert_eq!(
        err,
        anchor_lang::error!(crate::errors::IFTError::InvalidCounterpartyDenomLength)
    );
}

#[test]
fn test_chain_options_cosmos_validate_type_url_too_long() {
    let hrp = bech32::Hrp::parse("cosmos").expect("valid HRP");
    let mut addr = String::new();
    bech32::encode_to_fmt::<bech32::Bech32, _>(&mut addr, hrp, &[0u8; 20])
        .expect("valid bech32 encoding");

    let options = ChainOptions::Cosmos {
        denom: "uatom".to_string(),
        type_url: "x".repeat(crate::constants::MAX_COUNTERPARTY_ADDRESS_LENGTH + 1),
        ica_address: addr,
    };
    let err = options.validate().unwrap_err();
    assert_eq!(
        err,
        anchor_lang::error!(crate::errors::IFTError::InvalidCosmosTypeUrlLength)
    );
}

#[test]
fn test_chain_options_cosmos_validate_ica_address_too_long() {
    let hrp = bech32::Hrp::parse("cosmos").expect("valid HRP");
    let mut addr = String::new();
    // 100 data bytes → ~173 char bech32 string → exceeds MAX_COUNTERPARTY_ADDRESS_LENGTH (128)
    bech32::encode_to_fmt::<bech32::Bech32, _>(&mut addr, hrp, &[0u8; 100])
        .expect("valid bech32 encoding");
    assert!(addr.len() > crate::constants::MAX_COUNTERPARTY_ADDRESS_LENGTH);

    let options = ChainOptions::Cosmos {
        denom: "uatom".to_string(),
        type_url: "/cosmos.ift.v1.MsgIFTMint".to_string(),
        ica_address: addr,
    };
    let err = options.validate().unwrap_err();
    assert_eq!(
        err,
        anchor_lang::error!(crate::errors::IFTError::InvalidCosmosIcaAddressLength)
    );
}
