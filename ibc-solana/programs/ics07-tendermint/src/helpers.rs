// SPDX-License-Identifier: Apache-2.0

use crate::error::ErrorCode;
use anchor_lang::prelude::*;
use borsh::BorshDeserialize;
use ibc_client_tendermint::types::{Header, Misbehaviour};
use ibc_core_commitment_types::merkle::MerkleProof;
use ibc_proto::ibc::core::commitment::v1::MerkleProof as RawMerkleProof;
use ibc_proto::Protobuf;
use solana_ibc_borsh_header::{HeaderWrapper, MisbehaviourWrapper};

// Direct deserialization: bytes → Header in one pass (saves ~300k CU)
pub fn deserialize_header(bytes: &[u8]) -> Result<Header> {
    let wrapper =
        HeaderWrapper::try_from_slice(bytes).map_err(|_| error!(ErrorCode::InvalidHeader))?;

    Ok(wrapper.0)
}

pub fn deserialize_merkle_proof(bytes: &[u8]) -> Result<MerkleProof> {
    <MerkleProof as Protobuf<RawMerkleProof>>::decode_vec(bytes)
        .map_err(|_| error!(ErrorCode::InvalidProof))
}

// Direct deserialization: bytes → Misbehaviour in one pass (saves ~300k CU)
pub fn deserialize_misbehaviour(bytes: &[u8]) -> Result<Misbehaviour> {
    let wrapper = MisbehaviourWrapper::try_from_slice(bytes)
        .map_err(|_| error!(ErrorCode::InvalidMisbehaviour))?;

    Ok(wrapper.0)
}

/// Closes an account and transfers its lamports to the recipient
pub fn close_account(account: &AccountInfo, recipient: &AccountInfo) -> Result<()> {
    // Zero out account data
    {
        let mut data = account.try_borrow_mut_data()?;
        data.fill(0);
    }

    // Transfer lamports in a separate scope to ensure borrows are dropped
    {
        let mut lamports = account.try_borrow_mut_lamports()?;
        let mut recipient_lamports = recipient.try_borrow_mut_lamports()?;

        **recipient_lamports = recipient_lamports
            .checked_add(**lamports)
            .ok_or(ErrorCode::ArithmeticOverflow)?;
        **lamports = 0;
    }

    Ok(())
}
