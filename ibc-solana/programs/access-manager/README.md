<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Access Manager

Role-based access control for Solana IBC programs. Mirrors Ethereum's OpenZeppelin `AccessManager` pattern, providing unified governance over both runtime operations and program upgrades.

## Overview

The access manager maintains a central registry of roles and their members. Every permissioned operation across Solana IBC programs (relaying, pausing, upgrading) delegates authorization to this single account. It also controls program upgrade authority through PDAs, enabling role-based upgrades without exposing raw keypairs.

## State

```
AccessManager PDA (seeds: ["access_manager"])
  roles:                       Vec<RoleData>                      -- role ID -> member list
  whitelisted_programs:        Vec<Pubkey>                        -- programs allowed to call admin-gated instructions via CPI (e.g. multisig)
  pending_authority_transfers: Vec<PendingAuthorityTransfer>      -- pending two-step upgrade authority transfers (one per target program, max 16)
```

Role IDs are opaque `u64` values defined in [`solana-ibc-types::roles`](../../packages/solana-ibc-types/src/access_manager.rs). The access manager does not interpret them -- consuming programs define what each role means.

## Instructions

### `initialize`

Creates the `AccessManager` PDA and sets the initial admin. Only callable by the program's upgrade authority (deployer). Rejects CPI.

### `grant_role` / `revoke_role`

Adds or removes an account from a role. Requires `ADMIN_ROLE`. The last admin cannot be removed.

### `renounce_role`

Allows an account to remove itself from a role. Does not require admin authorization.

### `set_whitelisted_programs`

Replaces the list of programs allowed to invoke admin-gated instructions via CPI. Requires `ADMIN_ROLE`. This is needed for multisig programs that call admin-gated instructions on behalf of the admin.

### `upgrade_program`

Upgrades a target program's bytecode via BPF Loader Upgradeable. The access manager's PDA acts as the upgrade authority, signing the BPF Loader `Upgrade` call via `invoke_signed`. Requires `ADMIN_ROLE`. Allows whitelisted CPI.

### `propose_upgrade_authority_transfer`

Proposes transferring a target program's BPF Loader upgrade authority from this access manager's PDA to a new address. Adds a pending transfer to the `AccessManager` state. Requires `ADMIN_ROLE`. Allows whitelisted CPI. Supports concurrent transfers for different target programs (up to 16).

### `accept_upgrade_authority_transfer`

Accepts a pending upgrade authority transfer by executing the BPF Loader `SetAuthority` CPI. Must be signed by the proposed new authority. No CPI restriction (supports both keypair signers and multisig/PDA callers).

This operation is irreversible from this access manager's perspective -- once accepted, only the new authority can upgrade the target program.

### `cancel_upgrade_authority_transfer`

Cancels a pending upgrade authority transfer. Requires `ADMIN_ROLE`. Allows whitelisted CPI.

### `claim_upgrade_authority`

Claims upgrade authority from a source access manager that has proposed a transfer to this access manager's upgrade authority PDA. CPIs into the source AM's `accept_upgrade_authority_transfer` with this AM's PDA as signer. Requires `ADMIN_ROLE` because the CPI propagates PDA signer privileges to the source program.

## PDA Derivations

```
access_manager:    ["access_manager"]                              program: access_manager
upgrade_authority: ["upgrade_authority", target_program.as_ref()]   program: access_manager
program_data:      [target_program.as_ref()]                       program: BPF Loader Upgradeable
```

## Flows

### 1. Initial deployment and setup

1. Deployer deploys the Access Manager (AM) program and all IBC programs (ICS07, ICS26, GMP, etc.) -- the deployer's keypair is the upgrade authority for each
2. Deployer calls `initialize` on AM, setting the initial admin -- only the deployer (upgrade authority holder) can do this
3. Deployer calls `initialize` on each IBC program, passing the AM's address -- each program stores it for runtime role checks
4. Deployer uses `solana program set-upgrade-authority` (CLI) to transfer each managed program's upgrade authority from the deployer keypair to AM's per-program PDA (`["upgrade_authority", program_id]`)
5. Deployer sets the buffer authority to AM's PDA as well
6. From this point, the deployer keypair is no longer needed -- upgrades go through AM's role-gated `upgrade_program` instruction

See [`Test_ProgramUpgrade_Via_AccessManager`](../../../e2e/interchaintestv8/solana_upgrade_test.go#L69) (steps 1-5 of setup).

### 2. Upgrading an IBC program

1. Deployer (or anyone) writes the new bytecode to a buffer account and sets the buffer authority to AM's upgrade authority PDA
2. Admin (holder of `ADMIN_ROLE`) calls `upgrade_program` on AM, passing the target program and buffer

AM verifies the caller has `ADMIN_ROLE`, signs the BPF Loader `Upgrade` CPI with its upgrade authority PDA via `invoke_signed` and BPF Loader replaces the target program's bytecode.

See [`Test_ProgramUpgrade_Via_AccessManager`](../../../e2e/interchaintestv8/solana_upgrade_test.go#L69) (upgrade step) and [`Test_RevokeAdminRole`](../../../e2e/interchaintestv8/solana_upgrade_test.go#L274) (revoked admin cannot upgrade).

### 3. Migrating to a new Access Manager (AM-A -> AM-B)

There are two independent control planes to migrate:

**Upgrade authority** (who can replace program bytecode):

1. Deploy and initialize AM-B with its own admin
2. AM-A admin calls `propose_upgrade_authority_transfer` on AM-A for each target program, specifying AM-B's upgrade authority PDA as the new authority -- proposals for different programs can be batched in a single transaction
3. AM-B admin calls `claim_upgrade_authority` on AM-B for each target program -- AM-B CPIs into AM-A's `accept_upgrade_authority_transfer` signing with its own PDA (since only AM-B can `invoke_signed` with AM-B's PDA)

AM-A validates the pending transfer matches, then executes BPF Loader `set_authority` -- authority moves from AM-A's PDA to AM-B's PDA. See [`Test_AMtoAM_UpgradeAuthorityMigration`](../../../e2e/interchaintestv8/solana_upgrade_test.go#L693) and [`Test_BatchUpgradeAuthorityMigration`](../../../e2e/interchaintestv8/solana_upgrade_test.go#L1538).

**Runtime roles** (who can relay, pause, configure):

1. AM-A admin calls `propose_access_manager_transfer` on each IBC program (ICS07, ICS26, GMP, attestation) to propose repointing from AM-A to AM-B
2. AM-B admin calls `accept_access_manager_transfer` on each IBC program to finalize the switch

AM-B now controls both bytecode upgrades and runtime roles -- AM-A has no remaining authority. See [`Test_AccessManagerTransfer`](../../../e2e/interchaintestv8/solana_upgrade_test.go#L960).

> **Note:** IFT uses a different pattern (`admin: Pubkey` with two-step propose/accept transfer) and does not use access manager transfer.

## Security

#### CPI validation

`require_admin` checks the instructions sysvar to validate the caller. Direct calls and whitelisted CPI are allowed; unauthorized and nested CPI are rejected.

#### Sysvar address constraint

The instructions sysvar account has an `address` constraint preventing fake sysvar attacks (Wormhole-style).

#### Two-step authority transfer

Authority transfers require propose + accept, preventing irreversible mistakes from a single admin action.

#### Zero-address rejection

`propose_upgrade_authority_transfer` rejects `Pubkey::default()` to prevent irreversible lockout.

#### Self-transfer rejection

`propose_upgrade_authority_transfer` rejects transferring to the current upgrade authority PDA.

#### Concurrent transfer limit

The pending transfers vector is capped at 16 entries with a runtime guard, preventing unbounded growth that would corrupt account data.

#### Last admin protection

The last admin cannot be removed via `revoke_role`.

#### Per-program PDA scoping

Upgrade authority PDAs include the target program ID in their seeds, preventing cross-program authority reuse.

## Testing

### Unit and Integration Tests

```bash
just solana::build-solana-programs access-manager
cd ibc-solana && cargo test -p access-manager --lib --tests
```

The test suite includes Mollusk (SBF binary) unit tests and ProgramTest integration tests covering admin authorization, CPI rejection, fake sysvar attacks, wrong PDA derivation and zero-address rejection.

### E2E Tests

Tests are in [`solana_upgrade_test.go`](../../../e2e/interchaintestv8/solana_upgrade_test.go):
- [`Test_ProgramUpgrade_Via_AccessManager`](../../../e2e/interchaintestv8/solana_upgrade_test.go#L69) -- standard upgrade flow
- [`Test_RevokeAdminRole`](../../../e2e/interchaintestv8/solana_upgrade_test.go#L274) -- revoked admin cannot upgrade
- [`Test_TransferUpgradeAuthority`](../../../e2e/interchaintestv8/solana_upgrade_test.go#L409) -- two-step propose/accept authority transfer and migration verification
- [`Test_AMtoAM_UpgradeAuthorityMigration`](../../../e2e/interchaintestv8/solana_upgrade_test.go#L693) -- full AM-to-AM migration via propose + claim
- [`Test_BatchUpgradeAuthorityMigration`](../../../e2e/interchaintestv8/solana_upgrade_test.go#L1538) -- batch propose + claim for multiple programs
