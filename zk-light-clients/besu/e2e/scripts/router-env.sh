#!/usr/bin/env bash
# Source from test tooling only. Never fetch or modify the caller's Router checkout.
E2E_ROUTER_DIR="${E2E_ROUTER_DIR:?set E2E_ROUTER_DIR to an external ibc-contracts checkout}"
E2E_ROUTER_REV="${E2E_ROUTER_REV:-1d129afa1305069cb4ed709357922e5b44799dd8}"
E2E_ROUTER_DIR="$(cd "$E2E_ROUTER_DIR" && pwd)"
actual_revision="$(git -C "$E2E_ROUTER_DIR" rev-parse HEAD)"
if [[ "$actual_revision" != "$E2E_ROUTER_REV" ]]; then
  echo "E2E Router revision is $actual_revision; expected $E2E_ROUTER_REV" >&2
  exit 1
fi
if [[ -n "$(git -C "$E2E_ROUTER_DIR" status --porcelain --untracked-files=all -- ibc-solidity/contracts)" ]]; then
  echo "E2E Router contract sources differ from the selected revision" >&2
  exit 1
fi
[[ -f "$E2E_ROUTER_DIR/ibc-solidity/contracts/ICS26Router.sol" ]] || {
  echo "E2E_ROUTER_DIR does not contain ibc-solidity/contracts/ICS26Router.sol" >&2
  exit 1
}
export E2E_ROUTER_DIR E2E_ROUTER_REV
