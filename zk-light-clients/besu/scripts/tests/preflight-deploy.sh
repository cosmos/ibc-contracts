#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
fixture_dir="$(mktemp -d)"
trap 'rm -rf "$fixture_dir"' EXIT
export VKEYS_FILE="$fixture_dir/vkeys.json"
cp "$repo_root/release/expected-vkeys.json" "$VKEYS_FILE"

# The fakes refuse a URL among their arguments: an RPC URL reaches them only through the
# environment, FOUNDRY_ETH_RPC_URL for cast and SOURCE_RPC_URL for bootstrap-state.
cat >"$fixture_dir/cast" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
[[ "$*" != *://* ]] || { echo "cast was given a URL as an argument" >&2; exit 97; }
[[ "$1" == to-dec || -n "${FOUNDRY_ETH_RPC_URL:-}" ]] \
  || { echo "cast has no FOUNDRY_ETH_RPC_URL" >&2; exit 97; }
case "$1" in
  # The source answers 1337 and the destination 1111.
  rpc) if [[ "$FOUNDRY_ETH_RPC_URL" == http://127.0.0.1:9545 ]]; then printf '"0x457"\n'; else printf '"0x539"\n'; fi ;;
  to-dec) printf '%d\n' "$2" ;;
  block) printf '%s\n' "${FAKE_DESTINATION_TIME:-1060}" ;;
  code) if [[ "$2" == "${FAKE_NO_CODE_AT:-}" ]]; then printf '0x\n'; else printf '0x6000\n'; fi ;;
  call)
    if [[ "$3" == "getNextClientSeq()(uint256)" ]]; then
      printf '%s\n' "${FAKE_NEXT_CLIENT_SEQ:-0}"
    elif [[ "$3" == "authority()(address)" ]]; then
      printf '%s\n' "${FAKE_ROUTER_AUTHORITY:-0x5555555555555555555555555555555555555555}"
    else
      printf '0x1111111111111111111111111111111111111111\nfalse\n'
    fi
    ;;
  *) exit 64 ;;
esac
EOF
for name in cargo bootstrap-state; do
  cat >"$fixture_dir/$name" <<EOF
#!/usr/bin/env bash
set -euo pipefail
[[ "\$*" != *://* ]] || { echo "\$0 was given a URL as an argument" >&2; exit 97; }
printf '%s %s\n' "\$SOURCE_RPC_URL" "\$*" >"$fixture_dir/$name.log"
printf '{"height":7,"stateRoot":"0x%s","blockTimestamp":1000,"validatorSetHash":"0x%s","validatorCount":4}\n' \\
  "$(printf '1%.0s' {1..64})" "$(printf '2%.0s' {1..64})"
EOF
done
cat >"$fixture_dir/forge" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
exit 0
EOF
chmod +x "$fixture_dir/cast" "$fixture_dir/cargo" "$fixture_dir/bootstrap-state" \
  "$fixture_dir/forge"

export PATH="$fixture_dir:$PATH"
export CAST_BIN="$fixture_dir/cast"
export FORGE_BIN="$fixture_dir/forge"
export SOURCE_RPC_URL="http://127.0.0.1:8545"
export DESTINATION_RPC_URL="http://127.0.0.1:9545"
export EXPECTED_SOURCE_CHAIN_ID=1337
export SOURCE_VALIDATOR_MODE=header-qbft
export SP1_PROOF_MODE=groth16
export LOCAL_IBC_ROUTER=0x1111111111111111111111111111111111111111
export SOURCE_IBC_ROUTER=0x2222222222222222222222222222222222222222
export SP1_VERIFIER_GATEWAY=0x3333333333333333333333333333333333333333
export SP1_GROTH16_SELECTOR=0x12345678
export UPDATE_CLIENT_VKEY="$(jq -er '.["update-client"].programVKey' "$repo_root/release/expected-vkeys.json")"
export MEMBERSHIP_VKEY="$(jq -er '.["membership"].programVKey' "$repo_root/release/expected-vkeys.json")"
export NON_MEMBERSHIP_VKEY="$(jq -er '.["non-membership"].programVKey' "$repo_root/release/expected-vkeys.json")"
export INITIAL_HEIGHT=7
export INITIAL_STATE_ROOT=0x1111111111111111111111111111111111111111111111111111111111111111
export INITIAL_BLOCK_TIMESTAMP=1000
export INITIAL_VALIDATOR_SET_HASH=0x2222222222222222222222222222222222222222222222222222222222222222
export TRUSTING_PERIOD_SECONDS=1000
export MAX_CLOCK_DRIFT_SECONDS=20

run_preflight() {
  "$repo_root/deploy/validate.sh" >/dev/null 2>"$fixture_dir/stderr"
}

# Each fake logs the SOURCE_RPC_URL it was given, then its arguments.
bootstrap_args="--expected-source-chain-id 1337 --height 7"
source_url="$SOURCE_RPC_URL"

expect_bootstrap_run() {
  local ran="$1" expected_args="$2" skipped="$3"
  if [[ "$(cat "$fixture_dir/$ran.log")" != "$expected_args" ]]; then
    echo "$ran was not invoked with the expected arguments" >&2
    exit 1
  fi
  if [[ -e "$fixture_dir/$skipped.log" ]]; then
    echo "$skipped ran although $ran was selected" >&2
    exit 1
  fi
  rm -f "$fixture_dir/$ran.log"
}

run_preflight
expect_bootstrap_run cargo \
  "$source_url run --quiet --locked -p prover-script --bin bootstrap-state -- $bootstrap_args" \
  bootstrap-state

BOOTSTRAP_STATE_BIN="$fixture_dir/bootstrap-state" run_preflight
expect_bootstrap_run bootstrap-state "$source_url $bootstrap_args" cargo

expect_failure() {
  local expected="$1"
  shift
  if env "$@" "$repo_root/deploy/validate.sh" > /dev/null 2>"$fixture_dir/stderr"; then
    echo "preflight unexpectedly accepted invalid configuration" >&2
    exit 1
  fi
  if ! grep --fixed-strings --quiet -- "$expected" "$fixture_dir/stderr"; then
    echo "preflight failed for the wrong reason; expected: $expected" >&2
    cat "$fixture_dir/stderr" >&2
    exit 1
  fi
}

expect_failure "SOURCE_VALIDATOR_MODE must be header-qbft" SOURCE_VALIDATOR_MODE=contract
expect_failure "SP1_PROOF_MODE must be groth16" SP1_PROOF_MODE=plonk
expect_failure "SOURCE_RPC_URL must be a non-empty http(s) URL" SOURCE_RPC_URL=not-a-url
expect_failure "INITIAL_HEIGHT must be non-zero" INITIAL_HEIGHT=0
expect_failure "TRUSTING_PERIOD_SECONDS does not fit uint64" \
  TRUSTING_PERIOD_SECONDS=18446744073709551616
expect_failure "LOCAL_IBC_ROUTER must be a non-zero EVM address" \
  LOCAL_IBC_ROUTER=0x0000000000000000000000000000000000000000
expect_failure "the three program VKeys must be distinct" \
  MEMBERSHIP_VKEY="$UPDATE_CLIENT_VKEY"
expect_failure "SP1_GROTH16_SELECTOR must be exactly four bytes" SP1_GROTH16_SELECTOR=0x12
expect_failure "UPDATE_CLIENT_VKEY does not match generated artifacts/vkeys.json" \
  UPDATE_CLIENT_VKEY=0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
expect_failure "$fixture_dir/missing-bootstrap-state" \
  BOOTSTRAP_STATE_BIN="$fixture_dir/missing-bootstrap-state"
if [[ -e "$fixture_dir/cargo.log" ]]; then
  echo "preflight fell back to cargo although BOOTSTRAP_STATE_BIN was set" >&2
  exit 1
fi

# --- --profile: every value from the profile, the RPC URLs from the environment ---------------
profile_dir="$fixture_dir/config"
mkdir -p "$profile_dir"
profile="$profile_dir/destination-from-source.json"
jq -n \
  --arg local "$LOCAL_IBC_ROUTER" --arg source "$SOURCE_IBC_ROUTER" \
  --arg gateway "$SP1_VERIFIER_GATEWAY" --arg selector "$SP1_GROTH16_SELECTOR" \
  --arg update "$UPDATE_CLIENT_VKEY" --arg membership "$MEMBERSHIP_VKEY" \
  --arg non "$NON_MEMBERSHIP_VKEY" --arg root "$INITIAL_STATE_ROOT" \
  --arg validators "$INITIAL_VALIDATOR_SET_HASH" '{
    name: "destination-from-source", chainId: 1111,
    deployer: "0x4444444444444444444444444444444444444444",
    localIbcRouter: $local, sp1VerifierGateway: $gateway, sp1Groth16Selector: $selector,
    upgradeAuthority: "0x5555555555555555555555555555555555555555",
    proofMode: "groth16",
    source: {name: "source", chainId: 1337, ibcRouter: $source,
      validatorMode: "header-qbft"},
    vkeys: {updateClient: $update, membership: $membership, nonMembership: $non},
    bootstrap: {height: 7, stateRoot: $root, blockTimestamp: 1000, validatorSetHash: $validators},
    trustingPeriodSeconds: 1000, maxClockDriftSeconds: 20,
    registration: {clientId: "client-0", counterpartyClientId: "client-0"}}' >"$fixture_dir/profile.json"

unset SOURCE_RPC_URL DESTINATION_RPC_URL EXPECTED_SOURCE_CHAIN_ID SOURCE_VALIDATOR_MODE \
  SP1_PROOF_MODE LOCAL_IBC_ROUTER SOURCE_IBC_ROUTER SP1_VERIFIER_GATEWAY SP1_GROTH16_SELECTOR \
  UPDATE_CLIENT_VKEY MEMBERSHIP_VKEY NON_MEMBERSHIP_VKEY INITIAL_HEIGHT INITIAL_STATE_ROOT \
  INITIAL_BLOCK_TIMESTAMP INITIAL_VALIDATOR_SET_HASH TRUSTING_PERIOD_SECONDS MAX_CLOCK_DRIFT_SECONDS

# Writes the base profile through a jq filter, then runs preflight on it with extra env.
profile_run() {
  local filter="$1"
  shift
  jq "$filter" "$fixture_dir/profile.json" >"$profile"
  # The outer env sets the URLs; the inner one applies the case's own assignments and `-u`.
  env SOURCE_RPC_URL=http://127.0.0.1:8545 DESTINATION_RPC_URL=http://127.0.0.1:9545 env "$@" \
    "$repo_root/deploy/validate.sh" --profile "$profile" \
    >/dev/null 2>"$fixture_dir/stderr"
}

expect_profile_failure() {
  local expected="$1"
  shift
  if profile_run "$@"; then
    echo "preflight unexpectedly accepted profile: $expected" >&2
    exit 1
  fi
  # Exactly one line: the first refusal stops the run.
  if [[ "$(cat "$fixture_dir/stderr")" != *"$expected"* \
    || "$(wc -l <"$fixture_dir/stderr")" -ne 1 ]]; then
    echo "preflight failed for the wrong reason; expected only: $expected" >&2
    cat "$fixture_dir/stderr" >&2
    exit 1
  fi
}

profile_run . BOOTSTRAP_STATE_BIN="$fixture_dir/bootstrap-state" \
  || { cat "$fixture_dir/stderr" >&2; exit 1; }
expect_bootstrap_run bootstrap-state "$source_url $bootstrap_args" cargo
profile_run . FAKE_DESTINATION_TIME=1999 || { cat "$fixture_dir/stderr" >&2; exit 1; }
profile_run '.trustingPeriodSeconds = 0' FAKE_DESTINATION_TIME=999999999 \
  || { cat "$fixture_dir/stderr" >&2; exit 1; }
profile_run '.registration.clientId = "client-12"' FAKE_NEXT_CLIENT_SEQ=12 \
  || { cat "$fixture_dir/stderr" >&2; exit 1; }
# An address is compared without regard to its checksum case.
profile_run . FAKE_ROUTER_AUTHORITY=0x5555555555555555555555555555555555555555 \
  || { cat "$fixture_dir/stderr" >&2; exit 1; }
profile_run '.upgradeAuthority = "0xaBcDeF0000000000000000000000000000000000"' \
  FAKE_ROUTER_AUTHORITY=0xAbCdEf0000000000000000000000000000000000 \
  || { cat "$fixture_dir/stderr" >&2; exit 1; }
profile_run '.trustingPeriodSeconds = "1000"' || { cat "$fixture_dir/stderr" >&2; exit 1; }
# Read as decimal: octal 0001000 is 512, which would put the anchor out of date at 1999.
profile_run '.trustingPeriodSeconds = "0001000"' FAKE_DESTINATION_TIME=1999 \
  || { cat "$fixture_dir/stderr" >&2; exit 1; }

expect_profile_failure "profile field missing: .bootstrap.stateRoot" '.bootstrap.stateRoot = null'
expect_profile_failure "profile field missing: .localIbcRouter" 'del(.localIbcRouter)'
expect_profile_failure "profile field missing: .chainId" '.chainId = null'
expect_profile_failure "profile field missing: .source.ibcRouter" '.source.ibcRouter = {}'
expect_profile_failure "LOCAL_IBC_ROUTER is set in the environment" . \
  LOCAL_IBC_ROUTER=0x1111111111111111111111111111111111111111
expect_profile_failure "INITIAL_HEIGHT is set in the environment" . INITIAL_HEIGHT=
expect_profile_failure "missing or placeholder deployment input: DESTINATION_RPC_URL" . \
  -u DESTINATION_RPC_URL
expect_profile_failure "SOURCE_RPC_URL must be a non-empty http(s) URL" . SOURCE_RPC_URL=not-a-url
expect_profile_failure "profile name does not match its file name" '.name = "another-from-source"'
expect_profile_failure "destination chain ID mismatch" '.chainId = 2222'
expect_profile_failure "source chain ID mismatch" '.source.chainId = 1111'
expect_profile_failure "bootstrap height is outside the trusting period" . FAKE_DESTINATION_TIME=2000
expect_profile_failure \
  "registration.clientId is client-0 but the local Router's next client ID is client-1" . \
  FAKE_NEXT_CLIENT_SEQ=1
expect_profile_failure "profile field missing: .source.name" '.source.name = null'
expect_profile_failure "profile field missing: .registration.counterpartyClientId" \
  '.registration.counterpartyClientId = ""'
expect_profile_failure "profile field missing: .registration.clientId" '.registration.clientId = null'
expect_profile_failure "profile field missing: .registration.counterpartyClientId" \
  'del(.registration.counterpartyClientId)'
expect_profile_failure "profile field missing: .upgradeAuthority" '.upgradeAuthority = null'
expect_profile_failure "UPGRADE_AUTHORITY must be a non-zero EVM address" \
  '.upgradeAuthority = "0x0000000000000000000000000000000000000000"'
expect_profile_failure \
  "upgradeAuthority is 0x5555555555555555555555555555555555555555 but the local Router's authority is 0x6666666666666666666666666666666666666666" \
  . FAKE_ROUTER_AUTHORITY=0x6666666666666666666666666666666666666666
expect_profile_failure "upgrade authority has no deployed code" . \
  FAKE_NO_CODE_AT=0x5555555555555555555555555555555555555555
profile_run 'del(.vkeys)' || { cat "$fixture_dir/stderr" >&2; exit 1; }
profile_run '.vkeys.updateClient = "ignored legacy value"' || { cat "$fixture_dir/stderr" >&2; exit 1; }
expect_profile_failure "bootstrap values do not match the canonical source header" \
  '.bootstrap.validatorSetHash = "0x3333333333333333333333333333333333333333333333333333333333333333"'

# Every committed profile stops on a field that is still null rather than running.
for committed in "$repo_root"/config/*.json; do
  if "$repo_root/deploy/validate.sh" --profile "$committed" >/dev/null 2>"$fixture_dir/stderr"; then
    echo "committed profile $committed passed preflight with null fields" >&2
    exit 1
  fi
  grep --quiet "^profile field missing: " "$fixture_dir/stderr" || {
    echo "committed profile $committed failed for the wrong reason" >&2
    cat "$fixture_dir/stderr" >&2
    exit 1
  }
done

echo "deployment preflight fixture tests passed"
