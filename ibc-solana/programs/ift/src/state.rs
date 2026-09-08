// SPDX-License-Identifier: Apache-2.0

use anchor_lang::prelude::*;

use crate::{constants::*, errors::IFTError};

/// Account schema version for upgrades
#[derive(
    AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace, Debug, Default,
)]
pub enum AccountVersion {
    #[default]
    V1,
}

/// Chain-specific options for counterparty chain configuration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, InitSpace)]
pub enum ChainOptions {
    /// EVM chain - encode as ABI call to iftMint(address, uint256)
    Evm,
    /// Cosmos chain - encode as protojson `MsgIFTMint`
    Cosmos {
        /// Token denom on counterparty chain (Cosmos SDK max: 128 chars)
        #[max_len(128)]
        denom: String,
        /// Protobuf type URL for `MsgIFTMint` (e.g., "/cosmos.ift.v1.MsgIFTMint")
        #[max_len(128)]
        type_url: String,
        /// ICS27-GMP interchain account address (the signer for `MsgIFTMint`)
        #[max_len(128)]
        ica_address: String,
    },
    /// Solana chain - encode as protobuf `GmpSolanaPayload` dispatched to the
    /// counterparty IFT program's `ift_mint` instruction.
    ///
    /// The source chain must commit to the exact destination account list in
    /// the GMP payload, so all information needed to derive the destination
    /// PDAs (`app_state`, `app_mint_state`, `ift_bridge`, `mint_authority`,
    /// `gmp_account`, receiver ATA) is stored here.
    Solana {
        /// Counterparty IFT program ID (target of the GMP CPI)
        ift_program_id: Pubkey,
        /// Counterparty mint that will receive the newly-minted tokens
        counterparty_mint: Pubkey,
        /// Counterparty's local IBC client identifier (used to derive the
        /// counterparty `IFTBridge` PDA and the dispatch `GMPAccount` PDA)
        #[max_len(MAX_CLIENT_ID_LENGTH)]
        counterparty_client_id: String,
    },
}

/// Token type and configuration for `create_and_initialize_spl_token`
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub enum CreateTokenParams {
    /// Legacy SPL Token mint (no extensions)
    SplToken { decimals: u8 },
    /// Token 2022 mint with `MetadataPointer` and on-chain metadata
    Token2022 {
        decimals: u8,
        name: String,
        symbol: String,
        uri: String,
    },
}

impl ChainOptions {
    /// Validate Chain Options params
    pub fn validate(&self) -> Result<()> {
        match self {
            Self::Evm => {}
            Self::Cosmos {
                denom,
                type_url,
                ica_address,
            } => {
                require!(!denom.is_empty(), IFTError::CosmosEmptyCounterpartyDenom);
                require!(!type_url.is_empty(), IFTError::CosmosEmptyTypeUrl);
                require!(!ica_address.is_empty(), IFTError::CosmosEmptyIcaAddress);
                require!(
                    bech32::primitives::decode::CheckedHrpstring::new::<bech32::Bech32>(
                        ica_address
                    )
                    .is_ok(),
                    IFTError::InvalidCosmosIcaAddress
                );
                require!(
                    denom.len() <= MAX_COUNTERPARTY_ADDRESS_LENGTH,
                    IFTError::InvalidCounterpartyDenomLength
                );
                require!(
                    type_url.len() <= MAX_COUNTERPARTY_ADDRESS_LENGTH,
                    IFTError::InvalidCosmosTypeUrlLength
                );
                require!(
                    ica_address.len() <= MAX_COUNTERPARTY_ADDRESS_LENGTH,
                    IFTError::InvalidCosmosIcaAddressLength
                );
            }
            Self::Solana {
                counterparty_client_id,
                ..
            } => {
                require!(!counterparty_client_id.is_empty(), IFTError::EmptyClientId);
                require!(
                    counterparty_client_id.len() <= MAX_CLIENT_ID_LENGTH,
                    IFTError::InvalidClientIdLength
                );
            }
        }

        Ok(())
    }
}

/// Global IFT application state (singleton)
/// PDA Seeds: [`IFT_APP_STATE_SEED`]
#[account]
#[derive(InitSpace)]
pub struct IFTAppState {
    pub version: AccountVersion,
    pub bump: u8,

    /// Admin authority for IFT
    pub admin: Pubkey,

    /// Whether IFT is paused (blocks mint and transfer, not refunds)
    pub paused: bool,

    /// Pending admin for two-step transfer
    pub pending_admin: Option<Pubkey>,

    pub _reserved: [u8; 128],
}

impl IFTAppState {
    /// Get PDA seeds for global app state
    pub fn seeds() -> Vec<Vec<u8>> {
        vec![IFT_APP_STATE_SEED.to_vec()]
    }
}

/// Per-mint IFT application state
/// PDA Seeds: [`IFT_APP_MINT_STATE_SEED`, `mint.as_ref()`]
#[account]
#[derive(InitSpace)]
pub struct IFTAppMintState {
    pub version: AccountVersion,
    pub bump: u8,

    /// SPL Token mint address (this IFT controls this mint)
    pub mint: Pubkey,

    /// Mint authority PDA bump for signing mint/refund operations.
    /// Stored to use `create_program_address` (~1.5k CUs) instead of
    /// `find_program_address` (~10k CUs) on each mint/refund.
    pub mint_authority_bump: u8,

    /// Daily mint rate limit (0 = no limit)
    pub daily_mint_limit: u64,
    /// Current rate limit day (`unix_timestamp` / `SECONDS_PER_DAY`)
    pub rate_limit_day: u64,
    /// Net mint usage for the current day
    pub rate_limit_daily_usage: u64,

    pub _reserved: [u8; 128],
}

impl IFTAppMintState {
    /// Get PDA seeds for per-mint app state
    pub fn seeds(mint: &Pubkey) -> Vec<Vec<u8>> {
        vec![IFT_APP_MINT_STATE_SEED.to_vec(), mint.as_ref().to_vec()]
    }
}

/// IFT Bridge configuration for a counterparty chain
#[account]
#[derive(InitSpace)]
pub struct IFTBridge {
    pub version: AccountVersion,
    pub bump: u8,

    /// Mint this bridge is associated with
    pub mint: Pubkey,

    /// IBC client identifier for this bridge
    #[max_len(MAX_CLIENT_ID_LENGTH)]
    pub client_id: String,

    /// IFT contract address on counterparty chain (EVM address or Cosmos bech32)
    #[max_len(MAX_COUNTERPARTY_ADDRESS_LENGTH)]
    pub counterparty_ift_address: String,

    /// Chain-specific options for constructing mint calls
    pub chain_options: ChainOptions,

    /// Whether bridge is active
    pub active: bool,

    // TODO: USE SAME 256 FOR EVERY RESERVED
    pub _reserved: [u8; 64],
}

impl IFTBridge {
    pub fn seeds(mint: &Pubkey, client_id: &str) -> Vec<Vec<u8>> {
        vec![
            IFT_BRIDGE_SEED.to_vec(),
            mint.as_ref().to_vec(),
            client_id.as_bytes().to_vec(),
        ]
    }
}

/// Pending transfer tracking for ack/timeout handling
#[account]
#[derive(InitSpace)]
pub struct PendingTransfer {
    pub version: AccountVersion,
    pub bump: u8,

    /// Mint this transfer is for
    pub mint: Pubkey,

    /// Client ID the transfer was sent to
    #[max_len(MAX_CLIENT_ID_LENGTH)]
    pub client_id: String,

    /// Packet sequence number
    pub sequence: u64,

    /// Original sender (for refunds)
    pub sender: Pubkey,

    /// Amount transferred (for refunds)
    pub amount: u64,

    /// Transfer initiation timestamp
    pub timestamp: i64,

    pub _reserved: [u8; 32],
}

impl PendingTransfer {
    pub fn seeds(mint: &Pubkey, client_id: &str, sequence: u64) -> Vec<Vec<u8>> {
        vec![
            PENDING_TRANSFER_SEED.to_vec(),
            mint.as_ref().to_vec(),
            client_id.as_bytes().to_vec(),
            sequence.to_le_bytes().to_vec(),
        ]
    }
}

/// Message for registering an IFT bridge
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct RegisterIFTBridgeMsg {
    /// IBC client identifier
    pub client_id: String,
    /// Counterparty IFT contract address
    pub counterparty_ift_address: String,
    /// Chain-specific options
    pub chain_options: ChainOptions,
}

/// Message for initiating an IFT transfer
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct IFTTransferMsg {
    /// IBC client identifier for destination
    pub client_id: String,
    /// Receiver address on destination chain
    pub receiver: String,
    /// Amount to transfer
    pub amount: u64,
    /// Timeout timestamp (0 for default 15 minutes)
    pub timeout_timestamp: u64,
    /// Caller-chosen packet sequence number
    pub sequence: u64,
}

/// Message for minting IFT tokens (called by GMP)
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct IFTMintMsg {
    /// Receiver pubkey
    pub receiver: Pubkey,
    /// Amount to mint
    pub amount: u64,
}

/// Message for setting the daily mint rate limit
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct SetMintRateLimitMsg {
    /// Daily mint limit (0 = no limit)
    pub daily_mint_limit: u64,
}

/// Message for pausing/unpausing an IFT token
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct SetPausedMsg {
    /// Whether to pause (true) or unpause (false) the token
    pub paused: bool,
}

/// Message for admin mint
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug)]
pub struct AdminMintMsg {
    /// Receiver pubkey
    pub receiver: Pubkey,
    /// Amount to mint
    pub amount: u64,
}

#[cfg(test)]
mod tests;
