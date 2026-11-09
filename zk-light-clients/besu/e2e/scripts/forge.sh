#!/usr/bin/env bash
set -euo pipefail
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$repo_root/e2e/scripts/router-env.sh"
cd "$repo_root/e2e/solidity"
# All imports, including the light client's, resolve to one interface definition.
# Paths are absolute so Foundry's working directory cannot change the dependency.
remappings="$(cat remappings.txt)"
export FOUNDRY_REMAPPINGS="$remappings
@ibc-contracts/=$E2E_ROUTER_DIR/ibc-solidity/contracts/
@uniswap/permit2/=$E2E_ROUTER_DIR/ibc-solidity/node_modules/@uniswap/permit2/"
export FOUNDRY_ALLOW_PATHS="$(printf '%s\n' "$repo_root" "$E2E_ROUTER_DIR" | jq -R . | jq -sc .)"
exec "${FORGE_BIN:-forge}" "$@"
