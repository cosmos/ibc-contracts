// SPDX-License-Identifier: Apache-2.0

//! Shared Solana v0 transaction building utilities.
//!
//! These free functions encapsulate the common logic for building, compiling and
//! serializing Solana versioned (v0) transactions. Both `eth-to-solana` and
//! `cosmos-to-solana` proof API modules delegate to these helpers.

use anyhow::Result;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    address_lookup_table::{
        instruction::{create_lookup_table, extend_lookup_table},
        state::AddressLookupTable,
        AddressLookupTableAccount,
    },
    commitment_config::CommitmentConfig,
    compute_budget::ComputeBudgetInstruction,
    instruction::Instruction,
    message::{v0, VersionedMessage},
    pubkey::Pubkey,
    transaction::VersionedTransaction,
};

/// Maximum compute units allowed per Solana transaction.
pub const MAX_COMPUTE_UNIT_LIMIT: u32 = 1_400_000;

/// Priority fee in micro-lamports per compute unit.
pub const DEFAULT_PRIORITY_FEE: u64 = 1000;

/// Heap frame size used for compute-heavy transactions (256 KiB).
const HEAP_FRAME_SIZE: u32 = 256 * 1024;

/// Build a serialized v0 transaction from instructions, automatically fetching
/// the recent blockhash and resolving the ALT if present.
///
/// # Errors
/// Returns an error if no instructions are provided, RPC calls fail or
/// message compilation fails.
pub fn create_tx_bytes(
    client: &RpcClient,
    fee_payer: Pubkey,
    alt_address: Option<Pubkey>,
    instructions: &[Instruction],
) -> Result<Vec<u8>> {
    if instructions.is_empty() {
        anyhow::bail!("No instructions to execute on Solana");
    }

    let recent_blockhash = get_recent_blockhash(client)?;

    let alt_addresses = match alt_address {
        Some(addr) => fetch_alt_addresses(client, addr)?,
        None => vec![],
    };

    create_v0_tx(
        fee_payer,
        alt_address,
        instructions,
        recent_blockhash,
        alt_addresses,
    )
}

/// Build a serialized v0 transaction using an explicit ALT address and its entries.
///
/// # Errors
/// Returns an error if blockhash fetching, message compilation or serialization fails.
pub fn create_tx_bytes_with_alt(
    client: &RpcClient,
    fee_payer: Pubkey,
    instructions: &[Instruction],
    alt_address: Pubkey,
    alt_addresses: Vec<Pubkey>,
) -> Result<Vec<u8>> {
    let recent_blockhash = get_recent_blockhash(client)?;

    let alt_account = AddressLookupTableAccount {
        key: alt_address,
        addresses: alt_addresses,
    };

    let v0_message =
        compile_v0_message_with_alt(fee_payer, instructions, recent_blockhash, alt_account)?;
    serialize_v0_transaction(v0_message)
}

/// Fetch the latest blockhash from the Solana cluster.
///
/// # Errors
/// Returns an error if the RPC call fails.
pub fn get_recent_blockhash(client: &RpcClient) -> Result<solana_sdk::hash::Hash> {
    client
        .get_latest_blockhash()
        .map_err(|e| anyhow::anyhow!("Failed to get blockhash: {e}"))
}

/// Compile instructions into a serialized v0 transaction.
///
/// When `alt_addresses` is non-empty, the ALT at `alt_address` is used for
/// account compression.
///
/// # Errors
/// Returns an error if message compilation or serialization fails.
pub fn create_v0_tx(
    fee_payer: Pubkey,
    alt_address: Option<Pubkey>,
    instructions: &[Instruction],
    recent_blockhash: solana_sdk::hash::Hash,
    alt_addresses: Vec<Pubkey>,
) -> Result<Vec<u8>> {
    let v0_message = if alt_addresses.is_empty() {
        compile_v0_message(fee_payer, instructions, recent_blockhash)?
    } else {
        let alt_account = AddressLookupTableAccount {
            key: alt_address.ok_or_else(|| {
                anyhow::anyhow!("ALT address required when alt_addresses is non-empty")
            })?,
            addresses: alt_addresses,
        };
        compile_v0_message_with_alt(fee_payer, instructions, recent_blockhash, alt_account)?
    };

    serialize_v0_transaction(v0_message)
}

/// Compile a v0 message without an address lookup table.
///
/// # Errors
/// Returns an error if compilation fails (e.g. too many accounts).
pub fn compile_v0_message(
    fee_payer: Pubkey,
    instructions: &[Instruction],
    recent_blockhash: solana_sdk::hash::Hash,
) -> Result<v0::Message> {
    v0::Message::try_compile(&fee_payer, instructions, &[], recent_blockhash)
        .map_err(|e| anyhow::anyhow!("Failed to compile v0 message: {e}"))
}

/// Compile a v0 message with an address lookup table.
///
/// # Errors
/// Returns an error if compilation fails.
pub fn compile_v0_message_with_alt(
    fee_payer: Pubkey,
    instructions: &[Instruction],
    recent_blockhash: solana_sdk::hash::Hash,
    alt_account: AddressLookupTableAccount,
) -> Result<v0::Message> {
    v0::Message::try_compile(&fee_payer, instructions, &[alt_account], recent_blockhash)
        .map_err(|e| anyhow::anyhow!("Failed to compile v0 message with ALT: {e}"))
}

/// Serialize a v0 message into a versioned transaction with placeholder signatures.
///
/// # Errors
/// Returns an error if bincode serialization fails.
pub fn serialize_v0_transaction(v0_message: v0::Message) -> Result<Vec<u8>> {
    let num_signatures = v0_message.header.num_required_signatures as usize;
    let versioned_tx = VersionedTransaction {
        signatures: vec![solana_sdk::signature::Signature::default(); num_signatures],
        message: VersionedMessage::V0(v0_message),
    };
    let serialized_tx = bincode::serialize(&versioned_tx)?;
    Ok(serialized_tx)
}

/// Fetch the addresses stored in an on-chain Address Lookup Table.
///
/// # Errors
/// Returns an error if the account cannot be fetched or deserialized.
pub fn fetch_alt_addresses(client: &RpcClient, alt_address: Pubkey) -> Result<Vec<Pubkey>> {
    let alt_account = client
        .get_account_with_commitment(&alt_address, CommitmentConfig::confirmed())
        .map_err(|e| anyhow::anyhow!("Failed to fetch ALT account {alt_address}: {e}"))?
        .value
        .ok_or_else(|| anyhow::anyhow!("ALT account {alt_address} not found"))?;

    let lookup_table = AddressLookupTable::deserialize(&alt_account.data)
        .map_err(|e| anyhow::anyhow!("Failed to deserialize ALT: {e}"))?;

    Ok(lookup_table.addresses.to_vec())
}

/// Build a transaction that creates a new Address Lookup Table.
///
/// # Errors
/// Returns an error if transaction building fails.
pub fn build_create_alt_tx(
    client: &RpcClient,
    fee_payer: Pubkey,
    alt_address: Option<Pubkey>,
    slot: u64,
) -> Result<Vec<u8>> {
    let (create_ix, _alt_address) = create_lookup_table(fee_payer, fee_payer, slot);
    create_tx_bytes(client, fee_payer, alt_address, &[create_ix])
}

/// Build a transaction that extends an Address Lookup Table with new accounts.
///
/// # Errors
/// Returns an error if transaction building fails.
pub fn build_extend_alt_tx(
    client: &RpcClient,
    fee_payer: Pubkey,
    alt_address_for_tx: Option<Pubkey>,
    slot: u64,
    accounts: Vec<Pubkey>,
) -> Result<Vec<u8>> {
    let (alt_address, _) = derive_alt_address(slot, fee_payer);
    let extend_ix = extend_lookup_table(alt_address, fee_payer, Some(fee_payer), accounts);
    create_tx_bytes(client, fee_payer, alt_address_for_tx, &[extend_ix])
}

/// Build compute budget instructions (compute unit limit + priority fee).
#[must_use]
pub fn extend_compute_ix() -> Vec<Instruction> {
    vec![
        ComputeBudgetInstruction::set_compute_unit_limit(MAX_COMPUTE_UNIT_LIMIT),
        ComputeBudgetInstruction::set_compute_unit_price(DEFAULT_PRIORITY_FEE),
    ]
}

/// Build compute budget instructions with an increased heap frame.
#[must_use]
pub fn extend_compute_ix_with_heap() -> Vec<Instruction> {
    vec![
        ComputeBudgetInstruction::set_compute_unit_limit(MAX_COMPUTE_UNIT_LIMIT),
        ComputeBudgetInstruction::set_compute_unit_price(DEFAULT_PRIORITY_FEE),
        ComputeBudgetInstruction::request_heap_frame(HEAP_FRAME_SIZE),
    ]
}

/// Derive the Address Lookup Table address from a slot and authority.
#[must_use]
pub fn derive_alt_address(slot: u64, authority: Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[authority.as_ref(), &slot.to_le_bytes()],
        &solana_sdk::address_lookup_table::program::id(),
    )
}
