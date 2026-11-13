#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
action="${1:-all}"
case "$action" in
  all | host | proof-api | circuits | contracts | test) ;;
  *) echo "usage: $0 [all|host|proof-api|circuits|contracts|test]" >&2; exit 64 ;;
esac
command -v docker >/dev/null || { echo "Install Docker and start its engine first" >&2; exit 1; }
docker info >/dev/null

# Git metadata belongs to the parent repository and is outside the container mount.
# Initialize on the host; the builder only needs the checked-out contract sources.
case "$action" in
  all | contracts | test)
    git -C "$repo_root" submodule update --init --recursive contracts/lib/sp1-contracts
    ;;
esac

image="${BUILD_IMAGE:-besu-light-client-build:local}"
# The image bundles the pinned SP1 linux/amd64 compiler, including on Apple Silicon.
docker build --platform linux/amd64 --tag "$image" \
  --file "$repo_root/deploy/build/Dockerfile" "$repo_root/deploy/build"

mkdir -p "$repo_root/target/docker/.cache/home" "$repo_root/target/docker/.cache/cargo" \
  "$repo_root/artifacts"
owner="$(id -u):$(id -g)"
run_args=(run --rm --platform linux/amd64 --user "$owner"
  --mount "type=bind,source=$repo_root,target=/workspace"
  --workdir /workspace
  --env "HOME=/workspace/target/docker/.cache/home"
  --env "CARGO_HOME=/workspace/target/docker/.cache/cargo"
  --env "CARGO_TARGET_DIR=/workspace/target/docker"
  # SP1 host binaries have large release links. Avoid concurrent rustc/linker
  # processes exhausting Docker Desktop's memory, even when it exposes many CPUs.
  --env "CARGO_BUILD_JOBS=${CARGO_BUILD_JOBS:-1}"
)

# Every build runs as the caller; no child compiler container or root cleanup is needed.
docker "${run_args[@]}" "$image" scripts/docker-build-entrypoint.sh "$action"
