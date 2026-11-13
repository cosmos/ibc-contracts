#!/usr/bin/env bash
set -euo pipefail
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

generate_manifest() (
  output="${RELEASE_MANIFEST:-$repo_root/release/manifest.json}"
  forge_bin="${FORGE_BIN:-forge}"
  cast_bin="${CAST_BIN:-cast}"
  contract_artifact="$repo_root/contracts/out/BesuQBFTZKLightClient.sol/BesuQBFTZKLightClient.json"

  if [[ ! -f "$contract_artifact" ]]; then
    echo "missing contract artifact; run 'forge build' in contracts first" >&2
    exit 1
  fi

  for command in jq sha256sum git "$forge_bin" "$cast_bin"; do
    if ! command -v "$command" >/dev/null 2>&1; then
      echo "required command is unavailable: $command" >&2
      exit 1
    fi
  done

  repo_commit="$(git -C "$repo_root" rev-parse HEAD)"
  sp1_contracts_commit="$(git -C "$repo_root/contracts/lib/sp1-contracts" rev-parse HEAD)"
  foundry_version="$($forge_bin --version | sed -n '1p')"

  for entry in \
    "update-client:update-client.elf" \
    "membership:membership.elf" \
    "non-membership:non-membership.elf"; do
    program="${entry%%:*}"
    elf="${entry##*:}"
    if [[ ! -f "$repo_root/artifacts/$elf" ]]; then
      echo "missing canonical ELF: artifacts/$elf" >&2
      exit 1
    fi
    expected="$(jq -r --arg program "$program" '.[$program].elfSha256' \
      "$repo_root/artifacts/vkeys.json")"
    actual="0x$(sha256sum "$repo_root/artifacts/$elf" | cut -d' ' -f1)"
    if [[ "$actual" != "$expected" ]]; then
      echo "$elf hash does not match artifacts/vkeys.json" >&2
      exit 1
    fi
  done

  canonical_abi="$(jq -cS '.abi' "$contract_artifact")"
  abi_sha256="0x$(printf '%s' "$canonical_abi" | sha256sum | cut -d' ' -f1)"
  creation_bytecode="$(jq -er '.bytecode.object | select(startswith("0x") and length > 2)' \
    "$contract_artifact")"
  deployed_bytecode="$(jq -er '.deployedBytecode.object | select(startswith("0x") and length > 2)' \
    "$contract_artifact")"
  creation_hash="$($cast_bin keccak "$creation_bytecode")"
  deployed_hash="$($cast_bin keccak "$deployed_bytecode")"

  tmp="$(mktemp)"
  trap 'rm -f "$tmp"' EXIT
  jq -nS \
    --arg repositoryGitCommit "$repo_commit" \
    --arg sp1ContractsCommit "$sp1_contracts_commit" \
    --arg foundryVersion "$foundry_version" \
    --arg abiSha256 "$abi_sha256" \
    --arg creationBytecodeKeccak256 "$creation_hash" \
    --arg deployedBytecodeKeccak256 "$deployed_hash" \
    --slurpfile vkeys "$repo_root/artifacts/vkeys.json" \
    '{
      schemaVersion: 1,
      spec: {documentId: "besu-qbft-zk-client-spec-v1.1.0-throughput"},
      repositoryGitCommit: $repositoryGitCommit,
      toolchains: {
        rust: "1.97.1",
        sp1: "6.1.0",
        solidity: "0.8.28",
        foundry: $foundryVersion
      },
      dependencies: {
        besu: {release: "26.5.0", gitCommit: "9b674a5da6c3da11603ef4cb00e0beaecf24730a"},
        sp1Contracts: {release: "v6.1.1", gitCommit: $sp1ContractsCommit}
      },
      soliditySettings: {
        optimizer: true,
        optimizerRuns: 200,
        evmVersion: "cancun",
        bytecodeHash: "none",
        cborMetadata: false
      },
      programs: $vkeys[0],
      contract: {
        name: "BesuQBFTZKLightClient",
        abiCanonicalJsonSha256: $abiSha256,
        creationBytecodeKeccak256: $creationBytecodeKeccak256,
        deployedBytecodeKeccak256: $deployedBytecodeKeccak256
      },
      protocolConstants: {
        publicValuesSchemaVersions: {updateClient: 1, membership: 2, nonMembership: 1},
        maxBatchMessages: 32,
        preferredBatchSize: 10,
        membershipBatchMerkle: {
          leafDomain: "0x00",
          nodeDomain: "0x01",
          emptyLeafDomain: "0x02",
          emptyBatchLeaf: "0xf2ee15ea639b73fa3db9b34a245bdfa015c260c598b211bf05a1ecc4b3e3b4f2"
        },
        validatorSetHashDomain: "BESU_QBFT_VALIDATOR_SET_V1",
        ibcStoreStorageSlot: "0x1260944489272988d9df285149b5aa1b0f48f2136d6f416159f840a3e0747600",
        pathTypes: {packet: "0x01", receipt: "0x02", acknowledgement: "0x03"}
      }
    }' >"$tmp"
  mkdir -p "$(dirname "$output")"
  mv "$tmp" "$output"
  trap - EXIT
  echo "wrote $output"
)

validate_metadata() (
  manifest="${RELEASE_MANIFEST:-$repo_root/release/manifest.json}"
  jq -e '
    .schemaVersion == 1 and
    .spec.documentId == "besu-qbft-zk-client-spec-v1.1.0-throughput" and
    (.repositoryGitCommit | test("^[0-9a-f]{40}$")) and
    .toolchains.rust == "1.97.1" and
    .toolchains.sp1 == "6.1.0" and
    .toolchains.solidity == "0.8.28" and
    .dependencies.besu.release == "26.5.0" and
    .dependencies.besu.gitCommit == "9b674a5da6c3da11603ef4cb00e0beaecf24730a" and
    .dependencies.sp1Contracts.release == "v6.1.1" and
    (.dependencies.sp1Contracts.gitCommit | test("^[0-9a-f]{40}$")) and
    (.programs | keys | sort) == ["membership","non-membership","update-client"] and
    ([.programs[].elfSha256 | test("^0x[0-9a-f]{64}$")] | all) and
    ([.programs[].programVKey | test("^0x[0-9a-f]{64}$") and . != ("0x" + ("0" * 64))] | all) and
    ([.programs[].programVKey] | unique | length) == 3 and
    (.contract.abiCanonicalJsonSha256 | test("^0x[0-9a-f]{64}$")) and
    (.contract.creationBytecodeKeccak256 | test("^0x[0-9a-f]{64}$")) and
    (.contract.deployedBytecodeKeccak256 | test("^0x[0-9a-f]{64}$")) and
    .protocolConstants.publicValuesSchemaVersions == {updateClient:1,membership:2,nonMembership:1} and
    .protocolConstants.validatorSetHashDomain == "BESU_QBFT_VALIDATOR_SET_V1"
    and .protocolConstants.ibcStoreStorageSlot == "0x1260944489272988d9df285149b5aa1b0f48f2136d6f416159f840a3e0747600"
    and .protocolConstants.pathTypes == {packet:"0x01",receipt:"0x02",acknowledgement:"0x03"}
  ' "$manifest" >/dev/null
  echo "release metadata validation passed"
)

run_action() {
  case "$1" in
    all)
      run_action reproducible
      run_action storage-layout
      run_action manifest
      run_action metadata
      ;;
    reproducible) bash scripts/tests/tools/check-reproducible-build.sh ;;
    storage-layout) bash scripts/tests/tools/verify-ibc-storage-layout.sh ;;
    manifest) generate_manifest ;;
    metadata) validate_metadata ;;
    *) echo 'Release ACTION must be all, reproducible, storage-layout, manifest or metadata' >&2; return 64 ;;
  esac
}
run_action "${1:-all}"
