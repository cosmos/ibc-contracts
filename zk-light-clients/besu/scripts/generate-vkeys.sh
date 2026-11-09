#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

artifacts_dir="${ARTIFACTS_DIR:-artifacts}"
output="${1:-$artifacts_dir/vkeys.json}"

for elf in update-client.elf membership.elf non-membership.elf; do
  if [[ ! -f "$artifacts_dir/$elf" ]]; then
    echo "missing canonical ELF: $artifacts_dir/$elf" >&2
    exit 1
  fi
done

mkdir -p "$(dirname "$output")"
temporary="$(mktemp "$(dirname "$output")/.vkeys.XXXXXX")"
trap 'rm -f "$temporary"' EXIT
cargo run --locked --release -p generate-vkeys -- "$artifacts_dir" > "$temporary"
mv -f "$temporary" "$output"
trap - EXIT
