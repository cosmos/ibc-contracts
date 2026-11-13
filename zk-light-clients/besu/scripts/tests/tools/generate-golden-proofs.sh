#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$repo_root"

output="${1:-artifacts/proofs}"
for elf in update-client membership non-membership; do
  [[ -s "artifacts/$elf.elf" ]] || { echo "run make build-docker first" >&2; exit 1; }
done
for program in update-client membership non-membership; do
  SP1_PROVER="${SP1_PROVER:-cpu}" cargo run --locked --release \
    -p prover-script --bin generate-golden-proofs -- "$output" "$program"
done
