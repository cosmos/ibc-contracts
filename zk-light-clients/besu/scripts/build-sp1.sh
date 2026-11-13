#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

mode="${1:---docker}"
if (( $# > 1 )); then
  echo "usage: $0 [--docker|--local]" >&2
  exit 64
fi
build_args=(--locked)
case "$mode" in
  --docker) build_args+=(--docker --tag v6.1.0) ;;
  --local)
    # Used inside the self-contained build image. Fail if its compiler is absent
    # rather than downloading a guest toolchain while the build is running.
    rustup run succinct rustc --version
    export RUSTUP_TOOLCHAIN=succinct
    ;;
  *) echo "usage: $0 [--docker|--local]" >&2; exit 64 ;;
esac

required_cli_commit="d454975"
installed_version="$(cargo prove --version)"
if [[ "$installed_version" != *"$required_cli_commit"* ]]; then
  echo "cargo-prove does not match pinned SP1 v6.1.0 release ($required_cli_commit)" >&2
  exit 1
fi

if [[ -z "${ARTIFACTS_DIR:-}" && -e artifacts/current ]]; then
  echo "use make build-docker to replace the published ELF/VKey set together" >&2
  exit 1
fi
artifacts_dir="${ARTIFACTS_DIR:-artifacts}"
mkdir -p "$artifacts_dir"

cargo prove build "${build_args[@]}" -p update-client \
  --elf-name update-client.elf --output-directory "$artifacts_dir"
cargo prove build "${build_args[@]}" -p membership \
  --elf-name membership.elf --output-directory "$artifacts_dir"
cargo prove build "${build_args[@]}" -p non-membership \
  --elf-name non-membership.elf --output-directory "$artifacts_dir"
