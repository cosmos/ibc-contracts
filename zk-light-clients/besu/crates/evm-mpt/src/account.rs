use alloc::vec::Vec;

use besu_zk_types::keccak256;

use crate::{
    nibble::secure_key_nibbles,
    proof::{verify_trie, TrieResult},
    rlp::{decode_exact, exact_list_fields, validate_integer},
    MptError,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthenticatedAccount {
    pub nonce: [u8; 32],
    pub balance: [u8; 32],
    pub storage_root: [u8; 32],
    pub code_hash: [u8; 32],
}

struct AccountFields<'a> {
    nonce: &'a [u8],
    balance: &'a [u8],
    storage_root: [u8; 32],
    code_hash: [u8; 32],
}

pub fn verify_account(
    state_root: [u8; 32],
    address: [u8; 20],
    proof: &[Vec<u8>],
) -> Result<AuthenticatedAccount, MptError> {
    let value = authenticated_account_value(state_root, address, proof)?;
    let fields = decode_account(value)?;
    Ok(AuthenticatedAccount {
        nonce: integer_word(fields.nonce)?,
        balance: integer_word(fields.balance)?,
        storage_root: fields.storage_root,
        code_hash: fields.code_hash,
    })
}

/// Authenticates the complete account while returning only the field required
/// by the membership and non-membership guests.
pub fn verify_account_storage_root(
    state_root: [u8; 32],
    address: [u8; 20],
    proof: &[Vec<u8>],
) -> Result<[u8; 32], MptError> {
    let value = authenticated_account_value(state_root, address, proof)?;
    Ok(decode_account(value)?.storage_root)
}

fn authenticated_account_value(
    state_root: [u8; 32],
    address: [u8; 20],
    proof: &[Vec<u8>],
) -> Result<&[u8], MptError> {
    let path = secure_key_nibbles(keccak256(&address));
    let value = match verify_trie(state_root, &path, proof)? {
        TrieResult::Present(value) => value,
        TrieResult::Absent => return Err(MptError::AccountAbsent),
    };
    Ok(value)
}

fn decode_account(value: &[u8]) -> Result<AccountFields<'_>, MptError> {
    let [nonce_item, balance_item, storage_root_item, code_hash_item] =
        exact_list_fields::<4>(decode_exact(value)?)?;
    let nonce = nonce_item.as_bytes()?;
    let balance = balance_item.as_bytes()?;
    validate_account_integer(nonce)?;
    validate_account_integer(balance)?;
    let storage_root = storage_root_item
        .as_bytes()?
        .try_into()
        .map_err(|_| MptError::InvalidAccount)?;
    let code_hash = code_hash_item
        .as_bytes()?
        .try_into()
        .map_err(|_| MptError::InvalidAccount)?;
    Ok(AccountFields {
        nonce,
        balance,
        storage_root,
        code_hash,
    })
}

// Only called with canonical integer bytes already checked by decode_account.
fn integer_word(bytes: &[u8]) -> Result<[u8; 32], MptError> {
    let mut word = [0_u8; 32];
    let start = 32_usize
        .checked_sub(bytes.len())
        .ok_or(MptError::InvalidAccount)?;
    word[start..].copy_from_slice(bytes);
    Ok(word)
}

fn validate_account_integer(bytes: &[u8]) -> Result<(), MptError> {
    validate_integer(bytes)?;
    if bytes.len() > 32 {
        return Err(MptError::InvalidAccount);
    }
    Ok(())
}
