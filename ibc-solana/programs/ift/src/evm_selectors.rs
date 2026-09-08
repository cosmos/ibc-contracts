// SPDX-License-Identifier: Apache-2.0

//! Compile-time generated constants (EVM selectors, Anchor discriminators).

include!(concat!(env!("OUT_DIR"), "/evm_selectors.rs"));
