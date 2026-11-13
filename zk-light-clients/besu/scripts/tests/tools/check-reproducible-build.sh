#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
cd "$repo_root"

mkdir -p "$repo_root/artifacts"
work="$(mktemp -d "$repo_root/artifacts/reproducibility.XXXXXX")"

remove_docker_target() {
  local target="$1"
  local relative="${target#"$repo_root/"}"
  if [[ "$relative" == "$target" || "$relative" != artifacts/reproducibility.*/target-* ]]; then
    echo "refusing to remove unexpected Docker target: $target" >&2
    return 1
  fi
  docker run --rm --platform linux/amd64 \
    --volume "$repo_root:/root/program" \
    --entrypoint rm \
    ghcr.io/succinctlabs/sp1:v6.1.0 \
    -rf "/root/program/$relative"
}

cleanup() {
  local status="$?"
  trap - EXIT
  remove_docker_target "$work/target-one" >/dev/null 2>&1 || true
  remove_docker_target "$work/target-two" >/dev/null 2>&1 || true
  rm -rf "$work"
  exit "$status"
}
trap cleanup EXIT

declare -A first_hashes
CARGO_TARGET_DIR="$work/target-one" ARTIFACTS_DIR="$work/one" scripts/build-sp1.sh
for elf in update-client.elf membership.elf non-membership.elf; do
  first_hashes[$elf]="$(sha256sum "$work/one/$elf" | cut -d' ' -f1)"
done

remove_docker_target "$work/target-one"
rm -rf "$work/one"
CARGO_TARGET_DIR="$work/target-two" ARTIFACTS_DIR="$work/two" scripts/build-sp1.sh

for elf in update-client.elf membership.elf non-membership.elf; do
  first="${first_hashes[$elf]}"
  second="$(sha256sum "$work/two/$elf" | cut -d' ' -f1)"
  if [[ "$first" != "$second" ]]; then
    echo "$elf is not reproducible: $first != $second" >&2
    exit 1
  fi
  printf '%s  %s\n' "$first" "$elf"
done
