#!/usr/bin/env bash
set -euo pipefail

# Host boundary for deployment using the existing build tool image and mounted outputs.
# RPC URLs and the signing key are mounted as files. Chain checks run inside deployment.
# Usage: scripts/deploy.sh (bootstrap|deploy|register|preflight) <profile> [height|address]

usage() {
  echo "usage: scripts/deploy.sh (preflight|deploy) <profile> | bootstrap <profile> [height] |" \
    "register <profile> <address>" >&2
  exit 64
}
command="${1:-}"
name="${2:-}"
[[ "$name" =~ ^[A-Za-z0-9][A-Za-z0-9._-]*$ ]] || usage
# Make supplies action/profile positionally and optional values through the environment.
# Explicit CLI arguments continue to take precedence.
case "$command" in
  register)
    if (( $# == 2 )); then
      [[ -n "${LIGHT_CLIENT:-}" ]] || { echo "Set LIGHT_CLIENT to the existing proxy address" >&2; exit 1; }
      set -- "$@" "$LIGHT_CLIENT"
    fi
    ;;
  bootstrap)
    if (( $# == 2 )) && [[ -n "${HEIGHT:-}" ]]; then set -- "$@" "$HEIGHT"; fi
    ;;
esac
case "$command" in
  deploy|preflight) (( $# == 2 )) || usage ;;
  bootstrap) (( $# == 2 || $# == 3 )) || usage ;;
  register) (( $# == 3 )) || usage ;;
  *) usage ;;
esac

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
if [[ -n "${IBC_CONFIG:-}" ]]; then
  exec "${PYTHON:-python3}" "$repo_root/deploy/prepare-profile.py" run "$@"
fi
profile_dir="$(cd "${PROFILE_DIR:-$repo_root/config}" && pwd)"
[[ -f "$profile_dir/$name.json" ]] || { echo "no profile $profile_dir/$name.json" >&2; exit 1; }

image="${BUILD_IMAGE:-besu-light-client-build:local}"
image_id="$(docker image inspect --format '{{.Id}}' "$image")" \
  || { echo "no build environment $image; run make build-docker first" >&2; exit 1; }
[[ -x "$repo_root/target/docker/release/bootstrap-state" ]] \
  || { echo "missing Linux bootstrap-state; run make build-docker first" >&2; exit 1; }
mkdir -p "$repo_root/target/docker/.cache/home"
run_args=(
  --rm --platform linux/amd64 --user "$(id -u):$(id -g)"
  --volume "$repo_root:/workspace"
  --volume "$profile_dir:/workspace/config:ro"
  --volume "$repo_root/artifacts:/workspace/artifacts:ro"
  --workdir /workspace
  --env APP_DIR=/workspace
  --env HOME=/workspace/target/docker/.cache/home
  --env BOOTSTRAP_STATE_BIN=/workspace/target/docker/release/bootstrap-state
  --env FOUNDRY_OFFLINE=true
  --env "IMAGE_ID=$image_id" --env "IMAGE_TAG=$image"
)
[[ -z "${DOCKER_NETWORK:-}" ]] || run_args+=(--network "$DOCKER_NETWORK")
urls=(SOURCE_RPC_URL)
[[ "$command" == bootstrap ]] || urls+=(DESTINATION_RPC_URL)
secrets="$(mktemp -d)"
trap 'rm -rf "$secrets"' EXIT
for variable in "${urls[@]}"; do
  [[ -n "${!variable:-}" ]] || { echo "set $variable" >&2; exit 1; }
  # Readable by the container's uid; the directory, 0700, keeps other host users out.
  file="$(printf '%s' "$variable" | tr 'A-Z_' 'a-z-')"
  printf '%s\n' "${!variable}" >"$secrets/$file"
  chmod 0644 "$secrets/$file"
  run_args+=(--volume "$secrets/$file:/run/secrets/$file:ro")
done
case "$command" in
  deploy | register)
    key="${DEPLOYER_KEY_FILE:?set DEPLOYER_KEY_FILE to the deployer key file}"
    [[ -r "$key" ]] || { echo "cannot read DEPLOYER_KEY_FILE: $key" >&2; exit 1; }
    key="$(cd "$(dirname "$key")" && pwd)/$(basename "$key")"
    run_args+=(--volume "$key:/run/secrets/deployer-key:ro")
    ;;
esac

if [[ "$command" != deploy && "$command" != register ]]; then
  status=0
  docker run "${run_args[@]}" "$image" deploy/entrypoint.sh "$@" || status=$?
  exit "$status"
fi

output="$repo_root/deployments/$name/$(date -u +%Y%m%dT%H%M%SZ)-$$.json"
mkdir -p "$(dirname "$output")"
status=0
docker run "${run_args[@]}" "$image" deploy/entrypoint.sh "$@" >"$output" || status=$?
if [[ -s "$output" ]]; then
  cat "$output"
  echo "output written to ${output#"$repo_root/"}" >&2
else
  rm -f "$output"
fi
exit "$status"
