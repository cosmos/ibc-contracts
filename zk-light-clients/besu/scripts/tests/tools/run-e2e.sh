#!/usr/bin/env bash
set -euo pipefail

if [[ "${RUN_HEAVY_E2E:-0}" != 1 ]]; then
  echo "full two-Besu-chain E2E is a heavy validation; set RUN_HEAVY_E2E=1 explicitly" >&2
  exit 64
fi

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
source "$repo_root/e2e/scripts/router-env.sh"
# Fail before starting chains if the external integration workspace cannot build.
"$repo_root/e2e/scripts/forge.sh" build
compose=(docker compose --file "$repo_root/e2e/docker-compose.yml" --profile four-validator)

if [[ -e "$repo_root/.e2e-state" ]]; then
  echo "refusing to overwrite existing .e2e-state; remove it explicitly before a clean E2E run" >&2
  exit 1
fi

cleanup() {
  "${compose[@]}" down --volumes --remove-orphans >/dev/null 2>&1 || true
}
trap cleanup EXIT

E2E_VALIDATOR_COUNT=4 "$repo_root/e2e/scripts/generate-dev-networks.sh"
"${compose[@]}" up --detach
"$repo_root/e2e/scripts/deploy-all.sh"

E2E_DEPLOYMENTS="$repo_root/.e2e-state/deployments.json" \
RUN_BESU_E2E=1 \
  cargo test --locked --manifest-path "$repo_root/e2e/Cargo.toml" -- \
    --ignored --test-threads=1
