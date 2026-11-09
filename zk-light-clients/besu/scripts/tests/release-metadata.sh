#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
fixture_dir="$(mktemp -d)"
trap 'rm -rf "$fixture_dir"' EXIT
mkdir -p "$fixture_dir/scripts" "$fixture_dir/artifacts" "$fixture_dir/bin" \
  "$fixture_dir/contracts/out/BesuQBFTZKLightClient.sol"
cp "$repo_root/scripts/release-workflow.sh" "$fixture_dir/scripts/"
workflow="$fixture_dir/scripts/release-workflow.sh"
export RELEASE_MANIFEST="$fixture_dir/release/manifest.json"

cat >"$fixture_dir/bin/tool" <<'TOOL'
#!/usr/bin/env bash
set -euo pipefail
case "$(basename "$0")" in
  git) printf '%040d\n' 0 ;;
  forge) printf 'forge 1.8.1\n' ;;
  cast) printf '0x%s\n' "$(printf 'a%.0s' {1..64})" ;;
  *) exit 64 ;;
esac
TOOL
chmod +x "$fixture_dir/bin/tool"
for name in git forge cast; do ln -s tool "$fixture_dir/bin/$name"; done
export PATH="$fixture_dir/bin:$PATH"
export FORGE_BIN="$fixture_dir/bin/forge" CAST_BIN="$fixture_dir/bin/cast"

printf '%s\n' '{"abi":[],"bytecode":{"object":"0x6001"},"deployedBytecode":{"object":"0x6002"}}' \
  >"$fixture_dir/contracts/out/BesuQBFTZKLightClient.sol/BesuQBFTZKLightClient.json"
for program in update-client membership non-membership; do
  printf '%s' "$program" >"$fixture_dir/artifacts/$program.elf"
done
jq -nS \
  --arg update "0x$(sha256sum "$fixture_dir/artifacts/update-client.elf" | cut -d' ' -f1)" \
  --arg membership "0x$(sha256sum "$fixture_dir/artifacts/membership.elf" | cut -d' ' -f1)" \
  --arg non "0x$(sha256sum "$fixture_dir/artifacts/non-membership.elf" | cut -d' ' -f1)" \
  '{"update-client":{elfSha256:$update,programVKey:("0x"+("1"*64))},
    membership:{elfSha256:$membership,programVKey:("0x"+("2"*64))},
    "non-membership":{elfSha256:$non,programVKey:("0x"+("3"*64))}}' \
  >"$fixture_dir/artifacts/vkeys.json"

bash "$workflow" manifest >/dev/null
bash "$workflow" metadata >/dev/null
jq -e 'has("performanceBaseline") | not' "$RELEASE_MANIFEST" >/dev/null
cp "$RELEASE_MANIFEST" "$fixture_dir/reviewed.json"

# Invalid identities/toolchains must still be rejected after removing performance validation.
for filter in \
  '.toolchains.sp1 = "invalid"' \
  '.programs.membership.programVKey = .programs["update-client"].programVKey' \
  '.programs.membership.programVKey = ("0x" + ("0" * 64))' \
  'del(.contract.abiCanonicalJsonSha256)'; do
  jq "$filter" "$fixture_dir/reviewed.json" >"$RELEASE_MANIFEST"
  if bash "$workflow" metadata >/dev/null 2>&1; then
    echo "release metadata accepted invalid data: $filter" >&2; exit 1
  fi
done
cp "$fixture_dir/reviewed.json" "$RELEASE_MANIFEST"

# A mismatched ELF must not overwrite the reviewed manifest.
printf 'changed' >"$fixture_dir/artifacts/membership.elf"
if bash "$workflow" manifest >/dev/null 2>&1; then
  echo 'manifest generation accepted an ELF/hash mismatch' >&2; exit 1
fi
cmp "$fixture_dir/reviewed.json" "$RELEASE_MANIFEST"
rm "$RELEASE_MANIFEST"
if bash "$workflow" metadata >/dev/null 2>&1; then
  echo 'release metadata accepted a missing manifest' >&2; exit 1
fi

echo "release metadata fixture tests passed"
