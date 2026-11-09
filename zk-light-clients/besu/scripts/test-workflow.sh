#!/usr/bin/env bash
set -euo pipefail
cd "$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

run_action() {
  case "$1" in
    all)
      for action in rust shell contracts check coverage sp1 integration; do
        run_action "$action"
      done
      ;;
    rust)
      cargo test --workspace --locked --exclude prover-script
      cargo test --locked -p prover-script --lib --bins --test rpc --test witness
      ;;
    shell)
      for suite in scripts/tests/*.sh; do bash "$suite"; done
      "${PYTHON:-python3}" -m unittest discover -s scripts/tests -p '*.py'
      ;;
    contracts) (cd contracts && forge test) ;;
    check)
      cargo fmt --all -- --check
      (cd contracts && forge fmt --check && forge lint src script --severity high --severity med --severity low --deny warnings)
      cargo clippy --workspace --all-targets --locked -- -D warnings
      cargo fmt --manifest-path e2e/Cargo.toml -- --check
      cargo clippy --locked --manifest-path e2e/Cargo.toml --all-targets -- -D warnings
      cargo test --locked --manifest-path e2e/Cargo.toml
      ;;
    coverage) bash scripts/coverage.sh all ;;
    coverage-zk) bash scripts/coverage.sh zk ;;
    coverage-proof-api) bash scripts/coverage.sh proof-api ;;
    sp1)
      bash scripts/docker-build.sh circuits
      cargo test --locked -p prover-script --test sp1_execute -- --test-threads=1
      # The gateway suite reads fixtures from this canonical directory.
      bash scripts/tests/tools/generate-golden-proofs.sh artifacts/proofs
      (cd contracts && RUN_REAL_SP1=true forge test --match-contract SP1VerifierIntegrationTest)
      ;;
    integration) bash e2e/scripts/forge.sh test ;;
    *) echo 'Test ACTION must be all, rust, shell, contracts, check, coverage, coverage-zk, coverage-proof-api, sp1 or integration' >&2; return 64 ;;
  esac
}
run_action "${1:-all}"
