#!/usr/bin/env bash
set -euo pipefail

chain_a_rpc="${CHAIN_A_RPC_URL:-http://127.0.0.1:18545}"
chain_b_rpc="${CHAIN_B_RPC_URL:-http://127.0.0.1:28545}"

for command in curl jq; do
  if ! command -v "$command" >/dev/null 2>&1; then
    echo "required command is unavailable: $command" >&2
    exit 69
  fi
done

rpc_call() {
  local rpc="$1"
  local method="$2"
  local params="$3"
  local response
  response="$(curl --fail-with-body --silent --show-error \
    --connect-timeout 5 --max-time 30 \
    --header 'content-type: application/json' \
    --data "$(jq -cn --arg method "$method" --argjson params "$params" \
      '{jsonrpc:"2.0",id:1,method:$method,params:$params}')" \
    "$rpc")"
  if ! jq -e 'has("error") | not' >/dev/null <<<"$response"; then
    echo "$method failed at $rpc" >&2
    jq -c . <<<"$response" >&2 || true
    exit 1
  fi
  jq -c '.result' <<<"$response"
}

quantity_to_decimal() {
  local quantity="$1"
  quantity="${quantity#\"}"
  quantity="${quantity%\"}"
  if [[ ! "$quantity" =~ ^0x(0|[1-9a-fA-F][0-9a-fA-F]*)$ ]]; then
    echo "invalid JSON-RPC quantity: $quantity" >&2
    exit 1
  fi
  printf '%d\n' "$((quantity))"
}

check_chain() {
  local label="$1"
  local rpc="$2"
  local expected_chain_id="$3"
  local expected_validators="$4"
  local chain_id_hex validator_json validator_count block_json block_number

  chain_id_hex="$(rpc_call "$rpc" eth_chainId '[]')"
  if [[ "$(quantity_to_decimal "$chain_id_hex")" -ne "$expected_chain_id" ]]; then
    echo "$label chain ID does not equal $expected_chain_id" >&2
    exit 1
  fi

  validator_json="$(rpc_call "$rpc" qbft_getValidatorsByBlockNumber '["latest"]')"
  validator_count="$(jq -er 'if type == "array" then length else error("not an array") end' \
    <<<"$validator_json")"
  if [[ "$validator_count" -ne "$expected_validators" ]]; then
    echo "$label has $validator_count validators; expected $expected_validators" >&2
    exit 1
  fi

  block_json="$(rpc_call "$rpc" eth_getBlockByNumber '["latest",false]')"
  block_number="$(jq -er '.number' <<<"$block_json")"
  block_number="$(quantity_to_decimal "$block_number")"
  if ((block_number < 1)); then
    echo "$label has not produced a non-genesis block" >&2
    exit 1
  fi

  printf '%s chainId=%d validators=%d latestBlock=%d rpc=%s\n' \
    "$label" "$expected_chain_id" "$validator_count" "$block_number" "$rpc"
  jq -r '.[]' <<<"$validator_json" | sed "s/^/$label validator=/"
}

check_chain chainA "$chain_a_rpc" 13371 4
check_chain chainB "$chain_b_rpc" 13372 1

initial_a="$(quantity_to_decimal "$(rpc_call "$chain_a_rpc" eth_blockNumber '[]')")"
initial_b="$(quantity_to_decimal "$(rpc_call "$chain_b_rpc" eth_blockNumber '[]')")"
for _ in {1..120}; do
  current_a="$(quantity_to_decimal "$(rpc_call "$chain_a_rpc" eth_blockNumber '[]')")"
  current_b="$(quantity_to_decimal "$(rpc_call "$chain_b_rpc" eth_blockNumber '[]')")"
  if ((current_a > initial_a && current_b > initial_b)); then
    printf 'blockProductionCheck=PASS chainA=%d->%d chainB=%d->%d\n' \
      "$initial_a" "$current_a" "$initial_b" "$current_b"
    exit 0
  fi
  sleep 1
done

echo "both chains did not advance within 120 seconds" >&2
exit 1
