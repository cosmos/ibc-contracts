#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$repo_root/e2e/scripts/router-env.sh"
bash "$repo_root/scripts/tests/tools/verify-ibc-storage-layout.sh"
state_root="$repo_root/.e2e-state"
forge_bin="${FORGE_BIN:-forge}"
cast_bin="${CAST_BIN:-cast}"
chain_a_rpc="${CHAIN_A_RPC_URL:-http://127.0.0.1:18545}"
chain_b_rpc="${CHAIN_B_RPC_URL:-http://127.0.0.1:28545}"
gas_estimate_multiplier="${E2E_GAS_ESTIMATE_MULTIPLIER:-200}"
resume_after_core="${E2E_RESUME_AFTER_CORE:-0}"

if [[ ! "$gas_estimate_multiplier" =~ ^[0-9]+$ || "$gas_estimate_multiplier" -eq 0 ]]; then
  echo "E2E_GAS_ESTIMATE_MULTIPLIER must be a positive integer percentage" >&2
  exit 64
fi
if [[ "$resume_after_core" != 0 && "$resume_after_core" != 1 ]]; then
  echo "E2E_RESUME_AFTER_CORE must be 0 or 1" >&2
  exit 64
fi

for chain in chain-a chain-b; do
  if [[ ! -s "$state_root/$chain/deployer.key" ]]; then
    echo "missing generated E2E deployer key for $chain" >&2
    exit 1
  fi
done

wait_for_rpc() {
  local rpc="$1"
  local label="$2"
  for _ in {1..120}; do
    if "$cast_bin" block-number --rpc-url "$rpc" >/dev/null 2>&1; then
      return
    fi
    sleep 1
  done
  echo "$label RPC did not become ready" >&2
  exit 1
}
wait_for_rpc "$chain_a_rpc" "chain A"
wait_for_rpc "$chain_b_rpc" "chain B"

deploy_core() {
  local chain="$1"
  local rpc="$2"
  local key
  key="$(tr -d '\r\n' <"$state_root/$chain/deployer.key")"
  (
    DEPLOYER_PRIVATE_KEY="$key" \
    E2E_OUTPUT_PATH="$state_root/$chain/core.json" \
      "$repo_root/e2e/scripts/forge.sh" script script/dev/DeployE2ECore.s.sol:DeployE2ECore \
        --rpc-url "$rpc" --broadcast --slow --non-interactive \
        --gas-estimate-multiplier "$gas_estimate_multiplier"
  )
}
select_bootstrap_height() {
  local rpc="$1"
  local router="$2"
  for _ in {1..120}; do
    local head candidate quantity code
    head="$($cast_bin block-number --rpc-url "$rpc")"
    if (( head > 1 )); then
      candidate=$((head - 1))
      quantity="$(printf '0x%x' "$candidate")"
      code="$($cast_bin rpc --rpc-url "$rpc" eth_getCode "$router" "$quantity" | tr -d '"')"
      if [[ "$code" != "0x" && "$code" != "0x0" ]]; then
        printf '%s\n' "$candidate"
        return
      fi
    fi
    sleep 1
  done
  echo "could not select a historical block containing the source Router" >&2
  exit 1
}

validate_resume_state() {
  local chain="$1"
  local core="$state_root/$chain/core.json"
  local bootstrap="$state_root/$chain/bootstrap.json"
  if ! jq -e '
      (.router | type == "string" and test("^0x[0-9a-fA-F]{40}$")) and
      (.verifierGateway | type == "string" and test("^0x[0-9a-fA-F]{40}$"))
    ' "$core" >/dev/null 2>&1; then
    echo "cannot resume: invalid or missing $core" >&2
    exit 1
  fi
  if ! jq -e '
      (.height | type == "number" and . > 0) and
      (.stateRoot | type == "string" and test("^0x[0-9a-fA-F]{64}$")) and
      (.blockTimestamp | type == "number") and
      (.validatorSetHash | type == "string" and test("^0x[0-9a-fA-F]{64}$"))
    ' "$bootstrap" >/dev/null 2>&1; then
    echo "cannot resume: invalid or missing $bootstrap" >&2
    exit 1
  fi
}

if [[ "$resume_after_core" == 0 ]]; then
  deploy_core chain-a "$chain_a_rpc"
  deploy_core chain-b "$chain_b_rpc"
else
  validate_resume_state chain-a
  validate_resume_state chain-b
fi

router_a="$(jq -r .router "$state_root/chain-a/core.json")"
router_b="$(jq -r .router "$state_root/chain-b/core.json")"

if [[ "$resume_after_core" == 0 ]]; then
  height_a="$(select_bootstrap_height "$chain_a_rpc" "$router_a")"
  height_b="$(select_bootstrap_height "$chain_b_rpc" "$router_b")"

  CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-2}" cargo run --quiet --locked \
    --manifest-path "$repo_root/Cargo.toml" -p prover-script --bin bootstrap-state -- \
    --source-rpc "$chain_a_rpc" --expected-source-chain-id 13371 --height "$height_a" \
    >"$state_root/chain-a/bootstrap.json"
  CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-2}" cargo run --quiet --locked \
    --manifest-path "$repo_root/Cargo.toml" -p prover-script --bin bootstrap-state -- \
    --source-rpc "$chain_b_rpc" --expected-source-chain-id 13372 --height "$height_b" \
    >"$state_root/chain-b/bootstrap.json"
else
  height_a="$(jq -r .height "$state_root/chain-a/bootstrap.json")"
  height_b="$(jq -r .height "$state_root/chain-b/bootstrap.json")"
fi

assert_historical_proof() {
  local rpc="$1"
  local router="$2"
  local height="$3"
  local quantity response
  quantity="$(printf '0x%x' "$height")"
  response="$(curl --fail-with-body --silent --show-error \
    --header 'content-type: application/json' \
    --data "$(jq -cn --arg router "$router" --arg height "$quantity" \
      '{jsonrpc:"2.0",id:1,method:"eth_getProof",params:[$router,[],$height]}')" \
    "$rpc")"
  if ! jq -e '.result != null and (has("error") | not)' >/dev/null <<<"$response"; then
    echo "historical state is unavailable for Router proof at height $height" >&2
    jq -c . <<<"$response" >&2 || true
    exit 1
  fi
}
assert_historical_proof "$chain_a_rpc" "$router_a" "$height_a"
assert_historical_proof "$chain_b_rpc" "$router_b" "$height_b"

deploy_light_client() {
  local destination_chain="$1"
  local destination_rpc="$2"
  local destination_router="$3"
  local source_router="$4"
  local source_bootstrap="$5"
  local expected_source_chain_id="$6"
  local gateway key
  gateway="$(jq -r .verifierGateway "$state_root/$destination_chain/core.json")"
  key="$(tr -d '\r\n' <"$state_root/$destination_chain/deployer.key")"

  export SOURCE_RPC_URL
  if [[ "$expected_source_chain_id" == 13371 ]]; then
    SOURCE_RPC_URL="$chain_a_rpc"
  else
    SOURCE_RPC_URL="$chain_b_rpc"
  fi
  export DESTINATION_RPC_URL="$destination_rpc"
  export EXPECTED_SOURCE_CHAIN_ID="$expected_source_chain_id"
  export SOURCE_VALIDATOR_MODE=header-qbft
  export SP1_PROOF_MODE=groth16
  export LOCAL_IBC_ROUTER="$destination_router"
  export SOURCE_IBC_ROUTER="$source_router"
  export SP1_VERIFIER_GATEWAY="$gateway"
  export SP1_GROTH16_SELECTOR=0x4388a21c
  export UPDATE_CLIENT_VKEY
  UPDATE_CLIENT_VKEY="$(jq -r '."update-client".programVKey' "$repo_root/artifacts/vkeys.json")"
  export MEMBERSHIP_VKEY
  MEMBERSHIP_VKEY="$(jq -r '.membership.programVKey' "$repo_root/artifacts/vkeys.json")"
  export NON_MEMBERSHIP_VKEY
  NON_MEMBERSHIP_VKEY="$(jq -r '."non-membership".programVKey' "$repo_root/artifacts/vkeys.json")"
  export INITIAL_HEIGHT
  INITIAL_HEIGHT="$(jq -r .height "$source_bootstrap")"
  export INITIAL_STATE_ROOT
  INITIAL_STATE_ROOT="$(jq -r .stateRoot "$source_bootstrap")"
  export INITIAL_BLOCK_TIMESTAMP
  INITIAL_BLOCK_TIMESTAMP="$(jq -r .blockTimestamp "$source_bootstrap")"
  export INITIAL_VALIDATOR_SET_HASH
  INITIAL_VALIDATOR_SET_HASH="$(jq -r .validatorSetHash "$source_bootstrap")"
  export TRUSTING_PERIOD_SECONDS="${E2E_TRUSTING_PERIOD_SECONDS:-3600}"
  export MAX_CLOCK_DRIFT_SECONDS="${E2E_MAX_CLOCK_DRIFT_SECONDS:-30}"
  export DEPLOYER_PRIVATE_KEY="$key"
  export COUNTERPARTY_CLIENT_ID=client-0
  export E2E_OUTPUT_PATH="$state_root/$destination_chain/light-client.json"

  CAST_BIN="$cast_bin" FORGE_BIN="$forge_bin" "$repo_root/deploy/validate.sh"
  (
    "$repo_root/e2e/scripts/forge.sh" script script/DeployE2ELightClient.s.sol:DeployE2ELightClient \
      --rpc-url "$destination_rpc" --broadcast --slow --non-interactive \
      --gas-estimate-multiplier "$gas_estimate_multiplier"
  )
}

deploy_light_client chain-a "$chain_a_rpc" "$router_a" "$router_b" \
  "$state_root/chain-b/bootstrap.json" 13372
deploy_light_client chain-b "$chain_b_rpc" "$router_b" "$router_a" \
  "$state_root/chain-a/bootstrap.json" 13371

if [[ "$(jq -r .clientId "$state_root/chain-a/light-client.json")" != client-0 \
  || "$(jq -r .clientId "$state_root/chain-b/light-client.json")" != client-0 ]]; then
  echo "the generic E2E deployment did not register client-0 on both chains" >&2
  exit 1
fi

jq -nS \
  --slurpfile chainA "$state_root/chain-a/core.json" \
  --slurpfile chainALightClient "$state_root/chain-a/light-client.json" \
  --slurpfile chainABootstrap "$state_root/chain-a/bootstrap.json" \
  --slurpfile chainB "$state_root/chain-b/core.json" \
  --slurpfile chainBLightClient "$state_root/chain-b/light-client.json" \
  --slurpfile chainBBootstrap "$state_root/chain-b/bootstrap.json" \
  '{schemaVersion:1,chainA:($chainA[0]+$chainALightClient[0]+{bootstrap:$chainABootstrap[0]}),chainB:($chainB[0]+$chainBLightClient[0]+{bootstrap:$chainBBootstrap[0]})}' \
  >"$state_root/deployments.json"

echo "deployed and bootstrapped both E2E directions"
