#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
node_modules="$repo_root/contracts/node_modules"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT

# Standalone light-client dependencies; no Router checkout or Router-only packages.
npm install --prefix "$work" --cache "$work/cache" --ignore-scripts --no-save --package-lock=false \
  @openzeppelin/contracts@5.6.1 \
  @openzeppelin/contracts-upgradeable@5.6.1

mkdir -p "$node_modules/@openzeppelin/contracts" \
  "$node_modules/@openzeppelin/contracts-upgradeable"
cp -a "$work/node_modules/@openzeppelin/contracts/." "$node_modules/@openzeppelin/contracts/"
cp -a "$work/node_modules/@openzeppelin/contracts-upgradeable/." \
  "$node_modules/@openzeppelin/contracts-upgradeable/"
[[ "$(jq -r .version "$node_modules/@openzeppelin/contracts/package.json")" == "5.6.1" ]]
[[ "$(jq -r .version "$node_modules/@openzeppelin/contracts-upgradeable/package.json")" == "5.6.1" ]]

# A digest over every Solidity source of each package, so a registry or GitHub response that
# differs in any file the compiler reads stops the install.
sources_digest() {
  (cd "$1" && find . -type f -name '*.sol' -print0 | LC_ALL=C sort -z | xargs -0 sha256sum) \
    | sha256sum | cut -d' ' -f1
}
for entry in \
  "@openzeppelin/contracts:e8462ff8848f46da3d53d0d902a74ecf586edf98ab5b4e8dd26521d2570be159" \
  "@openzeppelin/contracts-upgradeable:9890b460137d0d0b9daec0af537e6880cb6dce64d7c42b9eb45c17e6ee103610"; do
  package="${entry%%:*}"
  if [[ "$(sources_digest "$node_modules/$package")" != "${entry##*:}" ]]; then
    echo "$package Solidity sources do not match the pinned digest" >&2
    exit 1
  fi
done
