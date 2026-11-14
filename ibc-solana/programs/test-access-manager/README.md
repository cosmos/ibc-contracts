<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Test Access Manager

A second access-manager instance with a different `declare_id!` for testing AM-to-AM migration.

## Why a separate program?

Anchor's `declare_id!` bakes the program ID into the binary at compile time. PDA derivations use `crate::ID`, so deploying two AM instances requires two separate binaries with different program IDs. This program reuses all source files from `access-manager` via symlinks -- only `lib.rs` is unique.

## Structure

```
test-access-manager/
  src/
    lib.rs              -- unique: different declare_id!, test constants
    errors.rs           -> ../../access-manager/src/errors.rs (symlink)
    events.rs           -> ../../access-manager/src/events.rs (symlink)
    helpers.rs          -> ../../access-manager/src/helpers.rs (symlink)
    instructions.rs     -> ../../access-manager/src/instructions.rs (symlink)
    instructions/       -> ../../access-manager/src/instructions/ (symlink)
    state.rs            -> ../../access-manager/src/state.rs (symlink)
    test_utils.rs       -> ../../access-manager/src/test_utils.rs (symlink)
    types.rs            -> ../../access-manager/src/types.rs (symlink)
```

## Usage

This program is used in:
- **ProgramTest integration tests** (`claim_upgrade_authority.rs`) -- both AM binaries are loaded to test cross-AM CPI
- **E2E tests** (`solana_upgrade_test.go`) -- `Test_AMtoAM_UpgradeAuthorityMigration` deploys both AM instances on a live validator
