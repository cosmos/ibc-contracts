#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
fixtures="$repo_root/.e2e-state/witness-fixtures/fixtures.json"
cast_bin="${CAST_BIN:-cast}"
selected_direction="${E2E_EXECUTE_DIRECTION:-all}"
selected_program="${E2E_EXECUTE_PROGRAM:-all}"
output_root="$repo_root/.e2e-state/witness-fixtures/execute"

if [[ "${RUN_SP1_EXECUTE_VALIDATION:-0}" != 1 ]]; then
  echo "RUN_SP1_EXECUTE_VALIDATION=1 is required for live SP1 execution" >&2
  exit 64
fi
case "$selected_direction" in
  all | a-to-b | b-to-a) ;;
  *) echo "E2E_EXECUTE_DIRECTION must be all, a-to-b, or b-to-a" >&2; exit 64 ;;
esac
case "$selected_program" in
  all | update-client | membership | non-membership) ;;
  *)
    echo "E2E_EXECUTE_PROGRAM must be all, update-client, membership, or non-membership" >&2
    exit 64
    ;;
esac
if [[ ! -s "$fixtures" ]]; then
  echo "missing live witness fixtures: $fixtures" >&2
  exit 1
fi
mkdir -p "$output_root"

run_program() {
  local expected="$1"
  shift
  if [[ "$selected_program" == all || "$selected_program" == "$expected" ]]; then
    CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-2}" cargo run --quiet --locked \
      --manifest-path "$repo_root/Cargo.toml" -p prover-script --bin prover-script -- "$@"
  fi
}

run_direction() {
  local name="$1"
  local query
  case "$name" in
    a-to-b) query=.aToB ;;
    b-to-a) query=.bToA ;;
    *) return 64 ;;
  esac

  local source_rpc destination_rpc chain_id source_router destination_client trusted_height
  local path expected_commitment sequence non_membership_path target_height direction_root
  source_rpc="$(jq -er "$query.sourceRpc" "$fixtures")"
  destination_rpc="$(jq -er "$query.destinationRpc" "$fixtures")"
  chain_id="$(jq -er "$query.expectedSourceChainId" "$fixtures")"
  source_router="$(jq -er "$query.sourceRouter" "$fixtures")"
  destination_client="$(jq -er "$query.destinationLightClient" "$fixtures")"
  trusted_height="$(jq -er "$query.proofHeight" "$fixtures")"
  path="$(jq -er "$query.path" "$fixtures")"
  expected_commitment="$(jq -er "$query.expectedCommitment" "$fixtures")"
  sequence="$(jq -er "$query.sequence" "$fixtures")"
  non_membership_path="$(printf '0x636c69656e742d3002%016x' "$sequence")"
  direction_root="$output_root/$name"
  mkdir -p "$direction_root"

  target_height="$("$cast_bin" block-number --rpc-url "$source_rpc")"
  for _ in {1..120}; do
    if ((target_height > trusted_height)); then
      break
    fi
    sleep 1
    target_height="$("$cast_bin" block-number --rpc-url "$source_rpc")"
  done
  if ((target_height <= trusted_height)); then
    echo "$name source chain did not advance above trusted height $trusted_height" >&2
    exit 1
  fi

  local common=(
    --source-rpc "$source_rpc"
    --destination-rpc "$destination_rpc"
    --expected-source-chain-id "$chain_id"
    --source-validator-mode header-qbft
    --proof-mode groth16
    --trusting-period-seconds "${E2E_TRUSTING_PERIOD_SECONDS:-3600}"
    --max-clock-drift-seconds "${E2E_MAX_CLOCK_DRIFT_SECONDS:-30}"
  )

  echo "executing update-client direction=$name trusted=$trusted_height target=$target_height"
  run_program update-client execute-update \
    "${common[@]}" \
    --destination-light-client "$destination_client" \
    --trusted-height "$trusted_height" \
    --target-height "$target_height" \
    --elf "$repo_root/artifacts/update-client.elf" \
    --witness-output "$direction_root/update-client-witness.json"

  echo "executing membership direction=$name height=$trusted_height path=$path"
  run_program membership execute-membership \
    "${common[@]}" \
    --source-router "$source_router" \
    --destination-light-client "$destination_client" \
    --proof-height "$trusted_height" \
    --path "$path" \
    --expected-commitment "$expected_commitment" \
    --elf "$repo_root/artifacts/membership.elf" \
    --witness-output "$direction_root/membership-witness.json"

  echo "executing non-membership direction=$name height=$trusted_height path=$non_membership_path"
  run_program non-membership execute-non-membership \
    "${common[@]}" \
    --source-router "$source_router" \
    --destination-light-client "$destination_client" \
    --proof-height "$trusted_height" \
    --path "$non_membership_path" \
    --elf "$repo_root/artifacts/non-membership.elf" \
    --witness-output "$direction_root/non-membership-witness.json"
}

if [[ "$selected_direction" == all || "$selected_direction" == a-to-b ]]; then
  run_direction a-to-b
fi
if [[ "$selected_direction" == all || "$selected_direction" == b-to-a ]]; then
  run_direction b-to-a
fi
echo "live SP1 execute validation completed"
