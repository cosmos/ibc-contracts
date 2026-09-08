// SPDX-License-Identifier: Apache-2.0

use anchor_lang::prelude::*;

use crate::state::{ChainOptions, CreateTokenParams};

/// Event emitted when IFT is initialized (global state created)
#[event]
pub struct IFTInitialized {
    /// Admin authority
    pub admin: Pubkey,
    /// Initialization timestamp
    pub timestamp: i64,
}

/// Event emitted when a new SPL token is created for IFT
#[event]
pub struct SplTokenCreated {
    /// SPL Token mint address
    pub mint: Pubkey,
    /// Token type and configuration
    pub params: CreateTokenParams,
    /// Initialization timestamp
    pub timestamp: i64,
}

/// Event emitted when an IFT bridge is registered
#[event]
pub struct IFTBridgeRegistered {
    /// SPL Token mint address
    pub mint: Pubkey,
    /// IBC client identifier
    pub client_id: String,
    /// Counterparty IFT contract address
    pub counterparty_ift_address: String,
    /// Chain-specific options
    pub chain_options: ChainOptions,
    /// Registration timestamp
    pub timestamp: i64,
}

/// Event emitted when an IFT bridge is removed
#[event]
pub struct IFTBridgeRemoved {
    /// SPL Token mint address
    pub mint: Pubkey,
    /// IBC client identifier
    pub client_id: String,
    /// Removal timestamp
    pub timestamp: i64,
}

/// Event emitted when an IFT transfer is initiated
#[event]
pub struct IFTTransferInitiated {
    /// SPL Token mint address
    pub mint: Pubkey,
    /// IBC client identifier
    pub client_id: String,
    /// Packet sequence number
    pub sequence: u64,
    /// Sender pubkey
    pub sender: Pubkey,
    /// Receiver address on destination chain
    pub receiver: String,
    /// Amount transferred
    pub amount: u64,
    /// Timeout timestamp
    pub timeout_timestamp: u64,
}

/// Event emitted when tokens are minted from a cross-chain transfer
#[event]
pub struct IFTMintReceived {
    /// SPL Token mint address
    pub mint: Pubkey,
    /// IBC client identifier
    pub client_id: String,
    /// Receiver pubkey
    pub receiver: Pubkey,
    /// Amount minted
    pub amount: u64,
    /// GMP account that authorized the mint
    pub gmp_account: Pubkey,
    /// Mint timestamp
    pub timestamp: i64,
}

/// Event emitted when an IFT transfer is completed (acknowledged)
#[event]
pub struct IFTTransferCompleted {
    /// SPL Token mint address
    pub mint: Pubkey,
    /// IBC client identifier
    pub client_id: String,
    /// Packet sequence number
    pub sequence: u64,
    /// Original sender
    pub sender: Pubkey,
    /// Amount that was transferred
    pub amount: u64,
    /// Completion timestamp
    pub timestamp: i64,
}

/// Event emitted when an IFT transfer is refunded (failed or timed out)
#[event]
pub struct IFTTransferRefunded {
    /// SPL Token mint address
    pub mint: Pubkey,
    /// IBC client identifier
    pub client_id: String,
    /// Packet sequence number
    pub sequence: u64,
    /// Original sender (refund recipient)
    pub sender: Pubkey,
    /// Amount refunded
    pub amount: u64,
    /// Reason for refund
    pub reason: RefundReason,
    /// Refund timestamp
    pub timestamp: i64,
}

/// Reason for transfer refund
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum RefundReason {
    /// Transfer timed out
    Timeout,
    /// Transfer failed on destination
    Failed,
}

/// Event emitted when mint authority is revoked and transferred back
#[event]
pub struct MintAuthorityRevoked {
    /// SPL Token mint address
    pub mint: Pubkey,
    /// New mint authority that received ownership
    pub new_authority: Pubkey,
    /// Revocation timestamp
    pub timestamp: i64,
}

/// Event emitted when the daily mint rate limit is updated
#[event]
pub struct MintRateLimitUpdated {
    /// SPL Token mint address
    pub mint: Pubkey,
    /// New daily mint limit (0 = no limit)
    pub daily_mint_limit: u64,
    /// Update timestamp
    pub timestamp: i64,
}

/// Event emitted when the paused state is updated
#[event]
pub struct TokenPausedUpdated {
    /// Whether IFT is now paused
    pub paused: bool,
    /// Update timestamp
    pub timestamp: i64,
}

/// Event emitted when IFT is initialized for an existing SPL token
#[event]
pub struct ExistingTokenInitialized {
    /// SPL Token mint address
    pub mint: Pubkey,
    /// Token decimals
    pub decimals: u8,
    /// Previous mint authority that transferred ownership
    pub previous_authority: Pubkey,
    /// Initialization timestamp
    pub timestamp: i64,
}

/// Event emitted when a new admin is proposed
#[event]
pub struct AdminProposed {
    /// Current admin who proposed the change
    pub current_admin: Pubkey,
    /// Proposed new admin
    pub proposed_admin: Pubkey,
    /// Proposal timestamp
    pub timestamp: i64,
}

/// Event emitted when a pending admin proposal is cancelled
#[event]
pub struct AdminProposalCancelled {
    /// Admin who cancelled the proposal
    pub admin: Pubkey,
    /// Proposed admin whose proposal was cancelled
    pub cancelled_admin: Pubkey,
    /// Cancellation timestamp
    pub timestamp: i64,
}

/// Event emitted when a proposed admin accepts the role
#[event]
pub struct AdminAccepted {
    /// Previous admin
    pub previous_admin: Pubkey,
    /// New admin who accepted
    pub new_admin: Pubkey,
    /// Acceptance timestamp
    pub timestamp: i64,
}

/// Event emitted when admin mints tokens
#[event]
pub struct AdminMintExecuted {
    /// SPL Token mint address
    pub mint: Pubkey,
    /// Receiver pubkey
    pub receiver: Pubkey,
    /// Amount minted
    pub amount: u64,
    /// Admin who executed the mint
    pub admin: Pubkey,
    /// Mint timestamp
    pub timestamp: i64,
}
