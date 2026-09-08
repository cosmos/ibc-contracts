# SPDX-License-Identifier: Apache-2.0

# Recipes for the Solana IBC implementation. Invoked from the repository
# root justfile as the `solana` module; recipes run with `ibc-solana/`
# as the working directory.

# Loads the repository root .env (the search walks up parent directories).
set dotenv-load

# Detect which Anchor command is available.
anchor_cmd := `command -v anchor-nix >/dev/null 2>&1 && echo "anchor-nix" || echo "anchor"`

# Helper function to run the solana-ibc CLI tool from the repository root.
solana_ibc := '''
  (cd ../tools/solana-ibc && go run . "$@")
'''

# List the recipes exposed by this module.
default:
  just --list solana

# Sync Solana program keypairs and update declare_id! macros
# Usage: just solana::sync-solana-keys [cluster]
# Example: just solana::sync-solana-keys devnet
[group('solana')]
sync-solana-keys cluster="localnet": (_validate-cluster cluster)
  #!/usr/bin/env bash
  set -euo pipefail

  echo "Syncing Solana program keys for cluster: {{cluster}}"

  # Validate cluster directory exists
  if [ ! -d "../solana-keypairs/{{cluster}}" ]; then
    echo "❌ Cluster directory not found: ../solana-keypairs/{{cluster}}"
    echo "   Available clusters: $(just solana::list-clusters)"
    exit 1
  fi

  # Check for keypairs (skip for localnet as it's tracked in git)
  if [ "{{cluster}}" != "localnet" ] && [ ! -f "../solana-keypairs/{{cluster}}/ics26_router-keypair.json" ]; then
    echo "❌ No keypairs found for cluster: {{cluster}}"
    echo "   Generate them first with: just solana::generate-solana-keypairs {{cluster}}"
    exit 1
  fi

  # Add [programs.{{cluster}}] section to Anchor.toml if missing
  ANCHOR_TOML="Anchor.toml"
  if ! grep -q "^\[programs\.{{cluster}}\]" "$ANCHOR_TOML"; then
    echo "📝 Adding [programs.{{cluster}}] section to Anchor.toml..."

    # Build section content without trailing newline
    SECTION="[programs.{{cluster}}]"
    for keypair in ../solana-keypairs/{{cluster}}/*-keypair.json; do
      [ -f "$keypair" ] || continue
      PROGRAM_NAME=$(basename "$keypair" -keypair.json)
      PROGRAM_ID=$(solana-keygen pubkey "$keypair")
      SECTION+=$'\n'"$PROGRAM_NAME = \"$PROGRAM_ID\""
    done

    # Insert after [programs.localnet] section with proper spacing
    awk -v section="$SECTION" '
      /^\[programs\.localnet\]/ { in_localnet=1; print; next }
      in_localnet && /^[[:space:]]*$/ { next }
      in_localnet && /^\[/ {
        print ""
        print section
        print ""
        in_localnet=0
      }
      { print }
      END {
        if (in_localnet) {
          print ""
          print section
          print ""
        }
      }
    ' "$ANCHOR_TOML" > "$ANCHOR_TOML.tmp" && mv "$ANCHOR_TOML.tmp" "$ANCHOR_TOML"
  fi

  # Copy keypairs to target/deploy
  mkdir -p target/deploy
  cp -f ../solana-keypairs/{{cluster}}/*-keypair.json target/deploy/ 2>/dev/null || true

  # Sync declare_id! macros
  echo "🦀 Using {{anchor_cmd}}"
  ({{anchor_cmd}} keys sync --provider.cluster {{cluster}})

  echo "✅ Keys synced for cluster: {{cluster}}"

# Build a Solana program instance under a different program ID (for tests)
# Usage: just solana::build-solana-test-instance <source-program> <instance-name>
# Example: just solana::build-solana-test-instance attestation test-attestation
[group('solana')]
build-solana-test-instance SOURCE INSTANCE:
  ../scripts/build-solana-test-instance.sh {{SOURCE}} {{INSTANCE}} {{anchor_cmd}}

# Build Solana test-only program artifacts
# Usage: just solana::build-solana-test-programs
[group('solana')]
build-solana-test-programs:
  echo "Building Solana test-only programs..."
  just solana::build-solana-test-instance attestation test-attestation
  echo "✅ Solana test program build complete"

# Build Solana Anchor programs
# Usage: just solana::build-solana-programs [program]
# Example: just solana::build-solana-programs              (builds all programs)
# Example: just solana::build-solana-programs ics26-router (builds only ics26-router)
[group('solana')]
build-solana-programs program="":
  #!/usr/bin/env bash
  set -euo pipefail

  if [ -z "{{program}}" ]; then
    echo "Building all programs..."
    echo "🦀 Using {{anchor_cmd}}"
    ({{anchor_cmd}} build)
    echo "✅ Build complete"
  else
    # Test instances are built via build-solana-test-programs, not anchor directly
    if [ "{{program}}" = "test-attestation" ]; then
      just solana::build-solana-test-programs
      exit 0
    fi

    echo "Building program: {{program}}"
    PROGRAM_DIR="programs/{{program}}"

    if [ ! -d "$PROGRAM_DIR" ]; then
      echo "❌ Program directory not found: $PROGRAM_DIR"
      echo "   Available programs:"
      ls -1 programs/ | grep -v "^\." || true
      exit 1
    fi

    echo "🦀 Using {{anchor_cmd}}"

    # Build specific program and generate its IDL
    ({{anchor_cmd}} build -- --manifest-path "$PWD/programs/{{program}}/Cargo.toml")

    echo "✅ Build complete for {{program}}"
  fi

# Deploy Solana Anchor programs to a specific cluster (default: localnet)
# Usage: just solana::deploy-solana [cluster] [max-len-multiplier]
# Example: just solana::deploy-solana devnet
# Example: just solana::deploy-solana localnet 3
# Use max-len-multiplier to reserve more space for future upgrades.
[group('solana')]
deploy-solana cluster="localnet" max_len_multiplier="2":
  #!/usr/bin/env bash
  set -euo pipefail

  echo "Deploying programs to {{cluster}}..."
  echo "🚀 Using {{anchor_cmd}}"
  echo "📏 Max length multiplier: {{max_len_multiplier}}x"

  # Get deployer wallet for localnet
  if [ "{{cluster}}" = "localnet" ]; then
    DEPLOYER_WALLET="$(pwd)/../solana-keypairs/{{cluster}}/deployer_wallet.json"
    WALLET_ARG="--provider.wallet $DEPLOYER_WALLET"
  else
    WALLET_ARG=""
  fi

  # Deploy each program individually with its own max-len
  for program_so in target/deploy/*.so; do
    if [ -f "$program_so" ]; then
      PROGRAM_NAME=$(basename "$program_so" .so)
      PROGRAM_SIZE=$(stat -f%z "$program_so" 2>/dev/null || stat -c%s "$program_so" 2>/dev/null)
      MAX_LEN=$((PROGRAM_SIZE * {{max_len_multiplier}}))

      echo "📦 Deploying $PROGRAM_NAME (size: $PROGRAM_SIZE bytes, max-len: $MAX_LEN bytes)"
      {{anchor_cmd}} deploy --provider.cluster {{cluster}} $WALLET_ARG -p "$PROGRAM_NAME" -- --max-len $MAX_LEN
    fi
  done

  echo "✅ Deployment complete for cluster: {{cluster}}"

# Full deployment: deploy programs, initialize access manager, set upgrade authorities, and grant upgrader role
# Usage: just solana::deploy-solana-full [cluster] [max-len-multiplier]
# Example: just solana::deploy-solana-full localnet
# Example: just solana::deploy-solana-full devnet 3
[group('solana')]
deploy-solana-full cluster="localnet" max_len_multiplier="2": (_validate-cluster cluster)
  #!/usr/bin/env bash
  set -euo pipefail

  echo "🚀 Starting full Solana deployment for cluster: {{cluster}}"
  echo ""

  # Step 1: Deploy all programs
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo "📦 Step 1/4: Deploying Solana programs"
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  just solana::deploy-solana {{cluster}} {{max_len_multiplier}}
  echo ""

  # Step 2: Initialize AccessManager
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo "🔑 Step 2/4: Initializing AccessManager"
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  just solana::initialize-access-manager "" {{cluster}}
  echo ""

  # Step 3: Set upgrade authority to AccessManager for all programs (except access_manager itself)
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo "🔐 Step 3/4: Setting upgrade authorities to AccessManager"
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

  PROGRAMS=(
    "ics26_router"
    "ics27_gmp"
    "ics07_tendermint"
    "test_ibc_app"
    "mock_ibc_app"
    "test_gmp_app"
    "mock_light_client"
  )

  for program in "${PROGRAMS[@]}"; do
    echo "Setting upgrade authority for $program..."
    just solana::set-upgrade-authority "$program" {{cluster}} || {
      echo "⚠️  Warning: Failed to set upgrade authority for $program"
    }
  done
  echo ""

  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo "✅ Full deployment complete for cluster: {{cluster}}"
  echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  echo ""
  echo "Summary:"
  echo "  ✓ All programs deployed"
  echo "  ✓ AccessManager initialized (deployer has ADMIN_ROLE)"
  echo "  ✓ Upgrade authorities set to AccessManager PDAs"
  echo ""
  echo "Next steps:"
  echo "  • Use 'just solana::upgrade-solana-program <program> {{cluster}}' to upgrade programs"
  echo "  • Use 'just solana::grant-solana-role <role-id> <account> {{cluster}}' to grant additional roles"

# Initialize AccessManager with an admin
# Usage: just solana::initialize-access-manager [admin-pubkey] [cluster]
# Example: just solana::initialize-access-manager
# Example: just solana::initialize-access-manager 8ntLtUdGwBaXfFPCrNis9MWsKMdEUYyonwuw7NQwhs5z
[group('solana')]
initialize-access-manager admin_pubkey="" cluster="localnet": (_validate-cluster cluster)
  #!/usr/bin/env bash
  set -euo pipefail

  # Get cluster URL from Anchor.toml
  CLUSTER_URL=$(just solana::get-cluster-url {{cluster}})

  # Get payer keypair (deployer for localnet, or specified payer)
  PAYER_KEYPAIR="../solana-keypairs/{{cluster}}/deployer_wallet.json"

  # Default admin to deployer if not specified
  if [ -z "{{admin_pubkey}}" ]; then
    ADMIN_PUBKEY=$(solana-keygen pubkey "$PAYER_KEYPAIR")
  else
    ADMIN_PUBKEY="{{admin_pubkey}}"
  fi

  if [ ! -f "$PAYER_KEYPAIR" ]; then
    echo "❌ Payer keypair not found: $PAYER_KEYPAIR"
    exit 1
  fi

  # Get access manager program ID
  ACCESS_MANAGER_ID=$(solana-keygen pubkey "../solana-keypairs/{{cluster}}/access_manager-keypair.json")

  echo "Initializing AccessManager"
  echo "Cluster: {{cluster}} ($CLUSTER_URL)"
  echo "Admin: $ADMIN_PUBKEY"
  echo "Access Manager: $ACCESS_MANAGER_ID"
  echo ""

  # Convert keypair to absolute path
  ABS_PAYER_KEYPAIR=$(cd "$(dirname "$PAYER_KEYPAIR")" && pwd)/$(basename "$PAYER_KEYPAIR")

  solana_ibc() {
    {{solana_ibc}}
  }
  solana_ibc access-manager initialize "$CLUSTER_URL" "$ABS_PAYER_KEYPAIR" "$ADMIN_PUBKEY" "$ACCESS_MANAGER_ID"

# Grant a role to an account
# Usage: just solana::grant-solana-role <role-id> [account-pubkey] [cluster]
# Example: just solana::grant-solana-role 8
# Example: just solana::grant-solana-role 8 Abc123...
[group('solana')]
grant-solana-role role_id account="" cluster="localnet": (_validate-cluster cluster)
  #!/usr/bin/env bash
  set -euo pipefail

  # Get cluster URL from Anchor.toml
  CLUSTER_URL=$(just solana::get-cluster-url {{cluster}})

  # Get admin keypair (deployer for localnet, or specified admin)
  ADMIN_KEYPAIR="../solana-keypairs/{{cluster}}/deployer_wallet.json"

  if [ ! -f "$ADMIN_KEYPAIR" ]; then
    echo "❌ Admin keypair not found: $ADMIN_KEYPAIR"
    exit 1
  fi

  # Default account to deployer if not specified
  if [ -z "{{account}}" ]; then
    ACCOUNT_PUBKEY=$(solana-keygen pubkey "$ADMIN_KEYPAIR")
  else
    ACCOUNT_PUBKEY="{{account}}"
  fi

  # Get access manager program ID
  ACCESS_MANAGER_ID=$(solana-keygen pubkey "../solana-keypairs/{{cluster}}/access_manager-keypair.json")

  echo "Granting role {{role_id}} to $ACCOUNT_PUBKEY"
  echo "Cluster: {{cluster}} ($CLUSTER_URL)"
  echo "Access Manager: $ACCESS_MANAGER_ID"
  echo ""

  # Convert keypair to absolute path
  ABS_ADMIN_KEYPAIR=$(cd "$(dirname "$ADMIN_KEYPAIR")" && pwd)/$(basename "$ADMIN_KEYPAIR")

  solana_ibc() {
    {{solana_ibc}}
  }
  solana_ibc access-manager grant "$CLUSTER_URL" "$ABS_ADMIN_KEYPAIR" "{{role_id}}" "$ACCOUNT_PUBKEY" "$ACCESS_MANAGER_ID"

# Set program upgrade authority to AccessManager PDA
# Usage: just solana::set-upgrade-authority <program-name> [cluster] [current-authority-keypair]
# Example: just solana::set-upgrade-authority ics26_router
# Example: just solana::set-upgrade-authority ics26_router localnet "$HOME/.config/solana/id.json"
[group('solana')]
set-upgrade-authority program_name cluster="localnet" current_authority="": (_validate-cluster cluster)
  #!/usr/bin/env bash
  set -euo pipefail

  # Get cluster URL from Anchor.toml
  CLUSTER_URL=$(just solana::get-cluster-url {{cluster}})

  # Get program ID from keypair
  PROGRAM_KEYPAIR="../solana-keypairs/{{cluster}}/{{program_name}}-keypair.json"
  if [ ! -f "$PROGRAM_KEYPAIR" ]; then
    echo "❌ Program keypair not found: $PROGRAM_KEYPAIR"
    exit 1
  fi
  PROGRAM_ID=$(solana-keygen pubkey "$PROGRAM_KEYPAIR")

  # Determine current authority keypair (default to deployer)
  if [ -n "{{current_authority}}" ]; then
    CURRENT_AUTHORITY="{{current_authority}}"
    # Namespaced recipes run from ibc-solana; interpret relative user input from the repository root.
    if [[ "$CURRENT_AUTHORITY" != /* ]]; then
      CURRENT_AUTHORITY="../$CURRENT_AUTHORITY"
    fi
  else
    CURRENT_AUTHORITY="../solana-keypairs/{{cluster}}/deployer_wallet.json"
    if [ ! -f "$CURRENT_AUTHORITY" ]; then
      echo "❌ Default authority keypair not found: $CURRENT_AUTHORITY"
      echo "   Please specify current authority explicitly"
      exit 1
    fi
  fi

  ACCESS_MANAGER_ID=$(solana-keygen pubkey "../solana-keypairs/{{cluster}}/access_manager-keypair.json")

  # Derive upgrade authority PDA
  solana_ibc() {
    {{solana_ibc}}
  }
  UPGRADE_AUTH_PDA=$(solana_ibc upgrade derive-pda "$ACCESS_MANAGER_ID" "$PROGRAM_ID")

  echo "Setting upgrade authority for program {{program_name}}"
  echo "Program ID: $PROGRAM_ID"
  echo "Cluster: {{cluster}} ($CLUSTER_URL)"
  echo "Current authority: $CURRENT_AUTHORITY"
  echo "New authority (PDA): $UPGRADE_AUTH_PDA"
  echo ""

  solana program set-upgrade-authority "$PROGRAM_ID" \
    --upgrade-authority "$CURRENT_AUTHORITY" \
    --new-upgrade-authority "$UPGRADE_AUTH_PDA" \
    --skip-new-upgrade-authority-signer-check \
    --url "$CLUSTER_URL"

  echo ""
  echo "✅ Upgrade authority set to AccessManager PDA"
  echo "   Program: {{program_name}}"
  echo "   Program ID: $PROGRAM_ID"
  echo "   Upgrade Authority: $UPGRADE_AUTH_PDA"

  echo "✅ Upgrade authority set successfully!"

# Revoke a role from an account
# Usage: just solana::revoke-solana-role <role-id> [account-pubkey] [cluster]
# Example: just solana::revoke-solana-role 8
# Example: just solana::revoke-solana-role 8 Abc123...
[group('solana')]
revoke-solana-role role_id account="" cluster="localnet": (_validate-cluster cluster)
  #!/usr/bin/env bash
  set -euo pipefail

  # Get cluster URL from Anchor.toml
  CLUSTER_URL=$(just solana::get-cluster-url {{cluster}})

  # Get admin keypair
  ADMIN_KEYPAIR="../solana-keypairs/{{cluster}}/deployer_wallet.json"

  if [ ! -f "$ADMIN_KEYPAIR" ]; then
    echo "❌ Admin keypair not found: $ADMIN_KEYPAIR"
    exit 1
  fi

  # Default account to deployer if not specified
  if [ -z "{{account}}" ]; then
    ACCOUNT_PUBKEY=$(solana-keygen pubkey "$ADMIN_KEYPAIR")
  else
    ACCOUNT_PUBKEY="{{account}}"
  fi

  # Get access manager program ID
  ACCESS_MANAGER_ID=$(solana-keygen pubkey "../solana-keypairs/{{cluster}}/access_manager-keypair.json")

  echo "Revoking role {{role_id}} from $ACCOUNT_PUBKEY"
  echo "Cluster: {{cluster}} ($CLUSTER_URL)"
  echo "Access Manager: $ACCESS_MANAGER_ID"
  echo ""

  # Convert keypair to absolute path
  ABS_ADMIN_KEYPAIR=$(cd "$(dirname "$ADMIN_KEYPAIR")" && pwd)/$(basename "$ADMIN_KEYPAIR")

  solana_ibc() {
    {{solana_ibc}}
  }
  solana_ibc access-manager revoke "$CLUSTER_URL" "$ABS_ADMIN_KEYPAIR" "{{role_id}}" "$ACCOUNT_PUBKEY" "$ACCESS_MANAGER_ID"

# Prepare program upgrade buffer (steps 1-3 of upgrade process)
# Usage: just solana::prepare-solana-upgrade <program-name> <upgrade-authority-pda> [cluster]
# Example: just solana::prepare-solana-upgrade ics26_router GzT...xyz
# Note: Requires deployer wallet funded with SOL
[group('solana')]
prepare-solana-upgrade program upgrade_authority_pda cluster="localnet": (_validate-cluster cluster) build-solana-programs
  #!/usr/bin/env bash
  set -euo pipefail

  PROGRAM_SO="target/deploy/{{program}}.so"
  DEPLOYER_KEYPAIR="../solana-keypairs/{{cluster}}/deployer_wallet.json"
  CLUSTER_URL=$(just solana::get-cluster-url {{cluster}})

  # Validate inputs
  if [ ! -f "$PROGRAM_SO" ]; then
    echo "❌ Program binary not found: $PROGRAM_SO"
    echo "   Run 'just solana::build-solana-programs' first"
    exit 1
  fi

  if [ ! -f "$DEPLOYER_KEYPAIR" ]; then
    echo "❌ Deployer keypair not found: $DEPLOYER_KEYPAIR"
    exit 1
  fi

  echo "📦 Preparing upgrade for program: {{program}}"
  echo "   Cluster: {{cluster}}"
  echo "   Upgrade Authority PDA: {{upgrade_authority_pda}}"
  echo ""

  # Step 2: Write program to buffer
  echo "Step 1/2: Writing program to buffer..."
  BUFFER_OUTPUT=$(solana program write-buffer "$PROGRAM_SO" \
    --url "$CLUSTER_URL" \
    --keypair "$DEPLOYER_KEYPAIR" \
    --use-rpc 2>&1)

  # Extract buffer address from output
  BUFFER_ADDRESS=$(echo "$BUFFER_OUTPUT" | grep -oP 'Buffer: \K[A-Za-z0-9]+' || true)

  if [ -z "$BUFFER_ADDRESS" ]; then
    echo "❌ Failed to create buffer"
    echo "$BUFFER_OUTPUT"
    exit 1
  fi

  echo "✅ Buffer created: $BUFFER_ADDRESS"
  echo ""

  # Step 3: Set buffer authority
  echo "Step 2/2: Setting buffer authority to upgrade authority PDA..."
  solana program set-buffer-authority "$BUFFER_ADDRESS" \
    --new-buffer-authority "{{upgrade_authority_pda}}" \
    --buffer-authority "$DEPLOYER_KEYPAIR" \
    --keypair "$DEPLOYER_KEYPAIR" \
    --url "$CLUSTER_URL"

  echo ""
  echo "✅ Upgrade buffer prepared successfully!"
  echo ""
  echo "📋 Next steps:"
  echo "   Buffer Address: $BUFFER_ADDRESS"
  echo "   Upgrade Authority: {{upgrade_authority_pda}}"
  echo ""
  echo "   To complete the upgrade, call AccessManager.upgrade_program with:"
  echo "   - buffer: $BUFFER_ADDRESS"
  echo "   - target_program: <PROGRAM_ID>"
  echo "   - authority: <ACCOUNT_WITH_ADMIN_ROLE>"
  echo ""
  echo "   See ../e2e/interchaintestv8/solana_upgrade_test.go for implementation example"

# Execute complete program upgrade (prepare buffer + execute upgrade instruction)
# Usage: just solana::upgrade-solana-program <program-name> [cluster] [upgrader-keypair]
# Example: just solana::upgrade-solana-program ics26_router
# Example: just solana::upgrade-solana-program ics26_router devnet solana-keypairs/devnet/admin.json
# Note: Requires ADMIN_ROLE granted to the upgrader keypair (defaults to deployer)
[group('solana')]
upgrade-solana-program program cluster="localnet" upgrader_keypair="": (_validate-cluster cluster)
  #!/usr/bin/env bash
  set -euo pipefail

  PROGRAM_SO="target/deploy/{{program}}.so"
  DEPLOYER_KEYPAIR="../solana-keypairs/{{cluster}}/deployer_wallet.json"

  # Default upgrader to deployer if not specified
  if [ -n "{{upgrader_keypair}}" ]; then
    UPGRADER_KEYPAIR="{{upgrader_keypair}}"
    # Namespaced recipes run from ibc-solana; interpret relative user input from the repository root.
    if [[ "$UPGRADER_KEYPAIR" != /* ]]; then
      UPGRADER_KEYPAIR="../$UPGRADER_KEYPAIR"
    fi
  else
    UPGRADER_KEYPAIR="$DEPLOYER_KEYPAIR"
  fi

  # Get cluster URL from Anchor.toml
  CLUSTER_URL=$(just solana::get-cluster-url {{cluster}})

  # Get program ID from keypair
  PROGRAM_KEYPAIR="../solana-keypairs/{{cluster}}/{{program}}-keypair.json"
  if [ ! -f "$PROGRAM_KEYPAIR" ]; then
    echo "❌ Program keypair not found: $PROGRAM_KEYPAIR"
    exit 1
  fi
  PROGRAM_ID=$(solana-keygen pubkey "$PROGRAM_KEYPAIR")

  # Get access-manager program ID
  ACCESS_MANAGER_ID=$(solana-keygen pubkey "../solana-keypairs/{{cluster}}/access_manager-keypair.json")

  # Derive upgrade authority PDA
  solana_ibc() {
    {{solana_ibc}}
  }
  UPGRADE_AUTH_PDA=$(solana_ibc upgrade derive-pda "$ACCESS_MANAGER_ID" "$PROGRAM_ID" 2>/dev/null || echo "")

  if [ -z "$UPGRADE_AUTH_PDA" ]; then
    echo "❌ Failed to derive upgrade authority PDA"
    exit 1
  fi

  echo "🔧 Starting program upgrade for: {{program}}"
  echo "   Program ID: $PROGRAM_ID"
  echo "   Cluster: {{cluster}} ($CLUSTER_URL)"
  echo "   Access Manager: $ACCESS_MANAGER_ID"
  echo "   Upgrade Authority PDA: $UPGRADE_AUTH_PDA"
  echo ""

  # Step 1-3: Prepare buffer
  echo "Step 1-3: Preparing upgrade buffer..."

  if ! BUFFER_OUTPUT=$(solana program write-buffer "$PROGRAM_SO" \
    --url "$CLUSTER_URL" \
    --keypair "$DEPLOYER_KEYPAIR" \
    --use-rpc 2>&1); then
    echo "❌ Failed to create buffer"
    echo "$BUFFER_OUTPUT"
    exit 1
  fi

  BUFFER_ADDRESS=$(echo "$BUFFER_OUTPUT" | grep -oP 'Buffer: \K[A-Za-z0-9]+' || true)

  if [ -z "$BUFFER_ADDRESS" ]; then
    echo "❌ Failed to extract buffer address from output"
    echo "$BUFFER_OUTPUT"
    exit 1
  fi

  echo "✅ Buffer created: $BUFFER_ADDRESS"

  # Set buffer authority to match upgrade authority PDA
  echo "Setting buffer authority to: $UPGRADE_AUTH_PDA"
  solana program set-buffer-authority "$BUFFER_ADDRESS" \
    --new-buffer-authority "$UPGRADE_AUTH_PDA" \
    --buffer-authority "$DEPLOYER_KEYPAIR" \
    --keypair "$DEPLOYER_KEYPAIR" \
    --url "$CLUSTER_URL"

  echo "✅ Buffer authority set to upgrade authority PDA"

  # Derive program data address
  PROGRAM_DATA_ADDR=$(solana program show "$PROGRAM_ID" --url "$CLUSTER_URL" | grep "ProgramData Address" | awk '{print $3}')

  if [ -z "$PROGRAM_DATA_ADDR" ]; then
    echo "❌ Failed to get program data address"
    exit 1
  fi

  echo "Program Data Address: $PROGRAM_DATA_ADDR"
  echo ""
  echo "Step 4: Executing upgrade instruction..."
  echo "(This requires ADMIN_ROLE on the upgrader keypair)"
  echo ""

  # Convert upgrader keypair to absolute path
  ABS_UPGRADER_KEYPAIR=$(cd "$(dirname "$UPGRADER_KEYPAIR")" && pwd)/$(basename "$UPGRADER_KEYPAIR")

  # Call the upgrade tool
  solana_ibc upgrade program \
    "$CLUSTER_URL" \
    "$ABS_UPGRADER_KEYPAIR" \
    "$PROGRAM_ID" \
    "$BUFFER_ADDRESS" \
    "$ACCESS_MANAGER_ID" \
    "$PROGRAM_DATA_ADDR"

  echo ""
  echo "✅ Program upgrade complete!"

# Generate Solana keypairs for a specific cluster
# Usage: just solana::generate-solana-keypairs <cluster>
# Example: just solana::generate-solana-keypairs devnet
[group('solana')]
generate-solana-keypairs cluster="localnet": (_validate-cluster cluster)
  #!/usr/bin/env bash
  set -euo pipefail

  # Cannot regenerate localnet keypairs (tracked in git for E2E tests)
  if [ "{{cluster}}" = "localnet" ]; then
    echo "❌ Cannot generate keypairs for localnet - they are tracked in git"
    echo "   Localnet keypairs are used for E2E tests and should not be regenerated"
    exit 1
  fi

  echo "Generating keypairs for cluster: {{cluster}}"
  mkdir -p ../solana-keypairs/{{cluster}}
  solana-keygen new --no-bip39-passphrase --force --outfile ../solana-keypairs/{{cluster}}/ics26_router-keypair.json
  solana-keygen new --no-bip39-passphrase --force --outfile ../solana-keypairs/{{cluster}}/ics07_tendermint-keypair.json
  solana-keygen new --no-bip39-passphrase --force --outfile ../solana-keypairs/{{cluster}}/ics27_gmp-keypair.json
  solana-keygen new --no-bip39-passphrase --force --outfile ../solana-keypairs/{{cluster}}/access_manager-keypair.json
  echo ""
  echo "✅ Keypairs generated in ../solana-keypairs/{{cluster}}/"
  echo "⚠️  IMPORTANT: Backup these keypairs securely! They are NOT tracked in git."
  echo ""
  echo "📋 Program IDs for {{cluster}}:"
  for keypair in ../solana-keypairs/{{cluster}}/*-keypair.json; do
    printf "   %-35s %s\n" "$(basename $keypair):" "$(solana-keygen pubkey $keypair)"
  done
  echo ""
  echo "Next step: just solana::build-solana-programs"

# Get cluster URL from Anchor.toml
[group('solana')]
get-cluster-url cluster="localnet": (_validate-cluster cluster)
  #!/usr/bin/env bash
  awk -F' = ' -v cluster="{{cluster}}" '
    /^\[clusters\]/ { in_clusters=1; next }
    in_clusters && /^\[/ { exit }
    in_clusters && $1 == cluster { gsub(/"/, "", $2); print $2; exit }
  ' Anchor.toml

# List available clusters from Anchor.toml
[group('solana')]
list-clusters:
  #!/usr/bin/env bash
  awk '
    /^\[clusters\]/ { in_clusters=1; next }
    in_clusters && /^\[/ { exit }
    in_clusters && /^[a-z]/ {
      split($0, parts, " = ")
      print parts[1]
    }
  ' Anchor.toml | tr '\n' ',' | sed 's/,$//'

# Validate cluster exists in Anchor.toml (internal helper recipe)
[private]
_validate-cluster cluster:
  #!/usr/bin/env bash
  if ! awk -F' = ' -v cluster="{{cluster}}" '
    /^\[clusters\]/ { in_clusters=1; next }
    in_clusters && /^\[/ { exit }
    in_clusters && $1 == cluster { found=1; exit }
    END { exit !found }
  ' Anchor.toml; then
    AVAILABLE=$(just solana::list-clusters)
    echo "❌ Unknown cluster: {{cluster}}" >&2
    echo "   Available clusters: $AVAILABLE" >&2
    exit 1
  fi

# Lint the Solana code using `cargo fmt` and `cargo clippy`.
# Pinned to the Rust version required by the Anchor version we use (Anchor 0.32 requires
# rustc >= 1.89.0). A floating `+nightly` regularly breaks on newly-added clippy lints, so
# we build lints against the same, deterministic toolchain Anchor targets.
[group('lint')]
lint-solana:
	@echo "Linting the Solana code..."
	cargo +1.89.0 fmt --all -- --check
	cargo +1.89.0 clippy --all-targets --all-features -- -D warnings

# Generate the types for interacting with SVM contracts using 'anchor-go'
[group('generate')]
generate-solana-types build="true":
	@if [ "{{build}}" = "true" ]; then just solana::build-solana-programs; fi
	just solana::generate-pda
	@echo "Generating SVM types..."
	# Core IBC apps
	rm -rf ../packages/go-anchor/ics07tendermint
	anchor-go --idl ./target/idl/ics07_tendermint.json --output ../packages/go-anchor/ics07tendermint --no-go-mod
	rm -rf ../packages/go-anchor/ics26router
	anchor-go --idl ./target/idl/ics26_router.json --output ../packages/go-anchor/ics26router --no-go-mod
	rm -rf ../packages/go-anchor/accessmanager
	anchor-go --idl ./target/idl/access_manager.json --output ../packages/go-anchor/accessmanager --no-go-mod
	rm -rf ../packages/go-anchor/ics27gmp
	anchor-go --idl ./target/idl/ics27_gmp.json --output ../packages/go-anchor/ics27gmp --no-go-mod
	rm -rf ../packages/go-anchor/attestation
	anchor-go --idl ./target/idl/attestation.json --output ../packages/go-anchor/attestation --no-go-mod
	rm -rf ../packages/go-anchor/ift
	anchor-go --idl ./target/idl/ift.json --output ../packages/go-anchor/ift --no-go-mod
	# Test apps
	rm -rf ../e2e/interchaintestv8/solana/go-anchor/testibcapp
	anchor-go --idl ./target/idl/test_ibc_app.json --output ../e2e/interchaintestv8/solana/go-anchor/testibcapp --no-go-mod
	rm -rf ../e2e/interchaintestv8/solana/go-anchor/mocklightclient
	anchor-go --idl ./target/idl/mock_light_client.json --output ../e2e/interchaintestv8/solana/go-anchor/mocklightclient --no-go-mod
	rm -rf ../e2e/interchaintestv8/solana/go-anchor/testgmpapp
	anchor-go --idl ./target/idl/test_gmp_app.json --output ../e2e/interchaintestv8/solana/go-anchor/testgmpapp --no-go-mod
	rm -rf ../e2e/interchaintestv8/solana/go-anchor/testcpiproxy
	anchor-go --idl ./target/idl/test_cpi_proxy.json --output ../e2e/interchaintestv8/solana/go-anchor/testcpiproxy --no-go-mod

# Generate Solana PDA helpers from Anchor IDL files
[group('generate')]
generate-pda:
	@echo "Generating Solana PDA helpers from Anchor IDL..."
	go run ../e2e/interchaintestv8/solana/generate-pdas/main.go \
		--idl-dir target/idl \
		--output ../e2e/interchaintestv8/solana/pda.go
	gofmt -w ../e2e/interchaintestv8/solana/pda.go
	@echo "✅ Generated ../e2e/interchaintestv8/solana/pda.go"

# Run the Solana Anchor tests
[group('test')]
test-anchor-solana *ARGS:
	@echo "Running Solana Client Anchor tests (anchor-nix preferred) ..."
	@if command -v anchor-nix >/dev/null 2>&1; then \
		echo "🦀 Using anchor-nix"; \
		(anchor-nix test {{ARGS}}); \
	else \
		echo "🦀 Using anchor"; \
		(anchor test {{ARGS}}); \
	fi

# Run Solana unit tests (mollusk + litesvm)
[group('test')]
test-solana *ARGS:
	@echo "Building and running Solana unit tests..."
	@echo "🦀 Using {{anchor_cmd}}"
	@if [ "{{anchor_cmd}}" = "anchor-nix" ]; then \
		(anchor-nix unit-test {{ARGS}}); \
	else \
		(anchor build) && \
		echo "✅ Build successful, running unit tests" && \
		(cargo test --workspace --exclude integration-tests {{ARGS}}); \
	fi

# Run Solana integration tests
[group('test')]
test-solana-integration *ARGS:
	@echo "Building and running Solana integration tests..."
	(cargo test -p integration-tests {{ARGS}})
