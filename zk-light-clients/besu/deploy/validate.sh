#!/usr/bin/env bash
set -euo pipefail

# shellcheck shell=bash
# Internal chain checks and profile reader, shared with the deployment entrypoint.
# The integration harness and deploy/entrypoint.sh read the profile named by $profile.
# Nothing is defaulted: an absent, null or empty field stops the caller under `set -e`, so call it
# only in a plain assignment. An empty string is refused as ProfileJson.readString refuses it.

profile_field() {
  local path="$1"
  # shellcheck disable=SC2154 # $profile is set by the sourcing script.
  if ! jq -er "$path | select((type == \"string\" and . != \"\") or type == \"number\")" \
    "$profile"; then
    echo "profile field missing: $path" >&2
    exit 1
  fi
}

validate_deployment() {

  repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
  cast_bin="${CAST_BIN:-cast}"

  standalone=false
  if [[ "${1:-}" == --standalone ]]; then
    standalone=true
    shift
  fi

  required=(
    SOURCE_RPC_URL DESTINATION_RPC_URL EXPECTED_SOURCE_CHAIN_ID SOURCE_VALIDATOR_MODE
    SP1_PROOF_MODE LOCAL_IBC_ROUTER SOURCE_IBC_ROUTER SP1_VERIFIER_GATEWAY
    SP1_GROTH16_SELECTOR UPDATE_CLIENT_VKEY MEMBERSHIP_VKEY NON_MEMBERSHIP_VKEY
    INITIAL_HEIGHT INITIAL_STATE_ROOT INITIAL_BLOCK_TIMESTAMP INITIAL_VALIDATOR_SET_HASH
    TRUSTING_PERIOD_SECONDS MAX_CLOCK_DRIFT_SECONDS
  )

  # With --profile, chain settings come from the profile and VKeys from generated artifacts.
  # RPC URLs come from the environment. Without --profile every value comes
  # from the environment, as the e2e harness supplies them.
  profile_owned=()

  for name in "${required[@]}"; do
    [[ "$name" == *_RPC_URL ]] || profile_owned+=("$name")
  done
  profile=""
  if [[ "${1:-}" == "--profile" ]]; then
    profile="${2:?usage: deploy/validate.sh [--standalone] [--profile config/<name>.json]}"
  elif (( $# > 0 )); then
    echo "usage: deploy/validate.sh [--standalone] [--profile config/<name>.json]" >&2
    exit 64
  fi

  if [[ -n "$profile" ]]; then
    for name in "${profile_owned[@]}"; do
      if [[ -n "${!name+set}" ]]; then
        echo "$name is set in the environment; with --profile it comes from the profile only" >&2
        exit 1
      fi
    done
    # Plain assignments, so a missing field stops the run under `set -e`.
    profile_name="$(profile_field .name)"
    if [[ "$profile_name" != "$(basename "$profile" .json)" ]]; then
      echo "profile name does not match its file name" >&2
      exit 1
    fi
    DESTINATION_CHAIN_ID="$(profile_field .chainId)"
    # Checked by nothing else, but copied into the deployment output.
    profile_field .source.name >/dev/null
    for entry in \
      EXPECTED_SOURCE_CHAIN_ID:.source.chainId SOURCE_VALIDATOR_MODE:.source.validatorMode \
      SP1_PROOF_MODE:.proofMode LOCAL_IBC_ROUTER:.localIbcRouter \
      SOURCE_IBC_ROUTER:.source.ibcRouter SP1_VERIFIER_GATEWAY:.sp1VerifierGateway \
      SP1_GROTH16_SELECTOR:.sp1Groth16Selector \
      INITIAL_HEIGHT:.bootstrap.height INITIAL_STATE_ROOT:.bootstrap.stateRoot \
      INITIAL_BLOCK_TIMESTAMP:.bootstrap.blockTimestamp \
      INITIAL_VALIDATOR_SET_HASH:.bootstrap.validatorSetHash \
      TRUSTING_PERIOD_SECONDS:.trustingPeriodSeconds MAX_CLOCK_DRIFT_SECONDS:.maxClockDriftSeconds; do
      value="$(profile_field "${entry#*:}")"
      printf -v "${entry%%:*}" '%s' "$value"
    done
  fi

  if [[ -n "$profile" ]]; then
    keys="${VKEYS_FILE:-$repo_root/artifacts/vkeys.json}"
    UPDATE_CLIENT_VKEY="$(jq -er '.["update-client"].programVKey' "$keys")"
    MEMBERSHIP_VKEY="$(jq -er '.membership.programVKey' "$keys")"
    NON_MEMBERSHIP_VKEY="$(jq -er '.["non-membership"].programVKey' "$keys")"
  fi

  for name in "${required[@]}"; do
    if [[ -z "${!name:-}" || "${!name}" == *PLACEHOLDER* || "${!name}" == *CHANGE_ME* ]]; then
      echo "missing or placeholder deployment input: $name" >&2
      exit 1
    fi
  done

  if [[ "$SOURCE_VALIDATOR_MODE" != "header-qbft" ]]; then
    echo "SOURCE_VALIDATOR_MODE must be header-qbft" >&2
    exit 1
  fi
  if [[ "$SP1_PROOF_MODE" != "groth16" ]]; then
    echo "SP1_PROOF_MODE must be groth16" >&2
    exit 1
  fi

  require_rpc_url() {
    local name="$1"
    local value="${!name}"
    if [[ ! "$value" =~ ^https?://[^[:space:]]+$ ]]; then
      echo "$name must be a non-empty http(s) URL" >&2
      exit 1
    fi
  }
  require_rpc_url SOURCE_RPC_URL
  require_rpc_url DESTINATION_RPC_URL

  require_uint64() {
    local name="$1"
    local value="${!name}"
    if [[ ! "$value" =~ ^[0-9]+$ ]]; then
      echo "$name must be an unsigned decimal integer" >&2
      exit 1
    fi
    value="${value##+(0)}"
    value="${value:-0}"
    if (( ${#value} > 20 )) || { (( ${#value} == 20 )) && [[ "$value" > 18446744073709551615 ]]; }; then
      echo "$name does not fit uint64" >&2
      exit 1
    fi
  }
  shopt -s extglob
  for name in EXPECTED_SOURCE_CHAIN_ID INITIAL_HEIGHT INITIAL_BLOCK_TIMESTAMP \
    TRUSTING_PERIOD_SECONDS MAX_CLOCK_DRIFT_SECONDS; do
    require_uint64 "$name"
  done
  for name in EXPECTED_SOURCE_CHAIN_ID INITIAL_HEIGHT INITIAL_BLOCK_TIMESTAMP; do
    if [[ "${!name}" =~ ^0+$ ]]; then
      echo "$name must be non-zero" >&2
      exit 1
    fi
  done

  require_nonzero_address() {
    local name="$1"
    local value="${!name}"
    if [[ ! "$value" =~ ^0x[0-9a-fA-F]{40}$ || "$value" == "0x0000000000000000000000000000000000000000" ]]; then
      echo "$name must be a non-zero EVM address" >&2
      exit 1
    fi
  }
  for name in LOCAL_IBC_ROUTER SOURCE_IBC_ROUTER SP1_VERIFIER_GATEWAY; do
    require_nonzero_address "$name"
  done

  require_bytes32() {
    local name="$1"
    local value="${!name}"
    if [[ ! "$value" =~ ^0x[0-9a-fA-F]{64}$ || "$value" == "0x0000000000000000000000000000000000000000000000000000000000000000" ]]; then
      echo "$name must be a non-zero bytes32 value" >&2
      exit 1
    fi
  }
  for name in UPDATE_CLIENT_VKEY MEMBERSHIP_VKEY NON_MEMBERSHIP_VKEY \
    INITIAL_STATE_ROOT INITIAL_VALIDATOR_SET_HASH; do
    require_bytes32 "$name"
  done
  if [[ "$UPDATE_CLIENT_VKEY" == "$MEMBERSHIP_VKEY" \
    || "$UPDATE_CLIENT_VKEY" == "$NON_MEMBERSHIP_VKEY" \
    || "$MEMBERSHIP_VKEY" == "$NON_MEMBERSHIP_VKEY" ]]; then
    echo "the three program VKeys must be distinct" >&2
    exit 1
  fi
  if [[ ! "$SP1_GROTH16_SELECTOR" =~ ^0x[0-9a-fA-F]{8}$ ]]; then
    echo "SP1_GROTH16_SELECTOR must be exactly four bytes" >&2
    exit 1
  fi

  expected_vkeys="${VKEYS_FILE:-$repo_root/artifacts/vkeys.json}"
  for entry in \
    "update-client:UPDATE_CLIENT_VKEY" \
    "membership:MEMBERSHIP_VKEY" \
    "non-membership:NON_MEMBERSHIP_VKEY"; do
    program="${entry%%:*}"
    name="${entry##*:}"
    expected="$(jq -r --arg program "$program" '.[$program].programVKey' "$expected_vkeys")"
    actual="${!name}"
    if [[ "$(printf '%s' "$actual" | tr '[:upper:]' '[:lower:]')" != "$(printf '%s' "$expected" | tr '[:upper:]' '[:lower:]')" ]]; then
      echo "$name does not match generated artifacts/vkeys.json" >&2
      exit 1
    fi
  done

  # An RPC URL can carry a token, so it reaches cast in the one process's environment, never as an
  # argument, which every process on the host can read. FOUNDRY_ETH_RPC_URL is the variable both
  # cast and `forge script` read.
  on_source() { FOUNDRY_ETH_RPC_URL="$SOURCE_RPC_URL" "$@"; }
  on_destination() { FOUNDRY_ETH_RPC_URL="$DESTINATION_RPC_URL" "$@"; }

  source_chain_id="$(on_source "$cast_bin" rpc eth_chainId | tr -d '"')"
  source_chain_id="$("$cast_bin" to-dec "$source_chain_id")"
  if [[ "$source_chain_id" != "$EXPECTED_SOURCE_CHAIN_ID" ]]; then
    echo "source chain ID mismatch" >&2
    exit 1
  fi

  if [[ -n "$profile" ]]; then
    destination_chain_id="$(on_destination "$cast_bin" rpc eth_chainId | tr -d '"')"
    destination_chain_id="$("$cast_bin" to-dec "$destination_chain_id")"
    if [[ "$destination_chain_id" != "$DESTINATION_CHAIN_ID" ]]; then
      echo "destination chain ID mismatch" >&2
      exit 1
    fi
    # The expiry test in `updateClient`, where a zero trusting period never expires. Bash integers
    # are signed 64-bit, so an operand past 18 digits is taken as an expiry far in the future;
    # DeployLightClient repeats the check exactly.
    destination_time="$(on_destination "$cast_bin" block latest --field timestamp)"
    if (( ${#INITIAL_BLOCK_TIMESTAMP} <= 18 && ${#TRUSTING_PERIOD_SECONDS} <= 18 )) \
      && (( 10#$TRUSTING_PERIOD_SECONDS != 0 \
        && 10#$INITIAL_BLOCK_TIMESTAMP + 10#$TRUSTING_PERIOD_SECONDS <= 10#$destination_time )); then
      echo "bootstrap height is outside the trusting period on the destination chain" >&2
      exit 1
    fi
  fi

  require_code() {
    local chain="$1"
    local address="$2"
    local label="$3"
    local code
    code="$("$chain" "$cast_bin" code "$address")"
    if [[ "$code" == "0x" || "$code" == "0x0" ]]; then
      echo "$label has no deployed code" >&2
      exit 1
    fi
  }
  if [[ "$standalone" == false ]]; then
    require_code on_source "$SOURCE_IBC_ROUTER" "source Router"
    require_code on_destination "$LOCAL_IBC_ROUTER" "local Router"
  fi
  require_code on_destination "$SP1_VERIFIER_GATEWAY" "SP1VerifierGateway"

  route=()
  while IFS= read -r line; do route+=("$line"); done < <(
    on_destination "$cast_bin" call "$SP1_VERIFIER_GATEWAY" \
      "routes(bytes4)(address,bool)" "$SP1_GROTH16_SELECTOR"
  )
  if [[ "${route[0]:-}" == "0x0000000000000000000000000000000000000000" \
    || "${route[1]:-}" != "false" ]]; then
    echo "Groth16 gateway route is missing or frozen" >&2
    exit 1
  fi

  # Registration runs after deployment. Sequential client-N IDs must be next in the Router's
  # sequence. Other IDs use the custom-ID overload, so preflight simulates that exact call from the
  # profile deployer before a client is deployed that could not be registered.
  if [[ -n "$profile" && "$standalone" == false ]]; then
    client_id="$(profile_field .registration.clientId)"
    counterparty_client_id="$(profile_field .registration.counterpartyClientId)"
    if [[ "$client_id" =~ ^client-[0-9]+$ ]]; then
      next_client_seq="$(on_destination "$cast_bin" call "$LOCAL_IBC_ROUTER" \
        "getNextClientSeq()(uint256)" | awk '{ print $1 }')"
      if [[ "client-$next_client_seq" != "$client_id" ]]; then
        echo "registration.clientId is $client_id but the local Router's next client ID is" \
          "client-$next_client_seq" >&2
        exit 1
      fi
    else
      deployer="$(profile_field .deployer)"
      if existing_client="$(on_destination "$cast_bin" call "$LOCAL_IBC_ROUTER" \
        "getClient(string)(address)" "$client_id" 2>&1)"; then
        echo "registration.clientId is already registered: $client_id ($existing_client)" >&2
        exit 1
      elif [[ "$existing_client" != *"0xa0db16fe"* ]]; then
        echo "$existing_client" >&2
        exit 1
      fi
      if ! custom_registration="$(on_destination "$cast_bin" call "$LOCAL_IBC_ROUTER" \
        "addClient(string,(string,bytes[]),address)" \
        "$client_id" "($counterparty_client_id,[0x])" "$SP1_VERIFIER_GATEWAY" \
        --from "$deployer" 2>&1)"; then
        echo "custom client registration simulation failed for $client_id" >&2
        echo "$custom_registration" >&2
        exit 1
      fi
    fi

    # The client's upgrades are authorized by the AccessManager that governs the local Router.
    UPGRADE_AUTHORITY="$(profile_field .upgradeAuthority)"
    require_nonzero_address UPGRADE_AUTHORITY
    router_authority="$(on_destination "$cast_bin" call "$LOCAL_IBC_ROUTER" \
      "authority()(address)")"
    if [[ "$(printf '%s' "$router_authority" | tr '[:upper:]' '[:lower:]')" != "$(printf '%s' "$UPGRADE_AUTHORITY" | tr '[:upper:]' '[:lower:]')" ]]; then
      echo "upgradeAuthority is $UPGRADE_AUTHORITY but the local Router's authority is" \
        "$router_authority" >&2
      exit 1
    fi
    require_code on_destination "$UPGRADE_AUTHORITY" "upgrade authority"
  fi

  bootstrap="$(mktemp)"
  trap 'rm -f "$bootstrap"' EXIT
  cd "$repo_root"
  if [[ -n "${BOOTSTRAP_STATE_BIN:-}" ]]; then
    bootstrap_cmd=("$BOOTSTRAP_STATE_BIN")
  else
    bootstrap_cmd=(cargo run --quiet --locked -p prover-script --bin bootstrap-state --)
  fi
  # bootstrap-state reads SOURCE_RPC_URL from its environment.
  SOURCE_RPC_URL="$SOURCE_RPC_URL" "${bootstrap_cmd[@]}" \
    --expected-source-chain-id "$EXPECTED_SOURCE_CHAIN_ID" \
    --height "$INITIAL_HEIGHT" > "$bootstrap"
  if [[ "$(jq -r .stateRoot "$bootstrap")" != "$INITIAL_STATE_ROOT" \
    || "$(jq -r .blockTimestamp "$bootstrap")" != "$INITIAL_BLOCK_TIMESTAMP" \
    || "$(jq -r .validatorSetHash "$bootstrap")" != "$INITIAL_VALIDATOR_SET_HASH" ]]; then
    echo "bootstrap values do not match the canonical source header" >&2
    exit 1
  fi

  # Source-layout conformance is an E2E/release integration check, not a dependency
  # of deployment or registration against an existing Router.
  echo "deployment preflight passed"

}

if [[ "${BASH_SOURCE[0]}" == "$0" ]]; then
  validate_deployment "$@"
fi
