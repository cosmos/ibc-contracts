#!/usr/bin/env bash
set -euo pipefail
repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT
mkdir -p "$work/router/ibc-solidity/contracts"
printf '// fixture\n' >"$work/router/ibc-solidity/contracts/ICS26Router.sol"
git -C "$work/router" init -q
git -C "$work/router" add .
git -C "$work/router" -c user.name=Fixture -c user.email=fixture@example.invalid commit -qm fixture
revision="$(git -C "$work/router" rev-parse HEAD)"
cat >"$work/forge" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
[[ "$PWD" == */e2e/solidity ]]
[[ "$FOUNDRY_REMAPPINGS" == *"@ibc-contracts/=$E2E_ROUTER_DIR/ibc-solidity/contracts/"* ]]
[[ "$FOUNDRY_REMAPPINGS" != *contracts/lib/ibc-contracts* ]]
printf '%s\n' "$*" >"$MOCK_FORGE_LOG"
EOF
chmod +x "$work/forge"
export E2E_ROUTER_DIR="$work/router" E2E_ROUTER_REV="$revision"
export FORGE_BIN="$work/forge" MOCK_FORGE_LOG="$work/forge.log"
bash "$repo_root/e2e/scripts/forge.sh" test --match-contract Fixture
[[ "$(cat "$work/forge.log")" == 'test --match-contract Fixture' ]]
rm "$work/forge.log"
if E2E_ROUTER_REV=0000000000000000000000000000000000000000 \
  bash "$repo_root/e2e/scripts/forge.sh" test >"$work/output" 2>&1; then
  echo "accepted the wrong Router revision" >&2; exit 1
fi
[[ ! -e "$work/forge.log" ]]
printf '// changed\n' >>"$work/router/ibc-solidity/contracts/ICS26Router.sol"
if bash "$repo_root/e2e/scripts/forge.sh" test >"$work/output" 2>&1; then
  echo "accepted dirty Router sources" >&2; exit 1
fi
[[ ! -e "$work/forge.log" ]]
if E2E_ROUTER_DIR= bash "$repo_root/e2e/scripts/forge.sh" test >"$work/output" 2>&1; then
  echo "accepted a missing Router checkout" >&2; exit 1
fi
echo "external Router workspace tests passed"
