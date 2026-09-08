// SPDX-License-Identifier: Apache-2.0

use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::set_return_data;
use ics25_handler::{MembershipMsg, NonMembershipMsg};
use solana_ibc_types::ics07::UpdateClientMsg;

declare_id!("CSLS3A9jS7JAD8aUe3LRXMYZ1U8Lvxn9usGygVrA2arZ");

/// Mock light client that always succeeds.
///
/// Implements the `ics25_handler` light-client interface with no-op logic so
/// the ICS26 router can be tested end-to-end without deploying a real
/// Tendermint or SP1 light client.
#[program]
pub mod mock_light_client {
    use super::*;

    pub fn initialize(
        _ctx: Context<Initialize>,
        _chain_id: String,
        _latest_height: u64,
        _client_state: Vec<u8>,
        _consensus_state: Vec<u8>,
    ) -> Result<()> {
        msg!("Mock light client: initialize always returns success");
        Ok(())
    }

    pub fn verify_membership(_ctx: Context<VerifyMembership>, _msg: MembershipMsg) -> Result<()> {
        if _msg.proof.starts_with(b"REJECT_PROOF") {
            return err!(MockLightClientError::ProofRejected);
        }

        msg!("Mock light client: verify_membership always returns success");

        // Return the height as bytes for membership verification
        let height_bytes = _msg.height.to_le_bytes();
        set_return_data(&height_bytes);

        Ok(())
    }

    pub fn verify_non_membership(
        _ctx: Context<VerifyNonMembership>,
        _msg: NonMembershipMsg,
    ) -> Result<()> {
        msg!("Mock light client: verify_non_membership always returns success");

        // For non-membership (timeout), return a very large timestamp
        // Using u64::MAX to ensure it's always greater than any test timeout value
        let timestamp: u64 = u64::MAX;
        let timestamp_bytes = timestamp.to_le_bytes();
        set_return_data(&timestamp_bytes);

        Ok(())
    }

    pub fn update_client(_ctx: Context<UpdateClient>, _msg: UpdateClientMsg) -> Result<u8> {
        msg!("Mock light client: update_client always returns success");

        // For mock client, always return Update (not NoOp)
        // This matches the UpdateResult enum: Update = 0, NoOp = 1
        Ok(0)
    }

    pub fn client_status(_ctx: Context<ClientStatusCheck>) -> Result<()> {
        set_return_data(&[ics25_handler::ClientStatus::Active.into()]);
        Ok(())
    }
}

/// Accounts for mock light client initialization (creates placeholder PDAs).
#[derive(Accounts)]
#[instruction(chain_id: String, latest_height: u64, client_state: Vec<u8>, consensus_state: Vec<u8>)]
pub struct Initialize<'info> {
    /// CHECK: Mock client state - account will be created but not used
    #[account(
        init,
        payer = payer,
        space = 8 + 1024, // Mock space allocation
        seeds = [b"client", chain_id.as_bytes()],
        bump
    )]
    pub client_state: AccountInfo<'info>,
    /// CHECK: Mock consensus state - account will be created but not used
    #[account(
        init,
        payer = payer,
        space = 8 + 512, // Mock space allocation
        seeds = [b"consensus_state", client_state.key().as_ref(), &latest_height.to_le_bytes()],
        bump
    )]
    pub consensus_state_store: AccountInfo<'info>,
    /// Fee payer for PDA creation.
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

/// Accounts for mock membership verification (always succeeds).
#[derive(Accounts)]
#[instruction(_msg: MembershipMsg)]
pub struct VerifyMembership<'info> {
    /// CHECK: Mock client state - not actually used
    pub client_state: AccountInfo<'info>,
    /// CHECK: Mock consensus state - not actually used
    pub consensus_state: AccountInfo<'info>,
}

/// Accounts for mock non-membership verification (always succeeds).
#[derive(Accounts)]
#[instruction(_msg: NonMembershipMsg)]
pub struct VerifyNonMembership<'info> {
    /// CHECK: Mock client state - not actually used
    pub client_state: AccountInfo<'info>,
    /// CHECK: Mock consensus state - not actually used
    pub consensus_state: AccountInfo<'info>,
}

/// Accounts for mock client status check (always returns `Active`).
#[derive(Accounts)]
pub struct ClientStatusCheck<'info> {
    /// CHECK: Mock client state - not actually used
    pub client_state: AccountInfo<'info>,
    /// CHECK: Mock consensus state - not actually used
    pub consensus_state: AccountInfo<'info>,
}

/// Accounts for mock client update (always returns `Update`).
#[derive(Accounts)]
#[instruction(_msg: UpdateClientMsg)]
pub struct UpdateClient<'info> {
    /// CHECK: Mock client state - not actually used
    pub client_state: AccountInfo<'info>,
    /// CHECK: Mock trusted consensus state - not actually used
    pub trusted_consensus_state: AccountInfo<'info>,
    /// CHECK: Mock new consensus state - not actually used
    pub new_consensus_state: AccountInfo<'info>,
    /// CHECK: Mock payer - not actually used
    pub payer: AccountInfo<'info>,
    /// CHECK: Mock system program - not actually used
    pub system_program: AccountInfo<'info>,
}

#[error_code]
pub enum MockLightClientError {
    #[msg("Proof rejected by mock light client")]
    ProofRejected,
}
