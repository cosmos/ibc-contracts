#!/usr/bin/env bash
set -euo pipefail
shopt -s inherit_errexit 2>/dev/null || true

# The mounted deployment entrypoint. Chain agnostic: the profile names the chains.
#
#   preflight <profile>             checks only; nothing is signed
#   bootstrap <profile> [height]    prints the profile's `bootstrap` object for a source block,
#                                   the source head when no height is given
#   deploy    <profile>             standalone preflight and DeployLightClient only
#   register  <profile> <address>   RegisterLightClient for a client (proxy) already deployed
#
# <profile> names $APP_DIR/config/<profile>.json. Runtime inputs, each a mounted file: the destination
# and source RPC URLs at $DESTINATION_RPC_URL_FILE and $SOURCE_RPC_URL_FILE (bootstrap reads the
# source only), and the deployer key at $DEPLOYER_KEY_FILE.
# stdout carries only JSON: the bootstrap object, or the deployment output; the rest goes to
# stderr.

usage() {
  echo "usage: (preflight|deploy) <profile> | bootstrap <profile> [height] |" \
    "register <profile> <address>" >&2
  exit 64
}

app="${APP_DIR:-/app}"
key_file="${DEPLOYER_KEY_FILE:-/run/secrets/deployer-key}"
destination_url_file="${DESTINATION_RPC_URL_FILE:-/run/secrets/destination-rpc-url}"
source_url_file="${SOURCE_RPC_URL_FILE:-/run/secrets/source-rpc-url}"
command="${1:-}"
name="${2:-}"
case "$command" in
  preflight | deploy) (( $# == 2 )) || usage ;;
  bootstrap) (( $# == 2 || $# == 3 )) || usage ;;
  register) (( $# == 3 )) || usage ;;
  *) usage ;;
esac
[[ "$name" =~ ^[A-Za-z0-9][A-Za-z0-9._-]*$ ]] || { echo "invalid profile name: $name" >&2; exit 64; }

# Forge may read ../config only, relative to contracts/.
profile="$app/config/$name.json"
[[ -f "$profile" ]] || { echo "no profile $name; mount it at $profile" >&2; exit 1; }

# A key or an RPC URL, which can carry a token, in the container's own environment is in its
# config and visible to `docker inspect`.
if [[ -n "${DEPLOYER_PRIVATE_KEY+set}" ]]; then
  echo "DEPLOYER_PRIVATE_KEY is set in the container environment; mount the key file at" \
    "$key_file instead" >&2
  exit 1
fi
for variable in DESTINATION_RPC_URL SOURCE_RPC_URL ETH_RPC_URL FOUNDRY_ETH_RPC_URL; do
  if [[ -n "${!variable+set}" ]]; then
    echo "$variable is set in the container environment; mount the RPC URLs as files" >&2
    exit 1
  fi
done

# Prints the http(s) URL held in a mounted file.
read_url() {
  local file="$1" url
  [[ -r "$file" ]] || { echo "cannot read the RPC URL file $file" >&2; return 1; }
  url="$(tr -d '\r\n' <"$file")"
  [[ "$url" =~ ^https?://[^[:space:]]+$ ]] \
    || { echo "$file does not hold an http(s) URL" >&2; return 1; }
  printf '%s\n' "$url"
}
# The URLs stay unexported and reach each cast or forge process in its own environment, never as
# an argument, which every process on the host can read. FOUNDRY_ETH_RPC_URL, not ETH_RPC_URL:
# `forge script` ignores ETH_RPC_URL, skips the broadcast and still exits 0.
source_url="$(read_url "$source_url_file")"
[[ "$command" == bootstrap ]] || destination_url="$(read_url "$destination_url_file")"
on_source() { FOUNDRY_ETH_RPC_URL="$source_url" "$@"; }
on_destination() { FOUNDRY_ETH_RPC_URL="$destination_url" "$@"; }

exec 3>&1 1>&2

# shellcheck source=deploy/validate.sh
source "$app/deploy/validate.sh"

if [[ "$command" == bootstrap ]]; then
  source_chain_id="$(profile_field .source.chainId)"
  height="${3:-$(on_source cast block-number)}"
  SOURCE_RPC_URL="$source_url" bootstrap-state --expected-source-chain-id "$source_chain_id" \
    --height "$height" | jq '{height, stateRoot, blockTimestamp, validatorSetHash}' >&3
  echo "source block $height is $(on_source cast block "$height" --field hash)"
  exit 0
fi

# Build a private effective profile from chain settings and the generated VKeys.
# Resolve the identity file once so a concurrent circuit build cannot change this deployment.
original_profile="$profile"
mkdir -p "$app/deployments"
work="$(mktemp -d "$app/deployments/.profile.XXXXXX")"
trap 'rm -rf "$work"' EXIT
cp "$app/artifacts/vkeys.json" "$work/vkeys.json"
export VKEYS_FILE="$work/vkeys.json"
jq --slurpfile keys "$VKEYS_FILE" '
  .vkeys = {
    updateClient: $keys[0]["update-client"].programVKey,
    membership: $keys[0].membership.programVKey,
    nonMembership: $keys[0]["non-membership"].programVKey
  }
' "$profile" >"$work/$name.json"
profile="$work/$name.json"
# Isolate the validator's working variables from the deployment entrypoint.
(
  export DESTINATION_RPC_URL="$destination_url" SOURCE_RPC_URL="$source_url"
  if [[ "$command" == register ]]; then
    validate_deployment --profile "$profile"
  else
    validate_deployment --standalone --profile "$profile"
  fi
)
[[ "$command" == preflight ]] && exit 0

if [[ ! -r "$key_file" ]]; then
  echo "cannot read the deployer key at $key_file; it must be readable by uid $(id -u)" >&2
  exit 1
fi
deployer_key="$(tr -d '\r\n' <"$key_file")"
if [[ ! "$deployer_key" =~ ^(0x)?[0-9a-fA-F]{64}$ ]]; then
  echo "$key_file does not hold a 32-byte hex private key" >&2
  exit 1
fi
# vm.envUint reads an unprefixed key of decimal digits as a decimal number: another key.
deployer_key="0x${deployer_key#0x}"

chain_id="$(profile_field .chainId)"
height="$(profile_field .bootstrap.height)"
cd "$app/contracts"

# Runs a Forge script against the destination, broadcasting, and prints the path of its broadcast
# file. The key and the URL are passed only in the environment of the forge process.
broadcast() {
  local script="$1" sig="$2"
  shift 2
  local file="broadcast/$script/$chain_id/run-latest.json"
  rm -f "$file"
  DEPLOYER_PRIVATE_KEY="$deployer_key" on_destination forge script "script/$script" \
    --sig "$sig" "$@" --broadcast --slow --non-interactive >&2 || return 1
  printf '%s\n' "$file"
}

# DeployLightClient's broadcast: exactly the implementation and the proxy, each created as the
# script's named returns say, each with a successful receipt.
deployed() {
  jq -ec '
    def one($what): if length == 1 then .[0] else error("expected one \($what)") end;
    def created($name): [.transactions[]
      | select(.transactionType == "CREATE" and .contractName == $name)] | one("\($name) creation");
    def same($a; $b): ($a | ascii_downcase) == ($b | ascii_downcase);
    . as $run
    | def receipt($tx): [$run.receipts[] | select(.transactionHash == $tx.hash)]
        | one("receipt for \($tx.hash)")
        | if .status == "0x1" then . else error("\($tx.hash) failed") end;
    if (.transactions | length) != 2 then error("expected two transactions") else . end
    | created("ERC1967Proxy") as $proxy
    | created("BesuQBFTZKLightClient") as $implementation
    | if same($proxy.contractAddress; .returns.lightClient.value)
        and same($implementation.contractAddress; .returns.implementation.value)
      then . else error("the creations do not match the returned addresses") end
    | {lightClient: .returns.lightClient.value,
       implementation: .returns.implementation.value,
       receipt: receipt($proxy), implementationReceipt: receipt($implementation)}' "$1"
}

# The implementation the proxy delegates to, from its ERC-1967 slot on the destination chain.
implementation_of() {
  local slot=0x360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc word
  word="$(on_destination cast storage "$1" "$slot")"
  [[ "$word" =~ ^0x0{24}([0-9a-fA-F]{40})$ \
    && "${BASH_REMATCH[1]}" != "$(printf '0%.0s' {1..40})" ]] \
    || { echo "$1 is not an ERC-1967 proxy" >&2; return 1; }
  printf '0x%s\n' "${BASH_REMATCH[1]}"
}

# The deployment output: the profile's public fields, read by name so nothing else in the file is
# copied, and what the deployment produced. No RPC URL and no key is in scope here.
output() {
  # Assigned first: a failure inside a jq argument would not stop the script. The client is already
  # deployed here, so an unreadable block hash is recorded as null rather than losing the output.
  local profile_sha256 block_hash
  profile_sha256="$(sha256sum "$original_profile" | cut -d' ' -f1)"
  if ! block_hash="$(on_source cast block "$height" --field hash)"; then
    echo "could not read the source block hash at $height; bootstrap.blockHash is null" >&2
    block_hash=""
  fi
  jq -n \
    --slurpfile profile "$profile" \
    --arg profileSha256 "$profile_sha256" \
    --arg blockHash "$block_hash" \
    --argjson registered "$registered" \
    --arg lightClient "$light_client" \
    --arg implementation "$implementation" \
    --argjson deployReceipt "${deploy_receipt:-null}" \
    --argjson implementationReceipt "${implementation_receipt:-null}" \
    --argjson registerReceipt "${register_receipt:-null}" \
    --arg imageId "${IMAGE_ID:-}" --arg imageTag "${IMAGE_TAG:-}" \
    --arg gitCommit "${GIT_COMMIT:-}" \
    --arg sp1ContractsCommit "${SP1_CONTRACTS_COMMIT:-}" \
    --arg recordedAt "$(date -u +%Y-%m-%dT%H:%M:%SZ)" '
    def orNull: if . == "" then null else . end;
    def block: if . == null then null else .blockNumber | ltrimstr("0x")
      | ascii_downcase | explode
      | reduce .[] as $c (0; . * 16 + ($c | if . >= 97 then . - 87 else . - 48 end)) end;
    $profile[0] as $p
    | {
        name: $p.name,
        chainId: $p.chainId,
        deployer: $p.deployer,
        localIbcRouter: $p.localIbcRouter,
        sp1VerifierGateway: $p.sp1VerifierGateway,
        upgradeAuthority: $p.upgradeAuthority,
        sp1Groth16Selector: $p.sp1Groth16Selector,
        proofMode: $p.proofMode,
        source: ($p.source | {name, chainId, ibcRouter, validatorMode}),
        vkeys: ($p.vkeys | {updateClient, membership, nonMembership}),
        bootstrap: (($p.bootstrap | {height, stateRoot, blockTimestamp, validatorSetHash})
          + {blockHash: ($blockHash | orNull)}),
        trustingPeriodSeconds: $p.trustingPeriodSeconds,
        maxClockDriftSeconds: $p.maxClockDriftSeconds,
        registration: ($p.registration | {clientId, counterpartyClientId}),
        deployment: {
          lightClient: $lightClient,
          implementation: $implementation,
          registered: $registered,
          deployTransaction: $deployReceipt.transactionHash,
          deployBlock: ($deployReceipt | block),
          implementationTransaction: $implementationReceipt.transactionHash,
          registerTransaction: $registerReceipt.transactionHash,
          registerBlock: ($registerReceipt | block),
          profileSha256: $profileSha256,
          image: {id: ($imageId | orNull), tag: ($imageTag | orNull)},
          gitCommit: ($gitCommit | orNull),
          sp1ContractsCommit: ($sp1ContractsCommit | orNull),
          recordedAt: $recordedAt
        }
      }' >&3
}

if [[ "$command" == deploy ]]; then
  deploy_file="$(broadcast DeployLightClient.s.sol 'run(string)' "$profile")"
  deployment="$(deployed "$deploy_file")"
  light_client="$(jq -er .lightClient <<<"$deployment")"
  implementation="$(jq -er .implementation <<<"$deployment")"
  deploy_receipt="$(jq -c .receipt <<<"$deployment")"
  implementation_receipt="$(jq -c .implementationReceipt <<<"$deployment")"
  on_chain="$(implementation_of "$light_client")"
  if [[ "$(printf '%s' "$on_chain" | tr '[:upper:]' '[:lower:]')" != "$(printf '%s' "$implementation" | tr '[:upper:]' '[:lower:]')" ]]; then
    echo "proxy $light_client delegates to $on_chain, not to $implementation" >&2
    exit 1
  fi
  echo "deployed $light_client (implementation $implementation); registration is a separate action"
  registered=false
  output
  exit 0
else
  light_client="$3"
  implementation="$(implementation_of "$light_client")"
fi

# A client that deployed but did not register is still recorded, so the address is not lost.
client_id="$(profile_field .registration.clientId)"
router="$(profile_field .localIbcRouter)"
run_status=0
if ! register_receipt="$(broadcast RegisterLightClient.s.sol 'run(string,address)' \
  "$profile" "$light_client")" \
  || ! register_receipt="$(jq -ec '.receipts | if length == 1 and .[0].status == "0x1"
    then .[0] else error("expected one successful receipt") end' "$register_receipt")"; then
  register_receipt=""
  run_status=1
fi

# `registered` is what the Router says, whatever the run reported: getClient reverts for an ID it
# does not hold.
registered=false
if named="$(on_destination cast call "$router" "getClient(string)(address)" "$client_id")" && [[ "$(printf '%s' "$named" | tr '[:upper:]' '[:lower:]')" == "$(printf '%s' "$light_client" | tr '[:upper:]' '[:lower:]')" ]]; then
  registered=true
fi

if [[ "$registered" == true ]]; then
  (( run_status == 0 )) \
    || echo "the registration run failed, but the Router names $light_client as $client_id" >&2
  echo "registered $light_client as $client_id"
  output
  exit 0
fi
output
if (( run_status == 0 )); then
  echo "the registration transaction succeeded, but the Router does not name $light_client as" \
    "$client_id" >&2
else
  echo "registration failed; the output above has registered false. Rerun with:" \
    "register $name $light_client" >&2
fi
exit 1
