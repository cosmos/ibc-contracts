# Prover and relayer integration

Use the unified Rust `proof-api` service for relayer-facing proving. It calls
`prover-script::proving_service::ProvingService` in process; it neither starts nor calls
`prover-host`. Besu RPC endpoints, the SP1 network, deployed contracts, and the relayer are external.
Commands below run from the repository root.

## Relayer protocol

One process serves one configured `(chainId, clientId)` pair. Requests for a different pair are
rejected. The listener supports h2c gRPC, unary Connect JSON/protobuf, and gRPC-Web using
`ibc.v2.prover.ProverService`.

| Method | Input beyond `client` | Result |
| --- | --- | --- |
| `LatestProvableHeight` | None | Current source head height/timestamp when newer than the client's latest accepted height and a trusted state exists |
| `StateProof` | `height` | Opaque ABI-encoded update envelope for a height newer than the latest accepted height |
| `PacketProofs` | `height`, `kind`, `packets` | One opaque ABI-encoded envelope per packet, preserving request order and duplicates |

Proof kinds are packet commitment, acknowledgement, and receipt absence. The service derives
packet paths and commitments, authenticates acknowledgement commitments from source storage,
and checks the source timestamp for timeout requests. The exact messages and provenance are in
[prover.proto](../tooling/proof-api/proto/cli/prover.proto) and its
[protocol notes](../tooling/proof-api/proto/README.md).

The unified API has no bearer-token configuration. Compose binds its port to loopback by default.
Its health, readiness, metrics, YAML settings, startup checks, and Docker commands are maintained
in the [runtime guide](../tooling/proof-api/README.md).

## Scheduling and relay ordering

Compatible membership intents at the same route and exact height can share a proof across
requests. The scheduler bounds admission, rejects conflicting commitments, deduplicates pending
and running intents, splits work into batches, and fans results out to callers. Update jobs are
shared by target height. All proof kinds share the configured proof-job permits. Jobs and queues
are in memory; persistent workflow state belongs to the relayer or an external coordinator.

For membership, each `PacketProofs` response includes a FULL envelope for the first occurrence of each batch root
in that response and CACHED envelopes for subsequent occurrences. A response does not depend on
another response having primed the destination cache. The API reconstructs and validates branches
and envelopes against the request, approved artifacts, and source snapshot. It rechecks integration
identity and relevant source/consensus state after proving.

Packet proofs can be generated before the destination installs their height. If that consensus
state already exists, it must match the source block. The relayer must submit any required update
before membership/non-membership, and submit each FULL item before its CACHED items. Preserve the
returned order or reconstruct an equivalent valid order. Inspect transaction receipts and final
Router/application state; receiving a proof does not establish successful IBC delivery. Proof and
cache semantics are defined in [Architecture](architecture.md#batched-membership).

Caller deadlines bound requests, including body reads and waiting. Shared jobs have a separate
configured job timeout; a caller timing out does not imply that shared proving was cancelled.
Capacity exhaustion returns `ResourceExhausted`; malformed requests return `InvalidArgument`,
route mismatches return `NotFound`, integration mismatches return `FailedPrecondition`, and RPC or
prover failures can return `Unavailable` or `DeadlineExceeded`. Correct configuration/input errors
before retrying and use backoff for capacity/transient failures. The service does not retry proofs.

## CLI entry points

```bash
cargo run --locked -p prover-script --bin prover-script -- --help
cargo run --locked -p prover-script --bin prover-script -- prove-membership --help
cargo run --locked -p prover-script --bin prover-script -- prove-membership-witness --help
```

CLI commands provide execution, proving from live RPC inputs, and proving from prepared witnesses.
Use `--witness-output` on live-input commands to retain the exact normalized input for replay.
Prepared-witness proving validates the selected ELF/VKey against generated identities. Use the
canonical ELFs from `make build-docker`; see [Development](DEVELOPMENT.md).

## Legacy HTTP utility

`prover-host` remains a separate low-level utility for callers that supply RPC inputs and paths.
It uses the same proving service but does not implement the relayer protocol above. Start it with
native toolchains, canonical build artifacts, reachable RPC endpoints and these environment values:

```bash
export SP1_PROVER=network
export NETWORK_RPC_URL='<SP1 network RPC URL>'
export NETWORK_PRIVATE_KEY='<dedicated SP1 signing key>'
export BESU_PROVER_HOST_TOKEN='<token of at least 32 bytes>'
export BESU_PROVER_ALLOWED_RPC_URLS='http://chain-a:8545,http://chain-b:8545'
export BESU_PROVER_ARTIFACTS_DIR=artifacts
export BESU_PROVER_HOST_BIND=127.0.0.1:8080
cargo run --locked -p prover-script --bin prover-host
```

### HTTP interface

| Method/path | Authentication | Purpose |
| --- | --- | --- |
| `GET /healthz` | None | Process liveness |
| `GET /readyz` | None | Readiness after program loading |
| `GET /v1/readyz` | Bearer token | Authenticated readiness |
| `POST /v1/proofs/update-client` | Bearer token | Prove an update from trusted to target height |
| `POST /v1/proofs/membership` | Bearer token | Prove 1–32 packet/ACK paths at one height |
| `POST /v1/proofs/non-membership` | Bearer token | Prove receipt absence |

These routes belong to `prover-host`, not the unified API. Readiness does not prove the remote
prover or both RPCs can complete a job. Requests have a 64-KiB
body limit. RPC timeout defaults to 30 seconds and must be between 1 and 300 seconds; it is not a
whole-proof deadline. One active proof request is permitted per process. Other requests receive
429 rather than entering a persistent queue. Health and readiness responses include
`apiRevision: "sp1-host-http-v2-batch"`; rebuild and recreate the host to pick up source changes.

Each request includes `rpc`:

```json
{
  "sourceRpc": "http://chain-a:8545",
  "destinationRpc": "http://chain-b:8545",
  "expectedSourceChainId": 13371,
  "rpcTimeoutSeconds": 30
}
```

| Operation | Additional JSON fields |
| --- | --- |
| Update | `destinationLightClient`, `trustedHeight`, `targetHeight` |
| Membership | `sourceRouter`, `destinationLightClient`, `proofHeight`, `entries` (1–32 objects with `path` and `expectedCommitment`) |
| Non-membership | `sourceRouter`, `destinationLightClient`, `proofHeight`, `path` |

Addresses are 0x-prefixed 20-byte hex; commitments are 32 bytes. `path` is the single terminal
path's hex bytes, not JSON encoding of `bytes[]`. Heights are JSON integers. The source/verifier
roles reverse with direction; receipt absence for a timeout concerns the chain whose receipt
state is being checked. Unknown request fields are rejected.

Example update request after exporting a token into the invoking shell:

```bash
curl --fail-with-body http://127.0.0.1:8080/v1/proofs/update-client \
  -H 'Content-Type: application/json' \
  -H "Authorization: Bearer $BESU_PROVER_HOST_TOKEN" \
  --data '{
    "rpc": {
      "sourceRpc": "http://chain-a:8545",
      "destinationRpc": "http://chain-b:8545",
      "expectedSourceChainId": 13371,
      "rpcTimeoutSeconds": 30
    },
    "destinationLightClient": "0x1111111111111111111111111111111111111111",
    "trustedHeight": 100,
    "targetHeight": 120
  }'
```

Replace the sample chain configuration/address/heights with the actual deployment values.
The consensus-state submission requirement described above also applies to these artifacts.

### Response and failures

A successful response has `proof` and `witness`. Update/non-membership use artifact schema version
1: `proof` includes `schemaVersion`, `program`, `elfSha256`, `programVKey`, `publicValues`,
`proofBytes`, and `proofEnvelope`. `witness` includes `serializedBytes` and `sha256`.
Membership uses artifact schema version 2: its `proof` has
`proofHeight`, `batchRoot`, `entryCount`, and `entries` instead of a top-level `proofEnvelope`.
Each entry includes `leafIndex`, `pathDigest`, `expectedCommitment`, `merkleProof`, and
`proofEnvelope`. The first envelope is FULL (public values and proof bytes); the remaining
ones are CACHED with Merkle branches. Entries follow canonical path-digest order.
Identities come from `artifacts/vkeys.json`. Unlike the unified API, this utility has no
YAML approval policy or cross-request scheduler.

| Status | Typical meaning | Integration action |
| --- | --- | --- |
| 400 | Invalid input, witness, or non-allowlisted RPC | Correct the request/configuration |
| 401 | Missing/wrong bearer token | Correct authentication |
| 429 | Another proof is active | Retry with backoff and avoid duplicate upstream jobs |
| 502 | RPC or SP1 failure | Inspect the response code/logs and backend state |
| 504 | RPC timeout | Check endpoint availability and request timeout |
| 500 | Host configuration/environment failure | Repair host configuration |

Malformed JSON and oversized bodies can be rejected by the HTTP framework before the custom error
mapping.
