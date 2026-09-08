// SPDX-License-Identifier: Apache-2.0

use crate::errors::AccessManagerError;
use crate::events::{
    AccessManagerTransferAccepted, AccessManagerTransferCancelled, AccessManagerTransferProposed,
};
use crate::helpers::role_checks::require_admin;
use crate::state::AccessManagerState;
use anchor_lang::prelude::*;

impl AccessManagerState {
    /// Proposes transferring the access manager to a new program.
    ///
    /// Validates admin authorization against the current AM, rejects zero
    /// addresses and self-transfers, and ensures no pending transfer already
    /// exists.
    pub fn propose_transfer(
        &mut self,
        new_access_manager: Pubkey,
        access_manager_account: &AccountInfo,
        admin: &AccountInfo,
        instructions_sysvar: &AccountInfo,
        program_id: &Pubkey,
    ) -> Result<()> {
        require_admin(
            access_manager_account,
            admin,
            instructions_sysvar,
            program_id,
        )?;

        require!(
            new_access_manager != Pubkey::default(),
            AccessManagerError::InvalidProposedAccessManager
        );

        require!(
            new_access_manager != self.access_manager,
            AccessManagerError::AccessManagerSelfTransfer
        );

        require!(
            self.pending_access_manager.is_none(),
            AccessManagerError::PendingAccessManagerTransferAlreadyExists
        );

        self.pending_access_manager = Some(new_access_manager);

        emit!(AccessManagerTransferProposed {
            current_access_manager: self.access_manager,
            proposed_access_manager: new_access_manager,
        });

        Ok(())
    }

    /// Accepts a pending access manager transfer.
    ///
    /// Anchor constraints on the consumer program must verify that a pending
    /// transfer exists and that `new_access_manager_account` matches the
    /// expected PDA before calling this method.
    /// Checks admin authorization against the **new** AM and updates state.
    pub fn accept_transfer(
        &mut self,
        new_access_manager_account: &AccountInfo,
        admin: &AccountInfo,
        instructions_sysvar: &AccountInfo,
        program_id: &Pubkey,
    ) -> Result<()> {
        require_admin(
            new_access_manager_account,
            admin,
            instructions_sysvar,
            program_id,
        )?;

        let pending_am_program = self
            .pending_access_manager
            .ok_or_else(|| error!(AccessManagerError::NoPendingAccessManagerTransfer))?;
        self.pending_access_manager = None;

        let old = self.access_manager;
        self.access_manager = pending_am_program;

        emit!(AccessManagerTransferAccepted {
            old_access_manager: old,
            new_access_manager: pending_am_program,
        });

        Ok(())
    }

    /// Cancels a pending access manager transfer.
    ///
    /// Validates admin authorization against the current AM and clears the
    /// pending transfer.
    pub fn cancel_transfer(
        &mut self,
        access_manager_account: &AccountInfo,
        admin: &AccountInfo,
        instructions_sysvar: &AccountInfo,
        program_id: &Pubkey,
    ) -> Result<()> {
        require_admin(
            access_manager_account,
            admin,
            instructions_sysvar,
            program_id,
        )?;

        let pending = self
            .pending_access_manager
            .ok_or_else(|| error!(AccessManagerError::NoPendingAccessManagerTransfer))?;
        self.pending_access_manager = None;

        emit!(AccessManagerTransferCancelled {
            access_manager: self.access_manager,
            cancelled_access_manager: pending,
        });

        Ok(())
    }
}
