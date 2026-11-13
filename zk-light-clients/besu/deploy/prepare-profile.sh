#!/usr/bin/env bash
set -euo pipefail
# Native profile preparation; inputs may be flags or Makefile/environment variables.
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
exec "${PYTHON:-python3}" "$repo_root/deploy/prepare-profile.py" prepare "$@"
