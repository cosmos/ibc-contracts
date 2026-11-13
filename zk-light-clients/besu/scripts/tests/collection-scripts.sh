#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
fixture_dir="$(mktemp -d)"
trap 'rm -rf "$fixture_dir"' EXIT

cat >"$fixture_dir/curl" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
request=""
while (( $# > 0 )); do
  if [[ "$1" == "--data" ]]; then
    request="$2"
    shift 2
  else
    shift
  fi
done
method="$(jq -r .method <<<"$request")"
if [[ -n "${MOCK_RPC_ERROR:-}" ]]; then
  jq -cn --arg message "$MOCK_RPC_ERROR" '{jsonrpc:"2.0",id:1,error:{code:-32000,message:$message}}'
  exit 0
fi
if [[ "${MOCK_BAD_ENVELOPE:-0}" == 1 ]]; then
  printf '{"result":null}\n'
  exit 0
fi
case "$method" in
  eth_chainId)
    jq -cn '{jsonrpc:"2.0",id:1,result:"0x539"}'
    ;;
  web3_clientVersion)
    jq -cn '{jsonrpc:"2.0",id:1,result:"Besu/v26.5.0"}'
    ;;
  eth_getBlockByNumber)
    height="$(jq -r '.params[0]' <<<"$request")"
    jq -cn --arg height "$height" \
      '{jsonrpc:"2.0",id:1,result:{
        number:$height,hash:("0x"+("11"*32)),parentHash:("0x"+("12"*32)),
        sha3Uncles:("0x"+("13"*32)),miner:("0x"+("14"*20)),
        stateRoot:("0x"+("15"*32)),transactionsRoot:("0x"+("16"*32)),
        receiptsRoot:("0x"+("17"*32)),logsBloom:("0x"+("00"*256)),difficulty:"0x1",
        gasLimit:"0x1",gasUsed:"0x0",timestamp:"0x1",extraData:"0xc0",
        mixHash:("0x"+("18"*32)),nonce:("0x"+("00"*8))
      }}'
    ;;
  eth_getProof)
    address="$(jq -r '.params[0]' <<<"$request")"
    key="$(jq -r '.params[1][0]' <<<"$request")"
    jq -cn --arg address "$address" --arg key "$key" \
      '{jsonrpc:"2.0",id:1,result:{address:$address,accountProof:["0xc0"],balance:"0x0",codeHash:("0x"+("00"*32)),nonce:"0x0",storageHash:("0x"+("00"*32)),storageProof:[{key:$key,value:"0x0",proof:["0xc0"]}]}}'
    ;;
  *)
    jq -cn '{jsonrpc:"2.0",id:1,error:{code:-32601,message:"unknown method"}}'
    ;;
esac
EOF
chmod +x "$fixture_dir/curl"
export PATH="$fixture_dir:$PATH"

qbft_output="$fixture_dir/qbft"
"$repo_root/scripts/collect-test-data.sh" qbft http://rpc.invalid 7 8 "$qbft_output" \
  >/dev/null
jq -e '.collectionSchemaVersion == 1 and .fromHeight == 7 and .toHeight == 8' \
  "$qbft_output/manifest.json" >/dev/null
jq -e '.number == "0x7"' "$qbft_output/block-7/normalized.json" >/dev/null
jq -e '.result.number == "0x8"' "$qbft_output/block-8/raw-rpc.json" >/dev/null
jq -e '.testDataVersion == 1 and .inputFiles == ["raw-rpc.json","normalized.json"]' \
  "$qbft_output/block-7/manifest.json" >/dev/null

slot=0xaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
printf '%s\n' "$slot" >"$fixture_dir/slots.txt"
mpt_output="$fixture_dir/mpt"
"$repo_root/scripts/collect-test-data.sh" mpt \
  http://rpc.invalid 0x1111111111111111111111111111111111111111 7 \
  "$fixture_dir/slots.txt" "$mpt_output" >/dev/null
jq -e '.collectionSchemaVersion == 1 and .blockHeight == 7 and .blockQuantity == "0x7" and .proofCount == 1' \
  "$mpt_output/manifest.json" >/dev/null
jq -e --arg slot "$slot" '.storageProof[0].key == $slot' "$mpt_output/proof-0/normalized.json" \
  >/dev/null
jq -e '.testDataVersion == 1 and .expectedSuccess == true' \
  "$mpt_output/proof-0/manifest.json" >/dev/null

if "$repo_root/scripts/collect-test-data.sh" qbft http://rpc.invalid 8 7 "$fixture_dir/bad" \
    >/dev/null 2>&1; then
  echo "QBFT collector accepted a reversed range" >&2
  exit 1
fi
printf '%s\n%s\n' "$slot" "$slot" >"$fixture_dir/duplicate-slots.txt"
if "$repo_root/scripts/collect-test-data.sh" mpt \
    http://rpc.invalid 0x1111111111111111111111111111111111111111 7 \
    "$fixture_dir/duplicate-slots.txt" "$fixture_dir/bad-mpt" >/dev/null 2>&1; then
  echo "MPT collector accepted a duplicate key" >&2
  exit 1
fi

for mode in qbft mpt; do
  if [[ "$mode" == qbft ]]; then
    arguments=(qbft http://rpc.invalid 7 8 "$fixture_dir/bad-envelope-qbft")
  else
    arguments=(mpt http://rpc.invalid 0x1111111111111111111111111111111111111111 7
      "$fixture_dir/slots.txt" "$fixture_dir/bad-envelope-mpt")
  fi
  if MOCK_BAD_ENVELOPE=1 bash "$repo_root/scripts/collect-test-data.sh" "${arguments[@]}" \
      >/dev/null 2>"$fixture_dir/stderr"; then
    echo "$mode collector accepted a malformed RPC envelope" >&2; exit 1
  fi
  grep -q 'RPC returned an error or malformed envelope' "$fixture_dir/stderr"
done
if MOCK_RPC_ERROR='historical state pruned' bash "$repo_root/scripts/collect-test-data.sh" \
    "${arguments[@]}" >/dev/null 2>"$fixture_dir/stderr"; then
  echo 'MPT collector accepted a pruned-state RPC error' >&2; exit 1
fi
grep -q 'historical state is unavailable at block 0x7' "$fixture_dir/stderr"

echo "collection script fixture tests passed"
