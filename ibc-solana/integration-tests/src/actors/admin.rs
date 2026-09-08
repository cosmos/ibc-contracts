// SPDX-License-Identifier: Apache-2.0

//! Access Manager admin actor.
//!
//! Signs access-manager transfer operations (propose, accept, cancel)
//! for both ICS26 Router and ICS27 GMP programs. Authorization is checked
//! via `ADMIN_ROLE` in the AM account.
//!
//! The admin is an independent keypair whose pubkey is passed to the AM
//! `initialize` instruction by the [`Deployer`](super::deployer::Deployer).

use super::Actor;
use crate::chain::Chain;
use solana_program_test::BanksClientError;
use solana_sdk::{pubkey::Pubkey, signature::Keypair};

/// Access Manager admin actor.
pub struct Admin {
    keypair: Keypair,
}

impl Default for Admin {
    fn default() -> Self {
        Self::new()
    }
}

impl Actor for Admin {
    fn keypair(&self) -> &Keypair {
        &self.keypair
    }
}

impl Admin {
    /// Create an admin with a fresh random keypair.
    pub fn new() -> Self {
        Self {
            keypair: Keypair::new(),
        }
    }

    // ── ICS26 Router AM transfer ────────────────────────────────────────

    /// Propose an ICS26 access-manager transfer.
    pub async fn ics26_propose_am_transfer(
        &self,
        chain: &mut Chain,
        new_access_manager: Pubkey,
    ) -> Result<(), BanksClientError> {
        let ix =
            crate::router::build_ics26_propose_am_transfer_ix(self.pubkey(), new_access_manager);
        self.send_tx(chain, &[ix]).await
    }

    /// Accept a pending ICS26 access-manager transfer.
    pub async fn ics26_accept_am_transfer(
        &self,
        chain: &mut Chain,
        new_am_program_id: Pubkey,
    ) -> Result<(), BanksClientError> {
        let ix = crate::router::build_ics26_accept_am_transfer_ix(self.pubkey(), new_am_program_id);
        self.send_tx(chain, &[ix]).await
    }

    /// Cancel a pending ICS26 access-manager transfer.
    pub async fn ics26_cancel_am_transfer(
        &self,
        chain: &mut Chain,
    ) -> Result<(), BanksClientError> {
        let ix = crate::router::build_ics26_cancel_am_transfer_ix(self.pubkey());
        self.send_tx(chain, &[ix]).await
    }

    // ── GMP AM transfer ─────────────────────────────────────────────────

    /// Propose a GMP access-manager transfer.
    pub async fn gmp_propose_am_transfer(
        &self,
        chain: &mut Chain,
        new_access_manager: Pubkey,
    ) -> Result<(), BanksClientError> {
        let ix = crate::gmp::build_gmp_propose_am_transfer_ix(self.pubkey(), new_access_manager);
        self.send_tx(chain, &[ix]).await
    }

    /// Accept a pending GMP access-manager transfer.
    pub async fn gmp_accept_am_transfer(
        &self,
        chain: &mut Chain,
        new_am_program_id: Pubkey,
    ) -> Result<(), BanksClientError> {
        let ix = crate::gmp::build_gmp_accept_am_transfer_ix(self.pubkey(), new_am_program_id);
        self.send_tx(chain, &[ix]).await
    }

    /// Cancel a pending GMP access-manager transfer.
    pub async fn gmp_cancel_am_transfer(&self, chain: &mut Chain) -> Result<(), BanksClientError> {
        let ix = crate::gmp::build_gmp_cancel_am_transfer_ix(self.pubkey());
        self.send_tx(chain, &[ix]).await
    }
}
