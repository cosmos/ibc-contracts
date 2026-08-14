// SPDX-License-Identifier: Apache-2.0

use alloy_sol_types::SolCall;
use anchor_lang::prelude::*;
use anchor_lang::InstructionData;
use anchor_spl::token_interface::{self, Burn, Mint, TokenAccount, TokenInterface};
use serde::Serialize;
use solana_ibc_proto::ProstMessage;

use crate::constants::*;
use crate::errors::IFTError;
use crate::events::IFTTransferInitiated;
use crate::gmp_cpi::{SendGmpCallAccounts, SendGmpCallMsg};
use crate::state::{
    AccountVersion, ChainOptions, IFTAppMintState, IFTAppState, IFTBridge, IFTMintMsg,
    IFTTransferMsg, PendingTransfer,
};

#[derive(Accounts)]
#[instruction(msg: IFTTransferMsg)]
pub struct IFTTransfer<'info> {
    /// Global IFT app state (read-only, for GMP program reference and pause check)
    #[account(
        seeds = [IFT_APP_STATE_SEED],
        bump = app_state.bump,
        constraint = !app_state.paused @ IFTError::AppPaused,
    )]
    pub app_state: Account<'info, IFTAppState>,

    /// Per-mint IFT app state (read-only, for mint and bridge references)
    #[account(
        seeds = [IFT_APP_MINT_STATE_SEED, app_mint_state.mint.as_ref()],
        bump = app_mint_state.bump,
    )]
    pub app_mint_state: Account<'info, IFTAppMintState>,

    /// IFT bridge for the destination.
    /// Boxed to reduce stack frame size and avoid BPF stack overflow.
    #[account(
        seeds = [IFT_BRIDGE_SEED, app_mint_state.mint.as_ref(), msg.client_id.as_bytes()],
        bump = ift_bridge.bump,
        constraint = !msg.client_id.is_empty() @ IFTError::EmptyClientId,
        constraint = msg.client_id.len() <= MAX_CLIENT_ID_LENGTH @ IFTError::InvalidClientIdLength,
        constraint = ift_bridge.active @ IFTError::BridgeNotActive,
    )]
    pub ift_bridge: Box<Account<'info, IFTBridge>>,

    /// SPL Token mint
    #[account(
        mut,
        address = app_mint_state.mint
    )]
    pub mint: InterfaceAccount<'info, Mint>,

    /// Sender's token account
    #[account(
        mut,
        // TODO: add its own error
        constraint = sender_token_account.mint == mint.key() @ IFTError::TokenAccountOwnerMismatch,
        constraint = sender_token_account.owner == sender.key() @ IFTError::TokenAccountOwnerMismatch
    )]
    pub sender_token_account: InterfaceAccount<'info, TokenAccount>,

    /// Sender who owns the tokens
    pub sender: Signer<'info>,

    /// Pays for pending transfer PDA creation and GMP CPI fees
    #[account(mut)]
    pub payer: Signer<'info>,

    /// Required for burning tokens from sender's account
    pub token_program: Interface<'info, TokenInterface>,
    /// Required for pending transfer PDA creation
    pub system_program: Program<'info, System>,

    pub gmp_program: Program<'info, ics27_gmp::program::Ics27Gmp>,

    /// GMP app state PDA
    /// CHECK: Validated by GMP program via CPI
    #[account(
        mut,
        seeds = [solana_ibc_types::GMPAppState::SEED],
        bump,
        seeds::program = gmp_program.key()
    )]
    pub gmp_app_state: AccountInfo<'info>,

    /// Router program
    pub router_program: Program<'info, ics26_router::program::Ics26Router>,

    /// Router state account
    /// CHECK: Router program validates this
    #[account()]
    pub router_state: AccountInfo<'info>,

    /// Packet commitment account; initialized by the router via GMP CPI.
    /// CHECK: PDA seeds verified against the router program.
    #[account(
        mut,
        seeds = [
            solana_ibc_types::Commitment::PACKET_COMMITMENT_SEED,
            msg.client_id.as_bytes(),
            &msg.sequence.to_le_bytes()
        ],
        bump,
        seeds::program = router_program
    )]
    pub packet_commitment: AccountInfo<'info>,

    /// GMP's IBC app registration account — required by the router for authorization.
    /// CHECK: Router program validates this
    #[account()]
    pub gmp_ibc_app: AccountInfo<'info>,
    /// IBC client account
    /// CHECK: Router program validates this
    #[account()]
    pub ibc_client: AccountInfo<'info>,

    /// CHECK: Light client program, forwarded through GMP to router
    pub light_client_program: AccountInfo<'info>,

    /// CHECK: Client state for light client status check
    pub light_client_state: AccountInfo<'info>,

    /// Instructions sysvar for CPI caller detection by GMP
    /// CHECK: Address constraint verifies this is the instructions sysvar
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instruction_sysvar: AccountInfo<'info>,

    /// CHECK: Consensus state account, forwarded through GMP to router for expiry check
    pub consensus_state: AccountInfo<'info>,

    #[account(
        init,
        payer = payer,
        space = 8 + PendingTransfer::INIT_SPACE,
        seeds = [
            PENDING_TRANSFER_SEED,
            app_mint_state.mint.as_ref(),
            msg.client_id.as_bytes(),
            &msg.sequence.to_le_bytes(),
        ],
        bump,
    )]
    pub pending_transfer: Account<'info, PendingTransfer>,
}

pub fn ift_transfer(ctx: Context<IFTTransfer>, msg: IFTTransferMsg) -> Result<u64> {
    let clock = Clock::get()?;

    require!(msg.amount > 0, IFTError::ZeroAmount);
    require!(!msg.receiver.is_empty(), IFTError::EmptyReceiver);
    require!(
        msg.receiver.len() <= MAX_RECEIVER_LENGTH,
        IFTError::InvalidReceiver
    );

    let current_time =
        u64::try_from(clock.unix_timestamp).map_err(|_| IFTError::ArithmeticOverflow)?;
    let timeout = if msg.timeout_timestamp == 0 {
        current_time + DEFAULT_TIMEOUT_DURATION
    } else {
        require!(
            msg.timeout_timestamp > current_time + MIN_TIMEOUT_DURATION,
            IFTError::TimeoutInPast
        );
        require!(
            msg.timeout_timestamp <= current_time + MAX_TIMEOUT_DURATION,
            IFTError::TimeoutTooLong
        );
        msg.timeout_timestamp
    };

    let burn_accounts = Burn {
        mint: ctx.accounts.mint.to_account_info(),
        from: ctx.accounts.sender_token_account.to_account_info(),
        authority: ctx.accounts.sender.to_account_info(),
    };
    let burn_ctx = CpiContext::new(ctx.accounts.token_program.to_account_info(), burn_accounts);
    token_interface::burn(burn_ctx, msg.amount)?;
    ctx.accounts.mint.reload()?;
    ctx.accounts.sender_token_account.reload()?;

    let mint_call_payload = construct_mint_call(
        &ctx.accounts.ift_bridge.chain_options,
        &msg.receiver,
        msg.amount,
    )?;

    let gmp_accounts = SendGmpCallAccounts {
        gmp_program: ctx.accounts.gmp_program.to_account_info(),
        gmp_app_state: ctx.accounts.gmp_app_state.clone(),
        payer: ctx.accounts.payer.to_account_info(),
        router_program: ctx.accounts.router_program.to_account_info(),
        router_state: ctx.accounts.router_state.clone(),
        packet_commitment: ctx.accounts.packet_commitment.clone(),
        ibc_app: ctx.accounts.gmp_ibc_app.clone(),
        client: ctx.accounts.ibc_client.clone(),
        light_client_program: ctx.accounts.light_client_program.clone(),
        client_state: ctx.accounts.light_client_state.clone(),
        instruction_sysvar: ctx.accounts.instruction_sysvar.clone(),
        consensus_state: ctx.accounts.consensus_state.clone(),
        system_program: ctx.accounts.system_program.to_account_info(),
    };

    let encoding = match &ctx.accounts.ift_bridge.chain_options {
        ChainOptions::Evm => ics27_gmp::constants::ICS27_ENCODING_ABI,
        ChainOptions::Cosmos { .. } | ChainOptions::Solana { .. } => {
            ics27_gmp::constants::ICS27_ENCODING_PROTOBUF
        }
    };

    let gmp_msg = SendGmpCallMsg {
        source_client: msg.client_id.clone(),
        timeout_timestamp: timeout,
        receiver: ctx.accounts.ift_bridge.counterparty_ift_address.clone(),
        payload: mint_call_payload,
        encoding: encoding.to_string(),
        sequence: msg.sequence,
    };

    let sequence = crate::gmp_cpi::send_gmp_call(gmp_accounts, gmp_msg)?;

    let pending = &mut ctx.accounts.pending_transfer;
    pending.version = AccountVersion::V1;
    pending.bump = ctx.bumps.pending_transfer;
    pending.mint = ctx.accounts.app_mint_state.mint;
    pending.client_id.clone_from(&msg.client_id);
    pending.sequence = sequence;
    pending.sender = ctx.accounts.sender.key();
    pending.amount = msg.amount;
    pending.timestamp = clock.unix_timestamp;
    pending._reserved = [0; 32];

    emit!(IFTTransferInitiated {
        mint: ctx.accounts.app_mint_state.mint,
        client_id: msg.client_id.clone(),
        sequence,
        sender: ctx.accounts.sender.key(),
        receiver: msg.receiver,
        amount: msg.amount,
        timeout_timestamp: timeout,
    });

    Ok(sequence)
}

/// Construct chain-specific mint call payload for the counterparty.
fn construct_mint_call(
    chain_options: &ChainOptions,
    receiver: &str,
    amount: u64,
) -> Result<Vec<u8>> {
    match chain_options {
        ChainOptions::Evm => construct_evm_mint_call(receiver, amount),
        ChainOptions::Cosmos {
            denom,
            type_url,
            ica_address,
        } => Ok(construct_cosmos_mint_call(
            type_url,
            ica_address,
            denom,
            receiver,
            amount,
        )),
        ChainOptions::Solana {
            ift_program_id,
            counterparty_mint,
            counterparty_client_id,
        } => construct_solana_mint_call(
            ift_program_id,
            counterparty_mint,
            counterparty_client_id,
            receiver,
            amount,
        ),
    }
}

/// Construct ABI-encoded call to `iftMint(address, uint256)` for EVM chains.
fn construct_evm_mint_call(receiver: &str, amount: u64) -> Result<Vec<u8>> {
    use alloy_sol_types::private::{Address, U256};

    let receiver: Address = receiver
        .parse()
        .map_err(|_| error!(IFTError::InvalidReceiver))?;

    Ok(IFT::iftMintCall {
        receiver,
        amount: U256::from(amount),
    }
    .abi_encode())
}

// Using ABI JSON because sol! macro can't resolve Solidity imports.
alloy_sol_types::sol!(IFT, "../../../ibc-solidity/abi/IFTOwnable.json");

/// Protojson representation of `MsgIFTMint` for Cosmos chains
#[derive(Serialize)]
struct MsgIFTMint<'a> {
    #[serde(rename = "@type")]
    type_url: &'a str,
    signer: &'a str,
    denom: &'a str,
    receiver: &'a str,
    amount: String,
}

/// Protojson representation of `CosmosTx` wrapper
#[derive(Serialize)]
struct CosmosTx<'a> {
    messages: Vec<MsgIFTMint<'a>>,
}

/// Construct protojson-encoded `CosmosTx` with `MsgIFTMint` for Cosmos chains
fn construct_cosmos_mint_call(
    type_url: &str,
    signer: &str,
    denom: &str,
    receiver: &str,
    amount: u64,
) -> Vec<u8> {
    // The signer is the ICS27-GMP interchain account on the Cosmos chain that controls the IFT mint
    let tx = CosmosTx {
        messages: vec![MsgIFTMint {
            type_url,
            signer,
            denom,
            receiver,
            amount: amount.to_string(),
        }],
    };
    // TODO: use proto
    serde_json::to_vec(&tx).expect("cannot fail for this simple struct")
}

/// Construct a protobuf-encoded `GmpSolanaPayload` that dispatches the
/// counterparty IFT program's `ift_mint` instruction.
///
/// Delegates PDA derivation and account-list construction to
/// [`IftMintAccounts`](crate::helpers::IftMintAccounts), which centralizes
/// the logic shared with the integration test helpers.
///
/// Only the classic SPL Token program is supported for the receiver ATA.
/// Token 2022 can be added later by threading a token-program field through
/// `ChainOptions::Solana`.
fn construct_solana_mint_call(
    ift_program_id: &Pubkey,
    counterparty_mint: &Pubkey,
    counterparty_client_id: &str,
    receiver: &str,
    amount: u64,
) -> Result<Vec<u8>> {
    let receiver_pubkey: Pubkey = receiver
        .parse()
        .map_err(|_| error!(IFTError::InvalidReceiver))?;

    let accounts = crate::helpers::IftMintAccounts::derive(
        ift_program_id,
        counterparty_mint,
        counterparty_client_id,
        &crate::ID.to_string(),
        &receiver_pubkey,
    )?;

    let ix_data = crate::instruction::IftMint {
        msg: IFTMintMsg {
            receiver: receiver_pubkey,
            amount,
        },
    }
    .data();

    Ok(accounts.to_payload(ix_data).encode_to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::IFTError;
    use crate::state::IFTTransferMsg;
    use crate::test_utils::*;
    use anchor_lang::InstructionData;
    use rstest::rstest;
    use solana_sdk::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
    };

    const TEST_CLIENT_ID: &str = "07-tendermint-0";
    const TEST_COUNTERPARTY_ADDRESS: &str = "0x1234567890abcdef1234567890abcdef12345678";
    const VALID_RECEIVER: &str = "0xabcdef1234567890abcdef1234567890abcdef12";

    #[rstest]
    #[case::invalid_hex("0xnothex")]
    #[case::short_address("0xabcd")]
    #[case::empty_address("")]
    #[case::only_prefix("0x")]
    fn test_construct_evm_mint_call_invalid_receiver(#[case] invalid_receiver: &str) {
        assert!(construct_evm_mint_call(invalid_receiver, 100).is_err());
    }

    #[test]
    fn test_construct_cosmos_mint_call() {
        let payload = construct_cosmos_mint_call(
            "/cosmos.ift.v1.MsgIFTMint",
            "cosmos1icaaddress",
            "uatom",
            "cosmos1abc123",
            1_000_000,
        );
        let json_str = String::from_utf8(payload).unwrap();

        assert!(json_str.contains("\"messages\":["));
        assert!(json_str.contains("\"@type\":\"/cosmos.ift.v1.MsgIFTMint\""));
        assert!(json_str.contains("\"signer\":\"cosmos1icaaddress\""));
        assert!(json_str.contains("\"denom\":\"uatom\""));
        assert!(json_str.contains("\"receiver\":\"cosmos1abc123\""));
        assert!(json_str.contains("\"amount\":\"1000000\""));
    }

    #[test]
    fn test_construct_cosmos_mint_call_with_ibc_denom() {
        let payload = construct_cosmos_mint_call(
            "/wfchain.ift.MsgIFTMint",
            "wf1icaaddress",
            "ibc/ABC123",
            "cosmos1xyz",
            42,
        );
        let json_str = String::from_utf8(payload).unwrap();
        assert!(json_str.contains("\"denom\":\"ibc/ABC123\""));
        assert!(json_str.contains("\"@type\":\"/wfchain.ift.MsgIFTMint\""));
        assert!(json_str.contains("\"signer\":\"wf1icaaddress\""));
    }

    #[derive(Clone)]
    struct MintCallTestCase {
        chain_options: ChainOptions,
        receiver: &'static str,
        expected_len: Option<usize>,
        expected_content: Vec<&'static str>,
    }

    fn evm_test_case() -> MintCallTestCase {
        MintCallTestCase {
            chain_options: ChainOptions::Evm,
            receiver: "0x1234567890abcdef1234567890abcdef12345678",
            expected_len: Some(68),
            expected_content: vec![],
        }
    }

    fn cosmos_test_case() -> MintCallTestCase {
        MintCallTestCase {
            chain_options: ChainOptions::Cosmos {
                denom: "uatom".to_string(),
                type_url: "/cosmos.ift.v1.MsgIFTMint".to_string(),
                ica_address: "cosmos1icaaddress".to_string(),
            },
            receiver: "cosmos1receiver",
            expected_len: None,
            expected_content: vec!["/cosmos.ift.v1.MsgIFTMint", "uatom", "cosmos1icaaddress"],
        }
    }

    /// Receiver must be base58-encoded — use a constant derived from a real keypair.
    const SOLANA_RECEIVER: &str = "11111111111111111111111111111112";
    const SOLANA_TEST_CLIENT_ID: &str = "07-tendermint-0";

    fn solana_test_case() -> MintCallTestCase {
        let ift_program_id = Pubkey::new_unique();
        let mint = Pubkey::new_unique();
        MintCallTestCase {
            chain_options: ChainOptions::Solana {
                ift_program_id,
                counterparty_mint: mint,
                counterparty_client_id: SOLANA_TEST_CLIENT_ID.to_string(),
            },
            receiver: SOLANA_RECEIVER,
            expected_len: None,
            expected_content: vec![],
        }
    }

    #[rstest]
    #[case::evm(evm_test_case())]
    #[case::cosmos(cosmos_test_case())]
    #[case::solana(solana_test_case())]
    fn test_construct_mint_call(#[case] test_case: MintCallTestCase) {
        let result = construct_mint_call(&test_case.chain_options, test_case.receiver, 100);
        assert!(result.is_ok());
        let payload = result.unwrap();

        if let Some(expected_len) = test_case.expected_len {
            assert_eq!(payload.len(), expected_len);
        }

        if !test_case.expected_content.is_empty() {
            let content = String::from_utf8(payload).unwrap();
            for expected in test_case.expected_content {
                assert!(
                    content.contains(expected),
                    "Expected to contain: {expected}"
                );
            }
        }
    }

    #[test]
    fn test_construct_solana_mint_call_payload_structure() {
        use solana_ibc_proto::RawGmpSolanaPayload;

        let ift_program_id = Pubkey::new_unique();
        let mint = Pubkey::new_unique();
        let receiver: Pubkey = SOLANA_RECEIVER.parse().unwrap();

        let bytes = construct_solana_mint_call(
            &ift_program_id,
            &mint,
            SOLANA_TEST_CLIENT_ID,
            SOLANA_RECEIVER,
            1_000,
        )
        .unwrap();

        let payload =
            RawGmpSolanaPayload::decode(bytes.as_slice()).expect("valid protobuf payload");

        // 12 accounts matching IFTMint<'info>
        assert_eq!(payload.accounts.len(), 12);
        assert_eq!(
            payload.prefund_lamports,
            crate::constants::SOLANA_MINT_PAYLOAD_PREFUND_LAMPORTS
        );

        // Verify each account pubkey is 32 bytes
        for meta in &payload.accounts {
            assert_eq!(meta.pubkey.len(), 32);
        }

        // Account 6 (index 5) is the receiver ATA
        let receiver_ata =
            anchor_spl::associated_token::get_associated_token_address(&receiver, &mint);
        assert_eq!(payload.accounts[5].pubkey, receiver_ata.to_bytes());
        assert!(payload.accounts[5].is_writable);

        // Account 7 (index 6) is the receiver owner (readonly)
        assert_eq!(payload.accounts[6].pubkey, receiver.to_bytes());
        assert!(!payload.accounts[6].is_signer);
        assert!(!payload.accounts[6].is_writable);

        // Accounts 8 and 9 (indices 7, 8) are the GMP account (both signer)
        assert_eq!(payload.accounts[7].pubkey, payload.accounts[8].pubkey);
        assert!(payload.accounts[7].is_signer);
        assert!(payload.accounts[8].is_signer);
        assert!(payload.accounts[8].is_writable);
    }

    #[test]
    fn test_construct_solana_mint_call_invalid_receiver() {
        let ift_program_id = Pubkey::new_unique();
        let mint = Pubkey::new_unique();

        let result = construct_solana_mint_call(
            &ift_program_id,
            &mint,
            SOLANA_TEST_CLIENT_ID,
            "not-a-valid-pubkey",
            100,
        );
        assert!(result.is_err());
    }

    #[test]
    fn test_ift_mint_accounts_derive() {
        use crate::helpers::IftMintAccounts;

        let ift_program_id = Pubkey::new_unique();
        let mint = Pubkey::new_unique();
        let receiver: Pubkey = SOLANA_RECEIVER.parse().unwrap();
        let source_address = crate::ID.to_string();

        let accounts = IftMintAccounts::derive(
            &ift_program_id,
            &mint,
            SOLANA_TEST_CLIENT_ID,
            &source_address,
            &receiver,
        )
        .unwrap();

        // Verify PDAs match manual derivation
        let (expected_app_state, _) =
            Pubkey::find_program_address(&[IFT_APP_STATE_SEED], &ift_program_id);
        assert_eq!(accounts.app_state, expected_app_state);

        let (expected_app_mint_state, _) = Pubkey::find_program_address(
            &[IFT_APP_MINT_STATE_SEED, mint.as_ref()],
            &ift_program_id,
        );
        assert_eq!(accounts.app_mint_state, expected_app_mint_state);

        let (expected_bridge, _) = Pubkey::find_program_address(
            &[
                IFT_BRIDGE_SEED,
                mint.as_ref(),
                SOLANA_TEST_CLIENT_ID.as_bytes(),
            ],
            &ift_program_id,
        );
        assert_eq!(accounts.ift_bridge, expected_bridge);

        let (expected_mint_authority, _) =
            Pubkey::find_program_address(&[MINT_AUTHORITY_SEED, mint.as_ref()], &ift_program_id);
        assert_eq!(accounts.mint_authority, expected_mint_authority);

        let expected_ata =
            anchor_spl::associated_token::get_associated_token_address(&receiver, &mint);
        assert_eq!(accounts.receiver_ata, expected_ata);

        assert_eq!(accounts.mint, mint);
        assert_eq!(accounts.receiver, receiver);
    }

    fn create_token_account(
        mint: &Pubkey,
        owner: &Pubkey,
        amount: u64,
    ) -> solana_sdk::account::Account {
        let mut data = vec![0u8; 165];
        data[0..32].copy_from_slice(&mint.to_bytes());
        data[32..64].copy_from_slice(&owner.to_bytes());
        data[64..72].copy_from_slice(&amount.to_le_bytes());
        data[108] = 1;

        solana_sdk::account::Account {
            lamports: 1_000_000,
            data,
            owner: anchor_spl::token::ID,
            executable: false,
            rent_epoch: 0,
        }
    }

    fn create_mint_account(mint_authority: Option<&Pubkey>) -> solana_sdk::account::Account {
        let mut data = vec![0u8; 82];
        if let Some(authority) = mint_authority {
            data[0..4].copy_from_slice(&1u32.to_le_bytes());
            data[4..36].copy_from_slice(&authority.to_bytes());
        }
        data[44] = 9;
        data[45] = 1;

        solana_sdk::account::Account {
            lamports: 1_000_000,
            data,
            owner: anchor_spl::token::ID,
            executable: false,
            rent_epoch: 0,
        }
    }

    #[derive(Clone, Copy)]
    enum TransferErrorCase {
        InactiveBridge,
        ZeroAmount,
        EmptyReceiver,
        EmptyClientId,
        SenderNotSigner,
        WrongTokenAccountOwner,
        WrongTokenMint,
        TimeoutInPast,
        TimeoutTooLong,
        ReceiverTooLong,
        AppPaused,
        InvalidGmpProgram,
        TimeoutAtExactCurrent,
        TimeoutBelowMinDuration,
        TimeoutAtExactMinDuration,
        TimeoutOneOverMax,
    }

    #[allow(clippy::struct_excessive_bools)]
    struct TransferTestConfig {
        client_id: String,
        bridge_active: bool,
        amount: u64,
        receiver: String,
        sender_is_signer: bool,
        use_wrong_token_owner: bool,
        use_wrong_token_mint: bool,
        use_wrong_gmp_program: bool,
        timeout_timestamp: u64,
        token_paused: bool,
        expected_error: u32,
    }

    impl From<TransferErrorCase> for TransferTestConfig {
        fn from(case: TransferErrorCase) -> Self {
            let default = Self {
                client_id: TEST_CLIENT_ID.to_string(),
                bridge_active: true,
                amount: 1000,
                receiver: VALID_RECEIVER.to_string(),
                sender_is_signer: true,
                use_wrong_token_owner: false,
                use_wrong_token_mint: false,
                use_wrong_gmp_program: false,
                timeout_timestamp: 0,
                token_paused: false,
                expected_error: 0,
            };

            match case {
                TransferErrorCase::InactiveBridge => Self {
                    bridge_active: false,
                    expected_error: ANCHOR_ERROR_OFFSET + IFTError::BridgeNotActive as u32,
                    ..default
                },
                TransferErrorCase::ZeroAmount => Self {
                    amount: 0,
                    expected_error: ANCHOR_ERROR_OFFSET + IFTError::ZeroAmount as u32,
                    ..default
                },
                TransferErrorCase::EmptyReceiver => Self {
                    receiver: String::new(),
                    expected_error: ANCHOR_ERROR_OFFSET + IFTError::EmptyReceiver as u32,
                    ..default
                },
                TransferErrorCase::EmptyClientId => Self {
                    client_id: String::new(),
                    expected_error: ANCHOR_ERROR_OFFSET + IFTError::EmptyClientId as u32,
                    ..default
                },
                TransferErrorCase::SenderNotSigner => Self {
                    sender_is_signer: false,
                    expected_error: anchor_lang::error::ErrorCode::AccountNotSigner as u32,
                    ..default
                },
                TransferErrorCase::WrongTokenAccountOwner => Self {
                    use_wrong_token_owner: true,
                    expected_error: ANCHOR_ERROR_OFFSET
                        + IFTError::TokenAccountOwnerMismatch as u32,
                    ..default
                },
                TransferErrorCase::WrongTokenMint => Self {
                    use_wrong_token_mint: true,
                    expected_error: ANCHOR_ERROR_OFFSET
                        + IFTError::TokenAccountOwnerMismatch as u32,
                    ..default
                },
                TransferErrorCase::TimeoutInPast => Self {
                    timeout_timestamp: 1,
                    expected_error: ANCHOR_ERROR_OFFSET + IFTError::TimeoutInPast as u32,
                    ..default
                },
                TransferErrorCase::TimeoutTooLong => Self {
                    timeout_timestamp: 1_700_000_000 + crate::constants::MAX_TIMEOUT_DURATION * 2,
                    expected_error: ANCHOR_ERROR_OFFSET + IFTError::TimeoutTooLong as u32,
                    ..default
                },
                TransferErrorCase::ReceiverTooLong => Self {
                    receiver: "x".repeat(crate::constants::MAX_RECEIVER_LENGTH + 1),
                    expected_error: ANCHOR_ERROR_OFFSET + IFTError::InvalidReceiver as u32,
                    ..default
                },
                TransferErrorCase::AppPaused => Self {
                    token_paused: true,
                    expected_error: ANCHOR_ERROR_OFFSET + IFTError::AppPaused as u32,
                    ..default
                },
                TransferErrorCase::InvalidGmpProgram => Self {
                    use_wrong_gmp_program: true,
                    expected_error: anchor_lang::error::ErrorCode::InvalidProgramId as u32,
                    ..default
                },
                TransferErrorCase::TimeoutAtExactCurrent => Self {
                    timeout_timestamp: 1_700_000_000, // exactly == clock
                    expected_error: ANCHOR_ERROR_OFFSET + IFTError::TimeoutInPast as u32,
                    ..default
                },
                TransferErrorCase::TimeoutBelowMinDuration => Self {
                    timeout_timestamp: 1_700_000_000 + 5, // 5s in future, below 10s min
                    expected_error: ANCHOR_ERROR_OFFSET + IFTError::TimeoutInPast as u32,
                    ..default
                },
                TransferErrorCase::TimeoutAtExactMinDuration => Self {
                    timeout_timestamp: 1_700_000_000 + crate::constants::MIN_TIMEOUT_DURATION, // exactly == clock + min
                    expected_error: ANCHOR_ERROR_OFFSET + IFTError::TimeoutInPast as u32,
                    ..default
                },
                TransferErrorCase::TimeoutOneOverMax => Self {
                    timeout_timestamp: 1_700_000_000 + crate::constants::MAX_TIMEOUT_DURATION + 1,
                    expected_error: ANCHOR_ERROR_OFFSET + IFTError::TimeoutTooLong as u32,
                    ..default
                },
            }
        }
    }

    fn run_transfer_error_test(case: TransferErrorCase) {
        let config = TransferTestConfig::from(case);
        let mut mollusk = setup_mollusk();
        mollusk.sysvars.clock.unix_timestamp = 1_700_000_000;

        let mint = Pubkey::new_unique();
        let wrong_mint = Pubkey::new_unique();
        let sender = Pubkey::new_unique();
        let wrong_owner = Pubkey::new_unique();
        let payer = Pubkey::new_unique();
        let gmp_program = ics27_gmp::ID;

        let (app_state_pda, app_state_bump) = get_app_state_pda();
        let (app_mint_state_pda, app_mint_state_bump) = get_app_mint_state_pda(&mint);
        let (_, mint_authority_bump) = get_mint_authority_pda(&mint);
        let (ift_bridge_pda, ift_bridge_bump) = get_bridge_pda(&mint, &config.client_id);
        let (system_program, system_account) = create_system_program_account();

        let app_state_account = create_ift_app_state_account_with_options(
            app_state_bump,
            Pubkey::new_unique(),
            config.token_paused,
        );

        let app_mint_state_account =
            create_ift_app_mint_state_account(mint, app_mint_state_bump, mint_authority_bump);

        let ift_bridge_account = create_ift_bridge_account(
            mint,
            &config.client_id,
            TEST_COUNTERPARTY_ADDRESS,
            ChainOptions::Evm,
            ift_bridge_bump,
            config.bridge_active,
        );

        let mint_account = create_mint_account(None);
        let sender_token_pda = Pubkey::new_unique();

        let token_account_owner = if config.use_wrong_token_owner {
            wrong_owner
        } else {
            sender
        };
        let token_account_mint = if config.use_wrong_token_mint {
            wrong_mint
        } else {
            mint
        };
        let sender_token_account =
            create_token_account(&token_account_mint, &token_account_owner, 10000);

        let token_program_account = solana_sdk::account::Account {
            lamports: 1,
            data: vec![],
            owner: solana_sdk::native_loader::ID,
            executable: true,
            rent_epoch: 0,
        };

        let wrong_gmp_program = Pubkey::new_unique();
        let gmp_program_key = if config.use_wrong_gmp_program {
            wrong_gmp_program
        } else {
            gmp_program
        };

        let (gmp_app_state_pda, _) =
            Pubkey::find_program_address(&[solana_ibc_types::GMPAppState::SEED], &gmp_program_key);

        let msg = IFTTransferMsg {
            client_id: config.client_id,
            receiver: config.receiver,
            amount: config.amount,
            timeout_timestamp: config.timeout_timestamp,
            sequence: 1,
        };

        let router_state = Pubkey::new_unique();
        let (packet_commitment, _) = Pubkey::find_program_address(
            &[
                solana_ibc_types::Commitment::PACKET_COMMITMENT_SEED,
                msg.client_id.as_bytes(),
                &msg.sequence.to_le_bytes(),
            ],
            &ics26_router::ID,
        );
        let gmp_ibc_app = Pubkey::new_unique();
        let ibc_client = Pubkey::new_unique();
        let light_client_program = Pubkey::new_unique();
        let light_client_state = Pubkey::new_unique();
        let (instructions_sysvar, instructions_account) = create_instructions_sysvar_account();
        let consensus_state = Pubkey::new_unique();
        let (pending_transfer, _) = Pubkey::find_program_address(
            &[
                PENDING_TRANSFER_SEED,
                mint.as_ref(),
                msg.client_id.as_bytes(),
                &msg.sequence.to_le_bytes(),
            ],
            &crate::ID,
        );

        let pending_transfer_account = solana_sdk::account::Account {
            lamports: 0,
            data: vec![],
            owner: solana_sdk::system_program::ID,
            executable: false,
            rent_epoch: 0,
        };

        let instruction = Instruction {
            program_id: crate::ID,
            accounts: vec![
                AccountMeta::new_readonly(app_state_pda, false),
                AccountMeta::new_readonly(app_mint_state_pda, false),
                AccountMeta::new(ift_bridge_pda, false),
                AccountMeta::new(mint, false),
                AccountMeta::new(sender_token_pda, false),
                AccountMeta::new_readonly(sender, config.sender_is_signer),
                AccountMeta::new(payer, true),
                AccountMeta::new_readonly(anchor_spl::token::ID, false),
                AccountMeta::new_readonly(system_program, false),
                AccountMeta::new_readonly(gmp_program_key, false),
                AccountMeta::new(gmp_app_state_pda, false),
                AccountMeta::new_readonly(ics26_router::ID, false),
                AccountMeta::new_readonly(router_state, false),
                AccountMeta::new(packet_commitment, false),
                AccountMeta::new_readonly(gmp_ibc_app, false),
                AccountMeta::new_readonly(ibc_client, false),
                AccountMeta::new_readonly(light_client_program, false),
                AccountMeta::new_readonly(light_client_state, false),
                AccountMeta::new_readonly(instructions_sysvar, false),
                AccountMeta::new_readonly(consensus_state, false),
                AccountMeta::new(pending_transfer, false),
            ],
            data: crate::instruction::IftTransfer { msg }.data(),
        };

        let accounts = vec![
            (app_state_pda, app_state_account),
            (app_mint_state_pda, app_mint_state_account),
            (ift_bridge_pda, ift_bridge_account),
            (mint, mint_account),
            (sender_token_pda, sender_token_account),
            (sender, create_signer_account()),
            (payer, create_signer_account()),
            (anchor_spl::token::ID, token_program_account.clone()),
            (system_program, system_account),
            (gmp_program_key, create_gmp_program_account()),
            (gmp_app_state_pda, create_signer_account()),
            (ics26_router::ID, token_program_account),
            (router_state, create_signer_account()),
            (packet_commitment, create_uninitialized_pda()),
            (gmp_ibc_app, create_signer_account()),
            (ibc_client, create_signer_account()),
            (light_client_program, create_signer_account()),
            (light_client_state, create_signer_account()),
            (instructions_sysvar, instructions_account),
            (consensus_state, create_signer_account()),
            (pending_transfer, pending_transfer_account),
        ];

        let result = mollusk.process_instruction(&instruction, &accounts);
        assert_eq!(
            result.program_result,
            Err(solana_sdk::instruction::InstructionError::Custom(
                config.expected_error,
            ))
            .into(),
        );
    }

    #[rstest]
    #[case::inactive_bridge(TransferErrorCase::InactiveBridge)]
    #[case::zero_amount(TransferErrorCase::ZeroAmount)]
    #[case::empty_receiver(TransferErrorCase::EmptyReceiver)]
    #[case::empty_client_id(TransferErrorCase::EmptyClientId)]
    #[case::sender_not_signer(TransferErrorCase::SenderNotSigner)]
    #[case::wrong_token_account_owner(TransferErrorCase::WrongTokenAccountOwner)]
    #[case::wrong_token_mint(TransferErrorCase::WrongTokenMint)]
    #[case::timeout_in_past(TransferErrorCase::TimeoutInPast)]
    #[case::timeout_too_long(TransferErrorCase::TimeoutTooLong)]
    #[case::receiver_too_long(TransferErrorCase::ReceiverTooLong)]
    #[case::app_paused(TransferErrorCase::AppPaused)]
    #[case::invalid_gmp_program(TransferErrorCase::InvalidGmpProgram)]
    #[case::timeout_at_exact_current(TransferErrorCase::TimeoutAtExactCurrent)]
    #[case::timeout_below_min_duration(TransferErrorCase::TimeoutBelowMinDuration)]
    #[case::timeout_at_exact_min_duration(TransferErrorCase::TimeoutAtExactMinDuration)]
    #[case::timeout_one_over_max(TransferErrorCase::TimeoutOneOverMax)]
    fn test_ift_transfer_validation(#[case] case: TransferErrorCase) {
        run_transfer_error_test(case);
    }
}
