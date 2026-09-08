// SPDX-License-Identifier: Apache-2.0

use anchor_lang::prelude::*;

#[error_code]
pub enum CounterError {
    #[msg("Counter overflow occurred")]
    CounterOverflow,

    #[msg("Counter underflow occurred")]
    CounterUnderflow,

    #[msg("Invalid payload format")]
    InvalidPayload,

    #[msg("Unauthorized GMP caller")]
    UnauthorizedGMPCaller,

    #[msg("Counter not found for user")]
    CounterNotFound,

    #[msg("Invalid instruction in payload")]
    InvalidInstruction,
}
