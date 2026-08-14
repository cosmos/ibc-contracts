// SPDX-License-Identifier: Apache-2.0

use crate::types::{PendingAuthorityTransfer, RoleData};
use anchor_lang::prelude::*;
use solana_ibc_types::roles;

const MAX_PENDING_TRANSFERS: usize = 16;

/// Embedded access manager state for IBC programs.
///
/// Each IBC program embeds this struct in its on-chain state account to track
/// which access manager program governs its permissioned instructions and to
/// support two-step access manager migration (propose/accept).
///
/// Does not carry its own `_reserved` field — future fields can eat into the
/// `_reserved` space of the higher-level state that embeds this struct.
#[derive(AnchorSerialize, AnchorDeserialize, Clone, InitSpace, Debug)]
pub struct AccessManagerState {
    /// Program ID of the access manager that governs this program's roles.
    pub access_manager: Pubkey,
    /// Proposed replacement access manager, set during a pending transfer.
    pub pending_access_manager: Option<Pubkey>,
}

impl AccessManagerState {
    pub const fn new(access_manager: Pubkey) -> Self {
        Self {
            access_manager,
            pending_access_manager: None,
        }
    }
}

/// Central role-based access control registry shared across all Solana IBC programs.
///
/// Every program that requires permissioned operations (e.g. relaying, pausing,
/// admin configuration) delegates authorization checks to this account.
/// It stores a list of roles with their members and a whitelist of program IDs
/// that are allowed to invoke admin-gated instructions via CPI.
///
/// Role IDs are opaque `u64` values — the access manager does not interpret them.
/// Consuming programs define and enforce what each role ID means.
#[account]
#[derive(InitSpace, Debug)]
pub struct AccessManager {
    #[max_len(16)]
    pub roles: Vec<RoleData>,
    #[max_len(8)]
    pub whitelisted_programs: Vec<Pubkey>,
    /// Pending upgrade authority transfers, one per managed program.
    ///
    /// Uses a bounded `Vec` rather than a single `Option` to support concurrent
    /// transfers — with timelocked multisigs, a single `Option` would require
    /// N sequential propose/accept cycles (N × timelock waits). With a `Vec`,
    /// all proposes can be batched in one multisig vote.
    ///
    /// A single shared upgrade authority PDA (without `target_program` in the
    /// seed) was considered but rejected: `SetAuthority` must be called once
    /// per program regardless, so a shared PDA saves no transactions, while
    /// per-program PDAs limit the blast radius of bugs and let Anchor's seeds
    /// constraint tie each signer to exactly one target program.
    #[max_len(MAX_PENDING_TRANSFERS)]
    pub pending_authority_transfers: Vec<PendingAuthorityTransfer>,
}

impl AccessManager {
    pub const SEED: &'static [u8] = b"access_manager";
    pub const UPGRADE_AUTHORITY_SEED: &'static [u8] = b"upgrade_authority";
    pub const MAX_PENDING_TRANSFERS: usize = MAX_PENDING_TRANSFERS;

    /// Get upgrade authority PDA for a target program
    pub fn upgrade_authority_pda(target_program: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[Self::UPGRADE_AUTHORITY_SEED, target_program.as_ref()],
            program_id,
        )
    }

    pub fn has_role(&self, role_id: u64, account: &Pubkey) -> bool {
        // PUBLIC_ROLE is accessible to everyone
        if role_id == roles::PUBLIC_ROLE {
            return true;
        }

        // Check if account has the specific role
        self.roles
            .iter()
            .find(|r| r.role_id == role_id)
            .is_some_and(|r| r.members.contains(account))
    }

    pub fn grant_role(&mut self, role_id: u64, account: Pubkey) -> Result<()> {
        if let Some(role) = self.roles.iter_mut().find(|r| r.role_id == role_id) {
            if !role.members.contains(&account) {
                role.members.push(account);
            }
        } else {
            self.roles.push(RoleData {
                role_id,
                members: vec![account],
            });
        }
        Ok(())
    }

    pub fn revoke_role(&mut self, role_id: u64, account: &Pubkey) -> Result<()> {
        // Prevent removing the last admin
        if role_id == roles::ADMIN_ROLE && self.is_last_admin(account) {
            return Err(crate::errors::AccessManagerError::CannotRemoveLastAdmin.into());
        }

        let role = self
            .roles
            .iter_mut()
            .find(|r| r.role_id == role_id)
            .ok_or(crate::errors::AccessManagerError::RoleNotGranted)?;

        let position = role
            .members
            .iter()
            .position(|m| m == account)
            .ok_or(crate::errors::AccessManagerError::RoleNotGranted)?;

        role.members.swap_remove(position);
        Ok(())
    }

    /// Check if an account is the last admin
    fn is_last_admin(&self, account: &Pubkey) -> bool {
        self.roles
            .iter()
            .find(|r| r.role_id == roles::ADMIN_ROLE)
            .is_some_and(|admin_role| {
                admin_role.members.len() == 1 && admin_role.members.contains(account)
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_access_manager() -> AccessManager {
        AccessManager {
            roles: vec![],
            whitelisted_programs: vec![],
            pending_authority_transfers: vec![],
        }
    }

    fn create_access_manager_with_roles(roles: Vec<RoleData>) -> AccessManager {
        AccessManager {
            roles,
            whitelisted_programs: vec![],
            pending_authority_transfers: vec![],
        }
    }

    #[test]
    fn test_admin_does_not_auto_pass_roles() {
        let admin = Pubkey::new_unique();
        let mut access_manager = create_access_manager();

        // Grant admin role
        access_manager.grant_role(roles::ADMIN_ROLE, admin).unwrap();

        // Admin should have ADMIN_ROLE
        assert!(access_manager.has_role(roles::ADMIN_ROLE, &admin));

        // Admin should NOT automatically have other roles
        assert!(!access_manager.has_role(roles::RELAYER_ROLE, &admin));
        assert!(!access_manager.has_role(roles::PAUSER_ROLE, &admin));
        assert!(!access_manager.has_role(roles::UNPAUSER_ROLE, &admin));
    }

    #[test]
    fn test_public_role_accessible_to_all() {
        let anyone = Pubkey::new_unique();
        let admin = Pubkey::new_unique();
        let access_manager = create_access_manager();

        assert!(access_manager.has_role(roles::PUBLIC_ROLE, &anyone));
        assert!(access_manager.has_role(roles::PUBLIC_ROLE, &admin));
    }

    #[test]
    fn test_grant_role() {
        let relayer = Pubkey::new_unique();
        let mut access_manager = create_access_manager();

        assert!(!access_manager.has_role(roles::RELAYER_ROLE, &relayer));

        access_manager
            .grant_role(roles::RELAYER_ROLE, relayer)
            .unwrap();

        assert!(access_manager.has_role(roles::RELAYER_ROLE, &relayer));
    }

    #[test]
    fn test_revoke_role() {
        let relayer = Pubkey::new_unique();
        let mut access_manager = create_access_manager_with_roles(vec![RoleData {
            role_id: roles::RELAYER_ROLE,
            members: vec![relayer],
        }]);

        assert!(access_manager.has_role(roles::RELAYER_ROLE, &relayer));

        access_manager
            .revoke_role(roles::RELAYER_ROLE, &relayer)
            .unwrap();

        assert!(!access_manager.has_role(roles::RELAYER_ROLE, &relayer));
    }

    #[test]
    fn test_cannot_remove_last_admin() {
        let admin = Pubkey::new_unique();
        let mut access_manager = create_access_manager_with_roles(vec![RoleData {
            role_id: roles::ADMIN_ROLE,
            members: vec![admin],
        }]);

        assert!(access_manager.has_role(roles::ADMIN_ROLE, &admin));

        // Should fail to revoke last admin
        let result = access_manager.revoke_role(roles::ADMIN_ROLE, &admin);
        assert!(result.is_err());

        // Admin should still have the role
        assert!(access_manager.has_role(roles::ADMIN_ROLE, &admin));
    }

    #[test]
    fn test_can_remove_non_last_admin() {
        let admin1 = Pubkey::new_unique();
        let admin2 = Pubkey::new_unique();
        let mut access_manager = create_access_manager_with_roles(vec![RoleData {
            role_id: roles::ADMIN_ROLE,
            members: vec![admin1, admin2],
        }]);

        // Should succeed to revoke one admin when multiple exist
        access_manager
            .revoke_role(roles::ADMIN_ROLE, &admin1)
            .unwrap();

        assert!(!access_manager.has_role(roles::ADMIN_ROLE, &admin1));
        assert!(access_manager.has_role(roles::ADMIN_ROLE, &admin2));
    }

    #[test]
    fn test_grant_role_idempotent() {
        let relayer = Pubkey::new_unique();
        let mut access_manager = create_access_manager();

        access_manager
            .grant_role(roles::RELAYER_ROLE, relayer)
            .unwrap();
        access_manager
            .grant_role(roles::RELAYER_ROLE, relayer)
            .unwrap();

        let role = access_manager
            .roles
            .iter()
            .find(|r| r.role_id == roles::RELAYER_ROLE)
            .unwrap();
        assert_eq!(role.members.len(), 1);
    }

    #[test]
    fn test_multiple_members_per_role() {
        let relayer1 = Pubkey::new_unique();
        let relayer2 = Pubkey::new_unique();
        let mut access_manager = create_access_manager();

        access_manager
            .grant_role(roles::RELAYER_ROLE, relayer1)
            .unwrap();
        access_manager
            .grant_role(roles::RELAYER_ROLE, relayer2)
            .unwrap();

        assert!(access_manager.has_role(roles::RELAYER_ROLE, &relayer1));
        assert!(access_manager.has_role(roles::RELAYER_ROLE, &relayer2));
    }

    #[test]
    fn test_has_role_empty_roles() {
        let account = Pubkey::new_unique();
        let access_manager = create_access_manager();

        assert!(!access_manager.has_role(roles::ADMIN_ROLE, &account));
        assert!(!access_manager.has_role(roles::RELAYER_ROLE, &account));
    }

    #[test]
    fn test_revoke_role_empty_roles() {
        let account = Pubkey::new_unique();
        let mut access_manager = create_access_manager();

        let result = access_manager.revoke_role(roles::RELAYER_ROLE, &account);
        assert!(result.is_err());
    }

    #[test]
    fn test_revoke_role_account_not_in_role() {
        let member = Pubkey::new_unique();
        let non_member = Pubkey::new_unique();
        let mut access_manager = create_access_manager_with_roles(vec![RoleData {
            role_id: roles::RELAYER_ROLE,
            members: vec![member],
        }]);

        let result = access_manager.revoke_role(roles::RELAYER_ROLE, &non_member);
        assert!(result.is_err());

        assert!(access_manager.has_role(roles::RELAYER_ROLE, &member));
    }

    #[test]
    fn test_revoke_admin_empty_roles() {
        let account = Pubkey::new_unique();
        let mut access_manager = create_access_manager();

        let result = access_manager.revoke_role(roles::ADMIN_ROLE, &account);
        assert!(result.is_err());
    }

    #[test]
    fn test_revoke_admin_when_account_not_admin() {
        let admin = Pubkey::new_unique();
        let non_admin = Pubkey::new_unique();
        let mut access_manager = create_access_manager_with_roles(vec![RoleData {
            role_id: roles::ADMIN_ROLE,
            members: vec![admin],
        }]);

        let result = access_manager.revoke_role(roles::ADMIN_ROLE, &non_admin);
        assert!(result.is_err());

        assert!(access_manager.has_role(roles::ADMIN_ROLE, &admin));
    }

    #[test]
    fn test_roles_are_isolated() {
        let account = Pubkey::new_unique();
        let mut access_manager = create_access_manager();

        access_manager
            .grant_role(roles::RELAYER_ROLE, account)
            .unwrap();
        access_manager
            .grant_role(roles::PAUSER_ROLE, account)
            .unwrap();

        assert!(access_manager.has_role(roles::RELAYER_ROLE, &account));
        assert!(access_manager.has_role(roles::PAUSER_ROLE, &account));

        // Revoking one role should not affect the other
        access_manager
            .revoke_role(roles::RELAYER_ROLE, &account)
            .unwrap();

        assert!(!access_manager.has_role(roles::RELAYER_ROLE, &account));
        assert!(access_manager.has_role(roles::PAUSER_ROLE, &account));
    }

    #[test]
    fn test_revoke_non_existent_role_id() {
        let account = Pubkey::new_unique();
        let mut access_manager = create_access_manager_with_roles(vec![RoleData {
            role_id: roles::RELAYER_ROLE,
            members: vec![account],
        }]);

        let non_existent_role = 999;
        let result = access_manager.revoke_role(non_existent_role, &account);
        assert!(result.is_err());

        assert!(access_manager.has_role(roles::RELAYER_ROLE, &account));
    }

    #[test]
    fn test_public_role_grant_revoke() {
        let account = Pubkey::new_unique();
        let mut access_manager = create_access_manager();

        // PUBLIC_ROLE is always accessible regardless of grants
        assert!(access_manager.has_role(roles::PUBLIC_ROLE, &account));

        // Granting PUBLIC_ROLE adds it to storage (though meaningless)
        access_manager
            .grant_role(roles::PUBLIC_ROLE, account)
            .unwrap();
        assert!(access_manager.has_role(roles::PUBLIC_ROLE, &account));

        // Revoking PUBLIC_ROLE removes from storage but still accessible
        access_manager
            .revoke_role(roles::PUBLIC_ROLE, &account)
            .unwrap();
        assert!(access_manager.has_role(roles::PUBLIC_ROLE, &account));
    }
}
