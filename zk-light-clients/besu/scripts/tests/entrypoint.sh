#!/usr/bin/env bash
set -euo pipefail

# Tests deploy/entrypoint.sh against fake forge, cast and bootstrap-state. Runs in the image's test
# stage, where the entrypoint's /app layout exists.

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
fixture_dir="$(mktemp -d)"
app="$fixture_dir/app"
mkdir -p "$app/config" "$app/contracts" "$app/scripts" "$app/deploy" "$app/artifacts" "$app/release"
cp "$repo_root/deploy/validate.sh" "$app/deploy/"
cp "$repo_root/deploy/entrypoint.sh" "$app/deploy/"
cp "$repo_root/release/expected-vkeys.json" "$app/release/"
cp "$repo_root/release/expected-vkeys.json" "$app/artifacts/vkeys.json"
export APP_DIR="$app"
profile="$app/config/fixture.json"
trap 'rm -rf "$fixture_dir"' EXIT

key=0x$(printf 'ab%.0s' {1..32})
router=0x1111111111111111111111111111111111111111
client=0x4444444444444444444444444444444444444444
implementation=0x6666666666666666666666666666666666666666
authority=0x7777777777777777777777777777777777777777
root=0x$(printf '1%.0s' {1..64})
validators=0x$(printf '2%.0s' {1..64})

# The RPC URLs carry a planted token. The fakes refuse one among their arguments and take it only
# from their environment, FOUNDRY_ETH_RPC_URL for cast and forge, SOURCE_RPC_URL for
# bootstrap-state.
destination_url="http://destination/?token=PLANTED-TOKEN-D"
source_url="http://source/?token=PLANTED-TOKEN-S"
cat >"$fixture_dir/cast" <<'EOF'
#!/usr/bin/env bash
set -euo pipefail
[[ "$*" != *PLANTED* ]] || { echo "cast was given an RPC URL as an argument" >&2; exit 97; }
[[ "$1" == to-dec || "${FOUNDRY_ETH_RPC_URL:-}" == *PLANTED* ]] \
  || { echo "cast has no FOUNDRY_ETH_RPC_URL" >&2; exit 97; }
if [[ "${FAKE_REFUSE_ROUTER_CALLS:-}" == 1 && "$1" == call && "$2" == 0x1111111111111111111111111111111111111111 ]]; then
  echo "standalone deployment called the Router" >&2
  exit 98
fi
case "$1" in
  rpc) if [[ "$FOUNDRY_ETH_RPC_URL" == http://destination/* ]]; then printf '"0x457"\n'
    else printf '"0x539"\n'; fi ;;
  to-dec) printf '%d\n' "$2" ;;
  code) printf '0x6000\n' ;;
  call)
    if [[ "$3" == "getNextClientSeq()(uint256)" ]]; then printf '0\n'
    elif [[ "$3" == "authority()(address)" ]]; then printf '0x7777777777777777777777777777777777777777\n'
    # The Router names a client once the fake forge has registered it, and reverts otherwise.
    elif [[ "$3" == "getClient(string)(address)" ]]; then cat "$FAKE_ROUTER_STATE"
    else printf '0x1111111111111111111111111111111111111111\nfalse\n'; fi ;;
  block)
    if [[ "$*" == *"--field hash"* ]]; then
      [[ -z "${FAKE_SOURCE_HASH_FAILS:-}" ]] || exit 1
      printf '0x%064d\n' 7
    else printf '1060\n'; fi ;;
  block-number) printf '7\n' ;;
  # The ERC-1967 implementation slot.
  storage) printf '0x%024d%s\n' 0 "${FAKE_IMPLEMENTATION_SLOT:-6666666666666666666666666666666666666666}" ;;
  *) exit 64 ;;
esac
EOF
cat >"$fixture_dir/bootstrap-state" <<EOF
#!/usr/bin/env bash
[[ "\$*" != *PLANTED* ]] || { echo "bootstrap-state was given an RPC URL as an argument" >&2; exit 97; }
[[ "\${SOURCE_RPC_URL:-}" == "$source_url" ]] || { echo "bootstrap-state has no SOURCE_RPC_URL" >&2; exit 97; }
printf '{"height":7,"stateRoot":"$root","blockTimestamp":1000,"validatorSetHash":"$validators","validatorCount":4}\n'
EOF
# `forge test` (the storage-layout check) passes. `forge script` writes the broadcast file a run
# would, in Forge 1.8.1's shape, after checking the key reached it through the environment and not
# as an argument. The deployment is transaction 1 (the implementation) and 2 (the proxy), the
# registration transaction 3. FAKE_DEPLOY_FILTER and FAKE_REGISTER_FILTER edit the file with jq.
# A registration reaches the fake Router unless FAKE_REGISTER_FAILS or FAKE_ROUTER_FORGETS is set;
# FAKE_REGISTER_LANDED makes forge fail after it has.
cat >"$fixture_dir/forge" <<EOF
#!/usr/bin/env bash
set -euo pipefail
[[ "\$1" == script ]] || exit 0
[[ "\${DEPLOYER_PRIVATE_KEY:-}" == "$key" ]] || { echo "key missing from forge's env" >&2; exit 99; }
[[ "\$*" != *"${key#0x}"* ]] || { echo "key passed as an argument" >&2; exit 99; }
[[ "\$*" != *PLANTED* ]] || { echo "RPC URL passed as an argument" >&2; exit 99; }
[[ "\${FOUNDRY_ETH_RPC_URL:-}" == "$destination_url" ]] \
  || { echo "forge has no FOUNDRY_ETH_RPC_URL for the destination" >&2; exit 99; }
# The Forge boundary must receive the generated effective profile, not the raw source profile.
jq -e --slurpfile keys "$app/artifacts/vkeys.json" \
  '.vkeys.membership == \$keys[0].membership.programVKey' "\$5" >/dev/null
script="\$(basename "\$2")"
[[ "\$script" != RegisterLightClient.s.sol || -z "\${FAKE_REGISTER_FAILS:-}" ]] || exit 1
if [[ "\$script" == RegisterLightClient.s.sol && -z "\${FAKE_ROUTER_FORGETS:-}" ]]; then
  printf '%s\n' "$client" >"$fixture_dir/router-client"
  [[ -z "\${FAKE_REGISTER_LANDED:-}" ]] || exit 1
fi
mkdir -p "$app/contracts/broadcast/\$script/1111"
if [[ "\$script" == DeployLightClient.s.sol ]]; then
  jq -n --arg i "$implementation" --arg p "$client" '{
    transactions: [
      {hash: "0x\(1 | tostring | ("0" * 63) + .)", transactionType: "CREATE",
        contractName: "BesuQBFTZKLightClient", contractAddress: \$i},
      {hash: "0x\(2 | tostring | ("0" * 63) + .)", transactionType: "CREATE",
        contractName: "ERC1967Proxy", contractAddress: \$p}],
    receipts: [
      {status: "0x1", transactionHash: "0x\(1 | tostring | ("0" * 63) + .)", contractAddress: \$i,
        blockNumber: "0x1b"},
      {status: "0x1", transactionHash: "0x\(2 | tostring | ("0" * 63) + .)", contractAddress: \$p,
        blockNumber: "0x1c"}],
    returns: {lightClient: {internal_type: "contract BesuQBFTZKLightClient", value: (\$p | ascii_upcase | sub("^0X"; "0x"))},
      implementation: {internal_type: "address", value: \$i}}}' \
    | jq "\${FAKE_DEPLOY_FILTER:-.}" >"$app/contracts/broadcast/\$script/1111/run-latest.json"
else
  jq -n '{transactions: [{hash: "0x\(3 | tostring | ("0" * 63) + .)", transactionType: "CALL"}],
    receipts: [{status: "0x1", transactionHash: "0x\(3 | tostring | ("0" * 63) + .)",
      contractAddress: null, blockNumber: "0x1d"}]}' \
    | jq "\${FAKE_REGISTER_FILTER:-.}" >"$app/contracts/broadcast/\$script/1111/run-latest.json"
fi
EOF
chmod +x "$fixture_dir/cast" "$fixture_dir/bootstrap-state" "$fixture_dir/forge"
printf '%s\n' "$key" >"$fixture_dir/key"
printf '%s\n' "$destination_url" >"$fixture_dir/destination-rpc-url"
printf '%s\n' "$source_url" >"$fixture_dir/source-rpc-url"

# Fields no output may carry: a key and an RPC token planted in the profile, and the free text.
jq -n --arg router "$router" --arg authority "$authority" --arg root "$root" --arg validators "$validators" --arg key "$key" \
  --slurpfile vkeys "$app/release/expected-vkeys.json" '{
    name: "fixture", description: "d", notes: ["n"], chainId: 1111,
    deployer: "0x9999999999999999999999999999999999999999",
    localIbcRouter: $router, sp1VerifierGateway: "0x3333333333333333333333333333333333333333",
    upgradeAuthority: $authority,
    sp1Groth16Selector: "0x12345678", proofMode: "groth16",
    source: {name: "source", chainId: 1337, ibcRouter: "0x2222222222222222222222222222222222222222",
      validatorMode: "header-qbft", rpc: {url: "http://source/?token=PLANTED-TOKEN"}},
    vkeys: {updateClient: $vkeys[0]["update-client"].programVKey,
      membership: $vkeys[0].membership.programVKey,
      nonMembership: $vkeys[0]["non-membership"].programVKey},
    bootstrap: {height: 7, stateRoot: $root, blockTimestamp: 1000, validatorSetHash: $validators},
    trustingPeriodSeconds: 1000, maxClockDriftSeconds: 20,
    registration: {clientId: "client-0", counterpartyClientId: "client-3"},
    privateKey: $key, rpc: {url: "http://destination/?token=PLANTED-TOKEN"}}' >"$profile"

run() {
  rm -f "$fixture_dir/router-client"
  env PATH="$fixture_dir:$PATH" BOOTSTRAP_STATE_BIN="$fixture_dir/bootstrap-state" \
    FAKE_ROUTER_STATE="$fixture_dir/router-client" \
    CAST_BIN="$fixture_dir/cast" FORGE_BIN="$fixture_dir/forge" \
    DESTINATION_RPC_URL_FILE="$fixture_dir/destination-rpc-url" \
    SOURCE_RPC_URL_FILE="$fixture_dir/source-rpc-url" \
    DEPLOYER_KEY_FILE="$fixture_dir/key" IMAGE_ID=sha256:image IMAGE_TAG=tag "$@" \
    "$app/deploy/entrypoint.sh" "${arguments[@]}" >"$fixture_dir/stdout" 2>"$fixture_dir/stderr"
}

fail() {
  echo "$1" >&2
  cat "$fixture_dir/stderr" >&2
  exit 1
}

expect_output() {
  local description="$1" filter="$2"
  jq -e "$filter" "$fixture_dir/stdout" >/dev/null || {
    jq . "$fixture_dir/stdout" >&2 || true
    fail "output: $description"
  }
}

public_keys='["bootstrap","chainId","deployer","deployment","localIbcRouter","maxClockDriftSeconds",
  "name","proofMode","registration","source","sp1Groth16Selector","sp1VerifierGateway",
  "trustingPeriodSeconds","upgradeAuthority","vkeys"]'

jq 'del(.vkeys)' "$profile" >"$fixture_dir/chain-only.json"
cp "$fixture_dir/chain-only.json" "$profile"
arguments=(deploy fixture)
run FAKE_REFUSE_ROUTER_CALLS=1 FAKE_REGISTER_FAILS=1 || fail "standalone deploy called or registered a Router"
expect_output "only the public fields" "keys == $public_keys"
expect_output "source fields" '.source | keys == ["chainId","ibcRouter","name","validatorMode"]'
expect_output "no planted secret" "tostring | (contains(\"${key#0x}\") or contains(\"PLANTED\")) | not"
expect_output "deployment" '.deployment.lightClient == "'"$client"'"
  and .deployment.implementation == "'"$implementation"'" and (.deployment.registered | not)
  and .deployment.deployBlock == 28 and .deployment.registerBlock == null
  and (.deployment.implementationTransaction | test("^0x0{63}1$"))
  and (.deployment.deployTransaction | test("^0x0{63}2$"))
  and .deployment.registerTransaction == null
  and .upgradeAuthority == "'"$authority"'"
  and .deployment.image.id == "sha256:image"'
expect_output "generated VKeys" '.vkeys.membership == "'"$(jq -r '.membership.programVKey' "$app/artifacts/vkeys.json")"'"'
expect_output "profile values" '.registration.counterpartyClientId == "client-3"
  and .bootstrap.blockHash == "0x'"$(printf '%064d' 7)"'" and .bootstrap.height == 7
  and .sp1Groth16Selector == "0x12345678" and .source.validatorMode == "header-qbft"'

arguments=(register fixture "$client")
run FAKE_REGISTER_FAILS=1 && fail "register succeeded although registration failed"
expect_output "a failed registration is recorded" '.deployment.lightClient == "'"$client"'"
  and (.deployment.registered | not) and .deployment.registerTransaction == null'
grep --quiet "register fixture $client" "$fixture_dir/stderr" || fail "no rerun command"

run FAKE_REGISTER_FILTER='.receipts[0].status = "0x0"' FAKE_ROUTER_FORGETS=1 \
  && fail "a reverted registration was accepted"
expect_output "a reverted registration is recorded" '(.deployment.registered | not)'

run FAKE_REGISTER_FILTER='.receipts[0].blockNumber = "0x1D"' || fail "deploy failed"
expect_output "an uppercase block number" '.deployment.registerBlock == 29'

run FAKE_SOURCE_HASH_FAILS=1 || fail "an unreadable block hash stopped the deployment"
expect_output "an unreadable block hash" '.bootstrap.blockHash == null and .deployment.registered'
grep --quiet "bootstrap.blockHash is null" "$fixture_dir/stderr" || fail "no warning for the block hash"

# The Router, not the run, decides `registered`.
run FAKE_REGISTER_LANDED=1 || fail "a registration the Router holds was reported as failed"
expect_output "a landed registration is recorded" '.deployment.registered
  and .deployment.registerTransaction == null'
grep --quiet "the registration run failed, but the Router names $client as client-0" \
  "$fixture_dir/stderr" || fail "no note that the run failed"
run FAKE_ROUTER_FORGETS=1 && fail "a registration the Router does not hold was accepted"
expect_output "a registration the Router does not hold" '(.deployment.registered | not)'
grep --quiet "the registration transaction succeeded, but the Router does not name" \
  "$fixture_dir/stderr" || fail "wrong refusal for a registration the Router does not hold"

# An unprefixed key reaches forge with its prefix; the fake forge compares it with $key.
printf '%s\n' "${key#0x}" >"$fixture_dir/key"
run || fail "an unprefixed key was not accepted"
printf '%s\n' "$key" >"$fixture_dir/key"

# A deployment whose record cannot be trusted stops without an output.
arguments=(deploy fixture)
expect_deploy_refusal() {
  local expected="$1"
  shift
  run "$@" && fail "deploy accepted: $expected"
  grep --quiet --fixed-strings -- "$expected" "$fixture_dir/stderr" || fail "wrong refusal: $expected"
  [[ ! -s "$fixture_dir/stdout" ]] || fail "output written although: $expected"
}
expect_deploy_refusal "the creations do not match the returned addresses" \
  FAKE_DEPLOY_FILTER='.returns.lightClient.value = .returns.implementation.value'
expect_deploy_refusal "expected two transactions" \
  FAKE_DEPLOY_FILTER='.transactions += [.transactions[0]]'
expect_deploy_refusal "expected one ERC1967Proxy creation" \
  FAKE_DEPLOY_FILTER='.transactions[1].contractName = "BesuQBFTZKLightClient"'
expect_deploy_refusal "0x$(printf '%064d' 2) failed" FAKE_DEPLOY_FILTER='.receipts[1].status = "0x0"'
expect_deploy_refusal "0x$(printf '%064d' 1) failed" FAKE_DEPLOY_FILTER='.receipts[0].status = "0x0"'
expect_deploy_refusal "expected one receipt for 0x$(printf '%064d' 2)" \
  FAKE_DEPLOY_FILTER='del(.receipts[1])'
expect_deploy_refusal "delegates to 0x5555555555555555555555555555555555555555" \
  FAKE_IMPLEMENTATION_SLOT=5555555555555555555555555555555555555555

arguments=(register fixture "$client")
run || fail "register failed"
expect_output "register" '.deployment.registered and .deployment.deployTransaction == null
  and .deployment.implementationTransaction == null
  and .deployment.implementation == "'"$implementation"'"'
run FAKE_IMPLEMENTATION_SLOT=0000000000000000000000000000000000000000 \
  && fail "a register of an address that is not a proxy was accepted"
grep --quiet "$client is not an ERC-1967 proxy" "$fixture_dir/stderr" || fail "wrong refusal for a non-proxy"

arguments=(bootstrap fixture)
run || fail "bootstrap failed"
expect_output "bootstrap object" "keys == [\"blockTimestamp\",\"height\",\"stateRoot\",\"validatorSetHash\"]
  and .stateRoot == \"$root\""

arguments=(deploy fixture)
run DEPLOYER_PRIVATE_KEY="$key" && fail "a key in the environment was accepted"
grep --quiet "DEPLOYER_PRIVATE_KEY is set in the container environment" "$fixture_dir/stderr" \
  || fail "wrong refusal for a key in the environment"
[[ ! -s "$fixture_dir/stdout" ]] || fail "output written although the key was refused"

for variable in DESTINATION_RPC_URL SOURCE_RPC_URL ETH_RPC_URL FOUNDRY_ETH_RPC_URL; do
  run "$variable=$destination_url" && fail "an RPC URL in the environment was accepted: $variable"
  grep --quiet "$variable is set in the container environment" "$fixture_dir/stderr" \
    || fail "wrong refusal for $variable in the environment"
done
printf 'not a url\n' >"$fixture_dir/bad-url"
run SOURCE_RPC_URL_FILE="$fixture_dir/bad-url" && fail "a file without a URL was accepted"
grep --quiet "$fixture_dir/bad-url does not hold an http(s) URL" "$fixture_dir/stderr" \
  || fail "wrong refusal for a file without a URL"

arguments=(deploy ../fixture)
status=0
run || status=$?
(( status == 64 )) || fail "a path was accepted as a profile name"

echo "entrypoint tests passed"
