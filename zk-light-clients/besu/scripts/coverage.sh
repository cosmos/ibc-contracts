#!/usr/bin/env bash
set -euo pipefail
cd "$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

report() {
  local ignore="$2" output="target/coverage/$1"
  mkdir -p "$output"
  if [[ "${SUMMARY:-0}" == 1 ]]; then
    cargo llvm-cov report --json --summary-only --ignore-filename-regex "$ignore" --output-path "$output/summary.json"
    "${PYTHON:-python3}" scripts/coverage-summary.py "$output/summary.json" | tee "$output/summary.txt"
  else
    cargo llvm-cov report --ignore-filename-regex "$ignore" | tee "$output/summary.txt"
  fi
  cargo llvm-cov report --ignore-filename-regex "$ignore" --html --output-dir "$output"
}
run_action() {
  case "$1" in
    all) run_action zk; run_action proof-api ;;
    zk)
      cargo llvm-cov clean --workspace
      cargo llvm-cov --locked -p besu-zk-types -p besu-public-values -p evm-mpt -p besu-qbft -p ibc-storage -p batch-merkle --no-report
      report zk '/(tooling|programs)/'
      ;;
    proof-api)
      cargo llvm-cov clean --workspace
      cargo llvm-cov --locked -p proof-api -p proof-api-integration --no-report
      cargo llvm-cov --locked -p prover-script --lib --bins --test rpc --test witness --no-report
      report proof-api '/(crates|programs)/'
      ;;
    *) echo 'Coverage action must be all, zk or proof-api' >&2; return 64 ;;
  esac
}
run_action "${1:-all}"
