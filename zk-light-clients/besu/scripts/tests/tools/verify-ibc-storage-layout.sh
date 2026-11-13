#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
source "$repo_root/e2e/scripts/router-env.sh"

expected="0x1260944489272988d9df285149b5aa1b0f48f2136d6f416159f840a3e0747600"
source_file="$E2E_ROUTER_DIR/ibc-solidity/contracts/utils/IBCStoreUpgradeable.sol"
path_file="$E2E_ROUTER_DIR/ibc-solidity/contracts/utils/ICS24Host.sol"
contains_fixed() {
  local pattern="$1"
  local file="$2"
  if command -v rg >/dev/null 2>&1; then
    rg --fixed-strings --quiet -- "$pattern" "$file"
  else
    grep --fixed-strings --quiet -- "$pattern" "$file"
  fi
}

if ! contains_fixed "IBCSTORE_STORAGE_SLOT" "$source_file" \
  || ! contains_fixed "$expected" "$source_file" \
  || ! contains_fixed "mapping(bytes32 hashedPath => bytes32 commitment) commitments" \
    "$source_file" >/dev/null; then
  echo "pinned IBCSTORE_STORAGE_SLOT no longer matches the canonical dependency" >&2
  exit 1
fi
for required in \
  "abi.encodePacked(clientId, uint8(1), uint64ToBigEndian(sequence))" \
  "abi.encodePacked(clientId, uint8(2), uint64ToBigEndian(sequence))" \
  "abi.encodePacked(clientId, uint8(3), uint64ToBigEndian(sequence))"; do
  if ! contains_fixed "$required" "$path_file"; then
    echo "pinned ICS24 Router path layout no longer matches: $required" >&2
    exit 1
  fi
done

cd "$repo_root/contracts"
"${FORGE_BIN:-forge}" test --match-test testRustPathDigestAndStorageSlotGoldenVectors
"$repo_root/e2e/scripts/forge.sh" test --match-contract UpstreamIBCCommitmentVectorsTest
