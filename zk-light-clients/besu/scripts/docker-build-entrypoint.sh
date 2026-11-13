#!/usr/bin/env bash
# Internal dispatcher for the mounted build environment.
set -euo pipefail
cd "$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

setup_contracts() {
  bash scripts/install-contract-deps.sh
}
build_host() {
  cargo build --locked --release -p prover-script -p proof-api --bins
}
build_contracts() {
  setup_contracts
  (cd contracts && forge build)
}
build_circuits() (
  # Consumers use current/ so a failed build never replaces a complete artifact set.
  mkdir -p artifacts/.sets
  exec 9>artifacts/.build.lock
  flock -n 9 || { echo "another circuit build is in progress" >&2; exit 1; }
  work="$(mktemp -d "$PWD/artifacts/.sets/build.XXXXXX")"
  trap 'rm -rf "$work"; rm -f artifacts/.current-next' EXIT
  ARTIFACTS_DIR="$work" bash scripts/build-sp1.sh --local
  ARTIFACTS_DIR="$work" bash scripts/generate-vkeys.sh "$work/vkeys.json"
  for file in update-client.elf membership.elf non-membership.elf vkeys.json; do
    [[ -s "$work/$file" ]] || { echo "missing build output: $file" >&2; exit 1; }
  done
  chmod a+rx artifacts artifacts/.sets "$work"
  chmod a+r "$work"/*
  ln -s ".sets/$(basename "$work")" artifacts/.current-next
  # GNU mv inside the Linux build container replaces the symlink itself.
  mv -Tf artifacts/.current-next artifacts/current
  trap - EXIT
  for file in update-client.elf membership.elf non-membership.elf vkeys.json; do
    ln -sfn "current/$file" "artifacts/$file"
  done
  echo "published circuits and VKeys in artifacts/current"
)
case "${1:-}" in
  all) build_host; build_contracts; build_circuits ;;
  host) build_host ;;
  proof-api) cargo build --locked --release -p proof-api --bin proof-api ;;
  circuits) build_circuits ;;
  contracts) build_contracts ;;
  test)
    setup_contracts
    (cd contracts && forge lint src script --severity high --severity med --severity low --deny warnings)
    for suite in rust shell contracts; do bash scripts/test-workflow.sh "$suite"; done
    ;;
  *) echo "usage: $0 [all|host|proof-api|circuits|contracts|test]" >&2; exit 64 ;;
esac
