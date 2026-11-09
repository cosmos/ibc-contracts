use alloc::vec::Vec;

use besu_zk_types::QbftHeaderInput;
use evm_mpt::rlp::{
    append_bytes, append_list_header, append_u256, append_u64, encoded_bytes_len, encoded_list_len,
    encoded_u256_len, encoded_u64_len,
};

use crate::QbftError;

pub fn validate_optional_tail(header: &QbftHeaderInput) -> Result<(), QbftError> {
    let present = [
        header.base_fee_per_gas.is_some(),
        header.withdrawals_root.is_some(),
        header.blob_gas_used.is_some(),
        header.excess_blob_gas.is_some(),
        header.parent_beacon_block_root.is_some(),
        header.requests_hash.is_some(),
        header.block_access_list_hash.is_some(),
    ];
    let mut absent_seen = false;
    for field_present in present {
        if absent_seen && field_present {
            return Err(QbftError::InvalidOptionalTail);
        }
        absent_seen |= !field_present;
    }
    Ok(())
}

pub fn encode_header(header: &QbftHeaderInput, extra_data: &[u8]) -> Result<Vec<u8>, QbftError> {
    validate_optional_tail(header)?;
    encode_header_validated(header, extra_data)
}

pub(crate) fn encode_header_validated(
    header: &QbftHeaderInput,
    extra_data: &[u8],
) -> Result<Vec<u8>, QbftError> {
    let mut payload_len = 0_usize;
    for length in [
        encoded_bytes_len(&header.parent_hash),
        encoded_bytes_len(&header.ommers_hash),
        encoded_bytes_len(&header.beneficiary),
        encoded_bytes_len(&header.state_root),
        encoded_bytes_len(&header.transactions_root),
        encoded_bytes_len(&header.receipts_root),
        encoded_bytes_len(&header.logs_bloom),
        encoded_u256_len(&header.difficulty),
        encoded_u64_len(header.number),
        encoded_u64_len(header.gas_limit),
        encoded_u64_len(header.gas_used),
        encoded_u64_len(header.timestamp),
        encoded_bytes_len(extra_data),
        encoded_bytes_len(&header.mix_hash),
        encoded_bytes_len(&header.nonce),
    ] {
        payload_len = payload_len
            .checked_add(length)
            .ok_or(QbftError::ArithmeticOverflow)?;
    }
    if let Some(value) = &header.base_fee_per_gas {
        payload_len = checked_add(payload_len, encoded_u256_len(value))?;
    }
    if let Some(value) = &header.withdrawals_root {
        payload_len = checked_add(payload_len, encoded_bytes_len(value))?;
    }
    if let Some(value) = header.blob_gas_used {
        payload_len = checked_add(payload_len, encoded_u64_len(value))?;
    }
    if let Some(value) = header.excess_blob_gas {
        payload_len = checked_add(payload_len, encoded_u64_len(value))?;
    }
    if let Some(value) = &header.parent_beacon_block_root {
        payload_len = checked_add(payload_len, encoded_bytes_len(value))?;
    }
    if let Some(value) = &header.requests_hash {
        payload_len = checked_add(payload_len, encoded_bytes_len(value))?;
    }
    if let Some(value) = &header.block_access_list_hash {
        payload_len = checked_add(payload_len, encoded_bytes_len(value))?;
    }

    let encoded_len = encoded_list_len(payload_len);
    let mut encoded = Vec::with_capacity(encoded_len);
    append_list_header(&mut encoded, payload_len);
    append_bytes(&mut encoded, &header.parent_hash);
    append_bytes(&mut encoded, &header.ommers_hash);
    append_bytes(&mut encoded, &header.beneficiary);
    append_bytes(&mut encoded, &header.state_root);
    append_bytes(&mut encoded, &header.transactions_root);
    append_bytes(&mut encoded, &header.receipts_root);
    append_bytes(&mut encoded, &header.logs_bloom);
    append_u256(&mut encoded, &header.difficulty);
    append_u64(&mut encoded, header.number);
    append_u64(&mut encoded, header.gas_limit);
    append_u64(&mut encoded, header.gas_used);
    append_u64(&mut encoded, header.timestamp);
    append_bytes(&mut encoded, extra_data);
    append_bytes(&mut encoded, &header.mix_hash);
    append_bytes(&mut encoded, &header.nonce);
    if let Some(value) = &header.base_fee_per_gas {
        append_u256(&mut encoded, value);
    }
    if let Some(value) = &header.withdrawals_root {
        append_bytes(&mut encoded, value);
    }
    if let Some(value) = header.blob_gas_used {
        append_u64(&mut encoded, value);
    }
    if let Some(value) = header.excess_blob_gas {
        append_u64(&mut encoded, value);
    }
    if let Some(value) = &header.parent_beacon_block_root {
        append_bytes(&mut encoded, value);
    }
    if let Some(value) = &header.requests_hash {
        append_bytes(&mut encoded, value);
    }
    if let Some(value) = &header.block_access_list_hash {
        append_bytes(&mut encoded, value);
    }
    debug_assert_eq!(encoded.len(), encoded_len);
    Ok(encoded)
}

fn checked_add(current: usize, additional: usize) -> Result<usize, QbftError> {
    current
        .checked_add(additional)
        .ok_or(QbftError::ArithmeticOverflow)
}
