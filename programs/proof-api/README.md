# IBC Proof API

The proof API is a gRPC service that prepares IBC transactions and proofs for clients. It does not listen for events, sign, or submit transactions. Instead, clients provide transaction hashes and the service returns the corresponding IBC messages and proofs that must be signed and submitted elsewhere.

The proof API:
1. Queries the source chain for the given transaction hashes.
2. Parses transaction events to extract packet data.
3. Builds IBC messages plus the required proof(s).
4. Returns an unsigned transaction payload to the caller.

## Overview

The service is composed of one-sided modules, each responsible for a specific source and target chain combination. Each module is a Rust struct that implements the `ProofApiModule` trait.

The gRPC service definition is in [`proto/proofapi/proofapi.proto`](../../proto/proofapi/proofapi.proto).

## Modules

Each module runs in one direction and specializes in how it builds proofs and transactions:

| Module | Source | Target | Tasks | Proof Modes |
| --- | --- | --- | --- | --- |
| `cosmos_to_eth` | Cosmos SDK | EVM | Parse Cosmos events, build EVM IBC txs | `sp1` — ZK proofs via SP1<br>`attested` — multisig attestations |
| `eth_to_cosmos` | EVM | Cosmos SDK | Parse EVM events, build Cosmos IBC txs | `real` — ethereum mainnet beacon API proofs<br>`mock` — dev/test mode<br>`attested` — multisig attestations |
| `eth_to_eth` | EVM | EVM | Parse EVM events, build attested txs | `attested` — multisig attestations |
| `besu_to_besu` | Besu BFT EVM | Besu BFT EVM | Parse EVM events, build Besu real-proof txs | direct Besu light-client updates + Besu account/storage proofs |
| `cosmos_to_cosmos` | Cosmos SDK | Cosmos SDK | Parse Cosmos events, build Cosmos IBC txs | ICS-07 light client proofs only (validator signatures and merkle proofs) |
| `cosmos_to_solana` | Cosmos SDK | Solana | Parse Cosmos events, build Solana IBC txs | ICS-07 light client proofs only (validator signatures and merkle proofs) |
| `solana_to_cosmos` | Solana | Cosmos SDK | Parse Solana events, build Cosmos IBC txs | multisig attestations only |

## Build and Run

1. Build/install the binary.

   ```sh
   just install-proof-api
   ```

2. Copy the example configuration and update it for your chains.

   ```sh
   cp programs/proof-api/config.example.json config.json
   ```

3. Start the service.

   ```sh
   proof-api start --config config.json
   ```

The gRPC server listens on `server.address:server.port` and the gRPC-web endpoint listens on `server.grpc_web_port`.

## Configuration

Configuration is provided as JSON. See the example in [`programs/proof-api/config.example.json`](./config.example.json).

Key settings:

- `server.address`: IP address to bind the gRPC server.
- `server.port`: gRPC port.
- `server.grpc_web_port`: gRPC-web port.
- `observability.level`: Logging level (`trace`, `debug`, `info`, `warn`, `error`).
- `observability.use_otel`: Enable OpenTelemetry export.
- `observability.service_name`: Service name used in logs/traces.
- `observability.otel_endpoint`: OpenTelemetry collector endpoint.
- `modules`: List of modules to run, each with `name`, `src_chain`, `dst_chain`, `config`, and optional `enabled` (defaults to true). For EVM-to-EVM modules, `Info` returns the routed `src_chain` as the source identity and the destination RPC chain ID as the target identity.

Module configuration varies by module type. Mode-specific options are enumerated below.

Module configuration varies by module type:

- `cosmos_to_eth`:
  - `tm_rpc_url`: Tendermint RPC endpoint for the source chain.
  - `eth_rpc_url`: EVM RPC endpoint for the destination chain.
  - `ics26_address`: IBC router contract address on the destination chain.
  - `mode`: Proof mode object.
  - `mode.type`: `sp1` or `attested`.
  - `mode.sp1_prover`: SP1 prover config when `type: sp1`.
    - `mode.sp1_prover.type`: `mock`, `env`, `network`, `cpu`, `cuda`.
    - `mode.sp1_prover.network_private_key`: Optional hex key for `network`.
    - `mode.sp1_prover.network_rpc_url`: Optional RPC URL for `network`.
    - `mode.sp1_prover.private_cluster`: Optional boolean for `network`.
  - `mode.sp1_programs`: SP1 program paths when `type: sp1`.
    - `mode.sp1_programs.update_client`.
    - `mode.sp1_programs.membership`.
    - `mode.sp1_programs.update_client_and_membership`.
    - `mode.sp1_programs.misbehaviour`.
  - `mode.attestor`: Attestor config when `type: attested`.
    - `mode.attestor.attestor_query_timeout_ms`.
    - `mode.attestor.quorum_threshold`.
    - `mode.attestor.attestor_endpoints`.
  - `mode.cache`: Optional cache config when `type: attested`.
    - `mode.cache.state_cache_max_entries`.
    - `mode.cache.packet_cache_max_entries`.

- `eth_to_cosmos`:
  - `eth_rpc_url`: EVM RPC endpoint for the source chain.
  - `eth_beacon_api_url`: Ethereum beacon API endpoint for consensus data (required for `real`).
  - `tm_rpc_url`: Tendermint RPC endpoint for the destination chain.
  - `ics26_address`: IBC router contract address on the destination chain.
  - `signer_address`: Cosmos address used for message construction metadata.
  - `mode`: Proof mode (string or object).
  - `mode`: `real`, `mock`, or `{ "type": "attested", ... }`.
  - `mode.attestor`: Attestor config when `type: attested`.
    - `mode.attestor.attestor_query_timeout_ms`.
    - `mode.attestor.quorum_threshold`.
    - `mode.attestor.attestor_endpoints`.
  - `mode.cache`: Optional cache config when `type: attested`.
    - `mode.cache.state_cache_max_entries`.
    - `mode.cache.packet_cache_max_entries`.
- `eth_to_cosmos_compat`:
  - Same settings as `eth_to_cosmos`, but routes requests to the v1.2 or current handler based on the client state checksum.

- `eth_to_eth`:
  - `src_rpc_url`: Source EVM RPC endpoint.
  - `src_ics26_address`: Source IBC router contract address.
  - `dst_rpc_url`: Destination EVM RPC endpoint.
  - `dst_ics26_address`: Destination IBC router contract address.
  - `mode`: Proof mode object.
  - `mode.type`: `attested`.
  - `mode.attestor`: Attestor config.
    - `mode.attestor.attestor_query_timeout_ms`.
    - `mode.attestor.quorum_threshold`.
    - `mode.attestor.attestor_endpoints`.
  - `mode.cache`: Optional cache config.
    - `mode.cache.state_cache_max_entries`. (default: 10000)
    - `mode.cache.packet_cache_max_entries`. (default: 10000)
- `besu_to_besu`:
  - `src_rpc_url`: Source Besu RPC endpoint.
  - `src_ics26_address`: Source IBC router contract address.
  - `dst_rpc_url`: Destination Besu RPC endpoint.
  - `dst_ics26_address`: Destination IBC router contract address.
  - `consensus_type`: Source Besu consensus type, `qbft` or `ibft2`.
- `cosmos_to_cosmos`:
  - `src_rpc_url`: RPC endpoint for the source chain.
  - `target_rpc_url`: RPC endpoint for the destination chain.
  - `signer_address`: Cosmos address used for message construction metadata.
- `cosmos_to_solana`:
  - `source_rpc_url`: Tendermint RPC endpoint for the source chain.
  - `target_rpc_url`: Solana RPC endpoint.
  - `solana_ics26_program_id`: Solana ICS26 router program ID.
  - `solana_fee_payer`: Solana fee payer address.
  - `solana_alt_address`: Address lookup table address (optional).
  - `skip_pre_verify_threshold`: Skip pre-verification when signatures are below this threshold. (default: 50)
- `solana_to_cosmos`:
  - `solana_chain_id`: Solana chain ID label.
  - `src_rpc_url`: Solana RPC endpoint.
  - `target_rpc_url`: Tendermint RPC endpoint for the destination chain.
  - `signer_address`: Cosmos address used for message construction metadata.
  - `solana_ics26_program_id`: Solana ICS26 router program ID.

## `besu_to_besu` create-client parameters

`besu_to_besu` `CreateClient` returns only deployment calldata for `BesuQBFTLightClient` or `BesuIBFT2LightClient`. It does not call `ICS26Router.addClient(...)`.

Required parameters:
- `trusting_period`: Decimal seconds for the weak-subjectivity window.
- `max_clock_drift`: Decimal seconds for allowed future header drift.

Optional parameters:
- `trusted_height`: Decimal source block height. Defaults to the latest source block.
- `role_manager`: Hex address. Defaults to the zero address.

Operational notes:
- `consensus_type` is fixed by module config and only affects `CreateClient` wrapper selection.
- `UpdateClient` is a single direct Besu update. No catch-up or intermediate-header search is performed.
- `RelayByTx` uses real Besu proofs: full header RLP, one account proof in the client update, and storage proofs for recv/ack/timeout packet verification.

## Using the gRPC API

After the service is running, use the gRPC endpoints defined in [`proto/proofapi/proofapi.proto`](../../proto/proofapi/proofapi.proto) to submit transaction hashes and retrieve the unsigned IBC transactions plus proofs. Typical usage is:

1. Submit transaction hash(es) to the proof API for the relevant module.
2. Receive the unsigned transaction payload with proof data.
3. Sign and submit the transaction using your own wallet or service.
