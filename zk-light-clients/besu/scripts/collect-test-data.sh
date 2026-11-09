#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "usage: $0 qbft RPC_URL FROM_HEIGHT TO_HEIGHT OUTPUT_DIR" >&2
  echo "       $0 mpt RPC_URL ROUTER_ADDRESS BLOCK_HEIGHT SLOT_FILE OUTPUT_DIR" >&2
  echo "SLOT_FILE contains one 0x-prefixed bytes32 storage key per line" >&2
  exit 64
}
mode="${1:-}"
case "$mode" in qbft|mpt) shift ;; *) usage ;; esac

rpc_envelope() {
  local method="$1"
  local params="$2"
  local response
  response="$(curl --fail-with-body --silent --show-error \
    --header 'content-type: application/json' \
    --data "$(jq -cn --arg method "$method" --argjson params "$params" \
      '{jsonrpc:"2.0",id:1,method:$method,params:$params}')" \
    "$rpc_url")"
  if ! jq -e 'type == "object" and .jsonrpc == "2.0" and .id == 1 and has("result") and (has("error") | not)' \
      >/dev/null <<<"$response"; then
    if [[ "$mode" == mpt ]] && jq -er '.error.message // empty' <<<"$response" | \
        tr '[:upper:]' '[:lower:]' | grep -E 'historical state|missing trie node|pruned|archive' >/dev/null; then
      echo "historical state is unavailable at block $quantity" >&2
    else
      echo "RPC returned an error or malformed envelope for $method" >&2
      if [[ "$mode" == qbft ]]; then jq -c . <<<"$response" >&2 || true; fi
    fi
    exit 1
  fi
  printf '%s\n' "$response"
}

rpc() {
  rpc_envelope "$1" "$2" | jq -c '.result'
}

collect_qbft() {
  [[ $# -eq 4 ]] || usage
  rpc_url="$1"
  from_height="$2"
  to_height="$3"
  output_dir="$4"

  if [[ ! "$rpc_url" =~ ^https?://[^[:space:]]+$ ]]; then
    echo "RPC_URL must be an http(s) URL" >&2
    exit 64
  fi
  for value in "$from_height" "$to_height"; do
    if [[ ! "$value" =~ ^[0-9]+$ ]]; then
      echo "heights must be unsigned decimal integers" >&2
      exit 64
    fi
  done
  if (( from_height > to_height )); then
    echo "FROM_HEIGHT must not exceed TO_HEIGHT" >&2
    exit 64
  fi

  mkdir -p "$output_dir"

  chain_id="$(rpc eth_chainId '[]')"
  client_version="$(rpc web3_clientVersion '[]')"

  for ((height = from_height; height <= to_height; height++)); do
    quantity="$(printf '0x%x' "$height")"
    response="$(rpc_envelope eth_getBlockByNumber \
      "$(jq -cn --arg height "$quantity" '[$height,false]')")"
    result="$(jq -c '.result' <<<"$response")"
    if [[ "$result" == "null" ]]; then
      echo "block $height is unavailable" >&2
      exit 1
    fi
    if ! jq -e '
        ["parentHash","sha3Uncles","miner","stateRoot","transactionsRoot","receiptsRoot",
         "logsBloom","difficulty","number","gasLimit","gasUsed","timestamp","extraData",
         "mixHash","nonce"] as $required |
        all($required[]; (. as $field | $result | has($field)))
      ' --argjson result "$result" >/dev/null <<<"{}"; then
      echo "block $height is missing a required header field" >&2
      exit 1
    fi
    returned="$(jq -r '.number // empty' <<<"$result")"
    if [[ "$(printf '%s' "$returned" | tr '[:upper:]' '[:lower:]')" != "$(printf '%s' "$quantity" | tr '[:upper:]' '[:lower:]')" ]]; then
      echo "RPC returned block $returned for requested height $quantity" >&2
      exit 1
    fi
    vector_dir="$output_dir/block-$height"
    mkdir -p "$vector_dir"
    printf '%s\n' "$response" >"$vector_dir/raw-rpc.json"
    jq -S . <<<"$result" >"$vector_dir/normalized.json"
    jq -nS \
      --arg testDataId "qbft-block-$height" \
      --arg sourceReference "chainId=$chain_id block=$quantity hash=$(jq -r '.hash // ""' <<<"$result")" \
      --arg number "$returned" \
      --arg stateRoot "$(jq -r '.stateRoot // ""' <<<"$result")" \
      --arg extraData "$(jq -r '.extraData // ""' <<<"$result")" \
      --arg blockHash "$(jq -r '.hash // ""' <<<"$result")" \
      '{
        testDataId:$testDataId,
        testDataVersion:1,
        sourceType:"local-besu",
        sourceReference:$sourceReference,
        inputFiles:["raw-rpc.json","normalized.json"],
        expectedSuccess:true,
        expectedDecodedFields:{number:$number,stateRoot:$stateRoot,extraData:$extraData},
        expectedHashes:{blockHash:$blockHash},
        expectedPublicValuesHex:null,
        expectedError:null
      }' >"$vector_dir/manifest.json"
  done

  jq -nS \
    --argjson chainId "$chain_id" \
    --argjson clientVersion "$client_version" \
    --argjson fromHeight "$from_height" \
    --argjson toHeight "$to_height" \
    '{collectionSchemaVersion:1,chainId:$chainId,clientVersion:$clientVersion,fromHeight:$fromHeight,toHeight:$toHeight}' \
    >"$output_dir/manifest.json"

  echo "collected canonical QBFT headers in $output_dir"
}

collect_mpt() {
  [[ $# -eq 5 ]] || usage
  rpc_url="$1"
  router_address="$2"
  block_height="$3"
  slot_file="$4"
  output_dir="$5"

  if [[ ! "$rpc_url" =~ ^https?://[^[:space:]]+$ ]]; then
    echo "RPC_URL must be an http(s) URL" >&2
    exit 64
  fi
  if [[ ! "$router_address" =~ ^0x[0-9a-fA-F]{40}$ \
    || "$router_address" == 0x0000000000000000000000000000000000000000 ]]; then
    echo "ROUTER_ADDRESS must be a non-zero EVM address" >&2
    exit 64
  fi
  if [[ ! "$block_height" =~ ^[0-9]+$ ]]; then
    echo "BLOCK_HEIGHT must be an unsigned decimal integer" >&2
    exit 64
  fi
  if [[ ! -f "$slot_file" ]]; then
    echo "SLOT_FILE does not exist" >&2
    exit 66
  fi

  slots=()
  while IFS= read -r slot || [[ -n "$slot" ]]; do
    [[ "$slot" =~ ^[[:space:]]*$ ]] || slots+=("$slot")
  done <"$slot_file"
  if (( ${#slots[@]} == 0 )); then
    echo "SLOT_FILE must contain at least one storage key" >&2
    exit 64
  fi
  seen=""
  for slot in "${slots[@]}"; do
    if [[ ! "$slot" =~ ^0x[0-9a-fA-F]{64}$ ]]; then
      echo "invalid storage key: $slot" >&2
      exit 64
    fi
    normalized="$(printf '%s' "$slot" | tr '[:upper:]' '[:lower:]')"
    if [[ "$seen" == *"$normalized "* ]]; then
      echo "duplicate storage key: $slot" >&2
      exit 64
    fi
    seen+="$normalized "
  done

  mkdir -p "$output_dir"
  quantity="$(printf '0x%x' "$block_height")"
  chain_id=""

  chain_id="$(rpc eth_chainId '[]')"
  for index in "${!slots[@]}"; do
    slot="${slots[$index]}"
    response="$(rpc_envelope eth_getProof \
      "$(jq -cn --arg address "$router_address" --arg slot "$slot" --arg height "$quantity" \
        '[$address,[$slot],$height]')")"
    result="$(jq -c '.result' <<<"$response")"
    if [[ "$result" == "null" ]]; then
      echo "eth_getProof returned null for storage key $slot" >&2
      exit 1
    fi
    if [[ "$(jq -r '.address // empty' <<<"$result" | tr '[:upper:]' '[:lower:]')" \
      != "$(printf '%s' "$router_address" | tr '[:upper:]' '[:lower:]')" ]]; then
      echo "eth_getProof returned a different account address" >&2
      exit 1
    fi
    if [[ "$(jq '.storageProof | length' <<<"$result")" != 1 ]]; then
      echo "eth_getProof must return exactly one storage proof" >&2
      exit 1
    fi
    returned_key="$(jq -r '.storageProof[0].key // empty' <<<"$result")"
    if [[ "$(printf '%s' "$returned_key" | tr '[:upper:]' '[:lower:]')" != "$(printf '%s' "$slot" | tr '[:upper:]' '[:lower:]')" ]]; then
      echo "eth_getProof returned a different storage key" >&2
      exit 1
    fi
    vector_dir="$output_dir/proof-$index"
    mkdir -p "$vector_dir"
    printf '%s\n' "$response" >"$vector_dir/raw-rpc.json"
    jq -S . <<<"$result" >"$vector_dir/normalized.json"
    jq -nS \
      --arg testDataId "mpt-proof-$block_height-$index" \
      --arg sourceReference "chainId=$chain_id block=$quantity router=$(printf '%s' "$router_address" | tr '[:upper:]' '[:lower:]') slot=$(printf '%s' "$slot" | tr '[:upper:]' '[:lower:]')" \
      --arg address "$(jq -r '.address' <<<"$result")" \
      --arg key "$returned_key" \
      --arg value "$(jq -r '.storageProof[0].value // ""' <<<"$result")" \
      --arg storageHash "$(jq -r '.storageHash // ""' <<<"$result")" \
      '{
        testDataId:$testDataId,
        testDataVersion:1,
        sourceType:"local-besu",
        sourceReference:$sourceReference,
        inputFiles:["raw-rpc.json","normalized.json"],
        expectedSuccess:true,
        expectedDecodedFields:{address:$address,key:$key,value:$value},
        expectedHashes:{storageRoot:$storageHash},
        expectedPublicValuesHex:null,
        expectedError:null
      }' >"$vector_dir/manifest.json"
  done

  jq -nS \
    --argjson chainId "$chain_id" \
    --arg routerAddress "$(printf '%s' "$router_address" | tr '[:upper:]' '[:lower:]')" \
    --argjson blockHeight "$block_height" \
    --arg blockQuantity "$quantity" \
    --argjson proofCount "${#slots[@]}" \
    '{collectionSchemaVersion:1,chainId:$chainId,routerAddress:$routerAddress,blockHeight:$blockHeight,blockQuantity:$blockQuantity,proofCount:$proofCount}' \
    >"$output_dir/manifest.json"

  echo "collected canonical EIP-1186 proofs in $output_dir"
}

case "$mode" in
  qbft) collect_qbft "$@" ;;
  mpt) collect_mpt "$@" ;;
esac
