// SPDX-License-Identifier: Apache-2.0

//! `AccessManager` transfer integration tests for ICS26 Router and ICS27 GMP.
//!
//! Each test loads a `TestAccessManager` alongside the default `access_manager`
//! and the attestation light client. The admin proposes transferring the AM on
//! a target program (ICS26/GMP) to the test AM, then the test AM's admin
//! accepts (or the original admin cancels).
//!
//! ## Coverage gaps (not testable at integration level)
//!
//! - **ICS07 Tendermint AM transfer**: the integration test crate does not load
//!   `ics07-tendermint`.
//! - **AM-to-AM upgrade authority migration**: requires buffer accounts with
//!   real ELF binaries; tested in per-program `ProgramTest` and e2e.

use access_manager::AccessManagerError;
use integration_tests::{
    admin::Admin,
    anchor_error_code,
    attestor::Attestors,
    chain::{Chain, ChainProgram},
    deployer::Deployer,
    extract_custom_error, gmp,
    programs::{AttestationLc, Ics27Gmp, TestAccessManager},
    relayer::Relayer,
    router,
};

mod am_transfer_gmp;
mod am_transfer_ics26;
