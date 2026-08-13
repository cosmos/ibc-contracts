// SPDX-License-Identifier: Apache-2.0

//! Test actors that model distinct on-chain roles.
//!
//! Each actor wraps a [`Keypair`] and exposes high-level async methods
//! that build, sign and submit transactions. The shared [`Actor`] trait
//! provides uniform `keypair()` / `pubkey()` accessors and a default
//! `send_tx` method that signs and submits a transaction.

use crate::chain::Chain;
use solana_program_test::BanksClientError;
use solana_sdk::{
    instruction::Instruction, pubkey::Pubkey, signature::Keypair, signer::Signer as _,
    transaction::Transaction,
};
use std::{future::Future, pin::Pin};

pub mod admin;
pub mod deployer;
pub mod ift_admin;
pub mod relayer;
pub mod user;

/// Shared interface for test actors (`Deployer`, `Admin`, `IftAdmin`, `User`, `Relayer`).
pub trait Actor: Sync {
    fn keypair(&self) -> &Keypair;

    fn pubkey(&self) -> Pubkey {
        self.keypair().pubkey()
    }

    /// Sign and submit a transaction with the actor's keypair as both fee payer and signer.
    fn send_tx<'a>(
        &'a self,
        chain: &'a mut Chain,
        ixs: &'a [Instruction],
    ) -> Pin<Box<dyn Future<Output = Result<(), BanksClientError>> + Send + 'a>> {
        Box::pin(async move {
            let keypair = self.keypair();
            let tx = Transaction::new_signed_with_payer(
                ixs,
                Some(&keypair.pubkey()),
                &[keypair],
                chain.blockhash(),
            );
            chain.process_transaction(tx).await
        })
    }
}
