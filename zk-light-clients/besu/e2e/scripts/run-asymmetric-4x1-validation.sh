#!/usr/bin/env bash
set -euo pipefail

if [[ "${RUN_HEAVY_E2E:-0}" != 1 ]]; then
  echo "set RUN_HEAVY_E2E=1 explicitly; this command generates real Groth16 proofs" >&2
  exit 64
fi
if [[ "$#" -ne 1 ]]; then
  echo "usage: $0 update-client|membership|non-membership|security|all" >&2
  exit 64
fi

case_name="$1"
case "$case_name" in
  update-client) tests=(update_client) ;;
  membership) tests=(packet_membership bidirectional state_reuse) ;;
  non-membership) tests=(timeout) ;;
  security) tests=(invalid_proofs) ;;
  all)
    tests=(update_client packet_membership bidirectional state_reuse timeout invalid_proofs)
    ;;
  *)
    echo "unknown validation case: $case_name" >&2
    exit 64
    ;;
esac

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
deployments="$repo_root/.e2e-state/deployments.json"
prover_bin="${E2E_PROVER_BIN:-$repo_root/target/release/prover-script}"
chain_a_rpc="${CHAIN_A_RPC_URL:-http://127.0.0.1:18545}"
chain_b_rpc="${CHAIN_B_RPC_URL:-http://127.0.0.1:28545}"
run_id="$(date -u +%Y%m%dT%H%M%SZ)"
log_root="$repo_root/validation-logs/asymmetric-4x1-$run_id"

for command in cargo jq curl /usr/bin/time; do
  if ! command -v "$command" >/dev/null 2>&1; then
    echo "required command is unavailable: $command" >&2
    exit 69
  fi
done
if [[ ! -s "$deployments" ]]; then
  echo "missing deployment manifest: $deployments" >&2
  exit 1
fi
if [[ ! -x "$prover_bin" ]]; then
  echo "missing release prover binary: $prover_bin" >&2
  echo "build it with: cargo build --locked --release -p prover-script --bin prover-script" >&2
  exit 1
fi
for elf in update-client membership non-membership; do
  if [[ ! -s "$repo_root/artifacts/$elf.elf" ]]; then
    echo "missing canonical guest ELF: $repo_root/artifacts/$elf.elf" >&2
    exit 1
  fi
done
if pgrep -f '[p]rover-script prove-' >/dev/null 2>&1; then
  echo "another SP1 proof process is already running; run heavy validations serially" >&2
  exit 1
fi

CHAIN_A_RPC_URL="$chain_a_rpc" CHAIN_B_RPC_URL="$chain_b_rpc" \
  "$repo_root/e2e/scripts/check-asymmetric-4x1.sh"
mkdir -p "$log_root"

for test_target in "${tests[@]}"; do
  log="$log_root/$test_target.log"
  status="$log_root/$test_target.exit"
  echo "starting $test_target; log=$log"
  set +e
  /usr/bin/time -v env \
    SP1_PROVER="${SP1_PROVER:-cpu}" \
    CARGO_BUILD_JOBS="${CARGO_BUILD_JOBS:-1}" \
    E2E_PROVER_BIN="$prover_bin" \
    E2E_TRUSTING_PERIOD_SECONDS="${E2E_TRUSTING_PERIOD_SECONDS:-86400}" \
    E2E_MAX_CLOCK_DRIFT_SECONDS="${E2E_MAX_CLOCK_DRIFT_SECONDS:-30}" \
    E2E_DEPLOYMENTS="$deployments" \
    CHAIN_A_RPC_URL="$chain_a_rpc" \
    CHAIN_B_RPC_URL="$chain_b_rpc" \
    CAST_BIN="${CAST_BIN:-cast}" \
    RUN_BESU_E2E=1 \
    cargo test --locked --manifest-path "$repo_root/e2e/Cargo.toml" \
      --test "$test_target" -- --ignored --nocapture --test-threads=1 \
    2>&1 | tee "$log"
  rc="${PIPESTATUS[0]}"
  set -e
  printf '%s\n' "$rc" >"$status"
  if [[ "$rc" -ne 0 ]]; then
    echo "$test_target failed with exit code $rc; see $log" >&2
    exit "$rc"
  fi
  echo "$test_target passed"
done

echo "$case_name validation passed; logs=$log_root"
