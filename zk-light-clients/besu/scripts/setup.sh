#!/usr/bin/env bash
set -euo pipefail
cd "$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

for tool in rustup sp1up foundryup "${PYTHON:-python3}"; do
  command -v "$tool" >/dev/null || { echo "Missing $tool; see docs/DEVELOPMENT.md" >&2; exit 1; }
done
rust_version="${RUST_VERSION:-$(sed -n 's/^channel = "\([^"]*\)"/\1/p' rust-toolchain.toml)}"
test -n "$rust_version"
rustup toolchain install "$rust_version" --profile minimal --component clippy --component rustfmt --component llvm-tools-preview
if ! command -v cargo-llvm-cov >/dev/null; then
  cargo install cargo-llvm-cov --locked
fi
sp1up --version "${SP1_VERSION:-6.1.0}"
cargo prove --version
foundryup --install "${FOUNDRY_VERSION:-v1.8.1}"
git submodule update --init --recursive contracts/lib/sp1-contracts
bash scripts/install-contract-deps.sh
"${PYTHON:-python3}" -m venv .venv
.venv/bin/python -m pip install -r deploy/requirements.txt
echo 'Setup complete. Run make build-docker before preparing a deployment profile.'
