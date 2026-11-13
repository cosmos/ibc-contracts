#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
state_root="$repo_root/.e2e-state"
besu_image="hyperledger/besu@sha256:f1dad21871b49bf54fce2a8c547d96cb7835005343af02eebd72da137723c3bc"
cast_bin="${CAST_BIN:-cast}"
default_validator_count="${E2E_VALIDATOR_COUNT:-1}"
chain_a_validator_count="${E2E_CHAIN_A_VALIDATOR_COUNT:-$default_validator_count}"
chain_b_validator_count="${E2E_CHAIN_B_VALIDATOR_COUNT:-$default_validator_count}"

validate_validator_count() {
  local variable_name="$1"
  local value="$2"
  case "$value" in
    1|4) ;;
    *)
      echo "$variable_name must be 1 (reduced test topology) or 4 (conformance topology)" >&2
      exit 64
      ;;
  esac
}
validate_validator_count E2E_CHAIN_A_VALIDATOR_COUNT "$chain_a_validator_count"
validate_validator_count E2E_CHAIN_B_VALIDATOR_COUNT "$chain_b_validator_count"

docker_user_args=(--user "$(id -u):$(id -g)")
docker_entrypoint_args=()
docker_security_options="$(docker info --format '{{json .SecurityOptions}}')"
if [[ "$docker_security_options" == *'name=rootless'* ]]; then
  # Container root is mapped to the invoking user by rootless Docker. Passing the
  # host UID explicitly would instead map it to a subordinate UID that cannot
  # write the bind-mounted state directory. Bypass Besu's root entrypoint wrapper
  # as well: its path-probing invocation executes this operator subcommand once
  # before the real invocation, leaving an output directory that the latter then
  # rejects as pre-existing.
  docker_user_args=()
  docker_entrypoint_args=(--entrypoint /opt/besu/bin/besu)
fi

if [[ -e "$state_root" ]]; then
  echo "refusing to overwrite existing generated state: $state_root" >&2
  echo "remove that ignored directory explicitly before regenerating" >&2
  exit 1
fi
mkdir -p "$state_root"

generate_chain() {
  local name="$1"
  local subnet_prefix="$2"
  local validator_count="$3"
  local template="$repo_root/e2e/besu/$name-genesis.template.json"
  local state="$state_root/$name"
  mkdir -p "$state"

  local deployer_key
  deployer_key="0x$(openssl rand -hex 32)"
  local deployer_address
  deployer_address="$($cast_bin wallet address --private-key "$deployer_key")"
  printf '%s\n' "$deployer_key" > "$state/deployer.key"
  chmod 600 "$state/deployer.key"

  local address_without_prefix="${deployer_address#0x}"
  jq --arg address "$address_without_prefix" --argjson validator_count "$validator_count" \
    '.genesis.alloc[$address] = {"balance":"0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"}
     | .blockchain.nodes.count = $validator_count' \
    "$template" > "$state/config.json"

  docker run --rm "${docker_user_args[@]}" "${docker_entrypoint_args[@]}" \
    --volume "$state:/state" \
    --volume "$state/config.json:/config.json:ro" \
    "$besu_image" operator generate-blockchain-config \
    --config-file=/config.json --to=/state/generated --private-key-file-name=key

  mapfile -t generated_keys < <(find "$state/generated/keys" -mindepth 1 -maxdepth 1 -type d | sort)
  if [[ "${#generated_keys[@]}" -ne "$validator_count" ]]; then
    echo "$name did not generate exactly $validator_count validator identities" >&2
    exit 1
  fi

  local static_nodes='[]'
  for ((index = 1; index <= validator_count; index++)); do
    local key_dir="${generated_keys[$((index - 1))]}"
    local node_dir="$state/node-$index"
    mkdir -p "$node_dir"
    cp "$key_dir/key" "$node_dir/key"
    cp "$key_dir/key.pub" "$node_dir/key.pub"
    local public_key
    public_key="$(tr -d '\r\n' < "$key_dir/key.pub")"
    public_key="${public_key#0x}"
    static_nodes="$(jq \
      --arg enode "enode://$public_key@$subnet_prefix.$((10 + index)):30303" \
      '. + [$enode]' <<< "$static_nodes")"
  done
  cp "$state/generated/genesis.json" "$state/genesis.json"
  for ((index = 1; index <= validator_count; index++)); do
    printf '%s\n' "$static_nodes" > "$state/node-$index/static-nodes.json"
  done
  printf '%s\n' "$deployer_address" > "$state/deployer.address"
}

generate_chain chain-a 172.30.0 "$chain_a_validator_count"
generate_chain chain-b 172.31.0 "$chain_b_validator_count"
echo "generated generic QBFT networks under $state_root (chain-a=$chain_a_validator_count, chain-b=$chain_b_validator_count)"
