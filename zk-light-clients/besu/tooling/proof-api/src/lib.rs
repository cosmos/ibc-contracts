#![forbid(unsafe_code)]
// tonic's generated service signatures use Status directly.
#![allow(clippy::result_large_err)]

#[allow(clippy::all, clippy::pedantic)]
pub mod proto {
    tonic::include_proto!("ibc.v2.prover");
    include!(concat!(env!("OUT_DIR"), "/ibc.v2.prover.serde.rs"));
}

pub mod service;
pub mod transport;

pub mod packet;

pub mod config;

pub mod backend;
pub mod contracts;
pub mod engine;
pub mod integration;

pub mod observability;
pub mod runtime;
