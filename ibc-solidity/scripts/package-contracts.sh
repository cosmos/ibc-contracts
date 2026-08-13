#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0

#
# Build and package pre-compiled solidity contract artifacts as a release tarball.
#
# Performs a clean build (bun install + forge build), then copies the full
# forge JSON (ABI + bytecode + metadata) for each shipped contract into
# release-artifacts/solidity-contracts/bytecode/, stamps a VERSION file and
# LICENSE, and produces solidity-contracts-<version>.tar.gz at the repo root.
#
# Requires `bun` and `forge` on PATH.
#
# Reads the release tag from the TAG_NAME environment variable (default: "dev").
# Usage:   TAG_NAME=solidity-v2.0.1 ./scripts/package-contracts.sh
# Example: ./scripts/package-contracts.sh                          # → dev
#          TAG_NAME=solidity-v2.0.1 ./scripts/package-contracts.sh # → tagged release

set -euo pipefail

TAG_NAME="${TAG_NAME:-dev}"
echo "📌 Version: $TAG_NAME"

# The Foundry project root (ibc-solidity/); the repository root is one level up.
PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$PROJECT_ROOT"

# Always clean up the staging tree on exit (success or failure)
trap 'rm -rf release-artifacts' EXIT

# Contracts to ship as full forge JSON (ABI + bytecode + metadata).
# Only concrete, deployable contracts are listed — interfaces, libraries,
# and abstract bases produce empty bytecode and are excluded.
contracts=(
  # Core IBC contracts
  ICS26Router
  ICS20Transfer
  ICS27GMP
  ICS27Account

  # Light clients
  AttestationLightClient
  SP1ICS07Tendermint
  ICS02PrecompileWrapper

  # IBC utilities
  IBCERC20
  Escrow
  RelayerHelper

  # IFT contracts
  IFTOwnable
  IFTAccessManaged
  CosmosIFTSendCallConstructor
  EVMIFTSendCallConstructor
  SolanaIFTSendCallConstructor

  # Reference / test
  ERC20
)

echo "🧹 Cleaning previous build output"
rm -rf cache out broadcast

echo "📦 Installing contract dependencies"
bun install --frozen-lockfile

echo "🔨 Building contracts"
forge build

staging="release-artifacts/solidity-contracts"
rm -rf "$staging"
mkdir -p "$staging/bytecode"

for c in "${contracts[@]}"; do
  src="out/${c}.sol/${c}.json"
  if [ ! -f "$src" ]; then
    echo "❌ Missing forge artifact: $src"
    exit 1
  fi
  cp "$src" "$staging/bytecode/${c}.json"
done

cp "$PROJECT_ROOT/../LICENSE.md" "$staging/"
echo "$TAG_NAME" > "$staging/TAG_NAME"

tarball="solidity-contracts-${TAG_NAME}.tar.gz"
tar -czvf "$tarball" -C release-artifacts solidity-contracts

echo "✅ Packaged: $tarball"
