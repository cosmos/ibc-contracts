# End to End Testing Suite with Interchaintest

The e2e tests are built using the [interchaintest](https://github.com/cosmos/interchaintest) library. It runs multiple docker container validators, and lets you test IBC enabled smart contracts.

These end to end tests are designed to run in the CI, but you can also run them locally.

## Running the tests locally

### Prerequisites

Besu tests additionally require Docker Compose v2 (`docker compose`).

In the repo root:

```
cp .env.example .env
just install-proof-api && just install-operator
```

### Run tests

To run the tests locally, run the following commands from the repo root:

```shell
# Run the following where `$TEST_NAME` is one of the test names of the `$TEST_SUITE_FN`:
just test-e2e $TEST_SUITE_FN/$TEST_NAME

# For example, to run the `TestDeploy` test, you would run:
just test-e2e TestWithIbcEurekaTestSuite/Test_Deploy

# Alternatively:
just test-e2e-eureka Test_Deploy
```

## Besu QBFT mode

Set `ETH_TESTNET_TYPE=besu-qbft` to start a real 4-validator Besu QBFT network.

### Supported in this mode

This pass supports the following Besu configuration:

- `ETH_TESTNET_TYPE=besu-qbft`
- `ETH_LC_ON_COSMOS=attestor-native`
- `COSMOS_LC_ON_ETH=sp1`
- `SP1_PROVER=mock`

Validated coverage in this mode:

- Besu chain bring-up
- contract deployment
- `eth_getProof` smoke coverage
- Dockerized Ethereum attestors reading Besu RPC
- relayer startup and client creation through `Test_Deploy`
- one-way Ethereum → Cosmos ICS20 transfer, including acknowledgement relay back to Ethereum

### Not supported

- `ETH_LC_ON_COSMOS=full`
- beacon-chain-based Ethereum verification on Cosmos
- full roundtrip Ethereum ↔ Cosmos ICS20 transfer in Besu mode

### Focused local test commands

From the repo root, run:

```shell
# Besu bring-up, deploy, and proof smoke coverage
ETH_TESTNET_TYPE=besu-qbft \
just test-e2e TestBesuQBFTChainBringUpAndDeploy

# Full harness validation in the supported Besu mode
ETH_TESTNET_TYPE=besu-qbft \
ETH_LC_ON_COSMOS=attestor-native \
COSMOS_LC_ON_ETH=sp1 \
SP1_PROVER=mock \
just test-e2e TestWithIbcEurekaTestSuite/Test_Deploy

# Focused one-way Ethereum -> Cosmos transfer in the supported Besu mode
ETH_TESTNET_TYPE=besu-qbft \
ETH_LC_ON_COSMOS=attestor-native \
COSMOS_LC_ON_ETH=sp1 \
SP1_PROVER=mock \
just test-e2e TestWithIbcEurekaTestSuite/Test_ICS20TransferERC20TokenFromEthereumToCosmos
```

## Focused Besu ↔ Besu e2e

This focused suite starts two independent Besu QBFT networks, deploys Eureka contracts on both, starts the Rust relayer with `besu_to_besu` in both directions, deploys and registers Besu light clients on both chains, and verifies a one-way A → B ICS20 transfer plus the B → A acknowledgement relay using real Besu proofs.

### Focused local test commands

From the repo root, run:

```shell
# Single Besu QBFT bring-up and deploy smoke test after helper changes
just test-e2e TestBesuQBFTChainBringUpAndDeploy

# Dual-Besu deploy / client-registration path
just test-e2e TestWithBesuToBesuTestSuite/Test_Deploy

# One-way Besu A -> Besu B ICS20 transfer with acknowledgement relay back to A
just test-e2e TestWithBesuToBesuTestSuite/Test_ICS20TransferERC20FromChainAToChainB

# Regenerate the QBFT light-client fixture used by test/besu-bft/*
GENERATE_BESU_LIGHT_CLIENT_FIXTURES=true \
just test-e2e TestWithBesuToBesuTestSuite/Test_ICS20TransferERC20FromChainAToChainB
```

When `GENERATE_BESU_LIGHT_CLIENT_FIXTURES=true` is set, the focused Besu↔Besu transfer test writes:

- `test/besu-bft/fixtures/qbft.json`
