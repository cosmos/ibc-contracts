# Besu Proof API
Connects prover infrastructure with compiled ZK circuits and collects required inputs from RPC to enable proof generation for provers. 
Run one API per client route; Besu nodes, contracts, the relayer, and the SP1 network are external.

## Run with Docker

From the repository root, with Docker and Compose running:

```sh
make build-docker
make build-proof-api
cp tooling/proof-api/config/proof-api.example.yaml /absolute/path/to/proof-api.yaml
# Edit the YAML using the configuration reference below before starting.
export NETWORK_PRIVATE_KEY='<SP1 network signing key>'
export NETWORK_RPC_URL='<SP1 network RPC URL>'
BESU_PROOF_API_CONFIG=/absolute/path/to/proof-api.yaml docker compose -f deploy/compose.yaml up --detach --force-recreate proof-api
docker compose -f deploy/compose.yaml logs --follow proof-api
# Stop:
docker compose -f deploy/compose.yaml stop proof-api
```

The runtime image uses bind-mounted binaries from `target/docker/release/` and circuit artifacts
from `artifacts/`. Recreate the service after replacing outputs or approving a new build.
Startup validates artifact approvals, chain identity, Router/client bindings, deployed VKeys,
and an authenticated source Router storage proof before opening the listener.

## YAML configuration

Start with [proof-api.example.yaml](config/proof-api.example.yaml). Defaults below are applied by
[the parser](src/config.rs) when a setting is omitted; **Required** means there is no usable default
for serving. In particular, the example explicitly sets `rpc.callTimeout` to `30s`, overriding
its `3s` default.

| YAML setting | Default | Purpose / valid values |
| --- | --- | --- |
| `schemaVersion` | Required | Must be `1`. |
| `prover.artifactsDir` | Required | Directory containing ELFs and `vkeys.json`; use `/workspace/artifacts` with Compose. Relative paths resolve from the YAML directory. |
| `prover.approvedPrograms.<program>.elfSha256` | Required | Approved ELF SHA-256; quoted, nonzero, `0x`-prefixed 32-byte hex. |
| `prover.approvedPrograms.<program>.programVKey` | Required | Approved SP1 program VKey; same hex format. |
| `server.listen` | Required | Listener address; use `"0.0.0.0:8085"` with Compose. |
| `server.maxJobDurationSeconds` | `3600` | Proof job timeout in seconds; `1–86400`. |
| `server.shutdownTimeoutSeconds` | `30` | Request drain timeout in seconds; `1–300`. |
| `client.chainId` | Required | Nonempty relayer chain identifier; independent of numeric EVM chain pins below. |
| `client.clientId` | Required | Nonempty client route identifier. |
| `rpc.source.proxyUrl` | Required | Source Besu HTTP(S) RPC endpoint, reachable from the container. |
| `rpc.verifier.proxyUrl` | Required | Verifier/destination chain HTTP(S) RPC endpoint. |
| `rpc.callTimeout` | `"3s"` | Per-call timeout; `1s–5m`. Example YAML uses `"30s"`. |
| `rpc.maxAttempts` | `2` | Maximum attempts per RPC call; `1–10`. |
| `rpc.retryBackoff` | `"100ms"` | Retry backoff; at most `60s`. |
| `rpc.maxResponseBytes` | `16777216` (16 MiB) | Maximum RPC response size; `1–67108864` bytes. |
| `rpc.expectedSourceChainId` | Unset | Optional numeric source EVM chain ID. |
| `rpc.expectedVerifierChainId` | Unset | Optional numeric verifier EVM chain ID. |
| `contracts.verifierRouter` | Required | Nonzero, 20-byte hex address of the Router on the verifier chain. |
| `scheduler.preferredBatchSize` | `10` | Target membership batch size; `1–maxBatchSize`. |
| `scheduler.maxBatchSize` | `32` | Maximum membership batch size; `preferredBatchSize–32`. |
| `scheduler.maxBatchWait` | `"3s"` | Batch collection threshold; at least `2s`, Prover capacity waits and proving can take longer. |
| `scheduler.maxPendingIntents` | `1024` | Caps unique queued/running membership intents and, separately, outstanding update jobs. At least `maxBatchSize`; excess work gets `ResourceExhausted`. |
| `scheduler.maxActiveJobs` | `1` | Shared active proof job limit; `1–64`. |
| `scheduler.allowParallelUpdateAndMembership` | `true` | When `false`, limits shared proof concurrency to one regardless of `maxActiveJobs`. |
| `scheduler.updateJobDeduplication` | `true` | Shares running update proofs at the same target height; must remain `true`. |

`<program>` means each of `update-client`, `membership`, and `non-membership`. Copy all six
identity values from the reviewed build's `artifacts/vkeys.json`; the template placeholders
intentionally fail validation. Loaded, approved, and deployed VKeys must agree.

Durations accept integer strings with `ms`, `s`, `m`, or `h`, or integer milliseconds.
The aliases `callTimeoutMs`, `retryBackoffMs`, and `maxBatchWaitMs` are also accepted.
Unknown fields, duplicate fields, and malformed values are rejected.


## Validation and endpoints

With native toolchains installed, use these commands before serving:

```sh
cargo run --locked -p proof-api -- validate-config --config /path/to/proof-api.yaml
cargo run --locked -p proof-api -- validate-artifacts --config /path/to/proof-api.yaml
cargo run --locked -p proof-api -- validate-integration --config /path/to/proof-api.yaml
SP1_PROVER=network cargo run --locked -p proof-api -- serve --config /path/to/proof-api.yaml
cargo run --locked -p proof-api -- healthcheck --address 127.0.0.1:8085
```

`validate-config` checks YAML and configured identities; `validate-artifacts` also checks local
ELFs/VKeys. Neither needs RPC access or SP1 credentials. `validate-integration` additionally
checks live chains and deployed VKeys without generating proofs. `serve` needs both network
credentials above. `healthcheck` checks `/readyz`.

One listener supports h2c gRPC, unary Connect JSON/protobuf, and gRPC-Web. `/healthz` reports
liveness, `/readyz` integration readiness, and `/metrics` Prometheus metrics. JSON logs on stderr
include request IDs, route/packet context, results, and errors, excluding payloads and proof bytes.

For development tests, fixtures, and compatibility evidence, see
[implementation status](../../docs/PROOF_API_IMPLEMENTATION.MD) and [Testing](../../docs/TESTING.md).
See [protocol provenance](proto/README.md) for the service schema and [Integration](../../docs/INTEGRATION.md)
for relayer boundaries.
