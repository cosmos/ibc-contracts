// SPDX-License-Identifier: Apache-2.0

//! Manual PDA helpers for accounts that Anchor doesn't export to the IDL.
//!
//! Anchor omits PDA seed definitions for `init_if_needed` accounts, so
//! the codegen cannot generate helpers for them. This module provides
//! hand-written equivalents keyed to the canonical seed constants in each
//! on-chain program.

use anchor_lang::prelude::Pubkey;

/// PDA helpers for the ICS07 Tendermint light client program.
pub mod ics07_tendermint {
    use super::Pubkey;

    /// Derives the header chunk PDA for chunked light-client header uploads.
    ///
    /// Seeds: `[b"header_chunk", submitter, height_le, chunk_index]`
    #[must_use]
    pub fn header_chunk_pda(
        submitter: &Pubkey,
        height: u64,
        chunk_index: u8,
        program_id: &Pubkey,
    ) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[
                b"header_chunk",
                submitter.as_ref(),
                &height.to_le_bytes(),
                &[chunk_index],
            ],
            program_id,
        )
    }

    /// Derives the signature verification PDA.
    ///
    /// Seeds: `[b"sig_verify", signature_hash]`
    #[must_use]
    pub fn sig_verify_pda(signature_hash: &[u8; 32], program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[b"sig_verify", signature_hash], program_id)
    }
}

/// PDA helpers for the ICS26 router program.
pub mod ics26_router {
    use super::Pubkey;

    /// Derives the payload chunk PDA for chunked packet payload uploads.
    ///
    /// Seeds: `[b"payload_chunk", payer, client_id, sequence_le, payload_index, chunk_index]`
    #[must_use]
    pub fn payload_chunk_pda(
        payer: &Pubkey,
        client_id: &str,
        sequence: u64,
        payload_index: u8,
        chunk_index: u8,
        program_id: &Pubkey,
    ) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[
                b"payload_chunk",
                payer.as_ref(),
                client_id.as_bytes(),
                &sequence.to_le_bytes(),
                &[payload_index],
                &[chunk_index],
            ],
            program_id,
        )
    }

    /// Derives the proof chunk PDA for chunked membership proof uploads.
    ///
    /// Seeds: `[b"proof_chunk", payer, client_id, sequence_le, chunk_index]`
    #[must_use]
    pub fn proof_chunk_pda(
        payer: &Pubkey,
        client_id: &str,
        sequence: u64,
        chunk_index: u8,
        program_id: &Pubkey,
    ) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[
                b"proof_chunk",
                payer.as_ref(),
                client_id.as_bytes(),
                &sequence.to_le_bytes(),
                &[chunk_index],
            ],
            program_id,
        )
    }
}

/// PDA helpers common to IBC application programs (GMP, IFT, etc.).
pub mod ibc_app {
    use super::Pubkey;

    /// Derives the `app_state` PDA for any IBC application program.
    ///
    /// Seeds: `[b"app_state"]`
    #[must_use]
    pub fn app_state_pda(program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[b"app_state"], program_id)
    }
}
