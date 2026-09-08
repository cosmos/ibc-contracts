// SPDX-License-Identifier: Apache-2.0

//! IFT program admin actor.
//!
//! Signs IFT-specific admin operations: pause/unpause, admin transfer
//! (propose, accept, cancel) and admin mint. Authorization is checked
//! against the `admin` field in `IFTAppState`, not the access manager.

use super::Actor;
use crate::chain::Chain;
use crate::ift::TokenKind;
use solana_program_test::BanksClientError;
use solana_sdk::{pubkey::Pubkey, signature::Keypair};

/// IFT program admin actor.
pub struct IftAdmin {
    keypair: Keypair,
}

impl Default for IftAdmin {
    fn default() -> Self {
        Self::new()
    }
}

impl Actor for IftAdmin {
    fn keypair(&self) -> &Keypair {
        &self.keypair
    }
}

impl IftAdmin {
    /// Create an IFT admin with a fresh random keypair.
    pub fn new() -> Self {
        Self {
            keypair: Keypair::new(),
        }
    }

    /// Create an IFT admin from an existing keypair.
    pub const fn from_keypair(keypair: Keypair) -> Self {
        Self { keypair }
    }

    /// Pause or unpause the IFT program.
    pub async fn set_paused(
        &self,
        chain: &mut Chain,
        paused: bool,
    ) -> Result<(), BanksClientError> {
        let ix = crate::ift::build_set_paused_ix(self.pubkey(), paused);
        self.send_tx(chain, &[ix]).await
    }

    /// Propose a new IFT admin.
    pub async fn propose_admin(
        &self,
        chain: &mut Chain,
        new_admin: Pubkey,
    ) -> Result<(), BanksClientError> {
        let ix = crate::ift::build_propose_admin_ix(self.pubkey(), new_admin);
        self.send_tx(chain, &[ix]).await
    }

    /// Accept the pending IFT admin proposal (must be signed by the proposed admin).
    pub async fn accept_admin(&self, chain: &mut Chain) -> Result<(), BanksClientError> {
        let ix = crate::ift::build_accept_admin_ix(self.pubkey());
        self.send_tx(chain, &[ix]).await
    }

    /// Cancel a pending IFT admin proposal.
    pub async fn cancel_admin_proposal(&self, chain: &mut Chain) -> Result<(), BanksClientError> {
        let ix = crate::ift::build_cancel_admin_proposal_ix(self.pubkey());
        self.send_tx(chain, &[ix]).await
    }

    /// Mint tokens to `receiver` using admin authority.
    pub async fn admin_mint(
        &self,
        chain: &mut Chain,
        mint: Pubkey,
        receiver: Pubkey,
        amount: u64,
        token_kind: TokenKind,
    ) -> Result<(), BanksClientError> {
        let ix = crate::ift::build_admin_mint_ix(
            self.pubkey(),
            self.pubkey(),
            mint,
            receiver,
            amount,
            token_kind,
        );
        self.send_tx(chain, &[ix]).await
    }
}
