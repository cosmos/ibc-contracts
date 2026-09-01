# `CosmWasm` ICS-08 Ethereum Light Client

This repository contains a `CosmWasm` implementation of an Ethereum light client, designed for integration with [`ibc-go`](https://github.com/cosmos/ibc-go)'s [`08-wasm`](https://github.com/cosmos/ibc-go/tree/main/modules/light-clients/08-wasm) client wrapper. It manages IBC client and consensus states, delegating all light client-specific logic to the core module in [`packages/ethereum/light-client`](../../../packages/ethereum/light-client).

## Light Client Functions

### Update Client Logic

The `update_client` functionality is based on the [Ethereum Light Client Sync Protocol](https://github.com/ethereum/consensus-specs/blob/dev/specs/altair/light-client/sync-protocol.md), with key modifications to support **historical updates** — a feature necessary for multiple IBC relayers to independently run.

Each update also includes an account proof for the Ethereum `IBCStore.sol` contract, allowing the `CosmWasm` contract to verify the commitment root on Ethereum. This enables IBC clients on Cosmos chains to validate Ethereum-side IBC state using the storage root of `IBCStore`.

### Membership Proofs

We support Merkle-Patricia Trie (MPT) inclusion and exclusion proofs to verify the (non-)membership of IBC commitments in the `IBCStore.sol` contract on Ethereum. These proofs ensure that the IBC handler on Ethereum has committed to the expected state.

### Misbehavior Handling

Misbehavior is defined as the submission of two conflicting—but individually valid—light client updates for the same block height. Upon detecting such a case, the light client will **freeze**, halting further updates. Recovery requires a governance proposal to reset or reinitialize the client.

## Deployment

### Build Requirements

- [Rust](https://rustup.rs/)
- [Just](https://just.systems/man/en/)
- [Protobuf compiler](https://grpc.io/docs/protoc-installation/)
- [Docker](https://docs.docker.com/get-docker/)

We use the [`cosmwasm/optimizer`](https://github.com/CosmWasm/optimizer) Docker image to build the `CosmWasm` contracts. Ensure you have Docker installed and running.

```bash
just solidity::build-cw-ics08-wasm-eth
```

### Storing the Wasm Binary

Once built, the wasm binary needs to be stored on the Cosmos chain. This can be done by following the [IBC-Go documentation](https://ibc.cosmos.network/v10/ibc/light-clients/wasm/governance/#storing-new-wasm-light-client-byte-code) for deploying `CosmWasm` contracts.

### Creating the Light Client

The proof API can be used to get a transaction that creates the light client on the Cosmos chain where the bytecode is stored via `CreateClientRequest`. The `checksum_hex` field must be passed as a parameter in this request, which is the checksum of the wasm bytecode in hex format.


## Acknowledgements

This implementation builds on the Ethereum light client work by [Union Labs](https://github.com/unionlabs/union).
