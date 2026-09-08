// SPDX-License-Identifier: Apache-2.0

//! GMP account extraction - delegates to shared `proof_api_lib::utils::solana_gmp`.

pub use proof_api_lib::utils::solana_gmp::{
    extract_gmp_accounts, extract_gmp_prefund_lamports, find_gmp_result_pda, AbiGmpPacketData,
    ABI_ENCODING, GMP_PORT_ID, MAX_PREFUND_LAMPORTS, PROTOBUF_ENCODING,
};

use alloy::sol_types::SolValue;
use solana_ibc_proto::{GmpPacketData, Protobuf, RawGmpPacketData};

/// Decode a GMP packet from either protobuf or ABI encoding.
///
/// Returns `None` when decoding fails (treated as a non-GMP packet).
pub(crate) fn decode_gmp_packet(payload_value: &[u8], encoding: &str) -> Option<GmpPacketData> {
    match encoding {
        ABI_ENCODING => {
            let abi = AbiGmpPacketData::abi_decode(payload_value).ok()?;
            let raw = RawGmpPacketData {
                sender: abi.sender,
                receiver: abi.receiver,
                salt: abi.salt.to_vec(),
                payload: abi.payload.to_vec(),
                memo: abi.memo,
            };
            GmpPacketData::try_from(raw).ok()
        }
        _ => GmpPacketData::decode_vec(payload_value).ok(),
    }
}
