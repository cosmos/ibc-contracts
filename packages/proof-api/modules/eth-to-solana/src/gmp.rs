// SPDX-License-Identifier: Apache-2.0

//! GMP re-exports from the shared proof API library.

pub use proof_api_lib::utils::solana_gmp::{
    extract_gmp_accounts, extract_gmp_prefund_lamports, find_gmp_result_pda, AbiGmpPacketData,
    ABI_ENCODING, GMP_PORT_ID, MAX_PREFUND_LAMPORTS, PROTOBUF_ENCODING,
};
