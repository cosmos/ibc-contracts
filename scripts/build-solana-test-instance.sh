#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0

#
# Build a Solana program instance under a different program ID.
#
# Creates a temporary copy of the source program, patches crate name,
# uses `anchor keys sync` to update declare_id!, builds the .so, and
# cleans up all artifacts except the final binary.
#
# Usage: ./scripts/build-solana-test-instance.sh <source-program> <instance-name> [anchor-cmd]
# Example: ./scripts/build-solana-test-instance.sh attestation test-attestation anchor-nix

set -euo pipefail

SOURCE="${1:?Usage: $0 <source-program> <instance-name> [anchor-cmd]}"
INSTANCE="${2:?Usage: $0 <source-program> <instance-name> [anchor-cmd]}"
ANCHOR_CMD="${3:-anchor}"

# Validate inputs: only allow alphanumeric characters and hyphens
if [[ ! "$SOURCE" =~ ^[a-zA-Z0-9-]+$ ]]; then
  echo "❌ Invalid source name (only alphanumeric and hyphens allowed): $SOURCE"
  exit 1
fi
if [[ ! "$INSTANCE" =~ ^[a-zA-Z0-9-]+$ ]]; then
  echo "❌ Invalid instance name (only alphanumeric and hyphens allowed): $INSTANCE"
  exit 1
fi
if [ "$SOURCE" = "$INSTANCE" ]; then
  echo "❌ Instance name must differ from source: $SOURCE"
  exit 1
fi

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
SOLANA_DIR="$REPO_ROOT/ibc-solana"
PROGRAMS_DIR="$SOLANA_DIR/programs"
SOURCE_DIR="$PROGRAMS_DIR/$SOURCE"
INSTANCE_DIR="$PROGRAMS_DIR/$INSTANCE"
INSTANCE_UNDERSCORE="${INSTANCE//-/_}"

KEYPAIR_SOURCE="$REPO_ROOT/solana-keypairs/localnet/${INSTANCE_UNDERSCORE}-keypair.json"
KEYPAIR_DEST="$SOLANA_DIR/target/deploy/${INSTANCE_UNDERSCORE}-keypair.json"

WORKSPACE_TOML="$SOLANA_DIR/Cargo.toml"
ANCHOR_TOML="$SOLANA_DIR/Anchor.toml"
LOCKFILE="$SOLANA_DIR/Cargo.lock"
LOCKFILE_BACKUP=""

if [ ! -d "$SOURCE_DIR" ]; then
  echo "❌ Source program not found: $SOURCE_DIR"
  exit 1
fi

if [ ! -f "$KEYPAIR_SOURCE" ]; then
  echo "❌ Keypair not found: $KEYPAIR_SOURCE"
  echo "   Generate one with: solana-keygen new --no-bip39-passphrase -o $KEYPAIR_SOURCE"
  exit 1
fi

PUBKEY="$(solana-keygen pubkey "$KEYPAIR_SOURCE")"
echo "🔑 Instance: $INSTANCE (ID: $PUBKEY)"

if [ -f "$LOCKFILE" ]; then
  LOCKFILE_BACKUP="$(mktemp "$SOLANA_DIR/.Cargo.lock.backup.XXXXXX")"
  cp "$LOCKFILE" "$LOCKFILE_BACKUP"
fi

cleanup() {
  echo "🧹 Cleaning up..."
  rm -rf "$INSTANCE_DIR"
  if [ -n "$LOCKFILE_BACKUP" ] && [ -f "$LOCKFILE_BACKUP" ]; then
    mv "$LOCKFILE_BACKUP" "$LOCKFILE"
  fi
  cd "$SOLANA_DIR"
  git checkout -- Cargo.toml Anchor.toml 2>/dev/null || true
  # Restore original source declare_id! if anchor keys sync modified it
  git checkout -- "programs/$SOURCE/src/lib.rs" 2>/dev/null || true
}
trap cleanup EXIT

# 1. Copy source program
echo "📋 Copying $SOURCE → $INSTANCE"
cp -r "$SOURCE_DIR" "$INSTANCE_DIR"

# 2. Patch Cargo.toml in the copy
sed -i.bak "s/^name.*=.*/name = \"$INSTANCE_UNDERSCORE\"/" "$INSTANCE_DIR/Cargo.toml"
# Patch [lib] name
sed -i.bak "s/^name.*=.*\"${SOURCE//-/_}\"/name = \"$INSTANCE_UNDERSCORE\"/" "$INSTANCE_DIR/Cargo.toml"
rm -f "$INSTANCE_DIR/Cargo.toml.bak"

# 3. Add instance to workspace members (skip if already present)
if ! grep -q "\"programs/$INSTANCE\"" "$WORKSPACE_TOML"; then
  sed -i.bak "s|\"programs/$SOURCE\"|\"programs/$SOURCE\",\n  \"programs/$INSTANCE\"|" "$WORKSPACE_TOML"
  rm -f "$WORKSPACE_TOML.bak"
fi

# 4. Add instance to Anchor.toml [programs.localnet]
#    Remove any stale entry first to avoid duplicates.
sed -i.bak "/^${INSTANCE_UNDERSCORE} = /d" "$ANCHOR_TOML"
rm -f "$ANCHOR_TOML.bak"
sed -i.bak "/^\[programs\.localnet\]/a\\
$INSTANCE_UNDERSCORE = \"$PUBKEY\"
" "$ANCHOR_TOML"
rm -f "$ANCHOR_TOML.bak"

# 5. Copy keypair to target/deploy
mkdir -p "$SOLANA_DIR/target/deploy"
cp -f "$KEYPAIR_SOURCE" "$KEYPAIR_DEST"

# 6. Sync declare_id! via anchor keys sync
echo "🔄 Syncing declare_id! for $INSTANCE_UNDERSCORE"
cd "$SOLANA_DIR"
$ANCHOR_CMD keys sync -p "$INSTANCE_UNDERSCORE" --provider.cluster localnet

# 7. Build (skip IDL — only the .so binary is needed)
echo "🔨 Building $INSTANCE"
$ANCHOR_CMD build --no-idl -- --manifest-path "$INSTANCE_DIR/Cargo.toml"

echo "✅ Built: target/deploy/${INSTANCE_UNDERSCORE}.so"
