#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
fixture_root="$(mktemp -d)"
trap 'rm -rf "$fixture_root"' EXIT
mkdir -p "$fixture_root/deploy" "$fixture_root/e2e/scripts" "$fixture_root/scripts" "$fixture_root/release" \
  "$fixture_root/contracts" "$fixture_root/artifacts" "$fixture_root/.e2e-state/chain-a" \
  "$fixture_root/.e2e-state/chain-b" "$fixture_root/bin"
cp "$repo_root/e2e/scripts/prepare-live-witness-fixtures.sh" "$fixture_root/e2e/scripts/"
cp "$repo_root/e2e/scripts/run-live-circuit-execute.sh" "$fixture_root/e2e/scripts/"
# Fixture only the external Foundry boundary; no Router checkout is needed for this shell unit test.
printf '#!/usr/bin/env bash\nexec "${FORGE_BIN:-forge}" "$@"\n' >"$fixture_root/e2e/scripts/forge.sh"
chmod +x "$fixture_root/e2e/scripts/forge.sh"

cat >"$fixture_root/.e2e-state/deployments.json" <<'EOF'
{
  "schemaVersion": 1,
  "chainA": {
    "router": "0x1111111111111111111111111111111111111111",
    "testApp": "0x1212121212121212121212121212121212121212",
    "verifierGateway": "0x1313131313131313131313131313131313131313"
  },
  "chainB": {
    "router": "0x2222222222222222222222222222222222222222",
    "testApp": "0x2323232323232323232323232323232323232323",
    "verifierGateway": "0x2424242424242424242424242424242424242424"
  }
}
EOF
printf '0x%s\n' "$(printf '11%.0s' {1..32})" >"$fixture_root/.e2e-state/chain-a/deployer.key"
printf '0x%s\n' "$(printf '22%.0s' {1..32})" >"$fixture_root/.e2e-state/chain-b/deployer.key"
cat >"$fixture_root/artifacts/vkeys.json" <<'EOF'
{
  "update-client": {"programVKey":"0x0101010101010101010101010101010101010101010101010101010101010101"},
  "membership": {"programVKey":"0x0202020202020202020202020202020202020202020202020202020202020202"},
  "non-membership": {"programVKey":"0x0303030303030303030303030303030303030303030303030303030303030303"}
}
EOF
for elf in update-client membership non-membership; do
  printf 'fixture' >"$fixture_root/artifacts/$elf.elf"
done

cat >"$fixture_root/deploy/validate.sh" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
exit 0
EOF
cat >"$fixture_root/bin/cast" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
case "$1" in
  block) printf '{"timestamp":"0x64"}\n' ;;
  block-number) printf '9\n' ;;
  to-dec)
    case "$2" in
      0x64) printf '100\n' ;;
      0x7) printf '7\n' ;;
      *) exit 64 ;;
    esac
    ;;
  call)
    if [[ " $* " == *" getCommitment(bytes32)(bytes32) "* ]]; then
      printf '0x%s\n' "$(printf 'ab%.0s' {1..32})"
    else
      printf '1\n'
    fi
    ;;
  send) printf '{"blockNumber":"0x7"}\n' ;;
  keccak) printf '0x%s\n' "$(printf 'cd%.0s' {1..32})" ;;
  *) exit 64 ;;
esac
EOF
cat >"$fixture_root/bin/forge" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
printf '{"clientId":"client-1","lightClient":"0x3434343434343434343434343434343434343434"}\n' \
  >"$E2E_OUTPUT_PATH"
EOF
cat >"$fixture_root/bin/cargo" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
if [[ " $* " == *" bootstrap-state "* ]]; then
  printf '{"height":7,"stateRoot":"0x%s","blockTimestamp":100,"validatorSetHash":"0x%s","validatorCount":4}\n' \
    "$(printf '44%.0s' {1..32})" "$(printf '55%.0s' {1..32})"
  exit 0
fi
printf '%s\n' "$*" >>"$MOCK_CARGO_LOG"
while (( $# > 0 )); do
  if [[ "$1" == --witness-output ]]; then
    printf '{}\n' >"$2"
    exit 0
  fi
  shift
done
exit 64
EOF
chmod +x "$fixture_root/deploy/validate.sh" "$fixture_root/bin/"*

PATH="$fixture_root/bin:$PATH" CAST_BIN="$fixture_root/bin/cast" \
FORGE_BIN="$fixture_root/bin/forge" \
  "$fixture_root/e2e/scripts/prepare-live-witness-fixtures.sh" >/dev/null
jq -e '
  .aToB.proofHeight == 7 and .aToB.sequence == 1 and
  .aToB.path == "0x636c69656e742d30010000000000000001" and
  .bToA.expectedSourceChainId == 13372
' "$fixture_root/.e2e-state/witness-fixtures/fixtures.json" >/dev/null
if PATH="$fixture_root/bin:$PATH" CAST_BIN="$fixture_root/bin/cast" \
  FORGE_BIN="$fixture_root/bin/forge" \
  "$fixture_root/e2e/scripts/prepare-live-witness-fixtures.sh" >/dev/null 2>&1; then
  echo "live fixture setup overwrote existing fixture state" >&2
  exit 1
fi

export MOCK_CARGO_LOG="$fixture_root/cargo.log"
PATH="$fixture_root/bin:$PATH" CAST_BIN="$fixture_root/bin/cast" \
RUN_SP1_EXECUTE_VALIDATION=1 \
  "$fixture_root/e2e/scripts/run-live-circuit-execute.sh" >/dev/null
test "$(grep -c -- ' execute-update ' "$MOCK_CARGO_LOG")" -eq 2
test "$(grep -c -- ' execute-membership ' "$MOCK_CARGO_LOG")" -eq 2
test "$(grep -c -- ' execute-non-membership ' "$MOCK_CARGO_LOG")" -eq 2
test "$(grep -c -- '--path 0x636c69656e742d30020000000000000001' "$MOCK_CARGO_LOG")" -eq 2

if PATH="$fixture_root/bin:$PATH" RUN_SP1_EXECUTE_VALIDATION=1 \
  E2E_EXECUTE_PROGRAM=invalid \
  "$fixture_root/e2e/scripts/run-live-circuit-execute.sh" >/dev/null 2>&1; then
  echo "live execute runner accepted an invalid program selector" >&2
  exit 1
fi

echo "live E2E tooling fixture tests passed"
