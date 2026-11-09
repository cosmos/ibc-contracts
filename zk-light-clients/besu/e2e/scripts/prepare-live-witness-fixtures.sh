#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
state_root="$repo_root/.e2e-state"
deployments="$state_root/deployments.json"
forge_bin="${FORGE_BIN:-forge}"
cast_bin="${CAST_BIN:-cast}"
chain_a_rpc="${CHAIN_A_RPC_URL:-http://127.0.0.1:18545}"
chain_b_rpc="${CHAIN_B_RPC_URL:-http://127.0.0.1:28545}"
gas_estimate_multiplier="${E2E_GAS_ESTIMATE_MULTIPLIER:-200}"
fixture_root="$state_root/witness-fixtures"

if [[ ! -s "$deployments" ]]; then
  echo "missing E2E deployments: $deployments" >&2
  exit 1
fi
if [[ -e "$fixture_root" ]]; then
  echo "refusing to overwrite existing witness fixtures: $fixture_root" >&2
  exit 1
fi
if [[ ! "$gas_estimate_multiplier" =~ ^[0-9]+$ || "$gas_estimate_multiplier" -eq 0 ]]; then
  echo "E2E_GAS_ESTIMATE_MULTIPLIER must be a positive integer percentage" >&2
  exit 64
fi
mkdir -p "$fixture_root"

prepare_direction() {
  local name="$1"
  local source_json="$2"
  local destination_json="$3"
  local source_state="$4"
  local destination_state="$5"
  local source_rpc="$6"
  local destination_rpc="$7"
  local expected_source_chain_id="$8"
  local direction_root="$fixture_root/$name"
  mkdir -p "$direction_root"

  local source_router destination_router source_app destination_app gateway source_key destination_key
  source_router="$(jq -er ".$source_json.router" "$deployments")"
  destination_router="$(jq -er ".$destination_json.router" "$deployments")"
  source_app="$(jq -er ".$source_json.testApp" "$deployments")"
  destination_app="$(jq -er ".$destination_json.testApp" "$deployments" | tr '[:upper:]' '[:lower:]')"
  gateway="$(jq -er ".$destination_json.verifierGateway" "$deployments")"
  source_key="$(tr -d '\r\n' <"$state_root/$source_state/deployer.key")"
  destination_key="$(tr -d '\r\n' <"$state_root/$destination_state/deployer.key")"

  local timestamp timeout sequence receipt height path path_digest commitment
  timestamp="$("$cast_bin" block latest --rpc-url "$source_rpc" --json | jq -er .timestamp)"
  timestamp="$("$cast_bin" to-dec "$timestamp")"
  timeout="$((timestamp + 3600))"
  sequence="$("$cast_bin" call --rpc-url "$source_rpc" "$source_app" \
    'sendPacket(string,string,uint64,bytes)(uint64)' \
    client-0 "$destination_app" "$timeout" 0x6c6976652d7769746e657373 | cut -d ' ' -f 1)"
  receipt="$("$cast_bin" send --rpc-url "$source_rpc" --private-key "$source_key" --json \
    "$source_app" \
    'sendPacket(string,string,uint64,bytes)(uint64)' \
    client-0 "$destination_app" "$timeout" 0x6c6976652d7769746e657373)"
  height="$(jq -er .blockNumber <<<"$receipt")"
  height="$("$cast_bin" to-dec "$height")"
  path="$(printf '0x636c69656e742d3001%016x' "$sequence")"
  path_digest="$("$cast_bin" keccak "$path")"
  commitment="$("$cast_bin" call --rpc-url "$source_rpc" "$source_router" \
    'getCommitment(bytes32)(bytes32)' "$path_digest")"
  if [[ "$commitment" == "0x$(printf '0%.0s' {1..64})" ]]; then
    echo "$name packet commitment is unexpectedly absent" >&2
    exit 1
  fi

  CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-2}" cargo run --quiet --locked \
    --manifest-path "$repo_root/Cargo.toml" -p prover-script --bin bootstrap-state -- \
    --source-rpc "$source_rpc" --expected-source-chain-id "$expected_source_chain_id" \
    --height "$height" >"$direction_root/bootstrap.json"

  export SOURCE_RPC_URL="$source_rpc"
  export DESTINATION_RPC_URL="$destination_rpc"
  export EXPECTED_SOURCE_CHAIN_ID="$expected_source_chain_id"
  export SOURCE_VALIDATOR_MODE=header-qbft
  export SP1_PROOF_MODE=groth16
  export LOCAL_IBC_ROUTER="$destination_router"
  export SOURCE_IBC_ROUTER="$source_router"
  export SP1_VERIFIER_GATEWAY="$gateway"
  export SP1_GROTH16_SELECTOR=0x4388a21c
  export UPDATE_CLIENT_VKEY
  UPDATE_CLIENT_VKEY="$(jq -er '."update-client".programVKey' "$repo_root/artifacts/vkeys.json")"
  export MEMBERSHIP_VKEY
  MEMBERSHIP_VKEY="$(jq -er '.membership.programVKey' "$repo_root/artifacts/vkeys.json")"
  export NON_MEMBERSHIP_VKEY
  NON_MEMBERSHIP_VKEY="$(jq -er '."non-membership".programVKey' "$repo_root/artifacts/vkeys.json")"
  export INITIAL_HEIGHT
  INITIAL_HEIGHT="$(jq -er .height "$direction_root/bootstrap.json")"
  export INITIAL_STATE_ROOT
  INITIAL_STATE_ROOT="$(jq -er .stateRoot "$direction_root/bootstrap.json")"
  export INITIAL_BLOCK_TIMESTAMP
  INITIAL_BLOCK_TIMESTAMP="$(jq -er .blockTimestamp "$direction_root/bootstrap.json")"
  export INITIAL_VALIDATOR_SET_HASH
  INITIAL_VALIDATOR_SET_HASH="$(jq -er .validatorSetHash "$direction_root/bootstrap.json")"
  export TRUSTING_PERIOD_SECONDS="${E2E_TRUSTING_PERIOD_SECONDS:-3600}"
  export MAX_CLOCK_DRIFT_SECONDS="${E2E_MAX_CLOCK_DRIFT_SECONDS:-30}"
  export DEPLOYER_PRIVATE_KEY="$destination_key"
  export COUNTERPARTY_CLIENT_ID=client-0
  export E2E_OUTPUT_PATH="$direction_root/light-client.json"

  CAST_BIN="$cast_bin" FORGE_BIN="$forge_bin" "$repo_root/deploy/validate.sh"
  (
    "$repo_root/e2e/scripts/forge.sh" script script/DeployE2ELightClient.s.sol:DeployE2ELightClient \
      --rpc-url "$destination_rpc" --broadcast --slow --non-interactive \
      --gas-estimate-multiplier "$gas_estimate_multiplier"
  )
  if [[ "$(jq -er .clientId "$direction_root/light-client.json")" != client-1 ]]; then
    echo "$name diagnostic client was not registered as client-1" >&2
    exit 1
  fi

  jq -nS \
    --arg direction "$name" \
    --arg sourceRpc "$source_rpc" \
    --arg destinationRpc "$destination_rpc" \
    --arg sourceRouter "$source_router" \
    --arg destinationLightClient "$(jq -er .lightClient "$direction_root/light-client.json")" \
    --argjson expectedSourceChainId "$expected_source_chain_id" \
    --argjson proofHeight "$height" \
    --argjson sequence "$sequence" \
    --arg path "$path" \
    --arg expectedCommitment "$commitment" \
    '{schemaVersion:1,$direction,$sourceRpc,$destinationRpc,$sourceRouter,
      $destinationLightClient,$expectedSourceChainId,$proofHeight,$sequence,$path,
      $expectedCommitment}' >"$direction_root/fixture.json"
}

prepare_direction a-to-b chainA chainB chain-a chain-b \
  "$chain_a_rpc" "$chain_b_rpc" 13371
prepare_direction b-to-a chainB chainA chain-b chain-a \
  "$chain_b_rpc" "$chain_a_rpc" 13372

jq -nS \
  --slurpfile aToB "$fixture_root/a-to-b/fixture.json" \
  --slurpfile bToA "$fixture_root/b-to-a/fixture.json" \
  '{schemaVersion:1,aToB:$aToB[0],bToA:$bToA[0]}' >"$fixture_root/fixtures.json"
echo "prepared live membership/non-membership witnesses in $fixture_root"
