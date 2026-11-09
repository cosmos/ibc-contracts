#!/usr/bin/env bash
set -euo pipefail

if [[ "${RUN_E2E_SETUP:-0}" != 1 ]]; then
  echo "set RUN_E2E_SETUP=1 to stop the current E2E chains and create the 4x1 topology" >&2
  exit 64
fi

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$repo_root/e2e/scripts/router-env.sh"
"$repo_root/e2e/scripts/forge.sh" build
state_root="$repo_root/.e2e-state"
backup_root="$repo_root/.e2e-state-backups"
cast_bin="${CAST_BIN:-cast}"
forge_bin="${FORGE_BIN:-forge}"
chain_a_rpc="${CHAIN_A_RPC_URL:-http://127.0.0.1:18545}"
chain_b_rpc="${CHAIN_B_RPC_URL:-http://127.0.0.1:28545}"
trusting_period="${E2E_TRUSTING_PERIOD_SECONDS:-86400}"
compose=(docker compose --file "$repo_root/e2e/docker-compose.yml")

for command in docker jq curl openssl "$cast_bin" "$forge_bin" cargo; do
  if ! command -v "$command" >/dev/null 2>&1; then
    echo "required command is unavailable: $command" >&2
    exit 69
  fi
done
if [[ ! "$trusting_period" =~ ^[0-9]+$ ]]; then
  echo "E2E_TRUSTING_PERIOD_SECONDS must be a non-negative integer" >&2
  exit 64
fi
if pgrep -f '[p]rover-script prove-' >/dev/null 2>&1; then
  echo "an SP1 proof process is running; stop or finish it before replacing the chains" >&2
  exit 1
fi

cd "$repo_root"
"${compose[@]}" \
  --profile four-validator \
  --profile chain-a-four-validator \
  --profile chain-b-four-validator \
  down --remove-orphans

if [[ -e "$state_root" ]]; then
  backup="$backup_root/$(date -u +%Y%m%dT%H%M%SZ)-$$"
  mkdir -p "$backup_root"
  mv "$state_root" "$backup"
  echo "archived prior disposable E2E state at $backup"
fi

CAST_BIN="$cast_bin" \
E2E_CHAIN_A_VALIDATOR_COUNT=4 \
E2E_CHAIN_B_VALIDATOR_COUNT=1 \
  "$repo_root/e2e/scripts/generate-dev-networks.sh"

"${compose[@]}" --profile chain-a-four-validator up --detach
CHAIN_A_RPC_URL="$chain_a_rpc" CHAIN_B_RPC_URL="$chain_b_rpc" \
  "$repo_root/e2e/scripts/check-asymmetric-4x1.sh"

CAST_BIN="$cast_bin" \
FORGE_BIN="$forge_bin" \
CHAIN_A_RPC_URL="$chain_a_rpc" \
CHAIN_B_RPC_URL="$chain_b_rpc" \
E2E_TRUSTING_PERIOD_SECONDS="$trusting_period" \
E2E_GAS_ESTIMATE_MULTIPLIER="${E2E_GAS_ESTIMATE_MULTIPLIER:-200}" \
CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-2}" \
  "$repo_root/e2e/scripts/deploy-all.sh"

if ! jq -e '
  .schemaVersion == 1 and
  (.chainA.lightClient | type == "string") and
  (.chainB.lightClient | type == "string")
' "$state_root/deployments.json" >/dev/null; then
  echo "deployment manifest is missing or malformed" >&2
  exit 1
fi

chain_a_client="$(jq -er '.chainA.lightClient' "$state_root/deployments.json")"
chain_b_client="$(jq -er '.chainB.lightClient' "$state_root/deployments.json")"
chain_a_period="$($cast_bin call --rpc-url "$chain_a_rpc" "$chain_a_client" \
  'trustingPeriodSeconds()(uint64)' | awk '{print $1}')"
chain_b_period="$($cast_bin call --rpc-url "$chain_b_rpc" "$chain_b_client" \
  'trustingPeriodSeconds()(uint64)' | awk '{print $1}')"
if [[ "$chain_a_period" -ne "$trusting_period" || "$chain_b_period" -ne "$trusting_period" ]]; then
  echo "deployed light-client trusting period does not match $trusting_period" >&2
  exit 1
fi

echo "asymmetric E2E setup complete: chain A=4 validators, chain B=1 validator"
echo "deployments=$state_root/deployments.json trustingPeriodSeconds=$trusting_period"
