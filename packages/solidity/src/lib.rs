// SPDX-License-Identifier: Apache-2.0

//! Solidity types for `solidity-ibc-eureka`

#![deny(clippy::nursery, clippy::pedantic, warnings, unused_crate_dependencies)]

pub mod attestation;
pub mod besu;
pub mod dummy;
pub mod ics26;
pub mod msgs;
pub mod sp1_ics07;

#[derive(thiserror::Error, Debug)]
#[allow(missing_docs)]
pub enum FromStrError {
    #[error("unsupported zk algorithm: {0}")]
    UnsupportedZkAlgorithm(String),
}
