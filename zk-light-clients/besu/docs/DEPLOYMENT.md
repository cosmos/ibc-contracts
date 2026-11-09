# Deployment

Deploy a directional light-client proxy on the destination chain to track a source Besu QBFT chain.
For the short command sequence, see [README](../README.md#deploy-to-existing-test-nodes).
[Architecture](architecture.md) explains bootstrap and proof binding. This guide covers profile
preparation, the public configuration schema, contract deployment, and registration.
Run commands from the repository root.

## Prerequisites

- Source and destination RPC endpoints, with historical source headers and `eth_getProof` state.
- An AccessManager upgrade authority and a compatible SP1 verifier gateway with the intended
  Groth16 route enabled. The Router addresses can be reserved addresses with no deployed code.
- A successful `make build-docker`, producing matching circuit ELFs/VKeys, host tools, and contracts.
- A public deployment profile and a funded authorized deployer key file. No Router checkout or
  private Router repository access is needed.
- Docker access and RPC reachability from its containers. Set `DOCKER_NETWORK` if needed.

The `deploy` action creates only the light-client implementation and initialized proxy.
Registration is a separate `register` action against an existing compatible Router. It does not create a Besu chain or deploy the
whole upstream Router/gateway stack. For a disposable complete stack, use [Testing](TESTING.md).

## Build once

```bash
make build-docker
```

Deployment reuses the build tool image (`BUILD_IMAGE`) and mounts the working tree, generated
artifacts (read-only), compiled contracts, and Linux bootstrap utility. There is no separate
image packaging stage or requirement to commit the working tree. Rebuild after changing source.

## Deployment profiles

One file per light client deployment, `config/<name>.json`, holding every public input the
deployment binds: the destination chain, its Router and verifier gateway, the source chain and its
Router, the bootstrap anchor, the client parameters and the registration IDs.
Copy `config/template.json` to `config/<name>.json`, set its `name` field to match the filename,
and fill the deployment inputs. Set `PROFILE_DIR` to use a different profile directory.
VKeys are read automatically from `artifacts/vkeys.json`; legacy profile VKeys are ignored and
replaced in the temporary effective profile. Registration fields may remain unset until
the separate registration action. Router addresses must be nonzero but need no deployed code
for standalone deployment. The scripts are chain agnostic: the profile is the only
thing that names a chain.

Runtime inputs that are not in the profile:

| Input | Set on the host as | Mounted in the container at |
| --- | --- | --- |
| Destination RPC | `DESTINATION_RPC_URL` | `/run/secrets/destination-rpc-url` |
| Source RPC | `SOURCE_RPC_URL` | `/run/secrets/source-rpc-url` |
| Deployer key | `DEPLOYER_KEY_FILE`, a file | `/run/secrets/deployer-key` |

## Prepare from an IBC config

`make prepare-profile` reads the IBC CLI's `ibc.yml` / `ibc.yaml` directly. No IBC CLI binary,
checkout, or intermediate directional `.env` file is needed. Use the rendered config after Router
deployment: each selected chain must have `evm.rpc`, `evm.ics26Router`, and a chain ID.

`make setup` creates `.venv` and installs the preparation dependencies; later Make commands
select that interpreter automatically. To manage your own Python environment (Python 3.9+):

```bash
python3 -m pip install -r deploy/requirements.txt
# Or use a virtual environment and pass PYTHON=/path/to/venv/bin/python to make.
```

Preparation needs host Foundry (`cast`, and `forge` for local verifier deployment), `jq`, generated
`artifacts/vkeys.json`, and the native `bootstrap-state` utility. Run `make build-docker` for
deployment contracts and canonical circuit/VKey outputs.
Preparation executes a native bootstrap utility: use `cargo build --locked --release -p prover-script --bin bootstrap-state`
or provide `BOOTSTRAP_STATE_BIN=/absolute/path/to/bootstrap-state`; without an override it uses `cargo run`.
The Linux binary from the Docker build is used by container deployment, not native preparation.
The existing Docker build/deployment workflow remains available for deploy/register.

```bash
export IBC_CONFIG=/absolute/path/to/ibc.yml
make prepare-profile \
  PROFILE=on-chain-a-tracking-chain-b \
  SOURCE_CHAIN_ID=41002 DESTINATION_CHAIN_ID=41001 \
  CLIENT_ID=zk-41001-41002 COUNTERPARTY_CLIENT_ID=zk-41001-41002 \
  SP1_VERIFIER_GATEWAY=0xYourExistingGateway \
  TRUSTING_PERIOD_SECONDS=86400 MAX_CLOCK_DRIFT_SECONDS=30
```

Replace the gateway placeholder with its actual address. For disposable devnets only, replace
`SP1_VERIFIER_GATEWAY=...` with `DEPLOY_LOCAL_VERIFIER=true` to deploy the real SP1 Groth16 verifier
and gateway, or recover them from a prior live Foundry broadcast. This option broadcasts transactions.
An explicitly supplied gateway takes precedence. Neither profile preparation nor local verifier
preparation deploys or registers a light client.

| Input | Resolved from |
| --- | --- |
| Source/destination IDs, RPCs and Router addresses | Selected `chains[]` entries in `IBC_CONFIG` |
| Deployer | Destination `chains[].deployer` → local `signers[]` key; public address derived locally |
| Upgrade authority | Destination Router's `authority()` |
| Bootstrap height, state root, timestamp and validator commitment | Canonical QBFT source header; default height is head minus `BOOTSTRAP_CONFIRMATIONS=1` |
| Verifier gateway | Explicit `SP1_VERIFIER_GATEWAY`, or opt-in local deployment |
| Timing policy | Required `TRUSTING_PERIOD_SECONDS` and `MAX_CLOCK_DRIFT_SECONDS` |
| Registration IDs | Required `CLIENT_ID` and `COUNTERPARTY_CLIENT_ID` |
| Program VKeys | Generated `artifacts/vkeys.json`, injected by deployment, never copied into source profiles |

Existing relayer connection IDs are deliberately not inferred: they can refer to already registered
attestation clients. Choose the intended ZK pair's IDs before preparing either direction. Preparation
checks sequential ID availability or simulates custom-ID registration, validates the live gateway
route and bootstrap state, then atomically saves `config/PROFILE.json`. Failure preserves any existing
profile. Pass `FORCE=true` to refresh a profile after reviewing the inputs; pass `HEIGHT=<n>` to choose
an anchor explicitly. `SP1_GROTH16_SELECTOR` defaults to `0x4388a21c`.

Relative signer paths resolve from the IBC config directory, including the CLI's `.json` and `keys/`
fallbacks. Set `IBC_HOME=/absolute/path/to/home` when the config is stored outside its IBC home.
The importer accepts IBC ECDSA JSON keys (`privateKeyBase64`). `DEPLOYER_KEY_FILE` overrides the
configured signer and accepts either that format or a raw 32-byte hex key, including when the
configured signer is remote. Remote KMS signing is not supported by this deployment workflow.
Environment references such as `${RPC_URL}` are expanded without executing shell code; unset
variables and duplicate YAML keys are rejected. Quote hex addresses in YAML.

Use the same config during later deployment, with the existing build image and artifacts:

```bash
make deploy ACTION=preflight PROFILE=on-chain-a-tracking-chain-b IBC_CONFIG="$IBC_CONFIG"
make deploy PROFILE=on-chain-a-tracking-chain-b IBC_CONFIG="$IBC_CONFIG"
make deploy ACTION=register PROFILE=on-chain-a-tracking-chain-b IBC_CONFIG="$IBC_CONFIG" LIGHT_CLIENT=0xProxyAddress
```

These commands resolve RPCs and the destination deployer key from the config again. Router addresses
must agree with the reviewed profile; deployment checks chain and deployer identity as before.
Temporary raw key files are private and removed after the command. RPC URLs and private keys are
never written into the public profile. Without `IBC_CONFIG`, the existing explicit RPC and
`DEPLOYER_KEY_FILE` inputs still work. The former `.env` adapter interface has been replaced by this
native config importer; re-run preparation from `ibc.yml` instead of sourcing old `zk-prepare` files.

## Profile validation

Deployment profile readers include: `deploy/validate.sh --standalone --profile` and
`deploy/entrypoint.sh` with `jq`, and `contracts/script/DeployLightClient.s.sol` and
`contracts/script/RegisterLightClient.s.sol` through `contracts/script/ProfileJson.sol`.

Required deployment fields have no implicit defaults. Preflight rejects absent, `null` or empty
fields with `profile field missing: <path>`. The Forge reader reports absent/null fields and empty
required strings as `ProfileFieldMissing(<path>)`; malformed typed values fail parsing before
anything is signed. With `--profile`, preflight refuses to run while any of its environment names
other than the two RPC URLs (`INITIAL_HEIGHT`, `LOCAL_IBC_ROUTER`, ...) is set, so a stale `.env`
cannot be mistaken for the profile.

Both Forge scripts refuse to broadcast unless the node reports the profile's `chainId`
(`ChainIdMismatch`) and the key belongs to the profile's `deployer` (`DeployerMismatch`). Initialization binds the configuration and bootstrap state described in
[Architecture](architecture.md#trusted-state-and-bootstrap). Updates add consensus states;
changing implementation-controlled configuration requires an upgrade.

No profile field holds a secret, and none may: profiles are meant to be committed and reviewed.

## Bootstrap and preflight

```bash
export SOURCE_RPC_URL=http://source-test-node:8545
export DESTINATION_RPC_URL=http://destination-test-node:8545
make deploy ACTION=bootstrap PROFILE=test-a-to-b
```

`HEIGHT=<number>` selects an explicit source block; without it, the manual bootstrap action
uses the source head. IBC config preparation defaults to head minus `BOOTSTRAP_CONFIRMATIONS=1`. Review the returned height, state
root, block timestamp, and validator-set hash, then put them together in the profile's `bootstrap`
object. This anchor is the deployment trust decision; an update proof does not establish it.

```bash
make deploy ACTION=preflight PROFILE=test-a-to-b
```

Deployment runs these checks automatically; the optional preflight action checks chain IDs, verifier code/route, validator/proof modes, generated VKeys,
and bootstrap consistency/expiry. It does not call either Router or require registration fields.
The deployment script checks that the configured upgrade authority has code. Registration uses
the integration preflight, which additionally checks Router code, registration, and authority
compatibility. Source-layout conformance runs in the optional E2E workspace.
The wrapper mounts RPC URLs and keys as files instead of putting their values in Docker arguments.

The values for a height never change, since a QBFT block is final once sealed. When
`trustingPeriodSeconds` is nonzero, the anchor expires: when destination time is at least `blockTimestamp + trustingPeriodSeconds`, the
client's first `updateClient` reverts `TrustedStateExpired`, so both preflight and the Forge script
refuse such a profile. Preflight also re-derives the three values from the source header and
refuses a mismatch.

## Deploy, register, and inspect

```bash
DEPLOYER_KEY_FILE=/absolute/path/to/test-deployer-key \
  make deploy PROFILE=test-a-to-b
```

The deployment creates an implementation and an ERC1967Proxy initialized in its constructor.
It does not register the proxy. Outputs are written under
`deployments/<profile>/<UTC timestamp>.json` and include the image identity and transaction evidence.
The proxy address is the light client; do not register the implementation address.

For a proxy already deployed from the reviewed release:

```bash
DEPLOYER_KEY_FILE=/absolute/path/to/test-deployer-key \
  make deploy ACTION=register PROFILE=test-a-to-b LIGHT_CLIENT="$LIGHT_CLIENT_PROXY"
```

Set `LIGHT_CLIENT_PROXY` to the actual proxy address before running registration. After deployment, check the destination
Router's client registration, proxy implementation/authority, configured Routers/gateway/VKeys,
and stored bootstrap consensus state against the reviewed profile. Inspect transaction receipts,
then prove an update and the relevant packet/ACK/timeout flow as described in [Testing](TESTING.md).
For the reverse direction, prepare another profile/deployment with source and destination exchanged.

## Registration

`RegisterLightClient` adds the deployed client to `localIbcRouter`. An ID of the form `client-<n>`
uses the Router's permissionless `addClient`, which assigns from a sequence. Any other ID uses the
explicit-ID overload and whatever authorization mechanism that Router release enforces. Preflight
simulates that call from the profile deployer without persisting state. Once stored, the
counterparty client ID can be changed only by the Router's AccessManager, through `migrateClient`.
Each client of a pair names the other's, so both IDs are fixed in the profiles before either side
registers. Registration preflight verifies either that `registration.clientId` is next in sequence or that
the custom ID is available and the deployer is authorized. Standalone deployment skips these
checks and may precede Router deployment.
`RegisterLightClient` checks it again, and that the client at the given address was initialized
with this profile's Routers, gateway, VKeys, upgrade authority, anchor and client parameters.

## Profile fields

| Field | Type | Meaning |
| --- | --- | --- |
| `name` | string | The file name without `.json`. |
| `description` | string | Free text. |
| `notes` | string[] | Rationale for readers. Ignored by the tooling. |
| `chainId` | integer | The destination chain ID (`uint256` in the Forge reader). The node must report it. |
| `deployer` | address | The account that signs the deployment and the registration. |
| `localIbcRouter` | address | The ICS26 Router on the destination chain. |
| `sp1VerifierGateway` | address | The SP1 verifier gateway on the destination chain. |
| `upgradeAuthority` | address | The AccessManager authorizing the client's upgrades. Must have code; registration preflight additionally requires it to match the Router authority. |
| `sp1Groth16Selector` | bytes4 | The Groth16 route the gateway must hold, unfrozen. |
| `proofMode` | `"groth16"` | The only accepted value. |
| `source.name` | string | The tracked chain, for readers. |
| `source.chainId` | integer | The source node must report it. |
| `source.ibcRouter` | address | The ICS26 Router on the source chain. |
| `source.validatorMode` | `"header-qbft"` | The only accepted value. |
| `vkeys` | generated | Not a source-profile input; injected from `artifacts/vkeys.json` at deployment. |
| `bootstrap.height` | integer | The finalized source block the client is anchored to. |
| `bootstrap.stateRoot`, `bootstrap.validatorSetHash` | bytes32 | That block's values. |
| `bootstrap.blockTimestamp` | integer | That block's timestamp. |
| `trustingPeriodSeconds` | integer | `0` disables expiry. |
| `maxClockDriftSeconds` | integer | Maximum accepted source timestamp ahead of destination time when updating. |
| `registration.clientId` | string | The local ID: next sequential `client-<n>`, or an available custom ID whose explicit registration succeeds from the deployer. |
| `registration.counterpartyClientId` | string | The ID of the client tracking this chain on the source chain's Router. |

Integers above 2^53 are written as decimal strings, because JSON numbers lose precision past it.
Source chain ID, bootstrap height/timestamp, and timing policies must fit `uint64`.

## Deployment output

`deploy` and `register` write `deployments/<name>/<UTC time>.json`: the profile's fields above,
except `description` and `notes`, with `bootstrap.blockHash` added, and a `deployment` object:

| Field | Meaning |
| --- | --- |
| `lightClient` | The client: the ERC1967Proxy the Router calls. |
| `implementation` | The implementation the proxy delegates to, read from its ERC-1967 slot. |
| `registered` | `false` for standalone deployment; registration reads the Router to confirm the registered client. |
| `deployTransaction`, `deployBlock` | The proxy's creation. `null` for `register`. |
| `implementationTransaction` | The implementation's creation. `null` for `register`. |
| `registerTransaction`, `registerBlock` | Registration receipt fields; `null` for standalone deployment. |
| `profileSha256` | The input profile file. |
| `image.id`, `image.tag` | The local image that ran. |
| `gitCommit`, `sp1ContractsCommit` | Optional provenance; null when not supplied. The tool image contains no application source. |
| `recordedAt` | UTC. |

The output is the input for services that act on the deployed client, such as the prover. Fields
are copied by name, so nothing else in the profile reaches it, and it holds no RPC URL and no key.
Archive the record with the generated VKeys and reviewed source revision; the shared tool image
does not identify the mounted application source.

## v1.1 batched-membership migration

Schema-2 membership needs the compatible light-client implementation, new membership artifacts,
and FULL/CACHED envelopes. Replacing a VKey in an old deployment is insufficient. Prepare a reviewed
v1.1 deployment and use the pinned Router's governance/client-migration process for the route.
Exact governance actions depend on the deployed upstream Router version.

Before deployment, rebuild the circuit/VKey set with `make build-docker`. Optional release
evidence is described in [Release](RELEASE.md). Shared guest dependencies may alter update/non-membership ELFs as well as
membership, even when their proof statements remain unchanged. Investigate and record every change.
Attach validation results to the rebuilt source and artifact identities.

## Proving service and diagnostics

The unified Proof API returns relay-ready envelopes and leaves transaction submission to the
relayer. See [Integration](INTEGRATION.md) for its protocol and the separate legacy HTTP utility,
and the [runtime guide](../tooling/proof-api/README.md) for service configuration.

`scripts/collect-test-data.sh` captures header/proof fixtures through two subcommands:

```bash
bash scripts/collect-test-data.sh qbft "$RPC_URL" "$FROM_HEIGHT" "$TO_HEIGHT" "$OUTPUT_DIR"
bash scripts/collect-test-data.sh mpt "$RPC_URL" "$ROUTER_ADDRESS" "$BLOCK_HEIGHT" "$SLOT_FILE" "$OUTPUT_DIR"
```

Set the variables to the intended RPC, block range or height, and output location. For MPT data,
`SLOT_FILE` contains one bytes32 storage key per line. Both modes share RPC transport and retain
their existing output formats. Their manifests omit RPC URLs. `eth_chainId` is an endpoint sanity check; it is not part
of the QBFT committed-seal signature domain. CLI time-policy inputs describe destination contract
policy and are not additional private witness fields.
