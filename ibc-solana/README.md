<!-- SPDX-License-Identifier: CC-BY-4.0 -->

# Solana IBC Programs

This directory contains the Solana implementation of IBC Eureka protocol, built using the Anchor framework. The programs enable trust-minimized interoperability between Solana and Cosmos SDK chains.

## Development Commands

Solana development recipes live in [`solana.just`](solana.just) and are exposed
through the root `justfile` as the `solana` module. Run them from the repository
root:

```bash
# List every Solana recipe
just solana

# Build, lint, and test
just solana::build-solana-programs
just solana::lint-solana
just solana::test-solana
```

The recipes themselves execute with `ibc-solana/` as their working directory.
Paths accepted as command arguments are documented relative to the repository
root.

## Dependency Graph

```mermaid
graph LR
    subgraph shared["Solana Packages"]
        types["solana-ibc-types"]
        sdk["solana-ibc-sdk"]
        constants["solana-ibc-constants"]
        proto["solana-ibc-proto"]
        borsh["solana-ibc-borsh-header"]
        ics25["ics25-handler"]
        gmp_types["solana-ibc-gmp-types"]
        macros["solana-ibc-macros"]
    end

    subgraph programs["Solana Programs"]
        am["access-manager"]
        ics07["ics07-tendermint"]
        ics26["ics26-router"]
        ics27["ics27-gmp"]
        ift_prog["ift"]
        att["attestation"]
    end

    idls["IDL files<br/><i>target/idl/*.json</i>"]

    subgraph tests["Integration Tests"]
        it_router["router"]
        it_gmp["gmp"]
        it_ift["ift"]
        it_att["attestation"]
        it_setup["programs + actors"]
    end

    subgraph proof_api["Proof API"]
        c2s["cosmos-to-solana"]
        e2s["eth-to-solana"]
        lib["proof-api-lib"]
    end

    subgraph operator["Operator"]
        op["operator"]
    end

    subgraph tm["Tendermint Light Client"]
        tm_update["update-client"]
        tm_membership["membership"]
        tm_misbehaviour["misbehaviour"]
    end

    types --> constants
    types --> proto
    types --> macros
    types --> gmp_types
    gmp_types --> proto

    am --> types
    ics07 --> am
    ics07 --> types
    ics07 --> ics25
    ics07 --> borsh
    ics07 --> constants
    ics07 --> tm_update
    ics07 --> tm_membership
    ics07 --> tm_misbehaviour
    ics26 --> am
    ics26 --> types
    ics26 --> ics25
    ics26 --> constants
    ics27 --> ics26
    ics27 --> am
    ics27 --> types
    ics27 --> proto
    ics27 --> macros
    ift_prog --> ics26
    ift_prog --> ics27
    ift_prog --> types
    ift_prog --> gmp_types
    ift_prog --> proto
    att --> am
    att --> types
    att --> ics25

    am -.- idls
    ics07 -.- idls
    ics26 -.- idls
    ics27 -.- idls
    ift_prog -.- idls
    att -.- idls
    idls -.-|build.rs codegen| sdk

    it_router -->|builders + types| sdk
    it_router -->|state| ics26
    it_gmp -->|builders + types| sdk
    it_gmp -->|protobuf| proto
    it_gmp -->|encoding, state| ics27
    it_ift -->|builders + types| sdk
    it_ift -->|protobuf| proto
    it_ift -->|state, helpers| ift_prog
    it_ift -->|encoding| ics27
    it_att -->|builders + types| sdk
    it_att -->|crypto, types| att
    it_setup -->|builders + types| sdk
    it_setup -->|program IDs| am

    c2s -->|builders + types| sdk
    c2s --> constants
    c2s --> proto
    c2s --> borsh
    c2s --> gmp_types
    e2s -->|builders + types| sdk
    e2s --> constants
    e2s --> proto
    e2s --> gmp_types
    lib -->|builders + types| sdk
    lib --> gmp_types
    lib --> constants
    lib --> proto
    op --> borsh

    style shared fill:#e0e7ff,stroke:#4f46e5,color:#1e1b4b
    style programs fill:#d1fae5,stroke:#059669,color:#064e3b
    style tests fill:#fef3c7,stroke:#d97706,color:#78350f
    style proof_api fill:#ffedd5,stroke:#ea580c,color:#7c2d12
    style operator fill:#e2e8f0,stroke:#475569,color:#1e293b
    style tm fill:#fce7f3,stroke:#db2777,color:#831843
    style idls fill:#fef9c3,stroke:#ca8a04,color:#713f12

    classDef sharedNode fill:#c7d2fe,stroke:#4f46e5,color:#1e1b4b
    classDef programNode fill:#a7f3d0,stroke:#059669,color:#064e3b
    classDef testNode fill:#fde68a,stroke:#d97706,color:#78350f
    classDef proofApiNode fill:#fed7aa,stroke:#ea580c,color:#7c2d12
    classDef operatorNode fill:#cbd5e1,stroke:#475569,color:#1e293b
    classDef tmNode fill:#fbcfe8,stroke:#db2777,color:#831843
    classDef idlNode fill:#fef08a,stroke:#ca8a04,color:#713f12

    class types,sdk,constants,proto,borsh,ics25,gmp_types,macros sharedNode
    class am,ics07,ics26,ics27,ift_prog,att programNode
    class it_router,it_gmp,it_ift,it_att,it_setup testNode
    class c2s,e2s,lib proofApiNode
    class op operatorNode
    class tm_update,tm_membership,tm_misbehaviour tmNode
    class idls idlNode

    linkStyle 0,1,2,3,4 stroke:#4f46e5
    linkStyle 5,6,7,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22,23,24,25,26,27,28,29,30 stroke:#059669
    linkStyle 31,32,33,34,35,36,37 stroke:#ca8a04,stroke-dasharray:5 5
    linkStyle 38,39,40,41,42,43,44,45,46,47,48,49,50 stroke:#d97706
    linkStyle 51,52,53,54,55,56,57,58,59,60,61,62,63 stroke:#ea580c
    linkStyle 64 stroke:#475569
```

### Generate Keypairs for a Cluster

```bash
just solana::generate-solana-keypairs <cluster>
```

`<cluster>` must be enabled in the `[clusters]` section of `Anchor.toml`.
`localnet` is enabled by default; uncomment or add `devnet`, `testnet`, or
`mainnet` before using those cluster names. Localnet keypairs are committed for
deterministic tests and cannot be regenerated.

Run namespaced recipes from the repository root. This creates the production
program keypairs in `solana-keypairs/<cluster>/`:

- `access_manager-keypair.json`
- `ics07_tendermint-keypair.json`
- `ics26_router-keypair.json`
- `ics27_gmp-keypair.json`

### Manual Keypair Generation

```bash
# Generate a program keypair
solana-keygen new -o solana-keypairs/<cluster>/<program>-keypair.json

# Display the public key (program ID)
solana-keygen pubkey solana-keypairs/<cluster>/<program>-keypair.json
```

**Security Notes:**

- Keypairs for `devnet`, `testnet`, and `mainnet` are **automatically gitignored**
- Only `localnet` keypairs are committed to the repository
- **Never commit production keypairs to version control**
- Store production keypairs securely

## Deployment

### Quick Deployment

After syncing keys and building the programs, deploy the existing artifacts to
devnet:

```bash
just solana::sync-solana-keys devnet
just solana::build-solana-programs
just solana::deploy-solana devnet
```

Together, these commands:

1. Sync the program IDs for the configured cluster.
2. Build all program artifacts and IDLs.
3. Deploy every `.so` artifact in `ibc-solana/target/deploy/`.
4. Reserve the configured upgrade space for each program.

### Step-by-Step Deployment

#### 1. Fund Your Deployer Wallet

```bash
# Devnet (free airdrops)
solana airdrop 5 solana-keypairs/devnet/deployer_wallet.json --url devnet

# Testnet
solana airdrop 2 solana-keypairs/testnet/deployer_wallet.json --url testnet

# Mainnet (fund with real SOL)
solana transfer <DEPLOYER_PUBKEY> <AMOUNT> --from <YOUR_WALLET> --url mainnet
```

#### 2. Build Programs

```bash
just solana::build-solana-programs
```

This compiles all programs and generates their IDLs.

#### 3. Deploy Programs

```bash
cd ibc-solana
anchor deploy --provider.cluster <cluster>
```

Or deploy individual programs:

```bash
anchor deploy -p access-manager --provider.cluster devnet
```

### Local Deployment (Testing)

For local development and testing:

```bash
# Terminal 1: Start local validator
solana-test-validator

# Terminal 2: Configure Solana CLI
solana config set --url localhost

# Terminal 3: Deploy
just solana::deploy-solana localnet
```

### Deployment Options

#### Deploy Specific Programs

```bash
anchor deploy -p access-manager --provider.cluster devnet
anchor deploy -p ics26-router --provider.cluster devnet
```

#### Upgrade Existing Programs

```bash
# Build new version
just solana::build-solana-programs

# Deploy upgrade
anchor upgrade target/deploy/ics26_router.so \
  --program-id <PROGRAM_ID> \
  --provider.cluster devnet
```

**Note:** After initial deployment, program upgrades should use the AccessManager's upgrade mechanism (see [Program Upgradability](#program-upgradability)).

### Verifying Deployment

```bash
# Check program account
solana program show <PROGRAM_ID> --url devnet

# Verify program executable
solana account <PROGRAM_ID> --url devnet

# Check deployer balance
solana balance solana-keypairs/devnet/deployer_wallet.json --url devnet
```

## Access Control & Roles

The AccessManager provides role-based access control across all IBC programs.

### Role Definitions

| Role ID    | Name                 | Purpose                                 |
| ---------- | -------------------- | --------------------------------------- |
| `0`        | `ADMIN_ROLE`         | Root administrator with all permissions |
| `1`        | `RELAYER_ROLE`       | Submit IBC packets and client updates   |
| `2`        | `PAUSER_ROLE`        | Pause operations during emergencies     |
| `3`        | `UNPAUSER_ROLE`      | Resume operations after emergency       |
| `6`        | `ID_CUSTOMIZER_ROLE` | Customize client and connection IDs     |
| `u64::MAX` | `PUBLIC_ROLE`        | Anyone (unrestricted access)            |

**Note:** Role IDs 4, 5 and 7 exist on Ethereum but are not used by any Solana program. The access manager does not restrict which IDs can be granted — consuming programs define and enforce role semantics.

### Role Management

#### Initialize AccessManager

Before granting roles, initialize the AccessManager with an initial admin:

```bash
just solana::initialize-access-manager [admin-pubkey] [cluster]
```

Example:

```bash
just solana::initialize-access-manager 8ntLtUdGwBaXfFPCrNis9MWsKMdEUYyonwuw7NQwhs5z localnet
```

This creates the AccessManager PDA and grants ADMIN_ROLE to the specified account.

#### Grant Role

Grant a role to an account:

```bash
just solana::grant-solana-role <role-id> [account-pubkey] [cluster]
```

Example (grant RELAYER_ROLE):

```bash
just solana::grant-solana-role 1 8ntLtUdGwBaXfFPCrNis9MWsKMdEUYyonwuw7NQwhs5z localnet
```

#### Revoke Role

Revoke a role from an account:

```bash
just solana::revoke-solana-role <role-id> [account-pubkey] [cluster]
```

Example:

```bash
just solana::revoke-solana-role 1 8ntLtUdGwBaXfFPCrNis9MWsKMdEUYyonwuw7NQwhs5z localnet
```

### Role Verification

Programs verify roles before executing sensitive operations:

```rust
use access_manager::helpers::require_role;

require_role(
    &access_manager_account,
    roles::RELAYER_ROLE,
    &authority_account,
    &instructions_sysvar,
    &program_id,
)?;
```

## Program Upgradability

Solana programs deployed via BPF Loader Upgradeable can be upgraded by the upgrade authority. We use AccessManager to manage upgrade permissions via role-based access control.

### Initial Setup

After deploying programs, configure upgrade authority management:

**Step 1: Initialize AccessManager (if not already done)**

```bash
just solana::initialize-access-manager [admin-pubkey] [cluster]
```

Example:

```bash
just solana::initialize-access-manager 8ntLtUdGwBaXfFPCrNis9MWsKMdEUYyonwuw7NQwhs5z localnet
```

**Step 2: Transfer upgrade authority to AccessManager PDA**

```bash
just solana::set-upgrade-authority <program-name> [cluster] [current-authority-keypair]
```

Example:

```bash
just solana::set-upgrade-authority ics26_router localnet "$HOME/.config/solana/id.json"
```

**Notes:**

- The upgrade authority PDA is automatically derived with seeds: `["upgrade_authority", program_id]`
- The third parameter (current authority keypair) is optional; it defaults to
  `solana-keypairs/<cluster>/deployer_wallet.json`.
- Use `--skip-new-upgrade-authority-signer-check` is automatically applied since PDAs cannot sign

### Performing Upgrades

Once upgrade authority is transferred to AccessManager, upgrades require ADMIN_ROLE.

#### Complete Upgrade (Recommended)

After building the updated program, execute the buffer creation and upgrade
instruction in one command:

```bash
just solana::build-solana-programs <program-name-with-hyphens>
just solana::upgrade-solana-program <program-name> [cluster] [upgrader-keypair-path]
```

Example:

```bash
just solana::upgrade-solana-program ics26_router devnet solana-keypairs/devnet/upgrader.json
```

This command automatically:

1. Reads the existing program artifact from `ibc-solana/target/deploy/`
2. Writes bytecode to a buffer account
3. Sets buffer authority to the upgrade authority PDA
4. Executes the AccessManager.upgrade_program instruction
5. Waits for confirmation

**Requirements:**

- Upgrader keypair must have ADMIN_ROLE
- Deployer wallet must have SOL for buffer creation
- Program must already be deployed with AccessManager as upgrade authority

#### Prepare Buffer Only

If you only want to prepare the buffer without executing the upgrade:

```bash
just solana::prepare-solana-upgrade <program-name> <upgrade-authority-pda> [cluster]
```

Example:

```bash
# Derive the upgrade authority PDA first (seeds: ["upgrade_authority", program_id])
# For ics26-router on devnet:
UPGRADE_AUTH_PDA="<DERIVED_PDA_ADDRESS>"

just solana::prepare-solana-upgrade ics26_router $UPGRADE_AUTH_PDA devnet
```

This command:

1. Builds the new program version
2. Writes bytecode to a buffer
3. Sets buffer authority to the upgrade authority PDA
4. Outputs the buffer address for manual upgrade execution

#### Option 2: Manual steps

**Step 1: Build new program version**

```bash
just solana::build-solana-programs
```

**Step 2: Write new bytecode to a buffer**

```bash
solana program write-buffer ibc-solana/target/deploy/<program>.so \
  --url <cluster> \
  --keypair solana-keypairs/<cluster>/deployer_wallet.json \
  --use-rpc
```

This outputs a buffer address like: `Buffer: <BUFFER_ADDRESS>`

**Step 3: Set buffer authority to match program upgrade authority**

```bash
solana program set-buffer-authority <BUFFER_ADDRESS> \
  --new-buffer-authority <UPGRADE_AUTHORITY_PDA> \
  --buffer-authority solana-keypairs/<cluster>/deployer_wallet.json \
  --keypair solana-keypairs/<cluster>/deployer_wallet.json \
  --url <cluster>
```

#### Final Step: Call AccessManager.upgrade_program

After preparing the buffer (using either option above), execute the upgrade instruction.

**Prerequisites:**

Build the `solana-ibc` CLI tool if not already built:

```bash
go build -o bin/solana-ibc ./tools/solana-ibc
```

**Using `solana-ibc` CLI:**

```bash
# First, derive the upgrade authority PDA
UPGRADE_AUTH_PDA=$(bin/solana-ibc upgrade derive-pda <ACCESS_MANAGER_PROGRAM_ID> <TARGET_PROGRAM_ID>)

# Get the program data address
PROGRAM_DATA_ADDR=$(solana program show <TARGET_PROGRAM_ID> --url <CLUSTER_URL> | grep "ProgramData Address" | awk '{print $3}')

# Execute the upgrade
bin/solana-ibc upgrade program \
  <CLUSTER_URL> \
  <UPGRADER_KEYPAIR_PATH> \
  <TARGET_PROGRAM_ID> \
  <BUFFER_ADDRESS> \
  <ACCESS_MANAGER_PROGRAM_ID> \
  $PROGRAM_DATA_ADDR
```

Example:

```bash
# For ics26-router on localnet
UPGRADE_AUTH_PDA=$(bin/solana-ibc upgrade derive-pda \
  4fMih2CidrXPeRx77kj3QcuBZwREYtxEbXjURUgadoe1 \
  FRGF7cthWUvDvAHMUARUHFycyUK2VDUtBchmkwrz7hgx)

PROGRAM_DATA_ADDR=$(solana program show FRGF7cthWUvDvAHMUARUHFycyUK2VDUtBchmkwrz7hgx \
  --url http://localhost:8899 | grep "ProgramData Address" | awk '{print $3}')

bin/solana-ibc upgrade program \
  http://localhost:8899 \
  solana-keypairs/localnet/deployer_wallet.json \
  FRGF7cthWUvDvAHMUARUHFycyUK2VDUtBchmkwrz7hgx \
  <BUFFER_ADDRESS_FROM_STEP_2> \
  4fMih2CidrXPeRx77kj3QcuBZwREYtxEbXjURUgadoe1 \
  $PROGRAM_DATA_ADDR
```

**Note:** The `just solana::upgrade-solana-program` command (documented above) automates all these steps, including buffer creation, PDA derivation, and upgrade execution. For programmatic integration, see [`solana_upgrade_test.go`](../e2e/interchaintestv8/solana_upgrade_test.go) for a complete Go implementation example.

### Upgrade Security

**Access Control:**

- Only accounts with `ADMIN_ROLE` can trigger upgrades
- CPI calls to `upgrade_program` are permitted only from whitelisted programs such as multisig wallets (via `require_admin`)
- Instructions sysvar verification prevents fake sysvar attacks

**Authority Verification:**

- Buffer authority must match program upgrade authority (BPF Loader requirement)
- Upgrade authority PDA seeds are validated via Anchor constraints
- Program account must be writable and executable

**Audit Trail:**

- All upgrades emit `ProgramUpgradedEvent` with:
  - Program ID
  - Upgrader public key
  - Timestamp

### Revoking Upgrade Permissions

To revoke upgrade capability, revoke ADMIN_ROLE from an account:

```bash
just solana::revoke-solana-role 0 <ADMIN_PUBKEY> <cluster>
```

Or using the `solana-ibc` CLI directly:

```bash
bin/solana-ibc access-manager revoke \
  <CLUSTER_URL> \
  <ADMIN_KEYPAIR> \
  0 \
  <ADMIN_PUBKEY_TO_REVOKE> \
  <ACCESS_MANAGER_PROGRAM_ID>
```

**Note:** Be careful not to revoke the last admin, as this would lock you out of admin operations.

## Development

### Building Programs

```bash
# Build all programs (generates all IDLs)
just solana::build-solana-programs

# Build specific program only (generates only its IDL)
just solana::build-solana-programs ics26-router

# Using anchor directly (from ibc-solana directory)
cd ibc-solana
anchor build
anchor build -p ics26-router
```

### Running Tests

```bash
# Solana unit tests
just solana::test-solana

# Solana integration tests (requires current artifacts in target/deploy)
just solana::test-solana-integration

# Specific program tests (requires the app being built)
cargo test --manifest-path ibc-solana/programs/ics26-router/Cargo.toml

# E2E tests
just test-e2e-solana Test_Deploy
just test-e2e-solana-gmp Test_GMPCounterFromCosmos
just test-e2e-solana-upgrade Test_ProgramUpgrade_Via_AccessManager
```

### Code Quality

```bash
# Format code
cd ibc-solana && cargo fmt --all

# Run linter
just solana::lint-solana
```

### IDL Generation

IDL (Interface Definition Language) files are automatically generated during build:

```bash
# Generate IDLs for all programs
just solana::build-solana-programs

# Generate IDL for specific program only
just solana::build-solana-programs ics26-router
```

IDL files are written to `target/idl/` and contain the program's interface definition for client libraries and tools.
