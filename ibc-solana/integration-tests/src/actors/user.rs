// SPDX-License-Identifier: Apache-2.0

//! End-user actor.
//!
//! Sends packets via `test_ibc_app`, initiates GMP calls through
//! `ics27_gmp` and starts IFT transfers. The user pays transaction
//! fees for all operations.

use super::Actor;
use crate::chain::Chain;
use crate::gmp::{self, GmpSendCallParams};
use crate::ift::{self, IftTransferParams, TokenKind};
use crate::router::{self, SendPacketParams, SendResult};
use solana_program_test::BanksClientError;
use solana_sdk::{pubkey::Pubkey, signature::Keypair};

/// End-user actor that initiates packets, GMP calls and IFT transfers.
pub struct User {
    keypair: Keypair,
}

impl Default for User {
    fn default() -> Self {
        Self::new()
    }
}

impl Actor for User {
    fn keypair(&self) -> &Keypair {
        &self.keypair
    }
}

impl User {
    /// Create a user with a fresh random keypair.
    pub fn new() -> Self {
        Self {
            keypair: Keypair::new(),
        }
    }

    /// Send a packet via `test_ibc_app` (user is the payer).
    pub async fn send_packet(
        &self,
        chain: &mut Chain,
        params: SendPacketParams<'_>,
    ) -> Result<SendResult, BanksClientError> {
        let result = router::build_send_packet_ix(
            self.pubkey(),
            chain.client_id(),
            chain.counterparty_client_id(),
            chain.clock_time(),
            &chain.lc_accounts(),
            params,
        );
        self.send_tx(chain, std::slice::from_ref(&result.ix))
            .await?;
        Ok(result)
    }

    /// Send a packet via `test_ibc_app` using explicit client IDs and LC accounts.
    ///
    /// Use when sending through a non-primary client (e.g. secondary connections
    /// in multi-hop scenarios).
    pub async fn send_packet_for_client(
        &self,
        chain: &mut Chain,
        client_id: &str,
        counterparty_client_id: &str,
        lc: &crate::chain::LcAccounts,
        params: SendPacketParams<'_>,
    ) -> Result<SendResult, BanksClientError> {
        let result = router::build_send_packet_ix(
            self.pubkey(),
            client_id,
            counterparty_client_id,
            chain.clock_time(),
            lc,
            params,
        );
        self.send_tx(chain, std::slice::from_ref(&result.ix))
            .await?;
        Ok(result)
    }

    /// Send a GMP call (user pays fees).
    pub async fn send_call(
        &self,
        chain: &mut Chain,
        params: GmpSendCallParams<'_>,
    ) -> Result<Pubkey, BanksClientError> {
        let (ix, commitment_pda) = gmp::build_gmp_send_call_ix(
            self.pubkey(),
            self.pubkey(),
            chain.client_id(),
            &chain.lc_accounts(),
            params,
        );
        self.send_tx(chain, &[ix]).await?;
        Ok(commitment_pda)
    }

    /// Send an IFT transfer (user pays fees).
    pub async fn ift_transfer(
        &self,
        chain: &mut Chain,
        mint: Pubkey,
        token_kind: TokenKind,
        params: IftTransferParams,
    ) -> Result<ift::IftTransferResult, BanksClientError> {
        let result = ift::build_ift_transfer_ix(
            self.pubkey(),
            self.pubkey(),
            chain.client_id(),
            mint,
            token_kind,
            &chain.lc_accounts(),
            params,
        );
        self.send_tx(chain, std::slice::from_ref(&result.ix))
            .await?;
        Ok(result)
    }
}
